# Troubleshooting

## Logs

| | macOS | Windows |
|---|---|---|
| Logs | `~/Library/Logs/app.framely.desktop/` | `%LOCALAPPDATA%\app.framely.desktop\logs\` |
| Settings | `~/Library/Application Support/app.framely.desktop/settings.json` | `%APPDATA%\app.framely.desktop\settings.json` |
| Data (library/transcripts SQLite, models) | `~/Library/Application Support/app.framely.desktop/` | `%APPDATA%\app.framely.desktop\` |
| Cache (thumbnails, in-progress recordings, AI temp) | `~/Library/Caches/app.framely.desktop/` | `%LOCALAPPDATA%\app.framely.desktop\` |

- Files `framely.log.<date>`: one JSON object per line, daily rotation, 7 files kept
  (`core/logging.rs`). Each start logs `"FrameLy starting"` with version, OS and architecture.
- Level: `FRAMELY_LOG=debug` (any `tracing` `EnvFilter`, e.g. `FRAMELY_LOG=framely_lib::recording=trace`).
  Debug builds also print readable lines to the terminal.
- UI errors from any window are logged with `"message":"ui"`.
- Useful filters:

  ```bash
  cd ~/Library/Logs/app.framely.desktop
  grep -h '"level":"ERROR"' framely.log.* | tail
  grep -h 'recording:' framely.log.* | tail -40      # audio sources, formats, devices chosen
  grep -h '"message":"ui"' framely.log.* | tail
  ```

- Logs never contain transcript text, prompts or keys; it is safe to ask users for them.

## Crashes

Release builds abort on panic and the log writer is asynchronous, so the panic hook writes
`last-crash.txt` synchronously into the logs folder. On the next start it is logged as
`"the previous session ended with a crash"` (with thread and message) and deleted. Native crashes
(Swift, COM) don't go through the hook: look for the last lines before a missing
`"FrameLy exited"`, and on macOS for a report in Console › Crash Reports.

An interrupted recording is recovered on the next start (`"(recovered)"` in the recordings folder);
an interrupted transcription continues from its next block.

## Known problems

### Development

| Symptom | Cause / fix |
|---|---|
| `cargo: command not found` / `pnpm` missing in a script | Fish login shell; source nvm and add `/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin` to `PATH` ([development.md](development.md#toolchain)). |
| Build stops with `missing binaries/framely-llm-<target>` | Build the sidecar: `pnpm llm:build` (or `scripts/build-llm-worker.sh <target>`); needs `cmake` and `ninja`. |
| Dev window is empty/white | Vite must listen on `127.0.0.1:1420` (not `localhost`). `FRAMELY_DEVTOOLS=1 pnpm tauri dev` opens the inspector. |
| Screenshot of a covered window is blank in a test | WebKit doesn't paint fully covered windows; bring it to front. |
| Permissions behave oddly under `pnpm tauri dev` | macOS attributes them to the terminal/IDE; use `pnpm app:mac` ([platform-macos.md](platform-macos.md#permissions-tcc)). |
| `makensis` aborts with `std::bad_alloc` | Set `LANG`/`LC_ALL=en_US.UTF-8`; on macOS 26 use `scripts/makensis-docker.sh` ([release.md](release.md#windows-installer-nsis)). |
| Windows CI: `cargo test` fails with `STATUS_ENTRYPOINT_NOT_FOUND` | The test binary lacks the Common Controls v6 manifest; CI compiles tests only on Windows (`--no-run`). |
| Every transcription aborts (whisper error -6) | `whisper-rs` abort-callback bug with an unboxed closure — keep the `Box<dyn FnMut>` in `speech.rs`. |

### macOS

| Symptom | Cause / fix |
|---|---|
| Screen Recording granted but capture still fails | The permission applies only after relaunch (Settings offers "Relaunch"). |
| Microphone never prompts, app missing from Privacy › Microphone | Missing `com.apple.security.device.audio-input` entitlement in the signed build. |
| Permissions asked again after an update | Ad-hoc signature changes per version until Developer ID signing ([signing.md](signing.md)). |
| A window isn't in the window list | ScreenCaptureKit lists only windows on the current Space. |
| Keychain password prompt | An item created by another build; FrameLy rewrites items so it owns them (`Keychain.swift`). |

### Windows

#### Windows: system audio missing or silent

1. In the log, find `recording: system audio from every app` (process loopback in use) or
   `app audio capture unavailable; capturing every output instead` (fallback).
2. `app audio: trying another setup` with `… closed FrameLy at step '…'` means a crash marker was
   found: `%TEMP%\framely-app-audio-s-{a,b,c}.attempt`. Each marker disables one setup for good.
   Delete them only to retry deliberately (e.g. after a Windows or driver update); if FrameLy
   closes again, the marker comes back.
3. Fallback with a quiet recording on HDMI/DisplayPort: check the log's
   `software volume … dB, compensated` — compensation is capped at 60 dB.

Rules for changing this code: [platform-windows.md](platform-windows.md#rules-that-must-not-be-broken).

| Symptom | Cause / fix |
|---|---|
| Recording is H.264 though HEVC is selected | "HEVC Video Extensions" not installed (`HEVC encoder unavailable` in the log). |
| Yellow border while recording | Windows 10 capture indicator; absent on Windows 11. |
| `GPU conversion unavailable; converting frames on the CPU` | The D3D11 video processor failed on this machine; recording continues via `core::yuv`. |
| Microphone silent, no prompt | Unpackaged apps get no prompt: enable microphone access for desktop apps in Windows Settings. |
| SmartScreen warning | Installer not Authenticode-signed yet ([signing.md](signing.md)). |
| FrameLy AI Local unavailable (`unsupported_cpu`) | The CPU lacks AVX2. |

### FrameLy AI

| Reason / error | Meaning |
|---|---|
| `local_only` | "Local only" is on: no network at all (Cloud, external providers, downloads, updates). |
| `no_consent` / `consent_required` | The user hasn't agreed to send text to that provider yet. |
| `no_key` / `missing_key` | No API key stored for Anthropic/OpenAI. |
| `model_missing` | The local model isn't downloaded (Settings › Intelligence). |
| `integrity` | Downloaded model failed its SHA-256 check — check or repair it in Settings › Intelligence. |
| `cloud_unavailable` | Build without `FRAMELY_CLOUD_URL`, or the service is down/busy. |
| `cloud_limit` | Daily/per-minute limit; `retryAfter` tells when. |
| `cloud_region` | FrameLy AI Cloud isn't offered in the user's region. |

### Settings

A settings file that can't be read is renamed `settings.corrupt-<timestamp>.json` next to
`settings.json` and defaults are restored (logged as `settings file unreadable; restoring
defaults`). The renamed file can be inspected or fixed and copied back.
