$ErrorActionPreference = 'Stop'
# Pin both redistributables; never depend on the build runner's playback installation.
$gstVersion = '1.28.2'
$gstUrl = "https://gstreamer.freedesktop.org/data/pkg/windows/$gstVersion/msvc/gstreamer-1.0-msvc-x86_64-$gstVersion.exe"
$installer = Join-Path $env:RUNNER_TEMP 'gstreamer.exe'
Invoke-WebRequest $gstUrl -OutFile $installer
$checksum = Join-Path $env:RUNNER_TEMP 'gstreamer.sha256sum'
Invoke-WebRequest "$gstUrl.sha256sum" -OutFile $checksum
$expected = ((Get-Content $checksum -Raw).Trim() -split '\s+')[0]
if ($expected -notmatch '^[0-9a-fA-F]{64}$') { throw 'Invalid GStreamer checksum document' }
if ((Get-FileHash $installer -Algorithm SHA256).Hash -ne $expected) { throw 'GStreamer checksum mismatch' }
$gst = Join-Path $env:RUNNER_TEMP 'gstreamer'
$process = Start-Process $installer -ArgumentList '/VERYSILENT','/SUPPRESSMSGBOXES','/NORESTART','/TYPE=devel',"/DIR=$gst" -Wait -PassThru
if ($process.ExitCode -ne 0) { throw 'GStreamer installation failed' }
$env:GSTREAMER_1_0_ROOT_MSVC_X86_64 = $gst
"GSTREAMER_1_0_ROOT_MSVC_X86_64=$gst" | Out-File -Append $env:GITHUB_ENV
"PKG_CONFIG_PATH=$gst\lib\pkgconfig" | Out-File -Append $env:GITHUB_ENV
"$gst\bin" | Out-File -Append $env:GITHUB_PATH
$env:PATH = "$gst\bin;$env:PATH"
& "$gst\bin\gst-inspect-1.0.exe" playbin3
if ($LASTEXITCODE -ne 0) { throw 'GStreamer playbin3 missing' }

$runtime = New-Item -ItemType Directory -Force 'src-tauri/runtime'
Copy-Item "$gst/bin" "$runtime/gstreamer-bin" -Recurse -Force
New-Item -ItemType Directory -Force "$runtime/gstreamer-lib" | Out-Null
Copy-Item "$gst/lib/gstreamer-1.0" "$runtime/gstreamer-lib/plugins" -Recurse -Force
Copy-Item "$gst/libexec" "$runtime/gstreamer-libexec" -Recurse -Force
Copy-Item "$gst/share/licenses" "$runtime/licenses" -Recurse -Force

$webviewVersion = '154.0.4258.37'
$webviewUrl = 'https://msedge.sf.dl.delivery.mp.microsoft.com/filestreamingservice/files/b82d47e8-d146-4563-94d1-3a3176b25c0a/Microsoft.WebView2.FixedVersionRuntime.154.0.4258.37.x64.cab'
$cab = Join-Path $env:RUNNER_TEMP 'webview2.cab'
Invoke-WebRequest $webviewUrl -OutFile $cab
& expand.exe $cab '-F:*' $runtime.FullName
if ($LASTEXITCODE -ne 0) { throw 'WebView2 extraction failed' }
$browser = Join-Path $runtime.FullName "Microsoft.WebView2.FixedVersionRuntime.$webviewVersion.x64"
if (!(Test-Path "$browser/msedgewebview2.exe")) { throw 'Fixed WebView2 runtime missing' }
$signature = Get-AuthenticodeSignature "$browser/msedgewebview2.exe"
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'Microsoft Corporation') { throw 'Invalid Microsoft runtime signature' }
$config = @{
  bundle = @{
    resources = @{
      'runtime/gstreamer-bin/' = './'
      'runtime/gstreamer-lib/plugins/' = 'gstreamer-plugins/'
      'runtime/gstreamer-libexec/' = 'gstreamer-libexec/'
      'runtime/licenses/' = 'licenses/gstreamer/'
    }
    windows = @{ webviewInstallMode = @{ type = 'fixedRuntime'; path = "runtime/Microsoft.WebView2.FixedVersionRuntime.$webviewVersion.x64" } }
  }
}
$config | ConvertTo-Json -Depth 8 | Set-Content 'src-tauri/tauri.windows.generated.json'
