/**
 * Screenshot editor document model. All coordinates are in source-image pixels, so edits are
 * independent of zoom and survive crop/resize. Pure and unit-tested.
 */

export type Tool = 'select' | 'arrow' | 'rect' | 'ellipse' | 'text' | 'highlight' | 'blur' | 'crop';

export interface Rect {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Point {
  x: number;
  y: number;
}

interface Base {
  id: number;
}
export type Shape =
  | (Base & { type: 'arrow'; x1: number; y1: number; x2: number; y2: number; color: string; width: number })
  | (Base & { type: 'rect' | 'ellipse'; x: number; y: number; w: number; h: number; color: string; width: number })
  | (Base & { type: 'highlight'; x: number; y: number; w: number; h: number; color: string })
  | (Base & { type: 'blur'; x: number; y: number; w: number; h: number })
  | (Base & { type: 'text'; x: number; y: number; w: number; h: number; text: string; color: string; size: number });

export interface Doc {
  shapes: Shape[];
  /** Visible part of the source image. */
  crop: Rect;
  /** Exported size; null = crop size (no resize). */
  output: { w: number; h: number } | null;
}

export const COLORS = ['#FF4B5C', '#FFB020', '#22C38E', '#4A7BFF', '#FFFFFF', '#0A1534'] as const;
export const HIGHLIGHT = '#FFE14D';

export function newDoc(width: number, height: number): Doc {
  return { shapes: [], crop: { x: 0, y: 0, w: width, h: height }, output: null };
}

/** Stroke unit that looks the same on a 1x laptop screenshot and a 5K one. */
export function baseStroke(imageW: number, imageH: number): number {
  return Math.max(2, Math.round(Math.max(imageW, imageH) / 600));
}

export function normRect(a: Point, b: Point): Rect {
  return { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.abs(a.x - b.x), h: Math.abs(a.y - b.y) };
}

export function clampRect(r: Rect, bounds: Rect): Rect {
  const x0 = Math.max(bounds.x, r.x);
  const y0 = Math.max(bounds.y, r.y);
  const x1 = Math.min(bounds.x + bounds.w, r.x + r.w);
  const y1 = Math.min(bounds.y + bounds.h, r.y + r.h);
  return { x: x0, y: y0, w: Math.max(0, x1 - x0), h: Math.max(0, y1 - y0) };
}

export function distToSegment(p: Point, a: Point, b: Point): number {
  const dx = b.x - a.x;
  const dy = b.y - a.y;
  const len2 = dx * dx + dy * dy;
  const t = len2 === 0 ? 0 : Math.max(0, Math.min(1, ((p.x - a.x) * dx + (p.y - a.y) * dy) / len2));
  return Math.hypot(p.x - (a.x + t * dx), p.y - (a.y + t * dy));
}

export function bounds(s: Shape): Rect {
  if (s.type === 'arrow') return normRect({ x: s.x1, y: s.y1 }, { x: s.x2, y: s.y2 });
  return { x: s.x, y: s.y, w: s.w, h: s.h };
}

function inside(p: Point, r: Rect, pad = 0): boolean {
  return p.x >= r.x - pad && p.x <= r.x + r.w + pad && p.y >= r.y - pad && p.y <= r.y + r.h + pad;
}

/** Whether `p` touches the shape. Outlines are hit near their edge, filled areas anywhere. */
export function hits(s: Shape, p: Point, tolerance: number): boolean {
  switch (s.type) {
    case 'arrow':
      return distToSegment(p, { x: s.x1, y: s.y1 }, { x: s.x2, y: s.y2 }) <= tolerance + s.width / 2;
    case 'rect': {
      const pad = tolerance + s.width / 2;
      return inside(p, s, pad) && !inside(p, { x: s.x + pad, y: s.y + pad, w: s.w - 2 * pad, h: s.h - 2 * pad });
    }
    case 'ellipse': {
      const rx = s.w / 2;
      const ry = s.h / 2;
      if (rx <= 0 || ry <= 0) return false;
      const nx = (p.x - (s.x + rx)) / rx;
      const ny = (p.y - (s.y + ry)) / ry;
      const d = Math.sqrt(nx * nx + ny * ny); // 1 on the outline
      return Math.abs(d - 1) * Math.min(rx, ry) <= tolerance + s.width / 2;
    }
    default:
      return inside(p, s, tolerance);
  }
}

/** Topmost shape under `p` (last drawn wins). */
export function shapeAt(shapes: Shape[], p: Point, tolerance: number): Shape | undefined {
  for (let i = shapes.length - 1; i >= 0; i--) if (hits(shapes[i]!, p, tolerance)) return shapes[i];
  return undefined;
}

export function moved(s: Shape, dx: number, dy: number): Shape {
  if (s.type === 'arrow') return { ...s, x1: s.x1 + dx, y1: s.y1 + dy, x2: s.x2 + dx, y2: s.y2 + dy };
  return { ...s, x: s.x + dx, y: s.y + dy };
}

/** Big enough to keep (ignores accidental clicks while drawing). */
export function isMeaningful(s: Shape, min: number): boolean {
  if (s.type === 'arrow') return Math.hypot(s.x2 - s.x1, s.y2 - s.y1) >= min;
  if (s.type === 'text') return s.text.trim().length > 0;
  return s.w >= min && s.h >= min;
}

export function outputSize(doc: Doc): { w: number; h: number } {
  return doc.output ?? { w: Math.round(doc.crop.w), h: Math.round(doc.crop.h) };
}

/** Keeps the crop's aspect ratio when the user edits one dimension of the export size. */
export function resizeKeepingAspect(crop: Rect, value: number, edited: 'w' | 'h'): { w: number; h: number } {
  const ratio = crop.w / crop.h;
  const v = Math.max(1, Math.round(value));
  return edited === 'w' ? { w: v, h: Math.max(1, Math.round(v / ratio)) } : { w: Math.max(1, Math.round(v * ratio)), h: v };
}

/** Pixelation block size for blur regions: coarse enough that text can't be read back. */
export function blurBlock(imageW: number, imageH: number): number {
  return Math.max(8, Math.round(Math.max(imageW, imageH) / 140));
}

/** Snapshot history (documents are small: shapes + crop). */
export class History {
  private past: Doc[] = [];
  private future: Doc[] = [];
  constructor(private limit = 100) {}

  /** Call with the state *before* a change. */
  record(before: Doc): void {
    this.past.push(structuredClone(before));
    if (this.past.length > this.limit) this.past.shift();
    this.future = [];
  }

  undo(current: Doc): Doc | null {
    const prev = this.past.pop();
    if (!prev) return null;
    this.future.push(structuredClone(current));
    return prev;
  }

  redo(current: Doc): Doc | null {
    const next = this.future.pop();
    if (!next) return null;
    this.past.push(structuredClone(current));
    return next;
  }

  get canUndo(): boolean {
    return this.past.length > 0;
  }

  get canRedo(): boolean {
    return this.future.length > 0;
  }
}
