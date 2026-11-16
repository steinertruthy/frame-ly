import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/** Mirrors `updates::Status` in Rust. */
export type UpdateStatus =
  | { state: 'idle' }
  | { state: 'checking' }
  | { state: 'upToDate'; checkedAt: number }
  | { state: 'available'; version: string; notes: string | null; date: string | null; dismissed: boolean }
  | { state: 'downloading'; version: string; received: number; total: number | null }
  | { state: 'waitingForRecording'; version: string }
  | { state: 'installing'; version: string }
  | { state: 'error'; reason: 'local_only' | 'check_failed' | 'download_failed' | 'install_failed'; detail: string | null };

export const updatesIpc = {
  status: (): Promise<UpdateStatus> => invoke('updates_status'),
  check: (): Promise<UpdateStatus> => invoke('updates_check'),
  install: (): Promise<void> => invoke('updates_install'),
  dismiss: (): Promise<void> => invoke('updates_dismiss'),
  onStatus: (cb: (s: UpdateStatus) => void): Promise<UnlistenFn> => listen<UpdateStatus>('updates://status', (e) => cb(e.payload)),
};
