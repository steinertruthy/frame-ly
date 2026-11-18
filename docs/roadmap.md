# Roadmap

| Stage | Scope | State |
|---|---|---|
| 0 | Toolchain, Tauri + Svelte skeleton, design tokens, main window, tray, settings, logs, CI | ✅ |
| 1 | `CaptureBackend`, FrameLyKit (Swift), display/window/audio listing, permissions | ✅ |
| 2 | Screenshots end to end (screen, monitor, window, region), clipboard, preview | ✅ |
| 3 | Video recording on macOS (SCK + AVAssetWriter), pause, recovery | ✅ |
| 4 | Video recording on Windows (WGC + Media Foundation) | ✅ (full screen validated on x64 hardware; window, region, pause and audio still to verify on more machines) |
| 5 | Audio: microphone, system, mix, sync (macOS and Windows) | ✅ |
| 6 | Tray quick panel + global shortcuts | ✅ |
| 7 | Library + dashboard | ✅ |
| 8 | Screenshot editor | ✅ |
| 8.5 | Redesign: dark identity, inline editing on the selection, settings by category, 5 languages, launch at login, notifications | ✅ |
| 8.6 | FrameLy AI: local transcription, summaries and topics; Apple Intelligence and optional external providers | ✅ (Windows: builds and packages; validate on more hardware) |
| 8.7 | FrameLy AI Cloud: FrameLy's own cloud provider, anonymous device registration, limits | ✅ (published; default for summaries/topics since 0.2.0) |
| 8.8 | Automatic updates, FrameLy's own Windows installer | ✅ (0.3.0–0.3.1) |
| 9 | Polish, Apple/Microsoft code signing | ⏳ |

## Known limitations

- One monitor per recording; regions don't span monitors.
- Windows 10 draws a yellow border during capture (absent on Windows 11).
- DRM-protected content records black / silent — OS behavior, not worked around.
- Variable frame rate: frames are recorded only when the screen changes, each with its real time.
  Common players handle it; some video editors prefer converting to constant frame rate.
- H.264 is limited to 4096×2304 by hardware: native 5K recordings in H.264 are scaled down (use HEVC).
- Windows: HEVC needs Microsoft's "HEVC Video Extensions"; without them FrameLy records H.264.
- The library lists files directly in the recordings and screenshots folders (no sub-folders) and
  refreshes when opened; there is no continuous file-system watching.
- FrameLy AI: no local speaker separation (diarization); on Windows models run on the CPU only
  (Vulkan/CUDA after hardware testing); PDF export uses standard fonts and Latin script only
  (TXT/Markdown cover the rest).
- Installer screens not yet checked at display scaling above 100 %.
- Next for FrameLy AI: Windows GPU, local diarization, Apple's SpeechAnalyzer as an alternative
  transcription engine on macOS 26.
