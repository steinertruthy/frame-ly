<script lang="ts">
  import { FolderOpen } from '@lucide/svelte';
  import Button from '../Button.svelte';
  import { commands } from '$lib/ipc/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { formatBytes } from '$lib/stores/format';
  import type { LibraryOverview } from '$lib/ipc/types';
  import { toast } from '$lib/stores/toasts.svelte';

  let { overview }: { overview: LibraryOverview } = $props();
  const used = $derived(overview.videoBytes + overview.imageBytes);
  const total = $derived(used + (overview.freeBytes ?? 0));
  // Show FrameLy's share of the space it could use; keep slivers visible.
  const pct = (b: number) => (total > 0 ? Math.max(b > 0 ? 0.8 : 0, (b / total) * 100) : 0);

  async function open(folder: 'recordings' | 'screenshots') {
    try {
      await commands.openFolder(folder);
    } catch {
      toast('error', t('error.io'));
    }
  }
</script>

<div class="card">
  <div class="top">
    <strong>{formatBytes(used)}</strong>
    {#if overview.freeBytes != null}<span class="free">{t('home.storage.free', { size: formatBytes(overview.freeBytes) })}</span>{/if}
  </div>
  <div class="bar" role="img" aria-label={t('home.storage')}>
    <span class="seg videos" style="width:{pct(overview.videoBytes)}%"></span>
    <span class="seg images" style="width:{pct(overview.imageBytes)}%"></span>
  </div>
  <div class="legend">
    <span class="item">
      <i class="sw videos"></i>{t('home.storage.videos', { n: overview.videos })} · {formatBytes(overview.videoBytes)}
      <Button size="sm" variant="ghost" onclick={() => open('recordings')}><FolderOpen size={13} /></Button>
    </span>
    <span class="item">
      <i class="sw images"></i>{t('home.storage.images', { n: overview.images })} · {formatBytes(overview.imageBytes)}
      <Button size="sm" variant="ghost" onclick={() => open('screenshots')}><FolderOpen size={13} /></Button>
    </span>
  </div>
</div>

<style>
  .card {
    padding: var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
  }
  .top {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  strong {
    font: 700 var(--text-xl) var(--font-display);
  }
  .free {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .bar {
    display: flex;
    height: 8px;
    margin: 12px 0;
    border-radius: 999px;
    background: var(--surface-hover);
    overflow: hidden;
  }
  .seg {
    height: 100%;
    transition: width var(--dur-slow) var(--ease-out);
  }
  .videos {
    background: var(--rec-500);
  }
  .images {
    background: var(--signal-500);
  }
  .legend {
    display: flex;
    gap: var(--space-5);
    flex-wrap: wrap;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .item {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .sw {
    width: 8px;
    height: 8px;
    border-radius: 2px;
  }
</style>
