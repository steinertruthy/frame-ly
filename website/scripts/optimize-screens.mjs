// Converts real app captures (PNG from `screencapture -o -l <window>`) into high-quality WebP
// sources in src/assets/screens/<locale>/. Astro then derives AVIF/WebP sizes at build time.
// Usage: node scripts/optimize-screens.mjs <dir-with-pngs> <locale>
import sharp from 'sharp';
import { readdir, mkdir } from 'node:fs/promises';
import { join, basename } from 'node:path';

const [src, locale] = process.argv.slice(2);
if (!src || !locale) throw new Error('usage: optimize-screens.mjs <dir> <locale>');
const out = new URL(`../src/assets/screens/${locale}/`, import.meta.url).pathname;
await mkdir(out, { recursive: true });
for (const f of (await readdir(src)).filter((f) => f.endsWith('.png'))) {
  const file = join(out, basename(f, '.png') + '.webp');
  await sharp(join(src, f)).resize({ width: 2000, withoutEnlargement: true }).webp({ quality: 90, alphaQuality: 100, effort: 6 }).toFile(file);
  console.log(file);
}
