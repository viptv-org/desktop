# Linux desktop playback controls — 2026-10-03

This adoption pins TV-web `d21b83f73e898f4c00073e0f92b95da3a586a4d8`, Video
`521812a29ff72d4e09ae6ceaf382bc53c5a7bf74`, native plugin
`4a37b03acfead3a99bd474a173808abcfbc7eaf5` and Core
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

## Desktop playback and pointer follow-up

Source replacement retires the outgoing native presentation before backend
preparation and restores its saved position on refusal. GTK hides stale sink
textures and mutes parked audio immediately. Native duration survives temporary
query failures; buffering comes from media-time ranges and AV queue statistics.
Compressed AV queues have a bounded 30-second/32-MiB cache, without growing
decoded-frame queues. Track titles are preserved; unknown languages use
numbered audio/video/subtitle labels.

Repeated skips accumulate from the requested target and coalesce pending work.
Backward seek pins no longer release against the pre-seek clock. Native polls
do not publish transient zero while waiting for the target. Desktop source
rows show available artwork/monograms, include IPTV providers under All, have
a smooth 16px radius and 3px best-match border, and omit keyboard legends.
Native desktop keyboard shortcuts, Tab and spatial/arrival focus are disabled;
clicked text editing remains. Backdrop clicks reveal rather than hide controls.
Player controls use crisp vectors with legible skip numbers.

77 focused controller/adapter/compositor tests, 29 focused UI tests and 45
ordinary native tests pass. Actual GTK GStreamer and MPV seek/pause/resume and
buffered-time readouts pass against an authorized real provider. Silent media
checks track/subtitle pixels and picture modes. The trusted HTTPS browser
fixture was visually inspected for source borders/artwork and player vectors,
time labels and played/buffered layers. Home scroll p95 was 16.8ms across 49
Chromium frames, with one 50.1ms maximum frame. This is scoped browser evidence;
actual GTK/WebKit Home scrolling has not been measured. Offscreen containment
and frame-bounded card/shelf measurement are implemented.

The owner-approved design exception is pinned in TV-web. Captures and private
provider data remain outside Git; Android source and pins are unchanged.
