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
60 seconds. MPV seek reliability remains under investigation; it is not counted
as a qualified seek result.

Performance is accepted for this integration phase. Track cold acquisition,
seek delay and MPV seek reliability in the gateway's
`docs/TORRENT_PERFORMANCE_FOLLOWUP.md`; no faster-than-Stremio claim is made.
