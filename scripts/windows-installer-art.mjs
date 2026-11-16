#!/usr/bin/env node
// Bitmap for the Windows (NSIS) installer's own screens (installer/ui.nsh):
//   src-tauri/installer/logo.bmp     192×192  the mark, on the app's background colour
// NSIS needs 24-bit BMPs. Text is rendered with the fonts of the machine
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

// Logo for the custom screens: drawn at 2x and scaled down by Windows, on the app's background
// colour (BMPs have no transparency).
await render(
  `<svg xmlns="http://www.w3.org/2000/svg" width="192" height="192"><rect width="192" height="192" fill="#0b0e14"/></svg>`,
  [{ input: await tile(192), left: 0, top: 0 }],
  192,
  192,
  'src-tauri/installer/logo.bmp',
  process.env.PREVIEW_DIR && `${process.env.PREVIEW_DIR}/logo.png`,
);
