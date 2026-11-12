/**
 * Canvas rendering shared by the on-screen view and the export, so what you see is exactly
 * what gets saved. `region` (source pixels) is mapped onto the context at `scale`.
 */
import { blurBlock, type Rect, type Shape } from './model';

export interface View {
  region: Rect;
  scale: number;
}

const FONT = '600 {size}px -apple-system, BlinkMacSystemFont, "Segoe UI Variable Display", "Segoe UI", system-ui, sans-serif';

export function font(size: number): string {
  return FONT.replace('{size}', String(size));
}

/** Text block size in source pixels (multi-line). */
export function measureText(ctx: CanvasRenderingContext2D, text: string, size: number): { w: number; h: number } {
  ctx.save();
  ctx.font = font(size);
  const lines = text.split('\n');
  const w = Math.max(...lines.map((l) => ctx.measureText(l).width), size * 0.5);
  ctx.restore();
  return { w: Math.ceil(w + size * 0.5), h: Math.ceil(lines.length * size * 1.25 + size * 0.3) };
}

function readableOutline(color: string): string {
  // Dark outline for light colors and vice versa keeps annotations legible on any background.
  const c = color.replace('#', '');
  const [r, g, b] = [0, 2, 4].map((i) => parseInt(c.slice(i, i + 2), 16));
  return 0.299 * r! + 0.587 * g! + 0.114 * b! > 150 ? 'rgba(10,21,52,0.85)' : 'rgba(255,255,255,0.95)';
}

/** Pixelates `r` from `source` (destructive in the export: the original pixels are gone). */
function drawBlur(ctx: CanvasRenderingContext2D, source: CanvasImageSource, r: Rect, block: number) {
  const w = Math.max(1, Math.ceil(r.w / block));
  const h = Math.max(1, Math.ceil(r.h / block));
  const small = new OffscreenCanvas(w, h);
  const sctx = small.getContext('2d')!;
  sctx.imageSmoothingEnabled = true;
  sctx.imageSmoothingQuality = 'high';
  sctx.drawImage(source, r.x, r.y, r.w, r.h, 0, 0, w, h);
  ctx.save();
  ctx.imageSmoothingEnabled = false;
  ctx.drawImage(small, 0, 0, w, h, r.x, r.y, r.w, r.h);
  ctx.restore();
}

function drawArrow(ctx: CanvasRenderingContext2D, s: Extract<Shape, { type: 'arrow' }>) {
  const angle = Math.atan2(s.y2 - s.y1, s.x2 - s.x1);
  const head = Math.max(10, s.width * 4.2);
  const len = Math.hypot(s.x2 - s.x1, s.y2 - s.y1);
  const shaftEnd = Math.max(0, len - head * 0.8);
  ctx.save();
  ctx.shadowColor = 'rgba(0,0,0,0.28)';
  ctx.shadowBlur = s.width * 1.5;
  ctx.strokeStyle = s.color;
  ctx.fillStyle = s.color;
  ctx.lineWidth = s.width;
  ctx.lineCap = 'round';
  ctx.beginPath();
  ctx.moveTo(s.x1, s.y1);
  ctx.lineTo(s.x1 + Math.cos(angle) * shaftEnd, s.y1 + Math.sin(angle) * shaftEnd);
  ctx.stroke();
  ctx.beginPath();
  ctx.moveTo(s.x2, s.y2);
  ctx.lineTo(s.x2 - head * Math.cos(angle - 0.45), s.y2 - head * Math.sin(angle - 0.45));
  ctx.lineTo(s.x2 - head * Math.cos(angle + 0.45), s.y2 - head * Math.sin(angle + 0.45));
  ctx.closePath();
  ctx.fill();
  ctx.restore();
}

export function drawShape(ctx: CanvasRenderingContext2D, s: Shape, source: CanvasImageSource, imageSize: { w: number; h: number }) {
  switch (s.type) {
    case 'blur':
      drawBlur(ctx, source, s, blurBlock(imageSize.w, imageSize.h));
      break;
    case 'highlight':
      ctx.save();
      ctx.globalCompositeOperation = 'multiply';
      ctx.fillStyle = s.color;
      ctx.globalAlpha = 0.45;
      ctx.fillRect(s.x, s.y, s.w, s.h);
      ctx.restore();
      break;
    case 'arrow':
      drawArrow(ctx, s);
      break;
    case 'rect':
    case 'ellipse':
      ctx.save();
      ctx.shadowColor = 'rgba(0,0,0,0.22)';
      ctx.shadowBlur = s.width * 1.2;
      ctx.strokeStyle = s.color;
      ctx.lineWidth = s.width;
      ctx.lineJoin = 'round';
      ctx.beginPath();
      if (s.type === 'rect') ctx.roundRect(s.x, s.y, s.w, s.h, s.width * 1.2);
      else ctx.ellipse(s.x + s.w / 2, s.y + s.h / 2, s.w / 2, s.h / 2, 0, 0, Math.PI * 2);
      ctx.stroke();
      ctx.restore();
      break;
    case 'text': {
      ctx.save();
      ctx.font = font(s.size);
      ctx.textBaseline = 'top';
      ctx.lineJoin = 'round';
      ctx.lineWidth = Math.max(2, s.size / 7);
      ctx.strokeStyle = readableOutline(s.color);
      ctx.fillStyle = s.color;
      s.text.split('\n').forEach((line, i) => {
        const y = s.y + s.size * 0.15 + i * s.size * 1.25;
        ctx.strokeText(line, s.x + s.size * 0.25, y);
        ctx.fillText(line, s.x + s.size * 0.25, y);
      });
      ctx.restore();
      break;
    }
  }
}

/**
 * Draws the image region and all shapes. Blur and highlight go first so outlines, arrows and
 * text stay crisp on top of them.
 */
export function drawDoc(
  ctx: CanvasRenderingContext2D,
  image: CanvasImageSource,
  imageSize: { w: number; h: number },
  shapes: Shape[],
  view: View,
) {
  const { region, scale } = view;
  ctx.save();
  ctx.scale(scale, scale);
  ctx.translate(-region.x, -region.y);
  ctx.imageSmoothingQuality = 'high';
  ctx.drawImage(image, 0, 0);
  const order = (s: Shape) => (s.type === 'blur' ? 0 : s.type === 'highlight' ? 1 : 2);
  for (const s of [...shapes].sort((a, b) => order(a) - order(b))) drawShape(ctx, s, image, imageSize);
  ctx.restore();
}

/** Renders the final image (crop + resize + annotations, blur baked in) as PNG bytes. */
export async function exportPng(image: HTMLImageElement, shapes: Shape[], crop: Rect, out: { w: number; h: number }): Promise<Uint8Array> {
  const canvas = new OffscreenCanvas(out.w, out.h);
  const ctx = canvas.getContext('2d')!;
  drawDoc(ctx as unknown as CanvasRenderingContext2D, image, { w: image.naturalWidth, h: image.naturalHeight }, shapes, {
    region: crop,
    scale: out.w / crop.w,
  });
  const blob = await canvas.convertToBlob({ type: 'image/png' });
  return new Uint8Array(await blob.arrayBuffer());
}
