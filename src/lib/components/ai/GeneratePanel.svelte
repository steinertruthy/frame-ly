<script lang="ts">
  import { CircleDollarSign, Clock, Cpu, Globe, RefreshCw, Sparkles, TriangleAlert } from '@lucide/svelte';
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
  import { consentPoints, offeredProvider, providerName, roughTime } from './format';
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

{#snippet detailField()}
  <Segmented
    label={t('ai.gen.detail')}
    size="sm"
    value={detail}
    options={(['brief', 'balanced', 'detailed'] as const).map((d) => ({ value: d, label: t(`ai.detail.${d}` as MessageKey) }))}
    onchange={(v) => (detail = v)}
  />
{/snippet}
{#snippet providerField()}
  <Select
    label={t('ai.gen.provider')}
    prefix={regenerate ? t('ai.gen.provider') : undefined}
    icon={info?.local ? Cpu : Globe}
    value={provider}
    options={providers.map((p) => ({ value: p.id, label: providerName(p.id) }))}
    onchange={(v) => (provider = v)}
  />
{/snippet}
{#snippet action()}
  <Button variant="primary" busy={busy} disabled={!!blocked || cloudDown} onclick={go}>
    {#if regenerate}<RefreshCw size={14} />{t('ai.gen.regenerate')}{:else}<Sparkles size={14} />{t('ai.gen.generate')}{/if}
  </Button>
{/snippet}

<div class="panel" class:compact={!!regenerate}>
  {#if regenerate}
    <div class="controls">
      {@render detailField()}
      {@render providerField()}
      {@render action()}
    </div>
  {:else}
    <div class="fields">
      <div class="opt">
        <span class="lbl">{t('ai.gen.detail')}</span>
        {@render detailField()}
      </div>
      <div class="opt">
        <span class="lbl">{t('ai.gen.provider')}</span>
        {@render providerField()}
      </div>
    </div>
  {/if}

  <ul class="facts">
    {#if blocked}
      <li class="warn">
        <TriangleAlert size={14} />
        <span>
          {t(`ai.reason.${blocked}` as MessageKey)}
          {#if blocked === 'model_missing' || blocked === 'no_key' || blocked === 'no_model' || blocked === 'external_disabled'}
            <button class="link" onclick={() => nav.openSettings('intelligence')}>{t('ai.gen.settings')}</button>
          {/if}
        </span>
      </li>
    {:else if info?.local}
      <li><Cpu size={14} /><span>{t('ai.gen.privacy.local')}</span></li>
    {:else if provider === 'cloud'}
      <li class="state"><CloudState /></li>
      <li><Globe size={14} /><span>{t('ai.cloud.privacy')}</span></li>
    {:else if info}
      <li><Globe size={14} /><span>{t('ai.gen.privacy.external', { provider: providerName(provider) })}</span></li>
      <li>
        <CircleDollarSign size={14} />
        <span>{#if estimate?.costUsd}{t('ai.gen.cost', { min: money(estimate.costUsd[0]), max: money(estimate.costUsd[1]) })}{:else}{t('ai.gen.costUnknown')}{/if}</span>
      </li>
    {/if}
    {#if estimate?.time && !blocked}
      <li><Clock size={14} /><span>{t('ai.gen.estimate', { time: roughTime(estimate.time.ms) })}{#if estimate.time.approximate} · {t('ai.approx')}{/if}</span></li>
    {/if}
  </ul>

  {#if !regenerate}
    <div class="go">{@render action()}</div>
  {/if}
</div>

{#if asking}
  <ConfirmDialog
    title={t('ai.consent.title', { provider: providerName(provider) })}
    points={consentPoints(provider, aiStore.cloud)}
    note={t('ai.consent.p.revoke')}
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
    gap: 14px;
    width: 100%;
    text-align: left;
  }
  /* First generation: a tidy card with labelled options, details and the action at the bottom. */
  .panel:not(.compact) {
    max-width: 520px;
    padding: 18px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .fields {
    display: flex;
    flex-wrap: wrap;
    gap: 12px 16px;
  }
  .opt {
    display: grid;
    gap: 6px;
  }
  .lbl {
    color: var(--text-3);
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }
  .opt > :global(label) {
    min-width: 190px;
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
    display: grid;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .panel:not(.compact) .facts {
    padding-top: 12px;
    border-top: 1px solid var(--border);
  }
  .compact .facts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
  }
  .facts li {
    display: grid;
    grid-template-columns: 14px 1fr;
    align-items: start;
    gap: 8px;
  }
  .facts li.state {
    display: flex;
  }
  .facts li > :global(svg) {
    margin-top: 2px;
    color: var(--text-3);
  }
  .warn,
  .warn > :global(svg) {
    color: var(--warn-500) !important;
  }
  .go {
    display: flex;
  }
  .go :global(.btn) {
    flex: 1;
    justify-content: center;
  }
  .link {
    margin-left: 4px;
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
