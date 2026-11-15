<script lang="ts">
  import { ChevronRight } from '@lucide/svelte';
  import type { TopicNode } from '$lib/ipc/ai';
  import { formatDuration } from '$lib/stores/format';
  import Self from './TopicNode.svelte';

  interface Props {
    node: TopicNode;
    depth: number;
    expanded: Set<string>;
    path: string;
    editing: boolean;
    onseek: (ms: number) => void;
    ontoggle: (path: string) => void;
  }
  let { node, depth, expanded, path, editing, onseek, ontoggle }: Props = $props();
  const hasBody = $derived(node.points.length > 0 || node.children.length > 0);
  const open = $derived(editing || expanded.has(path));
</script>

<div class="node depth-{depth}" class:open>
  <div class="row">
    <button class="twisty" disabled={!hasBody || editing} aria-expanded={open} aria-label={node.title} onclick={() => ontoggle(path)}>
      <ChevronRight size={14} />
    </button>
    {#if editing}
      <input class="title-edit" bind:value={node.title} />
    {:else}
      <button class="title" onclick={() => (hasBody ? ontoggle(path) : node.ms != null && onseek(node.ms))}>{node.title}</button>
    {/if}
    {#if node.ms != null}<button class="chip" onclick={() => onseek(node.ms!)}>{formatDuration(node.ms)}</button>{/if}
  </div>
  {#if open && hasBody}
    <div class="body">
      {#if node.points.length}
        <ul>
          {#each node.points as _, i (i)}
            <li>
              {#if editing}<input bind:value={node.points[i]} />{:else}{node.points[i]}{/if}
            </li>
          {/each}
        </ul>
      {/if}
      {#each node.children as child, i (i)}
        <Self node={child} depth={depth + 1} {expanded} path="{path}.{i}" {editing} {onseek} {ontoggle} />
      {/each}
    </div>
  {/if}
</div>

<style>
  .node.depth-0 {
    padding: 10px 12px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .node.depth-0 + :global(.node.depth-0) {
    margin-top: 8px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .twisty {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-3);
    transition: transform var(--dur-med) var(--ease-out);
  }
  .twisty:disabled {
    visibility: hidden;
  }
  .open > .row .twisty {
    transform: rotate(90deg);
  }
  .title {
    flex: 1;
    min-width: 0;
    padding: 2px 4px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text);
    font-weight: 600;
    text-align: left;
  }
  .depth-0 > .row .title {
    font-size: 14px;
    font-weight: 650;
  }
  .title:hover {
    background: var(--surface-hover);
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
  .body {
    margin: 4px 0 2px 28px;
    animation: open var(--dur-med) var(--ease-out);
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-3px);
    }
  }
  ul {
    margin: 2px 0 6px;
    padding-left: 18px;
    color: var(--text-2);
    line-height: 1.6;
    user-select: text;
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
  .title-edit {
    flex: 1;
    font-weight: 600;
  }
</style>
