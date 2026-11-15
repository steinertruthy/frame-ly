import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { inline, localized, parseChangelog } from './changelog';

describe('changelog', () => {
  const md = readFileSync(new URL('../../../CHANGELOG.md', import.meta.url), 'utf8');
  const entries = parseChangelog(md);

  it('parses released versions and skips Unreleased', () => {
    expect(entries[0]?.version).toBe('0.1.0');
    expect(entries[0]?.date).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    expect(entries.some((e) => e.version.toLowerCase() === 'unreleased')).toBe(false);
    expect(entries[0]?.sections.find((s) => s.kind === 'added')?.items.length).toBeGreaterThan(3);
  });

  it('every translation covers the same versions and section kinds', () => {
    for (const l of ['pt-BR', 'es', 'fr', 'de']) {
      const tr = parseChangelog(readFileSync(new URL(`../content/changelog/${l}.md`, import.meta.url), 'utf8'));
      for (const t of tr) {
        const en = entries.find((e) => e.version === t.version);
        expect(en, `${l} ${t.version}`).toBeDefined();
        expect(t.sections.map((s) => s.kind)).toEqual(en!.sections.map((s) => s.kind));
        t.sections.forEach((s, i) => expect(s.items.length, `${l} ${t.version} ${s.title}`).toBe(en!.sections[i]!.items.length));
      }
    }
  });

  it('falls back to English for untranslated versions', () => {
    const out = localized(entries, []);
    expect(out[0]?.translated).toBe(false);
  });

  it('escapes HTML in inline markdown', () => {
    expect(inline('<b>`x`</b> [a](https://e.x)')).toBe('&lt;b&gt;<code>x</code>&lt;/b&gt; <a href="https://e.x">a</a>');
    expect(inline('[a](javascript:alert(1))')).not.toContain('href');
  });
});
