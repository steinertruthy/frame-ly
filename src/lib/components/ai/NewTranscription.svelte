<script lang="ts">
  import { onMount } from 'svelte';
  import { Video, AudioLines, Clock, WifiOff, Info, Download, Users } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Select from '$lib/components/Select.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Estimate, type Precision } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { isReady, TRANSCRIPTION_LANGUAGES } from '$lib/ai';
  import Sheet from './Sheet.svelte';
  import { languageName, modelName, pct, roughTime } from './format';

  interface Props {
    path: string;
    onclose: () => void;
    onopen: (transcriptId: number) => void;
  }
  let { path, onclose, onopen }: Props = $props();

  let precision = $state<Exclude<Precision, 'auto'>>('balanced');
  let language = $state('auto');
  let estimate = $state<Estimate | null>(null);
  let failed = $state<string | null>(null);
  let starting = $state(false);

  onMount(() => {
    const s = settings.value?.intelligence;
    precision = s && s.precision !== 'auto' ? s.precision : (aiStore.overview?.defaultPrecision ?? 'balanced');
    language = s?.language ?? 'auto';
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
  const ready = $derived(isReady(model ?? undefined));
  const reused = $derived(
    !!estimate?.existing &&
      estimate.existing.status === 'done' &&
      estimate.existing.precision === precision &&
      estimate.existing.language === language &&
      estimate.existing.media.size === estimate.media.size &&
      estimate.existing.media.mtime === estimate.media.mtime,
  );
  const unsupported = $derived(aiStore.overview ? !aiStore.overview.hardware.supported : false);

  async function start() {
    if (reused && estimate?.existing) return onopen(estimate.existing.id);
    starting = true;
    try {
      const r = await ai.transcribe(path, precision, language);
      onopen(r.transcript.id);
    } catch (e) {
      toast('error', t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'), 5000);
    } finally {
      starting = false;
    }
  }
</script>

<Sheet title={t('ai.new.title')} {onclose}>
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
        <div class="facts">
          {t(m.kind === 'video' ? 'ai.kind.video' : 'ai.kind.audio')} · {formatDuration(m.durationMs)} · {formatBytes(m.size)}
        </div>
      </div>
    </div>

    <div class="field">
      <div class="label">{t('ai.new.precision')}</div>
      <Segmented
        label={t('ai.new.precision')}
        stretch
        value={precision}
        options={(['fast', 'balanced', 'max'] as const).map((p) => ({ value: p, label: t(`ai.precision.${p}` as MessageKey) }))}
        onchange={(v) => (precision = v)}
      />
      <div class="hint">{t(`ai.precision.${precision}.hint` as MessageKey)}</div>
    </div>

    <div class="field row">
      <div class="label">{t('ai.new.language')}</div>
      <Select
        label={t('ai.new.language')}
        value={language}
        options={[{ value: 'auto', label: t('ai.language.auto') }, ...TRANSCRIPTION_LANGUAGES.map((c) => ({ value: c, label: languageName(c) }))]}
        onchange={(v) => (language = v)}
      />
    </div>

    {#if model && !ready}
      <div class="need">
        <Download size={16} />
        <div class="need-text">
          <div>{t('ai.new.modelNeeded', { name: modelName(model.id), size: formatBytes(model.size) })}</div>
          {#if model.state === 'downloading' || model.state === 'verifying' || model.state === 'paused'}
            <div class="bar"><span style:width="{pct(model)}%"></span></div>
          {/if}
        </div>
        {#if model.state === 'missing' || model.state === 'paused' || model.state === 'corrupt'}
          <Button size="sm" variant="primary" disabled={settings.value?.intelligence.localOnly} onclick={() => aiStore.modelAction(model.id, model.state === 'corrupt' ? 'repair' : 'download')}>
            {t('ai.new.download', { size: formatBytes(model.size - (model.state === 'paused' ? model.received : 0)) })}
          </Button>
        {/if}
      </div>
    {/if}

    <div class="estimate">
      <Clock size={15} />
      <div>
        {#if reused}
          <div>{t('ai.new.existing')}</div>
        {:else}
          <div><strong>{t('ai.new.estimate')}: ~{roughTime(estimate.time.ms)}</strong></div>
          <div class="hint">{estimate.time.approximate ? t('ai.new.approx') : t('ai.new.measured', { n: estimate.time.runs })}</div>
        {/if}
      </div>
    </div>

    <ul class="notes">
      <li><WifiOff size={13} />{t('ai.new.where')}</li>
      <li><Users size={13} />{t('ai.new.speakers')}</li>
      <li><Info size={13} />{t('ai.new.disclaimer')}</li>
    </ul>
    {#if unsupported}<p class="error">{t('ai.new.unsupportedCpu')}</p>{/if}
  {/if}

  {#snippet footer()}
    <Button variant="ghost" onclick={onclose}>{t('library.cancel')}</Button>
    <Button variant="primary" busy={starting} disabled={!estimate || (!ready && !reused) || unsupported} onclick={start}>
      {reused ? t('ai.new.open') : t('ai.new.start')}
    </Button>
  {/snippet}
</Sheet>

<style>
  .loading {
    height: 180px;
  }
  .error {
    color: var(--warn-500);
  }
  .file {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .kind {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 40px;
    height: 40px;
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
  .facts,
  .hint {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 18px;
  }
  .field.row {
    flex-direction: row;
    align-items: center;
    justify-content: space-between;
  }
  .label {
    font-weight: 600;
  }
  .need {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 18px;
    padding: 12px;
    border-radius: var(--radius-lg);
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
  .estimate {
    display: flex;
    gap: 10px;
    margin-top: 18px;
    padding: 12px;
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .estimate > :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
    color: var(--text-3);
  }
  .notes {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin: 14px 0 0;
    padding: 0;
    list-style: none;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .notes li {
    display: flex;
    gap: 8px;
  }
  .notes :global(svg) {
    flex-shrink: 0;
    margin-top: 3px;
    color: var(--text-3);
  }
</style>
