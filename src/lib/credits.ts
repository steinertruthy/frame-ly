/**
 * Third-party software, models and services FrameLy is built with, as shown in
 * Settings › About › Credits. Licenses checked against each crate's / package's metadata and the
 * vendored sources (whisper.cpp and llama.cpp LICENSE files); model licenses as in the model catalog.
 */
export type CreditGroup = 'ai' | 'app' | 'libs' | 'platform';

export interface Credit {
  name: string;
  by: string;
  /** SPDX-style license, or null for parts of the operating system / hosted services. */
  license: string | null;
  url: string;
  /** Only on this platform ("macos" | "windows"). */
  only?: 'macos' | 'windows';
}

export const CREDITS: Record<CreditGroup, Credit[]> = {
  ai: [
    { name: 'whisper.cpp', by: 'ggml authors', license: 'MIT', url: 'https://github.com/ggml-org/whisper.cpp' },
    { name: 'Whisper', by: 'OpenAI', license: 'MIT', url: 'https://github.com/openai/whisper' },
    { name: 'llama.cpp', by: 'ggml authors', license: 'MIT', url: 'https://github.com/ggml-org/llama.cpp' },
    { name: 'Gemma 4 E2B', by: 'Google DeepMind', license: 'Apache-2.0', url: 'https://huggingface.co/ggml-org/gemma-4-E2B-it-GGUF' },
    { name: 'Qwen3.5 4B · 9B', by: 'Qwen Team, Alibaba Cloud', license: 'Apache-2.0', url: 'https://huggingface.co/ggml-org/Qwen3.5-4B-GGUF' },
    { name: 'Apple Foundation Models', by: 'Apple', license: null, url: 'https://developer.apple.com/documentation/foundationmodels', only: 'macos' },
    { name: 'Cloudflare Workers', by: 'Cloudflare', license: null, url: 'https://workers.cloudflare.com' },
  ],
  app: [
    { name: 'Tauri', by: 'Tauri Programme', license: 'Apache-2.0 OR MIT', url: 'https://tauri.app' },
    {
      name: 'Tauri plugins (dialog, opener, notification, global-shortcut, autostart, single-instance, updater)',
      by: 'Tauri Programme',
      license: 'Apache-2.0 OR MIT',
      url: 'https://github.com/tauri-apps/plugins-workspace',
    },
    { name: 'Svelte', by: 'Svelte contributors', license: 'MIT', url: 'https://svelte.dev' },
    { name: 'Vite', by: 'VoidZero', license: 'MIT', url: 'https://vite.dev' },
    { name: 'Lucide', by: 'Lucide contributors', license: 'ISC', url: 'https://lucide.dev' },
    { name: 'Rust', by: 'The Rust Project', license: 'MIT OR Apache-2.0', url: 'https://www.rust-lang.org' },
    { name: 'Swift', by: 'Apple and the Swift project', license: 'Apache-2.0', url: 'https://www.swift.org', only: 'macos' },
  ],
  libs: [
    { name: 'whisper-rs', by: 'tazz4843', license: 'Unlicense', url: 'https://codeberg.org/tazz4843/whisper-rs' },
    { name: 'llama-cpp-2', by: 'utilityai', license: 'MIT OR Apache-2.0', url: 'https://github.com/utilityai/llama-cpp-rs' },
    { name: 'SQLite', by: 'D. Richard Hipp', license: 'Public domain', url: 'https://sqlite.org/copyright.html' },
    { name: 'rusqlite', by: 'rusqlite contributors', license: 'MIT', url: 'https://github.com/rusqlite/rusqlite' },
    { name: 'ureq', by: 'Martin Algesten', license: 'MIT OR Apache-2.0', url: 'https://github.com/algesten/ureq' },
    { name: 'ed25519-dalek', by: 'dalek-cryptography', license: 'BSD-3-Clause', url: 'https://github.com/dalek-cryptography/curve25519-dalek' },
    { name: 'RustCrypto (sha2)', by: 'RustCrypto', license: 'MIT OR Apache-2.0', url: 'https://github.com/RustCrypto/hashes' },
    { name: 'nnnoiseless (RNNoise)', by: 'Joe Neeman, Xiph.Org', license: 'BSD-3-Clause', url: 'https://github.com/jneem/nnnoiseless' },
    { name: 'image · png', by: 'image-rs', license: 'MIT OR Apache-2.0', url: 'https://github.com/image-rs/image' },
    { name: 'Tokio', by: 'Tokio contributors', license: 'MIT', url: 'https://tokio.rs' },
    { name: 'Serde', by: 'serde-rs', license: 'MIT OR Apache-2.0', url: 'https://serde.rs' },
    { name: 'chrono', by: 'chronotope', license: 'MIT OR Apache-2.0', url: 'https://github.com/chronotope/chrono' },
    { name: 'tracing', by: 'Tokio contributors', license: 'MIT', url: 'https://github.com/tokio-rs/tracing' },
    { name: 'trash', by: 'Artur Kovacs', license: 'MIT', url: 'https://github.com/ArturKovacs/trash' },
    { name: 'windows-rs', by: 'Microsoft', license: 'MIT OR Apache-2.0', url: 'https://github.com/microsoft/windows-rs', only: 'windows' },
    { name: 'arboard', by: '1Password', license: 'MIT OR Apache-2.0', url: 'https://github.com/1Password/arboard', only: 'windows' },
  ],
  platform: [
    { name: 'ScreenCaptureKit · AVFoundation', by: 'Apple', license: null, url: 'https://developer.apple.com/documentation/screencapturekit', only: 'macos' },
    { name: 'Windows.Graphics.Capture · Media Foundation', by: 'Microsoft', license: null, url: 'https://learn.microsoft.com/windows/win32/medfound/microsoft-media-foundation-sdk', only: 'windows' },
  ],
};

/** Credits that apply to `platform` ("macos", "windows", …). */
export function creditsFor(platform: string): [CreditGroup, Credit[]][] {
  return (Object.entries(CREDITS) as [CreditGroup, Credit[]][])
    .map(([g, list]) => [g, list.filter((c) => !c.only || c.only === platform)] as [CreditGroup, Credit[]])
    .filter(([, list]) => list.length > 0);
}
