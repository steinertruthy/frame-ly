/**
 * English copy — the reference shape for every language (see ./index.ts). Every claim here is
 * checked against the app's code: no future features, no invented numbers.
 */
export const en = {
  "meta": {
    "title": "FrameLy — Screen recording, screenshots and on-device transcription",
    "description": "FrameLy captures screenshots and records your screen on macOS and Windows, then transcribes, summarizes and organizes recordings into topics with AI that can run entirely on your computer.",
    "ogAlt": "FrameLy app window showing a recording with its transcript, summary and topics"
  },
  "a11y": {
    "skip": "Skip to content",
    "menu": "Open menu",
    "close": "Close menu",
    "language": "Language",
    "home": "FrameLy home",
    "screenshotOf": "Screenshot of FrameLy:",
    "playPause": "Play or pause the demo video"
  },
  "nav": {
    "overview": "Overview",
    "features": "Features",
    "ai": "AI",
    "how": "How it works",
    "download": "Download",
    "cta": "Download"
  },
  "os": {
    "macos": "macOS",
    "windows": "Windows"
  },
  "hero": {
    "eyebrow": "Screen recording · Screenshots · Transcription",
    "title": "Capture it. Record it.\nUnderstand it.",
    "subtitle": "FrameLy is a fast, quiet screen recorder and screenshot tool for macOS and Windows that turns your recordings into transcripts, summaries and topics — with AI that can run entirely on your computer.",
    "ctaFor": "Download for {os}",
    "ctaGeneric": "Download FrameLy",
    "ctaSecondary": "See how it works",
    "detected": "Detected: {os}",
    "notDetected": "Choose your system below",
    "compat": "macOS 14+ on Apple Silicon · Windows 10 22H2+ (x64)",
    "version": "Version {version}",
    "noRelease": "Installers are being prepared",
    "free": "Free and open source (MIT)"
  },
  "intro": {
    "eyebrow": "Overview",
    "title": "One app from capture to meaning.",
    "body": "FrameLy lives in your menu bar or system tray. Take a screenshot and mark it up right on the selection, record your screen with system audio and microphone, then let FrameLy transcribe the recording and pull out what matters. Your files stay in folders you choose.",
    "points": [
      {
        "title": "Native and light",
        "body": "Built with Rust and each system’s own capture APIs — ScreenCaptureKit on macOS, Windows.Graphics.Capture on Windows. No FFmpeg, no browser extension."
      },
      {
        "title": "Keyboard first",
        "body": "Global shortcuts for every capture mode, a quick panel, and a command line for launchers like Raycast, Alfred, Shortcuts or AutoHotkey."
      },
      {
        "title": "Your choice of AI",
        "body": "Transcription always runs on your computer. Summaries and topics use FrameLy AI Cloud by default, or FrameLy’s local models, Apple Intelligence or your own API key."
      }
    ]
  },
  "features": {
    "eyebrow": "Features",
    "title": "Five tools, one quiet workflow.",
    "benefit": "Why it helps",
    "items": {
      "screenshot": {
        "kicker": "Screenshots",
        "title": "Select, annotate, share — without leaving the selection.",
        "body": "Capture the full screen, a window or any region. Arrows, shapes, text, highlight, blur and crop appear right on the selection, with undo and redo. Copy to the clipboard or save in your folder.",
        "benefit": "Explain a bug or a design detail in seconds, with sensitive parts blurred.",
        "alt": "FrameLy screenshot editor with annotation tools around a selected region"
      },
      "recording": {
        "kicker": "Screen recording",
        "title": "Record the screen, a window or a region — with sound.",
        "body": "Mix system audio and microphone, pause and resume, and stop from a small recording bar or a shortcut. H.264 or HEVC, with a countdown and estimated file size before you start.",
        "benefit": "Walkthroughs, demos and meeting notes without setting up a studio.",
        "alt": "FrameLy home screen with capture and recording shortcuts"
      },
      "transcription": {
        "kicker": "Transcription",
        "title": "Every word, on your computer.",
        "body": "FrameLy transcribes recordings and imported audio or video with open speech models that run locally, detects the language automatically and keeps word-level timing. Click any line to jump to that moment, edit the text, and export TXT, SRT, VTT, Markdown or PDF.",
        "benefit": "Searchable recordings and ready-made subtitles, without uploading the audio.",
        "alt": "FrameLy transcript view with a video player and timestamped text"
      },
      "summaries": {
        "kicker": "Summaries",
        "title": "The meeting in a paragraph.",
        "body": "Generate a brief, balanced or detailed summary from the transcript — with FrameLy AI Cloud (the default), FrameLy’s local models, Apple Intelligence on supported Macs, or Anthropic or OpenAI with your own key.",
        "benefit": "Share the decisions and next steps instead of a 40-minute video.",
        "alt": "FrameLy summary tab with key points and action items"
      },
      "topics": {
        "kicker": "Topics",
        "title": "A map of what was said.",
        "body": "FrameLy organizes the transcript into topics and subtopics, each with its time in the recording. Expand a topic to read its points, or click its time to play that part.",
        "benefit": "Find the part you need in a long recording at a glance.",
        "alt": "FrameLy topics tab with timestamped topics and subtopics"
      }
    }
  },
  "ai": {
    "eyebrow": "Artificial intelligence",
    "title": "Three ways to think. You decide where.",
    "subtitle": "Transcription always happens on your computer. For summaries and topics, pick the engine that fits the recording — and switch any time in Settings.",
    "modes": {
      "local": {
        "name": "FrameLy AI Local",
        "label": "On your computer",
        "tag": "On this computer · works offline",
        "body": "Open models downloaded by the app and run on your computer. No account, no key, no per-use cost. After the models are downloaded, transcription, summaries and topics work without internet.",
        "points": [
          "Speech: 3 precision levels (181 MB to 1 GB)",
          "Language: 3 profiles (2.6 to 5.3 GB), suggested from your RAM",
          "Downloads verified with SHA-256; pause, resume or remove any time",
          "“Local only” mode blocks every connection"
        ]
      },
      "cloud": {
        "name": "In the cloud",
        "label": "FrameLy AI Cloud · Anthropic · OpenAI",
        "tag": "Default for summaries and topics",
        "body": "Summaries and topics generated on servers instead of your computer. Only the transcript text is sent — never the audio or video — and only after you allow it.",
        "options": [
          {
            "name": "FrameLy AI Cloud",
            "status": "Available · free",
            "body": "FrameLy’s own cloud service, and the default for summaries and topics: free, no account and no key, with a daily limit per computer. A partner AI provider generates the result and may use the content to improve its services — prefer local AI for sensitive recordings. Not available in the EEA, Switzerland or the UK."
          },
          {
            "name": "Anthropic (Claude) or OpenAI",
            "status": "Available · paid, with your key",
            "body": "Use your own API key. Off by default: turn on external providers, add the key and allow each provider. Usage is billed to your account with them."
          }
        ],
        "points": [
          "Keys stored in the macOS Keychain or Windows Credential Manager",
          "“Local only” mode blocks every cloud option"
        ]
      },
      "apple": {
        "name": "Apple Intelligence",
        "label": "On your Mac",
        "tag": "macOS 26 or later",
        "body": "On a Mac with Apple Intelligence, FrameLy can write summaries and topics with Apple’s on-device foundation model. Nothing to download in FrameLy and no key.",
        "points": [
          "Requires macOS 26 or later",
          "A Mac eligible for Apple Intelligence, with it turned on in System Settings",
          "In the languages Apple Intelligence supports",
          "Not available on Windows"
        ]
      }
    },
    "table": {
      "caption": "Compare the AI options for summaries and topics",
      "feature": "Option",
      "rows": {
        "availability": "Availability",
        "where": "Where it runs",
        "connection": "Internet",
        "platforms": "Platforms",
        "hardware": "Hardware",
        "cost": "Cost",
        "setup": "Setup"
      },
      "columns": {
        "local": {
          "name": "FrameLy AI Local",
          "availability": "Available",
          "where": "On your computer",
          "connection": "Only to download the models",
          "platforms": "macOS and Windows",
          "hardware": "Apple Silicon, or an x64 CPU with AVX2; about 3 to 6 GB of free memory, depending on the profile",
          "cost": "Free",
          "setup": "Download the models in the app (3 to 6.5 GB in total)"
        },
        "cloud": {
          "name": "FrameLy AI Cloud",
          "availability": "Available, except in the EEA, Switzerland and the UK",
          "where": "FrameLy’s service and a partner AI provider",
          "connection": "Required",
          "platforms": "macOS and Windows",
          "hardware": "Any computer that runs FrameLy",
          "cost": "Free, with a daily limit",
          "setup": "Default; allow it on first use, no key"
        },
        "external": {
          "name": "Anthropic / OpenAI",
          "availability": "Available",
          "where": "Anthropic’s or OpenAI’s servers",
          "connection": "Required",
          "platforms": "macOS and Windows",
          "hardware": "Any computer that runs FrameLy",
          "cost": "Paid per use, to the provider",
          "setup": "Turn on, add your API key, allow the provider"
        },
        "apple": {
          "name": "Apple Intelligence",
          "availability": "macOS 26 or later",
          "where": "On your Mac",
          "connection": "Not needed",
          "platforms": "macOS 26 or later",
          "hardware": "Mac eligible for Apple Intelligence",
          "cost": "Free",
          "setup": "Turn on Apple Intelligence in System Settings"
        }
      },
      "note": "Transcription itself always uses FrameLy’s local speech models, whatever mode you choose for summaries and topics."
    },
    "figure": "The model manager in FrameLy’s Settings: install, check or remove each model. Sizes and licenses come from the app.",
    "figureAlt": "FrameLy model manager listing transcription and language models with size, memory and license"
  },
  "how": {
    "eyebrow": "How it works",
    "title": "From the first shortcut to the summary.",
    "steps": [
      {
        "title": "Install",
        "body": "Download the installer for your system and open FrameLy. It lives in the menu bar or system tray."
      },
      {
        "title": "Allow access",
        "body": "Grant screen recording and, if you want it, microphone access. FrameLy shows what each permission is for."
      },
      {
        "title": "Capture or record",
        "body": "Use a shortcut or the quick panel to take a screenshot or record the screen, a window or a region."
      },
      {
        "title": "Choose your AI",
        "body": "Summaries and topics use FrameLy AI Cloud once you allow it. Prefer your computer? Download the local models in one step, or pick Apple Intelligence or your own key."
      },
      {
        "title": "Transcribe",
        "body": "Send a recording or any audio or video file to Transcriptions. Long files resume where they stopped."
      },
      {
        "title": "Summarize and share",
        "body": "Generate the summary and topics, edit them, and export the transcript as TXT, SRT, VTT, Markdown or PDF."
      }
    ]
  },
  "uses": {
    "eyebrow": "Use cases",
    "title": "Made for everyday work.",
    "items": [
      {
        "title": "Bug reports",
        "body": "A screenshot with an arrow and a blurred email, or a short recording of the steps."
      },
      {
        "title": "Meeting notes",
        "body": "Record a call with system audio and microphone, then get the summary and next steps."
      },
      {
        "title": "Tutorials and demos",
        "body": "Record a walkthrough and export subtitles in SRT or VTT."
      },
      {
        "title": "Classes and talks",
        "body": "Turn a long recording into topics you can jump between."
      },
      {
        "title": "Design review",
        "body": "Point at details with shapes and text, right on the capture."
      },
      {
        "title": "Sensitive content",
        "body": "Keep everything on your computer with local AI and “Local only” mode."
      }
    ]
  },
  "demo": {
    "eyebrow": "See it in action",
    "title": "A real recording, transcribed and summarized by FrameLy.",
    "caption": "Captured from FrameLy running on macOS. The meeting audio is synthesized speech."
  },
  "download": {
    "eyebrow": "Download",
    "title": "Get FrameLy.",
    "subtitle": "Free for macOS and Windows. Pick your system — your choice is remembered.",
    "choose": "Operating system",
    "detectedHint": "We detected {os}.",
    "notDetectedHint": "We couldn’t tell your system. Choose one:",
    "button": "Download for {os}",
    "version": "Version",
    "released": "Released",
    "arch": "Architecture",
    "format": "Format",
    "size": "Size",
    "checksum": "SHA-256",
    "checksums": "All checksums",
    "requirements": "Requirements",
    "install": "How to install",
    "previous": "Previous versions",
    "noPrevious": "No previous versions yet.",
    "changelog": "What’s new",
    "allReleases": "All releases on GitHub",
    "portable": "Portable (zip)",
    "none": "There’s no published installer for {os} yet. Follow the repository to be notified when one is released.",
    "unavailableArch": "Not available yet",
    "archNames": {
      "arm64": "Apple Silicon (arm64)",
      "x64": "x64 (Intel/AMD 64-bit)",
      "universal": "Universal"
    },
    "formats": {
      "dmg": "Disk image (.dmg)",
      "exe": "Installer (.exe, NSIS)",
      "msi": "Installer (.msi)",
      "zip": "Portable (.zip)"
    },
    "req": {
      "macos": [
        "macOS 14 Sonoma or later",
        "Mac with Apple silicon (M1 or later)",
        "Intel Macs: no build yet",
        "Apple Intelligence features: macOS 26 or later"
      ],
      "windows": [
        "Windows 10 22H2 or Windows 11",
        "x64 (64-bit Intel or AMD) — ARM64: no build yet",
        "Microsoft Edge WebView2 (included in Windows 11; the installer can set it up)",
        "Local AI: CPU with AVX2"
      ]
    },
    "steps": {
      "macos": [
        "Open the .dmg and drag FrameLy to Applications.",
        "This version isn’t notarized by Apple yet. The first time, open FrameLy, then go to System Settings › Privacy & Security and click “Open Anyway”.",
        "Allow Screen Recording (and Microphone, if you use it) when FrameLy asks."
      ],
      "windows": [
        "Run the -setup.exe installer. It installs for your user only — no administrator needed.",
        "The installer isn’t code-signed yet, so Windows SmartScreen may warn you. Click “More info”, then “Run anyway”.",
        "FrameLy starts in the system tray. Use the quick panel or Ctrl+Alt+Shift+Space."
      ]
    },
    "verify": "Compare the file’s SHA-256 with the value above: shasum -a 256 <file> on macOS, or Get-FileHash <file> in PowerShell.",
    "windowsNote": "Windows: full-screen recording has been tested on real hardware; window and region recording, pause and audio, and local AI are still being verified on more Windows PCs."
  },
  "faq": {
    "eyebrow": "FAQ",
    "title": "Questions, answered.",
    "more": "All questions",
    "items": [
      {
        "q": "Is FrameLy free?",
        "a": "Yes. FrameLy is free and open source under the MIT license. FrameLy AI Cloud is free with a daily limit, and local AI and Apple Intelligence are free too; cloud providers you connect with your own key charge you directly."
      },
      {
        "q": "Does my audio or video leave my computer?",
        "a": "Not for transcription: it always runs on your computer. Summaries and topics use FrameLy AI Cloud by default, so the transcript text — never the audio or video — is sent to it after you allow it. Choose FrameLy AI Local or Apple Intelligence to keep everything on your computer, or turn on “Local only”."
      },
      {
        "q": "What is FrameLy AI Cloud?",
        "a": "FrameLy’s free cloud service for summaries and topics, and the default choice. There’s no account or key; each computer has a daily limit. The transcript text is sent to it and a partner AI provider generates the result; that provider may use the content to improve its services, so the app asks before the first use and you can switch to local AI any time. It isn’t available in the EEA, Switzerland or the UK."
      },
      {
        "q": "Does FrameLy work offline?",
        "a": "Screenshots, recording and transcription work without internet once the speech model is downloaded. For offline summaries and topics, choose FrameLy AI Local (after downloading its model) or Apple Intelligence; FrameLy AI Cloud and other cloud providers need a connection."
      },
      {
        "q": "How much disk space do the AI models need?",
        "a": "The speech model takes 181 MB to 1 GB, depending on precision, and the language model 2.6 to 5.3 GB, depending on the profile. FrameLy suggests a profile from your computer’s memory, and you can remove models any time."
      },
      {
        "q": "Which Macs support Apple Intelligence in FrameLy?",
        "a": "Macs eligible for Apple Intelligence running macOS 26 or later, with Apple Intelligence turned on in System Settings. FrameLy itself runs on macOS 14 or later."
      },
      {
        "q": "Does FrameLy support Intel Macs or Windows on ARM?",
        "a": "Not yet. Current builds are for Apple Silicon Macs and 64-bit Intel/AMD Windows PCs."
      },
      {
        "q": "Can it tell speakers apart?",
        "a": "Not yet. On-device speech recognition in FrameLy doesn’t identify speakers."
      },
      {
        "q": "Why does macOS or Windows warn me when installing?",
        "a": "The installers aren’t signed with an Apple Developer ID or a Windows code-signing certificate yet. You can check each file against its SHA-256 checksum on the download section and on GitHub."
      },
      {
        "q": "Which languages does the app support?",
        "a": "The interface is available in English, Portuguese (Brazil), Spanish, French and German. Transcription detects the spoken language automatically."
      }
    ]
  },
  "footer": {
    "tagline": "Screen recording, screenshots and on-device transcription for macOS and Windows.",
    "product": "Product",
    "resources": "Resources",
    "legal": "Legal",
    "github": "Source code",
    "releases": "Releases",
    "issues": "Report an issue",
    "changelog": "Changelog",
    "faq": "FAQ",
    "privacy": "Privacy",
    "terms": "Terms of use",
    "license": "License",
    "rights": "© {year} FrameLy. Released under the MIT license.",
    "trademarks": "Apple, macOS and Apple Intelligence are trademarks of Apple Inc. Windows is a trademark of Microsoft. Claude and Anthropic are trademarks of Anthropic. OpenAI is a trademark of OpenAI."
  },
  "pages": {
    "pending": "Draft — pending legal review. This text describes how FrameLy works today; it is not legal advice.",
    "updated": "Last updated: {date}",
    "back": "Back to home",
    "toc": "On this page",
    "notFound": {
      "title": "Page not found",
      "body": "The page you’re looking for doesn’t exist or was moved."
    },
    "changelog": {
      "title": "Changelog",
      "description": "Every FrameLy version, with what’s new, improvements and fixes.",
      "intro": "All notable changes to FrameLy. Versions follow Semantic Versioning.",
      "added": "New",
      "changed": "Improvements",
      "fixed": "Fixes",
      "known": "Known limitations",
      "assets": "Downloads",
      "release": "Release on GitHub",
      "empty": "No published versions yet."
    },
    "faq": {
      "title": "Frequently asked questions",
      "description": "Answers about FrameLy: price, privacy, AI models, compatibility and installation."
    },
    "license": {
      "title": "License",
      "description": "FrameLy is open source under the MIT license.",
      "intro": "FrameLy’s source code is released under the MIT license. The AI models FrameLy downloads have their own licenses, listed below.",
      "models": "Models downloaded by FrameLy",
      "modelsBody": "Speech recognition uses Whisper models (MIT) through whisper.cpp. Summaries and topics with local AI use open language models from the Gemma and Qwen families, distributed under Apache 2.0. Each model’s license is also shown in the app’s model manager.",
      "thirdParty": "Third-party software",
      "thirdPartyBody": "FrameLy is built with Tauri, Rust, Svelte, whisper.cpp and llama.cpp, among other open-source projects, each under its own license.",
      "full": "Full license text"
    },
    "privacy": {
      "title": "Privacy Policy",
      "description": "How FrameLy handles screenshots, recordings, transcripts and data sent to AI providers.",
      "sections": [
        {
          "h": "Summary",
          "p": [
            "FrameLy is a desktop app. It has no account, no sign-up and no analytics or advertising trackers. Your screenshots, recordings and transcripts are stored on your computer, in folders you choose.",
            "Data only leaves your computer when you use a feature that needs the internet: downloading AI models, or a cloud AI service you allowed — FrameLy AI Cloud (the default for summaries and topics), Anthropic or OpenAI."
          ]
        },
        {
          "h": "Screenshots and recordings",
          "p": [
            "Captures are saved as files in the folders set in Settings (by default, the Pictures and Movies folders). FrameLy keeps a local index of them for the library. They are not uploaded anywhere by FrameLy."
          ]
        },
        {
          "h": "Transcription (always local)",
          "p": [
            "Speech recognition runs on your computer with downloaded models. Audio and video are never sent to a server for transcription. Transcripts, summaries and topics are stored in a local database on your computer, and you can delete them in the app."
          ]
        },
        {
          "h": "Local AI and Apple Intelligence",
          "p": [
            "With FrameLy’s local AI, summaries and topics are generated on your computer. With Apple Intelligence, they are generated by Apple’s on-device model through macOS; FrameLy sends nothing to Apple servers for this. Apple’s own policies apply to Apple Intelligence."
          ]
        },
        {
          "h": "Cloud AI (optional)",
          "p": [
            "FrameLy AI Cloud is FrameLy’s own cloud service and the default for summaries and topics. Only after you allow it, FrameLy sends it the transcript text and the requested task, and a partner AI provider generates the result. Your computer is identified by an anonymous key created on it — no account. The service doesn’t keep the content: it stores the anonymous keys and daily usage counts. The partner provider may use the content to improve its services; the app says so before the first use, so prefer local AI for sensitive or confidential recordings. The service isn’t offered in the European Economic Area, Switzerland or the United Kingdom.",
            "If you turn on external providers, add your own API key and allow a provider, FrameLy sends the transcript text (not the audio or video) and the instructions for the summary or topics directly to that provider — Anthropic or OpenAI. That provider’s terms and privacy policy apply, including how long it keeps data. Usage is billed to your account with them.",
            "API keys and FrameLy AI Cloud’s anonymous key are stored in the macOS Keychain or the Windows Credential Manager, not in plain files. The “Local only” setting blocks all of these connections."
          ]
        },
        {
          "h": "Model downloads",
          "p": [
            "When you download AI models, your computer connects to the servers hosting them (such as Hugging Face). Those servers see a normal download request, including your IP address. Each file is checked against a known SHA-256 hash."
          ]
        },
        {
          "h": "This website",
          "p": [
            "This site is static and hosted on GitHub Pages. It uses no cookies and no analytics. Your language and operating-system choices are saved only in your browser’s local storage. GitHub may process technical data, such as IP addresses, to serve the site and downloads — see GitHub’s privacy statement."
          ]
        },
        {
          "h": "Your control",
          "p": [
            "You can delete captures, transcripts and models from the app or your file system at any time. Uninstalling FrameLy removes the app; files in your capture folders stay until you delete them."
          ]
        },
        {
          "h": "Contact",
          "p": [
            "Questions about privacy: open an issue in the FrameLy repository on GitHub."
          ]
        }
      ]
    },
    "terms": {
      "title": "Terms of Use",
      "description": "The terms for using FrameLy and this website.",
      "sections": [
        {
          "h": "The software",
          "p": [
            "FrameLy is provided free of charge under the MIT license. You can use, copy, modify and distribute it under that license’s terms."
          ]
        },
        {
          "h": "No warranty",
          "p": [
            "FrameLy is provided “as is”, without warranty of any kind. Transcripts, summaries and topics are generated automatically and can contain mistakes; check important information before relying on it."
          ]
        },
        {
          "h": "Your content and responsibility",
          "p": [
            "You are responsible for what you capture, record and process. Make sure you have the right and, where required, the consent of the people involved to record screens, audio and conversations, and to send transcripts to third-party AI providers."
          ]
        },
        {
          "h": "Third-party services",
          "p": [
            "Cloud AI providers (Anthropic, OpenAI), Apple Intelligence and model hosting services are run by third parties under their own terms. Using them with FrameLy is optional, and their charges are between you and them."
          ]
        },
        {
          "h": "Downloads",
          "p": [
            "Download FrameLy only from the official GitHub repository or this website, and check the SHA-256 checksum published with each release."
          ]
        },
        {
          "h": "Changes",
          "p": [
            "These terms may change as FrameLy evolves. The date at the top shows the latest version."
          ]
        }
      ]
    }
  }
};
