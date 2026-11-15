<script lang="ts">
  import { X } from '@lucide/svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { JobStatus } from '$lib/ipc/ai';
  import { roughTime } from './format';

  interface Props {
    job: JobStatus;
    oncancel?: () => void;
  }
  let { job, oncancel }: Props = $props();
  const percent = $derived(job.progress == null ? null : Math.round(job.progress * 100));
</script>

<div class="job" role="status" aria-live="polite">
  <div class="line">
    <span class="phase">
      {t(`ai.phase.${job.phase}` as MessageKey)}{#if job.part}<span class="part"> · {t('ai.progress.part', { n: job.part[0], m: job.part[1] })}</span>{/if}
    </span>
    {#if percent != null}<span class="pct">{percent}%</span>{/if}
    {#if oncancel}
      <button class="cancel" onclick={oncancel} title={t('ai.cancel')} aria-label={t('ai.cancel')}><X size={13} /></button>
    {/if}
  </div>
  <div class="bar" class:indeterminate={percent == null}>
    <span style:width={percent == null ? undefined : `${Math.max(2, percent)}%`}></span>
  </div>
  {#if job.phase !== 'queued' && job.phase !== 'loading'}
    <div class="eta">
      {#if job.etaMs != null}{t('ai.progress.eta', { time: roughTime(job.etaMs) })}{:else if percent != null && percent < 100}{t('ai.progress.calculating')}{/if}
    </div>
  {/if}
</div>

<style>
  .job {
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-width: 0;
  }
  .line {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--text-sm);
  }
  .phase {
    flex: 1;
    min-width: 0;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .part {
    color: var(--text-2);
    font-weight: 500;
  }
  .pct {
    color: var(--text-2);
    font-family: var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .cancel {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-3);
  }
  .cancel:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .bar {
    position: relative;
    height: 5px;
    border-radius: 99px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: inherit;
    background: var(--accent);
    transition: width var(--dur-slow) var(--ease-out);
  }
  .indeterminate span {
    width: 30%;
    animation: slide 1.4s var(--ease-inout) infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .eta {
    min-height: 15px;
    color: var(--text-3);
    font-size: var(--text-xs);
  }
</style>
