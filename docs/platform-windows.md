# Windows platform

The Windows backend is plain Rust over the `windows` crate (Win32 + WinRT), in
`src-tauri/src/platform/windows/`. It implements the same `CaptureBackend` trait as macOS
(`src-tauri/src/platform/mod.rs`). Shared, pure pieces of the recording pipeline live in
`src-tauri/src/core/media_timeline.rs` (pause timeline, audio mixer, resampler), `core/yuv.rs`
(CPU NV12 fallback) and `core/denoise.rs` (RNNoise), and are unit-tested on any OS.

You can lint this code from macOS (`pnpm check:windows`) but not run it: anything that touches
capture, audio or permissions must be tested on a real Windows machine.

## Files

| File | Owns |
|---|---|
| `mod.rs` | `WinBackend` (`CaptureBackend` impl), `win_err` helper |
| `displays.rs` | `EnumDisplayMonitors`, `GetMonitorInfoW`, `GetDpiForMonitor`, friendly names via `QueryDisplayConfig` |
| `enum_windows.rs` | `EnumWindows` filtered like Alt-Tab (visible, not minimized, no owner, no `WS_EX_TOOLWINDOW`, not DWM-cloaked), bounds via `DWMWA_EXTENDED_FRAME_BOUNDS` |
| `capture.rs` | Single-frame Windows.Graphics.Capture → staging texture → BGRA `Frame`; D3D11 device with WARP fallback |
| `recorder.rs` | Recording: WGC frames → D3D11 video processor → Media Foundation; audio threads; `calibration` feature |
| `media.rs` | `IMFSinkWriter` MP4 writer (H.264/HEVC + AAC); library video info/thumbnails via `IMFSourceReader`; decode for transcription |
| `wasapi.rs` | WASAPI capture: microphones, output loopback, **process loopback** (all apps), format conversion, volume compensation |
| `audio.rs` | Microphone list (`IMMDeviceEnumerator`) |
| `permissions.rs` | Microphone consent from the privacy registry (`ConsentStore\microphone`) |
| `keystore.rs` | Credential Manager (`CredWriteW`/`CredReadW`) for API keys and the Cloud device key |
| `chrome.rs` | Overlays in physical pixels, `WDA_EXCLUDEFROMCAPTURE` on floating windows |

Coordinates are physical pixels (the process is per-monitor DPI aware); `scaleFactor = DPI / 96`.

## Video

```
Windows.Graphics.Capture (frames only when the screen changes, capped at fps)
  → copy to our own texture
  → ID3D11VideoProcessor on the GPU: crop (region), aspect-preserving scale, BGRA → NV12 BT.709
       (fallback: core::yuv on the CPU when the GPU path fails on this machine)
  → NV12 samples from IMFVideoSampleAllocatorEx (encoder late → drop the frame, never block capture)
  → IMFSinkWriter: hardware H.264/HEVC + AAC → fragmented MP4 (recoverable after a crash)
```

- HEVC needs Microsoft's "HEVC Video Extensions"; without them the writer falls back to H.264 and
  logs it.
- The OS ending the capture (window closed, display gone) finalizes the file and sends
  `NativeEvent::Stopped`.
- `--features calibration` (test builds only) also encodes each recording at other quality levels
  (`calibration_writers`) to tune the bitrate mapping in `media.rs`.
- Windows 10 draws a yellow border around captured content (OS behavior; absent on Windows 11).

## Audio

```
system audio ── process loopback, every app except FrameLy (preferred, Windows 10 2004+)
             └─ fallback: loopback of every active output device (+ software-volume compensation)
microphone  ── WASAPI capture (AUTOCONVERTPCM to 48 kHz stereo f32, fallback: own conversion)
     → timestamps on the QPC clock (same clock as WGC frames)
     → core::media_timeline::AudioMixer (gains, soft-clip, pause) → AAC 192 kbps in the same MP4
```

- **System audio comes from the apps themselves** (`AppsCapture`, `VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK`
  with `PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE`): independent of the output in use and
  of its volume or mute, and without FrameLy's own sounds.
- If that stream can't be opened (older Windows, every setup crash-marked), the recorder captures
  a loopback of **every** active output, and adds outputs that appear mid-recording (rescans every
  second).
- The mic is denoised with RNNoise; the mixer subtracts its fixed latency from the timestamp.

## Rules that must not be broken

These come from crashes and bugs seen on real PCs. Each is commented in the code; keep the
comments and the behavior.

1. **Endpoint loopback ignores `AUTOCONVERTPCM`.** Windows accepts the flag and does nothing, so an
   8-channel HDMI mix would be read as stereo garbage. Loopback streams always read the device's
   own mix format and FrameLy converts it (`parse_format` + `convert` in `wasapi.rs`: any channel
   count, 16/24/32-bit int or float, any rate; multichannel keeps front L/R and folds center in).
2. **Software volume is applied before the loopback tap** on outputs without hardware volume
   (HDMI/DisplayPort monitors, some digital outputs). A low Windows slider would give a near-silent
   recording, so the loopback reads `IAudioEndpointVolume` on every packet and undoes it (up to
   60 dB, `MAX_VOLUME_COMPENSATION`). Compensation drops immediately when the volume goes up and
   rises over ~0.5 s when it goes down; output is clamped to full scale.
3. **The process-loopback stream is opened once and never stopped or released.** It lives on a
   permanent worker thread (`apps_worker`, thread `framely-app-audio`) for the whole life of the
   app; releasing it closed the app on a test PC. Between recordings its packets are read and
   discarded so the next recording starts fresh. Don't add a `Drop` that stops it, and don't move
   it to a per-recording thread.
4. **The activation objects stay alive** (`Activation`: the async operation, completion handler and
   params) until the stream closes — Microsoft's sample releases them early; on a test PC that
   crashed the app.
5. **Always pass both position pointers to `GetBuffer`.** Process loopback writes the device
   position without checking for null.
6. **Crash markers.** Each process-loopback setup (`AppAudioSetup` A/B/C) writes its current step to
   `%TEMP%\framely-app-audio-s-{a,b,c}.attempt` and removes it once audio flows or the stream closes
   cleanly. A marker left behind means that setup once took FrameLy down: it is skipped from then on
   and the next setup is tried; when all are marked, the per-output fallback is used. Deleting the
   markers re-enables a setup (see [troubleshooting.md](troubleshooting.md#windows-system-audio-missing-or-silent)).
7. **Continuous timestamps, no per-packet resampling.** Process-loopback packet timestamps are not
   reliable, so packets are stamped by `continuous_stamp`: each packet starts where the previous
   ended; arrival time is used only to start or after a real break (> 200 ms). Setups request
   48 kHz so nothing is resampled; the linear resampler is only for devices whose mix format isn't
   48 kHz, applied by `convert`, never to "fix" timing.
8. **One clock.** Video and audio both use QPC in 100 ns units. Don't introduce another time base.

## Permissions

| | Windows behavior |
|---|---|
| Screen | Always available to desktop apps (Windows 10 1903+); checked with `GraphicsCaptureSession::IsSupported`. |
| Microphone | No OS prompt for unpackaged apps. FrameLy reads the consent keys (global, user, "desktop apps") and deep-links to Settings. |

## Keys and data

- API keys and the Cloud device key: Credential Manager, generic credentials under `FrameLy/ai/*`;
  the uninstaller's "delete data" option removes them (`src-tauri/installer/hooks.nsh`).
- Settings, library, transcripts and models: `%APPDATA%\app.framely.desktop`; logs:
  `%LOCALAPPDATA%\app.framely.desktop\logs`.

## Local AI on Windows

whisper.cpp and llama.cpp run on the CPU and require AVX2 (checked at run time, see
`intelligence/hardware.rs`); GPU (Vulkan/CUDA) is not used yet. Build details:
[development.md](development.md#windows).
