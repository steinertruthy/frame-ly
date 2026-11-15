import { describe, expect, it } from 'vitest';
import { classify, latestFor, parseChecksums, type ReleaseData } from './releases';

describe('classify', () => {
  it('maps Tauri bundle names to platform, architecture and format', () => {
    expect(classify('FrameLy_0.1.0_aarch64.dmg')).toEqual({ os: 'macos', arch: 'arm64', format: 'dmg', kind: 'installer' });
    expect(classify('FrameLy_0.1.0_x64.dmg')).toEqual({ os: 'macos', arch: 'x64', format: 'dmg', kind: 'installer' });
    expect(classify('FrameLy_0.1.0_universal.dmg')?.arch).toBe('universal');
    expect(classify('FrameLy_0.1.0_x64-setup.exe')).toEqual({ os: 'windows', arch: 'x64', format: 'exe', kind: 'installer' });
    expect(classify('FrameLy_0.1.0_arm64-setup.exe')?.arch).toBe('arm64');
    expect(classify('FrameLy_0.1.0_x64_en-US.msi')?.format).toBe('msi');
    expect(classify('FrameLy-0.1.0-x64-portable.zip')).toEqual({ os: 'windows', arch: 'x64', format: 'zip', kind: 'portable' });
  });
  it('ignores checksums, signatures and updater archives', () => {
    expect(classify('SHA256SUMS.txt')).toBeNull();
    expect(classify('FrameLy_aarch64.app.tar.gz')).toBeNull();
    expect(classify('FrameLy_0.1.0_x64-setup.exe.sig')).toBeNull();
  });
});

describe('parseChecksums', () => {
  it('reads sha256sum and shasum output', () => {
    const h = 'a'.repeat(64);
    const m = parseChecksums(`${h}  FrameLy_0.1.0_aarch64.dmg\n${'B'.repeat(64)} *FrameLy_0.1.0_x64-setup.exe\n\n`);
    expect(m.get('FrameLy_0.1.0_aarch64.dmg')).toBe(h);
    expect(m.get('FrameLy_0.1.0_x64-setup.exe')).toBe('b'.repeat(64));
  });
});

const asset = (name: string, os: 'macos' | 'windows', kind: 'installer' | 'portable' = 'installer') => ({
  name,
  url: `https://example.invalid/${name}`,
  size: 1,
  os,
  arch: 'x64' as const,
  format: 'dmg' as const,
  kind,
  sha256: null,
});
const rel = (version: string, prerelease: boolean, assets: ReturnType<typeof asset>[]) => ({
  tag: `v${version}`,
  version,
  name: version,
  date: '2026-11-15T00:00:00Z',
  url: '',
  prerelease,
  assets,
  checksumsUrl: null,
});

describe('latestFor', () => {
  const data: ReleaseData = {
    repo: 'x/y',
    fetchedAt: null,
    releases: [
      rel('0.3.0-beta.1', true, [asset('m3', 'macos')]),
      rel('0.2.0', false, [asset('w2', 'windows')]),
      rel('0.1.0', false, [asset('m1', 'macos'), asset('w1', 'windows'), asset('wz', 'windows', 'portable')]),
    ],
  };
  it('picks the newest stable release per platform (versions can differ)', () => {
    expect(latestFor(data, 'macos')?.release.version).toBe('0.1.0');
    expect(latestFor(data, 'windows')?.release.version).toBe('0.2.0');
  });
  it('separates installers from portable archives', () => {
    const w = latestFor({ ...data, releases: [data.releases[2]!] }, 'windows');
    expect(w?.installers.map((a) => a.name)).toEqual(['w1']);
    expect(w?.portable.map((a) => a.name)).toEqual(['wz']);
  });
  it('falls back to a pre-release only when nothing stable exists, and to null when nothing exists', () => {
    expect(latestFor({ ...data, releases: [data.releases[0]!] }, 'macos')?.release.version).toBe('0.3.0-beta.1');
    expect(latestFor({ ...data, releases: [] }, 'windows')).toBeNull();
  });
});
