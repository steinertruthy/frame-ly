<script lang="ts">
  import { Cpu, Globe, Clock, Sparkles, RefreshCw } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Select from '$lib/components/Select.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detail, type GenerationEstimate, type ProviderId, type Task } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { nav } from '$lib/stores/nav.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { needsConsent } from '$lib/ai';
  import { consentBody, offeredProvider, providerName, roughTime } from './format';
  import CloudState from './CloudState.svelte';

  interface Props {
    transcriptId: number;
    task: Task;
    /** Regenerating: start from the current result's settings. */
    regenerate?: { detail: Detail; provider: ProviderId } | null;
  }
  let { transcriptId, task, regenerate = null }: Props = $props();

  const s = $derived(settings.value?.intelligence);
  let detail = $state<Detail>('balanced');
  let provider = $state<ProviderId>('framely');
  let initialized = false;
  $effect(() => {
    if (initialized || !s) return;
    initialized = true;
    detail = regenerate?.detail ?? (task === 'summary' ? s.summaryDetail : s.topicsDetail);
    provider = regenerate?.provider ?? (task === 'summary' ? s.summaryProvider : s.topicsProvider);
  });

  const providers = $derived((aiStore.overview?.providers ?? []).filter((p) => offeredProvider(p, s?.externalEnabled) || p.id === provider));
  const info = $derived(aiStore.overview?.providers.find((p) => p.id === provider));
  // "no_consent" is resolved by asking here, so it doesn't block.
  const cloudDown = $derived(provider === 'cloud' && !!aiStore.cloud && aiStore.cloud.state !== 'available');
  const blocked = $derived(info && !info.available && info.reason !== 'no_consent' ? info.reason : null);

  let estimate = $state<GenerationEstimate | null>(null);
  $effect(() => {
    const p = provider;
    estimate = null;
    ai.generationEstimate(transcriptId, task, p)
      .then((e) => (estimate = e))
      .catch(() => (estimate = null));
  });
  const money = (n: number) => n.toLocaleString(locale(), { minimumFractionDigits: n < 0.1 ? 3 : 2, maximumFractionDigits: n < 0.1 ? 3 : 2 });

  let asking = $state(false);
  let busy = $state(false);
  async function go() {
    if (s && needsConsent(provider, s)) {
      asking = true;
      return;
    }
    await run();
  }
  async function allowAndRun() {
    asking = false;
    const p = provider;
    if (await settings.update((d) => void (d.intelligence.consents.includes(p) || d.intelligence.consents.push(p)))) {
      await aiStore.loadOverview();
      await run();
    }
  }
  async function run() {
    busy = true;
    try {
      await ai.generate(transcriptId, task, provider, detail);
    } catch (e) {
      toast('error', t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'), 5000);
    } finally {
      busy = false;
    }
  }
</script>

<div class="panel" class:compact={!!regenerate}>
  <div class="controls">
    <Segmented
      label={t('ai.gen.detail')}
      size="sm"
      value={detail}
      options={(['brief', 'balanced', 'detailed'] as const).map((d) => ({ value: d, label: t(`ai.detail.${d}` as MessageKey) }))}
      onchange={(v) => (detail = v)}
    />
    <Select
      label={t('ai.gen.provider')}
      prefix={t('ai.gen.provider')}
      icon={info?.local ? Cpu : Globe}
      value={provider}
      options={providers.map((p) => ({ value: p.id, label: providerName(p.id) }))}
      onchange={(v) => (provider = v)}
    />
    <Button variant="primary" busy={busy} disabled={!!blocked || cloudDown} onclick={go}>
      {#if regenerate}<RefreshCw size={14} />{t('ai.gen.regenerate')}{:else}<Sparkles size={14} />{t('ai.gen.generate')}{/if}
    </Button>
  </div>

  <div class="facts">
    {#if blocked}
      <span class="warn">{t(`ai.reason.${blocked}` as MessageKey)}</span>
      {#if blocked === 'model_missing' || blocked === 'no_key' || blocked === 'no_model' || blocked === 'external_disabled'}
        <button class="link" onclick={() => nav.openSettings('intelligence')}>{t('ai.gen.settings')}</button>
      {/if}
    {:else if info?.local}
      <span><Cpu size={13} />{t('ai.gen.privacy.local')}</span>
    {:else if provider === 'cloud'}
      <CloudState />
      <span><Globe size={13} />{t('ai.cloud.privacy', { tokens: (estimate?.inputTokens ?? 0).toLocaleString(locale()), processor: aiStore.cloud?.processor ?? t('ai.cloud.processorUnknown') })}</span>
    {:else if info}
      <span><Globe size={13} />{t('ai.gen.privacy.external', { tokens: (estimate?.inputTokens ?? 0).toLocaleString(locale()), provider: providerName(provider) })}</span>
      <span>
        {#if estimate?.costUsd}{t('ai.gen.cost', { min: money(estimate.costUsd[0]), max: money(estimate.costUsd[1]) })}{:else}{t('ai.gen.costUnknown')}{/if}
      </span>
    {/if}
    {#if estimate?.time && !blocked}
      <span><Clock size={13} />{t('ai.gen.estimate', { time: roughTime(estimate.time.ms) })}{#if estimate.time.approximate} · {t('ai.approx')}{/if}</span>
    {/if}
  </div>
</div>

{#if asking}
  <ConfirmDialog
    title={t('ai.consent.title', { provider: providerName(provider) })}
    body={consentBody(provider, aiStore.cloud)}
    confirm={t('ai.consent.allow')}
    cancel={t('ai.consent.cancel')}
    onconfirm={allowAndRun}
    oncancel={() => (asking = false)}
  />
{/if}

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .controls {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .controls :global(.seg) {
    min-width: 0;
  }
  .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .facts span {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .facts :global(svg) {
    color: var(--text-3);
  }
  .warn {
    color: var(--warn-500);
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--link);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .link:hover {
    color: var(--link-hover);
  }
</style>
