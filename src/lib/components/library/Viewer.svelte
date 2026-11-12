<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { X, ChevronLeft, ChevronRight, ExternalLink, FolderOpen, Copy, Trash2, Pencil } from '@lucide/svelte';
  import ConfirmDialog from '../ConfirmDialog.svelte';
  import { t, locale } from '$lib/i18n/index.svelte';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import { stem } from '$lib/library';
  import type { LibraryItem } from '$lib/ipc/types';
  import { itemActions } from './actions';

  interface Props {
    items: LibraryItem[];
    id: number;
    focusName?: boolean;
    onclose: () => void;
  }
  let { items, id = $bindable(), focusName = false, onclose }: Props = $props();

  const index = $derived(items.findIndex((i) => i.id === id));
  const item = $derived(items[index]);
  let name = $state('');
  let confirming = $state(false);
  let nameInput: HTMLInputElement | undefined = $state();

  // Reset the editable name when the item changes.
  $effect(() => {
    if (item) name = stem(item.name);
  });
  // Close if the item disappeared (deleted elsewhere).
  $effect(() => {
    if (items.length && index < 0) onclose();
  });

  onMount(async () => {
    if (focusName) {
      await tick();
      nameInput?.select();
    }
  });

  function go(delta: number) {
    const next = items[index + delta];
    if (next) id = next.id;
  }

  async function commitName() {
    if (!item || !name.trim() || name.trim() === stem(item.name)) {
      if (item) name = stem(item.name);
      return;
    }
    const updated = await itemActions.rename(item, name);
    if (!updated) name = stem(item.name);
  }

  async function trash() {
    confirming = false;
    if (!item) return;
    const neighbor = items[index + 1] ?? items[index - 1];
    if (await itemActions.trash(item)) {
      if (neighbor) id = neighbor.id;
      else onclose();
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (confirming) return;
    const typing = (e.target as HTMLElement).tagName === 'INPUT';
    if (e.key === 'Escape') {
      if (typing) (e.target as HTMLInputElement).blur();
      else onclose();
    } else if (!typing && e.key === 'ArrowLeft') go(-1);
    else if (!typing && e.key === 'ArrowRight') go(1);
    else if (!typing && (e.key === 'Delete' || (e.key === 'Backspace' && e.metaKey))) confirming = true;
  }

  const date = $derived(
    item ? new Intl.DateTimeFormat(locale(), { dateStyle: 'long', timeStyle: 'short' }).format(new Date(item.createdAt)) : '',
  );
  const folder = $derived(item ? item.path.slice(0, Math.max(item.path.lastIndexOf('/'), item.path.lastIndexOf('\\'))) : '');
</script>

<svelte:window {onkeydown} />

{#if item}
  <div class="viewer" role="dialog" aria-modal="true" aria-label={item.name}>
    <div class="stage">
      <button class="close" onclick={onclose} aria-label="Close"><X size={18} /></button>
      {#if index > 0}<button class="nav prev" onclick={() => go(-1)} aria-label="Previous"><ChevronLeft size={22} /></button>{/if}
      {#if index < items.length - 1}<button class="nav next" onclick={() => go(1)} aria-label="Next"><ChevronRight size={22} /></button>{/if}
      {#key item.id}
        {#if item.kind === 'video'}
          <!-- svelte-ignore a11y_media_has_caption -->
          <video src={convertFileSrc(item.path)} controls autoplay playsinline></video>
        {:else}
          <img src={convertFileSrc(item.path)} alt={item.name} draggable="false" />
        {/if}
      {/key}
    </div>

    <aside class="info">
      <input
        class="name"
        bind:this={nameInput}
        bind:value={name}
        aria-label={t('library.rename')}
        spellcheck="false"
        onkeydown={(e) => e.key === 'Enter' && (e.currentTarget as HTMLInputElement).blur()}
        onblur={commitName}
      />
      <dl>
        <dt>{t('library.info.type')}</dt>
        <dd>{t(item.kind === 'video' ? 'library.info.video' : 'library.info.image')}</dd>
        <dt>{t('library.info.date')}</dt>
        <dd>{date}</dd>
        {#if item.width && item.height}
          <dt>{t('library.info.resolution')}</dt>
          <dd>{item.width} × {item.height}</dd>
        {/if}
        {#if item.durationMs != null}
          <dt>{t('library.info.duration')}</dt>
          <dd>{formatDuration(item.durationMs)}</dd>
        {/if}
        <dt>{t('library.info.size')}</dt>
        <dd>{formatBytes(item.size)}</dd>
        <dt>{t('library.info.location')}</dt>
        <dd class="path" title={folder}>{folder}</dd>
      </dl>
      <div class="actions">
        {#if item.kind === 'image'}
          <button onclick={() => itemActions.edit(item)}><Pencil size={15} />{t('editor.edit')}</button>
        {/if}
        <button onclick={() => itemActions.open(item)}><ExternalLink size={15} />{t('library.open')}</button>
        <button onclick={() => itemActions.reveal(item)}><FolderOpen size={15} />{t('library.show')}</button>
        <button onclick={() => itemActions.copy(item)}><Copy size={15} />{t('library.copy')}</button>
        <button class="danger" onclick={() => (confirming = true)}><Trash2 size={15} />{t('library.trash')}</button>
      </div>
      <div class="counter">{index + 1} / {items.length}</div>
    </aside>
  </div>

  {#if confirming}
    <ConfirmDialog
      title={t('library.trash.confirm', { name: item.name })}
      body={t('library.trash.body')}
      confirm={t('library.trash')}
      cancel={t('library.cancel')}
      danger
      onconfirm={trash}
      oncancel={() => (confirming = false)}
    />
  {/if}
{/if}

<style>
  .viewer {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    grid-template-columns: 1fr 300px;
    background: rgba(7, 9, 13, 0.94);
    backdrop-filter: blur(20px);
    animation: in var(--dur-med) var(--ease-out);
  }
  @keyframes in {
    from {
      opacity: 0;
    }
  }
  .stage {
    position: relative;
    display: grid;
    place-items: center;
    min-width: 0;
    min-height: 0;
    padding: 56px 64px 32px;
  }
  .stage img,
  .stage video {
    max-width: 100%;
    max-height: 100%;
    border-radius: 8px;
    box-shadow: 0 24px 60px -20px rgba(0, 0, 0, 0.7);
    animation: zoom var(--dur-slow) var(--ease-out);
  }
  @keyframes zoom {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
  }
  .close,
  .nav {
    position: absolute;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
    transition: background var(--dur-fast);
  }
  .close:hover,
  .nav:hover {
    background: rgba(255, 255, 255, 0.18);
  }
  .close {
    top: 14px;
    left: 14px;
    width: 32px;
    height: 32px;
  }
  :global(.platform-macos) .close {
    left: 84px; /* clear of the traffic lights */
  }
  .nav {
    top: 50%;
    width: 40px;
    height: 40px;
    margin-top: -20px;
  }
  .prev {
    left: 14px;
  }
  .next {
    right: 14px;
  }
  .info {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 56px 20px 20px;
    background: rgba(255, 255, 255, 0.03);
    border-left: 1px solid rgba(255, 255, 255, 0.07);
    color: #edf0f7;
    user-select: text;
  }
  .name {
    width: 100%;
    padding: 6px 8px;
    margin: 0 -8px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: #fff;
    font-family: var(--font-display);
    font-size: var(--text-lg);
    font-weight: 650;
  }
  .name:hover {
    background: rgba(255, 255, 255, 0.06);
  }
  .name:focus {
    background: rgba(255, 255, 255, 0.1);
    box-shadow: 0 0 0 2px var(--signal-500);
  }
  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 8px 14px;
    margin: 0;
    font-size: var(--text-sm);
  }
  dt {
    color: rgba(237, 240, 247, 0.5);
  }
  dd {
    margin: 0;
    font-variant-numeric: tabular-nums;
  }
  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .actions button {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #edf0f7;
    font-weight: 550;
    text-align: left;
  }
  .actions button:hover {
    background: rgba(255, 255, 255, 0.08);
  }
  .actions .danger {
    color: #e26d7a;
  }
  .counter {
    margin-top: auto;
    color: rgba(237, 240, 247, 0.4);
    font: 500 var(--text-xs) var(--font-mono);
  }
</style>
