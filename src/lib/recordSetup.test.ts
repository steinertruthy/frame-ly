import { describe, expect, it } from 'vitest';
import { applyAudioMode, audioMode, recordRequest, summaryKeys } from './recordSetup';

const audio = (microphone: boolean, systemAudio: boolean) => ({ microphone, systemAudio, microphoneDevice: null, systemGain: 1, microphoneGain: 1, noiseSuppression: true });

describe('record setup', () => {
  it('maps audio flags to a mode and back', () => {
    expect(audioMode(audio(true, true))).toBe('both');
    expect(audioMode(audio(true, false))).toBe('mic');
    expect(audioMode(audio(false, true))).toBe('system');
    expect(audioMode(audio(false, false))).toBe('none');
    for (const mode of ['both', 'mic', 'system', 'none'] as const) {
      const a = audio(false, false);
      applyAudioMode(a, mode);
      expect(audioMode(a)).toBe(mode);
    }
  });

  it('builds the right request per target', () => {
    expect(recordRequest('screen')).toEqual({ kind: 'display', displayId: null });
    expect(recordRequest('screen', '2')).toEqual({ kind: 'display', displayId: '2' });
    expect(recordRequest('window')).toEqual({ kind: 'select', mode: 'window' });
    expect(recordRequest('region')).toEqual({ kind: 'select', mode: 'region' });
  });

  it('summarizes what will be recorded', () => {
    expect(summaryKeys('window', 'none')).toEqual(['rec.target.window', 'rec.audio.none']);
  });
});
