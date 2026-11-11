import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AppError, AppInfo, Folder, Settings } from './types';

export const commands = {
  getSettings: () => invoke<Settings>('get_settings'),
  updateSettings: (settings: Settings) => invoke<Settings>('update_settings', { settings }),
  appInfo: () => invoke<AppInfo>('app_info'),
  openFolder: (folder: Folder) => invoke<void>('open_folder', { folder }),
};

export const events = {
  onSettingsChanged: (cb: (s: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>('settings://changed', (e) => cb(e.payload)),
};

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}
