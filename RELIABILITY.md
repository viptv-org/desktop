> The first entry below is the current pin record; earlier entries and
> [LOCAL_RETIREMENT.md](LOCAL_RETIREMENT.md) are historical records.

# Packaged frontend startup repair, 2026-10-02

The native package could not finish frontend startup for three independent
reasons: its entry requested hosted `/tv/assets/...` URLs absent from the
embedded root, its CSP prohibited Core WebAssembly compilation, and its
`connect-src` excluded the native origin from which Core fetches its WASM.
The desktop build wrapper now builds with `--base /` and checks the entry's
script/preload files before packaging. The CSP permits `'wasm-unsafe-eval'`
and same-origin fetches. The reviewed TV-web source and gitlink are unchanged.

Diagnosis used shell baseline `04a060919d1d3c50314c78656262a898c9926357`
plus this change, TV-web `e5789ab30361fba5816e0322bf3df98401604d75`, design
`4e153a7daca300389049e5fcfd5c3bc0af5edbee`, and both exact NATIVE_REFS pins.
Vite 6 public-base documentation and Tauri v2 CSP documentation were fetched
through Context7 before configuration changes.

The regression check failed against the original built entry with
`Desktop entry asset is missing from frontendDist: /tv/assets/app-DocpYtAc.js`.
After the root-base change, the same check passed, but native startup displayed
`viptv could not load. Please reload the app.` A trusted local HTTPS replay
using the exact native CSP reproduced a WebAssembly `CompileError`. Adding
the WASM permission alone made that replay reach Sign in; the native app
needed `connect-src 'self'` as well before reaching its account sign-in DOM.
These are frontend boot results, not proof that native pixels render correctly.

The Linux AppImage was freshly rebuilt with two Cargo jobs, extraction enabled
and NO_STRIP. Qualification explicitly compiled
`VITE_API_ORIGIN=https://viptv.local.test:8443`, then launched using disposable
XDG config/data/cache directories. The real backend was rebuilt at clean
`a85df000734c0938d9bab8b19dbac24036878cf7`; it used a newly initialized local
SQLite database, a synthetic account and a loopback-only TLS proxy. Health
returned HTTP 200 with SSL verification result 0. No production data or
production endpoint was used. Nothing was installed or deployed.

| Scope | Functional evidence and limit |
| --- | --- |
| Fresh corrected AppImage startup | Native WebKit accessibility tree reached account sign-in, including Continue in browser and Use another device. The original package had an empty document. |
| Device sign-in | Use another device opened the code/QR state; the fresh local account approved the pairing through the real backend API. Native token polling reached Who's watching. Browser-opener interaction was not exercised. |
| Session persistence and profile selection | Relaunching the corrected AppImage retained the local session. A synthetic profile created through the backend API appeared and could be selected; the native Home navigation/search chrome appeared. The account had no catalog sources. |
| Profile dialogs | Native accessibility activation reached Add a profile and its text-entry dialog. Profile creation/editing by native keyboard or pointer remains unqualified; the accessibility bridge offered no EditableText interface and window-targeted keyboard injection did not enter text. |
| Settings | The same corrected release binary with system GTK/WebKit libraries reached Settings, Appearance and Playback preferences through native accessibility actions, including the device playback-engine row. This additional scope is direct-binary evidence, not AppImage or decoder evidence. |
| Real display/pixels | KDE KWin 6.7.5 runs Wayland. The AppImage GTK hook forces X11 even when `GDK_BACKEND=wayland` is requested. Its owned XWayland window remained unmapped; X11 pixel capture failed. KWin's direct capture API refused an unauthorized caller. Spectacle captures were flat black/gray, and a system-library native Wayland launch was not confirmed active. No rendered screen, pointer/focus, edge resize, fullscreen or window-control claim follows. |

The corrected local-origin AppImage SHA-256 is
`03ee9dd54adde70a69b30f4ba43aaba844d8d9aece026cc18ff21ed6484cf7ef`.
Private package, diagnostic replay, accessibility evidence and captures were
kept outside the repositories. Final `npm run check` and `npm run test` passed
(six native tests, one existing interactive/media test ignored); the frontend
build passed its design/Core/video integrity checks and all TypeScript groups,
and the entry-asset regression check passed. Final AppImage packaging succeeded.

Still open: the complete installed Desk scenario matrix and real-display
rendering/input; authenticated GStreamer/MPV playback, Title/source selection,
Resume/Next, tracks, queue/history and their cancellation/return-focus checks;
clean-machine portability, Windows, macOS and signing. The existing board and
profile-lock gaps below remain. Startup progress does not close desktop#4 or
design#3.

# Remaining desktop board comparison, 2026-10-02

This read-only browser pass used desktop `04a060919d1d3c50314c78656262a898c9926357`,
its exact TV-web gitlink `e5789ab30361fba5816e0322bf3df98401604d75`, and an
export of pinned design `4e153a7daca300389049e5fcfd5c3bc0af5edbee`. The preview
harness mocked API/media boundaries, forced desktop chrome and fixed the clock.
It ran sequentially at 1440×900. Captures and comparison sheets remained private.
No backend account, production data or native media was used.

Of 33 previously unaudited executable Desk scenarios, 32 reached their intended
state and were visually inspected against the committed board. Reaching a state
is not a pixel-parity assertion. DeskPlayerRestore timed out waiting for its
restoration-failure message; its capture and behavior remain unqualified.
DeskStates is a composite board, not an executable scenario. The prior report's
named completed states comprise 13 distinct boards: together these passes have
inspected 45 of the 47 Desk boards, with Restore and States explicitly separate.

| Inspected boards | Observed result and limits |
| --- | --- |
| DeskSignIn, DeskProfilesPaged, DeskProfileEdit, DeskAvatars, DeskProfileDelete | Structure matches. Default avatar, catalog count, pointer/focus and previous-page disabled state differ with fixture/state. The existing missing per-profile lock badge also affects ProfilesPaged and still needs a design/backend decision. |
| DeskLibrary, DeskLibraryCW | Empty and Continue Watching geometry matches; title/art content and hover/focus are fixture states. |
| DeskAddons, DeskAddonInstall, DeskAddonManage, DeskAddonRemove | Group/list/dialog geometry and actions match; focus outlines and background selections differ with pointer/keyboard state. |
| DeskDiscover, DeskDiscoverCatalog, DeskDiscoverFilter | Dropdown/filter popovers match their dedicated boards. The base DeskDiscover board still shows quick catalog tabs while the app uses a catalog dropdown; the dedicated catalog board and components specify the dropdown. Catalog rows and required/optional filters depend on fixture metadata. |
| DeskPlayback, DeskEngine | Engine menu matches. Maximum quality is absent under BACKEND_V2's removal of profile quality caps; the engine section moves upward accordingly. Preferred languages are fixture preferences. |
| DeskPin, DeskSignOut | Dialog structure matches. PIN is exercised through Sign out, so purpose copy differs. Accent colour and OLED background settings follow current component/decision rules, predating these boards. |
| DeskCastSearch, DeskCastManual, DeskCastBusy, DeskCastPin, DeskCastRemote, DeskCastError | Pairing/recovery structures reached with synthetic native-command replies. Search completes immediately; pairing retains Change TV. Remote adds Power/Mute under the existing design decision, making its panel taller. No LAN discovery or TV commands were qualified. |
| DeskItemMenu, DeskHidden | Right-click menu/removal modal and actions reached. Menu position follows the clicked card/pointer; the removed card and visible row differ with fixture queue state. |
| DeskLiveDetails | Modal structure matches; channel counts/US labels/search copy intentionally follow BACKEND_V2 Guide cutover rather than the stale board. |
| DeskPlayerAudio | Track panel structure matches; row highlighting and still-image framing are pointer/media-stub states. No actual track switch/decode. |
| DeskPlayerError | Safe error/retry/source-selection dialog reached. Diagnostic details are collapsed until expanded; the board shows them expanded. Larger source fixture results alter the background list. |
| DeskSources, DeskSourceProvider, DeskSourceDetails | Summary, source rows, provider popover and source-info dialog reached. The left title artwork crops taller than the reference, and the fixture has more rows/qualities; keep this visual discrepancy open for a focused layout review. |

Real KDE Wayland/X11 display endpoints are available on this host. Existing local
AppImages predate the reconciled UI pin or belong to older worktrees, so none was
treated as a proven current installed candidate. The real-display window/chrome
matrix, authenticated native playback, clean-machine checks and signing remain
unverified. Nothing was installed or deployed in this pass.

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
