import { defineConfig } from 'astro/config';
import sitemap from '@astrojs/sitemap';

// GitHub Pages project site: https://steinertruthy.github.io/frame-ly/
// SITE_URL / BASE_PATH let a custom domain take over without code changes.
const site = process.env.SITE_URL ?? 'https://steinertruthy.github.io';
const base = process.env.BASE_PATH ?? '/frame-ly';

export default defineConfig({
  site,
  base,
  trailingSlash: 'always',
  build: { format: 'directory', inlineStylesheets: 'always' },
  compressHTML: true,
  // Pin the site's own tsconfig: otherwise Vite also looks up the app's tsconfig in the parent folder.
  vite: { tsconfig: './tsconfig.json' },
  image: { responsiveStyles: false },
  integrations: [
    sitemap({
      i18n: {
        defaultLocale: 'en',
        locales: { 'pt-br': 'pt-BR', en: 'en', es: 'es', fr: 'fr', de: 'de' },
      },
    }),
  ],
});
