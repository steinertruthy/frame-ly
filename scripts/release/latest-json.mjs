#!/usr/bin/env node
// Writes latest.json for tauri-plugin-updater from the files of a release.
// Usage: node scripts/release/latest-json.mjs <version> <files-dir> <base-url> <notes.md> > latest.json
// Fails unless every platform has both its package and its .sig, so a release can't ship without
// working updates.
import { readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const [version, dir, baseUrl, notesFile] = process.argv.slice(2);
if (!version || !dir || !baseUrl) throw new Error('usage: latest-json.mjs <version> <dir> <base-url> [notes.md]');

const packages = {
  'darwin-aarch64': `FrameLy_${version}_aarch64.app.tar.gz`,
  'windows-x86_64': `FrameLy_${version}_x64-setup.exe`,
};
const platforms = {};
for (const [platform, file] of Object.entries(packages)) {
  const sig = join(dir, `${file}.sig`);
  if (!existsSync(join(dir, file)) || !existsSync(sig)) {
    console.error(`✗ missing ${file} or its .sig`);
    process.exit(1);
  }
  const entry = { signature: readFileSync(sig, 'utf8').trim(), url: `${baseUrl.replace(/\/$/, '')}/${file}` };
  platforms[platform] = entry;
  // The updater looks for "<os>-<arch>-<installer>" first.
  platforms[`${platform}-${platform.startsWith('darwin') ? 'app' : 'nsis'}`] = entry;
}

// Only the changelog part of the notes (installation notes are for the GitHub page).
const notes = notesFile ? readFileSync(notesFile, 'utf8').split(/\n---\s*\n/)[0].trim() : '';
console.log(JSON.stringify({ version, notes, pub_date: new Date().toISOString(), platforms }, null, 2));
