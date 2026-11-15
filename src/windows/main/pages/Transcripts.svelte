<script lang="ts">
  import { onMount } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { FileUp, Library, Video, AudioLines, ChevronRight, FileText, ListTree, Sparkles } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import AiSetup from '$lib/components/ai/AiSetup.svelte';
  import JobProgress from '$lib/components/ai/JobProgress.svelte';
  import NewTranscription from '$lib/components/ai/NewTranscription.svelte';
  import LibraryPicker from '$lib/components/ai/LibraryPicker.svelte';
  import TranscriptView from '$lib/components/ai/TranscriptView.svelte';
  import { languageName } from '$lib/components/ai/format';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai } from '$lib/ipc/ai';
  import { formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { nav } from '$lib/stores/nav.svelte';
  import { languageModelId, setupPlan, speechModelId } from '$lib/ai';

  let detailId = $state<number | null>(null);
  let newPath = $state<string | null>(null);
  let picking = $state(false);
  let setupDismissed = $state(false);
  let setupShown = $state(false);

  onMount(() => {
    void aiStore.init();
  });
  // "Transcribe" from the library viewer (or anywhere else).
  $effect(() => {
    const p = nav.transcribePath;
    if (!p) return;
    nav.transcribePath = null;
    void openFor(p);
  });

  /** A file that already has a finished transcript opens it; otherwise the new-transcription sheet. */
  async function openFor(path: string) {
    const existing = await ai.find(path).catch(() => null);
    if (existing && existing.status === 'done') detailId = existing.id;
    else newPath = path;
  }

  async function importFile() {
    const exts = aiStore.overview?.importable ?? [];
    const picked = await openDialog({ multiple: false, directory: false, filters: [{ name: t('ai.title'), extensions: exts }] });
    if (typeof picked === 'string') newPath = picked;
  }

  const o = $derived(aiStore.overview);
  const plan = $derived(o ? setupPlan(o.models, speechModelId(o.defaultPrecision), languageModelId(o.profile)) : null);
  $effect(() => {
    if (plan && !plan.done) setupShown = true;
  });
  const showSetup = $derived(!!plan && !setupDismissed && (!plan.done || setupShown));
  const dateFmt = $derived(new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }));
</script>

{#if detailId != null}
  {#key detailId}
    <TranscriptView id={detailId} onback={() => (detailId = null)} />
  {/key}
{:else}
  <div class="page-root">
    <header class="head">
      <div>
        <h1>{t('ai.title')}</h1>
        <p class="sub">{t('ai.subtitle')}</p>
      </div>
      <div class="tools">
        <Button onclick={() => (picking = true)}><Library size={15} />{t('ai.fromLibrary')}</Button>
        <Button variant="primary" onclick={importFile}><FileUp size={15} />{t('ai.import')}</Button>
      </div>
    </header>

    {#if showSetup}
      <AiSetup onlater={() => (setupDismissed = true)} />
    {/if}

    {#if aiStore.transcripts.length === 0}
      {#if !showSetup}
        <div class="empty">
          <span class="empty-icon"><Sparkles size={26} /></span>
          <h2>{t('ai.empty.title')}</h2>
          <p>{t('ai.empty.body')}</p>
          <div class="empty-actions">
            <Button onclick={() => (picking = true)}><Library size={15} />{t('ai.fromLibrary')}</Button>
            <Button variant="primary" onclick={importFile}><FileUp size={15} />{t('ai.import')}</Button>
          </div>
        </div>
      {/if}
    {:else}
      <div class="list">
        {#each aiStore.transcripts as tr (tr.id)}
          {@const job = aiStore.active(tr.id, 'transcribe')}
          <button class="row" onclick={() => (detailId = tr.id)}>
            <span class="kind" class:video={tr.media.kind === 'video'}>
              {#if tr.media.kind === 'video'}<Video size={16} />{:else}<AudioLines size={16} />{/if}
            </span>
            <span class="text">
              <span class="name">{tr.media.name}</span>
              <span class="facts">
                {dateFmt.format(new Date(tr.createdAt))} · {formatDuration(tr.media.durationMs)}{#if tr.detectedLanguage} · {languageName(tr.detectedLanguage)}{/if}
              </span>
            </span>
            {#if job}
              <span class="job"><JobProgress {job} /></span>
            {:else}
              <span class="badges">
                {#if tr.status !== 'done'}<span class="status {tr.status}">{t(`ai.status.${tr.status}` as MessageKey)}</span>{/if}
                {#if tr.hasSummary}<span class="badge" title={t('ai.badge.summary')}><FileText size={12} />{t('ai.badge.summary')}</span>{/if}
                {#if tr.hasTopics}<span class="badge" title={t('ai.badge.topics')}><ListTree size={12} />{t('ai.badge.topics')}</span>{/if}
                {#if tr.edited}<span class="badge">{t('ai.badge.edited')}</span>{/if}
              </span>
            {/if}
            <ChevronRight size={16} />
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}

{#if picking}
  <LibraryPicker onclose={() => (picking = false)} onpick={(p) => { picking = false; void openFor(p); }} />
{/if}
{#if newPath}
  <NewTranscription path={newPath} onclose={() => (newPath = null)} onopen={(id) => { newPath = null; detailId = id; void aiStore.loadList(); }} />
{/if}

<style>
  .page-root {
    display: flex;
    flex-direction: column;
    gap: var(--space-5);
  }
  .head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
  }
  h1 {
    margin: 0;
    font-size: 22px;
    font-weight: 650;
  }
  .sub {
    margin: 4px 0 0;
    color: var(--text-2);
  }
  .tools {
    display: flex;
    gap: 8px;
  }
  .list {
    display: flex;
    flex-direction: column;
    border-radius: var(--radius-xl);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
    overflow: hidden;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 16px;
    border: 0;
    background: transparent;
    color: var(--text);
    text-align: left;
    transition: background var(--dur-fast);
  }
  .row + .row {
    box-shadow: inset 0 1px 0 var(--border);
  }
  .row:hover {
    background: var(--surface-hover);
  }
  .row > :global(svg:last-child) {
    flex-shrink: 0;
    color: var(--text-3);
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
  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .facts {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .job {
    width: 260px;
    flex-shrink: 0;
  }
  .badges {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
  .badge,
  .status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border-radius: 99px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-2);
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .status.interrupted,
  .status.failed,
  .status.cancelled {
    color: var(--warn-500);
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 64px 20px;
    text-align: center;
    color: var(--text-2);
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 60px;
    height: 60px;
    margin-bottom: 8px;
    border-radius: 16px;
    background: var(--icon-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  .empty h2 {
    margin: 0;
    color: var(--text);
    font-size: var(--text-lg);
  }
  .empty p {
    max-width: 460px;
    margin: 0 0 10px;
    line-height: 1.5;
  }
  .empty-actions {
    display: flex;
    gap: 8px;
  }
  @media (max-width: 760px) {
    .job {
      width: 160px;
    }
  }
</style>
