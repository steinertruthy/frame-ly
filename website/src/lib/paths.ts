import { pagePath, type Locale, type PageId } from '../i18n/locales';

/** Base path from astro.config (e.g. "/frame-ly"), without a trailing slash. */
export const BASE = import.meta.env.BASE_URL.replace(/\/$/, '');

/** Site-relative URL with the base path. */
export function href(path: string): string {
  return `${BASE}${path.startsWith('/') ? path : `/${path}`}`;
}

export function pageHref(page: PageId, locale: Locale, hash = ''): string {
  return href(pagePath(page, locale)) + hash;
}

export const REPO_URL = 'https://github.com/steinertruthy/frame-ly';
