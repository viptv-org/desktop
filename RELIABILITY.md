> Superseded by [LOCAL_RETIREMENT.md](LOCAL_RETIREMENT.md) for the current
> desktop qualification state. The entries below are historical records.

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
