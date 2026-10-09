# Shared native torrent runtime

Linux and Windows desktop bundle the shared Go worker from
`TORRENT_RUNTIME_REF`. The native shell owns its process, private cache namespace,
bounded stdio and cancellation. The canonical viewing client in the `tv` gitlink
owns backend negotiation and player lifecycle through the shared Rust/WASM grant
holder. Hosted browser/TV clients have no native runtime port.

The native worker qualifies before v2 is advertised. Descriptors preserve public
tracker hints, optional file/member selection and exact file-size information.
Grant source data stays outside ordinary UI/history state. Local original media
uses the existing GStreamer/MPV decoders; a native failure never starts gateway
conversion. The v1 server negotiation route remains compatible with older clients.

The default aggregate cache is 2 GiB, reduced by the shared runtime to preserve
free space. Verified pieces survive stop/restart and profile replacement. Account
retirement clears content after the worker has been joined. The worker is
terminated if an operation or joined close cannot settle within its bounded
budget. Generation-prefixed handles fence cleanup from a replacement process.
Frame acknowledgement uses native presented-frame counts. The caller's remaining
120-second startup budget reaches native player opening; Go also enforces the
60-second no-progress limit. Engine acquisition and decoded playback are separate
measurements.

## Pins and checks

- Core: `d1787f3910306822d68192a5d3c45f980a665234`.
- Shared viewing client: `7829a0c712acff4187303c4d425678c94fa59a74`.
- Video adapter: `35eb8612574119d76b1658905ae472ffe48f56ed`.
- Go worker/JNI source: `b636742efe1c89c658eaf5ac9277478fe72b2828`.

`node scripts/torrent-runtime-check.mjs` checks bundled source and byte hashes.
The producer uses `torrent-runtime/scripts/build-native.py` in playback-gateway;
Linux and Windows manifests must identify the same clean source as Android.
The shell test suite passes (13 tests, one existing host-dependent test ignored),
including bounded frames and terminating/reaping a stuck owned process. The
canonical frontend passes 297 tests, type checking and production build. The
native desktop debug binary and its pinned frontend build locally.

Native decoded-source tests, packaged installer execution and Windows-host
playback remain qualification work. These build and unit checks do not establish
general swarm reliability, hardware support or production rollout. Current
startup/seek performance is accepted for integration and tracked in the gateway's
`docs/TORRENT_PERFORMANCE_FOLLOWUP.md`.
