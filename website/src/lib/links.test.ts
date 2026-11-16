import { describe, expect, it } from 'vitest';
import { externalLinks } from './links';

describe('externalLinks', () => {
  const site = 'https://steinertruthy.github.io';
  it('opens external links in a new tab', () => {
    expect(externalLinks('<a class="x" href="https://github.com/a">a</a>', site)).toBe(
      '<a class="x" href="https://github.com/a" target="_blank" rel="noopener noreferrer">a</a>',
    );
  });
  it('leaves internal, relative and already-targeted links alone', () => {
    const html = '<a href="/frame-ly/en/">x</a><a href="https://steinertruthy.github.io/frame-ly/">y</a><a href="https://e.x" target="_self">z</a><a href="#ai">w</a>';
    expect(externalLinks(html, site)).toBe(html);
  });
  it('keeps an existing rel', () => {
    expect(externalLinks('<a href="https://e.x" rel="me">e</a>', site)).toBe('<a href="https://e.x" rel="me" target="_blank">e</a>');
  });
});
