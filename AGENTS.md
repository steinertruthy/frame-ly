# FrameLy — guide for AI agents and contributors

FrameLy is a desktop screen recorder, screenshot tool and AI transcriber for **macOS 14+ (Apple
Silicon)** and **Windows 10 22H2+ (x64)**. Tauri 2 + Svelte 5 (runes) + TypeScript UI, a shared
Rust core, and native media engines: Swift on macOS, Rust + the `windows` crate on Windows.

Read this file first, then only the doc for the area you touch. Docs are in English; the code
comments, commit messages and `CHANGELOG.md` are too. The UI ships in five languages.

## Map

| You are touching… | Read |
|---|---|
| Anything (layers, data flows, where things live) | [docs/architecture.md](docs/architecture.md) |
| Building, running, testing, CLI, env vars | [docs/development.md](docs/development.md) |
| Code style, errors, logs, tests, commits, **step-by-step checklists** | [docs/contributing.md](docs/contributing.md) |
| `native/macos/FrameLyKit`, `src-tauri/src/platform/macos` | [docs/platform-macos.md](docs/platform-macos.md) |
| `src-tauri/src/platform/windows` (capture, WASAPI, Media Foundation) | [docs/platform-windows.md](docs/platform-windows.md) |
| Transcription, summaries, models, providers, consent | [docs/ai.md](docs/ai.md) |
| Svelte windows, components, design tokens, i18n | [docs/ui.md](docs/ui.md) |
| `settings.json`, defaults, validation, migrations | [docs/settings.md](docs/settings.md) |
| Versions, workflows, updater, Windows installer | [docs/release.md](docs/release.md), [docs/signing.md](docs/signing.md) |
| `cloud/` (FrameLy AI Cloud Worker) | [docs/cloud.md](docs/cloud.md) |
| `website/` (Astro, GitHub Pages) | [docs/website.md](docs/website.md) |
| Logs, crashes, known problems | [docs/troubleshooting.md](docs/troubleshooting.md) |
| What is done / known limitations | [docs/roadmap.md](docs/roadmap.md), [CHANGELOG.md](CHANGELOG.md) |

## Commands (run from the repo root unless noted)

The login shell may be fish and may not have Node or cargo on `PATH`. In bash:

```bash
. ~/.nvm/nvm.sh && nvm use 22
export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:/opt/homebrew/opt/llvm/bin:$PATH"
```

| Purpose | Command |
|---|---|
| Install | `pnpm install` |
| Run the app (hot reload) | `pnpm tauri dev` |
| UI type check (fails on warnings) | `pnpm -s exec svelte-check --threshold warning` (or `pnpm check`) |
| UI tests | `pnpm -s exec vitest run` (or `pnpm test`) |
| Rust format | `cd src-tauri && cargo fmt --check` |
| Rust lint (macOS) | `cd src-tauri && cargo clippy --all-targets -- -D warnings` |
| Rust lint (Windows code, from macOS) | `cd src-tauri && cargo clippy --all-targets --target x86_64-pc-windows-gnu -- -D warnings` |
| Rust lint (calibration feature) | `cd src-tauri && cargo clippy --all-targets --target x86_64-pc-windows-gnu --features calibration -- -D warnings` |
| Rust tests | `cd src-tauri && cargo test -q --lib` |
| Swift tests | `cd native/macos/FrameLyKit && swift test` |
| Version files agree | `node scripts/release/check-version.mjs` |
| Cloud / website tests | `cd cloud && pnpm test && pnpm check` · `cd website && pnpm test && pnpm lint` |

Before you commit a code change, all of the lint and test commands for the areas you touched must
pass. The Windows clippy run is the only Windows check possible on a Mac: run it for **any** Rust
change, not only Windows files.

## Rules

1. **Docs follow the code.** Don't document what isn't there; when you change behavior, update the
   doc that describes it in the same commit (and `CHANGELOG.md › [Unreleased]` for user-visible
   changes — see [docs/contributing.md](docs/contributing.md#commits-and-changelog)).
2. **Layers stay separate.** UI → typed IPC wrapper (`src/lib/ipc`) → thin `#[tauri::command]`
   (`src-tauri/src/commands`) → service/`core` → `platform` backend. No business logic in commands;
   no OS calls outside `platform/` (and the few `cfg`-gated helpers next to it).
3. **Errors are `AppError`.** Every fallible Rust path returns `AppResult<T>`; the UI shows
   `error.<code>` from i18n. Never `unwrap()` on anything that depends on the OS, the disk or the
   network.
4. **Every UI string goes through i18n** in all five locales (`pt-BR`, `en`, `es`, `fr`, `de`).
   A missing key fails the type check and a unit test.
5. **The UI only shows what the backend saved.** Settings are changed through `update_settings`
   and adopted from its result.
6. **Privacy is a feature.** Nothing leaves the computer without the user's consent for that
   provider; "Local only" mode blocks every request (`intelligence/net.rs`); no silent fallback
   between providers; API keys only in the Keychain / Credential Manager, never in settings or logs.
   The real AI provider behind FrameLy AI Cloud must **never** be named in the app, its UI strings
   or its API responses.
7. **Never break a recording.** Recordings survive closing the window, updates wait for them, and
   sounds never play while capture is live.
8. **Windows audio is fragile.** Read [docs/platform-windows.md](docs/platform-windows.md#rules-that-must-not-be-broken)
   before touching `wasapi.rs` or `recorder.rs`: the app-audio stream is never released, crash
   markers must stay, loopback conversion is ours, timestamps stay continuous.
9. **Never commit secrets** (updater private key, cloud secrets, certificates). Names of secrets are
   fine; values never.
10. **Shared repo.** Other sessions commit to `main` too: `git pull --rebase` before pushing, commit
    only your files, never force-push, stash, reset or switch branches.

## Never

- Add FFmpeg or another media framework: capture and encoding are native by design.
- Link llama.cpp into the app binary (it conflicts with whisper.cpp's ggml): it lives in the
  `framely-llm` sidecar (`src-tauri/llm-worker`).
- Bypass OS permission prompts, DRM or capture exclusion.
- Overwrite a user's file: the editor saves `name (edited).png`; deletes go to the Trash.
- Overwrite a published release: fix forward with a new version.
