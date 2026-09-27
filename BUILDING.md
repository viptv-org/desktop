# Installer builds

`build.yml` builds on main pushes and manual dispatch. Windows x64 produces an
unsigned NSIS EXE; Linux x64 produces DEB and AppImage. Downloads last 30 days
and include SHA256SUMS and build.json. There is no publishing/deployment job.

The workflow checks out the committed `tv` submodule and `NATIVE_REFS.json`
revisions. Do not replace those pins with branch heads. The local sibling
workspace must contain compatible core and tauri-video-plugin checkouts.

Windows uses scripts/windows-runtime.ps1 to install pinned GStreamer 1.28.2
development files, package its runtime/plugins/licenses beside the executable,
and embed Microsoft WebView2 154.0.4258.37. It verifies the GStreamer checksum
and Microsoft runtime signature. This runtime candidate still needs a clean
Windows machine and actual TextureStream playback qualification.

Linux DEB declares media dependencies. AppImage includes the media framework;
it still requires a compatible Linux kernel, graphics drivers and glibc (built
on Ubuntu 24.04). Test launch and media on a clean machine; compilation alone
does not prove portable hardware decoding.

Local Linux build: `npm ci && npm ci --prefix tv && npm run test && npm run build -- --bundles deb,appimage`.
Windows: run the runtime preparation script, then
`npm run build -- --bundles nsis --config src-tauri/tauri.windows.generated.json`.
The preparation script expects RUNNER_TEMP, GITHUB_ENV and GITHUB_PATH as in
Actions. Its generated configuration and runtime files are ignored by Git.
