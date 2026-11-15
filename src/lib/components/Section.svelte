<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    title?: string;
    /** Target for `framely --show settings:<category>:<anchor>`. */
    anchor?: string;
    children: Snippet;
  }
  let { title, anchor, children }: Props = $props();
</script>

<section data-anchor={anchor}>
  {#if title}<h3 class="eyebrow">{title}</h3>{/if}
  <div class="card">{@render children()}</div>
</section>

<style>
  section {
    margin-bottom: var(--space-5);
  }
  h3 {
    margin: 0 0 var(--space-2) 4px;
  }
  .card {
    background: var(--surface);
    border-radius: 14px;
    box-shadow: inset 0 0 0 1px var(--border);
    overflow: hidden;
  }
</style>
