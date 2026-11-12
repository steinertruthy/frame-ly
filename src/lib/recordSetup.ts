/** Recording setup helpers shared by Home and the quick panel. Pure, unit-tested. */
import type { RecordRequest, RecordTarget, Settings } from '$lib/ipc/types';
import type { MessageKey } from '$lib/i18n/en';

export type AudioMode = 'both' | 'mic' | 'system' | 'none';

export function audioMode(a: Settings['audio']): AudioMode {
  if (a.microphone && a.systemAudio) return 'both';
  if (a.microphone) return 'mic';
  if (a.systemAudio) return 'system';
  return 'none';
}

export function applyAudioMode(a: Settings['audio'], mode: AudioMode): void {
  a.microphone = mode === 'both' || mode === 'mic';
  a.systemAudio = mode === 'both' || mode === 'system';
}

export function recordRequest(target: RecordTarget, displayId: string | null = null): RecordRequest {
  if (target === 'screen') return { kind: 'display', displayId };
  return { kind: 'select', mode: target };
}

/** Message keys that describe what will be recorded, e.g. "Entire screen · Microphone and computer sound". */
export function summaryKeys(target: RecordTarget, mode: AudioMode): [MessageKey, MessageKey] {
  const t: Record<RecordTarget, MessageKey> = { screen: 'rec.target.screen', window: 'rec.target.window', region: 'rec.target.region' };
  const a: Record<AudioMode, MessageKey> = { both: 'rec.audio.both', mic: 'rec.audio.mic', system: 'rec.audio.system', none: 'rec.audio.none' };
  return [t[target], a[mode]];
}
