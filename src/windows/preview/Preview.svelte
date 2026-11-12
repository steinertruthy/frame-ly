<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Copy, Check, FolderOpen, Download, X } from '@lucide/svelte';
  import { commands, events } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { logUi } from '$lib/ipc/log';
  import { setLanguage, t } from '$lib/i18n/index.svelte';
  import type { CaptureInfo, Theme } from '$lib/ipc/types';

  const LIFETIME_MS = 7000;

  let info = $state<CaptureInfo | null>(null);
  let src = $state('');
  let copied = $state(false);
  let hovering = $state(false);
  let remaining = $state(LIFETIME_MS);
  let leaving = $state(false);
  let timer: ReturnType<typeof setInterval> | undefined;

  async function load() {
    info = await commands.lastCapture();
    if (!info) return close();
    copied = info.copied;
    remaining = LIFETIME_MS;
    src = framelyUrl(`last?v=${Date.now()}`);
  }

  async function close() {
    leaving = true;
    setTimeout(() => void commands.closePreview(), 180);
  }

  onMount(() => {
    void (async () => {
      const s = await commands.getSettings();
      setLanguage(s.general.language);
      applyTheme(s.appearance.theme);
      await load();
      await getCurrentWindow().show();
    })().catch((e) => logUi('error', `preview init failed: ${JSON.stringify(e)}`));
    const unlisten = events.onPreviewRefresh(() => void load());
    timer = setInterval(() => {
      if (hovering || leaving) return;
      remaining -= 100;
      if (remaining <= 0) void close();
    }, 100);
    return () => {
      clearInterval(timer);
      void unlisten.then((u) => u());
    };
  });

  function applyTheme(theme: Theme) {
    if (theme !== 'system') document.documentElement.dataset.theme = theme;
  }

  async function act(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (e) {
      logUi('error', `preview action failed: ${JSON.stringify(e)}`);
    }
  }

  const copy = () =>
    act(async () => {
      await commands.copyLastCapture();
      copied = true;
    });
  const save = () => act(async () => (info = await commands.saveLastCapture()));
  const reveal = () => act(() => commands.revealLastCapture());
  const open = () =>
    act(async () => {
      if (!info?.path) info = await commands.saveLastCapture();
      await commands.openLastCapture();
      await close();
    });
</script>

<div
  class="card"
  class:leaving
  role="dialog"
  aria-label="FrameLy"
  tabindex="-1"
  onmouseenter={() => (hovering = true)}
  onmouseleave={() => (hovering = false)}
>
  <button class="thumb" onclick={open} title={info?.fileName ?? ''}>
    {#if src}<img {src} alt="" draggable="false" />{/if}
  </button>
  <div class="meta">
    <div class="text">
      <div class="name">{info?.fileName ?? t('preview.notSaved')}</div>
      {#if info}<div class="dims">{info.width} × {info.height}</div>{/if}
    </div>
    <div class="actions">
      <button class="icon" onclick={copy} title={t(copied ? 'preview.copied' : 'preview.copy')} class:ok={copied}>
        {#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}
      </button>
      {#if info?.path}
        <button class="icon" onclick={reveal} title={t('preview.show')}><FolderOpen size={15} /></button>
      {:else}
        <button class="icon" onclick={save} title={t('preview.save')}><Download size={15} /></button>
      {/if}
      <button class="icon" onclick={close} title={t('preview.close')}><X size={15} /></button>
    </div>
  </div>
  <div class="progress" style="transform:scaleX({remaining / LIFETIME_MS})"></div>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
  }
  .card {
    position: fixed;
    inset: 8px;
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    overflow: hidden;
    background: var(--surface);
    box-shadow:
      0 0 0 1px var(--border-strong),
      0 14px 40px -12px rgba(6, 13, 38, 0.55);
    animation: enter var(--dur-slow) var(--ease-spring);
  }
  .card.leaving {
    animation: leave 180ms var(--ease-out) forwards;
  }
  @keyframes enter {
    from {
      opacity: 0;
      transform: translateX(24px) scale(0.96);
    }
  }
  @keyframes leave {
    to {
      opacity: 0;
      transform: translateX(24px) scale(0.97);
    }
  }
  .thumb {
    flex: 1;
    min-height: 0;
    padding: 0;
    border: 0;
    background: var(--surface-2);
    cursor: pointer;
  }
  .thumb img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: contain;
    transition: transform var(--dur-med) var(--ease-out);
  }
  .thumb:hover img {
    transform: scale(1.02);
  }
  .meta {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 9px 10px 10px 14px;
    border-top: 1px solid var(--border);
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-size: var(--text-sm);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .dims {
    color: var(--text-3);
    font: 500 11px/1.4 var(--font-mono);
  }
  .actions {
    display: flex;
    gap: 2px;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2);
    transition:
      background var(--dur-fast),
      color var(--dur-fast),
      transform var(--dur-fast);
  }
  .icon:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .icon:active {
    transform: scale(0.92);
  }
  .icon.ok {
    color: var(--ok-500);
  }
  .progress {
    height: 2px;
    background: var(--accent);
    transform-origin: left;
    transition: transform 100ms linear;
  }
</style>
