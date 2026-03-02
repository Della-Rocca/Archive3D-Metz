Param(
  [string]$ExePath = "src-tauri/target/release/bundle/nsis/Archive Metz_0.1.0_x64-setup.exe"
)

# Usage:
# $env:WINDOWS_CERT_PFX_PATH="C:\secrets\codesign.pfx"
# $env:WINDOWS_CERT_PASSWORD="***"
# $env:WINDOWS_TIMESTAMP_URL="http://timestamp.digicert.com"
# pwsh -File .\scripts\release\windows-sign.ps1

if (-not $env:WINDOWS_CERT_PFX_PATH -or -not $env:WINDOWS_CERT_PASSWORD) {
  Write-Error "Missing env vars WINDOWS_CERT_PFX_PATH or WINDOWS_CERT_PASSWORD"
  exit 1
}

if (-not (Test-Path $ExePath)) {
  Write-Error "Installer not found: $ExePath. Build first with npm run build."
  exit 1
}

$timestampUrl = $env:WINDOWS_TIMESTAMP_URL
if (-not $timestampUrl) {
  $timestampUrl = "http://timestamp.digicert.com"
}

Write-Host "Signing installer: $ExePath"
& signtool sign `
  /f $env:WINDOWS_CERT_PFX_PATH `
  /p $env:WINDOWS_CERT_PASSWORD `
  /fd SHA256 `
  /tr $timestampUrl `
  /td SHA256 `
  $ExePath

& signtool verify /pa /v $ExePath
Write-Host "Windows signing flow completed."
