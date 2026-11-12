import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError,
  AppInfo,
  AudioDevice,
  CaptureInfo,
  CaptureRequest,
  OverlayInit,
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
  takeScreenshot: (request: CaptureRequest) => invoke<void>('take_screenshot', { request }),
  overlayInit: (displayId: string) => invoke<OverlayInit>('overlay_init', { displayId }),
  finishRegion: (displayId: string, r: { x: number; y: number; width: number; height: number }, viewport: { width: number; height: number }) =>
    invoke<void>('finish_region', {
      displayId,
      selection: { ...r, viewportWidth: viewport.width, viewportHeight: viewport.height },
    }),
  finishWindow: (windowId: string) => invoke<void>('finish_window', { windowId }),
  cancelCapture: () => invoke<void>('cancel_capture'),
  lastCapture: () => invoke<CaptureInfo | null>('last_capture'),
  copyLastCapture: () => invoke<void>('copy_last_capture'),
  saveLastCapture: () => invoke<CaptureInfo>('save_last_capture'),
  revealLastCapture: () => invoke<void>('reveal_last_capture'),
  openLastCapture: () => invoke<void>('open_last_capture'),
  closePreview: () => invoke<void>('close_preview'),
};

export const events = {
  onSettingsChanged: (cb: (s: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>('settings://changed', (e) => cb(e.payload)),
  onCaptureFailed: (cb: (e: AppError) => void): Promise<UnlistenFn> =>
    listen<AppError>('capture://failed', (e) => cb(e.payload)),
  onPreviewRefresh: (cb: () => void): Promise<UnlistenFn> => listen('capture://preview-refresh', () => cb()),
};

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}
