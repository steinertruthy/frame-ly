# Changelog

All notable changes to FrameLy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Translations of each entry
for the website live in `website/src/content/changelog/`.

## [Unreleased]

## [0.1.0] - 2026-11-15

First public version.

### Added

- Screenshots of the full screen, a window or a region, with editing right on the selection: arrows, shapes, text, highlight, blur and crop, with undo and redo. Copy to the clipboard or save.
- Screen recording of the full screen, a window or a region, with system audio and microphone, pause and resume, countdown, H.264 or HEVC, and an estimated file size before starting.
- Quick panel in the menu bar or system tray, global shortcuts for every capture mode and a command line (`--capture`, `--record`, `--panel`, `--show`, `--edit`, `--transcribe`).
- Library of screenshots and recordings, and settings by category.
- Interface in English, Portuguese (Brazil), Spanish, French and German.
- Transcription of recordings and imported audio or video on your computer (whisper.cpp), with automatic language detection, word timing, editing and export to TXT, SRT, VTT, Markdown and PDF.
- Summaries and topics with FrameLy AI on your computer (llama.cpp), Apple Intelligence on macOS 26 or later, or Anthropic and OpenAI with your own API key, after your consent.
- Model manager with SHA-256-verified downloads, pause and resume, and a profile suggested from your computer's memory.
- "Local only" mode, which blocks every connection.

### Known limitations

- The Windows recorder has been validated on real hardware for full-screen recording; window and region recording, pause, audio and local AI still need more testing on Windows PCs.
- Installers are not yet signed with an Apple Developer ID or a Windows code-signing certificate.
- Builds are available for Apple Silicon Macs and 64-bit Intel/AMD Windows only.
- Speakers are not identified in transcripts.
- A recording captures one display at a time.

[Unreleased]: https://github.com/steinertruthy/frame-ly/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/steinertruthy/frame-ly/releases/tag/v0.1.0
