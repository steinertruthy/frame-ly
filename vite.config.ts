import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { resolve } from 'node:path';

const host = process.env.TAURI_DEV_HOST;

// Each native window type has its own HTML entry so it only loads what it needs.
export default defineConfig({
  plugins: [svelte()],
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
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, 'main.html'),
        overlay: resolve(import.meta.dirname, 'overlay.html'),
        preview: resolve(import.meta.dirname, 'preview.html'),
        countdown: resolve(import.meta.dirname, 'countdown.html'),
        frame: resolve(import.meta.dirname, 'frame.html'),
      },
    },
  },
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
