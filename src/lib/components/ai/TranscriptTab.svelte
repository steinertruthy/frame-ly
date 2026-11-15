<script lang="ts">
  import { tick } from 'svelte';
  import { Search, Pencil, Check, Copy, LocateFixed, RotateCcw } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detailed, type JobStatus, type Segment } from '$lib/ipc/ai';
  import { formatDuration } from '$lib/stores/format';
  import { toast } from '$lib/stores/toasts.svelte';
  import { highlight, LOW_CONFIDENCE, segmentAt } from '$lib/ai';
  import JobProgress from './JobProgress.svelte';

  interface Props {
    detail: Detailed;
    currentMs: number;
    keywords: string[];
    job?: JobStatus;
    onseek: (ms: number) => void;
    onchanged: () => void;
    onresume: () => void;
  }
  let { detail, currentMs, keywords, job, onseek, onchanged, onresume }: Props = $props();

  let search = $state('');
  let follow = $state(true);
  let editing = $state(false);
  let body: HTMLDivElement | undefined = $state();

  const segments = $derived(detail.segments);
  const active = $derived(segmentAt(segments, currentMs));
  const matches = $derived(
    search.trim() ? segments.reduce((n, s) => n + (s.text.toLowerCase().split(search.trim().toLowerCase()).length - 1), 0) : 0,
  );
  const t0 = $derived(detail.transcript);
  const resumable = $derived(!job && (t0.status === 'interrupted' || t0.status === 'cancelled' || t0.status === 'failed'));

  // Keep the passage being played in view (only when it leaves the visible area).
  $effect(() => {
    const i = active;
    if (!follow || editing || i < 0 || !body) return;
    const el = body.querySelector<HTMLElement>(`[data-seg="${i}"]`);
    const scroller = el?.closest('.scroll');
    if (!el || !scroller) return;
    const r = el.getBoundingClientRect();
    const v = scroller.getBoundingClientRect();
    if (r.top < v.top + 120 || r.bottom > v.bottom - 60) el.scrollIntoView({ block: 'center', behavior: 'smooth' });
  });

  async function save(seg: Segment, el: HTMLElement) {
    const text = el.innerText.replace(/\s+/g, ' ').trim();
    if (!text || text === seg.text) {
      el.innerText = seg.text;
      return;
    }
    try {
      await ai.editSegment(t0.id, seg.idx, text);
      onchanged();
    } catch {
      toast('error', t('error.generic'));
      el.innerText = seg.text;
    }
  }

  async function copy() {
    const text = detail.paragraphs
      .map((p) => `[${formatDuration(p.startMs)}] ${p.segments.map((i) => segments[i]?.text ?? '').join(' ')}`)
      .join('\n\n');
    await navigator.clipboard.writeText(text);
    toast('success', t('ai.copied'));
  }

  async function toggleEdit() {
    editing = !editing;
    if (editing) {
      await tick();
      body?.querySelector<HTMLElement>('[contenteditable="true"]')?.focus();
    }
  }
</script>

{#if job}
  <div class="job"><JobProgress {job} oncancel={() => ai.cancelJob(job.id)} /></div>
{:else if resumable}
  <div class="resume">
    <span>{t(`ai.status.${t0.status}` as MessageKey)}{#if t0.error} · {t(`error.${t0.error}` as MessageKey) || t0.error}{/if}</span>
    <Button size="sm" variant="primary" onclick={onresume}><RotateCcw size={13} />{t(t0.status === 'failed' ? 'ai.retry' : 'ai.resume')}</Button>
  </div>
{/if}

<div class="toolbar">
  <label class="search">
    <Search size={14} />
    <input type="search" placeholder={t('ai.search')} bind:value={search} spellcheck="false" />
    {#if search.trim()}<span class="count">{matches ? t('ai.matches', { n: matches }) : t('ai.noResults', { q: search.trim() })}</span>{/if}
  </label>
  <div class="tools">
    <button class="tool" class:on={follow} aria-pressed={follow} onclick={() => (follow = !follow)} title={t('ai.follow')}><LocateFixed size={15} /></button>
    <Button size="sm" variant={editing ? 'primary' : 'secondary'} disabled={!segments.length || !!job} onclick={toggleEdit}>
      {#if editing}<Check size={13} />{t('ai.done')}{:else}<Pencil size={13} />{t('ai.edit')}{/if}
    </Button>
    <Button size="sm" disabled={!segments.length} onclick={copy}><Copy size={13} />{t('ai.copy')}</Button>
  </div>
</div>

{#if editing}<p class="note">{t('ai.editHint')}</p>{/if}

<div class="text" class:editing bind:this={body}>
  {#each detail.paragraphs as p (p.startMs)}
    <div class="para">
      <button class="time" onclick={() => onseek(p.startMs)}>{formatDuration(p.startMs)}</button>
      <p>
        {#each p.segments as i (i)}
          {@const seg = segments[i]!}
          {#if editing}
            <span class="seg" data-seg={i} contenteditable="true" role="textbox" tabindex="0" spellcheck="true" onblur={(e) => save(seg, e.currentTarget)}>{seg.text}</span>{' '}
          {:else}
            <span
              class="seg"
              class:active={i === active}
              class:edited={seg.edited}
              data-seg={i}
              role="button"
              tabindex="-1"
              onclick={() => onseek(seg.startMs)}
              onkeydown={(e) => e.key === 'Enter' && onseek(seg.startMs)}
            >
              {#if search.trim() || !seg.words.length}
                {#each highlight(seg.text, search, keywords) as piece, k (k)}{#if piece.mark}<mark class={piece.mark}>{piece.text}</mark>{:else}{piece.text}{/if}{/each}
              {:else}
                {#each seg.words as w, k (k)}{#if k > 0}{' '}{/if}{#each highlight(w.text, '', keywords) as piece, j (j)}<span class:low={w.p < LOW_CONFIDENCE} class:key={piece.mark === 'key'}>{piece.text}</span>{/each}{/each}
              {/if}
            </span>{' '}
          {/if}
        {/each}
      </p>
    </div>
  {/each}
  {#if !segments.length && t0.status === 'done'}
    <p class="empty">{t('ai.noText')}</p>
  {/if}
</div>

{#if job && segments.length}<p class="note">{t('ai.partial')}</p>{/if}
{#if segments.some((s) => s.words.some((w) => w.p < LOW_CONFIDENCE))}<p class="note low-note"><span class="low">abc</span> {t('ai.lowConfidence')}</p>{/if}

<style>
  .job,
  .resume {
    margin-bottom: 16px;
    padding: 14px 16px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .resume {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    color: var(--warn-500);
    font-weight: 600;
  }
  .toolbar {
    position: sticky;
    top: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0 12px;
    background: linear-gradient(var(--bg) 80%, transparent);
  }
  .search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    min-width: 0;
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
  .count {
    flex-shrink: 0;
    font-size: var(--text-xs);
  }
  .tools {
    display: flex;
    gap: 6px;
  }
  .tool {
    display: grid;
    place-items: center;
    width: 30px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--text-3);
  }
  .tool:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .tool.on {
    color: var(--icon-accent);
    background: var(--accent-soft);
  }
  .text {
    display: flex;
    flex-direction: column;
    gap: 18px;
    user-select: text;
  }
  .para {
    display: grid;
    grid-template-columns: 56px 1fr;
    gap: 12px;
  }
  .time {
    align-self: start;
    margin-top: 2px;
    padding: 2px 6px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-3);
    font: 500 var(--text-xs) / 18px var(--font-mono);
    font-variant-numeric: tabular-nums;
    text-align: left;
  }
  .time:hover {
    background: var(--accent-soft);
    color: var(--link);
  }
  p {
    margin: 0;
    line-height: 1.7;
    font-size: 14px;
    color: var(--text);
  }
  .seg {
    border-radius: 4px;
    cursor: pointer;
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .seg:hover {
    background: var(--surface-hover);
  }
  .seg.active {
    background: var(--accent-soft);
    box-shadow: 0 0 0 2px var(--accent-soft);
  }
  .editing .seg {
    cursor: text;
    padding: 1px 2px;
    box-shadow: inset 0 -1px 0 var(--border-strong);
  }
  .editing .seg:focus {
    outline: none;
    background: var(--accent-soft);
  }
  .low {
    text-decoration: underline dotted color-mix(in srgb, var(--warn-500) 70%, transparent);
    text-underline-offset: 3px;
  }
  .key {
    color: var(--link);
    font-weight: 600;
  }
  mark {
    border-radius: 3px;
    color: inherit;
    background: color-mix(in srgb, var(--warn-500) 35%, transparent);
  }
  mark.key {
    background: none;
    color: var(--link);
    font-weight: 600;
  }
  .note {
    margin-top: 14px;
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .low-note {
    margin-top: 24px;
  }
  .empty {
    color: var(--text-2);
    text-align: center;
    padding: 40px 0;
  }
</style>
