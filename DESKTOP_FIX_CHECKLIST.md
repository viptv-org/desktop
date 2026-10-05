# Desktop fixes checklist

Updated 2026-10-03. Checked means the specific behavior has supporting evidence.
An item stays open when the desktop report still reproduces it or the required
verification is missing. This supersedes earlier broad completion claims.
Work stays solo, uses the local HTTPS backend, and leaves Android unchanged.
Historical qualification status: **31 checked / 3 open**. The current integrated
pins and newly run checks are recorded separately in NATIVE_CONTROLS.md; these
checkboxes are not a new physical-device certification. Evidence boundaries and exact pins are
recorded in NATIVE_CONTROLS.md; checked items apply to the stated scenarios.

## Open: remaining qualification

- [ ] **Sidebar scroll does not flicker or teleport.** Native navigation-rail wheel testing at 960×540 found no scroll range. The reported scrolling region/flicker has not been reproduced; Home wheel measurements do not close this item.

- [ ] **Lord Stream reproduction.** Exact Lanterns S1E1 local discovery had no source match. Rechecking the original authorized Lord endpoint with a bounded diagnostic GET returned HTTP 407 without a Proxy-Authenticate challenge. This does not establish the exact episode failure or its underlying account/proxy cause.

- [ ] **Smooth Home scrolling on native GTK/WebKit.** Actual GTK/WebKit on isolated Xvfb/KWin X11 was measured: 134–136 frame samples, p95 17–18 ms, max 22–32 ms, no frames over 50 ms, with OS wheel input moving Home from 0 to 800 px. The ordinary desktop display returned zero frame samples, so its smoothness remains unverified.

## Implemented and checked

- [x] Volume slider publishes immediately and coalesces obsolete drag commands.
  100 changes produce two native commands; mute/source-switch regressions pass.
- [x] Agent-run native media checks start muted; qualification fixtures do not
  unexpectedly play loud test audio. This does not claim provider loudness
  normalization or change the user's system volume.
- [x] Repeated skip presses accumulate from the latest requested target.
  Regression: 60 → 90 → 120 → 150 despite a transient zero engine clock.
- [x] Backward seeking keeps the target pinned through old/zero clock reports.
  Native adapter and UI regressions cover 27 → 17 seconds.
- [x] Player controls use crisp vector icons and readable skip numbers.
  Actual React controls were visually inspected in the HTTPS browser fixture.
- [x] Unknown track languages use numbered fallback labels; real titles remain.
  Native audio/video labels and title-preservation regression implemented.
- [x] Source rows have a smooth 16 px radius and a 3 px best-match border.
  Browser geometry and private visual inspection verified both.
- [x] Available provider artwork is shown; missing artwork uses a named monogram.
  Loaded-artwork and fallback rows were visually inspected.
- [x] All providers includes observed IPTV and add-on producers.
  Grouping/filter regression and browser picker check pass.
- [x] Click-to-hide is removed from responsive player controls.
  Backdrop clicks reveal controls; popup dismissal remains separate.
- [x] Native desktop app shortcuts, Tab navigation and spatial/arrival focus are disabled.
  Input-policy regression and browser Tab-on-BODY check pass; clicked text edits.
- [x] Desktop source-picker keyboard hints are removed. Browser fixture verified.
- [x] Dismissed repeated playback errors stay dismissed for that attempt.
  Terminal polling stops and retired owners cannot publish delayed errors.
- [x] Pause/resume, seeking, audio/subtitle selection and paused Fit/Fill have
  actual GTK engine checks on both GStreamer and MPV.
- [x] Disguised JPG/GIF/PNG-prefixed HLS has actual-engine regression coverage.
- [x] Local HTTPS backend runs with the current viewing API; health returns 200.

## Verified in the current batch

- [x] **Bottom player bar does not flash or get clipped.** Native GStreamer and MPV picture/timeline/controls were privately inspected during playback and recorded resize through 960×540, 1000×650 and 1280×720. No clipping in 158–159 frame samples per engine; fullscreen bounds also passed. Scope: isolated GTK/X11 display.

- [x] **Backend-unreachable popup does not flash.** Outage dismissal remains separate from connection recovery. Hook regression verifies bounded 10-second probes; HTTPS browser acceptance verifies one notice, persistent dismissal through repeated failures and a new notice after recovery.

- [x] **Selecting a source does not freeze the app.** Actual native HTTP 407 failures kept GTK polling and the React renderer responsive on both engines. Integrated fixture failures produced 5 GStreamer/20 MPV frame opportunities and readable errors. Exact Lanterns/provider qualification remains separate.

- [x] **Controls appear promptly on click.** Trusted OS pointer clicks revealed controls in 14–46 ms during actual muted playback. Failure tests kept rendering active and the error/control surface usable; scope is the tested GTK/X11 build.

- [x] **Playback info names the actual engine.** The rendered info panel names GStreamer and MPV correctly in the native app. Real-backend VOD admission stayed direct on both engines.

- [x] **Duration before the first seek.** Normal local-HTTPS backend admission plus native React controls showed 5317.845 s on GStreamer before any seek despite native live=true, and 5317.802 s on MPV. Selected VOD intent owns display; finite timeline regressions pass.

- [x] **Visible buffer bar before seeking.** Both native engines reported positive real buffer lead and the rendered buffer span was visible before the first seek. Example GStreamer current=1.33/buffer=2.33 s and MPV current=1.25/buffer=21.15 s; no synthetic buffer amounts.

- [x] **Seek-bar click works immediately.** The first trusted OS pointer seek, before any skip, requested 90 s and native telemetry reached about 89.98 s on both engines. This establishes the tested timeline/control path, not universal provider frame accuracy.

- [x] **Readable source errors.** Real HTTP 407/body tests pass for direct and proxied GStreamer/MPV failures. The integrated React player shows HTTP status and safe GStreamer HTTP-source/resource code or MPV loading-failed/-13, plus a bounded redacted fixture excerpt. Diagnostic GET remains distinct from observed playback traffic.

- [x] **Keep the source-provider filter when returning to the open selector.** HTTPS acceptance retains provider and quality through player Back, titlebar Back and browser Back to the same episode; another episode resets to All. Stack/browser snapshots keep filters in memory.

- [x] **Reset on leaving playback.** Native app Back clears duration/position; adapter tests also verify immediate empty time/tracks/error while native close is held. Controller stop now publishes empty immediately and waits for retirement cleanup.

- [x] **Failed new source stays empty.** Both native engines failed explicit VOD replacement against the HTTP 407 fixture with duration=null and position=0. Controller regressions cover refused admission, held retirement, latest replacement and stop cancellation; managed seek/Next rollback stays separate.

- [x] **Fullscreen button fills the window with video.** Both native engines filled the 1440×900 fullscreen viewport, removed titlebar/chrome and restored windowed layout. Integrated video bounds were [0,0,1440,900]. Primary-command ownership, fallback, late completion and pre-existing fullscreen regressions pass.

- [x] **Top-bar double-click enters video fullscreen during playback.** The native titlebar uses the same player fullscreen action on both engines; native chrome removal/restoration and HTTPS browser acceptance pass. Outside playback it keeps maximize behavior.

- [x] **Direct playback stays direct when supported.** Actual local-backend admission reported direct delivery on both engines for the qualified VOD and decoded real frames. No conversion was forced. Other sources still use the backend delivery/capability decision.

## Verification rules

Keep source URLs, credentials, provider headers, tokens and captures outside Git.
Distinguish mocked IPC, browser visuals and actual native playback. Do not
check off native scrolling or every-provider playback from browser/unit tests.
Update this file as each open item is fixed and verified. Detailed historical
evidence and pinned revisions are in NATIVE_CONTROLS.md.
