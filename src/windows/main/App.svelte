<script lang="ts">
  import { House, Images, AudioLines, PanelLeftClose, PanelLeftOpen, Settings as SettingsIcon, Square } from '@lucide/svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Aurora from '$lib/components/Aurora.svelte';
  import LogoMark from '$lib/components/LogoMark.svelte';
  import Toaster from '$lib/components/Toaster.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { AppInfo } from '$lib/ipc/types';
  import { commands, events } from '$lib/ipc/commands';
  import { recording, formatDuration } from '$lib/stores/recording.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import Home from './pages/Home.svelte';
  import Settings from './pages/Settings.svelte';
  import Library from './pages/Library.svelte';
  import Transcripts from './pages/Transcripts.svelte';
  import { nav as navigation, type Page } from '$lib/stores/nav.svelte';

  let { info }: { info: AppInfo } = $props();

  const nav: { id: Page; label: MessageKey; icon: typeof House }[] = [
    { id: 'home', label: 'nav.home', icon: House },
    { id: 'library', label: 'nav.library', icon: Images },
    { id: 'transcripts', label: 'nav.transcripts', icon: AudioLines },
    { id: 'settings', label: 'nav.settings', icon: SettingsIcon },
  ];
  // Shared with the pages so any of them can navigate (e.g. "Transcribe" in the library).
  const page = $derived(navigation.page);

  // Icons only: by choice (saved with the appearance settings) or because the window is narrow.
  let width = $state(window.innerWidth);
  const narrow = $derived(width <= 760);
  const collapsed = $derived(narrow || (settings.value?.appearance.sidebarCollapsed ?? false));
  function toggleSidebar() {
    void settings.update((d) => (d.appearance.sidebarCollapsed = !d.appearance.sidebarCollapsed));
  }

  // The quick panel and tray can open a specific page.
  $effect(() => {
    const un = events.onNavigate((p) => {
      if (p === 'home' || p === 'library' || p === 'transcripts' || p === 'settings') navigation.go(p);
    });
    // `framely --transcribe <file>` (launchers, Shortcuts…).
    const un2 = events.onTranscribe((path) => navigation.transcribe(path));
    return () => {
      void un.then((u) => u());
      void un2.then((u) => u());
    };
  });

  // Cmd/Ctrl+, opens Settings, as in every native app.
  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === ',') {
      e.preventDefault();
      navigation.go('settings');
    }
    // Cmd/Ctrl+\ shows or hides the sidebar labels.
    if ((e.metaKey || e.ctrlKey) && e.key === '\\' && !narrow) {
      e.preventDefault();
      toggleSidebar();
    }
  }

  // Double-clicking the custom title area zooms the window like a native title bar.
  function ondblclick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest('[data-drag]')) void getCurrentWindow().toggleMaximize();
  }
</script>

<svelte:window {onkeydown} bind:innerWidth={width} />

<Aurora />

<div class="shell" class:mac={info.platform === 'macos'} class:collapsed>
  <aside class="sidebar">
    <div class="titlebar" data-drag data-tauri-drag-region role="presentation" {ondblclick}></div>
    <div class="brand" data-tauri-drag-region>
      <LogoMark size={30} tile />
      <span>FrameLy</span>
    </div>
    <nav>
      {#each nav as item (item.id)}
        <button
          class="nav-item"
          class:active={page === item.id}
          onclick={() => navigation.go(item.id)}
          aria-current={page === item.id ? 'page' : undefined}
          aria-label={collapsed ? t(item.label) : undefined}
          title={collapsed ? t(item.label) : undefined}
        >
          <item.icon size={16} strokeWidth={2} />
          <span>{t(item.label)}</span>
        </button>
      {/each}
    </nav>
    {#if recording.active}
      <button class="live" class:paused={recording.status.phase === 'paused'} onclick={() => commands.stopRecording()} title={t('record.stop')}>
        <span class="dot"></span>
        <span class="time">{formatDuration(recording.status.elapsedMs)}</span>
        <Square size={11} fill="currentColor" />
      </button>
    {/if}
    {#if !narrow}
      {@const label = collapsed ? t('nav.expand') : t('nav.collapse')}
      <button class="nav-item toggle" onclick={toggleSidebar} aria-label={label} aria-expanded={!collapsed} title={label}>
        {#if collapsed}
          <PanelLeftOpen size={16} strokeWidth={2} />
        {:else}
          <PanelLeftClose size={16} strokeWidth={2} />
        {/if}
      </button>
    {/if}
  </aside>

  <main>
    <div class="titlebar" data-drag data-tauri-drag-region role="presentation" {ondblclick}></div>
    {#key page}
      <div class="page" class:wide={page !== 'home'}>
        {#if page === 'home'}
          <Home {info} onnavigate={(p) => navigation.go(p)} />
        {:else if page === 'library'}
          <Library />
        {:else if page === 'transcripts'}
          <Transcripts />
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
    position: relative;
    z-index: 1;
    display: grid;
    grid-template-columns: 208px 1fr;
    grid-template-rows: 100%;
    height: 100%;
    transition: grid-template-columns var(--dur-slow) var(--ease-out);
  }
  .shell.collapsed {
    grid-template-columns: 64px 1fr;
  }
  .sidebar {
    position: relative;
    display: flex;
    flex-direction: column;
    padding: 0 12px 14px;
    background: var(--bg-sidebar);
    box-shadow: inset -1px 0 0 var(--border);
    transition: padding var(--dur-slow) var(--ease-out);
    overflow: hidden;
  }
  .titlebar {
    height: 18px;
    flex-shrink: 0;
  }
  .mac .sidebar > .titlebar {
    height: 44px;
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 4px 2px 22px;
    font-family: var(--font-display);
    font-weight: 650;
    font-size: var(--text-lg);
    letter-spacing: -0.015em;
    color: var(--text);
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .nav-item {
    position: relative;
    display: flex;
    align-items: center;
    gap: 11px;
    height: 34px;
    padding: 0 11px;
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
  .nav-item span {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .nav-item :global(svg) {
    flex-shrink: 0;
  }
  /* Pinned to the bottom, under the recording timer when there is one. */
  .toggle {
    align-self: flex-start;
    justify-content: center;
    width: 38px; /* icon lines up with the nav icons (11px inset) */
    margin-top: auto;
    padding: 0;
    color: var(--text-3);
  }
  .collapsed .toggle {
    align-self: center;
  }
  .live ~ .toggle {
    margin-top: 8px;
  }
  .nav-item:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .nav-item.active {
    background: var(--nav-active);
    box-shadow: inset 0 0 0 1px var(--nav-active-border);
    color: var(--text);
  }
  .nav-item.active :global(svg) {
    color: var(--icon-accent);
  }
  .live {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: auto;
    height: 36px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: color-mix(in srgb, var(--rec-500) 14%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--rec-500) 30%, transparent);
    color: var(--rec-500);
    font-weight: 650;
    animation: in var(--dur-slow) var(--ease-out);
  }
  .live .time {
    flex: 1;
    text-align: left;
    color: var(--text);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .live .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: currentColor;
    animation: pulse 1.6s infinite;
  }
  .live.paused {
    color: var(--warn-500);
    background: color-mix(in srgb, var(--warn-500) 12%, transparent);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--warn-500) 28%, transparent);
  }
  .live.paused .dot {
    animation: none;
  }
  @keyframes pulse {
    50% {
      opacity: 0.3;
    }
  }
  @keyframes in {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  main {
    position: relative;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  main > .titlebar {
    position: absolute;
    inset: 0 0 auto 0;
    height: 44px;
    z-index: 1;
  }
  .page {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 52px 36px 40px;
    animation: page var(--dur-slow) var(--ease-out);
  }
  .page > :global(*) {
    max-width: 980px;
    margin-left: auto;
    margin-right: auto;
  }
  .page.wide > :global(*) {
    max-width: 1180px;
  }
  @keyframes page {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  .collapsed .brand span,
  .collapsed .nav-item span,
  .collapsed .live .time {
    display: none;
  }
  .collapsed .brand {
    justify-content: center;
    padding-left: 0;
    padding-right: 0;
  }
  .collapsed .nav-item {
    justify-content: center;
    padding: 0;
  }
  .collapsed .live {
    justify-content: center;
    padding: 0;
  }
  @media (max-width: 760px) {
    .page {
      padding: 52px 20px 32px;
    }
  }
</style>
