<script lang="ts">
  import { Copy, Pencil, Check, RefreshCw, Info } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detailed, type JobStatus, type SummaryContent } from '$lib/ipc/ai';
  import { formatDuration } from '$lib/stores/format';
  import { toast } from '$lib/stores/toasts.svelte';
  import GeneratePanel from './GeneratePanel.svelte';
  import JobProgress from './JobProgress.svelte';
  import { providerName } from './format';

  interface Props {
    detail: Detailed;
    job?: JobStatus;
    onseek: (ms: number) => void;
    onchanged: () => void;
  }
  let { detail, job, onseek, onchanged }: Props = $props();

  const g = $derived(detail.summary);
  let editing = $state(false);
  let regen = $state(false);
  let draft = $state<SummaryContent>({ overview: '', points: [] });

  function startEdit() {
    if (!g) return;
    draft = structuredClone($state.snapshot(g.content) as SummaryContent);
    editing = true;
  }
  async function save() {
    draft.points = draft.points.filter((p) => p.text.trim());
    try {
      await ai.saveGenerated(detail.transcript.id, 'summary', $state.snapshot(draft));
      editing = false;
      onchanged();
    } catch {
      toast('error', t('error.generic'));
    }
  }
  async function copy() {
    if (!g) return;
    const c = g.content;
    const text = `${c.overview}\n\n${c.points.map((p) => `• ${p.ms != null ? `[${formatDuration(p.ms)}] ` : ''}${p.text}`).join('\n')}`;
    await navigator.clipboard.writeText(text);
    toast('success', t('ai.copied'));
  }
  const date = (ms: number) => new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(ms));
  // A finished regeneration closes the panel.
  $effect(() => {
    if (g?.createdAt) regen = false;
  });
</script>

{#if job}
  <div class="card job"><JobProgress {job} oncancel={() => ai.cancelJob(job.id)} /></div>
{/if}

{#if !g && !job}
  <div class="empty">
    <GeneratePanel transcriptId={detail.transcript.id} task="summary" />
  </div>
{:else if g}
  <div class="head">
    <span class="meta">
      {t('ai.gen.meta', { provider: providerName(g.provider), detail: t(`ai.detail.${g.detail}` as MessageKey), date: date(g.createdAt) })}
      {#if g.edited}· {t('ai.gen.edited')}{/if}
    </span>
    <div class="tools">
      {#if editing}
        <Button size="sm" variant="ghost" onclick={() => (editing = false)}>{t('library.cancel')}</Button>
        <Button size="sm" variant="primary" onclick={save}><Check size={13} />{t('ai.done')}</Button>
      {:else}
        <Button size="sm" variant="ghost" disabled={!!job} onclick={() => (regen = !regen)}><RefreshCw size={13} />{t('ai.gen.regenerate')}</Button>
        <Button size="sm" disabled={!!job} onclick={startEdit}><Pencil size={13} />{t('ai.edit')}</Button>
        <Button size="sm" onclick={copy}><Copy size={13} />{t('ai.copy')}</Button>
      {/if}
    </div>
  </div>

  {#if regen && !job}
    <div class="card regen"><GeneratePanel transcriptId={detail.transcript.id} task="summary" regenerate={{ detail: g.detail, provider: g.provider }} /></div>
  {/if}

  <article class="card summary" class:dim={!!job}>
    {#if editing}
      <textarea class="overview-edit" bind:value={draft.overview} rows="5"></textarea>
      <h4 class="eyebrow">{t('ai.summary.points')}</h4>
      {#each draft.points as p, i (i)}
        <div class="point-edit">
          {#if p.ms != null}<span class="chip static">{formatDuration(p.ms)}</span>{/if}
          <input bind:value={p.text} />
        </div>
      {/each}
    {:else}
      <p class="overview">{g.content.overview}</p>
      {#if g.content.points.length}
        <h4 class="eyebrow">{t('ai.summary.points')}</h4>
        <ul>
          {#each g.content.points as p, i (i)}
            <li>
              {#if p.ms != null}<button class="chip" onclick={() => onseek(p.ms!)}>{formatDuration(p.ms)}</button>{/if}
              <span>{p.text}</span>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  </article>
  <p class="verify"><Info size={13} />{t('ai.verify')}</p>
{/if}

<style>
  .card {
    padding: 16px 18px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .job,
  .regen {
    margin-bottom: 14px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 32px 16px;
  }
  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 12px;
  }
  .meta {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .tools {
    display: flex;
    gap: 6px;
  }
  .summary {
    user-select: text;
    transition: opacity var(--dur-med);
  }
  .dim {
    opacity: 0.5;
  }
  .overview {
    margin: 0 0 18px;
    font-size: 14.5px;
    line-height: 1.7;
  }
  h4 {
    margin: 0 0 10px;
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 10px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  li {
    display: flex;
    gap: 10px;
    align-items: baseline;
    line-height: 1.6;
  }
  .chip {
    flex-shrink: 0;
    padding: 1px 7px;
    border: 0;
    border-radius: 6px;
    background: var(--accent-soft);
    color: var(--link);
    font: 600 var(--text-xs) / 18px var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  button.chip:hover {
    background: color-mix(in srgb, var(--accent) 26%, transparent);
  }
  .overview-edit,
  .point-edit input {
    width: 100%;
    padding: 8px 10px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text);
    font: inherit;
    line-height: 1.6;
    resize: vertical;
    user-select: text;
  }
  .overview-edit {
    margin-bottom: 16px;
  }
  .point-edit {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 6px;
  }
  .verify {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 12px;
    color: var(--text-3);
    font-size: var(--text-sm);
  }
</style>
