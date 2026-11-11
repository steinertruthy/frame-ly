// Mirrors of the Rust IPC types (src-tauri/src/core/settings.rs, commands/system.rs).
// Keep in sync; serde uses camelCase on the wire.

export type Language = 'system' | 'pt-BR' | 'en';
export type Theme = 'system' | 'light' | 'dark';

export interface Settings {
  version: number;
  general: { language: Language };
  appearance: { theme: Theme };
  storage: { recordingsDir: string; screenshotsDir: string };
}

export interface AppInfo {
  version: string;
  platform: 'macos' | 'windows' | 'linux' | string;
  arch: string;
  logDir: string;
}

export type Folder = 'recordings' | 'screenshots' | 'logs';

export interface AppError {
  code: 'io' | 'json' | 'invalid_settings' | 'runtime' | string;
  message: string;
}
