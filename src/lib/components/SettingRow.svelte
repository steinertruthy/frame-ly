<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    title: string;
    hint?: string;
    children: Snippet;
  }
  let { title, hint, children }: Props = $props();
</script>

<div class="row">
  <div class="text">
    <div class="title">{title}</div>
    {#if hint}<div class="hint">{hint}</div>{/if}
  </div>
  <div class="control">{@render children()}</div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-5);
    padding: 14px var(--space-4);
    min-height: 56px;
  }
  .row + :global(.row) {
    border-top: 1px solid var(--border);
  }
  .text {
    min-width: 0;
  }
  .title {
    font-weight: 550;
  }
  .hint {
    color: var(--text-3);
    font-size: var(--text-sm);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .control {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
  }
</style>
