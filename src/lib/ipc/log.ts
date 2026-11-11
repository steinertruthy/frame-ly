import { invoke } from '@tauri-apps/api/core';

type Level = 'info' | 'warn' | 'error';

/** Sends a message to the Rust structured log (falls back to the console outside Tauri). */
export function logUi(level: Level, message: string): void {
  invoke('log_ui', { level, message }).catch(() => console[level](message));
}

function describe(e: unknown): string {
  return e instanceof Error ? `${e.name}: ${e.message}\n${e.stack ?? ''}` : String(e);
}

/** Routes uncaught errors and rejections from this window into the app log. */
export function installErrorForwarding(): void {
  window.addEventListener('error', (e) => logUi('error', describe(e.error ?? e.message)));
  window.addEventListener('unhandledrejection', (e) => logUi('error', describe(e.reason)));
}
