> The first entry below is the current pin record; earlier entries and
> [LOCAL_RETIREMENT.md](LOCAL_RETIREMENT.md) are historical records.

# Desktop parity pin and Linux package smoke, 2026-09-30

The tv gitlink pins TV-web main `e5789ab` (design `4e153a7`, Core `1f8483e`,
video `514a332`): the responsive player now labels both timeline ends as clocks
per the Desk player boards. NATIVE_REFS.json is unchanged (core `1f8483e`,
tauri-video-plugin `cfdb71d`). Desktop parity comparison and remaining gaps are
tracked in design#3; tv-web TESTING.md holds the browser evidence.

Linux x64, built in a sibling scratch layout at exactly those pins
(`CARGO_BUILD_JOBS=2`, `APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 npm run build --
--bundles deb,appimage`): `npm run check` and `npm run test` passed (6 passed,
the real-device test ignored); `VIPTV_0.1.0_amd64.deb` (12.8 MB) and
`VIPTV_0.1.0_amd64.AppImage` (192 MB) were produced. Nothing was installed.

Install-free AppImage smoke, functional evidence only: the AppImage ran in an
unprivileged network namespace with only a down loopback (no route to any
backend, so no pairing request could leave the host), a throwaway HOME and a
1440x900 Xvfb display. After 30 s the process was still running, its
WebKitWebProcess and WebKitNetworkProcess had started, and a top-level `VIPTV`
window was mapped at 1280x720 (the configured size). The captured window
content was blank white under Xvfb's software path, with and without
`WEBKIT_DISABLE_DMABUF_RENDERER`/`WEBKIT_DISABLE_COMPOSITING_MODE`, so the
rendered pairing screen is **visually unverified**; this is not a launch,
visual or media claim for a real display or clean machine.

Still unverified: Desk* scenario matrix in the installed app against the local
HTTPS backend, authenticated GStreamer/MPV playback, clean-machine DEB/AppImage
launch and hardware decoding, Windows NSIS install/playback, macOS, signing.

# Consumer re-pin, 2026-09-30

The tv gitlink pins TV-web main `4ffc748` (Core `1f8483e`, video `514a332`).
NATIVE_REFS.json pins core `1f8483e` and tauri-video-plugin `cfdb71d`, which
adds the loopback HLS sanitizing proxy (protocol stays 1). The engine picker
offers Auto plus only the engines `native_diagnostics` reports and migrates a
stored engine this build lacks to Auto. The lockfile adds the plugin's proxy
dependencies (`httpdate` is new; hyper, reqwest 0.12 and tokio were already present).

Checks ran in a sibling scratch layout with core and the plugin at those
exact commits: cargo fmt, clippy `-D warnings`, `npm run check` and
`npm test` (6 passed, the real-device test ignored). TV-web passed 242 unit
tests, the build and targeted single-worker browser specs. No installed desktop,
Windows or real-media playback was exercised and nothing was deployed.

# BE-002 — raw live transport preparation, 2026-09-29

The tv gitlink pins TV-web bf92331 and NATIVE_REFS.json pins matching core
b75393e. Raw default/override cursor pages, exact live source and v2 guide API
methods are available without adding UI or indexing a playlist. TV-web passed
250 unit tests/build/typechecks and 32 trusted-HTTPS browser regressions.
Ordinary Guide/Home live callers remain legacy until their cursor/lease cutover.
The app is not deployed and no Windows/physical media claim is made.
Linux cargo check and five shell tests passed; the existing real-device/media
test remains ignored. This does not qualify an installed desktop surface.

# BE-002 — v2 native playback promotion, 2026-09-29

The tv gitlink pins TV-web 6ede3ca with active v2 VOD discovery/admission,
lease renewal/release and foreground authorization checks. NATIVE_REFS.json
pins core 4418f1d and native video c7e4aa6. Required original-source headers
reach the native transport; typed engine errors use safe messages.

Local Linux cargo check and five shell tests passed (one existing real-device
test ignored). TV-web passed 244 unit tests/build/typechecks, 32 trusted-HTTPS
browser cases and SolidTV Home/player acceptance simulations. Native plugin
passed 13 Linux tests, including real header-required HTTP MP4 decode/seek
and a safe typed HTTP401 refusal. This is not installed-app surface, Windows,
4K/HDR/DRM or physical-device qualification. Live remains legacy pending raw
catalog migration. No installer publication or deployment occurred.

# REL-001 — desktop promotion, 2026-09-28

The TV-web gitlink promotes the tested error projection, Home layer recovery,
profile-to-Settings navigation and native SmartCast Power/Mute/device-name UI.
NATIVE_REFS.json pins matching shared-core behavior for packaged native builds.

Local cargo check and five native tests pass; the existing hardware/media test
remains ignored. TV-web's 221 tests and HTTPS Chromium fixture checks cover the
UI/native-command boundary, not real SmartCast hardware or installed Windows
playback. Windows/Linux installers are built by the existing Actions workflow;
a dispatched build is not proof of successful installation or deployment.
