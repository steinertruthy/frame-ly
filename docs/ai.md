# FrameLy AI: transcription, summaries and topics

Code: `src-tauri/src/intelligence/` (use cases), `src-tauri/src/core/transcript*.rs`,
`core/pdf.rs`, `core/resample.rs` (pure, tested), `src-tauri/llm-worker/` (sidecar),
`native/macos/FrameLyKit/Sources/FrameLyKit/Intelligence.swift` (Apple), `cloud/` (service, see
[cloud.md](cloud.md)). UI: `src/windows/main/pages/Transcripts.svelte`,
`src/lib/components/ai/`, `src/lib/stores/ai.svelte.ts`, `src/lib/ipc/ai.ts`.

The UI talks only to `commands::intelligence` (`ai_*`, `transcript*`, `generation_start`,
`provider_*`, `cloud_status`, `media_probe`); it never reaches runtimes directly.

## Pipeline

```
media (recording, imported video or audio)
  → decode natively to 16 kHz mono f32 PCM in a private temp file
      (macOS: AVAssetReader in Transcode.swift · Windows: IMFSourceReader + core::resample)
  → whisper.cpp (whisper-rs) in ~5 min blocks cut at the quietest moment (speech.rs)
      → each block saved to transcripts.sqlite3 before the next one (resumable)
  → summary / topics (generation.rs): shared prompts + strict JSON schema,
      map-reduce when the transcript doesn't fit the provider's context
      → provider Runner (providers.rs / cloud.rs / llm.rs)
  → export TXT, SRT, VTT, Markdown, PDF (core/transcript.rs, core/pdf.rs)
```

- **Jobs** (`intelligence/mod.rs`): one at a time on a background thread (transcription and
  generation compete for CPU/GPU), with real progress (`ai://job` events), cancellation, and
  resumption. On restart, a job that was running becomes "interrupted" and continues from the next
  block (the temp PCM is reused when complete). The same file with the same settings is never
  processed twice. Events: `ai://job`, `ai://models`, `ai://transcript`, `ai://notice`.
- **Storage:** `<app data>/transcripts.sqlite3` (`core/transcripts.rs`), linked to the media file by
  path + size + modification time. A transcript has its own editable title; the media file is never
  renamed.
- **Progress** comes from the decoder, the whisper callback and prompt evaluation; remaining time
  from the measured throughput of the current run. Loading a model shows activity only.
- **Estimates** (`estimate.rs`): median of the last runs on this machine (table `perf`); without
  history, reference rates marked "approximate" in the UI.
- **Diarization** is not supported locally (whisper.cpp only has English `tinydiarize`, and
  recordings have one mixed track); the setting shows as unavailable with an explanation. Don't
  simulate it.

## Two runtimes, two processes

whisper.cpp runs inside the app. llama.cpp runs in **`framely-llm`** (`src-tauri/llm-worker`,
bundled as a Tauri sidecar, `externalBin` in `tauri.conf.json`), one process per generation job,
JSON lines over stdin/stdout (protocol in the header of `llm-worker/src/main.rs`; client in
`intelligence/llm.rs`). Reasons:

- whisper.cpp and llama.cpp vendor different `ggml` versions (0.9 vs 0.26); linked into one binary
  the linker silently mixes symbols — undefined behavior. **Never link llama.cpp into the app.**
- The model's memory is returned when the job ends; cancelling kills the process instantly; a model
  crash doesn't take the app down.

Known dependency bug: `whisper-rs 0.16` `set_abort_callback_safe` with a non-boxed closure reads
garbage and aborts every transcription; FrameLy passes a `Box<dyn FnMut() -> bool>` (commented in
`speech.rs`).

## Models and hardware

- **Catalog** (`catalog.rs`): each `ModelSpec` has a stable `id` (UI label `ai.model.<id>`), version,
  file name, URL, exact size, SHA-256, license, RAM and context.

  | id | Model | Size | License |
  |---|---|---|---|
  | `speech-fast` | whisper small q5_1 | 190 MB | MIT |
  | `speech-balanced` | whisper large-v3-turbo q5_0 | 574 MB | MIT |
  | `speech-max` | whisper large-v3 q5_0 + beam search | 1.08 GB | MIT |
  | `language-basic` | Gemma 4 E2B | 2.8 GB | Apache 2.0 |
  | `language-balanced` | Qwen3.5-4B | 3.0 GB | Apache 2.0 |
  | `language-advanced` | Qwen3.5-9B | 5.7 GB | Apache 2.0 |

  Chosen on a real pt-BR meeting: Qwen3.5-2B and Granite 4.2 3B were rejected because they invented
  decisions and owners.
- **Model manager** (`models.rs`): downloads to `<app data>/models` (never in the installer),
  resumable with HTTP Range, pausable, cancellable; SHA-256 verification, free-space checks,
  repair, `manifest.json`. **To ship a new model version**, bump `version` with the new file, size
  and hash: people with the previous one are offered the update.
- **Hardware** (`hardware.rs`): RAM, Apple Silicon (Metal), AVX2, GPU/VRAM (informative). Only the
  Apple GPU is used; Windows runs on the CPU and needs AVX2. Drives the recommended profile.

## Providers

Transcription is always local (whisper.cpp). Summaries and topics go through a provider
(`ProviderId` in `core/settings.rs`; info and runners in `providers.rs`):

| `ProviderId` | Name in the UI | Where | Key | Notes |
|---|---|---|---|---|
| `cloud` | FrameLy AI Cloud | FrameLy's service | none | **Default** when the build has `FRAMELY_CLOUD_URL`; external for privacy purposes |
| `framely` | FrameLy AI Local | `framely-llm` | none | Default without a Cloud URL; needs a language model |
| `apple` | Apple Intelligence | on device | none | macOS 26+, eligible devices; Foundation Models with guided generation (`@Generable`), 4096-token context → smaller parts; never on Windows |
| `anthropic` | Anthropic | external | user's | Messages API, structured output (`output_config.format`) |
| `openai` | OpenAI | external | user's | Chat Completions with strict `json_schema` |

Each provider reports `ProviderInfo` (local/external, internet, key, context, paid, availability
with a `reason`: `model_missing`, `local_only`, `external_disabled`, `no_key`, `no_consent`,
`os_version`, …). A provider implements `Runner` (`context_tokens`, `model_name`, `run`, and
optionally `generate` — the Cloud overrides it to send the structured request instead of prompts).
`intelligence/mod.rs::open_runner` builds the runner and turns unavailability into `AppError`s.
Checklist to add one: [contributing.md](contributing.md#add-an-ai-provider-summariestopics).

### FrameLy AI Cloud client (`cloud.rs`)

- Sends the structured request (transcript text with `[mm:ss]` lines, language, detail, part) —
  never prompts or media.
- Anonymous Ed25519 device identity kept in the OS keychain (account `framely-cloud-device-v2`),
  registered once with a proof-of-work challenge; every request is signed (method, path, timestamp,
  request id, body hash).
- `/v1/status` gives availability, limits and the data-use policy — **not** the provider or model.
  The app doesn't know, and must never show, which AI provider processes the text; consent text
  says "a partner AI provider" plus the data policy.
- Base URL from `FRAMELY_CLOUD_URL` (embedded at build time; also read at run time in development).
  Without it the provider is unavailable.

## Privacy rules

1. **Consent per provider** before the first external request (`intelligence.consents`); never
   implied by enabling something else.
2. **Local only** (`intelligence.localOnly`) blocks every network request — external providers,
   the Cloud, model downloads and update checks. All AI HTTP goes through `intelligence/net.rs`
   (`agent`) or `providers::external_agent`, which refuse in this mode.
3. **No silent fallback** to another provider: a provider that can't run returns an error with a
   reason; the UI explains it.
4. **Keys** only in the Keychain (`Keychain.swift`) / Credential Manager (`platform/windows/keystore.rs`),
   via `platform/keystore.rs`. Never in settings, SQLite or logs.
5. **Logs** never contain transcript text, prompts or responses.
6. Third-party models and services are listed with licenses in `src/lib/credits.ts`
   (Settings › About).
