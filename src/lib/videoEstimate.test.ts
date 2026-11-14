import { describe, expect, it } from 'vitest';
import { estimate, formatEstimate, outputSize } from './videoEstimate';

describe('outputSize', () => {
  it('matches the Rust presets', () => {
    expect(outputSize([3024, 1964], 'native', 'h264')).toEqual([3024, 1964]);
    expect(outputSize([3024, 1964], '1080p', 'h264')).toEqual([1662, 1080]);
    expect(outputSize([1920, 1080], '720p', 'h264')).toEqual([1280, 720]);
    expect(outputSize([800, 600], '1080p', 'h264')).toEqual([800, 600]);
    // H.264 caps at 4096×2304, HEVC keeps a 5K screen.
    expect(outputSize([5120, 2880], 'native', 'h264')[0]).toBe(4096);
    expect(outputSize([5120, 2880], 'native', 'hevc')).toEqual([5120, 2880]);
  });
});

describe('estimate', () => {
  const base = { quality: 'high', resolution: 'native', codec: 'h264', fps: 30 } as const;

  it('reproduces the measured recording (3024×1964, video playing)', () => {
    const e = estimate([3024, 1964], base);
    expect(e.busy).toBeGreaterThan(60);
    expect(e.busy).toBeLessThan(72);
    expect(e.calm).toBeLessThan(e.busy / 3);
  });

  it('shrinks with HEVC, 1080p and lower quality; grows with 60 fps', () => {
    const ref = estimate([3024, 1964], base).busy;
    expect(estimate([3024, 1964], { ...base, codec: 'hevc' }).busy).toBeCloseTo(ref * 0.6);
    expect(estimate([3024, 1964], { ...base, resolution: '1080p' }).busy).toBeLessThan(ref / 3);
    expect(estimate([3024, 1964], { ...base, quality: 'medium' }).busy).toBeLessThan(ref);
    expect(estimate([3024, 1964], { ...base, fps: 60 }).busy).toBeGreaterThan(ref * 1.5);
  });
});

describe('formatEstimate', () => {
  it('rounds for humans', () => {
    expect(formatEstimate(0.4, 'en')).toBe('< 1 MB');
    expect(formatEstimate(7.6, 'en')).toBe('8 MB');
    expect(formatEstimate(66.2, 'en')).toBe('65 MB');
    expect(formatEstimate(3972, 'pt-BR')).toBe('4 GB');
    expect(formatEstimate(1310, 'en')).toBe('1.3 GB');
  });
});
