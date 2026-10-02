//! Resolve bundled media helpers before GStreamer initializes.
use std::path::{Path, PathBuf};

pub fn configure() {
    let Some(appdir) = std::env::var_os("APPDIR") else {
        return;
    };
    if let Some(scanner) = bundled_scanner(Path::new(&appdir)) {
        std::env::set_var("GST_PLUGIN_SCANNER_1_0", scanner);
    }
}

fn bundled_scanner(appdir: &Path) -> Option<PathBuf> {
    // linuxdeploy copies the helper beside its bundled plugins, while its
    // generated hook still points at a distro-specific nested libexec path.
    let path = appdir.join("usr/lib/gstreamer-1.0/gst-plugin-scanner");
    path.is_file().then_some(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_the_actual_linuxdeploy_scanner_layout() {
        let root =
            std::env::temp_dir().join(format!("viptv-appimage-scanner-{}", std::process::id()));
        let scanner = root.join("usr/lib/gstreamer-1.0/gst-plugin-scanner");
        std::fs::create_dir_all(scanner.parent().unwrap()).unwrap();
        std::fs::write(&scanner, b"fixture scanner").unwrap();
        assert_eq!(bundled_scanner(&root), Some(scanner));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn absent_bundled_scanner_has_no_replacement() {
        let root =
            std::env::temp_dir().join(format!("viptv-no-appimage-scanner-{}", std::process::id()));
        assert_eq!(bundled_scanner(&root), None);
    }
}
