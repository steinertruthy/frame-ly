import type { Os } from './platform';

export type Arch = 'arm64' | 'x64' | 'universal';
export type Format = 'dmg' | 'exe' | 'msi' | 'zip';

export interface Asset {
  name: string;
  url: string;
  size: number;
  os: Os;
  arch: Arch;
  format: Format;
  /** Installer (the main download) or a portable archive. */
  kind: 'installer' | 'portable';
  sha256: string | null;
}

export interface Release {
  tag: string;
  version: string;
  name: string;
  date: string;
  url: string;
  prerelease: boolean;
  assets: Asset[];
  checksumsUrl: string | null;
}

export interface ReleaseData {
  repo: string;
  fetchedAt: string | null;
  releases: Release[];
}

/** What a GitHub release asset is, from its file name (Tauri's naming). Unknown files → null. */
export function classify(name: string): Pick<Asset, 'os' | 'arch' | 'format' | 'kind'> | null {
  const n = name.toLowerCase();
  const arch: Arch | null = /universal/.test(n)
    ? 'universal'
    : /aarch64|arm64/.test(n)
      ? 'arm64'
      : /x64|x86_64|amd64/.test(n)
        ? 'x64'
        : null;
  if (!arch) return null;
  if (n.endsWith('.dmg')) return { os: 'macos', arch, format: 'dmg', kind: 'installer' };
  if (n.endsWith('-setup.exe')) return { os: 'windows', arch, format: 'exe', kind: 'installer' };
  if (n.endsWith('.msi')) return { os: 'windows', arch, format: 'msi', kind: 'installer' };
  if (n.endsWith('.zip') && /portable/.test(n)) return { os: 'windows', arch, format: 'zip', kind: 'portable' };
  return null;
}

/** "sha256  file" lines (sha256sum / shasum format). */
export function parseChecksums(text: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const line of text.split(/\r?\n/)) {
    const m = /^([a-f0-9]{64})\s+\*?(.+)$/i.exec(line.trim());
    if (m?.[1] && m[2]) out.set(m[2].trim(), m[1].toLowerCase());
  }
  return out;
}

export interface Latest {
  release: Release;
  installers: Asset[];
  portable: Asset[];
}

/**
 * Newest stable release with an installer for `os` (falls back to a pre-release only when no stable
 * one exists). Platforms can be on different versions.
 */
export function latestFor(data: ReleaseData, os: Os): Latest | null {
  const withOs = data.releases.filter((r) => r.assets.some((a) => a.os === os && a.kind === 'installer'));
  const release = withOs.find((r) => !r.prerelease) ?? withOs[0];
  if (!release) return null;
  return {
    release,
    installers: release.assets.filter((a) => a.os === os && a.kind === 'installer'),
    portable: release.assets.filter((a) => a.os === os && a.kind === 'portable'),
  };
}

export function formatBytes(bytes: number, locale: string): string {
  const mb = bytes / (1024 * 1024);
  return `${new Intl.NumberFormat(locale, { maximumFractionDigits: mb >= 100 ? 0 : 1 }).format(mb)} MB`;
}

export function formatDate(iso: string, locale: string): string {
  return new Intl.DateTimeFormat(locale, { year: 'numeric', month: 'long', day: 'numeric', timeZone: 'UTC' }).format(new Date(iso));
}
