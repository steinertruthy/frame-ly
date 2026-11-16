#!/usr/bin/env bash
# makensis through Docker (Linux NSIS 3.11), for macOS hosts where Homebrew's makensis crashes
# (bad_alloc on macOS 26). Paths are mounted 1:1 so Tauri's generated installer.nsi works as is.
# Use: put a `makensis` symlink to this script first on PATH before `pnpm build:windows`.
set -euo pipefail
image=framely-makensis
if ! docker image inspect "$image" >/dev/null 2>&1; then
  printf 'FROM debian:trixie-slim\nRUN apt-get update && apt-get install -y --no-install-recommends nsis && rm -rf /var/lib/apt/lists/*\nENTRYPOINT ["makensis"]\n' |
    docker build -q -t "$image" - >/dev/null
fi
exec docker run --rm -v "$HOME:$HOME" -v /tmp:/tmp -v /private:/private -w "$PWD" "$image" "$@"
