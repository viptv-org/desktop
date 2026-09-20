# VIPTV Desktop Agent Guide

Read `DESIGN_REF` in the design repository before changing desktop app behavior. This repository owns the native desktop shell (Tauri v2) for Linux, Windows, and macOS.

## Architecture

- **UI Layer**: Provided by [`tv-web`](../tv-web). In development, `tauri.conf.json` connects to `http://localhost:5173` with Vite started from `tv-web`. In production packaging, it bundles `tv-web/dist`.
- **Media Engine**: Native video decoding is driven by [`tauri-video-plugin`](../tauri-video-plugin) via path dependency `../../tauri-video-plugin`.
- **Window & System Features**: Frameless custom titlebar, Wayland / X11 edge-resize handlers, minimizing, maximizing, fullscreen toggle, dragging, engine switching, and test autoplay harness are implemented in `src-tauri/src/lib.rs`.

## Validation

- Run `npm run check` (`cargo check --manifest-path src-tauri/Cargo.toml`) and `npm run test` (`cargo test --manifest-path src-tauri/Cargo.toml`) locally.
- When launching the desktop app for testing (`npm run dev`), ensure local HTTPS backend is running (`./.local-https/up.sh start` in the workspace root).
