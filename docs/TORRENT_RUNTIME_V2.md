# Desktop shared torrent integration

Linux and Windows acquire torrents locally through the pinned Go executable.
The Rust host is imported from the gateway's `torrent-runtime-host` crate;
`RUNTIME_HOST_REF` and its file hashes qualify that source independently from
`TORRENT_RUNTIME_REF`. No local copy of the supervision implementation remains.
Private pipes consume the executable's readiness banner before control replies.
The host owns timeout/termination/reaping, generation-fenced handles and stable
cache storage. Source grants, authorization, selection binding and startup
policy remain in shared core and the pinned viewing client. Native torrent
failures never trigger gateway conversion.

The normal Linux build and 11 Rust tests pass (one LAN fixture remains ignored).
The viewing client passes 297 tests and its production build. Linux/Windows Go
executables were built from one immutable source; Windows import inspection
rejects missing non-system DLLs. Actual Windows native presentation remains
unverified and physical-device/production rollout is separate.

An owned Xvfb application window opened a privately recorded failing H.264
source through the real Rust worker host and native plugin. MPV produced
rendered frames in one cold trial after 18.8 seconds (endpoint 12.9 seconds).
GStreamer reused retained verified cache: endpoint 0.232 seconds, rendered
frames 3.37 seconds. Its 120-second forward seek produced new rendered frames
at the destination after 58.0 seconds. The timer required frames after position
landing; position alone was not accepted. These checks directly exercise worker
and native decoding IPC; backend account/source-screen admission was not part
of this harness. They do not establish public-swarm reliability or comparative
speed. MPV also opened retained cache in 0.411 seconds, but two forward-seek
checks failed to produce the required subsequent rendered frames within
60 seconds. A subsequent MPV trial rendered frames at the requested position after
20.4 seconds. The earlier timeouts remain recorded; one later success does not
establish consistent seek reliability.

Performance is accepted for this integration phase. Track cold acquisition,
seek delay and MPV seek reliability in the gateway's
`docs/TORRENT_PERFORMANCE_FOLLOWUP.md`; no faster-than-Stremio claim is made.

## Installer qualification, 2026-10-09

[Installer run 37987042834](https://github.com/viptv-org/desktop/actions/runs/37987042834)
passed on Windows 2022 and Ubuntu 24.04 at desktop revision `6c01dd6`.
The Windows release native suite passed six tests (one LAN fixture ignored).
The NSIS installer was downloaded, checksum-verified and its torrent-runtime
resources extracted; every worker/dependency notice matched the imported hash
manifest and shared source revision. This establishes packaged bytes, not
Windows TextureStream presentation. A direct worker-artifact test now also
requires startup, definitive input refusal and owned settlement on Linux/Windows.

[Installer run 37996307698](https://github.com/viptv-org/desktop/actions/runs/37996307698)
also passed on both platforms at `649a1d0`. The Windows release suite now includes
actual pinned Go worker startup, invalid-input refusal without losing the warm
worker, and owned shutdown/reaping; seven tests pass, one LAN test is ignored.
This proves Windows runtime loading/control and packaging, not TextureStream
presentation on a physical Windows system.

The worker pin includes durable resolved-file/archive selections with restart
revalidation and shared bounded accounting. The Linux actual-worker control
check passes for this revision. Both OS binaries and all Android ABIs come from
one clean Go source revision; physical Windows presentation remains separate.

[Installer run 38003799432](https://github.com/viptv-org/desktop/actions/runs/38003799432)
passed on Ubuntu 24.04 and Windows 2022 at `59f85e2`, including the durable-selection
worker revision shared with Android and gateway. The Windows release suite passed
seven tests with one LAN fixture ignored; the actual pinned worker loading,
invalid-input refusal and joined shutdown check ran. Linux DEB/AppImage and
Windows NSIS artifacts were uploaded with checksums. These are build/control
results; Windows TextureStream presentation remains unverified.

The current UI pin is `11329bd` with core `0f3d9f7`, matching Android/backend.
Native torrents still acknowledge the local worker from presented-frame facts;
gateway web/Roku acknowledgement stays separate. Local Rust checks/tests and
the native-origin frontend build pass after adoption. The Go worker and host
source revisions are unchanged.

## Native heartbeat correction

Manual desktop playback exposed a control serialization mismatch: native
heartbeats sent `{}`, but the backend's native renewal contract is bodyless.
The backend returned `invalid_playback_request` (HTTP 400) about twenty seconds
after admission, causing playback to stop even without a seek. Recorded starts
were admitted at zero; a saved-position refusal was not the observed cause.

TV-web `11329bd` fixes the private native HTTP adapter to omit the heartbeat body
and content type. Ordinary gateway renewal retains its existing JSON request.
Two regression cases exercise the real TvApi, actual WASM authority and HTTP
serialization for zero-position and resumed playback. Both reproduced the exact
400 before the fix and pass afterward. All 304 frontend tests, the production
build, five trusted-HTTPS UI cases and the real backend native-platform renewal
test pass. The local desktop debug bundle was rebuilt with the explicit local
HTTPS API origin and reopened against the same isolated account data.

The HTTP correction does not qualify the separately recorded slow native seeks
or prove consistent public-swarm seek recovery. Earlier direct worker/decoder
probes bypassed this backend heartbeat boundary; this regression retains it.
