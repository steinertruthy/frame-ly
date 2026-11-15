<script lang="ts">
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { ArrowLeft, Download, Trash2, AudioLines, FileText, ListTree, Captions, Check, LoaderCircle, Clock3, Circle, AlertTriangle } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import ContextMenu, { type MenuItem } from '$lib/components/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detailed, type ExportFormat, type ExportWhat } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { formatBytes, formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import TranscriptTab from './TranscriptTab.svelte';
  import SummaryTab from './SummaryTab.svelte';
  import TopicsTab from './TopicsTab.svelte';
  import { languageName, modelName, providerName } from './format';

  interface Props {
    id: number;
    onback: () => void;
  }
  let { id, onback }: Props = $props();

  type Tab = 'transcript' | 'summary' | 'topics';
  let tab = $state<Tab>('transcript');
  let detail = $state<Detailed | null>(null);
  let currentMs = $state(0);
  let player: HTMLMediaElement | undefined = $state();
  let menu = $state<{ x: number; y: number } | null>(null);
  let confirmDelete = $state(false);

  async function load() {
    try {
      detail = await ai.get(id);
    } catch {
      onback();
    }
  }
  onMount(load);
  // Live updates: each finished block, generated content, edits.
  $effect(() => {
    if (aiStore.changed.id === id && aiStore.changed.n) void load();
  });

  const transcribeJob = $derived(aiStore.active(id, 'transcribe'));
  const summaryJob = $derived(aiStore.active(id, 'summary'));
  const topicsJob = $derived(aiStore.active(id, 'topics'));
  const keywords = $derived(detail?.topics?.content.keywords ?? []);

  type StepState = 'done' | 'running' | 'queued' | 'off' | 'problem';
  const steps = $derived.by(() => {
    if (!detail) return [];
    const tr = detail.transcript;
    const one = (kind: Tab, job: typeof transcribeJob, done: boolean, problem: boolean, sub: string | null) => {
      const state: StepState = job ? (job.state === 'running' ? 'running' : 'queued') : done ? 'done' : problem ? 'problem' : 'off';
      return { kind, job, state, sub };
    };
    return [
      one('transcript', transcribeJob, tr.status === 'done', tr.status !== 'done', tr.status === 'done' ? t('ai.step.segments', { n: detail.segments.length }) : null),
      one('summary', summaryJob, !!detail.summary, false, detail.summary ? providerName(detail.summary.provider) : null),
      one('topics', topicsJob, !!detail.topics, false, detail.topics ? providerName(detail.topics.provider) : null),
    ];
  });
  const tabIcons: Record<Tab, Component<{ size?: number }>> = { transcript: AudioLines, summary: FileText, topics: ListTree };

  function seek(ms: number) {
    currentMs = ms;
    if (!player) return;
    player.currentTime = ms / 1000;
    void player.play().catch(() => {});
  }

  async function resume() {
    if (!detail) return;
    const tr = detail.transcript;
    try {
      await ai.transcribe(tr.media.path, tr.precision, tr.language);
      await load();
    } catch (e) {
      toast('error', t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'), 5000);
    }
  }

  const stem = (name: string) => name.replace(/\.[^.]+$/, '');
  async function exportAs(what: ExportWhat, format: ExportFormat) {
    if (!detail) return;
    const d = detail;
    const suffix = what === 'transcript' ? '' : ` (${t(`ai.export.${what}` as MessageKey)})`;
    const path = await saveDialog({
      defaultPath: `${stem(d.transcript.media.name)}${suffix}.${format}`,
      filters: [{ name: t(`ai.format.${format}` as MessageKey), extensions: [format] }],
    });
    if (typeof path !== 'string') return;
    try {
      await ai.export({
        id: d.transcript.id,
        what,
        format,
        path,
        title: stem(d.transcript.media.name),
        meta: meta,
        labels: {
          decisions: t('ai.topics.decisions'),
          actions: t('ai.topics.actions'),
          dates: t('ai.topics.dates'),
          numbers: t('ai.topics.numbers'),
          owner: t('ai.topics.owner'),
          due: t('ai.topics.due'),
        },
      });
      toast('success', t('ai.exported'));
    } catch (e) {
      const code = isAppError(e) ? (e.message.includes('pdf_text') ? 'pdf_text' : e.code) : 'generic';
      toast('error', t(`error.${code}` as MessageKey) || t('error.generic'), 5000);
    }
  }

  const exportItems = $derived.by((): MenuItem[] => {
    if (!detail) return [];
    const items: MenuItem[] = [];
    const tr = t('ai.export.transcript');
    for (const f of ['txt', 'srt', 'vtt', 'md', 'pdf'] as const) items.push({ label: `${tr} · ${t(`ai.format.${f}` as MessageKey)}`, icon: f === 'srt' || f === 'vtt' ? Captions : FileText, run: () => exportAs('transcript', f) });
    for (const what of ['summary', 'topics'] as const) {
      if (!detail[what]) continue;
      for (const f of ['txt', 'md', 'pdf'] as const)
        items.push({ label: `${t(`ai.export.${what}` as MessageKey)} · ${t(`ai.format.${f}` as MessageKey)}`, icon: what === 'topics' ? ListTree : FileText, run: () => exportAs(what, f) });
    }
    return items;
  });

  const meta = $derived.by(() => {
    if (!detail) return '';
    const tr = detail.transcript;
    const date = new Intl.DateTimeFormat(locale(), { dateStyle: 'long', timeStyle: 'short' }).format(new Date(tr.createdAt));
    const lang = tr.detectedLanguage ?? (tr.language !== 'auto' ? tr.language : null);
    return [date, formatDuration(tr.media.durationMs), lang ? languageName(lang) : null, t(`ai.precision.${tr.precision}` as MessageKey)].filter(Boolean).join(' · ');
  });

  async function remove() {
    confirmDelete = false;
    try {
      await ai.remove(id);
      await aiStore.loadList();
      onback();
    } catch {
      toast('error', t('error.generic'));
    }
  }

  function onkeydown(e: KeyboardEvent) {
    const typing = (e.target as HTMLElement).closest('input, textarea, [contenteditable="true"]');
    if (typing || !player) return;
    if (e.key === ' ' && !e.metaKey && !e.ctrlKey) {
      e.preventDefault();
      if (player.paused) void player.play();
      else player.pause();
    }
  }
</script>

<svelte:window {onkeydown} />

{#if detail}
  {@const tr = detail.transcript}
  <div class="view">
    <header>
      <button class="back" onclick={onback}><ArrowLeft size={15} />{t('ai.back')}</button>
      <div class="title-row">
        <div class="titles">
          <h1 title={tr.media.path}>{stem(tr.media.name)}</h1>
          <div class="meta">{meta}</div>
        </div>
        <div class="actions">
          <Button size="sm" disabled={!detail.segments.length} onclick={(e) => { const r = (e.currentTarget as HTMLElement).getBoundingClientRect(); menu = { x: r.left, y: r.bottom + 6 }; }}>
            <Download size={14} />{t('ai.export')}
          </Button>
          <Button size="sm" variant="ghost" onclick={() => (confirmDelete = true)} title={t('ai.delete')} aria-label={t('ai.delete')}><Trash2 size={14} /></Button>
        </div>
      </div>
    </header>

    <div class="grid">
      <aside class="side">
        <div class="player" class:audio={tr.media.kind === 'audio'}>
          {#if detail.mediaAvailable}
            {#if tr.media.kind === 'video'}
              <!-- svelte-ignore a11y_media_has_caption -->
              <video bind:this={player} src={convertFileSrc(tr.media.path)} poster={framelyUrl(`transcript-thumb/${tr.id}?v=${tr.media.mtime}`)} controls preload="metadata" ontimeupdate={(e) => (currentMs = e.currentTarget.currentTime * 1000)}></video>
            {:else}
              <div class="wave"><AudioLines size={28} /></div>
              <audio bind:this={player} src={convertFileSrc(tr.media.path)} controls preload="metadata" ontimeupdate={(e) => (currentMs = e.currentTarget.currentTime * 1000)}></audio>
            {/if}
          {:else}
            <p class="missing">{t('ai.mediaMissing')}</p>
          {/if}
        </div>

        <div class="card steps">
          <h4 class="eyebrow">{t('ai.steps')}</h4>
          {#each steps as s (s.kind)}
            {@const Icon = tabIcons[s.kind]}
            <button class="step {s.state}" class:current={tab === s.kind} onclick={() => (tab = s.kind)}>
              <span class="dot">
                {#if s.state === 'done'}<Check size={13} />{:else if s.state === 'running'}<LoaderCircle size={13} class="spin" />{:else if s.state === 'queued'}<Clock3 size={13} />{:else if s.state === 'problem'}<AlertTriangle size={13} />{:else}<Circle size={13} />{/if}
              </span>
              <span class="step-text">
                <span class="step-name"><Icon size={13} />{t(`ai.tab.${s.kind}` as MessageKey)}</span>
                <span class="step-sub">
                  {#if s.job}
                    {t(`ai.phase.${s.job.phase}` as MessageKey)}{#if s.job.progress != null} · {Math.round(s.job.progress * 100)}%{/if}
                  {:else if s.state === 'problem'}
                    {t(`ai.status.${tr.status}` as MessageKey)}
                  {:else if s.state === 'off'}
                    {t('ai.step.notGenerated')}
                  {:else}
                    {s.sub}
                  {/if}
                </span>
                {#if s.job && s.job.progress != null}<span class="mini"><span style:width="{Math.round(s.job.progress * 100)}%"></span></span>{/if}
              </span>
            </button>
          {/each}
        </div>

        <dl class="card details">
          <div><dt>{t('ai.duration')}</dt><dd>{formatDuration(tr.media.durationMs)}</dd></div>
          <div><dt>{t('ai.size')}</dt><dd>{formatBytes(tr.media.size)}</dd></div>
          <div><dt>{t('ai.new.precision')}</dt><dd>{t(`ai.precision.${tr.precision}` as MessageKey)}</dd></div>
          <div><dt>{t('ai.step.model')}</dt><dd>{modelName(tr.modelId)}</dd></div>
        </dl>
      </aside>

      <section class="main">
        <Segmented
          label={t('ai.title')}
          stretch
          value={tab}
          options={[
            { value: 'transcript', label: t('ai.tab.transcript'), icon: AudioLines },
            { value: 'summary', label: t('ai.tab.summary'), icon: FileText },
            { value: 'topics', label: t('ai.tab.topics'), icon: ListTree },
          ]}
          onchange={(v) => (tab = v)}
        />
        <!-- Only this area scrolls. Tabs stay mounted so switching keeps search, scroll and expanded topics. -->
        <div class="scroll" hidden={tab !== 'transcript'}>
          <TranscriptTab {detail} {currentMs} {keywords} job={transcribeJob} onseek={seek} onchanged={load} onresume={resume} />
        </div>
        <div class="scroll" hidden={tab !== 'summary'}>
          {#if tr.status === 'done'}<SummaryTab {detail} job={summaryJob} onseek={seek} onchanged={load} />{:else}<p class="wait">{t('ai.step.afterTranscript')}</p>{/if}
        </div>
        <div class="scroll" hidden={tab !== 'topics'}>
          {#if tr.status === 'done'}<TopicsTab {detail} job={topicsJob} onseek={seek} onchanged={load} />{:else}<p class="wait">{t('ai.step.afterTranscript')}</p>{/if}
        </div>
      </section>
    </div>
  </div>

  {#if menu}<ContextMenu x={menu.x} y={menu.y} items={exportItems} onclose={() => (menu = null)} />{/if}
  {#if confirmDelete}
    <ConfirmDialog
      title={t('ai.delete.confirm', { name: tr.media.name })}
      body={t('ai.delete.body')}
      confirm={t('ai.delete')}
      cancel={t('library.cancel')}
      danger
      onconfirm={remove}
      oncancel={() => (confirmDelete = false)}
    />
  {/if}
{/if}

<style>
  /* Fills the page; only the tab content scrolls. */
  .view {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    min-height: 0;
  }
  header {
    flex-shrink: 0;
  }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 8px -6px;
    padding: 4px 8px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--text-2);
    font-weight: 600;
    font-size: var(--text-sm);
  }
  .back:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .title-row {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
  }
  .titles {
    min-width: 0;
  }
  h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 650;
    letter-spacing: -0.01em;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    margin-top: 4px;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
  .grid {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(260px, 0.8fr) minmax(0, 1.7fr);
    gap: 22px;
  }
  .side {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-height: 0;
    overflow-y: auto;
  }
  .main {
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 0;
    min-height: 0;
  }
  .scroll {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0 6px 24px 2px;
    scroll-padding-top: 60px;
  }
  .scroll[hidden] {
    display: none;
  }
  @media (max-width: 960px) {
    .view {
      height: auto;
    }
    .grid {
      grid-template-columns: 1fr;
    }
    .side,
    .scroll {
      overflow: visible;
    }
  }
  .player {
    flex-shrink: 0;
    overflow: hidden;
    border-radius: var(--radius-lg);
    background: #000;
    box-shadow:
      0 0 0 1px var(--border),
      0 24px 50px -28px rgba(0, 0, 0, 0.7);
  }
  .player video {
    display: block;
    width: 100%;
    max-height: 40vh;
    background: #000;
  }
  .player.audio {
    padding: 18px 14px 14px;
    background: var(--hero), var(--surface);
  }
  .wave {
    display: grid;
    place-items: center;
    height: 72px;
    color: var(--icon-accent);
  }
  .player audio {
    width: 100%;
  }
  .missing {
    margin: 0;
    padding: 28px 18px;
    color: var(--text-2);
    background: var(--surface);
    text-align: center;
  }
  .card {
    flex-shrink: 0;
    margin: 0;
    padding: 12px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .steps h4 {
    margin: 2px 4px 8px;
  }
  .step {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    width: 100%;
    padding: 8px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text);
    text-align: left;
  }
  .step:hover {
    background: var(--surface-hover);
  }
  .step.current {
    background: var(--nav-active);
    box-shadow: inset 0 0 0 1px var(--nav-active-border);
  }
  .dot {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
  }
  .done .dot {
    background: color-mix(in srgb, var(--ok-500) 16%, transparent);
    box-shadow: none;
    color: var(--ok-500);
  }
  .running .dot,
  .queued .dot {
    background: var(--accent-soft);
    box-shadow: none;
    color: var(--link);
  }
  .problem .dot {
    color: var(--warn-500);
  }
  .dot :global(.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .step-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .step-name {
    display: flex;
    align-items: center;
    gap: 6px;
    font-weight: 600;
  }
  .step-name :global(svg) {
    color: var(--text-3);
  }
  .step-sub {
    color: var(--text-2);
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .mini {
    position: relative;
    height: 3px;
    margin-top: 4px;
    border-radius: 99px;
    background: var(--bg);
    overflow: hidden;
  }
  .mini span {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--accent);
    transition: width var(--dur-slow) var(--ease-out);
  }
  .details {
    padding: 4px 0;
    font-size: var(--text-sm);
  }
  .details div {
    display: flex;
    justify-content: space-between;
    gap: 12px;
    padding: 7px 14px;
  }
  .details div + div {
    box-shadow: inset 0 1px 0 var(--border);
  }
  dt {
    color: var(--text-2);
  }
  dd {
    margin: 0;
    text-align: right;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .wait {
    color: var(--text-2);
    padding: 32px 0;
    text-align: center;
  }
</style>
