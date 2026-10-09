//! Native platform effects for the shared runtime. Private control never enters app state.
use serde_json::{json, Value};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::Manager;
use torrent_runtime_host::{Worker, WorkerError};

#[derive(Default)]
pub struct RuntimeState(Arc<Mutex<State>>);
#[derive(Default)]
struct State {
    worker: Option<Arc<Worker>>,
    generation: u64,
    quarantined: bool,
    retiring: bool,
}
impl RuntimeState {
    pub fn shutdown(&self) -> Result<(), String> {
        settle(&self.0, None)
    }
}
fn executable(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    if !cfg!(any(target_os = "linux", target_os = "windows")) {
        return Err("native_torrent_unsupported".into());
    }
    let name = if cfg!(target_os = "windows") {
        "torrent-worker.exe"
    } else {
        "torrent-worker"
    };
    #[cfg(debug_assertions)]
    {
        let local = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../vendor/torrent-runtime")
            .join(name);
        if local.is_file() {
            return Ok(local);
        }
    }
    app.path()
        .resource_dir()
        .map(|path| path.join("torrent-runtime").join(name))
        .map_err(|_| "torrent_worker_unavailable".into())
}
fn cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_cache_dir()
        .map_err(|_| "native_cache_unavailable")?;
    let path = root.join("torrent-runtime-v2");
    std::fs::create_dir_all(&path).map_err(|_| "native_cache_unavailable")?;
    if path
        .symlink_metadata()
        .map_err(|_| "native_cache_unavailable")?
        .file_type()
        .is_symlink()
    {
        return Err("native_cache_unavailable".into());
    }
    Ok(path)
}
fn initialize(state: &mut State, app: &tauri::AppHandle) -> Result<(), String> {
    if state.quarantined || state.retiring {
        return Err("torrent_worker_unsettled".into());
    }
    if state
        .worker
        .as_ref()
        .is_some_and(|worker| worker.is_alive())
    {
        return Ok(());
    }
    if let Some(previous) = &state.worker {
        if previous.terminate().is_err() {
            state.quarantined = true;
            return Err("torrent_worker_unsettled".into());
        }
    }
    state.worker = Some(Arc::new(
        Worker::start(&executable(app)?, &cache_dir(app)?, 2 << 30)
            .map_err(|error| error.to_string())?,
    ));
    state.generation = state
        .generation
        .checked_add(1)
        .ok_or("torrent_worker_unavailable")?;
    Ok(())
}

#[tauri::command]
pub async fn torrent_runtime_available(
    app: tauri::AppHandle,
    state: tauri::State<'_, RuntimeState>,
) -> Result<bool, String> {
    let state = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Engine loading and cache opening precede native advertisement.
        let mut guard = state.lock().map_err(|_| "torrent_worker_unavailable")?;
        initialize(&mut guard, &app)?;
        elapsed_millis()?;
        Ok(true)
    })
    .await
    .map_err(|_| "torrent_worker_unavailable".to_string())?
}
#[tauri::command]
pub fn torrent_runtime_clock() -> Result<u64, String> {
    elapsed_millis()
}
fn elapsed_millis() -> Result<u64, String> {
    #[cfg(target_os = "linux")]
    {
        let mut time = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        // SAFETY: the initialized timespec is writable; CLOCK_BOOTTIME includes suspension.
        if unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut time) } != 0
            || time.tv_sec < 0
            || time.tv_nsec < 0
        {
            return Err("native_clock_unavailable".into());
        }
        return (time.tv_sec as u64)
            .checked_mul(1000)
            .and_then(|value| value.checked_add(time.tv_nsec as u64 / 1_000_000))
            .ok_or("native_clock_unavailable".into());
    }
    #[cfg(target_os = "windows")]
    {
        #[link(name = "kernel32")]
        extern "system" {
            fn GetTickCount64() -> u64;
        }
        // SAFETY: GetTickCount64 takes no pointers and includes sleep/hibernation.
        return Ok(unsafe { GetTickCount64() });
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    Err("native_torrent_unsupported".into())
}

#[tauri::command]
pub async fn torrent_runtime_call(
    app: tauri::AppHandle,
    state: tauri::State<'_, RuntimeState>,
    request: String,
) -> Result<String, String> {
    let state = state.0.clone();
    tauri::async_runtime::spawn_blocking(move || execute(&app, &state, &request))
        .await
        .map_err(|_| "torrent_worker_unavailable".to_string())?
}
fn settle(state: &Arc<Mutex<State>>, clear: Option<PathBuf>) -> Result<(), String> {
    let worker = {
        let mut guard = state.lock().map_err(|_| "torrent_worker_unavailable")?;
        if guard.retiring {
            return Err("torrent_worker_unsettled".into());
        }
        guard.retiring = true;
        guard.worker.clone()
    };
    let result = worker
        .map_or(Ok(()), |worker| {
            worker.shutdown().map_err(|error| error.to_string())
        })
        .and_then(|()| {
            clear.map_or(Ok(()), |path| {
                std::fs::remove_dir_all(path).map_err(|_| "native_cache_unavailable".to_string())
            })
        });
    let mut guard = state.lock().map_err(|_| "torrent_worker_unavailable")?;
    if result.is_err() {
        guard.quarantined = true;
    } else {
        guard.worker = None;
    }
    guard.retiring = false;
    result
}
fn execute(
    app: &tauri::AppHandle,
    shared: &Arc<Mutex<State>>,
    request: &str,
) -> Result<String, String> {
    if request.len() > 6 * 1024 * 1024 {
        return Err("torrent_worker_protocol_invalid".into());
    }
    let mut command: Value =
        serde_json::from_str(request).map_err(|_| "torrent_worker_protocol_invalid")?;
    let operation = command["op"]
        .as_str()
        .ok_or("torrent_worker_protocol_invalid")?
        .to_owned();
    if !matches!(
        operation.as_str(),
        "prepare" | "observe" | "renew" | "first_frame" | "close" | "shutdown" | "clear"
    ) || command.get("config").is_some()
    {
        return Err("torrent_worker_protocol_invalid".into());
    }
    if matches!(operation.as_str(), "shutdown" | "clear") {
        let clear = if operation == "clear" {
            Some(cache_dir(app)?)
        } else {
            None
        };
        settle(shared, clear)?;
        return Ok(json!({"version":2,"ok":true}).to_string());
    }
    let (worker, generation) = {
        let mut state = shared.lock().map_err(|_| "torrent_worker_unavailable")?;
        if state.retiring || state.quarantined {
            return Err("torrent_worker_unsettled".into());
        }
        if operation == "prepare" {
            initialize(&mut state, app)?;
        } else {
            let id = command["handle"]
                .as_str()
                .ok_or("torrent_worker_protocol_invalid")?;
            let (generation, id) = id
                .split_once(':')
                .ok_or("torrent_worker_protocol_invalid")?;
            if generation.parse::<u64>().ok() != Some(state.generation) {
                return if operation == "close" {
                    Ok(json!({"version":2,"ok":true}).to_string())
                } else {
                    Err("torrent_worker_stale".into())
                };
            }
            command["handle"] = json!(id);
        }
        (
            state.worker.clone().ok_or("torrent_worker_unavailable")?,
            state.generation,
        )
    };
    let budget = if operation == "close" {
        Duration::from_millis(1750)
    } else {
        Duration::from_secs(3)
    };
    let mut reply = match worker.call(command, budget) {
        Ok(reply) => reply,
        Err(error) => {
            if error == WorkerError::Unsettled {
                shared
                    .lock()
                    .map_err(|_| "torrent_worker_unavailable")?
                    .quarantined = true;
            }
            return Err(error.to_string());
        }
    };
    if let Some(handle) = reply["handle"].as_str() {
        reply["handle"] = json!(format!("{generation}:{handle}"));
    }
    Ok(reply.to_string())
}
