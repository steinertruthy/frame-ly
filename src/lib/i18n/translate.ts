import { en, type MessageKey, type Messages } from './en';
import { ptBR } from './pt-BR';
import type { Language } from '$lib/ipc/types';

export type Locale = 'pt-BR' | 'en';

export const dictionaries: Record<Locale, Messages> = { en, 'pt-BR': ptBR };

/** Mirrors `core::locale::resolve` in Rust so the tray and the UI always agree. */
export function resolveLocale(setting: Language, systemLocale: string | undefined): Locale {
  if (setting !== 'system') return setting;
  return systemLocale?.toLowerCase().startsWith('pt') ? 'pt-BR' : 'en';
}

export function format(locale: Locale, key: MessageKey, params?: Record<string, string | number>): string {
  let msg = dictionaries[locale][key] ?? en[key];
  if (params) {
    for (const [k, v] of Object.entries(params)) msg = msg.replaceAll(`{${k}}`, String(v));
  }
  return msg;
}
