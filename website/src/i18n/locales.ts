/** Exactly the app's languages (src/lib/i18n/translate.ts in the app). */
export const LOCALES = ['pt-BR', 'en', 'es', 'fr', 'de'] as const;
export type Locale = (typeof LOCALES)[number];
export const DEFAULT_LOCALE: Locale = 'en';

/** URL segment per locale (lowercase, BCP 47). */
export const PREFIX: Record<Locale, string> = { 'pt-BR': 'pt-br', en: 'en', es: 'es', fr: 'fr', de: 'de' };
export const LOCALE_NAMES: Record<Locale, string> = {
  'pt-BR': 'Português (Brasil)',
  en: 'English',
  es: 'Español',
  fr: 'Français',
  de: 'Deutsch',
};
/** Open Graph locale codes. */
export const OG_LOCALE: Record<Locale, string> = { 'pt-BR': 'pt_BR', en: 'en_US', es: 'es_ES', fr: 'fr_FR', de: 'de_DE' };

export type PageId = 'home' | 'privacy' | 'terms' | 'license' | 'changelog' | 'faq';

/** Localized slugs; '' is the locale's home. */
export const SLUGS: Record<PageId, Record<Locale, string>> = {
  home: { 'pt-BR': '', en: '', es: '', fr: '', de: '' },
  privacy: { 'pt-BR': 'privacidade', en: 'privacy', es: 'privacidad', fr: 'confidentialite', de: 'datenschutz' },
  terms: { 'pt-BR': 'termos', en: 'terms', es: 'terminos', fr: 'conditions', de: 'nutzungsbedingungen' },
  license: { 'pt-BR': 'licenca', en: 'license', es: 'licencia', fr: 'licence', de: 'lizenz' },
  changelog: { 'pt-BR': 'novidades', en: 'changelog', es: 'novedades', fr: 'nouveautes', de: 'aenderungen' },
  faq: { 'pt-BR': 'perguntas-frequentes', en: 'faq', es: 'preguntas-frecuentes', fr: 'questions-frequentes', de: 'haeufige-fragen' },
};

export function localeFromPrefix(prefix: string): Locale | undefined {
  return LOCALES.find((l) => PREFIX[l] === prefix);
}

/** Path (without base) of a page in a locale, always with a trailing slash. */
export function pagePath(page: PageId, locale: Locale): string {
  const slug = SLUGS[page][locale];
  return `/${PREFIX[locale]}/${slug ? `${slug}/` : ''}`;
}

/** Same rule as the app (resolveLocale): pt* → pt-BR, es/fr/de, otherwise English. */
export function matchLocale(languages: readonly string[]): Locale {
  for (const raw of languages) {
    const l = raw.toLowerCase().slice(0, 2);
    if (l === 'pt') return 'pt-BR';
    if (l === 'en') return 'en';
    if (l === 'es' || l === 'fr' || l === 'de') return l;
  }
  return DEFAULT_LOCALE;
}
