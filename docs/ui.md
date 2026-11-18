# User interface

Svelte 5 (runes) + TypeScript, rendered by the system WebView (WebKit on macOS, WebView2 on
Windows). Built by Vite with `src/windows` as root (`vite.config.ts`).

## Windows

Each native window has its own HTML entry and folder, so it loads only what it uses. The Rust side
creates the window with `WebviewWindowBuilder` and the matching `<name>.html`.

| Window (label) | Entry | Created in | Purpose |
|---|---|---|---|
| `main` | `src/windows/main.html` → `main/App.svelte`, `main/pages/{Home,Library,Transcripts,Settings}.svelte` | `windows.rs` | Main window (hidden on close) |
| `panel` | `panel.html` → `panel/Panel.svelte` | `panel.rs` | Quick panel anchored to the tray icon; `NSPanel` on macOS |
| `overlay-<n>` | `overlay.html` → `overlay/Overlay.svelte` (+ `geometry.ts`) | `screenshot.rs` | Selection + inline editing, one per monitor |
| `preview` | `preview.html` | `screenshot.rs` | Post-capture card |
| `editor` | `editor.html` → `editor/Editor.svelte`, `model.ts`, `render.ts` | `editor.rs` | Screenshot editor |
| `countdown` | `countdown.html` | `recording.rs` | Countdown before recording |
| `recbar` | `recbar.html` | `recording.rs` | Floating recording controls (time, meters, pause/stop), excluded from capture |
| `rec-frame` | `frame.html` | `recording.rs` | Outline of the recorded region |

Each `main.ts` imports `src/app.css`, calls `installErrorForwarding()` and mounts its root component.

### Add a window

1. `src/windows/<name>.html` + `src/windows/<name>/main.ts` + root component (copy `preview/`).
2. Add the entry to `build.rollupOptions.input` in `vite.config.ts`.
3. Create it in Rust with `WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("<name>.html".into()))`
   — usually created hidden and shown after the first render (no white flash).
4. Add the label to a capability in `src-tauri/capabilities/` (`main.json` or `capture.json`) with
   only the permissions it needs.
5. Floating capture-related windows must be excluded from capture (see `platform/windows/chrome.rs`
   and the ScreenCaptureKit filter on macOS).

## State and IPC

- Stores in `src/lib/stores/*.svelte.ts` are classes with `$state` fields, exported as singletons
  (`settings`, `recording`, `library`, `ai`, `nav`, `system`, `toasts`, `updates`, `recordPrefs`).
- `settings.update(draft => …)` sends the whole settings object to `update_settings` and applies
  **what the backend returns**; other windows sync via `settings://changed`.
- `nav` (`nav.svelte.ts`) holds the main window's page and one-shot targets (settings category and
  section anchor, transcript and tab) used by `framely --show` and cross-page links.
- All backend calls go through `src/lib/ipc/*` ([contributing.md](contributing.md#add-an-ipc-command)).
- Errors: `isAppError(e)` → `t('error.' + e.code)`, shown with `toast('error', …)`.

## Design system

- **Tokens** in `src/lib/design/tokens.css`: brand blues (`--blue-*`), recording coral `--rec-500`
  (only for recording), `--ok-500`, `--warn-500`; semantic tokens (`--bg`, `--surface`, `--border`,
  `--text`, `--text-2`, `--accent`, `--focus-ring`…); type scale `--text-xs … --text-3xl`; radii
  `--radius-*`; spacing `--space-1 … --space-7`; motion `--ease-*`, `--dur-*`.
- Themes via attributes on `<html>` set by `applyAppearance` (`settings.svelte.ts`):
  `data-theme="dark|light|system"` (dark is FrameLy's identity and the default), `data-motion`
  (`reduced` zeroes durations; `system` follows `prefers-reduced-motion`), `data-ambient="on|off"`
  for the `Aurora.svelte` ambient light (transform-only animation).
- Use semantic tokens in components, never hex values. Global styles: `src/app.css`.
- Icons: `@lucide/svelte`.
- **Components** (`src/lib/components/`): `Button`, `Toggle`, `Segmented`, `Select`, `Slider`,
  `Badge`, `ConfirmDialog`, `ContextMenu`, `Toaster`, `Section` (settings card, `anchor` for
  `--show settings:<cat>:<anchor>`), `SettingRow`, `PermissionRow`, `ShortcutRecorder`,
  `LevelMeter`, `MicTest`, `MuteMeter`, `RecordOptions`, `RecordSetup`, `RecordingBanner`,
  `RecordingEstimate`, `UpdateNotice`, `LogoMark`, `Aurora`; feature folders `ai/` and `library/`.
  Reuse these before writing new controls; every control takes a `label` for accessibility.
- `pnpm check` fails on Svelte a11y warnings — fix them, don't suppress.

## Settings page

`src/windows/main/pages/Settings.svelte` lists categories (`general`, `captures`, `recording`,
`audio`, `intelligence`, `shortcuts`, `appearance`, `storage`, `about`). A row is:

```svelte
<SettingRow title={t('settings.capture.copy')} hint={t('settings.capture.copy.hint')}>
  <Toggle label={t('settings.capture.copy')} checked={s.capture.copyToClipboard}
          onchange={(v) => set((d) => (d.capture.copyToClipboard = v))} />
</SettingRow>
```

Backend side of settings: [settings.md](settings.md).

## Internationalization

- Five locales: `pt-BR`, `en`, `es`, `fr`, `de` (`src/lib/i18n/translate.ts`, `LOCALES`).
- `en.ts` defines the canonical keys and the `MessageKey`/`Messages` types; `ai/<locale>.ts` and
  `updates/<locale>.ts` are merged into each locale. Every locale must have exactly the same keys
  (type check + `translate.test.ts`).
- `t(key, params?)` from `$lib/i18n/index.svelte` is reactive; `{name}` placeholders are replaced
  from `params`.
- The `system` language resolves the same way in TS (`resolveLocale`) and Rust
  (`core/locale.rs::resolve`) — keep them in sync. The tray menu's strings live in Rust
  (`core/locale.rs`), the installer's in `src-tauri/installer/lang/*.nsh`.
- How to add a string: [contributing.md](contributing.md#add-a-translated-string).

## Global shortcuts

Defaults (`src-tauri/src/core/shortcuts.rs::defaults`; `CmdOrCtrl` = ⌘ on macOS, Ctrl on Windows):

| Action | Default |
|---|---|
| Open quick panel | CmdOrCtrl+Alt+Shift+Space |
| Capture screen / region / window | CmdOrCtrl+Alt+Shift+3 / 4 / 5 |
| Record region | CmdOrCtrl+Alt+Shift+R |
| Record window / screen | (none) |
| Stop recording | CmdOrCtrl+Alt+Shift+S |
| Pause / resume | CmdOrCtrl+Alt+Shift+P |

The core normalizes accelerators, requires ⌘/Ctrl/Alt (so typing isn't hijacked), rejects
OS-reserved combos and duplicates. Registration (`shortcuts.rs`, `tauri-plugin-global-shortcut`)
reports per-action failures (another app owns the combo) to Settings via `shortcuts://status`.
While the user records a new shortcut, the current ones are suspended.
