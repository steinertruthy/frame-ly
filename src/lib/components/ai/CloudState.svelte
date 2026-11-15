<script lang="ts">
  import { onMount } from 'svelte';
  import { Cloud, CloudOff, LoaderCircle, RefreshCw } from '@lucide/svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { aiStore } from '$lib/stores/ai.svelte';

  /** FrameLy Cloud connection state (checking, available, offline, unavailable…). */
  let { compact = false }: { compact?: boolean } = $props();
  onMount(() => void aiStore.checkCloud());
  const state = $derived(aiStore.cloud?.state);
  const ok = $derived(state === 'available');
</script>

<span class="cloud {state ?? 'checking'}" class:compact role="status">
  {#if aiStore.cloudChecking && !aiStore.cloud}
    <LoaderCircle size={13} class="spin" />{t('ai.cloud.state.checking')}
  {:else if state}
    {#if ok}<Cloud size={13} />{:else}<CloudOff size={13} />{/if}
    {t(`ai.cloud.state.${state}` as MessageKey)}
    {#if !ok && state !== 'not_configured' && state !== 'local_only'}
      <button onclick={() => aiStore.checkCloud(true)} title={t('ai.cloud.retry')} aria-label={t('ai.cloud.retry')}>
        {#if aiStore.cloudChecking}<LoaderCircle size={12} class="spin" />{:else}<RefreshCw size={12} />{/if}
      </button>
    {/if}
  {/if}
</span>

<style>
  .cloud {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .available {
    color: var(--ok-500);
  }
  .offline,
  .unavailable,
  .region_blocked,
  .not_configured {
    color: var(--warn-500);
  }
  button {
    display: grid;
    place-items: center;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: inherit;
  }
  button:hover {
    background: var(--surface-hover);
  }
  .cloud :global(.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
