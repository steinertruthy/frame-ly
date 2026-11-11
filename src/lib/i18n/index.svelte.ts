import type { MessageKey } from './en';
import { format, resolveLocale, type Locale } from './translate';
import type { Language } from '$lib/ipc/types';

export type { MessageKey };

const state = $state<{ locale: Locale }>({ locale: resolveLocale('system', navigator.language) });

export function setLanguage(language: Language): void {
  state.locale = resolveLocale(language, navigator.language);
  document.documentElement.lang = state.locale;
}

export function locale(): Locale {
  return state.locale;
}

/** Reactive translate: reading `state.locale` makes templates re-render on language change. */
export function t(key: MessageKey, params?: Record<string, string | number>): string {
  return format(state.locale, key, params);
}
