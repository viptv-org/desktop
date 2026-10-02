//! Last-resort termination is scoped to this app and its own media helpers.
//! A stuck decoder can block GTK itself, so the watchdog uses a separate OS thread.
use std::time::Duration;

pub const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(20);

pub fn start_watchdog() {
    #[cfg(target_os = "linux")]
    let initial_helpers = owned_helpers(std::process::id());
    std::thread::spawn(move || {
        std::thread::sleep(SHUTDOWN_DEADLINE);
        eprintln!("Native app shutdown exceeded its deadline; stopping owned media helpers.");
        #[cfg(target_os = "linux")]
        {
            let mut helpers = initial_helpers;
            helpers.extend(owned_helpers(std::process::id()));
            terminate_owned_helpers(helpers);
        }
        // Native GStreamer/libmpv threads belong to this process. This path
        // deliberately bypasses the blocked UI event loop and destructors.
        #[cfg(target_os = "linux")]
        {
            unsafe extern "C" {
                fn _exit(status: std::os::raw::c_int) -> !;
            }
            // SAFETY: _exit takes only a status and terminates this process.
            // It avoids GTK/C atexit handlers on the watchdog thread after
            // the app's owned media-helper cleanup above has completed.
            unsafe { _exit(0) }
        }
        #[cfg(not(target_os = "linux"))]
        std::process::exit(0);
    });
}

#[cfg(target_os = "linux")]
fn owned_helpers(root: u32) -> Vec<(u32, (u32, u64))> {
    use std::{collections::HashSet, path::PathBuf};
    let mut pending = vec![root];
    let mut seen = HashSet::from([root]);
    let mut helpers = vec![];
    while let Some(parent) = pending.pop() {
        let Ok(tasks) = std::fs::read_dir(format!("/proc/{parent}/task")) else {
            continue;
        };
        for task in tasks.flatten() {
            let Ok(children) = std::fs::read_to_string(task.path().join("children")) else {
                continue;
            };
            for child in children
                .split_whitespace()
                .filter_map(|value| value.parse::<u32>().ok())
            {
                if !seen.insert(child) {
                    continue;
                }
                let base = PathBuf::from(format!("/proc/{child}"));
                let Ok(executable) = std::fs::read_link(base.join("exe")) else {
                    continue;
                };
                let name = executable
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                if media_helper(name) || matches!(name, "bwrap" | "xdg-dbus-proxy") {
                    pending.push(child);
                }
                if media_helper(name) {
                    if let Ok(stat) = std::fs::read_to_string(base.join("stat")) {
                        if let Some(identity) = process_identity(&stat) {
                            helpers.push((child, identity));
                        }
                    }
                }
            }
        }
    }
    helpers
}

#[cfg(target_os = "linux")]
fn terminate_owned_helpers(helpers: Vec<(u32, (u32, u64))>) {
    use std::process::Command;
    for (pid, identity) in helpers {
        // Recheck identity before signaling: a recycled PID has no authority
        // merely because it once belonged to the app's process tree.
        let current = std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|stat| process_identity(&stat));
        if current.is_some_and(|current| current.1 == identity.1) {
            let _ = Command::new("kill")
                .args(["-KILL", &pid.to_string()])
                .status();
        }
    }
}

#[cfg(target_os = "linux")]
fn media_helper(name: &str) -> bool {
    matches!(
        name,
        "WebKitWebProcess"
            | "WebKitNetworkProcess"
            | "WebKitGPUProcess"
            | "gst-plugin-scanner"
            | "ffmpeg"
            | "ffprobe"
            | "mpv"
    )
}

#[cfg(target_os = "linux")]
fn process_identity(stat: &str) -> Option<(u32, u64)> {
    let fields = stat
        .rsplit_once(')')?
        .1
        .split_whitespace()
        .collect::<Vec<_>>();
    Some((fields.get(1)?.parse().ok()?, fields.get(19)?.parse().ok()?))
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    #[test]
    fn fallback_does_not_target_an_external_account_browser() {
        assert!(media_helper("WebKitWebProcess"));
        assert!(media_helper("mpv"));
        assert!(!media_helper("brave"));
        assert!(!media_helper("chrome"));
        assert!(!media_helper("ssh"));
    }
    #[test]
    fn fallback_reaps_owned_probe_and_leaves_non_media_child_alive() {
        use std::process::{Command, Stdio};
        let mut probe = Command::new("ffprobe")
            .args(["-v", "quiet", "-i", "pipe:0"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut unrelated = Command::new("sleep").arg("60").spawn().unwrap();
        let helpers = owned_helpers(std::process::id());
        assert!(helpers.iter().any(|(pid, _)| *pid == probe.id()));
        terminate_owned_helpers(helpers);
        assert!(!probe.wait().unwrap().success());
        assert!(unrelated.try_wait().unwrap().is_none());
        unrelated.kill().unwrap();
        unrelated.wait().unwrap();
    }

    #[test]
    fn process_identity_handles_spaces_and_parentheses_in_process_names() {
        let fields = [
            "S", "42", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
            "0", "0", "1234",
        ];
        assert_eq!(
            process_identity(&format!("7 (name (with spaces)) {}", fields.join(" "))),
            Some((42, 1234))
        );
        assert_eq!(process_identity("7 (unfinished"), None);
    }
}
