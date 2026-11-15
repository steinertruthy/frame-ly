#!/usr/bin/env node
// Release gate: the tag (vX.Y.Z), package.json, src-tauri/Cargo.toml and tauri.conf.json must agree,
// the version must be SemVer, and CHANGELOG.md must have a dated section for it.
// Usage: node scripts/release/check-version.mjs [vX.Y.Z]   (without a tag: checks the files agree)
import { readFileSync } from 'node:fs';

const read = (p) => readFileSync(new URL(`../../${p}`, import.meta.url), 'utf8');
const semver = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/;

const pkg = JSON.parse(read('package.json')).version;
const conf = JSON.parse(read('src-tauri/tauri.conf.json')).version;
const cargo = /^\[package\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(read('src-tauri/Cargo.toml'))?.[1];
const tag = process.argv[2];

const errors = [];
if (!semver.test(pkg)) errors.push(`package.json version "${pkg}" is not SemVer`);
if (conf !== pkg) errors.push(`tauri.conf.json ${conf} ≠ package.json ${pkg}`);
if (cargo !== pkg) errors.push(`src-tauri/Cargo.toml ${cargo} ≠ package.json ${pkg}`);
if (tag && tag !== `v${pkg}`) errors.push(`tag ${tag} ≠ v${pkg}`);
const changelog = read('CHANGELOG.md');
const esc = pkg.replace(/[.+]/g, '\\$&');
if (!new RegExp(`^## \\[${esc}\\] - \\d{4}-\\d{2}-\\d{2}$`, 'm').test(changelog)) {
  errors.push(`CHANGELOG.md has no "## [${pkg}] - YYYY-MM-DD" section`);
}

if (errors.length) {
  for (const e of errors) console.error(`✗ ${e}`);
  process.exit(1);
}
console.log(`✓ version ${pkg}${tag ? ` matches ${tag}` : ''}`);
if (process.env.GITHUB_OUTPUT) {
  const { appendFileSync } = await import('node:fs');
  appendFileSync(process.env.GITHUB_OUTPUT, `version=${pkg}\nprerelease=${pkg.includes('-')}\n`);
}
