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

export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Display {
  id: string;
  name: string;
  bounds: Rect;
  pixelWidth: number;
  pixelHeight: number;
  scaleFactor: number;
  refreshRate: number | null;
  isPrimary: boolean;
}

export interface WindowInfo {
  id: string;
  title: string;
  appName: string;
  appId: string | null;
  pid: number;
  bounds: Rect;
}

export interface AudioDevice {
  id: string;
  name: string;
  isDefault: boolean;
}

export type Permission = 'screen' | 'microphone';
export type PermissionState = 'granted' | 'denied' | 'notDetermined' | 'restricted';
export type Permissions = Record<Permission, PermissionState>;

export interface AppError {
  code: 'io' | 'json' | 'invalid_settings' | 'runtime' | 'permission_denied' | 'platform' | string;
  message: string;
}
