<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    title?: string;
    children: Snippet;
  }
  let { title, children }: Props = $props();
</script>

<section>
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
    background: color-mix(in srgb, var(--surface) 86%, transparent);
    border-radius: var(--radius-lg);
    box-shadow:
      inset 0 0 0 1px var(--border),
      inset 0 1px 0 rgba(255, 255, 255, 0.03);
    overflow: hidden;
  }
</style>
