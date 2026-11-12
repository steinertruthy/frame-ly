<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    title: string;
    hint?: string;
    /** Stack the control under the text (wide controls on narrow windows). */
    stacked?: boolean;
    children: Snippet;
  }
  let { title, hint, stacked = false, children }: Props = $props();
</script>

<div class="row" class:stacked>
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
    padding: 14px 18px;
    min-height: 60px;
  }
  .row + :global(.row) {
    box-shadow: inset 0 1px 0 var(--border);
  }
  .stacked {
    flex-direction: column;
    align-items: stretch;
    gap: var(--space-3);
  }
  .text {
    min-width: 0;
  }
  .title {
    font-weight: 600;
  }
  .hint {
    color: var(--text-2);
    font-size: var(--text-sm);
    margin-top: 2px;
    line-height: 1.4;
  }
  .control {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex-shrink: 0;
    flex-wrap: wrap;
    justify-content: flex-end;
  }
  .stacked .control {
    justify-content: flex-start;
  }
  @media (max-width: 860px) {
    .row {
      flex-wrap: wrap;
    }
    .control {
      justify-content: flex-start;
    }
  }
</style>
