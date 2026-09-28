# REL-001 — desktop promotion, 2026-09-28

The TV-web gitlink promotes the tested error projection, Home layer recovery,
profile-to-Settings navigation and native SmartCast Power/Mute/device-name UI.
NATIVE_REFS.json pins matching shared-core behavior for packaged native builds.

Local cargo check and five native tests pass; the existing hardware/media test
remains ignored. TV-web's 221 tests and HTTPS Chromium fixture checks cover the
UI/native-command boundary, not real SmartCast hardware or installed Windows
playback. Windows/Linux installers are built by the existing Actions workflow;
a dispatched build is not proof of successful installation or deployment.
