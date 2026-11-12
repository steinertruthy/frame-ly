import { en, type MessageKey, type Messages } from './en';
import { ptBR } from './pt-BR';
import { es } from './es';
import { fr } from './fr';
import { de } from './de';
import type { Language } from '$lib/ipc/types';

export type Locale = 'pt-BR' | 'en' | 'es' | 'fr' | 'de';

export const LOCALES: Locale[] = ['pt-BR', 'en', 'es', 'fr', 'de'];

/** Each language's own name, shown in the language picker. */
export const LOCALE_NAMES: Record<Locale, string> = {
  'pt-BR': 'Português (Brasil)',
  en: 'English',
  es: 'Español',
  fr: 'Français',
  de: 'Deutsch',
};

export const dictionaries: Record<Locale, Messages> = { en, 'pt-BR': ptBR, es, fr, de };

/** Mirrors `core::locale::resolve` in Rust so the tray and the UI always agree. */
export function resolveLocale(setting: Language, systemLocale: string | undefined): Locale {
  if (setting !== 'system') return setting;
  const l = (systemLocale ?? '').toLowerCase().slice(0, 2);
  if (l === 'pt') return 'pt-BR';
  if (l === 'es' || l === 'fr' || l === 'de') return l;
  return 'en';
}

export function format(locale: Locale, key: MessageKey, params?: Record<string, string | number>): string {
  let msg = dictionaries[locale][key] ?? en[key];
  if (params) {
    for (const [k, v] of Object.entries(params)) msg = msg.replaceAll(`{${k}}`, String(v));
  }
  return msg;
}
