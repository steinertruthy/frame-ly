import { describe, expect, it } from 'vitest';
import { groupByDate, stem } from './library';
import type { LibraryItem } from '$lib/ipc/types';

const item = (id: number, createdAt: Date): LibraryItem => ({
  id,
  kind: 'image',
  path: `/x/${id}.png`,
  name: `${id}.png`,
  createdAt: createdAt.getTime(),
  size: 1,
  width: null,
  height: null,
  durationMs: null,
});

describe('library grouping', () => {
  const now = new Date(2026, 10, 11, 15, 0); // 11 Nov 2026, 15:00 local

  it('groups by today / yesterday / week / month', () => {
    const items = [
      item(1, new Date(2026, 10, 11, 9)),
      item(2, new Date(2026, 10, 11, 0, 5)),
      item(3, new Date(2026, 10, 10, 23)),
      item(4, new Date(2026, 10, 6, 12)),
      item(5, new Date(2026, 10, 1)),
      item(6, new Date(2026, 9, 30)),
      item(7, new Date(2025, 9, 2)),
    ];
    const g = groupByDate(items, now);
    expect(g.map((x) => [x.key, x.items.map((i) => i.id)])).toEqual([
      ['today', [1, 2]],
      ['yesterday', [3]],
      ['week', [4]],
      ['month', [5]],
      ['month', [6]],
      ['month', [7]],
    ]);
    expect(g[3]!.month).toEqual(new Date(2026, 10, 1));
    expect(g[5]!.id).toBe('2025-9');
  });

  it('keeps file extensions out of the editable name', () => {
    expect(stem('FrameLy 2026-11-11 at 21.31.28.mp4')).toBe('FrameLy 2026-11-11 at 21.31.28');
    expect(stem('.hidden')).toBe('.hidden');
    expect(stem('noext')).toBe('noext');
  });
});
