import type { ImageMetadata } from 'astro';
import type { Locale } from '../i18n/locales';

/** Real captures of the app (scripts/optimize-screens.mjs). pt-BR has its own; others use English. */
const images = import.meta.glob<{ default: ImageMetadata }>('../assets/screens/*/*.webp', { eager: true });

export type ShotName =
  | 'home'
  | 'library'
  | 'transcripts'
  | 'transcript'
  | 'summary'
  | 'topics'
  | 'models'
  | 'editor'
  | 'panel'
  | 'recbar'
  | 'preview';

export function shot(locale: Locale, name: ShotName): ImageMetadata {
  const dir = locale === 'pt-BR' ? 'pt-BR' : 'en';
  const img = images[`../assets/screens/${dir}/${name}.webp`] ?? images[`../assets/screens/en/${name}.webp`];
  if (!img) throw new Error(`missing screenshot ${name}`);
  return img.default;
}
