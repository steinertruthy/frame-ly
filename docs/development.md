# Development

## Toolchain

| Tool | Version / install |
|---|---|
| Rust | stable via rustup (`rust-version = "1.80"` minimum), with `clippy` and `rustfmt` |
| Node | 22 (`.nvmrc`), e.g. `. ~/.nvm/nvm.sh && nvm use 22` |
| pnpm | 10 (`packageManager` in `package.json`; `corepack enable`) |
| macOS | Xcode 15+ (Xcode 26 / macOS 26 SDK to compile Apple Intelligence; without it that code is compiled out) |
| cmake, ninja | `brew install cmake ninja` — build whisper.cpp and llama.cpp |
| Windows lint from macOS | `brew install mingw-w64` and `rustup target add x86_64-pc-windows-gnu` |
| Windows build from macOS | `brew install nsis llvm lld`, `cargo install --locked cargo-xwin`, `rustup target add x86_64-pc-windows-msvc` |
| Windows native dev | Visual Studio Build Tools (C++), WebView2 runtime |

The login shell on the main dev machine is fish, and Homebrew's rustup is not linked into `PATH`.
In a bash script or agent shell, set up the environment first:

```bash
. ~/.nvm/nvm.sh && nvm use 22
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
```

### The `framely-llm` sidecar

FrameLy AI Local's language model runs in a separate binary, `src-tauri/llm-worker` →
`src-tauri/binaries/framely-llm-<target>[.exe]` ([ai.md](ai.md) explains why). `pnpm tauri dev` and
`pnpm tauri build` build it automatically (`beforeDevCommand`/`beforeBuildCommand`); by hand:
`pnpm llm:build` or `scripts/build-llm-worker.sh <target>`. `build.rs` stops with a clear message if
it is missing; for the lint-only `x86_64-pc-windows-gnu` target it writes a placeholder.

## Everyday commands

| Command | What it does |
|---|---|
| `pnpm install` | Install UI dependencies. |
| `pnpm tauri dev` | App with hot reload (UI and Rust). |
| `pnpm check` | `svelte-check --fail-on-warnings` + `tsc` of the Vite config. |
| `pnpm -s exec svelte-check --threshold warning` | Same Svelte check, quieter. |
| `pnpm test` / `pnpm -s exec vitest run` | UI unit tests (`src/**/*.test.ts`). |
| `cd src-tauri && cargo fmt --check` | Rust formatting (`rustfmt.toml`: width 110). |
| `cd src-tauri && cargo clippy --all-targets -- -D warnings` | Rust lint, macOS. |
| `pnpm check:windows` | Rust lint of the Windows code from macOS (`--target x86_64-pc-windows-gnu`). |
| `cd src-tauri && cargo clippy --all-targets --target x86_64-pc-windows-gnu --features calibration -- -D warnings` | Lint with the Windows `calibration` feature (extra encodes per quality, test builds only). |
| `cd src-tauri && cargo test -q --lib` | Rust unit tests (~130; a few are `#[ignore]`d live tests). |
| `cd native/macos/FrameLyKit && swift test` | Swift engine tests (some need a visible window / real-time audio; CI skips those). |
| `node scripts/release/check-version.mjs` | The three version files agree. |
| `pnpm app:mac` | Debug build signed with the local "FrameLy Development" identity, then opens it (needed to test permissions — see below). |
| `pnpm tauri build` | Release bundle for the host (`.app`/`.dmg` or NSIS). |
| `pnpm build:windows` | Windows NSIS installer cross-built from macOS (see below). |

CI (`.github/workflows/ci.yml`) runs on macOS 15 and Windows: `pnpm check`, `pnpm test`, version
check, `swift test` (with skips), the sidecar build, `cargo fmt --check`, clippy, `cargo test --lib`
(compile-only on Windows), and `pnpm tauri build --no-bundle`. It ignores `docs/**`, `website/**`
and `*.md` changes.

## Live and end-to-end tests (ignored by default)

| Command (in `src-tauri/`) | What it does |
|---|---|
| `cargo test -- --ignored live_enumeration` | Lists displays/windows/microphones through the real backend. |
| `cargo test --release -- --ignored --nocapture png_encode_speed` | Times PNG encoding of a 3024×1964 frame. |
| `cargo test -- --ignored round_trip` | Writes and deletes a test key in the Keychain / Credential Manager. |
| `FRAMELY_TEST_SPEECH_MODEL=<ggml file> FRAMELY_TEST_PCM=<16 kHz mono f32 file> cargo test -- --ignored recognizes_real_speech` | whisper.cpp on real audio. |
| `FRAMELY_TEST_MEDIA=<audio> FRAMELY_LLM_WORKER=binaries/framely-llm-<target> cargo test -- --ignored --nocapture end_to_end_with_real_models` | Full FrameLy AI pipeline with real models (downloads ~3.5 GB into FrameLy's data folder). |
| `FRAMELY_CLOUD_URL=http://127.0.0.1:8787 cargo test -- --ignored cloud_round_trip` | Device registration and signed requests against a local Cloud (`cd cloud && pnpm dev:mock`). |
| `FRAMELY_CLOUD_URL=… FRAMELY_TEST_MEDIA=<already transcribed file> cargo test -- --ignored cloud_generation_end_to_end` | Summary/topics through the Cloud via the app's job queue. |
| `FRAMELY_DUMP_SOUNDS=<dir> cargo test sounds` | Writes the synthesized UI sounds as WAV files to listen to. |

## Environment variables

| Variable | When | Effect |
|---|---|---|
| `FRAMELY_LOG` | run time | Log filter (`info` default; e.g. `debug`, `framely_lib=trace`). |
| `FRAMELY_DEVTOOLS=1` | run time, debug | Shows the main window immediately and opens the Web Inspector. |
| `FRAMELY_UPDATE_URL` | run time | Reads another `latest.json` (updater tests). |
| `FRAMELY_CLOUD_URL` | **build** time (also read at run time) | FrameLy AI Cloud base URL embedded in the binary (`FRAMELY_CLOUD_URL=http://127.0.0.1:8787 pnpm tauri dev` for a local Cloud); unset → the Cloud provider is unavailable. In releases it comes from the repo variable `vars.FRAMELY_CLOUD_URL`. |
| `FRAMELY_LLM_WORKER` | tests | Path to the sidecar binary. |

## Command line

With FrameLy running, the executable forwards actions to the running instance (parsed in
`src-tauri/src/lib.rs::cli_action`). Useful for launchers and for driving the app while testing.
On macOS the binary is `FrameLy.app/Contents/MacOS/framely`; on Windows `framely.exe`.

```bash
framely --capture region                 # region | window | screen | cancel
framely --record screen                  # region | window | screen | pause | stop
framely --panel                          # toggle the quick panel
framely --show library                   # home | library | transcripts | settings[:<category>[:<section>]]
framely --show transcript:3:summary      # transcript 3, tab transcript | summary | topics
framely --edit ~/Pictures/FrameLy/shot.png
framely --transcribe ~/Movies/FrameLy/meeting.mp4
framely --update                         # check and install a new version now
framely --background                     # start hidden in the tray (used by the login item)
```

## Debugging tips

- `FRAMELY_LOG=debug pnpm tauri dev` — debug builds also log human-readable lines to stderr.
- The Vite dev server listens on `127.0.0.1:1420` explicitly: with `localhost`, Node may bind only
  `::1` and WebKit shows an empty window.
- WebKit doesn't paint fully covered windows: a screenshot of a window behind others is blank
  even when the UI works.
- Uncaught UI errors land in the Rust log with `"message":"ui"` (see [troubleshooting.md](troubleshooting.md)).

## macOS permissions and the dev signature

macOS ties Screen Recording and Microphone permissions to the app's signature, and under
`pnpm tauri dev` it attributes them to the terminal or IDE that launched the process. To test the
real flow, sign every build with the same local identity:

```bash
./scripts/macos-dev-cert.sh   # once; asks for your password to trust the certificate
pnpm app:mac                  # signed debug build, opens FrameLy.app
```

Remove the identity with `security delete-identity -c "FrameLy Development"`.

## Windows

`pnpm check:windows` compiles the Windows code from macOS against mingw-w64 headers (enough for
clippy and the bundled SQLite). Real Windows builds use MSVC: in CI, or cross-built locally:

```bash
export XWIN_ACCEPT_LICENSE=1 LANG=en_US.UTF-8 LC_ALL=en_US.UTF-8
export PATH="/opt/homebrew/opt/llvm/bin:/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"
pnpm build:windows
```

- `cargo-xwin` downloads the Windows SDK/CRT headers (`XWIN_ACCEPT_LICENSE=1` accepts Microsoft's
  license) and produces `src-tauri/target/x86_64-pc-windows-msvc/release/framely.exe`; NSIS packs the
  installer into `…/release/bundle/nsis/`.
- `LANG`/`LC_ALL` must be set: with an empty locale Homebrew's `makensis` aborts with
  `std::bad_alloc`. On macOS 26 it can crash anyway — use the Docker wrapper
  (`scripts/makensis-docker.sh`, see [release.md](release.md#windows-installer-nsis)).
- whisper.cpp and `framely-llm` are built with AVX2/FMA/F16C; `src-tauri/cmake/ggml-windows.cmake`
  and `src-tauri/.cargo/config.toml` work around `whisper-rs-sys` deciding by the build host instead
  of the target.
- The bare `.exe` also runs (portable; needs the WebView2 runtime). The installer is unsigned
  (SmartScreen warns).
- Test on real Windows hardware (or the Windows 11 ARM VM) for anything touching capture or audio;
  see [platform-windows.md](platform-windows.md).
