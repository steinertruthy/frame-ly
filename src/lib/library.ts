/** Pure helpers for the library UI. */
import type { LibraryItem } from '$lib/ipc/types';

export type GroupKey = 'today' | 'yesterday' | 'week' | 'month';

export interface Group {
  id: string;
  key: GroupKey;
  /** First day of the month, for `month` groups. */
  month?: Date;
  items: LibraryItem[];
}

function startOfDay(d: Date): number {
  return new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
}

/**
 * Groups items (already sorted by date) into Today / Yesterday / This week (last 7 days) /
 * one group per calendar month — in the local time zone.
 */
export function groupByDate(items: LibraryItem[], now: Date): Group[] {
  const today = startOfDay(now);
  const day = 86_400_000;
  const groups: Group[] = [];
  for (const item of items) {
    const t = item.createdAt;
    let id: string;
    let key: GroupKey;
    let month: Date | undefined;
    if (t >= today) [id, key] = ['today', 'today'];
    else if (t >= today - day) [id, key] = ['yesterday', 'yesterday'];
    else if (t >= today - 6 * day) [id, key] = ['week', 'week'];
    else {
      const d = new Date(t);
      month = new Date(d.getFullYear(), d.getMonth(), 1);
      [id, key] = [`${d.getFullYear()}-${d.getMonth()}`, 'month'];
    }
    const last = groups.at(-1);
    if (last?.id === id) last.items.push(item);
    else groups.push({ id, key, month, items: [item] });
  }
  return groups;
}

/** Name without extension (what the user edits when renaming). */
export function stem(name: string): string {
  const i = name.lastIndexOf('.');
  return i > 0 ? name.slice(0, i) : name;
}
