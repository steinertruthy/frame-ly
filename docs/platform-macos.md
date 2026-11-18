# macOS platform (FrameLyKit)

All OS work goes through the `CaptureBackend` trait (`src-tauri/src/platform/mod.rs`). On macOS the
implementation (`src-tauri/src/platform/macos/mod.rs`) is a thin FFI over **FrameLyKit**, a Swift
package in `native/macos/FrameLyKit`, compiled as a static library by `src-tauri/build.rs`
(`swift-rs`, macOS 14 deployment target) and linked with ScreenCaptureKit, AVFoundation,
CoreMedia, CoreGraphics, AppKit and Security. `FoundationModels` is **weak-linked** so the app
still opens on macOS 14/15. The Swift runtime comes from the OS (`/usr/lib/swift`).

The macOS app can only be built on macOS (`build.rs` panics otherwise).

## Files

| Swift file | Owns |
|---|---|
| `FFI.swift` | JSON envelope (`respond`), `fl_free_string`, `onMain`, `blocking` helpers |
| `Displays.swift`, `Windows.swift` | Monitors (`CGGetActiveDisplayList`, `NSScreen`), shareable windows (`SCShareableContent`) |
| `Capture.swift` | Screenshots (`SCScreenshotManager`) → BGRA buffer; clipboard PNG |
| `Recorder.swift` | `SCStream` → `AVAssetWriter` (VideoToolbox H.264/HEVC), fragmented MP4, events callback |
| `Audio.swift`, `Microphone.swift`, `AudioMixer.swift` | Device list, mic capture, mixing |
| `PauseTimeline.swift` | Pause/resume retiming |
| `Media.swift`, `Transcode.swift` | Video info/thumbnails; decode any media to 16 kHz mono PCM for transcription |
| `Permissions.swift` | Screen Recording / Microphone status and prompts |
| `Keychain.swift` | API keys and the Cloud device key |
| `Intelligence.swift` | Apple Intelligence (Foundation Models) |
| `Panel.swift`, `WindowHide.swift` | Non-activating `NSPanel` for the quick panel; invisible full-screen hide |
| `Sound.swift` | UI sounds via `NSSound` |

The Rust side of window appearance (vibrancy) is `src-tauri/src/platform/macos/chrome.rs`.

## The C boundary

Every export returns a heap-allocated JSON string, `{"ok": …}` or `{"err": {"code", "message"}}`,
which Rust frees with `fl_free_string`. Types are `Codable` in Swift and `serde` (camelCase) in Rust.

```swift
@_cdecl("fl_list_displays")
public func fl_list_displays() -> UnsafeMutablePointer<CChar> {
    respond { try listDisplays() }      // throws FFIError → {"err": …}
}
```

```rust
extern "C" { fn fl_list_displays() -> *mut c_char; }
// decoded by `call`/`call_with` + `decode::<T>()` in platform/macos/mod.rs
```

- **Pixels** don't go through JSON: Swift fills a `malloc`'d BGRA buffer (`fl_capture_display`,
  `fl_capture_window`), Rust copies it into a `Frame` and frees it with `fl_free_buffer`.
- **Recording events** go Swift → Rust through a registered C callback (`fl_rec_set_callback`,
  JSON `{"kind": "started" | "stopped", …}`).
- **Noise suppression** runs in Rust (`core/denoise.rs`) and is registered into Swift as a callback
  (`fl_set_denoiser`).
- **Threads:** Rust calls exports from blocking worker threads. `blocking { }` bridges async Swift
  APIs (it asserts it is not on the main thread); `onMain { }` runs AppKit work on the main thread
  with a timeout instead of deadlocking.

### Adding an export

1. Swift: `@_cdecl("fl_<name>") public func …` returning `respond { … }`; throw `FFIError` with a
   code the Rust `AppError` mapping understands (`permission_denied`, `platform`, …).
2. Rust: declare it in the `extern "C"` block of `platform/macos/mod.rs`, call it through
   `call`/`call_with`, and expose it via `CaptureBackend` (implement the Windows side too) or a
   macOS-only helper.
3. `swift test` and `cargo clippy` on macOS; `pnpm check:windows` to be sure Windows still builds.

## Capture and recording

| Function | API |
|---|---|
| Screenshot | `SCScreenshotManager.captureImage` (display: filter excluding FrameLy; window: `desktopIndependentWindow`) → sRGB BGRA |
| Clipboard | PNG straight into `NSPasteboard` (`fl_copy_png`) |
| Overlays | `NSWindow.level = .screenSaver`, covers menu bar and Dock, on all Spaces |
| Recording | `SCStream` (NV12, `minimumFrameInterval`, `sourceRect` for regions, single-window filter) → `AVAssetWriter` H.264/HEVC via VideoToolbox, fragmented MP4 (2 s), BT.709 |

Coordinates: global points, origin top-left (ScreenCaptureKit's space). Tauri's
`Monitor::work_area()` returns points labelled as physical pixels on macOS, so FrameLy positions
the panel and preview with its own monitor list (`NSScreen.visibleFrame`).

## Audio

```
ScreenCaptureKit system audio (excludesCurrentProcessAudio) ─┐
AVCaptureSession microphone (clock converted to host time) ──┤→ AudioMixer → AAC 192 kbps, 48 kHz stereo
                         video: first frame = time zero ─────┘   (gains, soft-clip, pause, RNNoise on mic)
```

- One AAC track with everything mixed, so players that read only the first track keep the mic.
- Every buffer is placed by its host-clock time; jitter ≤ 10 ms is glued contiguously, larger
  drift resynchronizes. Measured A/V offset +6–8 ms (`AVSyncTests`).
- A slow or removed source never blocks the others (output is at most 250 ms behind the most
  advanced source); an unplugged mic becomes silence.
- Mic permission is asked when recording starts with the mic on; if denied or absent, the
  recording continues without it and warns. Missing chosen device → system default.

## Permissions (TCC)

| | Behavior |
|---|---|
| Screen Recording | System prompt the first time, then only in System Settings. **Takes effect after relaunch** (`relaunch` command). macOS doesn't report "not determined", so FrameLy remembers in `UserDefaults` whether it already asked. `SCShareableContent` never triggers the prompt. |
| Microphone | Prompt via `AVCaptureDevice.requestAccess`; needs `NSMicrophoneUsageDescription` (`src-tauri/Info.plist`) **and** the `com.apple.security.device.audio-input` entitlement (`src-tauri/Entitlements.plist`) under the Hardened Runtime — without it macOS denies silently and FrameLy doesn't even appear in Privacy › Microphone. |

The UI re-reads permissions and devices whenever a window regains focus. FrameLy never tries to
work around these mechanisms.

Permissions are tied to the code signature: use `pnpm app:mac` (stable dev identity) to test them,
not `pnpm tauri dev` ([development.md](development.md#macos-permissions-and-the-dev-signature)).
Ad-hoc-signed releases may ask again after each update ([signing.md](signing.md)).

## Known limitations

- The window list shows only windows on the current Space (`onScreenWindowsOnly`).
- One monitor per recording; regions don't span monitors.
- Apple Intelligence requires macOS 26 and an eligible device; elsewhere it reports unavailable.
