# Architecture

## Overview

```
┌──────────────────────────── UI (Svelte 5 in the system WebView) ────────────────────────────┐
│ src/windows/<window>/   pages & window roots      src/lib/components   reusable components  │
│ src/lib/stores          reactive state (runes)    src/lib/i18n         5 locales            │
│ src/lib/ipc             typed invoke()/listen() wrappers ◄── the only door to the backend    │
└──────────────────────────────────────────┬───────────────────────────────────────────────────┘
                                 Tauri IPC │ commands (snake_case)  ▲ events ("area://name")
┌──────────────────────────────────────────▼───────────────────────────────────────────────────┐
│ src-tauri/src/commands/*    thin #[tauri::command] layer: parse, spawn_blocking, delegate     │
│ src-tauri/src/*.rs          services with app state: screenshot, recording, library, editor, │
│                             panel, tray, shortcuts, updates, windows, protocol (framely://)   │
│ src-tauri/src/intelligence  FrameLy AI: jobs, speech, models, providers, cloud client         │
│ src-tauri/src/core          pure, OS-free, unit-tested: settings, errors, logging, SQLite,    │
│                             media timeline, image, video sizing, transcripts, export, locale  │
└──────────────────────────────────────────┬───────────────────────────────────────────────────┘
                     trait CaptureBackend   │   (+ keystore, clipboard, sound, disk helpers)
┌──────────────────────────────────────────▼───────────────────────────────────────────────────┐
│ src-tauri/src/platform/macos  → C ABI → native/macos/FrameLyKit (Swift: SCK, AVFoundation…)  │
│ src-tauri/src/platform/windows (Rust: WGC, D3D11, WASAPI, Media Foundation, Credential Mgr)  │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
       sidecar process: src-tauri/llm-worker → `framely-llm` (llama.cpp), JSON over stdin/stdout
```

**Principle:** the media pipeline (capture → encode → mux) is 100 % native per platform, because
that is where performance and quality live. Everything else is shared. There is one platform
abstraction (`CaptureBackend` in `src-tauri/src/platform/mod.rs`); don't add parallel ones.

## Stack decisions

| Decision | Why |
|---|---|
| Tauri 2, not Electron | System WebView: ~30–60 MB RAM idle, ~10 MB binary, fast start. |
| Svelte 5 (runes) | No virtual DOM, tiny runtime; each window loads only what it uses. |
| macOS engine in Swift | ScreenCaptureKit/AVFoundation are delegate/async APIs; Swift is idiomatic and safe there. Linked through a small C boundary. |
| Windows engine in Rust (`windows` crate) | Direct WinRT/COM access without a third language. |
| No FFmpeg | Hardware encoders (VideoToolbox, Media Foundation) already produce MP4 H.264/HEVC + AAC. Avoids ~60 MB, GPL/LGPL and extra processes. |
| SQLite for indexes, JSON for settings | Fast search/sort; settings stay readable and migratable. |
| llama.cpp in a separate process | whisper.cpp and llama.cpp vendor different `ggml` versions; linked together the linker silently mixes symbols (UB). See [ai.md](ai.md). |

## Where things live

| Path | Responsibility |
|---|---|
| `src-tauri/src/lib.rs` | App entry: plugins, `AppState`, setup order, window events, CLI (`--capture`, `--record`, `--show`…), **the list of registered IPC commands**. |
| `src-tauri/src/commands/` | IPC layer, one file per area (`capture`, `screenshot`, `recording`, `library`, `editor`, `settings`, `system`, `intelligence`, `updates`). |
| `src-tauri/src/core/` | OS-free logic: `errors` (`AppError`), `settings`, `logging`, `paths`, `locale` (tray strings), `library` (SQLite index), `recorder` (state machine), `video` (size/bitrate), `media_timeline` (pause timeline, audio mixer, resampler — Windows), `denoise` (RNNoise, both OSes), `yuv` (CPU NV12 fallback), `image` (BGRA/PNG/BMP), `transcript`/`transcripts` (model + SQLite), `pdf`, `resample`, `shortcuts`, `sounds`. |
| `src-tauri/src/platform/` | `CaptureBackend` trait and shared types; `macos/` (FFI to FrameLyKit), `windows/`; cross-platform helpers `keystore.rs`, `clipboard.rs`, `sound.rs`, `disk.rs`. |
| `src-tauri/src/screenshot.rs` | Capture session, overlays, delivery (file, clipboard), preview. |
| `src-tauri/src/recording.rs` | Recording flow: target, countdown, controls bar, region frame, stop, delivery, crash recovery. |
| `src-tauri/src/library.rs` | Library service over `core::library` (reconcile with folders, thumbnails, rename, trash). |
| `src-tauri/src/editor.rs` | Screenshot editor window (open, source image, save/copy). |
| `src-tauri/src/intelligence/` | FrameLy AI. See [ai.md](ai.md). |
| `src-tauri/src/panel.rs`, `tray.rs`, `shortcuts.rs` | Quick panel, tray/menu-bar icon, global shortcuts. |
| `src-tauri/src/windows.rs` | Main window creation, theme, `show_main`/`hide_main`. |
| `src-tauri/src/protocol.rs` | `framely://localhost/...` in-memory images (frozen frames, last capture, thumbnails, editor source). |
| `src-tauri/src/updates.rs` | Auto-update (see [release.md](release.md#automatic-updates)). |
| `src-tauri/src/notify.rs` | System notifications. |
| `src-tauri/llm-worker/` | `framely-llm` sidecar (llama.cpp). |
| `src-tauri/installer/` | Custom NSIS installer (see [release.md](release.md#windows-installer-nsis)). |
| `src-tauri/capabilities/` | Tauri permissions per window group. |
| `native/macos/FrameLyKit/` | Swift engine (static library). See [platform-macos.md](platform-macos.md). |
| `src/lib/ipc/` | `commands.ts` (+ `ai.ts`, `updates.ts`): typed command and event wrappers; `types.ts`: shared types mirroring Rust serde; `log.ts`: UI errors → Rust log. |
| `src/lib/stores/` | Reactive state as classes with `$state` (`settings`, `recording`, `library`, `ai`, `nav`…). |
| `src/lib/components/` | Reusable components (`ai/`, `library/` sub-folders for feature components). |
| `src/lib/design/tokens.css`, `src/app.css` | Design tokens and global styles. See [ui.md](ui.md). |
| `src/windows/` | One HTML entry + folder per window: `main`, `panel`, `overlay`, `preview`, `editor`, `countdown`, `recbar`, `frame`. |

## Runtime model

- **One process** (plus the optional `framely-llm` child). `lib.rs::run` builds the app; `setup`
  initializes, in order: logging + panic hook, settings, `AppState`, recording recovery, library,
  intelligence, updater, main window, CLI action, tray, panel, shortcuts, autostart.
- **`AppState`** (managed by Tauri) holds `SettingsStore`, the `CaptureBackend`, and the
  `screenshot`, `recording`, `library` and `intelligence` services.
- **Single instance:** a second launch forwards its CLI arguments to the running instance
  (`tauri-plugin-single-instance`). That is how `framely --capture region` etc. work.
- **Blocking work** (OS capture, disk, SQLite, FFI) runs in `tauri::async_runtime::spawn_blocking`
  from the command layer. AI jobs run one at a time on their own background thread.
- **Events** go backend → UI with `app.emit` / `emit_to`, named `area://name`
  (`settings://changed`, `recording://status`, `library://changed`, `ai://job`, `updates://status`…).
  Every window listens through `src/lib/ipc`.
- **Closing the main window hides it** (default; app stays in the tray). With "close to tray" off,
  closing quits — except during a recording, which always continues. Hiding in macOS full screen
  uses `windows::hide_main` (alpha 0, leave full screen, then hide) to avoid a black Space.

## Screenshot flow (`screenshot.rs`)

```
Full screen:   capture_display(monitor under the cursor) ───────────────────────────┐
Region/Window: capture_display(all, in parallel) → frozen frames                    │
               → one overlay window per monitor (overlay.html)                      │
               → overlay loads its frame from framely://localhost/frame/<id>        │
               → region: crop of the frozen frame                                   │
                 window: native capture_window (fallback: crop of the frame) ───────┤
                 inline edit (arrows, text, blur…) → finish_annotated (raw PNG body)│
                                                                                    ▼
            encode PNG/JPEG → save (optional) → clipboard (optional) → preview / notification
```

- The user selects on the exact image that will be saved (menus and hover states survive), and
  the magnifier shows real pixels.
- `framely://` serves in-memory images (BMP for overlays, PNG for the preview); nothing goes to
  temp files and the asset protocol stays limited to the media folders.
- The overlay sends the selection in CSS pixels + viewport size; Rust converts with the real
  frame size, so points (macOS) and DPI (Windows) behave the same.
- FrameLy windows never appear in captures: the ScreenCaptureKit filter excludes the app; on
  Windows every floating window gets `WDA_EXCLUDEFROMCAPTURE`, and the main window is hidden
  before the first recorded frame.
- Inline editing reuses the editor's model and renderer (`src/windows/editor/model.ts`,
  `render.ts`); the export is sent as a raw IPC body with header `x-framely-intent: copy|save`.

## Recording flow (`recording.rs`)

```
target: screen (monitor under cursor) | window/region (same overlay, red)
  → countdown window (Esc/click cancels)
  → native engine: frames + audio → hardware encoder → fragmented MP4 in <cache>/recordings/<id>.mp4
    + manifest <id>.json + thumbnail <id>.jpg
  → status every 500 ms → tray icon/tooltip, controls bar (recbar), home banner
  → stop (UI, tray, shortcut, CLI, or the OS: window closed, display gone, permission revoked)
  → file finalized → moved to the recordings folder → library → preview
```

- **State machine** `core/recorder.rs`: `Idle → Countdown → Recording ⇄ Paused → Finalizing`;
  invalid transitions are rejected; the clock subtracts pauses.
- **Pause** never stops the stream: samples inside a pause are dropped and later ones are
  re-timed (`PauseTimeline.swift` / `core::media_timeline`). Resume is instant.
- **Size/bitrate** `core/video.rs`: native/1080p/720p keeping aspect, codec limits (H.264 ≤
  4096×2304), even dimensions, bitrate by bits/pixel/frame per quality.
- **Recovery:** MP4 fragments every 2 s. If FrameLy dies, the next start finds the manifest and
  moves the file to the recordings folder as "(recovered)".
- **Audio:** system audio + microphone mixed into **one** AAC track (48 kHz stereo, gains,
  soft-clip, RNNoise on the mic), positioned by the same clock as video. Platform specifics:
  [platform-macos.md](platform-macos.md#audio), [platform-windows.md](platform-windows.md#audio).
- **Sounds** (`core/sounds.rs`, synthesized in memory) never play while capture is live, because
  the mic would record them.

## Transcription and AI flow

```
media → native decode → 16 kHz mono PCM → whisper.cpp in ~5 min blocks → transcripts.sqlite3
      → summary/topics: provider (FrameLy AI Local | Cloud | Apple | Anthropic | OpenAI)
      → export TXT/SRT/VTT/Markdown/PDF
```

Details, consent and providers: [ai.md](ai.md).

## Library

- **The folders are the source of truth.** The SQLite index (`core/library.rs`,
  `<app data>/library.sqlite3`) holds metadata for instant search/sort. `reconcile` adds, updates
  and removes items by comparing with disk; only new or changed files are probed. Runs at start,
  when opening the library and when the folders change.
- New captures are registered immediately with the metadata FrameLy already knows.
- Thumbnails (JPEG ≤ 480 px) are generated on demand at `framely://localhost/thumb/<id>` and cached.
- Playback uses Tauri's asset protocol (range requests for `<video>`), scoped to the two media
  folders (`library::allow_media_dirs`).
- Delete = move to the Trash (`trash` crate), always confirmed. Rename validates names valid on
  both OSes and keeps the extension.

## Screenshot editor

- `src/windows/editor/model.ts` (tested): shapes in original-image coordinates, crop, output size,
  snapshot history for undo/redo, hit testing.
- `render.ts`: one function draws both the canvas and the exported file (WYSIWYG). Blur is block
  pixelation applied at export; the original pixels don't stay in the file.
- The editor only opens images inside FrameLy's folders; "Save" never overwrites: it writes
  `name (edited).png` and registers it in the library.

## Security model

- Restrictive CSP; the asset protocol reaches only the recordings and screenshots folders.
- The UI has no generic "open path": `open_folder` accepts only FrameLy's folders, `open_link`
  only `https` URLs, preview actions only the last capture made by the app.
- Capabilities per window group in `src-tauri/capabilities/` (`main.json` for `main`/`editor`,
  `capture.json` for overlays, preview, countdown, frame, recbar, panel).
- Secrets live only in the OS keychain (`platform/keystore.rs`). See [ai.md](ai.md#privacy-rules).
