/**
 * File-size estimates for the recording settings. Pure and unit-tested.
 *
 * Calibrated on real measurements (constant-quality H.264 at "high"):
 * - a calm screen (pointer, typing, a blinking caret) ≈ 2.7 MB per minute per megapixel;
 * - a screen with continuous motion (a video playing full screen) ≈ 11 MB per minute per
 *   megapixel (a 65 s 3024×1964 recording at 30 fps took 72 MB).
 * Other presets, codecs and frame rates scale those figures.
 */
import type { Fps, VideoCodec, VideoQuality, VideoResolution } from '$lib/ipc/types';

const MB_PER_MIN_PER_MPX = { calm: 2.7, busy: 11 } as const;
const QUALITY_FACTOR: Record<VideoQuality, number> = { low: 0.45, medium: 0.72, high: 1, max: 1.45 };
const CODEC_FACTOR: Record<VideoCodec, number> = { h264: 1, hevc: 0.6 };

/** Same rules as `core::video::output_size` in Rust: preset by height, codec maximum, even sizes. */
export function outputSize(src: [number, number], resolution: VideoResolution, codec: VideoCodec): [number, number] {
  const [w, h] = [Math.max(2, src[0]), Math.max(2, src[1])];
  let scale = resolution === '1080p' ? Math.min(1, 1080 / h) : resolution === '720p' ? Math.min(1, 720 / h) : 1;
  const [mw, mh] = codec === 'h264' ? [4096, 2304] : [8192, 4320];
  scale = Math.min(scale, mw / w, mh / h);
  const even = (v: number) => Math.max(2, Math.floor(v) & ~1);
  return [even(w * scale), even(h * scale)];
}

export interface Estimate {
  size: [number, number];
  /** MB per minute: calm screen / lots of motion. */
  calm: number;
  busy: number;
}

export function estimate(
  screen: [number, number],
  opts: { quality: VideoQuality; resolution: VideoResolution; codec: VideoCodec; fps: Fps },
): Estimate {
  const size = outputSize(screen, opts.resolution, opts.codec);
  const mpx = (size[0] * size[1]) / 1e6;
  const k = QUALITY_FACTOR[opts.quality] * CODEC_FACTOR[opts.codec];
  // A calm screen sends few new frames whatever the rate; motion scales almost with fps.
  const fpsCalm = Math.sqrt(opts.fps / 30);
  const fpsBusy = Math.pow(opts.fps / 30, 0.85);
  return {
    size,
    calm: MB_PER_MIN_PER_MPX.calm * mpx * k * fpsCalm,
    busy: MB_PER_MIN_PER_MPX.busy * mpx * k * fpsBusy,
  };
}

/** "8 MB", "1.2 GB" — rounded for an estimate, in the user's locale. */
export function formatEstimate(mb: number, locale: string): string {
  const n = (v: number, digits: number) => new Intl.NumberFormat(locale, { maximumFractionDigits: digits }).format(v);
  if (mb < 1) return '< 1 MB';
  if (mb < 1000) return `${n(mb < 10 ? Math.round(mb) : Math.round(mb / 5) * 5, 0)} MB`;
  return `${n(mb / 1000, mb < 10_000 ? 1 : 0)} GB`;
}
