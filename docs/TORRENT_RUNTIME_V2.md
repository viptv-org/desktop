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
