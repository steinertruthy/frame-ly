import { describe, expect, it } from 'vitest';
import {
  History,
  baseStroke,
  blurBlock,
  clampRect,
  distToSegment,
  hits,
  isMeaningful,
  moved,
  newDoc,
  normRect,
  outputSize,
  resizeKeepingAspect,
  shapeAt,
  type Shape,
} from './model';

const rect: Shape = { id: 1, type: 'rect', x: 100, y: 100, w: 200, h: 100, color: 'red', width: 4 };
const arrow: Shape = { id: 2, type: 'arrow', x1: 0, y1: 0, x2: 100, y2: 0, color: 'red', width: 4 };
const ellipse: Shape = { id: 3, type: 'ellipse', x: 0, y: 0, w: 200, h: 100, color: 'red', width: 4 };
const blur: Shape = { id: 4, type: 'blur', x: 10, y: 10, w: 50, h: 50 };

describe('editor geometry', () => {
  it('normalizes and clamps rectangles', () => {
    expect(normRect({ x: 50, y: 10 }, { x: 10, y: 40 })).toEqual({ x: 10, y: 10, w: 40, h: 30 });
    expect(clampRect({ x: -10, y: 5, w: 50, h: 500 }, { x: 0, y: 0, w: 100, h: 100 })).toEqual({ x: 0, y: 5, w: 40, h: 95 });
  });

  it('measures distance to a segment', () => {
    expect(distToSegment({ x: 50, y: 10 }, { x: 0, y: 0 }, { x: 100, y: 0 })).toBe(10);
    expect(distToSegment({ x: 130, y: 40 }, { x: 0, y: 0 }, { x: 100, y: 0 })).toBe(50);
  });

  it('hit-tests outlines near the edge and fills anywhere', () => {
    expect(hits(rect, { x: 101, y: 150 }, 4)).toBe(true);
    expect(hits(rect, { x: 200, y: 150 }, 4)).toBe(false); // hollow middle
    expect(hits(arrow, { x: 50, y: 3 }, 4)).toBe(true);
    expect(hits(arrow, { x: 50, y: 30 }, 4)).toBe(false);
    expect(hits(ellipse, { x: 100, y: 1 }, 4)).toBe(true);
    expect(hits(ellipse, { x: 100, y: 50 }, 4)).toBe(false);
    expect(hits(blur, { x: 30, y: 30 }, 0)).toBe(true);
  });

  it('picks the topmost shape', () => {
    const over: Shape = { ...blur, id: 9, x: 90, y: 90, w: 50, h: 50 };
    expect(shapeAt([rect, over], { x: 101, y: 120 }, 4)?.id).toBe(9);
    expect(shapeAt([rect, over], { x: 500, y: 500 }, 4)).toBeUndefined();
  });

  it('moves every shape type', () => {
    expect(moved(arrow, 5, 6)).toMatchObject({ x1: 5, y1: 6, x2: 105, y2: 6 });
    expect(moved(rect, -100, 0)).toMatchObject({ x: 0, y: 100 });
  });

  it('ignores accidental tiny shapes', () => {
    expect(isMeaningful({ ...rect, w: 2 }, 4)).toBe(false);
    expect(isMeaningful(arrow, 4)).toBe(true);
    expect(isMeaningful({ id: 5, type: 'text', x: 0, y: 0, w: 0, h: 0, text: '  ', color: 'red', size: 20 }, 4)).toBe(false);
  });
});

describe('export sizing', () => {
  it('defaults to the crop and keeps aspect when resizing', () => {
    const doc = newDoc(3024, 1964);
    expect(outputSize(doc)).toEqual({ w: 3024, h: 1964 });
    expect(resizeKeepingAspect(doc.crop, 1512, 'w')).toEqual({ w: 1512, h: 982 });
    expect(resizeKeepingAspect({ x: 0, y: 0, w: 400, h: 200 }, 50, 'h')).toEqual({ w: 100, h: 50 });
    expect(outputSize({ ...doc, output: { w: 10, h: 5 } })).toEqual({ w: 10, h: 5 });
  });

  it('scales stroke and blur with the image', () => {
    expect(baseStroke(800, 600)).toBe(2);
    expect(baseStroke(3024, 1964)).toBe(5);
    expect(blurBlock(800, 600)).toBe(8);
    expect(blurBlock(3024, 1964)).toBe(22);
  });
});

describe('history', () => {
  it('undoes and redoes, and a new change clears redo', () => {
    const h = new History();
    const a = newDoc(10, 10);
    const b = { ...a, shapes: [rect] };
    h.record(a);
    const undone = h.undo(b)!;
    expect(undone.shapes).toHaveLength(0);
    expect(h.canRedo).toBe(true);
    expect(h.redo(undone)!.shapes).toHaveLength(1);
    h.record(a);
    expect(h.canRedo).toBe(false);
  });

  it('snapshots are independent copies', () => {
    const h = new History();
    const doc = newDoc(10, 10);
    h.record(doc);
    doc.shapes.push(rect);
    expect(h.undo(doc)!.shapes).toHaveLength(0);
  });
});
