#!/usr/bin/env bash
# Builds a debug FrameLy.app signed with "FrameLy Development" (see macos-dev-cert.sh) and opens it.
# Use this — not `tauri dev` — to test permission flows: under `tauri dev` macOS attributes
# Screen Recording/Microphone access to the terminal or IDE that launched the process.
set -euo pipefail
cd "$(dirname "$0")/.."
NAME="FrameLy Development"
if ! security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "Missing signing identity. Run ./scripts/macos-dev-cert.sh first." >&2
  exit 1
fi
APPLE_SIGNING_IDENTITY="$NAME" pnpm tauri build --debug --bundles app
APP="src-tauri/target/debug/bundle/macos/FrameLy.app"
codesign --verify --verbose=1 "$APP"
pkill -x framely 2>/dev/null || true
open "$APP"
