<script lang="ts">
  import { AudioLines, FileText, ListTree, Check, LoaderCircle, AlertTriangle } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import type { Transcript } from '$lib/ipc/ai';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { formatDuration } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { languageName, transcriptTitle } from './format';

  interface Props {
    tr: Transcript;
    onopen: () => void;
  }
  let { tr, onopen }: Props = $props();

  let thumbFailed = $state(false);
  const job = $derived(aiStore.current(tr.id));
  const stem = $derived(transcriptTitle(tr));
  const date = $derived(new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(tr.createdAt)));
  const percent = $derived(job?.progress != null ? Math.round(job.progress * 100) : null);

  type StepState = 'done' | 'active' | 'queued' | 'off' | 'problem';
  function stepState(kind: 'transcribe' | 'summary' | 'topics'): StepState {
    const j = aiStore.active(tr.id, kind);
    if (j) return j.state === 'running' ? 'active' : 'queued';
    if (kind === 'transcribe') return tr.status === 'done' ? 'done' : tr.status === 'queued' || tr.status === 'running' ? 'queued' : 'problem';
    return (kind === 'summary' ? tr.hasSummary : tr.hasTopics) ? 'done' : 'off';
  }
  const steps = $derived<{ kind: 'transcribe' | 'summary' | 'topics'; label: MessageKey; icon: Component<{ size?: number }> }[]>([
    { kind: 'transcribe', label: 'ai.tab.transcript', icon: AudioLines },
    { kind: 'summary', label: 'ai.tab.summary', icon: FileText },
    { kind: 'topics', label: 'ai.tab.topics', icon: ListTree },
  ]);
</script>

<button class="card" onclick={onopen} title={tr.media.path}>
  <span class="thumb" class:audio={tr.media.kind === 'audio' || thumbFailed}>
    {#if tr.media.kind === 'video' && !thumbFailed}
      <img src={framelyUrl(`transcript-thumb/${tr.id}?v=${tr.media.mtime}`)} alt="" loading="lazy" onerror={() => (thumbFailed = true)} />
    {:else}
      <span class="wave" aria-hidden="true">
        {#each Array.from({ length: 28 }) as _, i (i)}<i style:height="{18 + Math.abs(Math.sin(i * 1.7 + tr.id)) * 62}%"></i>{/each}
      </span>
    {/if}
    <span class="duration">{formatDuration(tr.media.durationMs)}</span>
    {#if job}
      <span class="working">
        <span class="phase"><LoaderCircle size={13} />{t(`ai.phase.${job.phase}` as MessageKey)}{#if percent != null} · {percent}%{/if}</span>
        <span class="bar" class:indeterminate={percent == null}><span style:width={percent == null ? undefined : `${percent}%`}></span></span>
      </span>
    {:else if tr.status !== 'done'}
      <span class="flag"><AlertTriangle size={12} />{t(`ai.status.${tr.status}` as MessageKey)}</span>
    {/if}
  </span>
  <span class="body">
    <span class="name">{stem}</span>
    <span class="meta">{[date, tr.detectedLanguage ? languageName(tr.detectedLanguage) : null].filter(Boolean).join(' · ')}</span>
    <span class="steps">
      {#each steps as s (s.kind)}
        {@const state = stepState(s.kind)}
        <span class="step {state}" title={t(s.label)}>
          {#if state === 'done'}<Check size={11} />{:else if state === 'active'}<LoaderCircle size={11} class="spin" />{:else}<s.icon size={11} />{/if}
          {t(s.label)}
        </span>
      {/each}
    </span>
  </span>
</button>

<style>
  .card {
    display: flex;
    flex-direction: column;
    min-width: 0;
    padding: 0;
    border: 0;
    border-radius: var(--radius-xl);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text);
    text-align: left;
    overflow: hidden;
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out);
  }
  .card:hover {
    transform: translateY(-2px);
    box-shadow:
      inset 0 0 0 1px var(--border-strong),
      0 18px 36px -24px rgba(0, 0, 0, 0.7);
  }
  .card:focus-visible {
    box-shadow: var(--focus-ring);
  }
  .thumb {
    position: relative;
    display: block;
    aspect-ratio: 16 / 9;
    background: #000;
    overflow: hidden;
  }
  .thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform var(--dur-slow) var(--ease-out);
  }
  .card:hover .thumb img {
    transform: scale(1.03);
  }
  .thumb.audio {
    background: var(--hero), linear-gradient(135deg, var(--brand-tile), var(--surface-2));
  }
  .wave {
    position: absolute;
    inset: 22% 14%;
    display: flex;
    align-items: center;
    gap: 3px;
  }
  .wave i {
    flex: 1;
    border-radius: 99px;
    background: color-mix(in srgb, var(--icon-accent) 55%, transparent);
  }
  .duration {
    position: absolute;
    right: 8px;
    bottom: 8px;
    padding: 1px 6px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.62);
    color: #fff;
    font: 600 var(--text-xs) / 18px var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .working {
    position: absolute;
    inset: auto 0 0 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 22px 10px 10px;
    background: linear-gradient(transparent, rgba(5, 8, 14, 0.88));
    color: #fff;
  }
  .phase {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .phase :global(svg),
  .step :global(.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  .bar {
    position: relative;
    height: 3px;
    border-radius: 99px;
    background: rgba(255, 255, 255, 0.18);
    overflow: hidden;
  }
  .bar span {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--accent);
    transition: width var(--dur-slow) var(--ease-out);
  }
  .bar.indeterminate span {
    width: 30%;
    animation: slide 1.4s var(--ease-inout) infinite;
  }
  @keyframes slide {
    from {
      transform: translateX(-100%);
    }
    to {
      transform: translateX(340%);
    }
  }
  .flag {
    position: absolute;
    left: 8px;
    top: 8px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 8px;
    border-radius: 99px;
    background: rgba(0, 0, 0, 0.66);
    color: #f7b23b;
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .body {
    display: flex;
    flex-direction: column;
    gap: 3px;
    padding: 12px 14px 14px;
  }
  .name {
    font-weight: 650;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .meta {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .steps {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 8px;
  }
  .step {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 7px;
    border-radius: 99px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
    font-size: var(--text-xs);
    font-weight: 600;
  }
  .step.done {
    color: var(--ok-500);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--ok-500) 28%, transparent);
  }
  .step.active,
  .step.queued {
    color: var(--link);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--accent) 35%, transparent);
  }
  .step.problem {
    color: var(--warn-500);
  }
</style>
