# Versioning and releases

## SemVer

FrameLy follows [Semantic Versioning](https://semver.org/): `MAJOR.MINOR.PATCH`. While the version is
`0.y.z`, behavior may change in `MINOR` versions. Pre-releases use a suffix (`0.2.0-beta.1`) and are
published as GitHub pre-releases.

The version lives in three files that must agree (CI and the release workflow check with
`scripts/release/check-version.mjs`):

- `package.json`
- `src-tauri/Cargo.toml` (`[package] version`)
- `src-tauri/tauri.conf.json` (shown in Settings › About)

## Publish a version

1. Bump the version in the three files.
2. In `CHANGELOG.md`, move `[Unreleased]` into `## [X.Y.Z] - YYYY-MM-DD` (with a one-line summary)
   and update the link references at the bottom. Translate the section into
   `website/src/content/changelog/<locale>.md` (optional: without a translation the site shows English).
3. Check: `node scripts/release/check-version.mjs vX.Y.Z`.
4. Commit (`FrameLy X.Y.Z: <summary>`), `git tag -a vX.Y.Z -m "FrameLy X.Y.Z"`,
   `git push origin main vX.Y.Z`.

The **Release** workflow (`.github/workflows/release.yml`, on `v*` tags) then:

1. checks the tag, the three files and the changelog; checks that the updater signing key secret
   exists; fails if a release with that tag already exists (**releases are never overwritten** —
   fix forward with a new version);
2. builds the installers: `macos-15` + Xcode 26 → `FrameLy_X.Y.Z_aarch64.dmg` and the updater
   bundle; `windows-latest` → `FrameLy_X.Y.Z_x64-setup.exe` (NSIS); signs them when signing
   secrets exist ([signing.md](signing.md));
3. only if **every** build passed: `SHA256SUMS.txt`, `latest.json` (`scripts/release/latest-json.mjs`,
   requires both platforms and their signatures), release notes from the changelog
   (`scripts/release/notes.mjs`), and the GitHub release;
4. triggers the **Website** workflow on `main` so the download page lists the new files
   ([website.md](website.md)).

Dry run without publishing: *Actions › Release › Run workflow* (installers kept as run artifacts).

### Release assets

`FrameLy_X.Y.Z_aarch64.dmg`, `FrameLy_X.Y.Z_aarch64.app.tar.gz` + `.sig` (macOS updater),
`FrameLy_X.Y.Z_x64-setup.exe` + `.sig`, `latest.json`, `SHA256SUMS.txt`.

## Workflows

| Workflow | Trigger | Does |
|---|---|---|
| `ci.yml` (CI) | push to `main`, PRs (not docs/website/`*.md`) | Checks and tests on macOS and Windows ([development.md](development.md#everyday-commands)) |
| `release.yml` (Release) | tag `v*`, manual | Builds, signs, publishes ([above](#publish-a-version)) |
| `site.yml` (Website) | push touching `website/**` or `CHANGELOG.md`, after a release, manual | Builds and deploys GitHub Pages |
| `installer-ui.yml` (Installer screens) | PRs touching `src-tauri/installer/**`, manual | Screenshots of every installer screen |
| `updater-e2e.yml` (Updater end-to-end) | PRs touching the updater or installer, manual | Installs an old build and updates it to a new one |

Repository configuration (names only — values never go in the repo): secrets
`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, the optional signing secrets in
[signing.md](signing.md), and the variable `FRAMELY_CLOUD_URL`.

## Automatic updates

`tauri-plugin-updater`, wired in `src-tauri/src/updates.rs` (UI: `src/lib/stores/updates.svelte.ts`,
`UpdateNotice.svelte`):

- reads `https://github.com/steinertruthy/frame-ly/releases/latest/download/latest.json`
  (`FRAMELY_UPDATE_URL` overrides it for tests);
- checks ~45 s after launch and every 6 h, unless turned off (Settings › About) or in "Local only"
  mode; `framely --update` checks and installs now;
- downloads only when the person chooses "Update and restart"; the package signature is verified
  against the public key in `tauri.conf.json › plugins.updater.pubkey`;
- never interrupts a recording: it installs when the recording ends;
- macOS replaces the `.app` and relaunches; Windows runs the NSIS installer in passive `/UPDATE`
  mode, which updates in place, keeps data and reopens FrameLy.

### Signing keys

- Public key: `src-tauri/tauri.conf.json › plugins.updater.pubkey`.
- Private key and password: **never in git**. Repository secrets `TAURI_SIGNING_PRIVATE_KEY` and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, plus an offline copy outside the repo
  (`~/.tauri/framely-updater.key`, `~/.tauri/framely-updater.password`). Without the private key no
  update that installed versions accept can ever be published — keep a safe backup.

### End-to-end test

*Actions › Updater end-to-end*: generates a throwaway key, builds 0.0.1 and a signed 0.0.2, serves
`latest.json` on `127.0.0.1`, runs `framely --update` on the old one and checks it downloaded,
verified, installed and reopened as 0.0.2. On Windows it also checks that user data survives and
that uninstalling is clean.

### Limitations

- Versions without the updater (0.1.0, 0.2.0) must download the first updater-enabled version by hand.
- macOS without Developer ID: the ad-hoc signature changes with every version, so macOS may ask
  for Screen Recording (and Microphone) again after an update.

## Windows installer (NSIS)

FrameLy's own installer, start to finish, on top of Tauri's NSIS support:

- `src-tauri/installer/installer.nsi` — Tauri's template (tauri-cli 2.12.4) with the pages replaced
  by FrameLy's screens. All install/update (`/UPDATE`), passive (`/P`), silent (`/S`) and uninstall
  logic is Tauri's; FrameLy changes are marked `FrameLy:`. **When upgrading `@tauri-apps/cli`,
  diff against the new template.**
- `src-tauri/installer/ui.nsh` — full-window screens in the app's colors: welcome (install location,
  desktop shortcut, state: new install / update / reinstall / older version), progress, finish with
  "Open FrameLy now"; uninstaller confirmation (with "delete data"), progress and finish. Flat
  rounded buttons; native buttons hidden but Enter/Esc still work; dark title bar on Windows 10
  2004+/11.
- `src-tauri/installer/lang/*.nsh` — Tauri + FrameLy strings in the app's five languages.
- `src-tauri/installer/hooks.nsh` — stops `framely-llm.exe` before install/uninstall; with "delete
  data" removes the `FrameLy/ai/*` credentials.
- `src-tauri/installer/logo.bmp` — generated by `node scripts/windows-installer-art.mjs`.
- `src-tauri/installer/preview/preview.nsi` — the same screens with a simulated install
  (`/PAGE=`, `/STATE=` open each screen); the *Installer screens* workflow screenshots them.

Installing over an older version updates in place: settings, library, transcripts and models in
`%APPDATA%\app.framely.desktop` are kept.

**Local build on macOS:** Homebrew's `makensis` crashes on macOS 26; use the Docker wrapper:
`ln -s "$PWD/scripts/makensis-docker.sh" ~/bin/makensis` (with `~/bin` first on `PATH`), then
`pnpm build:windows` ([development.md](development.md#windows)).
