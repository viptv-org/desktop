# Linux desktop playback controls — 2026-10-03

This adoption pins TV-web `31a9943dc893cb13bef6beced0afef46312ec17f`, Video
`e30f6af29ce68aafa2bc1a8a5f2d5075803cb558`, native plugin
`4f1fd390da8db9ebde6634a55f98ba38f666cd96` and Core
`f66c87e13a2c93b6dad3234da9694c57f8530b0a`. The sibling native source archives were
compared byte-for-byte with the committed pins before building.

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

- Native plugin: 43 ordinary checks pass; two opt-in checks are excluded from
  that count. This includes actual-engine disguised HLS coverage.
- Both actual GTK native engines decode an authorized real-provider VOD,
  seek to 30 seconds, pause/resume, change paused Fit/Fill, keep volume muted,
  and continue playback. The final run took 6.4 seconds.
- A separate silent fixture checks alternate audio, actual rendered subtitle
  pixel changes, subtitle removal, volume and picture controls on both engines.
  The final run took 4.7 seconds. Controls must return within 250 ms.
- Video: 35 focused adapter/compositor checks pass. TV-web: 24 focused UI,
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
