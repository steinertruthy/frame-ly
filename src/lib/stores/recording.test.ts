import { describe, expect, it } from 'vitest';
import { formatBytes, formatDuration } from './format';

describe('formatting', () => {
  it('formats durations like the tray', () => {
    expect(formatDuration(7_400)).toBe('00:07');
    expect(formatDuration(754_000)).toBe('12:34');
    expect(formatDuration(3_723_000)).toBe('1:02:03');
  });
  it('formats sizes', () => {
    expect(formatBytes(500)).toBe('1 KB');
    expect(formatBytes(5 * 1024 * 1024)).toBe('5.0 MB');
    expect(formatBytes(3 * 1024 ** 3)).toBe('3.00 GB');
  });
});
