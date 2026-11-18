# Settings and migrations

Code: `src-tauri/src/core/settings.rs` (model, defaults, validation, migration, store — pure and
tested), `src-tauri/src/commands/settings.rs` (IPC and side effects), `src/lib/ipc/types.ts`
(`Settings` mirror), `src/lib/stores/settings.svelte.ts` (UI store),
`src/windows/main/pages/Settings.svelte` (page).

## File

- `settings.json` in the app config dir: `~/Library/Application Support/app.framely.desktop/` on
  macOS, `%APPDATA%\app.framely.desktop\` on Windows.
- Pretty JSON, camelCase keys, with a top-level `version` (`CURRENT_VERSION`, currently **4**).
- Written atomically (temp file + rename). A file that can't be parsed or validated is moved to
  `settings.corrupt-<timestamp>.json` and defaults are restored — never silently overwritten.
- No secrets here: API keys live in the OS keychain ([ai.md](ai.md#privacy-rules)).

## Shape

| Section | Contents |
|---|---|
| `general` | `language` (`system`/`pt-BR`/`en`/`es`/`fr`/`de`), `launchAtLogin`, `closeToTray`, `showOnLaunch`, `notifications`, `sounds` |
| `appearance` | `theme` (`dark` default, `light`, `system`), `motion`, `ambientLight`, `sidebarCollapsed` |
| `capture` | `copyToClipboard`, `saveToDisk`, `showPreview`, `format` (`png`/`jpeg`), `jpegQuality` (50–100) |
| `recording` | `fps` (24/30/60), `quality`, `resolution`, `codec`, `countdown` (≤ 10 s), `showCursor`, `target`, `showPreview`, `showControls` |
| `audio` | `systemAudio`, `microphone`, `microphoneDevice`, `systemGain`/`microphoneGain` (0–2), `noiseSuppression` |
| `shortcuts` | action → accelerator (`""` = off), validated by `core/shortcuts.rs` |
| `storage` | `recordingsDir`, `screenshotsDir` (absolute; default `~/Movies/FrameLy` or `Videos\FrameLy`, `~/Pictures/FrameLy`) |
| `intelligence` | profile, precision, language (`auto` or ISO 639-1), summary/topics detail, auto and provider, `localOnly`, `externalEnabled`, model names, `consents`, `appleEnabled` |
| `updates` | `autoCheck` |

The struct definitions in `core/settings.rs` are the reference; each field has a doc comment.

## Loading: defaults, merge, migrate

`Settings::from_json`:

1. Parse the JSON (must be an object).
2. `migrate(&mut value)` — run every step with `version < N`.
3. Overlay the stored values on `Settings::defaults(dirs)` (`merge`), so **missing fields take
   defaults** and partial or older files load without a migration.
4. Deserialize and set `version = CURRENT_VERSION`.

Then `validate()` checks ranges and formats (fps, gains, countdown, JPEG quality, absolute folders,
language code, shortcuts).

So: **adding a field** with a sensible default needs no migration. A migration is needed only to
change what existing users get — a new default that should apply to people who never chose, a
renamed/moved field, or a changed meaning.

## Migration history

| Step | Change |
|---|---|
| v0 → v1 | Establishes `version`. |
| v1 → v2 | Default theme became Dark; files without a theme keep `system` explicitly. |
| v2 → v3 | Summary and topics run automatically after transcription (turned on once). |
| v3 → v4 | FrameLy AI Cloud became the default for summaries/topics: `"framely"` → `"cloud"` once, only in builds with a Cloud URL. |

### Write a migration

```rust
pub const CURRENT_VERSION: u32 = 5;
// in migrate():
if version < 5 {
    // v4 → v5: <what and why>. Operate on serde_json::Value; keep user choices.
}
```

Add a test that loads an old JSON (`Settings::from_json(r#"{"version":4, …}"#, &d)`) and checks the
result, next to the existing `v3`/`v4` tests.

## Saving and side effects

- The UI changes settings only through `update_settings` with the whole object; the store applies
  the returned value (`settings.update(draft => …)`), and every window receives `settings://changed`.
- `SettingsStore::replace` validates, creates the media folders, writes, and swaps.
- `update_settings` reacts to changes: theme → `windows::apply_theme`; language → tray labels;
  `launchAtLogin` → autostart; storage folders → asset scope + library refresh; shortcuts →
  re-register. Add new side effects there.
- `reset_settings(scope)` restores one category (`general`, `captures`, `recording`, `audio`,
  `intelligence`, `shortcuts`, `appearance`, `storage`) or `all`; data (recordings, transcripts,
  models, keys) is never touched. A new section needs a case in `reset_scope`.

Checklist: [contributing.md](contributing.md#add-a-setting).
