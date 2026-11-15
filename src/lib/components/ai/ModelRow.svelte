<script lang="ts">
  import { AudioLines, Sparkles, AlertTriangle } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { ModelStatus } from '$lib/ipc/ai';
  import { formatBytes } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { modelName, modelStateLabel, pct } from './format';

  interface Props {
    model: ModelStatus;
    /** Hide maintenance actions (first-use setup). */
    simple?: boolean;
  }
  let { model, simple = false }: Props = $props();
  const busy = $derived(model.state === 'downloading' || model.state === 'verifying');
  const act = (a: Parameters<typeof aiStore.modelAction>[1]) => aiStore.modelAction(model.id, a);
  const facts = $derived(
    t('ai.model.facts', { size: formatBytes(model.size), ram: formatBytes(model.ramMb * 1024 * 1024), license: model.license }),
  );
</script>

<div class="model" class:ready={model.state === 'installed'}>
  <span class="icon">
    {#if model.kind === 'speech'}<AudioLines size={16} />{:else}<Sparkles size={16} />{/if}
  </span>
  <div class="text">
    <div class="name">
      {modelName(model.id)}
      <span class="state {model.state}">{modelStateLabel(model)}</span>
      {#if model.inUse}<span class="state in-use">{t('ai.model.inUse')}</span>{/if}
    </div>
    <div class="facts">{facts}</div>
    {#if model.state === 'downloading' || model.state === 'paused' || model.state === 'verifying'}
      <div class="bar" class:indeterminate={model.state === 'verifying'}>
        <span style:width={model.state === 'verifying' ? undefined : `${pct(model)}%`}></span>
      </div>
    {/if}
    {#if model.error && !busy}
      <div class="error"><AlertTriangle size={12} />{t(`error.${model.error}` as MessageKey)}</div>
    {/if}
  </div>
  <div class="actions">
    {#if model.state === 'missing'}
      <Button size="sm" variant={simple ? 'secondary' : 'primary'} onclick={() => act('download')}>{t('ai.action.download')}</Button>
    {:else if model.state === 'downloading'}
      <Button size="sm" variant="ghost" onclick={() => act('pause')}>{t('ai.action.pause')}</Button>
      <Button size="sm" variant="ghost" onclick={() => act('cancel')}>{t('ai.action.cancel')}</Button>
    {:else if model.state === 'paused'}
      <Button size="sm" onclick={() => act('download')}>{t('ai.action.resume')}</Button>
      <Button size="sm" variant="ghost" onclick={() => act('cancel')}>{t('ai.action.cancel')}</Button>
    {:else if model.state === 'corrupt'}
      <Button size="sm" variant="primary" onclick={() => act('repair')}>{t('ai.action.repair')}</Button>
    {:else if model.state === 'update' && !simple}
      <Button size="sm" variant="primary" onclick={() => act('download')}>{t('ai.action.update')}</Button>
    {/if}
    {#if !simple && (model.state === 'installed' || model.state === 'update')}
      <Button size="sm" variant="ghost" onclick={() => act('verify')}>{t('ai.action.verify')}</Button>
    {/if}
    {#if !simple && (model.state === 'installed' || model.state === 'update' || model.state === 'corrupt')}
      <Button size="sm" variant="ghost" disabled={model.inUse} onclick={() => act('remove')}>{t('ai.action.remove')}</Button>
    {/if}
  </div>
</div>

<style>
  .model {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
  }
  .model + :global(.model) {
    box-shadow: inset 0 1px 0 var(--border);
  }
  .icon {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 34px;
    height: 34px;
    border-radius: 9px;
    background: var(--icon-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .name {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    font-weight: 600;
  }
  .state {
    padding: 1px 7px;
    border-radius: 99px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-2);
    font-size: var(--text-xs);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .state.installed {
    color: var(--ok-500);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--ok-500) 30%, transparent);
  }
  .state.corrupt,
  .state.update {
    color: var(--warn-500);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--warn-500) 30%, transparent);
  }
  .state.downloading,
  .state.verifying {
    color: var(--link);
  }
  .facts {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .bar {
    position: relative;
    height: 4px;
    margin-top: 4px;
    border-radius: 99px;
    background: var(--bg);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: inherit;
    background: var(--accent);
    transition: width var(--dur-med) var(--ease-out);
  }
  .indeterminate span {
    width: 30%;
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
  .error {
    display: flex;
    align-items: center;
    gap: 5px;
    color: var(--warn-500);
    font-size: var(--text-sm);
  }
  .actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
</style>
