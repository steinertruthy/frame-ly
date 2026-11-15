import { describe, expect, it } from 'vitest';
import { dictionaries } from './index';
import { LOCALES, matchLocale, pagePath, SLUGS } from './locales';

function shape(v: unknown): unknown {
  if (Array.isArray(v)) return v.map(shape);
  if (v && typeof v === 'object') return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, shape(x)]));
  return typeof v;
}
function strings(v: unknown, out: string[] = []): string[] {
  if (typeof v === 'string') out.push(v);
  else if (v && typeof v === 'object') Object.values(v).forEach((x) => strings(x, out));
  return out;
}

describe('dictionaries', () => {
  it('match the app languages exactly', () => {
    expect([...LOCALES].sort()).toEqual(['de', 'en', 'es', 'fr', 'pt-BR']);
  });
  it('have the same keys and list lengths as English', () => {
    const en = shape(dictionaries.en);
    for (const l of LOCALES) expect(shape(dictionaries[l]), l).toEqual(en);
  });
  it('have no empty strings and keep every placeholder', () => {
    const en = strings(dictionaries.en);
    for (const l of LOCALES) {
      const s = strings(dictionaries[l]);
      s.forEach((x) => expect(x.trim().length, l).toBeGreaterThan(0));
      s.forEach((x, i) => expect(x.match(/\{\w+\}/g) ?? [], `${l}: ${x}`).toEqual(en[i]!.match(/\{\w+\}/g) ?? []));
    }
  });
});

describe('locales', () => {
  it('resolve like the app', () => {
    expect(matchLocale(['pt-PT'])).toBe('pt-BR');
    expect(matchLocale(['de-AT', 'en'])).toBe('de');
    expect(matchLocale(['ja', 'fr-CA'])).toBe('fr');
    expect(matchLocale(['ja'])).toBe('en');
  });
  it('give unique localized paths', () => {
    const all = LOCALES.flatMap((l) => (Object.keys(SLUGS) as (keyof typeof SLUGS)[]).map((p) => pagePath(p, l)));
    expect(new Set(all).size).toBe(all.length);
    expect(pagePath('privacy', 'pt-BR')).toBe('/pt-br/privacidade/');
  });
});
