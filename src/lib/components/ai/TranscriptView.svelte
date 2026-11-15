<script lang="ts">
  import { onMount } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { save as saveDialog } from '@tauri-apps/plugin-dialog';
  import { ArrowLeft, Download, Trash2, AudioLines, FileText, ListTree, Captions } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import ContextMenu, { type MenuItem } from '$lib/components/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detailed, type ExportFormat, type ExportWhat } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import { formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import TranscriptTab from './TranscriptTab.svelte';
  import SummaryTab from './SummaryTab.svelte';
  import TopicsTab from './TopicsTab.svelte';
  import { languageName } from './format';

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
      <div class="side">
        <div class="player" class:audio={tr.media.kind === 'audio'}>
          {#if detail.mediaAvailable}
            {#if tr.media.kind === 'video'}
              <!-- svelte-ignore a11y_media_has_caption -->
              <video bind:this={player} src={convertFileSrc(tr.media.path)} controls preload="metadata" ontimeupdate={(e) => (currentMs = e.currentTarget.currentTime * 1000)}></video>
            {:else}
              <div class="wave"><AudioLines size={28} /></div>
              <audio bind:this={player} src={convertFileSrc(tr.media.path)} controls preload="metadata" ontimeupdate={(e) => (currentMs = e.currentTarget.currentTime * 1000)}></audio>
            {/if}
          {:else}
            <p class="missing">{t('ai.mediaMissing')}</p>
          {/if}
        </div>
      </div>

      <div class="main">
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
        <!-- Tabs stay mounted so switching keeps search, scroll and expanded topics. -->
        <div class="tab" hidden={tab !== 'transcript'}>
          <TranscriptTab {detail} {currentMs} {keywords} job={transcribeJob} onseek={seek} onchanged={load} onresume={resume} />
        </div>
        <div class="tab" hidden={tab !== 'summary'}>
          {#if tr.status === 'done'}<SummaryTab {detail} job={summaryJob} onseek={seek} onchanged={load} />{:else}<p class="wait">{t('ai.partial')}</p>{/if}
        </div>
        <div class="tab" hidden={tab !== 'topics'}>
          {#if tr.status === 'done'}<TopicsTab {detail} job={topicsJob} onseek={seek} onchanged={load} />{:else}<p class="wait">{t('ai.partial')}</p>{/if}
        </div>
      </div>
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
  .view {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .back {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 10px -6px;
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
    display: grid;
    grid-template-columns: minmax(280px, 0.85fr) minmax(0, 1.6fr);
    gap: 22px;
    align-items: start;
  }
  .side {
    position: sticky;
    top: 0;
  }
  @media (max-width: 960px) {
    .grid {
      grid-template-columns: 1fr;
    }
    .side {
      position: static;
    }
  }
  .player {
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
    max-height: 60vh;
    background: #000;
  }
  .player.audio {
    padding: 18px 14px 14px;
    background: var(--hero), var(--surface);
  }
  .wave {
    display: grid;
    place-items: center;
    height: 80px;
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
  .main {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-width: 0;
  }
  .tab[hidden] {
    display: none;
  }
  .wait {
    color: var(--text-2);
    padding: 32px 0;
    text-align: center;
  }
</style>
