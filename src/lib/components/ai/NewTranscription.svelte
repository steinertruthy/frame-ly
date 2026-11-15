<script lang="ts">
  import { onMount } from 'svelte';
  import { Video, AudioLines, Clock, Cpu, Globe, Download, FileText, ListTree, AlertTriangle, ShieldCheck } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Select from '$lib/components/Select.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detail, type Estimate, type Precision, type ProviderId, type TimeEstimate } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { isReady, needsConsent, TRANSCRIPTION_LANGUAGES } from '$lib/ai';
  import Sheet from './Sheet.svelte';
  import { consentPoints, languageName, modelName, offeredProvider, pct, providerName, roughTime } from './format';
  import CloudState from './CloudState.svelte';

  interface Props {
    path: string;
    onclose: () => void;
    onopen: (transcriptId: number) => void;
  }
  let { path, onclose, onopen }: Props = $props();

  type Task = 'summary' | 'topics';
  interface Step {
    on: boolean;
    provider: ProviderId;
    detail: Detail;
  }
  let precision = $state<Exclude<Precision, 'auto'>>('balanced');
  let language = $state('auto');
  let steps = $state<Record<Task, Step>>({
    summary: { on: true, provider: 'framely', detail: 'balanced' },
    topics: { on: true, provider: 'framely', detail: 'balanced' },
  });
  let estimate = $state<Estimate | null>(null);
  let failed = $state<string | null>(null);
  let starting = $state(false);
  let consentFor = $state<ProviderId | null>(null);

  onMount(() => {
    const s = settings.value?.intelligence;
    precision = s && s.precision !== 'auto' ? s.precision : (aiStore.overview?.defaultPrecision ?? 'balanced');
    language = s?.language ?? 'auto';
    if (s) {
      steps = {
        summary: { on: s.summaryAuto, provider: s.summaryProvider, detail: s.summaryDetail },
        topics: { on: s.topicsAuto, provider: s.topicsProvider, detail: s.topicsDetail },
      };
    }
  });

  $effect(() => {
    const p = precision;
    failed = null;
    ai.estimate(path, p)
      .then((e) => (estimate = e))
      .catch((e) => (failed = isAppError(e) ? e.code : 'generic'));
  });

  // Live model state (downloads progress through the store).
  const model = $derived(estimate ? (aiStore.model(estimate.model.id) ?? estimate.model) : null);
  const speechReady = $derived(isReady(model ?? undefined));
  const unsupported = $derived(aiStore.overview ? !aiStore.overview.hardware.supported : false);
  const s = $derived(settings.value?.intelligence);
  const providers = $derived((aiStore.overview?.providers ?? []).filter((p) => offeredProvider(p, s?.externalEnabled)));
  const info = (id: ProviderId) => aiStore.overview?.providers.find((p) => p.id === id);
  /** Why a step can't run as configured ("no_consent" is resolved by asking at start). */
  function blocker(task: Task): string | null {
    const st = steps[task];
    if (!st.on) return null;
    const i = info(st.provider);
    if (st.provider === 'cloud' && aiStore.cloud && aiStore.cloud.state !== 'available') return `cloud_${aiStore.cloud.state}`;
    return i && !i.available && i.reason !== 'no_consent' ? i.reason : null;
  }
  const reused = $derived(
    !!estimate?.existing &&
      estimate.existing.status === 'done' &&
      estimate.existing.precision === precision &&
      estimate.existing.language === language &&
      estimate.existing.media.size === estimate.media.size &&
      estimate.existing.media.mtime === estimate.media.mtime,
  );

  function stepTime(task: Task): TimeEstimate | null {
    if (!estimate || !steps[task].on || steps[task].provider !== 'framely') return null;
    if (reused && (task === 'summary' ? estimate.existing?.hasSummary : estimate.existing?.hasTopics)) return null;
    return task === 'summary' ? estimate.summaryLocal : estimate.topicsLocal;
  }
  const total = $derived.by(() => {
    if (!estimate) return null;
    const parts = [reused ? null : estimate.time, stepTime('summary'), stepTime('topics')].filter((x): x is TimeEstimate => !!x);
    return { ms: parts.reduce((n, p) => n + p.ms, 0), approximate: parts.some((p) => p.approximate) };
  });
  const external = $derived((['summary', 'topics'] as const).filter((k) => steps[k].on && !info(steps[k].provider)?.local).map((k) => steps[k].provider));
  const money = (n: number) => n.toLocaleString(locale(), { minimumFractionDigits: n < 0.1 ? 3 : 2, maximumFractionDigits: n < 0.1 ? 3 : 2 });
  $effect(() => {
    if (steps.summary.provider === 'cloud' || steps.topics.provider === 'cloud') void aiStore.checkCloud();
  });
  const usesCloud = $derived(external.includes('cloud'));
  const anyBlocked = $derived(!!blocker('summary') || !!blocker('topics'));
  const willGenerate = $derived(steps.summary.on || steps.topics.on);

  async function start() {
    if (!s) return;
    // Consent first for any external provider not yet allowed.
    const ask = [...new Set(external)].find((p) => needsConsent(p, s));
    if (ask) {
      consentFor = ask;
      return;
    }
    // Remember the choices as the new defaults.
    await settings.update((d) => {
      const i = d.intelligence;
      i.summaryAuto = steps.summary.on;
      i.summaryProvider = steps.summary.provider;
      i.summaryDetail = steps.summary.detail;
      i.topicsAuto = steps.topics.on;
      i.topicsProvider = steps.topics.provider;
      i.topicsDetail = steps.topics.detail;
    });
    starting = true;
    try {
      // A finished transcript of the same file and settings is reused; only missing parts run.
      const r = await ai.transcribe(path, precision, language, {
        summary: steps.summary.on,
        topics: steps.topics.on,
        summaryProvider: steps.summary.provider,
        summaryDetail: steps.summary.detail,
        topicsProvider: steps.topics.provider,
        topicsDetail: steps.topics.detail,
      });
      onopen(r.transcript.id);
    } catch (e) {
      toast('error', t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'), 5000);
    } finally {
      starting = false;
    }
  }

  async function allow() {
    const p = consentFor!;
    consentFor = null;
    if (await settings.update((d) => void (d.intelligence.consents.includes(p) || d.intelligence.consents.push(p)))) {
      await aiStore.loadOverview();
      await start();
    }
  }

  const detailOptions = $derived((['brief', 'balanced', 'detailed'] as const).map((d) => ({ value: d, label: t(`ai.detail.${d}` as MessageKey) })));
</script>

<Sheet title={t('ai.new.title')} width={640} {onclose}>
  {#if failed}
    <p class="error">{t(`error.${failed}` as MessageKey) || t('error.generic')}</p>
  {:else if !estimate}
    <div class="loading" aria-busy="true"></div>
  {:else}
    {@const m = estimate.media}
    <div class="file">
      <span class="kind" class:video={m.kind === 'video'}>
        {#if m.kind === 'video'}<Video size={18} />{:else}<AudioLines size={18} />{/if}
      </span>
      <div class="meta">
        <div class="name" title={m.path}>{m.name}</div>
        <div class="sub">{t(m.kind === 'video' ? 'ai.kind.video' : 'ai.kind.audio')} · {formatDuration(m.durationMs)} · {formatBytes(m.size)}</div>
      </div>
    </div>

    <!-- 1 · Transcription -->
    <section class="step">
      <header>
        <span class="num">1</span>
        <span class="icon"><AudioLines size={15} /></span>
        <span class="title">{t('ai.tab.transcript')}</span>
        <span class="when">
          {#if reused}{t('ai.new.reused')}{:else}<Clock size={12} />~{roughTime(estimate.time.ms)}{/if}
        </span>
      </header>
      <div class="rows">
        <div class="grid">
          <div class="field">
            <span class="label">{t('ai.new.precision')}</span>
            <Segmented
              label={t('ai.new.precision')}
              size="sm"
              stretch
              value={precision}
            options={(['fast', 'balanced', 'max'] as const).map((p) => ({ value: p, label: t(`ai.precision.${p}` as MessageKey) }))}
              onchange={(v) => (precision = v)}
            />
          </div>
          <div class="field">
            <span class="label">{t('ai.new.language')}</span>
            <Select
              label={t('ai.new.language')}
              value={language}
              options={[{ value: 'auto', label: t('ai.language.auto') }, ...TRANSCRIPTION_LANGUAGES.map((c) => ({ value: c, label: languageName(c) }))]}
              onchange={(v) => (language = v)}
            />
          </div>
        </div>
        <p class="hint">{t(`ai.precision.${precision}.hint` as MessageKey)}</p>
        {#if model && !speechReady}
          <div class="need">
            <Download size={15} />
            <div class="need-text">
              <div>{t('ai.new.modelNeeded', { name: modelName(model.id), size: formatBytes(model.size) })}</div>
              {#if model.state === 'downloading' || model.state === 'verifying' || model.state === 'paused'}
                <div class="bar"><span style:width="{pct(model)}%"></span></div>
              {/if}
            </div>
            {#if model.state === 'missing' || model.state === 'paused' || model.state === 'corrupt'}
              <Button size="sm" variant="primary" disabled={s?.localOnly} onclick={() => aiStore.modelAction(model.id, model.state === 'corrupt' ? 'repair' : 'download')}>
                {t('ai.new.download', { size: formatBytes(model.size - (model.state === 'paused' ? model.received : 0)) })}
              </Button>
            {/if}
          </div>
        {/if}
      </div>
    </section>

    <!-- 2 · Summary, 3 · Topics -->
    {#each ['summary', 'topics'] as const as task, i (task)}
      {@const st = steps[task]}
      {@const pi = info(st.provider)}
      {@const time = stepTime(task)}
      {@const block = blocker(task)}
      <section class="step" class:off={!st.on}>
        <header>
          <span class="num">{i + 2}</span>
          <span class="icon">{#if task === 'summary'}<FileText size={15} />{:else}<ListTree size={15} />{/if}</span>
          <span class="title">{t(task === 'summary' ? 'ai.new.alsoSummary' : 'ai.new.alsoTopics')}</span>
          <span class="when">
            {#if st.on && time}<Clock size={12} />~{roughTime(time.ms)}{/if}
          </span>
          <Toggle checked={st.on} label={t(task === 'summary' ? 'ai.new.alsoSummary' : 'ai.new.alsoTopics')} onchange={(v) => (steps[task].on = v)} />
        </header>
        {#if st.on}
          <div class="rows">
            <div class="grid">
              <div class="field">
                <span class="label">{t('ai.gen.provider')}</span>
                <Select
                  label={t('ai.gen.provider')}
                  icon={pi?.local ? Cpu : Globe}
                  value={st.provider}
                  options={providers.map((p) => ({ value: p.id, label: providerName(p.id) }))}
                  onchange={(v) => (steps[task].provider = v)}
                />
              </div>
              <div class="field">
                <span class="label">{t(task === 'summary' ? 'ai.gen.detail' : 'ai.new.organization')}</span>
                <Segmented label={t('ai.gen.detail')} size="sm" stretch value={st.detail} options={detailOptions} onchange={(v) => (steps[task].detail = v)} />
              </div>
            </div>
            {#if block}
              <p class="warn"><AlertTriangle size={13} />{t(`ai.reason.${block}` as MessageKey)}</p>
            {:else if pi && !pi.local && st.provider !== 'cloud'}
              <!-- Paid providers: what is sent and what it may cost, per step. FrameLy Cloud is summed up below. -->
              <p class="hint">
                <Globe size={13} />
                <span>
                  {t('ai.gen.privacy.external', { provider: providerName(st.provider) })}
                  {#if st.provider === 'anthropic' && estimate.anthropicCostUsd}{t('ai.gen.cost', { min: money(estimate.anthropicCostUsd[0]), max: money(estimate.anthropicCostUsd[1]) })}.{:else}{t('ai.gen.costUnknown')}{/if}
                </span>
              </p>
            {/if}
          </div>
        {/if}
      </section>
    {/each}

    <div class="totals">
      <div class="total">
        {#if external.length}<Globe size={16} />{:else}<ShieldCheck size={16} />{/if}
        <div>
          <strong>{external.length ? t('ai.new.privacyExternal', { providers: [...new Set(external)].map(providerName).join(', ') }) : t('ai.new.privacyLocal')}</strong>
          {#if usesCloud}
            <div class="hint">{t('ai.cloud.privacy')}</div>
            <div class="cloud-state"><CloudState /></div>
          {/if}
          <div class="hint">{t('ai.new.privacyMedia')}</div>
        </div>
      </div>
    </div>

    <p class="fine">{t('ai.new.speakers')} {t('ai.new.disclaimer')}</p>
    {#if unsupported}<p class="error">{t('ai.new.unsupportedCpu')}</p>{/if}
  {/if}

  {#snippet footer()}
    {#if estimate}
      <span class="eta" title={total?.approximate ? t('ai.new.approx') : t('ai.new.measuredShort')}>
        <Clock size={14} />{t('ai.new.total', { time: total ? roughTime(total.ms) : '—' })}{#if total?.approximate}<span class="approx">· {t('ai.approx')}</span>{/if}
      </span>
    {/if}
    <Button variant="ghost" onclick={onclose}>{t('library.cancel')}</Button>
    <Button variant="primary" busy={starting} disabled={!estimate || (!speechReady && !reused) || unsupported || anyBlocked} onclick={start}>
      {willGenerate ? t('ai.new.startAll') : t('ai.new.start')}
    </Button>
  {/snippet}
</Sheet>

{#if consentFor}
  <ConfirmDialog
    title={t('ai.consent.title', { provider: providerName(consentFor) })}
    points={consentPoints(consentFor, aiStore.cloud)}
    note={t('ai.consent.p.revoke')}
    confirm={t('ai.consent.allow')}
    cancel={t('ai.consent.cancel')}
    onconfirm={allow}
    oncancel={() => (consentFor = null)}
  />
{/if}

<style>
  .loading {
    height: 220px;
  }
  .error {
    color: var(--warn-500);
  }
  .file {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .kind {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--icon-tile);
    color: var(--icon-accent);
  }
  .kind.video {
    background: var(--rec-tile);
    color: var(--rec-icon);
  }
  .meta {
    min-width: 0;
  }
  .name {
    font-weight: 650;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .sub,
  .hint {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .step {
    margin-top: 10px;
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
    transition: opacity var(--dur-fast);
  }
  .step header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
  }
  .step.off header {
    opacity: 0.7;
  }
  .num {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--link);
    font-size: var(--text-xs);
    font-weight: 700;
  }
  .icon {
    display: grid;
    place-items: center;
    color: var(--icon-accent);
  }
  .title {
    flex: 1;
    min-width: 0;
    font-weight: 650;
  }
  .when {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    color: var(--text-3);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 0 14px 12px 44px;
    animation: open var(--dur-med) var(--ease-out);
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(210px, 1fr));
    gap: 10px 12px;
  }
  .field {
    display: grid;
    gap: 5px;
    min-width: 0;
  }
  .field :global(.seg) {
    width: 100%;
    min-width: 0;
  }
  /* Selects keep a size that fits their short choices instead of filling the column. */
  .field > :global(label) {
    justify-self: start;
    min-width: 190px;
    max-width: 100%;
  }
  .label {
    color: var(--text-3);
    font-size: var(--text-xs);
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: uppercase;
  }
  .rows .hint {
    font-size: var(--text-xs);
  }
  .rows .hint,
  .warn {
    display: flex;
    align-items: flex-start;
    gap: 7px;
    margin: 0;
    line-height: 1.45;
  }
  .rows .hint :global(svg),
  .warn :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
  }
  .warn {
    color: var(--warn-500);
    font-size: var(--text-sm);
  }
  .need {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    font-size: var(--text-sm);
  }
  .need > :global(svg) {
    flex-shrink: 0;
    color: var(--icon-accent);
  }
  .need-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .bar {
    position: relative;
    height: 4px;
    border-radius: 99px;
    background: var(--bg);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--accent);
    transition: width var(--dur-med) var(--ease-out);
  }
  .totals {
    margin-top: 12px;
  }
  .eta {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin-right: auto;
    color: var(--text-2);
    font-size: var(--text-sm);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .eta :global(svg) {
    color: var(--icon-accent);
  }
  .approx {
    color: var(--text-3);
    font-weight: 500;
  }
  .total {
    display: flex;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    background: var(--surface-raised);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .total > :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--icon-accent);
  }
  .total .hint {
    margin-top: 3px;
    font-size: var(--text-xs);
    line-height: 1.45;
  }
  .cloud-state {
    margin-top: 4px;
  }
  .fine {
    margin: 12px 0 0;
    color: var(--text-3);
    font-size: var(--text-xs);
    line-height: 1.5;
  }
</style>
