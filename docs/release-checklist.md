# Release Checklist (V1)

## 1. Preconditions
- Node/npm installed (`npm ci` already done).
- Rust toolchain and Tauri prerequisites installed.
- App config paths set in `/settings` and validated.
- `npm run check` and `cargo test` pass.
- `cargo check` and `npm run build` pass.

## 2. Build Unsigned Installers
1. `npm run build`
2. Verify outputs in `src-tauri/target/release/bundle/`:
   - macOS: `.app`, `.dmg`
   - Windows: `.exe` installer (NSIS/MSI depending on target)

## 3. macOS Signing + Notarization
1. Export env vars:
   - `APPLE_TEAM_ID`
   - `APPLE_SIGN_IDENTITY`
   - `APPLE_NOTARY_PROFILE`
   - `APP_BUNDLE_ID`
2. Run:
   - `./scripts/release/macos-sign-notarize.sh`
3. Verify:
   - `codesign --verify --verbose "<app>.app"`
   - `spctl -a -vv "<app>.app"`

## 4. Windows Signing
1. Export env vars:
   - `WINDOWS_CERT_PFX_PATH`
   - `WINDOWS_CERT_PASSWORD`
   - `WINDOWS_TIMESTAMP_URL` (optional)
2. Run:
   - `pwsh -File .\scripts\release\windows-sign.ps1`
3. Verify:
   - `signtool verify /pa /v "<installer>.exe"`

## 4b. Free Self-Signed (Internal Use Only)
- This mode is free and useful for local/internal testing.
- It is not publicly trusted by macOS Gatekeeper / Windows SmartScreen.

1. Build:
   - `npm run build`
2. macOS ad-hoc signing:
   - `npm run release:macos:selfsign`
3. Windows self-signed cert + signing:
   - `npm run release:windows:selfsign`

## 5. Smoke Tests (Manual)
- Install/uninstall on clean machine.
- Validate deposit -> validation -> archive workflow.
- Override workflow logs an audit entry.
- Consultation filters and pagination perform correctly.

## 6. Release Artifacts
- Archive signed installers with build metadata (version, date, commit hash).
- Publish checksum (SHA256) and release notes.
