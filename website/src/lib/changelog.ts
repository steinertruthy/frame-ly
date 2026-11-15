/** Minimal Keep a Changelog parser for the website (CHANGELOG.md is the source of truth). */
export type SectionKind = 'added' | 'changed' | 'fixed' | 'known' | 'other';

export interface Entry {
  version: string;
  date: string | null;
  intro: string[];
  sections: { kind: SectionKind; title: string; items: string[] }[];
}

const KINDS: Record<string, SectionKind> = {
  added: 'added',
  changed: 'changed',
  improved: 'changed',
  fixed: 'fixed',
  'known limitations': 'known',
};

export function parseChangelog(md: string): Entry[] {
  const entries: Entry[] = [];
  let cur: Entry | null = null;
  let section: Entry['sections'][number] | null = null;
  for (const raw of md.split(/\r?\n/)) {
    const line = raw.trimEnd();
    const v = /^##\s+\[([^\]]+)\](?:\s+-\s+(\d{4}-\d{2}-\d{2}))?/.exec(line);
    if (v) {
      section = null;
      cur = v[1]!.toLowerCase() === 'unreleased' ? null : { version: v[1]!, date: v[2] ?? null, intro: [], sections: [] };
      if (cur) entries.push(cur);
      continue;
    }
    if (!cur) continue;
    const h = /^###\s+(.+)$/.exec(line);
    if (h) {
      const title = h[1]!.trim();
      section = { kind: KINDS[title.toLowerCase()] ?? 'other', title, items: [] };
      cur.sections.push(section);
      continue;
    }
    const item = /^\s*[-*]\s+(.+)$/.exec(line);
    if (item && section) section.items.push(item[1]!);
    else if (line && !line.startsWith('[') && !section) cur.intro.push(line);
  }
  return entries;
}

/** Entries in a locale: translated versions where available, English for the rest. */
export function localized(english: Entry[], translated: Entry[]): (Entry & { translated: boolean })[] {
  const byVersion = new Map(translated.map((e) => [e.version, e]));
  return english.map((e) => {
    const tr = byVersion.get(e.version);
    return tr ? { ...tr, date: e.date, translated: true } : { ...e, translated: false };
  });
}

const escape = (s: string) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');

/** Inline Markdown subset: `code`, **bold**, [text](https://…). Everything else is escaped. */
export function inline(md: string): string {
  return escape(md)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\[([^\]]+)\]\((https:\/\/[^)\s]+)\)/g, '<a href="$2">$1</a>');
}
