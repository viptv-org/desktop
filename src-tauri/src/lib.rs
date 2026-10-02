//! The VIPTV desktop shell: the shared tv-web UI in a Tauri webview with the
//! native video engine attached. The DOM owns every visible control and
//! overlay; `tauri-plugin-video` owns only the native playback surface, and
//! the HTTP and opener plugins carry the backend transport and external
//! sign-in links. SmartCast TV pairing and LAN discovery stay native behind
//! the `smartcast_*` commands from the `viptv-core-tauri` integration crate:
//! TV credentials live in the OS keyring, never in the renderer or the
//! Tauri store. All behavior beyond this registration lives in the shared
//! frontend (`../tv/src`).

mod shutdown;
mod smartcast_discover;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};
use tauri_plugin_video::VideoExt;
use viptv_core_tauri::smartcast;

#[derive(Default)]
struct ShutdownState {
    started: AtomicBool,
    renderer_ready: tokio::sync::Notify,
}

fn begin_shutdown(app: tauri::AppHandle) {
    if app
        .state::<ShutdownState>()
        .started
        .swap(true, Ordering::AcqRel)
    {
        return;
    }
    shutdown::start_watchdog();
    let _ = app.emit("app-shutdown-requested", ());
    tauri::async_runtime::spawn(async move {
        let _ = tokio::time::timeout(
            std::time::Duration::from_secs(3),
            app.state::<ShutdownState>().renderer_ready.notified(),
        )
        .await;
        tauri::async_runtime::spawn_blocking(move || {
            if app.video().shutdown_native().is_err() {
                eprintln!("Native playback cleanup could not finish during shutdown.");
            }
            app.exit(0);
        });
    });
}

#[tauri::command]
fn app_shutdown_ready(state: tauri::State<'_, ShutdownState>) {
    if state.started.load(Ordering::Acquire) {
        state.renderer_ready.notify_one();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    if let Ok(executable) = std::env::current_exe() {
        if let Some(directory) = executable.parent() {
            let plugins = directory.join("gstreamer-plugins");
            if plugins.is_dir() {
                std::env::set_var("GST_PLUGIN_SYSTEM_PATH_1_0", plugins);
                std::env::set_var(
                    "GST_PLUGIN_SCANNER_1_0",
                    directory.join("gstreamer-libexec/gstreamer-1.0/gst-plugin-scanner.exe"),
                );
            }
        }
    }
    tauri::Builder::default()
        .manage(ShutdownState::default())
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    begin_shutdown(window.app_handle().clone());
                }
                tauri::WindowEvent::Destroyed => begin_shutdown(window.app_handle().clone()),
                _ => {}
            }
        })
        .plugin(tauri_plugin_video::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_opener::init())
        // One app-wide SmartCast pairing session; the smartcast_* commands
        // serialize on it while discovery stays lock-free.
        .manage(smartcast::SmartCastState::default())
        .invoke_handler(tauri::generate_handler![
            test_autoplay_enabled,
            playback_engine_override,
            test_log,
            app_window_minimize,
            app_window_toggle_maximize,
            app_window_close,
            app_shutdown_ready,
            app_window_start_dragging,
            app_window_toggle_fullscreen,
            smartcast::smartcast_configure,
            smartcast::smartcast_run,
            smartcast::smartcast_cancel,
            smartcast::smartcast_forget,
            smartcast_discover::smartcast_discover
        ])
        .run(tauri::generate_context!())
        .expect("error while running the VIPTV desktop app");
}

#[tauri::command]
fn app_window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
fn app_window_toggle_maximize(window: tauri::Window) -> Result<(), String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())
    } else {
        window.maximize().map_err(|e| e.to_string())
    }
}

#[tauri::command]
fn app_window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

#[tauri::command]
fn app_window_start_dragging(window: tauri::Window) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

#[tauri::command]
fn app_window_toggle_fullscreen(window: tauri::Window) -> Result<bool, String> {
    let current = window.is_fullscreen().map_err(|e| e.to_string())?;
    window.set_fullscreen(!current).map_err(|e| e.to_string())?;
    Ok(!current)
}

/// The developer test harness (autoplay, engine override, stdout log) only
/// runs in debug builds. Release builds keep the three commands registered
/// so the shared frontend's unconditional `test_log` calls still resolve,
/// but they ignore the environment and print nothing.
const HARNESS_ENABLED: bool = cfg!(debug_assertions);

/// Harness switch: `VIPTV_TEST_AUTOPLAY=1` (or `true`) makes the webview
/// autoplay the first playable title and mirror player snapshots to stdout,
/// so a shell can watch real playback without driving the UI. Always off in
/// release builds.
#[tauri::command]
fn test_autoplay_enabled() -> bool {
    HARNESS_ENABLED && autoplay_test_mode(std::env::var("VIPTV_TEST_AUTOPLAY").ok().as_deref())
}

/// `VIPTV_ENGINE=mpv|gstreamer|auto` overrides the persisted engine choice
/// for this launch without rewriting it. Debug builds only.
#[tauri::command]
fn playback_engine_override() -> Option<String> {
    if !HARNESS_ENABLED {
        return None;
    }
    std::env::var("VIPTV_ENGINE")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// The harness's stdout channel; the shell prefixes every line so the
/// output stays greppable regardless of webview logging. A no-op in
/// release builds.
#[tauri::command]
fn test_log(message: String) {
    if HARNESS_ENABLED {
        println!("[test] {message}");
    }
}

fn autoplay_test_mode(value: Option<&str>) -> bool {
    matches!(value.map(str::trim), Some("1" | "true"))
}

#[cfg(test)]
mod tests {
    use super::autoplay_test_mode;

    #[test]
    fn autoplay_mode_accepts_only_one_or_true() {
        assert!(autoplay_test_mode(Some("1")));
        assert!(autoplay_test_mode(Some("true")));
        assert!(autoplay_test_mode(Some(" 1 ")));
        assert!(!autoplay_test_mode(None));
        assert!(!autoplay_test_mode(Some("")));
        assert!(!autoplay_test_mode(Some("0")));
        assert!(!autoplay_test_mode(Some("yes")));
    }
}
