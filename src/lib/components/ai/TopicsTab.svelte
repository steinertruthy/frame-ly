<script lang="ts">
  import { ListTree, Copy, Pencil, Check, RefreshCw, Info, Gavel, ListTodo, CalendarDays, Hash, ChevronsDownUp, ChevronsUpDown } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import Button from '$lib/components/Button.svelte';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type Detailed, type JobStatus, type TopicNode as Node, type TopicsContent } from '$lib/ipc/ai';
  import { formatDuration } from '$lib/stores/format';
  import { toast } from '$lib/stores/toasts.svelte';
  import GeneratePanel from './GeneratePanel.svelte';
  import JobProgress from './JobProgress.svelte';
  import TopicNode from './TopicNode.svelte';
  import { providerName } from './format';

  interface Props {
    detail: Detailed;
    job?: JobStatus;
    onseek: (ms: number) => void;
    onchanged: () => void;
  }
  let { detail, job, onseek, onchanged }: Props = $props();

  const g = $derived(detail.topics);
  let editing = $state(false);
  let regen = $state(false);
  let draft = $state<TopicsContent | null>(null);
  const content = $derived(editing && draft ? draft : g?.content);

  function paths(nodes: Node[], prefix: string): string[] {
    return nodes.flatMap((n, i) => [`${prefix}${i}`, ...paths(n.children, `${prefix}${i}.`)]);
  }
  // First level open by default; state survives switching tabs (component stays mounted).
  let expanded = $state(new Set<string>());
  let seeded = 0;
  $effect(() => {
    if (g && g.createdAt !== seeded) {
      seeded = g.createdAt;
      expanded = new Set(g.content.topics.map((_, i) => `${i}`));
      regen = false;
    }
  });
  const all = $derived(g ? paths(g.content.topics, '') : []);
  const allOpen = $derived(all.length > 0 && all.every((p) => expanded.has(p)));
  function toggle(p: string) {
    const next = new Set(expanded);
    if (next.has(p)) next.delete(p);
    else next.add(p);
    expanded = next;
  }

  function startEdit() {
    if (!g) return;
    draft = structuredClone($state.snapshot(g.content) as TopicsContent);
    editing = true;
  }
  async function save() {
    if (!draft) return;
    try {
      await ai.saveGenerated(detail.transcript.id, 'topics', $state.snapshot(draft));
      editing = false;
      onchanged();
    } catch {
      toast('error', t('error.generic'));
    }
  }
  function asText(c: TopicsContent): string {
    const lines: string[] = [];
    const walk = (n: Node, d: number) => {
      lines.push(`${'  '.repeat(d)}${d ? '-' : '#'} ${n.title}${n.ms != null ? ` [${formatDuration(n.ms)}]` : ''}`);
      n.points.forEach((p) => lines.push(`${'  '.repeat(d + 1)}• ${p}`));
      n.children.forEach((ch) => walk(ch, d + 1));
    };
    c.topics.forEach((n) => walk(n, 0));
    for (const [key, items] of [['decisions', c.decisions], ['actions', c.actions], ['dates', c.dates], ['numbers', c.numbers]] as const) {
      if (!items.length) continue;
      lines.push('', t(`ai.topics.${key}` as MessageKey));
      for (const i of items) lines.push(`• ${i.ms != null ? `[${formatDuration(i.ms)}] ` : ''}${i.text}`);
    }
    return lines.join('\n');
  }
  async function copy() {
    if (!g) return;
    await navigator.clipboard.writeText(asText(g.content));
    toast('success', t('ai.copied'));
  }
  const date = (ms: number) => new Intl.DateTimeFormat(locale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(ms));
  const sections: { key: 'decisions' | 'actions' | 'dates' | 'numbers'; icon: Component<{ size?: number }> }[] = [
    { key: 'decisions', icon: Gavel },
    { key: 'actions', icon: ListTodo },
    { key: 'dates', icon: CalendarDays },
    { key: 'numbers', icon: Hash },
  ];
</script>

{#if job}
  <div class="card job"><JobProgress {job} oncancel={() => ai.cancelJob(job.id)} /></div>
{/if}

{#if !g && !job}
  <div class="empty">
    <span class="icon"><ListTree size={22} /></span>
    <p>{t('ai.gen.topics.empty')}</p>
    <GeneratePanel transcriptId={detail.transcript.id} task="topics" />
  </div>
{:else if g && content}
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
        <Button size="sm" variant="ghost" onclick={() => (expanded = allOpen ? new Set() : new Set(all))}>
          {#if allOpen}<ChevronsDownUp size={13} />{t('ai.topics.collapseAll')}{:else}<ChevronsUpDown size={13} />{t('ai.topics.expandAll')}{/if}
        </Button>
        <Button size="sm" variant="ghost" disabled={!!job} onclick={() => (regen = !regen)}><RefreshCw size={13} />{t('ai.gen.regenerate')}</Button>
        <Button size="sm" disabled={!!job} onclick={startEdit}><Pencil size={13} />{t('ai.edit')}</Button>
        <Button size="sm" onclick={copy}><Copy size={13} />{t('ai.copy')}</Button>
      {/if}
    </div>
  </div>

  {#if regen && !job}
    <div class="card regen"><GeneratePanel transcriptId={detail.transcript.id} task="topics" regenerate={{ detail: g.detail, provider: g.provider }} /></div>
  {/if}

  <div class="layout" class:dim={!!job}>
    <div class="tree">
      {#each content.topics as node, i (i)}
        <TopicNode {node} depth={0} {expanded} path={`${i}`} {editing} {onseek} ontoggle={toggle} />
      {/each}
    </div>
    <aside>
      {#each sections as s (s.key)}
        {#if content[s.key].length}
          <section>
            <h4 class="eyebrow"><s.icon size={13} />{t(`ai.topics.${s.key}` as MessageKey)}</h4>
            <ul>
              {#each content[s.key] as item, i (i)}
                <li>
                  {#if item.ms != null}<button class="chip" onclick={() => onseek(item.ms!)}>{formatDuration(item.ms)}</button>{/if}
                  <span>
                    {#if editing}<input bind:value={item.text} />{:else}{item.text}{/if}
                    {#if s.key === 'actions' && 'owner' in item && (item.owner || item.due)}
                      <span class="extra">
                        {#if item.owner}{t('ai.topics.owner')}: {item.owner}{/if}{#if item.owner && item.due} · {/if}{#if item.due}{t('ai.topics.due')}: {item.due}{/if}
                      </span>
                    {/if}
                  </span>
                </li>
              {/each}
            </ul>
          </section>
        {/if}
      {/each}
      {#if content.keywords.length}
        <section>
          <h4 class="eyebrow">{t('ai.topics.keywords')}</h4>
          <div class="keywords">{#each content.keywords as k (k)}<span>{k}</span>{/each}</div>
        </section>
      {/if}
    </aside>
  </div>
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
    gap: 14px;
    padding: 48px 16px;
    text-align: center;
  }
  .empty .icon {
    display: grid;
    place-items: center;
    width: 52px;
    height: 52px;
    border-radius: 15px;
    background: var(--icon-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  .empty p {
    margin: 0;
    color: var(--text-2);
    max-width: 440px;
  }
  .empty :global(.panel) {
    align-items: center;
  }
  .empty :global(.facts),
  .empty :global(.controls) {
    justify-content: center;
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
    flex-wrap: wrap;
  }
  .layout {
    display: grid;
    grid-template-columns: minmax(0, 1.5fr) minmax(220px, 1fr);
    gap: 16px;
    align-items: start;
    transition: opacity var(--dur-med);
  }
  .dim {
    opacity: 0.5;
  }
  @media (max-width: 1100px) {
    .layout {
      grid-template-columns: 1fr;
    }
  }
  aside {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  aside section {
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  h4 {
    display: flex;
    align-items: center;
    gap: 6px;
    margin: 0 0 8px;
  }
  ul {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin: 0;
    padding: 0;
    list-style: none;
    user-select: text;
  }
  li {
    display: flex;
    gap: 8px;
    align-items: baseline;
    line-height: 1.5;
    font-size: var(--text-md);
  }
  .extra {
    display: block;
    color: var(--text-2);
    font-size: var(--text-sm);
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
  .keywords {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .keywords span {
    padding: 2px 9px;
    border-radius: 99px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--link);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  input {
    width: 100%;
    padding: 4px 8px;
    border: 0;
    border-radius: 6px;
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text);
    font: inherit;
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
