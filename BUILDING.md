# Installer builds

`build.yml` builds on main pushes and manual dispatch. Windows x64 produces an
unsigned NSIS EXE; Linux x64 produces DEB and AppImage. Downloads last 30 days
and include SHA256SUMS and build.json. There is no publishing/deployment job.
Artifact builds do not install or launch the VIPTV client and cannot exercise
its default backend origin. Pinned frontend integrity and release native unit
tests remain; runtime SDK installation/probing is not a VIPTV client launch.

The workflow checks out the committed `tv` submodule and `NATIVE_REFS.json`
revisions. Do not replace those pins with branch heads. The local sibling
workspace must contain compatible core and tauri-video-plugin checkouts.

Windows uses scripts/windows-runtime.ps1 to install pinned GStreamer 1.28.2
development files, package its runtime/plugins/licenses beside the executable,
and embed Microsoft WebView2 154.0.4258.37. It verifies the GStreamer checksum
and Microsoft runtime signature. This runtime candidate still needs a clean
Windows machine and actual TextureStream playback qualification.

NSIS uses zlib compression: the default LZMA spent 14 minutes 24 seconds packaging
the bundled runtimes after native compilation had already finished. This trades
some installer size for a shorter package build without omitting runtime files.

Linux DEB declares media dependencies. AppImage includes the media framework;
it still requires a compatible Linux kernel, graphics drivers and glibc (built
on Ubuntu 24.04). Test launch and media on a clean machine; compilation alone
does not prove portable hardware decoding.

Local Linux build: `npm ci && npm ci --prefix tv && npm run test && APPIMAGE_EXTRACT_AND_RUN=1 NO_STRIP=1 npm run build -- --bundles deb,appimage`.
Extraction avoids requiring FUSE during packaging; NO_STRIP avoids the bundled
linuxdeploy binutils rejecting newer ELF RELR sections. Rust still uses its
normal release build profile.
Windows: run the runtime preparation script, then
`npm run build -- --bundles nsis --config src-tauri/tauri.windows.generated.json`.
The preparation script expects RUNNER_TEMP, GITHUB_ENV and GITHUB_PATH as in
Actions. Its generated configuration and runtime files are ignored by Git.

## Manual-only Windows install/loader smoke

The former automatic launch was non-hermetic: the installed production build
may attempt unauthenticated pairing at its default production origin. It is
removed from Actions. The script below is retained only for separately authorized
local Windows qualification in a disposable, appropriately network-controlled
environment. It does not change the packaged origin or prove media/hardware.
Never add this script to the artifact workflow or run it against real account data.

```powershell
$ErrorActionPreference = 'Stop'
$installer = Get-ChildItem src-tauri/target/release/bundle/nsis/*.exe | Select-Object -First 1
if (!$installer) { throw 'No local installer' }
$destination = Join-Path $env:TEMP ('viptv-manual-smoke-' + [guid]::NewGuid())
$setup = Start-Process $installer.FullName -ArgumentList '/S',"/D=$destination" -Wait -PassThru
if ($setup.ExitCode -ne 0) { throw 'NSIS installation failed' }
$env:PATH = ($env:PATH -split ';' | Where-Object { $_ -notlike '*gstreamer*' }) -join ';'
Remove-Item Env:GSTREAMER_1_0_ROOT_MSVC_X86_64 -ErrorAction SilentlyContinue
$app = Start-Process "$destination/viptv-desktop.exe" -PassThru
try {
  Start-Sleep -Seconds 8
  $app.Refresh()
  if ($app.HasExited) { throw "Installed application exited early: $($app.ExitCode)" }
} finally {
  if (!$app.HasExited) { Stop-Process -Id $app.Id }
}
# Retain/review the disposable installation; remove only this exact directory
# under the operator's local cleanup procedure, never a shared/user data path.
```
