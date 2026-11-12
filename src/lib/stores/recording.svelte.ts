import { commands, events } from '$lib/ipc/commands';
import type { RecordingStatus } from '$lib/ipc/types';

/** Live recording status (phase + elapsed), pushed by the backend twice a second. */
class RecordingStore {
  status = $state<RecordingStatus>({ phase: 'idle', elapsedMs: 0, target: null, audio: null });

  async init(): Promise<void> {
    this.status = await commands.recordingStatus();
    await events.onRecordingStatus((s) => (this.status = s));
  }

  get active(): boolean {
    return this.status.phase !== 'idle';
  }
}

export const recording = new RecordingStore();

export { formatBytes, formatDuration } from './format';
