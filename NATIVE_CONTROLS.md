# Linux desktop playback controls — 2026-10-03

This adoption pins TV-web `098f4b1364881bff6e7cc496ddefa347bb42127e`, Video
`9429307450cd7e7d856f57f14272acb9e4566113`, native plugin
`c41a5037c664135616b0ea60a2715480e2172df2` and Core
`f66c87e13a2c93b6dad3234da9694c57f8530b0a`. The sibling native source archives were
materialized from the committed pins for the desktop build.

GStreamer engine transitions, seeks, stream selection and timing queries run
on a serialized worker so GTK remains available for input and presentation.
Volume uses an independent audio filter to avoid playbin topology locks.
Confirmed seekable track changes reset the current timeline, aligning new
subtitle branches with video. MPV's redraw callback only sets an atomic flag;
GTK owns render updates and makes its GL context current for destruction.

The shared controller retains polled state and reports volume immediately.
Control refusal does not terminate playback. The desktop rail has a separate
scrolling layer; native controls and global error feedback are protected from
aperture masking, and bare-picture taps reveal controls through the screen.

## Evidence

- Native plugin: 45 ordinary checks pass; two opt-in checks are excluded from
  that count. This includes actual-engine disguised HLS coverage.
- Both actual GTK native engines decode an authorized real-provider VOD,
  seek to 30 seconds, pause/resume, change paused Fit/Fill, keep volume muted,
  and continue playback. The final run took 6.4 seconds.
- A separate silent fixture checks alternate audio, actual rendered subtitle
  pixel changes, subtitle removal, volume and picture controls on both engines.
  The final run took 4.7 seconds. Controls must return within 250 ms.
- Video: 41 focused adapter/compositor checks pass. TV-web: 24 focused UI,
  preference, input and shutdown checks pass. Its production build passes
  strict type checks and design/Core/video integrity.
- Desktop: 11 native shell tests pass; the real LAN probe is excluded.
  Tauri CLI `build --debug --no-bundle` succeeds with the local HTTPS API
  origin, and the embedded entry's assets resolve at the native origin root.
  This is a local Linux debug application, not an installer or deployment.

## Limits

A GStreamer seek into an already-active sparse subtitle cue can lose that cue
until the next cue starts; this remains a known limitation. Provider endpoints
that return proxy errors, time out or return JSON rather than media are not
qualified by the successful VOD check. These results do not establish universal
codec/DRM/HDR support, Windows/macOS behavior or physical TV acceptance.

The owning plugin's README and opt-in `real_provider_native_surface_controls`
test describe reproduction using silent media or private authorized deliveries.
Source URLs, provider headers, account credentials and media captures stay
outside Git.

## Native failures and dismissal follow-up

Generic pipeline, video/audio output, source loading and protected-media errors
are distinguished from explicit decoder/format failures. Unknown pipeline
errors no longer trigger format conversion or reuse the misleading message
from older plugin builds. Startup frame timeouts do not establish a codec fault.

Terminal failure stops polling, layout observation and startup watchdogs. Late
stats cannot revive the failed session. React reports an unresolved error once
per player session/code/message, so dismissal remains effective. A new attempt
or different failure can still report an error; retired playback owners cannot
publish delayed recovery errors into the current UI.

The regression reproduced 13 copies of one error in three seconds before the
fix. It now verifies a single notification, no further polling and successful
polling after a new source opens. The late-stats race and misleading legacy
message also have focused coverage. Nine UI/recovery checks pass, including
dismissal replay and disposal. Both actual GTK engines report a missing source
as a source failure, then decode and control valid media in the same engine
on the local desktop display (4.7 seconds). Real-provider controls also pass
on that display with both engines (8.5 seconds). These results do not claim
that every configured provider is available or that every video now plays.

## Volume slider responsiveness follow-up

Native volume updates publish the latest requested level immediately and allow
only one volume command in flight. Intermediate drag samples are replaced by
the latest value, avoiding a queue of stale audio commands. Acknowledgements
do not trigger duplicate UI publications or overwrite current playback facts.
Mute uses the same ordered path; a retired source cannot block the new slider
or publish its late failure.

41 focused adapter/compositor checks pass. The held-IPC regression changes
volume 100 times: native requests decrease from 100 to two while the requested
level stays immediate. Mute ordering and source replacement also pass. Four
focused React feedback/control checks pass, including 26 slider changes with
pending IPC and immediate input/knob updates. Audio-device latency is not
measured by these boundary tests. Strict type checks, vendor integrity and the
production frontend build pass.
