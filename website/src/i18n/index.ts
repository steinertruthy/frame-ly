import { en } from './en';
import { ptBR } from './pt-BR';
import { es } from './es';
import { fr } from './fr';
import { de } from './de';
import type { Locale } from './locales';

/** Same shape as English, with any string allowed (so translations are checked key by key). */
type Widen<T> = T extends string
  ? string
  : T extends readonly (infer U)[]
    ? readonly Widen<U>[]
    : { readonly [K in keyof T]: Widen<T[K]> };
export type Messages = Widen<typeof en>;

export const dictionaries: Record<Locale, Messages> = { en, 'pt-BR': ptBR, es, fr, de };

export function t(locale: Locale): Messages {
  return dictionaries[locale];
}

export function fill(template: string, params: Record<string, string | number>): string {
  return template.replace(/\{(\w+)\}/g, (m, k: string) => (k in params ? String(params[k]) : m));
}
