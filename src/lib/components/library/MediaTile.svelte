<script lang="ts">
  import { Play } from '@lucide/svelte';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { formatDuration } from '$lib/stores/format';
  import { locale } from '$lib/i18n/index.svelte';
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
  const time = $derived(new Intl.DateTimeFormat(locale(), { hour: '2-digit', minute: '2-digit' }).format(new Date(item.createdAt)));
  const name = $derived(item.name.replace(/\.[^.]+$/, ''));
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
  <span class="thumb" class:video={item.kind === 'video'}>
    {#if !failed}
      <img {src} alt="" loading="lazy" decoding="async" draggable="false" onerror={() => (failed = true)} />
    {/if}
    {#if item.kind === 'video'}
      <span class="play"><Play size={compact ? 12 : 15} fill="currentColor" /></span>
      {#if item.durationMs != null}<span class="badge">{formatDuration(item.durationMs)}</span>{/if}
    {/if}
  </span>
  {#if !compact}
    <span class="name">{name}</span>
    <span class="meta">{time}</span>
  {/if}
</button>

<style>
  .tile {
    display: flex;
    flex-direction: column;
    gap: 3px;
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
    border-radius: 10px;
    overflow: hidden;
    background: linear-gradient(135deg, var(--surface-raised), var(--surface-2));
    box-shadow: inset 0 0 0 1px var(--border);
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out);
  }
  .compact .thumb {
    border-radius: 8px;
  }
  .tile:hover .thumb {
    transform: translateY(-2px);
    box-shadow:
      inset 0 0 0 1px rgba(108, 157, 230, 0.35),
      0 14px 30px -14px rgba(0, 0, 0, 0.7),
      0 0 22px -8px var(--glow);
  }
  .tile:focus-visible .thumb {
    box-shadow: var(--focus-ring);
  }
  img {
    display: block;
    width: 100%;
    height: 100%;
    object-fit: cover;
    animation: fade var(--dur-slow) var(--ease-out);
  }
  @keyframes fade {
    from {
      opacity: 0;
      transform: scale(1.02);
    }
  }
  .video::after {
    content: '';
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, transparent 55%, rgba(6, 10, 20, 0.55));
    pointer-events: none;
  }
  .play {
    position: absolute;
    left: 8px;
    bottom: 7px;
    z-index: 1;
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    padding-left: 1px;
    border-radius: 50%;
    background: rgba(241, 244, 250, 0.92);
    color: #0c1423;
    transition: transform var(--dur-med) var(--ease-spring);
  }
  .compact .play {
    width: 18px;
    height: 18px;
    left: 5px;
    bottom: 5px;
  }
  .tile:hover .play {
    transform: scale(1.12);
  }
  .badge {
    position: absolute;
    right: 7px;
    bottom: 7px;
    z-index: 1;
    color: #fff;
    font: 600 11px/16px var(--font-mono);
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.6);
  }
  .compact .badge {
    right: 5px;
    bottom: 5px;
    font-size: 10px;
  }
  .name {
    margin-top: 6px;
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
