import { describe, expect, it } from 'vitest';
import { clampRect, hitTest, rectFromPoints, windowToLocal } from './geometry';
import type { Display, WindowInfo } from '$lib/ipc/types';

const display: Display = {
  id: '2',
  name: 'Ext',
  bounds: { x: 1512, y: 0, width: 2560, height: 1440 },
  workArea: { x: 1512, y: 0, width: 2560, height: 1400 },
  pixelWidth: 5120,
  pixelHeight: 2880,
  scaleFactor: 2,
  refreshRate: 60,
  isPrimary: false,
};

describe('overlay geometry', () => {
  it('normalizes drags in any direction', () => {
    expect(rectFromPoints({ x: 50, y: 40 }, { x: 10, y: 60 })).toEqual({ x: 10, y: 40, width: 40, height: 20 });
  });

  it('clamps to the viewport', () => {
    expect(clampRect({ x: -10, y: 5, width: 30, height: 1000 }, 100, 100)).toEqual({ x: 0, y: 5, width: 20, height: 95 });
  });

  it('maps window bounds on a secondary display into local CSS pixels', () => {
    const w: WindowInfo = { id: '9', title: 't', appName: 'a', appId: null, pid: 1, bounds: { x: 1612, y: 100, width: 800, height: 600 } };
    expect(windowToLocal(w, display, 2560)).toEqual({ x: 100, y: 100, width: 800, height: 600 });
    // Windows: bounds in physical px, viewport in DIPs at 150%.
    const winDisplay = { ...display, bounds: { x: 0, y: 0, width: 3840, height: 2160 } };
    const w2 = { ...w, bounds: { x: 300, y: 150, width: 1500, height: 900 } };
    expect(windowToLocal(w2, winDisplay, 2560)).toEqual({ x: 200, y: 100, width: 1000, height: 600 });
  });

  it('hit-tests front-most first', () => {
    const items = [
      { id: 'front', rect: { x: 0, y: 0, width: 10, height: 10 } },
      { id: 'back', rect: { x: 0, y: 0, width: 100, height: 100 } },
    ];
    expect(hitTest(items, 5, 5)?.id).toBe('front');
    expect(hitTest(items, 50, 50)?.id).toBe('back');
    expect(hitTest(items, 500, 5)).toBeUndefined();
  });
});
