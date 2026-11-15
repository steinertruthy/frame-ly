/** Pure helpers for the transcription UI (tested in ai.test.ts). */
import type { AiProfile, ModelStatus, Precision, ProviderId, Segment } from '$lib/ipc/ai';
import type { Settings } from '$lib/ipc/types';

export function speechModelId(precision: Exclude<Precision, 'auto'>): string {
  return precision === 'fast' ? 'speech-fast' : precision === 'max' ? 'speech-max' : 'speech-balanced';
}

export function languageModelId(profile: Exclude<AiProfile, 'auto'>): string {
  return profile === 'basic' ? 'language-basic' : profile === 'advanced' ? 'language-advanced' : 'language-balanced';
}

export function resolvePrecision(choice: Precision, defaultPrecision: Exclude<Precision, 'auto'>): Exclude<Precision, 'auto'> {
  return choice === 'auto' ? defaultPrecision : choice;
}

export const isReady = (m: ModelStatus | undefined): boolean => m?.state === 'installed' || m?.state === 'update';

/** Models the first-use setup installs for a profile, and how much is left to download. */
export function setupPlan(models: ModelStatus[], speechId: string, languageId: string): { models: ModelStatus[]; remainingBytes: number; done: boolean } {
  const needed = models.filter((m) => m.id === speechId || m.id === languageId);
  const remainingBytes = needed.filter((m) => !isReady(m)).reduce((n, m) => n + m.size - (m.state === 'corrupt' ? 0 : m.received), 0);
  return { models: needed, remainingBytes, done: needed.length > 0 && needed.every(isReady) };
}

/** Index of the segment playing at `ms` (last one starting at or before it), or -1. */
export function segmentAt(segments: Pick<Segment, 'startMs'>[], ms: number): number {
  let lo = 0;
  let hi = segments.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (segments[mid]!.startMs <= ms) lo = mid + 1;
    else hi = mid;
  }
  return lo - 1;
}

export interface Piece {
  text: string;
  /** "match" = search hit, "key" = keyword from the topics. */
  mark?: 'match' | 'key';
}

const escape = (s: string) => s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');

/** Splits text into pieces marking search matches (priority) and keywords, case-insensitively. */
export function highlight(text: string, search: string, keywords: string[]): Piece[] {
  const terms: { re: string; mark: Piece['mark'] }[] = [];
  if (search.trim()) terms.push({ re: escape(search.trim()), mark: 'match' });
  for (const k of keywords) if (k.trim()) terms.push({ re: `(?<![\\p{L}\\p{N}])${escape(k.trim())}(?![\\p{L}\\p{N}])`, mark: 'key' });
  if (!terms.length) return [{ text }];
  const re = new RegExp(terms.map((t) => `(${t.re})`).join('|'), 'giu');
  const out: Piece[] = [];
  let last = 0;
  for (const m of text.matchAll(re)) {
    const i = m.index ?? 0;
    if (i > last) out.push({ text: text.slice(last, i) });
    const group = m.slice(1).findIndex((g) => g !== undefined);
    out.push({ text: m[0], mark: terms[group]?.mark });
    last = i + m[0].length;
  }
  if (last < text.length) out.push({ text: text.slice(last) });
  return out;
}

/** Below this confidence a word gets a subtle underline ("check this"). */
export const LOW_CONFIDENCE = 0.45;

/** Whether generating with `provider` needs the user's consent first. */
export function needsConsent(provider: ProviderId, s: Settings['intelligence']): boolean {
  // Everything that leaves the computer (FrameLy Cloud included) needs consent first.
  return provider !== 'framely' && provider !== 'apple' && !s.consents.includes(provider);
}

/** Compact "about 3 min" style duration for estimates (rounded up, never "0"). */
export function roughDuration(ms: number): { unit: 'sec' | 'min' | 'hour'; n: number } {
  const s = Math.max(1, Math.ceil(ms / 1000));
  if (s < 60) return { unit: 'sec', n: s < 10 ? s : Math.ceil(s / 5) * 5 };
  const m = Math.ceil(s / 60);
  if (m < 60) return { unit: 'min', n: m };
  return { unit: 'hour', n: Math.round((m / 60) * 10) / 10 };
}

/** Languages offered for transcription (Whisper supports many more; these cover most use). */
export const TRANSCRIPTION_LANGUAGES = ['pt', 'en', 'es', 'fr', 'de', 'it', 'nl', 'pl', 'ru', 'uk', 'tr', 'ar', 'hi', 'ja', 'ko', 'zh', 'sv'];
