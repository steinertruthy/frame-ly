// Mirrors of the Rust IPC types (src-tauri/src/core/settings.rs, commands/system.rs).
// Keep in sync; serde uses camelCase on the wire.

export type Language = 'system' | 'pt-BR' | 'en';
export type Theme = 'system' | 'light' | 'dark';

export interface Settings {
  version: number;
  general: { language: Language };
  appearance: { theme: Theme };
  capture: { copyToClipboard: boolean; saveToDisk: boolean; showPreview: boolean };
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
  workArea: Rect;
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

export type SelectMode = 'region' | 'window';
export type CaptureRequest = { kind: 'display'; displayId: string | null } | { kind: 'select'; mode: SelectMode };

export interface OverlayInit {
  mode: SelectMode;
  display: Display;
  frameWidth: number;
  frameHeight: number;
  windows: WindowInfo[];
}

export interface CaptureInfo {
  width: number;
  height: number;
  path: string | null;
  fileName: string | null;
  copied: boolean;
}
