# viptv desktop

Native Tauri v2 shell for VIPTV on Linux, Windows and macOS. It hosts the shared
viewing client and provides the native capabilities a browser cannot: decoding,
window management, trusted HTTP and SmartCast discovery.

Tickets: [design#3](https://github.com/viptv-org/design/issues/3) (product
parity), this repository's [spec issue](https://github.com/viptv-org/desktop/issues/3)
and [execution ticket](https://github.com/viptv-org/desktop/issues/4).
Product behavior: [PLATFORM_PLAN.md](https://github.com/viptv-org/design/blob/main/PLATFORM_PLAN.md),
the [design system](https://github.com/viptv-org/design/tree/main/viptv-design-system)
(the `Desk*` reference screens) and the VIPTV Redesign canvas that design records
as the visual source.

## Owns

- The Tauri v2 application in `src-tauri/`: native window commands (minimize,
  maximize, close, drag, fullscreen), the debug-only test harness, the
  `tauri-plugin-http` transport with platform trust roots and an explicit
  first-party `Origin`, and LAN SmartCast discovery (`smartcast_discover.rs`).
- Composition of pinned inputs, never their source:
  - UI: the `tv` submodule gitlink to [tv-web](https://github.com/viptv-org/tv-web)
    (Vite dev server in development, `tv/dist` in release builds). Never edit
    files under `tv/`; promote a reviewed tv-web commit by moving the gitlink.
  - Native core bridge: `viptv-core-tauri` from [core](https://github.com/viptv-org/core).
  - Playback: [tauri-video-plugin](https://github.com/viptv-org/tauri-video-plugin)
    (GStreamer / MPV / system decoders).
  - `NATIVE_REFS.json` pins the exact core and plugin commits; CI checks them out.
    Move the gitlink and native pins together, never to unrelated branch tips.
- Packaging (`BUILDING.md`, `.github/workflows/build.yml`): unsigned Windows x64
  NSIS installer, Linux x64 DEB and AppImage, with SHA256SUMS. Builds run on main
  pushes and manual dispatch only; there is no publishing or deployment job.

## Does not own

Viewing screens, focus, copy and playback UX (tv-web), shared state and wire
contracts (core), decoder internals (tauri-video-plugin), accounts and catalog
(backend), or design decisions (design).

## Design pin

This repository has no root `DESIGN_REF`. The effective design revision is the
one pinned by the `tv` submodule: `tv/DESIGN_REF` plus its verified
`tv/design-contract/` snapshot ([DESIGN_SYNC.md](https://github.com/viptv-org/design/blob/main/DESIGN_SYNC.md)).
Adopting a new design revision means adopting a tv-web commit that pins it.
Desktop-only behavior (window chrome, native menus, installers) is specified in
design before implementation, like any other platform exception.

## Platform scope

| Platform | Status |
|---|---|
| Linux x64 (DEB, AppImage) | Built in CI; host `cargo check`/`cargo test` pass. Clean-machine launch and hardware decoding unqualified. |
| Windows x64 (NSIS) | Built in CI with pinned GStreamer and WebView2 runtimes; unsigned. Clean-machine install and actual playback unqualified. |
| macOS | Supported by Tauri and the plugin, but no CI artifact or qualification yet. |

## Acceptance

- `npm run check` and `npm run test` pass; `npm run dev` opens the window against
  the local HTTPS backend.
- Release builds consume only the committed gitlink and `NATIVE_REFS.json`.
- The design scenario matrix (design#3) is recorded per platform with
  functional, visual and installed-device evidence kept separate; a successful
  build is not playback or installer proof.
- Installed authenticated playback, decoder constraints, credential storage and
  code signing are qualified per platform before any parity claim.
- No secrets, private hostnames or captures are committed.
