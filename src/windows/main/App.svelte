<script lang="ts">
  import { House, Settings as SettingsIcon } from '@lucide/svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import LogoMark from '$lib/components/LogoMark.svelte';
  import Toaster from '$lib/components/Toaster.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { AppInfo } from '$lib/ipc/types';
  import { events } from '$lib/ipc/commands';
  import Home from './pages/Home.svelte';
  import Settings from './pages/Settings.svelte';

  let { info }: { info: AppInfo } = $props();

  type Page = 'home' | 'settings';
  const nav: { id: Page; label: MessageKey; icon: typeof House }[] = [
    { id: 'home', label: 'nav.home', icon: House },
    { id: 'settings', label: 'nav.settings', icon: SettingsIcon },
  ];
  let page = $state<Page>('home');

  // The quick panel and tray can open a specific page.
  $effect(() => {
    const un = events.onNavigate((p) => {
      if (p === 'home' || p === 'settings') page = p;
    });
    return () => void un.then((u) => u());
  });

  // Cmd/Ctrl+, opens Settings, as in every native app.
  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === ',') {
      e.preventDefault();
      page = 'settings';
    }
  }

  // Double-clicking the custom title area zooms the window like a native title bar.
  function ondblclick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('[data-drag]')) void getCurrentWindow().toggleMaximize();
  }
</script>

<svelte:window {onkeydown} />

<div class="shell" class:mac={info.platform === 'macos'}>
  <aside class="sidebar">
    <div class="titlebar" data-drag data-tauri-drag-region role="presentation" {ondblclick}></div>
    <div class="brand" data-tauri-drag-region>
      <LogoMark size={26} />
      <span>FrameLy</span>
    </div>
    <nav>
      {#each nav as item (item.id)}
        <button class="nav-item" class:active={page === item.id} onclick={() => (page = item.id)} aria-current={page === item.id ? 'page' : undefined}>
          <item.icon size={16} strokeWidth={2} />
          <span>{t(item.label)}</span>
        </button>
      {/each}
    </nav>
  </aside>

  <main>
    <div class="titlebar" data-drag data-tauri-drag-region role="presentation" {ondblclick}></div>
    {#key page}
      <div class="page">
        {#if page === 'home'}
          <Home {info} />
        {:else}
          <Settings {info} />
        {/if}
      </div>
    {/key}
  </main>
</div>

<Toaster />

<style>
  .shell {
    display: grid;
    grid-template-columns: 220px 1fr;
    grid-template-rows: 100%;
    height: 100%;
  }
  .sidebar {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--bg-sidebar);
    border-right: 1px solid var(--border);
    padding: 0 var(--space-3);
  }
  .titlebar {
    height: 14px;
    flex-shrink: 0;
  }
  .mac .titlebar {
    height: 52px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px var(--space-2) var(--space-4);
    font-family: var(--font-display);
    font-weight: 700;
    font-size: var(--text-lg);
    letter-spacing: -0.02em;
  }
  .mac .brand {
    display: none;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 32px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-2);
    font-weight: 550;
    text-align: left;
    transition:
      background var(--dur-fast) var(--ease-out),
      color var(--dur-fast);
  }
  .nav-item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
  }
  main {
    position: relative;
    display: flex;
    flex-direction: column;
    background: var(--bg);
    min-width: 0;
  }
  main > .titlebar {
    position: absolute;
    inset: 0 0 auto 0;
    height: 52px;
    z-index: 1;
  }
  .page {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 56px var(--space-6) var(--space-6);
    animation: enter var(--dur-slow) var(--ease-out);
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
