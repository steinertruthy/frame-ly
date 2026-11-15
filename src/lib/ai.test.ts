import { describe, expect, it } from 'vitest';
import { highlight, languageModelId, needsConsent, resolvePrecision, roughDuration, segmentAt, setupPlan, speechModelId } from './ai';
import type { ModelStatus } from '$lib/ipc/ai';
import type { IntelligenceSettings } from '$lib/ipc/types';

const model = (id: string, state: ModelStatus['state'], size: number, received = 0): ModelStatus => ({
  id,
  kind: id.startsWith('speech') ? 'speech' : 'language',
  version: 1,
  file: `${id}.bin`,
  size,
  license: 'MIT',
  ramMb: 1,
  context: 0,
  state,
  received,
  installedVersion: state === 'installed' ? 1 : null,
  diskBytes: 0,
  error: null,
  inUse: false,
});

describe('model mapping', () => {
  it('maps precision and profile to catalog ids', () => {
    expect(speechModelId('fast')).toBe('speech-fast');
    expect(speechModelId('max')).toBe('speech-max');
    expect(languageModelId('balanced')).toBe('language-balanced');
    expect(resolvePrecision('auto', 'max')).toBe('max');
    expect(resolvePrecision('fast', 'max')).toBe('fast');
  });

  it('plans the first-use download', () => {
    const models = [model('speech-balanced', 'paused', 100, 40), model('language-balanced', 'missing', 300), model('speech-max', 'missing', 999)];
    const plan = setupPlan(models, 'speech-balanced', 'language-balanced');
    expect(plan.models.map((m) => m.id)).toEqual(['speech-balanced', 'language-balanced']);
    expect(plan.remainingBytes).toBe(60 + 300);
    expect(plan.done).toBe(false);
    expect(setupPlan([model('speech-balanced', 'installed', 1), model('language-balanced', 'update', 1)], 'speech-balanced', 'language-balanced').done).toBe(true);
  });
});

describe('transcript navigation', () => {
  const segs = [{ startMs: 0 }, { startMs: 4000 }, { startMs: 9000 }];
  it('finds the segment playing at a time', () => {
    expect(segmentAt(segs, 0)).toBe(0);
    expect(segmentAt(segs, 4500)).toBe(1);
    expect(segmentAt(segs, 60_000)).toBe(2);
    expect(segmentAt([], 10)).toBe(-1);
  });
});

describe('highlighting', () => {
  it('marks search hits and whole-word keywords', () => {
    const p = highlight('O lançamento do Lumen: lançamentos e Lumen.', 'do', ['Lumen', 'lançamento']);
    expect(p.filter((x) => x.mark === 'key').map((x) => x.text)).toEqual(['lançamento', 'Lumen', 'Lumen']);
    expect(p.filter((x) => x.mark === 'match').map((x) => x.text)).toEqual(['do']);
    expect(p.map((x) => x.text).join('')).toBe('O lançamento do Lumen: lançamentos e Lumen.');
    expect(highlight('texto', '', [])).toEqual([{ text: 'texto' }]);
    expect(highlight('a (b)', '(b', []).find((x) => x.mark)?.text).toBe('(b');
  });
});

describe('privacy and estimates', () => {
  const s = { consents: ['anthropic'] } as unknown as IntelligenceSettings;
  it('asks consent only for external providers not yet allowed', () => {
    expect(needsConsent('framely', s)).toBe(false);
    expect(needsConsent('apple', s)).toBe(false);
    expect(needsConsent('anthropic', s)).toBe(false);
    expect(needsConsent('openai', s)).toBe(true);
    expect(needsConsent('cloud', s)).toBe(true);
    expect(needsConsent('cloud', { ...s, consents: ['cloud'] })).toBe(false);
  });
  it('rounds durations for display', () => {
    expect(roughDuration(4_200)).toEqual({ unit: 'sec', n: 5 });
    expect(roughDuration(42_000)).toEqual({ unit: 'sec', n: 45 });
    expect(roughDuration(61_000)).toEqual({ unit: 'min', n: 2 });
    expect(roughDuration(5_400_000)).toEqual({ unit: 'hour', n: 1.5 });
  });
});
