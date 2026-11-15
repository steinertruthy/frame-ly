import { describe, expect, it } from 'vitest';
import { chooseOs, detectOs } from './platform';

const ua = {
  mac: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15',
  win: 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',
  iphone: 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1',
  android: 'Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Mobile Safari/537.36',
  linux: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/140.0.0.0 Safari/537.36',
};

describe('detectOs', () => {
  it('recognizes desktop macOS and Windows', () => {
    expect(detectOs({ userAgent: ua.mac })).toBe('macos');
    expect(detectOs({ userAgent: ua.win })).toBe('windows');
  });
  it('prefers client hints', () => {
    expect(detectOs({ userAgent: ua.linux, userAgentData: { platform: 'Windows', mobile: false } })).toBe('windows');
    expect(detectOs({ userAgent: ua.win, userAgentData: { platform: 'macOS', mobile: false } })).toBe('macos');
  });
  it('is neutral for phones, tablets, Linux and unknowns', () => {
    expect(detectOs({ userAgent: ua.iphone })).toBe('other');
    expect(detectOs({ userAgent: ua.android })).toBe('other');
    expect(detectOs({ userAgent: ua.linux })).toBe('other');
    expect(detectOs({ userAgent: ua.mac }, 5)).toBe('other'); // iPadOS desktop mode
    expect(detectOs({ userAgent: '' })).toBe('other');
    expect(detectOs({ userAgent: ua.win, userAgentData: { platform: 'Android', mobile: true } })).toBe('other');
  });
});

describe('chooseOs', () => {
  it('lets a saved manual choice win over detection', () => {
    expect(chooseOs('windows', 'macos')).toBe('windows');
    expect(chooseOs(null, 'macos')).toBe('macos');
    expect(chooseOs('linux', 'other')).toBe('other');
  });
});
