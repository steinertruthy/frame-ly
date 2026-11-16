import { updatesIpc, type UpdateStatus } from '$lib/ipc/updates';
import { logUi } from '$lib/ipc/log';

/** Update status pushed by the backend (see src-tauri/src/updates.rs). */
class UpdatesStore {
  status = $state<UpdateStatus>({ state: 'idle' });
  #started = false;

  async init(): Promise<void> {
    if (this.#started) return;
    this.#started = true;
    await updatesIpc.onStatus((s) => (this.status = s));
    this.status = await updatesIpc.status();
  }

  async check(): Promise<void> {
    this.status = await updatesIpc.check();
  }

  async install(): Promise<void> {
    try {
      await updatesIpc.install();
    } catch (e) {
      logUi('error', `update install failed: ${JSON.stringify(e)}`);
    }
  }

  dismiss(): void {
    void updatesIpc.dismiss();
  }

  /** A version is on offer (or on its way in): the notice in the main window shows. */
  get notice(): boolean {
    const s = this.status;
    return (
      (s.state === 'available' && !s.dismissed) ||
      s.state === 'downloading' ||
      s.state === 'waitingForRecording' ||
      s.state === 'installing' ||
      (s.state === 'error' && s.reason !== 'local_only' && s.reason !== 'check_failed')
    );
  }
}

export const updates = new UpdatesStore();

/** Download progress as a 0–1 fraction, or null when the size is unknown. */
export function progress(s: UpdateStatus): number | null {
  return s.state === 'downloading' && s.total ? Math.min(1, s.received / s.total) : null;
}
