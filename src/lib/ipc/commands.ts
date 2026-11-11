import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError,
  AppInfo,
  AudioDevice,
  Display,
  Folder,
  Permission,
  PermissionState,
  Permissions,
  Settings,
  WindowInfo,
} from './types';

export const commands = {
  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (settings: Settings) => invoke<Settings>('update_settings', { settings }),
  appInfo: () => invoke<AppInfo>('app_info'),
  openFolder: (folder: Folder) => invoke<void>('open_folder', { folder }),
  listDisplays: () => invoke<Display[]>('list_displays'),
  listWindows: () => invoke<WindowInfo[]>('list_windows'),
  listAudioInputs: () => invoke<AudioDevice[]>('list_audio_inputs'),
  getPermissions: () => invoke<Permissions>('get_permissions'),
  requestPermission: (permission: Permission) => invoke<PermissionState>('request_permission', { permission }),
  openPermissionSettings: (permission: Permission) => invoke<void>('open_permission_settings', { permission }),
  relaunch: () => invoke<void>('relaunch'),
};

export const events = {
  onSettingsChanged: (cb: (s: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>('settings://changed', (e) => cb(e.payload)),
};

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}
