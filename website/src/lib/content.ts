import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { localized, parseChangelog } from './changelog';
import type { Locale } from '../i18n/locales';
import releasesJson from '../data/releases.json';
import type { ReleaseData } from './releases';

export const releases = releasesJson as ReleaseData;

const translations = import.meta.glob<string>('../content/changelog/*.md', { query: '?raw', import: 'default', eager: true });

/** CHANGELOG.md at the repository root (the build runs from website/). */
export function changelog(locale: Locale) {
  const english = parseChangelog(readFileSync(resolve(process.cwd(), '../CHANGELOG.md'), 'utf8'));
  const raw = translations[`../content/changelog/${locale}.md`];
  return localized(english, raw ? parseChangelog(raw) : []);
}

/** Date the legal texts were last reviewed (shown on those pages). */
export const LEGAL_UPDATED = '2026-11-15';
