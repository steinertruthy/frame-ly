import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError,
  AppInfo,
  AudioDevice,
  CaptureInfo,
  CaptureRequest,
  LibraryItem,
  LibraryOverview,
  LibraryQuery,
  OverlayInit,
  RecordRequest,
  RecordingInfo,
  RecordingStatus,
  RecordingWarning,
  ShortcutStatus,
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
  resetShortcuts: () => invoke<Settings>('reset_shortcuts'),
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
  /** Annotated screenshot from the selection overlay (raw PNG body). */
  finishAnnotated: (png: Uint8Array, intent: 'copy' | 'save') =>
    invoke<void>('finish_annotated', png, { headers: { 'x-framely-intent': intent } }),
  cancelCapture: () => invoke<void>('cancel_capture'),
  lastCapture: () => invoke<CaptureInfo | null>('last_capture'),
  copyLastCapture: () => invoke<void>('copy_last_capture'),
  saveLastCapture: () => invoke<CaptureInfo>('save_last_capture'),
  revealLastCapture: () => invoke<void>('reveal_last_capture'),
  openLastCapture: () => invoke<void>('open_last_capture'),
  closePreview: () => invoke<void>('close_preview'),
  startRecording: (request: RecordRequest) => invoke<void>('start_recording', { request }),
  pauseRecording: () => invoke<void>('pause_recording'),
  resumeRecording: () => invoke<void>('resume_recording'),
  stopRecording: () => invoke<void>('stop_recording'),
  cancelCountdown: () => invoke<void>('cancel_countdown'),
  setRecordingAudioMuted: (source: 'system' | 'microphone', muted: boolean) =>
    invoke<void>('set_recording_audio_muted', { source, muted }),
  setRecordingControlsVisible: (visible: boolean) => invoke<void>('set_recording_controls_visible', { visible }),
  recordingControlsHidden: () => invoke<boolean>('recording_controls_hidden'),
  recordingStatus: () => invoke<RecordingStatus>('recording_status'),
  recoveredRecordings: () => invoke<string[]>('recovered_recordings'),
  lastRecording: () => invoke<RecordingInfo | null>('last_recording'),
  revealLastRecording: () => invoke<void>('reveal_last_recording'),
  openLastRecording: () => invoke<void>('open_last_recording'),
  copyLastRecording: () => invoke<void>('copy_last_recording'),
  micTestStart: (deviceId: string | null) => invoke<void>('mic_test_start', { deviceId }),
  micTestLevel: () => invoke<number>('mic_test_level'),
  micTestStop: () => invoke<void>('mic_test_stop'),
  panelHide: () => invoke<void>('panel_hide'),
  panelResize: (height: number) => invoke<void>('panel_resize', { height }),
  startupShouldShow: () => invoke<boolean>('startup_should_show'),
  storageCache: () => invoke<{ bytes: number }>('storage_cache'),
  storageCleanup: () => invoke<{ bytes: number }>('storage_cleanup'),
  previewSound: () => invoke<void>('preview_sound'),
  showMainWindow: (page: 'home' | 'library' | 'settings' | null = null) => invoke<void>('show_main_window', { page }),
  shortcutStatus: () => invoke<ShortcutStatus>('shortcut_status'),
  suspendShortcuts: () => invoke<void>('suspend_shortcuts'),
  resumeShortcuts: () => invoke<void>('resume_shortcuts'),
  libraryList: (query: LibraryQuery) => invoke<LibraryItem[]>('library_list', { query }),
  libraryOverview: () => invoke<LibraryOverview>('library_overview'),
  libraryRefresh: () => invoke<void>('library_refresh'),
  libraryRename: (id: number, name: string) => invoke<LibraryItem>('library_rename', { id, name }),
  libraryTrash: (id: number) => invoke<void>('library_trash', { id }),
  libraryCopy: (id: number) => invoke<void>('library_copy', { id }),
  libraryReveal: (id: number) => invoke<void>('library_reveal', { id }),
  libraryOpen: (id: number) => invoke<void>('library_open', { id }),
  editorOpen: (path: string) => invoke<void>('editor_open', { path }),
  editorSource: () => invoke<{ path: string; name: string }>('editor_source'),
  /** Raw PNG bytes travel as the request body (not JSON). */
  editorSave: (png: Uint8Array) => invoke<{ path: string; name: string }>('editor_save', png),
  editorCopy: (png: Uint8Array) => invoke<void>('editor_copy', png),
};

export const events = {
  onSettingsChanged: (cb: (s: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>('settings://changed', (e) => cb(e.payload)),
  onCaptureFailed: (cb: (e: AppError) => void): Promise<UnlistenFn> =>
    listen<AppError>('capture://failed', (e) => cb(e.payload)),
  onPreviewRefresh: (cb: (kind: 'image' | 'video') => void): Promise<UnlistenFn> =>
    listen<'image' | 'video'>('capture://preview-refresh', (e) => cb(e.payload)),
  onRecordingStatus: (cb: (s: RecordingStatus) => void): Promise<UnlistenFn> =>
    listen<RecordingStatus>('recording://status', (e) => cb(e.payload)),
  onRecordingWarning: (cb: (code: RecordingWarning) => void): Promise<UnlistenFn> =>
    listen<RecordingWarning>('recording://warning', (e) => cb(e.payload)),
  onPanelShown: (cb: () => void): Promise<UnlistenFn> => listen('panel://shown', () => cb()),
  onLibraryChanged: (cb: () => void): Promise<UnlistenFn> => listen('library://changed', () => cb()),
  onEditorLoad: (cb: () => void): Promise<UnlistenFn> => listen('editor://load', () => cb()),
  onNavigate: (cb: (page: string) => void): Promise<UnlistenFn> => listen<string>('main://navigate', (e) => cb(e.payload)),
  onShortcutStatus: (cb: (s: ShortcutStatus) => void): Promise<UnlistenFn> =>
    listen<ShortcutStatus>('shortcuts://status', (e) => cb(e.payload)),
  onRecordingFailed: (cb: (e: AppError) => void): Promise<UnlistenFn> =>
    listen<AppError>('recording://failed', (e) => cb(e.payload)),
};

export function isAppError(e: unknown): e is AppError {
  return typeof e === 'object' && e !== null && 'code' in e && 'message' in e;
}
