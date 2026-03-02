#!/usr/bin/env bash
set -euo pipefail

# Usage:
# npm run build
# npm run release:macos:selfsign
#
# Optional explicit paths:
# ./scripts/release/macos-sign-self.sh "src-tauri/target/release/bundle/macos/Archive Metz.app" "src-tauri/target/release/bundle/dmg/Archive Metz_0.1.0_aarch64.dmg"

APP_PATH="${1:-}"
DMG_PATH="${2:-}"

if [[ -z "${APP_PATH}" ]]; then
  APP_PATH="$(ls -t src-tauri/target/release/bundle/macos/*.app 2>/dev/null | head -n 1 || true)"
fi

if [[ -z "${DMG_PATH}" ]]; then
  DMG_PATH="$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -n 1 || true)"
fi

if [[ -z "${APP_PATH}" && -z "${DMG_PATH}" ]]; then
  echo "No .app or .dmg found. Build first with: npm run build"
  exit 1
fi

if [[ -n "${APP_PATH}" ]]; then
  if [[ ! -d "${APP_PATH}" ]]; then
    echo "App bundle not found: ${APP_PATH}"
    exit 1
  fi

  echo "Ad-hoc signing app bundle: ${APP_PATH}"
  codesign --force --deep --options runtime --sign - "${APP_PATH}"
  codesign --verify --verbose "${APP_PATH}"
fi

if [[ -n "${DMG_PATH}" ]]; then
  if [[ ! -f "${DMG_PATH}" ]]; then
    echo "DMG not found: ${DMG_PATH}"
    exit 1
  fi

  echo "Ad-hoc signing dmg: ${DMG_PATH}"
  codesign --force --sign - "${DMG_PATH}"
  codesign --verify --verbose "${DMG_PATH}"
fi

echo "macOS self-signed flow completed."
