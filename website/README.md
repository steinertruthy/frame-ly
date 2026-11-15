# FrameLy — website

Static site built with [Astro](https://astro.build), published to GitHub Pages at
<https://steinertruthy.github.io/frame-ly/> by `.github/workflows/site.yml`.

```bash
pnpm install
pnpm dev            # http://localhost:4321/frame-ly/
pnpm lint           # astro check + tsc
pnpm test           # vitest (OS detection, release parsing, i18n parity, changelog)
pnpm build          # fetches release data, then builds to dist/
```

- **Languages:** exactly the app's (`pt-BR`, `en`, `es`, `fr`, `de`), in `src/i18n/`. English is the
  reference shape; the type system and `i18n.test.ts` reject missing or extra keys. URLs are
  localized (`/pt-br/privacidade/`, `/de/datenschutz/`…) with `hreflang` alternates; `/` picks a
  language from the saved choice or the browser.
- **Downloads:** `scripts/fetch-releases.mjs` reads the GitHub Releases API at build time into
  `src/data/releases.json` (set `GITHUB_TOKEN` to avoid rate limits). If the API can't be reached the
  committed snapshot is kept. The page links only to assets that exist; a platform without one shows
  an explicit "no installer yet" state.
- **Screenshots** are real captures of the app (`screencapture -l <window>` while driving it with
  `framely --show …`), converted with `node scripts/optimize-screens.mjs <dir> <locale>`. Astro
  derives AVIF/WebP sizes at build time. Open Graph images: `node scripts/og.mjs`.
- **Changelog:** the root `CHANGELOG.md` (English) plus translations in `src/content/changelog/`.
- **Legal pages** describe how the app works today and are marked as pending legal review.
- Set `SITE_URL` / `BASE_PATH` to move to a custom domain.
