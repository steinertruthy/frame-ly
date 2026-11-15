#!/usr/bin/env bash
# Builds FrameLy AI's local language-model runtime (src-tauri/llm-worker → `framely-llm`) for a
# target triple and places it where Tauri expects its sidecar:
#   src-tauri/binaries/framely-llm-<target>[.exe]
#
# Usage: scripts/build-llm-worker.sh [target]
# Without an argument it uses TAURI_ENV_TARGET_TRIPLE (set by `tauri dev/build`) or the host.
# Windows targets are cross-compiled with cargo-xwin (see docs/development.md).
set -euo pipefail
cd "$(dirname "$0")/../src-tauri/llm-worker"

target="${1:-${TAURI_ENV_TARGET_TRIPLE:-$(rustc -vV | sed -n 's/^host: //p')}}"
ext=""
case "$target" in
  *windows*)
    ext=".exe"
    # Cross builds can't detect the CPU: target AVX2/FMA/F16C (FrameLy checks AVX2 at runtime).
    export CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="-C target-feature=+avx2,+fma,+f16c"
    if [[ "$(uname)" == "Darwin" ]]; then
      cargo xwin build --release --target "$target"
    else
      cargo build --release --target "$target"
    fi
    ;;
  *)
    cargo build --release --target "$target"
    ;;
esac

mkdir -p ../binaries
cp "target/$target/release/framely-llm$ext" "../binaries/framely-llm-$target$ext"
echo "framely-llm → src-tauri/binaries/framely-llm-$target$ext"
