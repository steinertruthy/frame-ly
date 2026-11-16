<script lang="ts">
  import { ArrowDownToLine, ChevronDown, X } from '@lucide/svelte';
  import Button from './Button.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { updates, progress } from '$lib/stores/updates.svelte';
  import { releaseNotes } from '$lib/updates';

  const s = $derived(updates.status);
  const version = $derived('version' in s ? s.version : '');
  const notes = $derived(s.state === 'available' ? releaseNotes(s.notes) : []);
  const fraction = $derived(progress(s));
  let open = $state(false);
</script>

{#if updates.notice}
  <aside class="notice" role="status" aria-live="polite">
    <div class="head">
      <span class="icon"><ArrowDownToLine size={15} /></span>
      <div class="text">
        {#if s.state === 'available'}
          <strong>{t('updates.available', { version })}</strong>
          <span class="hint">{t('updates.availableHint')}</span>
        {:else if s.state === 'downloading'}
          <strong>
            {fraction == null
              ? t('updates.downloadingUnknown', { version })
              : t('updates.downloading', { version, percent: `${Math.round(fraction * 100)}%` })}
          </strong>
        {:else if s.state === 'waitingForRecording'}
          <strong>{t('updates.waiting', { version })}</strong>
        {:else if s.state === 'installing'}
          <strong>{t('updates.installing', { version })}</strong>
        {:else if s.state === 'error'}
          <strong>{t(`updates.error.${s.reason}`)}</strong>
        {/if}
      </div>
      {#if s.state === 'available'}
        <button class="close" onclick={() => updates.dismiss()} aria-label={t('updates.later')} title={t('updates.later')}><X size={14} /></button>
      {/if}
    </div>

    {#if s.state === 'downloading'}
      <div class="bar" class:indeterminate={fraction == null} role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={fraction == null ? undefined : Math.round(fraction * 100)}>
        <span style:width={fraction == null ? '30%' : `${fraction * 100}%`}></span>
      </div>
    {/if}

    {#if notes.length}
      <button class="notes-toggle" onclick={() => (open = !open)} aria-expanded={open}>
        {t('updates.notes')}<ChevronDown size={13} class={open ? 'flip' : ''} />
      </button>
      {#if open}
        <ul class="notes">
          {#each notes as line, i (i)}<li class:heading={line.heading}>{line.text}</li>{/each}
        </ul>
      {/if}
    {/if}

    {#if s.state === 'available' || s.state === 'error'}
      <div class="actions">
        {#if s.state === 'available'}
          <Button size="sm" variant="ghost" onclick={() => updates.dismiss()}>{t('updates.later')}</Button>
          <Button size="sm" variant="primary" onclick={() => updates.install()}>{t('updates.install')}</Button>
        {:else}
          <Button size="sm" onclick={() => updates.install()}>{t('updates.retry')}</Button>
        {/if}
      </div>
    {/if}
  </aside>
{/if}

<style>
  .notice {
    position: fixed;
    right: 18px;
    bottom: 18px;
    z-index: 40;
    width: 340px;
    padding: 14px 14px 12px;
    border-radius: var(--radius-xl);
    background: var(--surface-raised);
    box-shadow: var(--shadow-md);
    animation: rise var(--dur-slow) var(--ease-out);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  .head {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }
  .icon {
    display: grid;
    place-items: center;
    flex: none;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    background: var(--icon-tile);
    color: var(--icon-accent);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .text {
    display: grid;
    gap: 2px;
    min-width: 0;
    flex: 1;
    font-size: var(--text-md);
    line-height: 1.4;
  }
  strong {
    font-weight: 600;
  }
  .hint {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .close {
    flex: none;
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--text-3);
  }
  .close:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .bar {
    position: relative;
    height: 4px;
    margin-top: 12px;
    border-radius: 4px;
    background: var(--border);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: 4px;
    background: var(--accent);
    transition: width var(--dur-med) var(--ease-out);
  }
  .bar.indeterminate span {
    animation: slide 1.2s var(--ease-inout) infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .notes-toggle {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    margin: 10px 0 0 38px;
    padding: 0;
    border: 0;
    background: none;
    color: var(--link);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .notes-toggle :global(.flip) {
    transform: rotate(180deg);
  }
  .notes {
    max-height: 160px;
    overflow: auto;
    margin: 8px 0 0 38px;
    padding: 0 4px 0 14px;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .notes li + li {
    margin-top: 4px;
  }
  .notes li.heading {
    list-style: none;
    margin-left: -14px;
    margin-top: 8px;
    color: var(--text);
    font-weight: 600;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
    margin-top: 12px;
  }
  @media (prefers-reduced-motion: reduce) {
    .notice,
    .bar.indeterminate span {
      animation: none;
    }
  }
</style>
