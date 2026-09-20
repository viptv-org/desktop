# VIPTV Desktop Client

The native Tauri v2 application for VIPTV on Linux, Windows, and macOS.

The desktop client provides:
- Native GStreamer / MPV / system video decoding via the sibling [`tauri-video-plugin`](../tauri-video-plugin) repository.
- Frameless custom titlebar with Wayland & X11 edge-resize handlers, minimizing, maximizing, fullscreen toggle, and window dragging.
- The shared viewing UI from [`tv-web`](../tv-web), loaded via Vite during dev (`http://localhost:5173`) and bundled from `tv-web/dist` in release builds.
- Native HTTP networking via `@tauri-apps/plugin-http` with system CA trust integration (`rustls-tls-native-roots`).

## Prerequisites

- **Rust 1.85+** (`rustc --version`, `cargo --version`).
- **Node.js LTS (20+)** and npm.
- **Linux (Debian/Ubuntu)**:
  ```sh
  sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
    gstreamer1.0-dev gstreamer1.0-plugins-{base,good,bad,ugly} libmpv-dev
  ```
- **Windows**: WebView2 and GStreamer runtime (see `../tauri-video-plugin/docs/windows.md`).
- **Workspace layout**: The crate references `../tauri-video-plugin` and `../tv-web` as sibling path dependencies, so this repository builds inside the canonical `viptv-org` workspace.

## Quick Start

From this directory:

```sh
npm install
npm run dev        # Starts tv-web Vite server & launches Tauri desktop window with hot reload
```

## Release Build

```sh
npm run build      # Builds tv-web production bundle and packages the native desktop executable
```

## Rust Checks

```sh
npm run check      # cargo check inside src-tauri
npm run test       # cargo test inside src-tauri
```
