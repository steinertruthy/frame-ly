<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Copy, Check, FolderOpen, Download, X, Play, Pencil } from '@lucide/svelte';
  import { commands, events } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { logUi } from '$lib/ipc/log';
  import { setLanguage, t } from '$lib/i18n/index.svelte';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import type { CaptureInfo, RecordingInfo, Theme } from '$lib/ipc/types';

  const LIFETIME_MS = 7000;
  type Item = { kind: 'image'; info: CaptureInfo } | { kind: 'video'; info: RecordingInfo };

  let item = $state<Item | null>(null);
  let src = $state('');
  let copied = $state(false);
  let hovering = $state(false);
  let remaining = $state(LIFETIME_MS);
  let leaving = $state(false);

  async function load(kind: 'image' | 'video') {
    leaving = false;
    remaining = LIFETIME_MS;
    if (kind === 'video') {
      const info = await commands.lastRecording();
      if (!info) return close();
      item = { kind, info };
      copied = false;
      src = info.hasThumbnail ? framelyUrl(`last-recording-thumb?v=${Date.now()}`) : '';
    } else {
      const info = await commands.lastCapture();
      if (!info) return close();
      item = { kind, info };
      copied = info.copied;
      src = framelyUrl(`last?v=${Date.now()}`);
    }
  }

  async function close() {
    leaving = true;
    setTimeout(() => void commands.closePreview(), 180);
  }

  onMount(() => {
    const kind = new URLSearchParams(location.search).get('kind') === 'video' ? 'video' : 'image';
    void (async () => {
      const s = await commands.getSettings();
      setLanguage(s.general.language);
      applyTheme(s.appearance.theme);
      await load(kind);
      await getCurrentWindow().show();
    })().catch((e) => logUi('error', `preview init failed: ${JSON.stringify(e)}`));
    const unlisten = events.onPreviewRefresh((k) => void load(k));
    const timer = setInterval(() => {
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

  const isVideo = $derived(item?.kind === 'video');
  const path = $derived(item ? item.info.path : null);
  const name = $derived(item ? (item.kind === 'video' ? item.info.fileName : item.info.fileName) : null);

  const copy = () =>
    act(async () => {
      await (isVideo ? commands.copyLastRecording() : commands.copyLastCapture());
      copied = true;
    });
  const save = () =>
    act(async () => {
      if (item?.kind === 'image') item = { kind: 'image', info: await commands.saveLastCapture() };
    });
  const edit = () =>
    act(async () => {
      if (item?.kind !== 'image') return;
      let p = item.info.path;
      if (!p) p = (item = { kind: 'image', info: await commands.saveLastCapture() }).info.path;
      if (p) await commands.editorOpen(p);
      await close();
    });
  const reveal = () => act(() => (isVideo ? commands.revealLastRecording() : commands.revealLastCapture()));
  const open = () =>
    act(async () => {
      if (isVideo) await commands.openLastRecording();
      else {
        if (!path) await commands.saveLastCapture();
        await commands.openLastCapture();
      }
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
  <button class="thumb" onclick={open} title={name ?? ''}>
    {#if src}<img {src} alt="" draggable="false" />{/if}
    {#if isVideo}<span class="play"><Play size={20} fill="currentColor" /></span>{/if}
    {#if item?.kind === 'video'}<span class="badge">{formatDuration(item.info.durationMs)}</span>{/if}
  </button>
  <div class="meta">
    <div class="text">
      <div class="name">{name ?? t('preview.notSaved')}</div>
      {#if item?.kind === 'image'}
        <div class="dims">{item.info.width} × {item.info.height}</div>
      {:else if item?.kind === 'video'}
        <div class="dims">{item.info.width} × {item.info.height} · {formatBytes(item.info.bytes)}</div>
      {/if}
    </div>
    <div class="actions">
      {#if !isVideo}
        <button class="icon" onclick={edit} title={t('editor.edit')}><Pencil size={15} /></button>
      {/if}
      <button class="icon" onclick={copy} title={t(copied ? 'preview.copied' : 'preview.copy')} class:ok={copied}>
        {#if copied}<Check size={15} />{:else}<Copy size={15} />{/if}
      </button>
      {#if path}
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
      0 14px 40px -12px rgba(5, 7, 12, 0.55);
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
    position: relative;
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
  .play {
    position: absolute;
    left: 50%;
    top: 50%;
    display: grid;
    place-items: center;
    width: 46px;
    height: 46px;
    margin: -23px 0 0 -23px;
    padding-left: 3px;
    border-radius: 50%;
    background: rgba(5, 7, 12, 0.72);
    color: #fff;
    backdrop-filter: blur(8px);
    transition: transform var(--dur-med) var(--ease-spring);
  }
  .thumb:hover .play {
    transform: scale(1.08);
  }
  .badge {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 2px 7px;
    border-radius: 6px;
    background: rgba(5, 7, 12, 0.8);
    color: #fff;
    font: 600 11px/16px var(--font-mono);
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
