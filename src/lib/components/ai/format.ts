import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
import { roughDuration } from '$lib/ai';
import type { ModelStatus, ProviderId } from '$lib/ipc/ai';

/** "3 min", "45 s", "1,5 h" in the interface language. */
export function roughTime(ms: number): string {
  const r = roughDuration(ms);
  return t(`ai.time.${r.unit}` as MessageKey, { n: r.n.toLocaleString(locale()) });
}

export const providerName = (p: ProviderId): string => t(`ai.provider.${p}` as MessageKey);
export const modelName = (id: string): string => t(`ai.model.${id}` as MessageKey);

export function pct(m: ModelStatus): number {
  return m.size ? Math.min(100, Math.floor((m.received / m.size) * 100)) : 0;
}

export function modelStateLabel(m: ModelStatus): string {
  return t(`ai.state.${m.state}` as MessageKey, { pct: pct(m) });
}

export function languageName(code: string): string {
  try {
    const name = new Intl.DisplayNames([locale()], { type: 'language' }).of(code) ?? code;
    return name.charAt(0).toUpperCase() + name.slice(1);
  } catch {
    return code;
  }
}

export const PROVIDER_HOSTS: Record<ProviderId, string> = {
  framely: '',
  apple: '',
  anthropic: 'api.anthropic.com',
  openai: 'api.openai.com',
};
