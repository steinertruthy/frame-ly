<script lang="ts">
  import { onMount } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { FileUp, Library, Sparkles, Search } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import AiSetup from '$lib/components/ai/AiSetup.svelte';
  import TranscriptCard from '$lib/components/ai/TranscriptCard.svelte';
  import NewTranscription from '$lib/components/ai/NewTranscription.svelte';
  import LibraryPicker from '$lib/components/ai/LibraryPicker.svelte';
  import TranscriptView from '$lib/components/ai/TranscriptView.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { ai } from '$lib/ipc/ai';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { nav, type TranscriptTab } from '$lib/stores/nav.svelte';
  import { languageModelId, setupPlan, speechModelId } from '$lib/ai';

  let detailId = $state<number | null>(null);
  let detailTab = $state<TranscriptTab>('transcript');
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
  // `framely --show transcript:<id>[:summary|topics]`.
  $effect(() => {
    const target = nav.transcriptTarget;
    if (!target) return;
    nav.transcriptTarget = null;
    detailTab = target.tab;
    detailId = target.id;
  });

  /** A file that already has a finished transcript opens it; otherwise the new-transcription sheet. */
  async function openFor(path: string) {
    const existing = await ai.find(path).catch(() => null);
    if (existing && existing.status === 'done') {
      detailTab = 'transcript';
      detailId = existing.id;
    }
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
  let query = $state('');
  const filtered = $derived(
    query.trim() ? aiStore.transcripts.filter((x) => x.media.name.toLowerCase().includes(query.trim().toLowerCase())) : aiStore.transcripts,
  );
  const working = $derived(filtered.filter((x) => aiStore.current(x.id)));
  const rest = $derived(filtered.filter((x) => !aiStore.current(x.id)));
</script>

{#if detailId != null}
  {#key `${detailId}:${detailTab}`}
    <TranscriptView id={detailId} initialTab={detailTab} onback={() => { detailId = null; detailTab = 'transcript'; }} />
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
      <label class="search">
        <Search size={14} />
        <input type="search" placeholder={t('ai.searchList')} bind:value={query} spellcheck="false" />
      </label>
      {#if working.length}
        <section>
          <h2 class="eyebrow">{t('ai.list.working')}</h2>
          <div class="grid">{#each working as tr (tr.id)}<TranscriptCard {tr} onopen={() => (detailId = tr.id)} />{/each}</div>
        </section>
      {/if}
      {#if rest.length}
        <section>
          {#if working.length}<h2 class="eyebrow">{t('ai.list.all')}</h2>{/if}
          <div class="grid">{#each rest as tr (tr.id)}<TranscriptCard {tr} onopen={() => (detailId = tr.id)} />{/each}</div>
        </section>
      {:else if !working.length}
        <p class="none">{t('ai.noResults', { q: query.trim() })}</p>
      {/if}
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
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    align-self: flex-start;
    width: min(320px, 100%);
    height: 32px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
  }
  .search:focus-within {
    box-shadow: var(--focus-ring);
  }
  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: none;
    color: var(--text);
    font-size: var(--text-sm);
    user-select: text;
  }
  .search input:focus {
    outline: none;
  }
  section h2 {
    margin: 0 0 var(--space-3);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    gap: 16px;
  }
  .none {
    color: var(--text-2);
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
</style>
