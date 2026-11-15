<script lang="ts">
  import { onMount } from 'svelte';
  import { Video } from '@lucide/svelte';
  import { t, locale } from '$lib/i18n/index.svelte';
  import { commands } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import type { LibraryItem } from '$lib/ipc/types';
  import { formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import Sheet from './Sheet.svelte';

  interface Props {
    onpick: (path: string) => void;
    onclose: () => void;
  }
  let { onpick, onclose }: Props = $props();
  let items = $state<LibraryItem[] | null>(null);
  onMount(async () => {
    items = await commands.libraryList({ search: '', filter: 'video', sort: 'newest' });
  });
  const done = $derived(new Set(aiStore.transcripts.filter((x) => x.status === 'done').map((x) => x.media.path)));
  const fmt = $derived(new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }));
</script>

<Sheet title={t('ai.pick.title')} width={620} {onclose}>
  {#if items && items.length === 0}
    <p class="empty">{t('ai.pick.empty')}</p>
  {:else if items}
    <div class="list">
      {#each items as item (item.id)}
        <button class="row" onclick={() => onpick(item.path)}>
          <span class="thumb"><img src={framelyUrl(`thumb/${item.id}?v=${item.size}`)} alt="" loading="lazy" /><Video size={14} /></span>
          <span class="text">
            <span class="name">{item.name}</span>
            <span class="facts">{fmt.format(new Date(item.createdAt))}{#if item.durationMs != null} · {formatDuration(item.durationMs)}{/if}</span>
          </span>
          {#if done.has(item.path)}<span class="badge">{t('ai.status.done')}</span>{/if}
        </button>
      {/each}
    </div>
  {/if}
</Sheet>

<style>
  .empty {
    color: var(--text-2);
    padding: 24px 0;
    text-align: center;
  }
  .list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text);
    text-align: left;
  }
  .row:hover,
  .row:focus-visible {
    background: var(--surface-hover);
  }
  .thumb {
    position: relative;
    flex-shrink: 0;
    width: 88px;
    height: 52px;
    border-radius: 7px;
    overflow: hidden;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }
  .thumb :global(svg) {
    position: absolute;
    left: 6px;
    bottom: 5px;
    color: #fff;
    filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.6));
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .facts {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .badge {
    padding: 2px 8px;
    border-radius: 99px;
    color: var(--ok-500);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--ok-500) 30%, transparent);
    font-size: var(--text-xs);
    font-weight: 600;
  }
</style>
