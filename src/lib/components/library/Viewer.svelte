<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { X, ChevronLeft, ChevronRight, ExternalLink, FolderOpen, Copy, Trash2, Pencil, Video, Image, AudioLines } from '@lucide/svelte';
  import { nav } from '$lib/stores/nav.svelte';
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
  /** Renders the viewer at the top of the document: inside the page, an ancestor's transform
   * (the page's entry animation) would confine `position: fixed` to the page area. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }
  const folder = $derived(item ? item.path.slice(0, Math.max(item.path.lastIndexOf('/'), item.path.lastIndexOf('\\'))) : '');
  const folderName = $derived(folder.split(/[\\/]/).filter(Boolean).pop() ?? folder);
</script>

<svelte:window {onkeydown} />

{#if item}
  <div class="viewer" use:portal role="dialog" aria-modal="true" aria-label={item.name}>
    <section class="stage">
      <header class="bar" data-tauri-drag-region>
        <button class="round close" onclick={onclose} aria-label={t('rec.options.close')} title={t('rec.options.close')}><X size={17} /></button>
        <span class="count">{index + 1} / {items.length}</span>
      </header>
      <div class="media">
        {#key item.id}
          {#if item.kind === 'video'}
            <!-- svelte-ignore a11y_media_has_caption -->
            <video src={convertFileSrc(item.path)} controls autoplay playsinline></video>
          {:else}
            <img src={convertFileSrc(item.path)} alt={item.name} draggable="false" />
          {/if}
        {/key}
      </div>
      {#if index > 0}<button class="round nav prev" onclick={() => go(-1)} aria-label="←"><ChevronLeft size={20} /></button>{/if}
      {#if index < items.length - 1}<button class="round nav next" onclick={() => go(1)} aria-label="→"><ChevronRight size={20} /></button>{/if}
    </section>

    <aside class="info">
      <span class="kind" class:video={item.kind === 'video'}>
        {#if item.kind === 'video'}<Video size={13} />{:else}<Image size={13} />{/if}
        {t(item.kind === 'video' ? 'library.info.video' : 'library.info.image')}
      </span>
      <label class="name-field" title={t('library.rename')}>
        <input
          class="name"
          bind:this={nameInput}
          bind:value={name}
          aria-label={t('library.rename')}
          spellcheck="false"
          onkeydown={(e) => e.key === 'Enter' && (e.currentTarget as HTMLInputElement).blur()}
          onblur={commitName}
        />
        <Pencil size={13} />
      </label>

      <dl class="details">
        <div><dt>{t('library.info.date')}</dt><dd>{date}</dd></div>
        {#if item.width && item.height}
          <div><dt>{t('library.info.resolution')}</dt><dd>{item.width} × {item.height}</dd></div>
        {/if}
        {#if item.durationMs != null}
          <div><dt>{t('library.info.duration')}</dt><dd>{formatDuration(item.durationMs)}</dd></div>
        {/if}
        <div><dt>{t('library.info.size')}</dt><dd>{formatBytes(item.size)}</dd></div>
        <div><dt>{t('library.info.location')}</dt><dd class="path" title={folder}>{folderName}</dd></div>
      </dl>

      <div class="actions">
        <button class="primary" onclick={() => itemActions.open(item)}><ExternalLink size={15} />{t('library.open')}</button>
        {#if item.kind === 'image'}
          <button onclick={() => itemActions.edit(item)}><Pencil size={15} />{t('editor.edit')}</button>
        {:else}
          <button
            onclick={() => {
              nav.transcribe(item.path);
              onclose();
            }}><AudioLines size={15} />{t('ai.transcribe')}</button
          >
        {/if}
        <button onclick={() => itemActions.reveal(item)}><FolderOpen size={15} />{t('library.show')}</button>
        <button onclick={() => itemActions.copy(item)}><Copy size={15} />{t('library.copy')}</button>
      </div>

      <button class="trash" onclick={() => (confirming = true)}><Trash2 size={15} />{t('library.trash')}</button>
    </aside>

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
  </div>
{/if}

<style>
  /* Always dark, like any media lightbox, in both themes. */
  .viewer {
    position: fixed;
    inset: 0;
    z-index: 100;
    display: grid;
    grid-template-columns: minmax(0, 1fr) 320px;
    background: rgba(9, 11, 16, 0.97);
    backdrop-filter: blur(24px);
    color: #edf0f7;
    animation: in var(--dur-med) var(--ease-out);
  }
  @keyframes in {
    from {
      opacity: 0;
    }
  }

  /* Stage */
  .stage {
    position: relative;
    display: grid;
    grid-template-rows: 56px minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
  }
  .bar {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 16px;
  }
  .close {
    position: absolute;
    left: 16px;
  }
  :global(.platform-macos) .close {
    left: 88px; /* clear of the traffic lights */
  }
  .count {
    padding: 3px 10px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.05);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.07);
    color: #8b95a8;
    font: 500 var(--text-xs) / 18px var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .media {
    display: grid;
    place-items: center;
    min-width: 0;
    min-height: 0;
    padding: 8px 80px 40px;
  }
  .media img,
  .media video {
    max-width: 100%;
    max-height: 100%;
    border-radius: 12px;
    background: #000;
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.08),
      0 30px 70px -24px rgba(0, 0, 0, 0.85);
    animation: zoom var(--dur-slow) var(--ease-out);
  }
  @keyframes zoom {
    from {
      opacity: 0;
      transform: scale(0.985);
    }
  }
  .round {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: #171d2a;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.1);
    color: #c9d1df;
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .round:hover {
    background: #1b273d;
    color: #fff;
  }
  .round:focus-visible {
    box-shadow: 0 0 0 2px #0b0e14, 0 0 0 4px #4c7bd9;
  }
  .nav {
    position: absolute;
    top: calc(50% + 28px);
    width: 42px;
    height: 42px;
    margin-top: -21px;
  }
  .prev {
    left: 22px;
  }
  .next {
    right: 22px;
  }

  /* Info panel */
  .info {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 0;
    overflow-y: auto;
    padding: 64px 22px 22px;
    background: #0d1018;
    box-shadow: inset 1px 0 0 rgba(255, 255, 255, 0.06);
    user-select: text;
  }
  .kind {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    align-self: flex-start;
    padding: 3px 9px;
    border-radius: 999px;
    background: #161f30;
    box-shadow: inset 0 0 0 1px rgba(139, 176, 238, 0.18);
    color: #8bb0ee;
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .kind.video {
    background: #24161d;
    box-shadow: inset 0 0 0 1px rgba(226, 109, 122, 0.2);
    color: #e26d7a;
  }
  .name-field {
    position: relative;
    display: flex;
    align-items: center;
    margin: -6px -8px 0;
    color: #6b7588;
  }
  .name-field :global(svg) {
    position: absolute;
    right: 10px;
    opacity: 0;
    pointer-events: none;
    transition: opacity var(--dur-fast);
  }
  .name-field:hover :global(svg) {
    opacity: 1;
  }
  .name {
    width: 100%;
    min-width: 0;
    padding: 6px 30px 6px 8px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #edf0f7;
    font: 650 15px / 1.3 var(--font-display);
    letter-spacing: -0.01em;
    text-overflow: ellipsis;
  }
  .name:hover {
    background: rgba(255, 255, 255, 0.04);
  }
  .name:focus {
    background: #0b0e14;
    box-shadow: inset 0 0 0 1px #325188;
  }
  .details {
    margin: 0;
    border-radius: 12px;
    background: #11151f;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
    font-size: var(--text-sm);
  }
  .details div {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 14px;
    padding: 9px 12px;
  }
  .details div + div {
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
  }
  dt {
    flex-shrink: 0;
    color: #7e889b;
  }
  dd {
    min-width: 0;
    margin: 0;
    color: #e4e9f2;
    text-align: right;
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
    gap: 2px;
  }
  .actions button,
  .trash {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: #c9d1df;
    font-size: var(--text-md);
    font-weight: 550;
    text-align: left;
    transition: background var(--dur-fast);
  }
  .actions button :global(svg) {
    color: #8b95a8;
  }
  .actions button:hover {
    background: #141925;
    color: #fff;
  }
  .actions button:hover :global(svg) {
    color: #8bb0ee;
  }
  .actions .primary {
    justify-content: center;
    height: 36px;
    margin-bottom: 6px;
    background: #1f3459;
    box-shadow: inset 0 0 0 1px #325188;
    color: #e6edfa;
  }
  .actions .primary :global(svg) {
    color: #8bb0ee;
  }
  .actions .primary:hover {
    background: #27416f;
  }
  .trash {
    margin-top: auto;
    color: #e26d7a;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.06);
    border-radius: 0 0 8px 8px;
    padding-top: 2px;
  }
  .trash:hover {
    background: color-mix(in srgb, #d94656 14%, transparent);
    box-shadow: none;
    border-radius: 8px;
  }
  @media (max-width: 860px) {
    .viewer {
      grid-template-columns: minmax(0, 1fr) 260px;
    }
    .media {
      padding: 8px 64px 28px;
    }
  }
</style>
