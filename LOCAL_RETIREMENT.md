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

No install/launch, real provider playback, physical SmartCast pairing, GPU/4K,
portable clean-machine acceptance or Windows build/dispatch is claimed here.
