// Open Graph images (1200×630) per locale: brand, headline and a real app capture.
// Generated once and committed (text rendering depends on the fonts of the machine that runs it).
// Usage: node scripts/og.mjs
import sharp from 'sharp';
import { mkdir, readFile } from 'node:fs/promises';

const root = new URL('../', import.meta.url);
const titles = {
  en: ['Capture it. Record it.', 'Understand it.'],
  'pt-BR': ['Capture. Grave.', 'Entenda.'],
  es: ['Captura. Graba.', 'Entiende.'],
  fr: ['Capturez. Enregistrez.', 'Comprenez.'],
  de: ['Festhalten. Aufnehmen.', 'Verstehen.'],
};
const subs = {
  en: 'Screen recording, screenshots and on-device transcription',
  'pt-BR': 'Gravação de tela, screenshots e transcrição no seu computador',
  es: 'Grabación de pantalla, capturas y transcripción en tu ordenador',
  fr: 'Enregistrement d’écran, captures et transcription locale',
  de: 'Bildschirmaufnahme, Screenshots und lokale Transkription',
};
const wrap = (s, n) => s.split(' ').reduce((lines, w) => {
  const last = lines[lines.length - 1];
  if (last && (last + ' ' + w).length <= n) lines[lines.length - 1] = last + ' ' + w;
  else lines.push(w);
  return lines;
}, []);
const esc = (s) => s.replace(/&/g, '&amp;').replace(/</g, '&lt;');
const logo = (await readFile(new URL('../assets/logo.svg', root))).toString();
await mkdir(new URL('public/og/', root), { recursive: true });
await sharp(Buffer.from(logo)).resize(180, 180).png().toFile(new URL('public/apple-touch-icon.png', root).pathname);

for (const [locale, [l1, l2]] of Object.entries(titles)) {
  const dir = locale === 'pt-BR' ? 'pt-BR' : 'en';
  const shot = await sharp(new URL(`src/assets/screens/${dir}/topics.webp`, root).pathname).resize({ width: 760 }).png().toBuffer();
  const fs = Math.min(58, Math.floor(540 / (Math.max(l1.length, l2.length) * 0.56)));
  const bg = Buffer.from(`<svg xmlns="http://www.w3.org/2000/svg" width="1200" height="630">
    <defs>
      <radialGradient id="a" cx="0.75" cy="0.1" r="0.8"><stop offset="0" stop-color="#1d3b75" stop-opacity="0.9"/><stop offset="1" stop-color="#04060c" stop-opacity="0"/></radialGradient>
      <linearGradient id="t" x1="0" y1="0" x2="0" y2="1"><stop offset="0.2" stop-color="#ffffff"/><stop offset="1" stop-color="#a9c0ea"/></linearGradient>
    </defs>
    <rect width="1200" height="630" fill="#04060c"/>
    <rect width="1200" height="630" fill="url(#a)"/>
    <text x="72" y="250" font-family="SF Pro Display, Helvetica Neue, Arial, sans-serif" font-size="${fs}" font-weight="700" letter-spacing="-2" fill="url(#t)">${esc(l1)}</text>
    <text x="72" y="${250 + Math.round(fs * 1.18)}" font-family="SF Pro Display, Helvetica Neue, Arial, sans-serif" font-size="${fs}" font-weight="700" letter-spacing="-2" fill="url(#t)">${esc(l2)}</text>
    ${wrap(subs[locale], 34).map((line, i) => `<text x="72" y="${384 + i * 32}" font-family="SF Pro Text, Helvetica Neue, Arial, sans-serif" font-size="22" fill="#a1abc0">${esc(line)}</text>`).join('')}
    <text x="140" y="138" font-family="SF Pro Display, Helvetica Neue, Arial, sans-serif" font-size="30" font-weight="650" fill="#eaeef7">FrameLy</text>
    <text x="72" y="560" font-family="SF Pro Text, Helvetica Neue, Arial, sans-serif" font-size="18" fill="#6f7a91">macOS · Windows</text>
  </svg>`);
  const mark = await sharp(Buffer.from(logo)).resize(56, 56).png().toBuffer();
  await sharp(bg)
    .composite([
      { input: mark, left: 72, top: 98 },
      { input: shot, left: 640, top: 170 },
    ])
    .png({ compressionLevel: 9 })
    .toFile(new URL(`public/og/${locale}.png`, root).pathname);
  console.log(`og/${locale}.png`);
}
