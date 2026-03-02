Param(
  [string]$ExePath = "",
  [string]$Subject = "CN=Archive Metz Dev",
  [string]$PfxPath = "$env:USERPROFILE\archive-metz-dev.pfx",
  [string]$PfxPassword = "ChangeMe123!",
  [string]$TimestampUrl = ""
)

# Usage:
# npm run build
# npm run release:windows:selfsign
#
# Optional:
# pwsh -File .\scripts\release\windows-self-sign.ps1 -PfxPassword "YourPassword123!"

if (-not $ExePath) {
  $candidate = Get-ChildItem "src-tauri/target/release/bundle" -Recurse -Filter "*setup.exe" -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

  if ($candidate) {
    $ExePath = $candidate.FullName
  }
}

if (-not $ExePath -or -not (Test-Path $ExePath)) {
  Write-Error "Installer .exe not found. Build first with npm run build."
  exit 1
}

$securePwd = ConvertTo-SecureString -String $PfxPassword -AsPlainText -Force

if (-not (Test-Path $PfxPath)) {
  Write-Host "Creating self-signed code-signing certificate in CurrentUser\\My: $Subject"
  $cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $Subject `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -KeyExportPolicy Exportable `
    -KeySpec Signature `
    -HashAlgorithm SHA256 `
    -NotAfter (Get-Date).AddYears(3)

  if (-not $cert) {
    Write-Error "Failed to create self-signed certificate."
    exit 1
  }

  Write-Host "Exporting PFX: $PfxPath"
  Export-PfxCertificate -Cert $cert -FilePath $PfxPath -Password $securePwd | Out-Null
}

if (-not (Get-Command signtool -ErrorAction SilentlyContinue)) {
  Write-Error "signtool not found. Install Windows SDK / Signing Tools, then retry."
  exit 1
}

Write-Host "Signing installer: $ExePath"
if ($TimestampUrl) {
  & signtool sign `
    /f $PfxPath `
    /p $PfxPassword `
    /fd SHA256 `
    /tr $TimestampUrl `
    /td SHA256 `
    $ExePath
} else {
  & signtool sign `
    /f $PfxPath `
    /p $PfxPassword `
    /fd SHA256 `
    $ExePath
}

$sig = Get-AuthenticodeSignature $ExePath
Write-Host "Signature status: $($sig.Status)"
if ($sig.Status -eq "NotSigned") {
  Write-Error "File is still not signed."
  exit 1
}

Write-Host "Windows self-signed flow completed."
