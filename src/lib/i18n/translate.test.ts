import { describe, expect, it } from 'vitest';
import { dictionaries, format, resolveLocale } from './translate';
import { en } from './en';

describe('i18n', () => {
  it('every locale defines every key with non-empty text', () => {
    for (const [loc, dict] of Object.entries(dictionaries)) {
      for (const key of Object.keys(en)) {
        const v = (dict as Record<string, string>)[key];
        expect(v, `${loc}:${key}`).toBeTruthy();
      }
      expect(Object.keys(dict).sort()).toEqual(Object.keys(en).sort());
    }
  });

  it('resolves the system language like the Rust core', () => {
    expect(resolveLocale('system', 'pt-PT')).toBe('pt-BR');
    expect(resolveLocale('system', 'en-GB')).toBe('en');
    expect(resolveLocale('system', undefined)).toBe('en');
    expect(resolveLocale('en', 'pt-BR')).toBe('en');
  });

  it('interpolates params', () => {
    expect(format('en', 'app.name')).toBe('FrameLy');
  });
});
