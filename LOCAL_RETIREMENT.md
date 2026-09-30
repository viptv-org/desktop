# Desktop BE-002 local-mode cleanup adoption

Desktop baseline: `71d75cfb7ffc7ea5bddfcb6059c1092c21618c2b`.
Sibling native pins and UI gitlink are adopted together:

- Core: `8ae9f81bb753aaf2de53af5b594ead29845e1a8a`.
- Video plugin: `c7e4aa6cb127c69ebac7f2f165cf920c617865a0` (unchanged).
- TV-web: `db9c5ab2da35867e83d970763f53a10628924501`.

Core-Tauri no longer enables the retired anonymous `provider` feature. Its
transitive provider dependency is gone from the desktop lockfile; the standalone
parser crate remains in Core for backend reuse. No desktop native runtime or
SmartCast/media implementation changed. Core's own native/actual-WASM baseline
comparison proves unchanged v2/startup operations and protocol declarations.

Qualification uses a new sibling workspace under
`.worktrees/desktop-local-workspace/{desktop,core,tauri-video-plugin}`. Existing
root, shared worktrees, Android UI/generated files and production are untouched.
The reviewed TV submodule is unmodified; no vendor/generated file was hand-edited.

Run `npm run check`, `npm run test`, strict all-target Clippy and formatting,
then `npm test --prefix tv` and `npm run build --prefix tv`. The retained desktop
suite includes five host tests and one explicitly ignored physical/interactive
test; the frontend suite has 234 tests after retiring local-mode-only wrappers.
Build Linux installers with the documented
`APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 npm run build -- --bundles deb,appimage`.
Build results and artifact checksums are recorded in the review handoff.

2026-09-30 qualification: native check, five host tests (one explicitly ignored),
strict all-target Clippy/formatting, two Core-Tauri SmartCast adapter tests,
234 frontend tests, integrity/typecheck and production frontend build passed.
The workflow-equivalent release/custom-protocol host tests also passed five tests
with the same physical/interactive test ignored.
Linux release packaging produced both DEB and AppImage. Artifacts remain ignored
local build output; they are not installed or uploaded:

| Artifact | SHA-256 |
| --- | --- |
| `VIPTV_0.1.0_amd64.deb` | `ad9b4ce5f16527ec61d6d88171897ddf36dae8c1268090cd5b71868f288e4a08` |
| `VIPTV_0.1.0_amd64.AppImage` | `4534c5dfbaa24d1ff6e75d25f95611edf665140d91c76d81a620dd31d1e9296e` |

The first offline metadata preflight lacked a cached cross-target dependency;
subsequent normal/locked native checks and builds succeeded. `dpkg-deb` is absent
on this host; package metadata inspection uses `ar` and `tar`, not an install.

The local Linux checkpoint above did not install/launch or qualify Windows.
Real provider playback, physical SmartCast pairing, GPU/4K and portable
clean-machine acceptance remain unqualified.

## Hosted installer qualification

Dispatched `build.yml` exactly once on published `refactor/backend-v2` at
2026-09-30 08:34:40 UTC. Run
[36690568896](https://github.com/viptv-org/desktop/actions/runs/36690568896)
is `workflow_dispatch`, with verified head
`54854cf29273ca35a98423d5485153822e35a5d4`. Only this run is monitored;
timeout does not authorize redispatch. Result: success, completed
2026-09-30 08:54:12 UTC. Linux completed 08:46:11 UTC; Windows completed
08:54:11 UTC. Both pinned frontend/build steps, release native tests (five passed,
one explicitly ignored per platform) and artifact uploads passed. Windows install
and eight-second process-survival smoke passed; Linux does not run that smoke.

Downloaded only this run's Windows artifact `11086407339`, named
`desktop-windows-x64-54854cf29273ca35a98423d5485153822e35a5d4`, into the new ignored
folder `artifacts/run-36690568896.1808ls/`. `build.json` independently matches the
exact source revision, Windows x64 and Core/video native pins above. Independent
SHA-256 matches the artifact's `SHA256SUMS`:

`VIPTV_0.1.0_x64-setup.exe` —
`34a4421192a173ee91545aaece14833466b0355e179294cfa2057a19f5efcb76`.

Read-only NSIS archive listing/integrity inspection confirms the desktop binary,
fixed WebView2 executable and GStreamer playback/plugin-scanner resources are
included. It does not execute those resources or prove their media behavior.

The workflow builds Linux DEB/AppImage and Windows NSIS, verifies pinned frontend
integrity, and runs release native tests. Windows preparation verifies GStreamer
1.28.2 against its published SHA-256 and Microsoft's fixed WebView2
154.0.4258.37 executable signature. Its install smoke silently installs the NSIS
artifact into the disposable runner, excludes SDK GStreamer from PATH, then
requires the installed process to survive eight seconds before stopping it.
That is installer/loader survival evidence, not media decoding, rendered UI,
real account/provider integration, physical SmartCast or hardware qualification.
It is also **non-hermetic**: the workflow does not block egress and the packaged
production native origin defaults to `https://viptv.syek.tech`. Unauthenticated
startup/pairing requests may have been attempted; this run is not proof of zero
production-origin contact. No deployment, authenticated account/media test or
production migration was performed by this qualification task.

Future smoke-only proposal, not implemented in this run: keep the shipping build
and its production origin unchanged, then fail-closed block outbound traffic for
the installed native executable and bundled WebView2 executables before launch.
Verify the temporary firewall rules are effective and clean up only those named
rules in `finally`; if protection cannot be verified, refuse the smoke launch.
Do not change `VITE_API_ORIGIN` in shipping artifacts merely to make a smoke test
hermetic. Synthetic backend integration belongs to a separate loopback fixture
test, not an expanded claim from this loader-survival check.
