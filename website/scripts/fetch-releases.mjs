// Builds src/data/releases.json from the GitHub Releases API at build time, so visitors never hit
// the API (no rate limits) and the page only links to files that really exist. When the API can't be
// reached the committed snapshot is kept and the build continues.
import { readFile, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const REPO = process.env.FRAMELY_REPO ?? 'steinertruthy/frame-ly';
const out = fileURLToPath(new URL('../src/data/releases.json', import.meta.url));
const headers = { Accept: 'application/vnd.github+json', 'X-GitHub-Api-Version': '2022-11-28', 'User-Agent': 'framely-website-build' };
if (process.env.GITHUB_TOKEN) headers.Authorization = `Bearer ${process.env.GITHUB_TOKEN}`;

// Kept in sync with classify() in src/lib/releases.ts (tested there).
function classify(name) {
  const n = name.toLowerCase();
  const arch = /universal/.test(n) ? 'universal' : /aarch64|arm64/.test(n) ? 'arm64' : /x64|x86_64|amd64/.test(n) ? 'x64' : null;
  if (!arch) return null;
  if (n.endsWith('.dmg')) return { os: 'macos', arch, format: 'dmg', kind: 'installer' };
  if (n.endsWith('-setup.exe')) return { os: 'windows', arch, format: 'exe', kind: 'installer' };
  if (n.endsWith('.msi')) return { os: 'windows', arch, format: 'msi', kind: 'installer' };
  if (n.endsWith('.zip') && /portable/.test(n)) return { os: 'windows', arch, format: 'zip', kind: 'portable' };
  return null;
}

function parseChecksums(text) {
  const map = new Map();
  for (const line of text.split(/\r?\n/)) {
    const m = /^([a-f0-9]{64})\s+\*?(.+)$/i.exec(line.trim());
    if (m) map.set(m[2].trim(), m[1].toLowerCase());
  }
  return map;
}

// Right after a release is published the API can still list the previous state: when the
// release workflow passes EXPECT_TAG, wait (up to ~3 min) until that release is listed.
async function listReleases() {
  const expect = process.env.EXPECT_TAG;
  for (let attempt = 0; ; attempt++) {
    const res = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=30`, { headers });
    if (!res.ok) throw new Error(`GitHub API ${res.status}`);
    const list = await res.json();
    if (!expect || list.some((r) => r.tag_name === expect && !r.draft)) return list;
    if (attempt >= 18) throw new Error(`release ${expect} not listed yet`);
    console.log(`waiting for ${expect} to appear in the API…`);
    await new Promise((r) => setTimeout(r, 10_000));
  }
}

async function main() {
  const list = await listReleases();
  const releases = [];
  for (const r of list) {
    if (r.draft) continue;
    const sums = r.assets.find((a) => a.name === 'SHA256SUMS.txt');
    let checksums = new Map();
    if (sums) {
      const s = await fetch(sums.browser_download_url, { headers: { 'User-Agent': headers['User-Agent'] } });
      if (s.ok) checksums = parseChecksums(await s.text());
    }
    const assets = [];
    for (const a of r.assets) {
      const kind = classify(a.name);
      if (!kind || a.state !== 'uploaded') continue;
      const digest = typeof a.digest === 'string' && a.digest.startsWith('sha256:') ? a.digest.slice(7) : null;
      assets.push({ name: a.name, url: a.browser_download_url, size: a.size, ...kind, sha256: checksums.get(a.name) ?? digest });
    }
    releases.push({
      tag: r.tag_name,
      version: r.tag_name.replace(/^v/, ''),
      name: r.name || r.tag_name,
      date: r.published_at,
      url: r.html_url,
      prerelease: r.prerelease,
      assets,
      checksumsUrl: sums ? sums.browser_download_url : null,
    });
  }
  const data = { repo: REPO, fetchedAt: new Date().toISOString(), releases };
  await writeFile(out, JSON.stringify(data, null, 2) + '\n');
  console.log(`releases.json: ${releases.length} release(s) from ${REPO}`);
}

main().catch(async (e) => {
  const old = JSON.parse(await readFile(out, 'utf8').catch(() => '{"releases":[]}'));
  console.warn(`releases.json: kept the committed snapshot (${old.releases.length} release(s)) — ${e.message}`);
  if (process.env.REQUIRE_RELEASES === '1') process.exit(1);
});
