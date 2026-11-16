# Changelog

All notable changes to FrameLy are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html). Translations of each entry
for the website live in `website/src/content/changelog/`.

## [Unreleased]

## [0.3.1] - 2026-11-16

FrameLy's own Windows installer, editable transcript titles and full credits.

### Added

- Transcripts have their own title: click it in the transcript view to rename it (Enter saves, Esc cancels). The media file is never renamed; the title is used in the list, search and export file names.
- Settings › About lists every open-source component and AI model FrameLy uses, with its license and a link.

### Changed

- The Windows installer and uninstaller are FrameLy's own, from start to finish: full-window screens in the app's colours, with install location, desktop shortcut and "Open FrameLy now". An existing installation is updated in place, keeping settings, transcripts and AI models.

### Known limitations

- The installer screens haven't been checked yet at display scaling above 100%.

## [0.3.0] - 2026-11-16

FrameLy now updates itself, and the Windows installer has FrameLy's look.

### Added

- Automatic updates: FrameLy checks GitHub Releases in the background (turn it off in Settings › About, never in "Local only" mode), shows a quiet notice with the release notes, and updates with one click — download progress, signature check, install and restart. A recording in progress is never interrupted: the update installs when it ends. `framely --update` checks and installs from the command line.

### Changed

- The Windows installer now has FrameLy's look, the app's five languages, FrameLy's own welcome and finish texts, and an uninstaller option that also removes saved API keys.

### Known limitations

- Versions 0.1.0 and 0.2.0 don't have the updater: download the first version with it once from the website.
- macOS builds aren't notarized yet, so macOS may ask for the Screen Recording permission again after an update.

## [0.2.0] - 2026-11-16

FrameLy AI Cloud is live and is now the default for summaries and topics.

### Added

- FrameLy AI Cloud in published builds: summaries and topics in the cloud, free, with no account or key, and a daily limit per computer. A partner AI provider generates the result and may use the content to improve its services — the app says so before the first use.

### Changed

- New installs and settings still on the previous default use FrameLy AI Cloud for summaries and topics; any other choice is kept. FrameLy asks for consent before sending anything, and "Local only" mode still blocks the cloud.
- The first-use setup downloads the local summary model only when FrameLy AI Local is the one generating summaries or topics.

### Fixed

- Confirmation dialogs (such as the consent prompt) now open above the New transcription sheet, and Escape closes the dialog first.

### Known limitations

- FrameLy AI Cloud isn't available in the European Economic Area, Switzerland or the United Kingdom; FrameLy AI Local, Apple Intelligence and your own Anthropic or OpenAI key work there.

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

[Unreleased]: https://github.com/steinertruthy/frame-ly/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/steinertruthy/frame-ly/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/steinertruthy/frame-ly/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/steinertruthy/frame-ly/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/steinertruthy/frame-ly/releases/tag/v0.1.0
