# Contributing

How code is written here, and step-by-step checklists for the usual changes. Architecture and
locations: [architecture.md](architecture.md). Commands: [development.md](development.md).

## General principles

- **Read before writing.** Find the existing pattern for what you are adding (there almost always
  is one) and follow it. Prefer extending a module to creating a parallel one.
- **Small, real, verified.** Don't add settings, abstractions or code paths "for later". A setting
  exists only when the feature that reads it exists.
- **Pure logic goes to `core/` (Rust) or a `.ts` helper (UI) and gets unit tests.** OS calls,
  windows and I/O stay thin around it.
- **Comments explain *why*,** especially for workarounds (OS bugs, measured numbers, crashes seen on
  real hardware). Every module starts with a `//!` doc comment saying what it owns. Keep them true
  when you change behavior.
- **English** for code, comments, docs, commits and the changelog. User-visible text goes through
  i18n in five languages.

## Rust

- Edition 2021, `rustfmt.toml` (max width 110, `use_small_heuristics = "Max"`), clippy with
  `-D warnings` on macOS **and** on the Windows target.
- **Errors:** return `AppResult<T>` (`core/errors.rs`). Use the variant that matches; the UI
  translates `error.<code>`. If you add a variant: give it a `code()`, a message, and add
  `error.<code>` to every locale. Wrap OS errors with context, e.g. `win_err("IMMDevice::GetId", e)`
  on Windows or `AppError::Platform(format!(…))`.
- **No panics on runtime data.** `unwrap()`/`expect()` only for invariants (poisoned locks, constant
  data). Release builds use `panic = "abort"`; the panic hook writes `last-crash.txt` (see
  [troubleshooting.md](troubleshooting.md)).
- **`unsafe`:** every block has a `// SAFETY:` comment stating why it is sound (ownership of FFI
  buffers, COM lifetimes, thread affinity).
- **Platform code:** gate with `#[cfg(target_os = "macos")]` / `#[cfg(windows)]` inside
  `platform/`, behind `CaptureBackend` or a small cross-platform module (`keystore.rs`,
  `clipboard.rs`, `sound.rs`, `disk.rs`) that has one `os` sub-module per platform.
- **Threads:** never block the async runtime — commands that touch the OS, disk or SQLite use
  `spawn_blocking`. Long work emits progress events instead of blocking the UI.
- **Serde:** types crossing IPC use `#[serde(rename_all = "camelCase")]`; enums use explicit lowercase
  names that the TS union types mirror.

## TypeScript and Svelte

- Svelte 5 **runes only** (`$state`, `$derived`, `$effect`, `$props`); no legacy stores.
- Shared state is a class with `$state` fields exported as a singleton from `src/lib/stores/*.svelte.ts`.
- The UI never calls `invoke`/`listen` directly in components: add a typed wrapper to
  `src/lib/ipc/commands.ts` (or `ai.ts`, `updates.ts`) and types to `src/lib/ipc/types.ts`.
- Pure helpers (formatting, geometry, plans) live in plain `.ts` files with a `*.test.ts` next to them.
- Styles use design tokens (`var(--…)` from `src/lib/design/tokens.css`), never raw colors. See [ui.md](ui.md).
- `pnpm check` must pass with **zero warnings** (a11y warnings included).

## Swift (FrameLyKit)

- Every export is `@_cdecl("fl_…")`, returns a JSON envelope via `respond { … }`, and is matched by
  an `extern "C"` declaration in `src-tauri/src/platform/macos/mod.rs`. See
  [platform-macos.md](platform-macos.md#the-c-boundary).
- Exports are called from Rust worker threads: use `blocking { … }` for async APIs and `onMain { … }`
  for AppKit; never block the main thread.
- Tests in `native/macos/FrameLyKit/Tests/FrameLyKitTests/` (XCTest).

## Logging

- Rust: `tracing` with structured fields — `tracing::info!(device = %name, format = %fmt, "recording: microphone")`.
  Message is a short lowercase phrase; data goes in fields. Use `warn!` for recovered failures,
  `error!` for user-visible failures.
- **Never log** transcript text, prompts, API keys, tokens or file contents. Paths and sizes are fine.
- UI: uncaught errors are forwarded automatically (`installErrorForwarding` in each window's
  `main.ts`); use `logUi(level, message)` from `src/lib/ipc/log.ts` for explicit events.

## Tests

| Where | What | Run |
|---|---|---|
| `#[cfg(test)] mod tests` at the bottom of each Rust file | Pure logic (settings, migrations, timelines, mixer, library index, transcripts, exports, prompts, provider availability) | `cd src-tauri && cargo test -q --lib` |
| `#[test] #[ignore = "…"]` | Live tests needing real OS APIs, models, network or the Keychain | see [development.md](development.md#live-and-end-to-end-tests-ignored-by-default) |
| `src/**/*.test.ts` (Vitest, Node env) | UI helpers, i18n completeness, editor model, overlay geometry, credits | `pnpm test` |
| `native/macos/FrameLyKit/Tests` | Swift mixer, pause timeline, media, A/V sync | `swift test` |
| `cloud/test`, `website/src/**/*.test.ts` | Worker and site | `pnpm test` in each |

Write a test for every bug fixed in pure code (reproduce first). For OS-level fixes that can't be
unit-tested, say in the commit how it was verified (machine, OS, what you measured).

## Commits and changelog

- Commit subject: `Area: what changed` or a plain sentence, imperative/descriptive, no trailing
  period — e.g. `Windows app audio: keep the stream open for the app's lifetime`,
  `Transcripts get their own editable title`. Body (optional) explains why.
- End with the `Co-Authored-By:` line your tool requires, if any.
- User-visible changes go into `CHANGELOG.md › [Unreleased]` (Keep a Changelog sections: Added,
  Changed, Fixed, Known limitations). Features announced on the site also need
  `website/src/i18n/*.ts` updates.
- One logical change per commit. Run the checks in [AGENTS.md](../AGENTS.md#commands-run-from-the-repo-root-unless-noted) first.
- Several sessions commit to `main`: `git pull --rebase` before `git push`; never force-push.

---

## Checklists

### Add a feature

1. Find the area doc ([architecture.md](architecture.md#where-things-live)) and the closest existing feature.
2. Pure logic in `src-tauri/src/core/` (or a TS helper) with tests.
3. OS work behind `CaptureBackend` or a `platform/` helper, implemented for **both** OSes (or
   reported as unsupported with `AppError::Unsupported` and hidden in the UI).
4. IPC: [add a command](#add-an-ipc-command) and, if the backend pushes updates, an event
   `area://name` emitted from the service.
5. UI: store/component/page; every string [translated](#add-a-translated-string).
6. Settings, if any: [add a setting](#add-a-setting).
7. Docs + `CHANGELOG.md [Unreleased]`; credits in `src/lib/credits.ts` if you added a third-party
   library or model.
8. Run all checks; test the real app (`pnpm tauri dev`, or `pnpm app:mac` for permissions).

### Add an IPC command

1. Implement the logic in the service (`src-tauri/src/<area>.rs`) or `core`.
2. Add the command to `src-tauri/src/commands/<area>.rs`, thin:
   ```rust
   #[tauri::command]
   pub async fn library_rename(app: AppHandle, id: i64, name: String) -> AppResult<Item> {
       blocking(move || library::rename(&app, id, &name)).await
   }
   ```
   (`blocking` = `spawn_blocking` helper defined at the top of the file.) Name it `<area>_<verb>`
   in snake_case.
3. Register it in `tauri::generate_handler![…]` in `src-tauri/src/lib.rs` (a missing entry fails at
   run time, not compile time).
4. Add the typed wrapper in `src/lib/ipc/commands.ts` (camelCase name, camelCase argument keys —
   Tauri maps them to the Rust snake_case parameters):
   ```ts
   libraryRename: (id: number, name: string) => invoke<LibraryItem>('library_rename', { id, name }),
   ```
   and any new types in `src/lib/ipc/types.ts`, mirroring the Rust serde shape.
5. App commands need no capability entry; plugin APIs used from a window do
   (`src-tauri/capabilities/*.json`).
6. Handle errors in the UI with `isAppError(e)` and `t('error.' + e.code)` (see `settings.svelte.ts`).

### Add a setting

Details: [settings.md](settings.md).

1. Add the field to the right section struct in `src-tauri/src/core/settings.rs` with a doc comment
   and a value in that struct's `Default`. Sections are `#[serde(default)]`, so old files load.
2. Constrain it in `Settings::validate` if it has a range or format; add a test.
3. Changing the meaning or default **for existing users** needs a migration: bump
   `CURRENT_VERSION`, add an `if version < N` step in `migrate`, add a test with an old JSON.
4. Side effects on change (theme, tray, shortcuts, autostart, folders) go into
   `commands::settings::update_settings`.
5. Mirror the field in `src/lib/ipc/types.ts` (`Settings`).
6. Add the row in `src/windows/main/pages/Settings.svelte` with `SettingRow` + a control, writing
   through `set((d) => (d.section.field = v))`. Strings in all locales.
7. If it belongs to a category that "Restore defaults" covers, nothing else to do (`reset_scope`
   resets the whole section); a new section needs a case there.

### Add a translated string

1. Add the key to `src/lib/i18n/en.ts` (or `ai/en.ts`, `updates/en.ts` for those areas). Keys are
   dotted, prefixed by area: `settings.capture.copy`, `ai.empty.title`, `error.<code>`.
2. Add the same key to `pt-BR.ts`, `es.ts`, `fr.ts`, `de.ts` in the same folder. The `Messages` type
   and `translate.test.ts` reject missing or extra keys.
3. Use it: `t('settings.capture.copy')`, with params `t('key', { n: 3 })` for `{n}` placeholders.
4. Tray menu strings are separate, in Rust: `src-tauri/src/core/locale.rs` (`tray_strings`).
   Installer strings: `src-tauri/installer/lang/*.nsh`.
5. Tone: short, warm, plain language; no jargon.

### Add an AI provider (summaries/topics)

Details: [ai.md](ai.md#providers).

1. `ProviderId` in `src-tauri/src/core/settings.rs`: new variant, `as_str`, `is_external`,
   `needs_key`; mirror in `src/lib/ipc/ai.ts` (`ProviderId` union).
2. `src-tauri/src/intelligence/providers.rs`: a `Runner` impl (context window, model name, `run`
   returning JSON matching `generation::schema`), availability `reason`s, and key account if it
   uses one. HTTP only through `providers::external_agent` / `intelligence/net.rs` (respects Local
   only).
3. `intelligence/mod.rs::open_runner`: build the runner; return `LocalOnly`, `ConsentRequired`,
   `MissingKey` etc. instead of falling back.
4. External providers: consent is per provider (`intelligence.consents`); the UI must ask before
   the first request.
5. UI: provider lists/labels in `src/lib/components/ai/` and `src/lib/i18n/ai/*.ts`; credits in
   `src/lib/credits.ts`.
6. Tests: availability reasons (see `external_providers_need_switch_key_and_consent`), output
   normalization.

### Fix a platform-specific bug

1. Reproduce and read the log (`FRAMELY_LOG=debug`; [troubleshooting.md](troubleshooting.md)).
2. Read the platform doc's rules first: [platform-macos.md](platform-macos.md),
   [platform-windows.md](platform-windows.md#rules-that-must-not-be-broken).
3. If the fix can be expressed as pure logic (timing, conversion, sizing), move that logic to `core/`
   and test it there.
4. Lint **both** targets (`cargo clippy …` and `pnpm check:windows`); Swift changes: `swift test`.
5. Verify on real hardware for capture/audio/permissions; record the machine and result in the
   commit message. Add a line to the platform doc if you learned a new rule.
