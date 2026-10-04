# Linux desktop playback controls — 2026-10-03

## 2026-10-04 full application movie and IPTV verification

This follow-up runs the bundled React screens, real `TvApi` discovery and
playback admission, `PlaybackSessionController`, native IPC and rendered GTK
surface together. The controller's backend remains the application's API;
neither playback admission nor decoding is mocked. An opt-in debug-only
`VIPTV_TEST_SCRIPT` file is evaluated after the main page finishes loading so
checks can drive the existing app actions and inspect facts. It is absent from
release builds and inactive in ordinary debug launches.

The original local desktop bundle used `VITE_API_ORIGIN=https://viptv.local.test:8443`.
A rebuild without that environment setting had reverted to the production
default. The full-app check caught HTTP 404 for production `/api/v2/streams`,
`/api/v2/iptv/live/channels` and `/api/v2/iptv/live/:id/source`; production does
not yet serve this client's newer API. The original local HTTPS target is now
restored. This is verification against the configured local backend and real
upstream providers, not a claim of production backend deployment.

`Dune` was discovered in the account catalog and opened from a real IPTV VOD
producer at 30 seconds, on both MPV and GStreamer. Native clock and frames
advanced, and private captures were visually inspected in the actual VIPTV
window. Live `SE: CNN` and `CA CNN` source entries from two configured
connections also played on both engines with direct live delivery, advancing
frames and clock, and visible broadcast pictures. The final healthy-source
full-app run passed all ten checks: one movie and four IPTV source entries on
each engine. Captures contain app chrome plus actual movie/broadcast imagery;
they remain private and outside Git.

The broader IPTV sample intentionally includes unavailable sources. On each
engine, four of ten entries played; four CNN Espanol entries timed out before
the first frame, and two entries returned JSON indicating an expired upstream
account with HTTP 200. Independent authenticated delivery probes confirmed
the four timeout entries supplied no data within twenty seconds and the two
expired entries returned non-media JSON. These sources are still unavailable;
they are not counted as successful player checks or silently replaced.

The first native movie batch had twenty rejected premature MPV resume seeks.
When the diagnostic harness continued despite that command error, six movies
completed decode/control checks; fourteen had source-loading/format or later
control failures. Subsequent tests separated those source failures from the
startup bug. The later playable corpus initially had five GStreamer graphics
failures; all twenty native surfaces passed after the graphics fix.

When rebuilding this local app, preserve the explicit API origin:
`VITE_API_ORIGIN=https://viptv.local.test:8443 npm run build -- --debug --no-bundle`.
The production build default is unchanged. The completed checks retain the
existing user session and restore the original engine preference afterward.

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


## 2026-10-03 desktop timeline, lifecycle and failure qualification

Pins for this batch: TV `47c6b81d5487fdf1cdb02e464a18edba566fde91`,
Video `f22a2a6672c01f55c2c777e27a1428bf5e998d88`, native plugin
`2a282e3461deb37cc7a89f3dc210b8d068a794e0`, design
`a8b5acac2f810e3d447334e64b7fc3f44c696b4d`. Core remains
`f66c87e13a2c93b6dad3234da9694c57f8530b0a`.

The original normal local-HTTPS backend admission was exercised through the
packaged React player and actual GStreamer/MPV IPC. Before any seek, GStreamer
reported duration 5317.845 s, current time 1.33 s, buffer end 2.33 s and
`live=true`; the UI displayed duration and a real buffer span. MPV reported
5317.802 s, current 1.25 s and buffer end 21.15 s with the same visible UI
behavior. The first OS pointer seek requested 90 s, with native telemetry
around 89.98 s; no skip action preceded it. Delivery remained direct. These
facts qualify the tested source/control path, not every provider's frame-accurate
seeking, codec or platform.

Both player-button and titlebar double-click fullscreen removed chrome, gave
native video bounds `[0,0,1440,900]`, and restored the windowed presentation.
The primary host-command path now participates in ownership and releases only
player-owned fullscreen on leaving playback. Hook regressions also cover
fallback, pre-existing fullscreen and late completion after leave/unmount.

Additional native React qualification reused the already-authorized private
VOD delivery with a fixture playback-admission port, and a real loopback HTTP
407 server. Only the admission/release boundary was substituted; native IPC,
network fetching, decoding, DOM controls and OS pointer input remained real.
Do not describe this fixture run as new backend admission or a Lord episode
reproduction. Both engines kept actual buffer display, pointer seeking,
fullscreen and bottom controls intact through 960×540, 1000×650 and 1280×720
resizes, with zero clipping over 158–159 frame samples each. Private captures
and resize recordings were inspected. Trusted pointer control reveal measured
14–46 ms. The 407 replacement showed safe GStreamer HTTP-source/resource facts
or MPV loading-failed/-13 plus HTTP status/body, left duration null/position
zero and maintained 5/20 renderer frame opportunities during failure. Native
surface checks separately covered direct/proxied refusals and valid-source
recovery on both engines, with silent fixtures.

Fast checks: Video typecheck/build and 147 tests; frontend 281 unit tests,
all seven typecheck groups, integrity checks and production build; three HTTPS
browser acceptance scenarios for filter return, fullscreen and outage dismissal/
recovery. Native library: 52 passed, three display-specific tests ignored in
that command; the HTTP refusal and surface-control checks were then run
explicitly on GTK with both engines. Six focused diagnostic tests cover actual
HTTP bodies, timeout, redaction, repeated probes and stale-key reuse. The bus
regression passes independently after explicit GStreamer initialization.

Measurements used actual WebKitGTK/GTK on an isolated Xvfb/KWin X11 display.
Home OS-wheel samples moved 0→800 px, with 134–136 frames, p95 17–18 ms, max
22–32 ms and no frames over 50 ms. This is distinct from Chromium and from the
ordinary desktop display, which returned no frame samples. Its smoothness is
still open. The navigation rail had no overflow at the supported minimum size,
so the reported sidebar scrolling symptom remains open. Exact Lanterns S1E1
had no local source match; the original Lord endpoint diagnostic GET still
returned 407 without a proxy challenge. No account/proxy cause is inferred.

Native diagnostics are gated to Linux/Windows with unchanged-error handling on
unsupported desktop hosts. macOS/Windows compilation and hardware playback
were not executed here. Android code and pins were not changed. Qualification
inputs, credentials and captures remain outside Git. The final executable is a
local Linux debug app built with the Tauri CLI against trusted local HTTPS;
private scratch QA injection is removed before final packaging.


Final packaging checks passed: desktop `npm run check`, `npm run test` and
`tauri build --debug --no-bundle`, all against the exact archived pins and the
shared native target with two build jobs. The normal app was launched with no
private probe/autoplay environment; the binary contains neither private QA
marker. Fresh exact Lord Lanterns S1E1 discovery completed with zero sources and
no episode lease file. The original disposable repro auth session was removed
by its exact ID/hash and its next `/api/auth/me` returned 401. Temporary QA
services stopped; the local HTTPS backend and normal desktop remain running.
