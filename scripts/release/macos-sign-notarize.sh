#!/usr/bin/env bash
set -euo pipefail

# Usage:
# APP_BUNDLE_ID=fr.archive_metz.app \
# APPLE_TEAM_ID=XXXXXXXXXX \
# APPLE_SIGN_IDENTITY="Developer ID Application: Your Company (TEAMID)" \
# APPLE_NOTARY_PROFILE="notary-profile-name" \
# ./scripts/release/macos-sign-notarize.sh

if [[ -z "${APPLE_TEAM_ID:-}" || -z "${APPLE_SIGN_IDENTITY:-}" || -z "${APPLE_NOTARY_PROFILE:-}" || -z "${APP_BUNDLE_ID:-}" ]]; then
  echo "Missing required environment variables."
  echo "Required: APPLE_TEAM_ID, APPLE_SIGN_IDENTITY, APPLE_NOTARY_PROFILE, APP_BUNDLE_ID"
  exit 1
fi

APP_PATH="${1:-src-tauri/target/release/bundle/macos/Archive Metz.app}"
DMG_PATH="${2:-src-tauri/target/release/bundle/dmg/Archive Metz_0.1.0_aarch64.dmg}"

if [[ ! -d "${APP_PATH}" && ! -f "${DMG_PATH}" ]]; then
  echo "No app bundle or dmg found. Build first with: npm run build"
  exit 1
fi

if [[ -d "${APP_PATH}" ]]; then
  echo "Signing app bundle: ${APP_PATH}"
  codesign --force --options runtime --timestamp --sign "${APPLE_SIGN_IDENTITY}" "${APP_PATH}"
  codesign --verify --verbose "${APP_PATH}"
fi

if [[ -f "${DMG_PATH}" ]]; then
  echo "Submitting dmg for notarization: ${DMG_PATH}"
  xcrun notarytool submit "${DMG_PATH}" --keychain-profile "${APPLE_NOTARY_PROFILE}" --wait
  xcrun stapler staple "${DMG_PATH}"
  xcrun stapler validate "${DMG_PATH}"
fi

echo "macOS signing/notarization flow completed."
