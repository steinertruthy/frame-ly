import type { Rect, WindowInfo, Display } from '$lib/ipc/types';

/** Normalized rectangle from two drag points. */
export function rectFromPoints(a: { x: number; y: number }, b: { x: number; y: number }): Rect {
  return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), width: Math.abs(a.x - b.x), height: Math.abs(a.y - b.y) };
}

/** Clamps a rect to the viewport. */
export function clampRect(r: Rect, w: number, h: number): Rect {
  const x0 = Math.max(0, r.x);
  const y0 = Math.max(0, r.y);
  const x1 = Math.min(w, r.x + r.width);
  const y1 = Math.min(h, r.y + r.height);
  return { x: x0, y: y0, width: Math.max(0, x1 - x0), height: Math.max(0, y1 - y0) };
}

/** Window bounds (OS desktop space) → this overlay's CSS pixels. */
export function windowToLocal(w: WindowInfo, d: Display, viewportWidth: number): Rect {
  const k = viewportWidth / d.bounds.width;
  return {
    x: (w.bounds.x - d.bounds.x) * k,
    y: (w.bounds.y - d.bounds.y) * k,
    width: w.bounds.width * k,
    height: w.bounds.height * k,
  };
}

/** Front-most window containing the point (list is front-to-back). */
export function hitTest<T extends { rect: Rect }>(items: T[], x: number, y: number): T | undefined {
  return items.find(({ rect: r }) => x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height);
}
