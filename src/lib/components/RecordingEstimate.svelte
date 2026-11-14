<script lang="ts">
  /** Live file-size estimate for the recording settings, with two everyday examples. */
  import { Coffee, Zap } from '@lucide/svelte';
  import { locale, t } from '$lib/i18n/index.svelte';
  import { estimate, formatEstimate } from '$lib/videoEstimate';
  import type { Fps, VideoCodec, VideoQuality, VideoResolution } from '$lib/ipc/types';

  let {
    screen,
    quality,
    resolution,
    codec,
    fps,
  }: { screen: [number, number]; quality: VideoQuality; resolution: VideoResolution; codec: VideoCodec; fps: Fps } = $props();

  const e = $derived(estimate(screen, { quality, resolution, codec, fps }));
  const rows = $derived([
    { kind: 'calm' as const, icon: Coffee, mb: e.calm },
    { kind: 'busy' as const, icon: Zap, mb: e.busy },
  ]);
</script>

<div class="estimate">
  <div class="head">
    <div class="title">{t('settings.recording.estimate.title')}</div>
    <div class="hint">{t('settings.recording.estimate.hint')}</div>
  </div>
  <div class="rows">
    {#each rows as r (r.kind)}
      <div class="row" class:busy={r.kind === 'busy'}>
        <span class="icon" aria-hidden="true"><r.icon size={15} /></span>
        <span class="what">
          <span class="label">{t(`settings.recording.estimate.${r.kind}`)}</span>
          <span class="example">{t(`settings.recording.estimate.${r.kind}.example`)}</span>
        </span>
        <span class="values">
          <span class="minute">{t('settings.recording.estimate.perMinute', { value: formatEstimate(r.mb, locale()) })}</span>
          <span class="hour">{t('settings.recording.estimate.perHour', { value: formatEstimate(r.mb * 60, locale()) })}</span>
        </span>
      </div>
    {/each}
  </div>
</div>

<style>
  .estimate {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 18px 16px;
    box-shadow: inset 0 -1px 0 var(--border);
  }
  .title {
    font-weight: 600;
  }
  .hint {
    margin-top: 2px;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.4;
  }
  .rows {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .row {
    display: grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: auto auto;
    column-gap: 10px;
    row-gap: 8px;
    padding: 12px;
    border-radius: 10px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    background: var(--icon-tile);
    color: var(--icon-accent);
  }
  .busy .icon {
    background: var(--rec-tile);
    color: var(--rec-icon);
  }
  .what {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .label {
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .example {
    color: var(--text-3);
    font-size: var(--text-xs);
  }
  .values {
    grid-column: 1 / -1;
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 8px;
    font-variant-numeric: tabular-nums;
  }
  .minute {
    font-size: var(--text-md);
    font-weight: 600;
  }
  .hour {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  @media (max-width: 860px) {
    .rows {
      grid-template-columns: 1fr;
    }
  }
</style>
