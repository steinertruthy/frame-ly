#!/usr/bin/env node
// Release notes for a version: its CHANGELOG.md section plus install and verification notes.
// Usage: node scripts/release/notes.mjs <version>  > notes.md
import { readFileSync } from 'node:fs';

const version = process.argv[2];
if (!version) throw new Error('usage: notes.mjs <version>');
const md = readFileSync(new URL('../../CHANGELOG.md', import.meta.url), 'utf8');
const start = md.search(new RegExp(`^## \\[${version.replace(/[.+]/g, '\\$&')}\\]`, 'm'));
if (start < 0) throw new Error(`no CHANGELOG section for ${version}`);
const rest = md.slice(start).split('\n').slice(1);
const end = rest.findIndex((l) => /^## \[/.test(l) || /^\[[^\]]+\]: /.test(l));
const body = (end < 0 ? rest : rest.slice(0, end)).join('\n').trim();

console.log(`${body}

---

### Downloads

| System | File | Requirements |
|---|---|---|
| macOS (Apple Silicon) | \`FrameLy_${version}_aarch64.dmg\` | macOS 14 or later; Apple Intelligence features need macOS 26 |
| Windows (x64) | \`FrameLy_${version}_x64-setup.exe\` | Windows 10 22H2 or Windows 11, 64-bit Intel/AMD |

**First launch.** These builds are not signed with an Apple Developer ID or a Windows code-signing
certificate yet.
- macOS: open FrameLy once, then System Settings › Privacy & Security › **Open Anyway**.
- Windows: if SmartScreen warns, choose **More info › Run anyway**.

**Verify the download** against \`SHA256SUMS.txt\`:
\`shasum -a 256 -c SHA256SUMS.txt --ignore-missing\` (macOS) or \`Get-FileHash <file>\` (PowerShell).

Website: https://steinertruthy.github.io/frame-ly/`);
