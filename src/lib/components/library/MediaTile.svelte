<script lang="ts">
  import { Play } from '@lucide/svelte';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import type { LibraryItem } from '$lib/ipc/types';

  interface Props {
    item: LibraryItem;
    compact?: boolean;
    onopen: (item: LibraryItem) => void;
    oncontext?: (item: LibraryItem, e: MouseEvent) => void;
  }
  let { item, compact = false, onopen, oncontext }: Props = $props();
  let failed = $state(false);
  // `v` busts the cache when the file changes (the thumbnail cache is keyed by size too).
  const src = $derived(framelyUrl(`thumb/${item.id}?v=${item.size}`));
</script>

<button
  class="tile"
  class:compact
  onclick={() => onopen(item)}
  oncontextmenu={(e) => {
    if (!oncontext) return;
    e.preventDefault();
    oncontext(item, e);
  }}
  title={item.name}
>
  <span class="thumb">
    {#if !failed}
      <img {src} alt="" loading="lazy" decoding="async" draggable="false" onerror={() => (failed = true)} />
    {/if}
    {#if item.kind === 'video'}
      <span class="play"><Play size={16} fill="currentColor" /></span>
      {#if item.durationMs != null}<span class="badge">{formatDuration(item.durationMs)}</span>{/if}
    {/if}
  </span>
  {#if !compact}
    <span class="name">{item.name}</span>
    <span class="meta">
      {formatBytes(item.size)}{item.width && item.height ? ` · ${item.width}×${item.height}` : ''}
    </span>
  {/if}
</button>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    text-align: left;
  }
  .thumb {
    position: relative;
    display: block;
    aspect-ratio: 16 / 10;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out);
  }
  .tile:hover .thumb {
    transform: translateY(-2px);
    box-shadow:
      inset 0 0 0 1px var(--border),
      var(--shadow-md);
  }
  .tile:focus-visible .thumb {
    box-shadow: var(--focus-ring);
  }
  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: fade var(--dur-med) var(--ease-out);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  .play {
    position: absolute;
    left: 50%;
    top: 50%;
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    margin: -17px 0 0 -17px;
    padding-left: 2px;
    border-radius: 50%;
    background: rgba(6, 13, 38, 0.66);
    color: #fff;
    backdrop-filter: blur(6px);
    opacity: 0;
    transform: scale(0.9);
    transition:
      opacity var(--dur-fast),
      transform var(--dur-med) var(--ease-spring);
  }
  .tile:hover .play {
    opacity: 1;
    transform: scale(1);
  }
  .badge {
    position: absolute;
    right: 6px;
    bottom: 6px;
    padding: 1px 6px;
    border-radius: 5px;
    background: rgba(6, 13, 38, 0.78);
    color: #fff;
    font: 600 11px/16px var(--font-mono);
  }
  .name {
    margin-top: 4px;
    font-size: var(--text-sm);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .meta {
    color: var(--text-3);
    font-size: var(--text-xs);
    font-variant-numeric: tabular-nums;
  }
</style>
