#!/usr/bin/env bash
# Builds a debug FrameLy.app signed with "FrameLy Development" (see macos-dev-cert.sh) and opens it.
# Use this — not `tauri dev` — to test permission flows: under `tauri dev` macOS attributes
# Screen Recording/Microphone access to the terminal or IDE that launched the process.
set -euo pipefail
cd "$(dirname "$0")/.."

# Works from any shell (fish included): load Node 22 via nvm when pnpm isn't on PATH.
if ! command -v pnpm >/dev/null 2>&1 || ! node -e 'process.exit(+process.versions.node.split(".")[0] < 20)' 2>/dev/null; then
  if [ -s "$HOME/.nvm/nvm.sh" ]; then
    # shellcheck disable=SC1091
    . "$HOME/.nvm/nvm.sh" >/dev/null
    nvm use 22 >/dev/null || { echo "Node 22 not installed: run 'nvm install 22'." >&2; exit 1; }
    corepack enable >/dev/null 2>&1 || true
  fi
fi
command -v pnpm >/dev/null 2>&1 || { echo "pnpm not found. Install Node 22 and run 'corepack enable'." >&2; exit 1; }
command -v cargo >/dev/null 2>&1 || export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
NAME="FrameLy Development"
if ! security find-identity -v -p codesigning | grep -q "\"$NAME\""; then
  echo "Missing signing identity. Run ./scripts/macos-dev-cert.sh first." >&2
  exit 1
fi
APPLE_SIGNING_IDENTITY="$NAME" pnpm tauri build --debug --bundles app
APP="src-tauri/target/debug/bundle/macos/FrameLy.app"
codesign --verify --verbose=1 "$APP"
# Quit the running copy and wait for it to exit (relaunching too early fails with -600).
pkill -x framely 2>/dev/null || true
for _ in $(seq 1 50); do pgrep -x framely >/dev/null || break; sleep 0.1; done
open "$APP"
