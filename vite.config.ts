import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

const host = process.env.TAURI_DEV_HOST;
const project = import.meta.dirname;
const windows = resolve(project, 'src/windows');

// Each native window type has its own HTML entry (src/windows/<name>.html) so it only loads what
// it needs. Vite's root is src/windows, so the build still emits dist/<name>.html, the paths the
// Rust side opens (WebviewUrl::App("main.html"), …).
export default defineConfig({
  root: windows,
  envDir: project,
  cacheDir: resolve(project, 'node_modules/.vite'),
  plugins: [svelte({ configFile: resolve(project, 'svelte.config.js') })],
  clearScreen: false,
  resolve: {
    alias: { $lib: resolve(import.meta.dirname, 'src/lib') },
  },
  server: {
    port: 1420,
    strictPort: true,
    // Explicit IPv4: Node may bind 'localhost' to ::1 only, which WebKit then fails to reach.
    host: host || '127.0.0.1',
    hmr: host ? { protocol: 'ws', host, port: 1421 } : undefined,
    watch: { ignored: ['**/src-tauri/**', '**/native/**'] },
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome110' : 'safari16',
    outDir: resolve(project, 'dist'),
    emptyOutDir: true,
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(windows, 'main.html'),
        overlay: resolve(windows, 'overlay.html'),
        preview: resolve(windows, 'preview.html'),
        countdown: resolve(windows, 'countdown.html'),
        frame: resolve(windows, 'frame.html'),
        recbar: resolve(windows, 'recbar.html'),
        panel: resolve(windows, 'panel.html'),
        editor: resolve(windows, 'editor.html'),
      },
    },
  },
  test: {
    root: project,
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
