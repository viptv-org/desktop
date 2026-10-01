# VIPTV Desktop Agent Guide

Delivery policy (owner approved 2026-09-27): only Android, desktop, Roku and TV-web
build workflows remain, triggered by main pushes and manual dispatch. No PR
gates, automatic releases, image publishing or deployment. Retain local checks.
This supersedes older automation/release-gate instructions below.

Before changing desktop app behavior, read the design snapshot pinned by the `tv` submodule: `tv/DESIGN_REF` names the imported `viptv-org/design` revision and `tv/design-contract/` holds its design-sync copy (see `tv/design-contract/DESIGN_SYNC.md`). There is no `DESIGN_REF` at the design repository root. This repository owns the native desktop shell (Tauri v2) for Linux, Windows, and macOS.

## Architecture

- **UI Layer**: Provided by the pinned [`tv`](tv) submodule of [`tv-web`](../tv-web). In development, `tauri.conf.json` connects to `http://localhost:5173` with Vite started from the submodule. In production packaging, it bundles `tv/dist`. The gitlink is the promotion step: check out the reviewed tv-web commit inside the submodule and commit the gitlink before building; never hand-edit anything under `tv/`.
- **Media Engine**: Native video decoding is driven by [`tauri-video-plugin`](../tauri-video-plugin) via path dependency `../../tauri-video-plugin`.
- **SmartCast & core bridge**: pairing and native core commands come from the `viptv-core-tauri` crate ([`core/adapters/tauri`](../core/adapters/tauri)) via path dependency; LAN SSDP discovery stays a local module (`smartcast_discover.rs`).
- **Window & System Features**: `src-tauri/src/lib.rs` implements the native window commands (minimize, maximize, close, dragging, fullscreen toggle) and the debug-only test harness (autoplay, `VIPTV_ENGINE` engine override, `test_log`). The frameless titlebar and the Wayland / X11 edge-resize handles are frontend components in the `tv` submodule (`tv/src/ui/DesktopTitlebar.tsx`, `tv/src/ui/WindowResizeBorders.tsx`).

## Validation

- Run `npm run check` (`cargo check --manifest-path src-tauri/Cargo.toml`) and `npm run test` (`cargo test --manifest-path src-tauri/Cargo.toml`) locally.
- When launching the desktop app for testing (`npm run dev`), ensure local HTTPS backend is running (`./.local-https/up.sh start` in the workspace root).
