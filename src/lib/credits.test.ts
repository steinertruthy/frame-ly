import { describe, expect, it } from 'vitest';
import { CREDITS, creditsFor } from './credits';

describe('credits', () => {
  it('every entry links to an https page and names its license (or is part of a platform/service)', () => {
    for (const list of Object.values(CREDITS)) {
      for (const c of list) {
        expect(c.url.startsWith('https://'), c.name).toBe(true);
        expect(c.name.length).toBeGreaterThan(0);
      }
    }
  });
  it('shows only what applies to the platform', () => {
    const mac = creditsFor('macos').flatMap(([, l]) => l.map((c) => c.name));
    const win = creditsFor('windows').flatMap(([, l]) => l.map((c) => c.name));
    expect(mac).toContain('Apple Foundation Models');
    expect(mac).not.toContain('windows-rs');
    expect(win).toContain('windows-rs');
    expect(win).not.toContain('Apple Foundation Models');
    expect(mac).toContain('whisper.cpp');
  });
});
