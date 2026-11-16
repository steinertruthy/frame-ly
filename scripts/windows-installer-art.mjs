#!/usr/bin/env node
// Bitmaps for the Windows (NSIS) installer, in FrameLy's identity:
//   src-tauri/installer/sidebar.bmp  164×314  Welcome and Finish pages
//   src-tauri/installer/header.bmp   150×57   header of the other pages (white header bar)
// NSIS needs 24-bit BMPs at exactly these sizes. Text is rendered with the fonts of the machine
// that runs this, so the output is committed. Uses the website's sharp: run `pnpm -C website i` first.
// Usage: node scripts/windows-installer-art.mjs
import { createRequire } from 'node:module';
import { readFileSync, writeFileSync } from 'node:fs';

const require = createRequire(new URL('../website/package.json', import.meta.url));
const sharp = require('sharp');
const root = new URL('../', import.meta.url);
const logo = readFileSync(new URL('assets/logo.svg', root));
const font = "font-family=\"Segoe UI Variable Display, SF Pro Display, Helvetica Neue, Arial, sans-serif\"";

/** Raw RGB → 24-bit bottom-up BMP. */
function bmp(rgb, w, h) {
  const row = Math.ceil((w * 3) / 4) * 4;
  const size = 54 + row * h;
  const b = Buffer.alloc(size);
  b.write('BM', 0);
  b.writeUInt32LE(size, 2);
  b.writeUInt32LE(54, 10);
  b.writeUInt32LE(40, 14);
  b.writeInt32LE(w, 18);
  b.writeInt32LE(h, 22);
  b.writeUInt16LE(1, 26);
  b.writeUInt16LE(24, 28);
  b.writeUInt32LE(row * h, 34);
  b.writeInt32LE(3780, 38);
  b.writeInt32LE(3780, 42);
  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const s = (y * w + x) * 3;
      const d = 54 + (h - 1 - y) * row + x * 3;
      b[d] = rgb[s + 2];
      b[d + 1] = rgb[s + 1];
      b[d + 2] = rgb[s];
    }
  }
  return b;
}

async function render(svg, overlays, w, h, out, preview) {
  const img = sharp(Buffer.from(svg)).composite(overlays).flatten({ background: '#ffffff' });
  const png = await img.png().toBuffer();
  const { data } = await sharp(png).removeAlpha().raw().toBuffer({ resolveWithObject: true });
  writeFileSync(new URL(out, root), bmp(data, w, h));
  if (preview) writeFileSync(preview, png);
  console.log(out);
}

const tile = (px) => sharp(logo).resize(px, px).png().toBuffer();

// Sidebar: the app's navy, one soft light behind the mark, the wordmark below.
await render(
  `<svg xmlns="http://www.w3.org/2000/svg" width="164" height="314">
    <defs>
      <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
        <stop offset="0" stop-color="#14295a"/><stop offset="0.55" stop-color="#0a1428"/><stop offset="1" stop-color="#05080f"/>
      </linearGradient>
      <radialGradient id="glow" cx="0.5" cy="0.3" r="0.55">
        <stop offset="0" stop-color="#3a6fe0" stop-opacity="0.38"/><stop offset="1" stop-color="#3a6fe0" stop-opacity="0"/>
      </radialGradient>
    </defs>
    <rect width="164" height="314" fill="url(#bg)"/>
    <rect width="164" height="314" fill="url(#glow)"/>
    <rect x="163" width="1" height="314" fill="#ffffff" fill-opacity="0.08"/>
    <text x="82" y="168" text-anchor="middle" ${font} font-size="22" font-weight="600" letter-spacing="-0.4" fill="#edf0f7">FrameLy</text>
    <rect x="66" y="184" width="32" height="1" fill="#6c9de6" fill-opacity="0.6"/>
  </svg>`,
  [{ input: await tile(76), left: 44, top: 56 }],
  164,
  314,
  'src-tauri/installer/sidebar.bmp',
  process.env.PREVIEW_DIR && `${process.env.PREVIEW_DIR}/sidebar.png`,
);

// Header: sits on the installer's white header bar, aligned right.
await render(
  `<svg xmlns="http://www.w3.org/2000/svg" width="150" height="57">
    <rect width="150" height="57" fill="#ffffff"/>
    <text x="140" y="35" text-anchor="end" ${font} font-size="17" font-weight="600" letter-spacing="-0.3" fill="#0b1424">FrameLy</text>
  </svg>`,
  [{ input: await tile(32), left: 40, top: 12 }],
  150,
  57,
  'src-tauri/installer/header.bmp',
  process.env.PREVIEW_DIR && `${process.env.PREVIEW_DIR}/header.png`,
);
