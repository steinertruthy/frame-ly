# Website (`website/`)

Static site built with [Astro](https://astro.build), published to GitHub Pages at
<https://steinertruthy.github.io/frame-ly/> by `.github/workflows/site.yml` (on pushes to `main`
that touch `website/**` or `CHANGELOG.md`, after each release, or manually).

```bash
cd website
pnpm install
pnpm dev            # http://localhost:4321/frame-ly/
pnpm lint           # astro check + tsc
pnpm test           # vitest (OS detection, release parsing, i18n parity, changelog, links)
pnpm build          # fetches release data, then builds to dist/ (pnpm build:offline skips the fetch)
```

For a Claude Code preview, add a `website` entry to `.claude/launch.json` (git-ignored, per
machine) that runs `cd website && pnpm exec astro dev --port 4321 --host 127.0.0.1`.

## Structure

| Path | Contents |
|---|---|
| `website/src/pages/[lang]/`, `website/src/pages/index.astro` | Localized pages; `/` picks a language from the saved choice or the browser |
| `website/src/components/*.astro` | Sections (Hero, Features, Ai, Download, Faq…) |
| `website/src/i18n/<locale>.ts` | All site text; `en.ts` is the reference shape |
| `website/src/lib/` | Pure helpers with tests: releases, platform detection, changelog, links, paths |
| `website/src/content/changelog/<locale>.md` | Changelog translations (the root `CHANGELOG.md` is English) |
| `website/src/data/releases.json` | Download data generated at build time |
| `website/src/assets/screens/`, `website/public/` | Screenshots, icons, Open Graph images |
| `website/scripts/` | `fetch-releases.mjs`, `optimize-screens.mjs`, `og.mjs` |

## Rules

- **Languages** are exactly the app's (`pt-BR`, `en`, `es`, `fr`, `de`). Types and `i18n.test.ts`
  reject missing or extra keys. URLs are localized (`/pt-br/privacidade/`, `/de/datenschutz/`…) with
  `hreflang` alternates.
- **Downloads** come from the GitHub Releases API at build time (`website/scripts/fetch-releases.mjs`; set
  `GITHUB_TOKEN` to avoid rate limits). If the API can't be reached the committed snapshot is kept.
  The page links only to assets that exist; a platform without one shows "no installer yet".
- **Keep the site true to the app.** When you change an announced feature, update
  `website/src/i18n/*.ts` (all locales) together with `CHANGELOG.md`. Never name the AI provider behind
  FrameLy AI Cloud.
- **Screenshots** are real captures of the app (`screencapture -l <window>` while driving it with
  `framely --show …`), converted (in `website/`) with `node scripts/optimize-screens.mjs <dir> <locale>`.
  Astro derives AVIF/WebP at build time. Open Graph images: `node scripts/og.mjs`.
- **Legal pages** describe how the app works today and are marked as pending legal review.
- `SITE_URL` / `BASE_PATH` move the site to a custom domain without code changes.
