<script lang="ts">
  import type { Snippet } from 'svelte';
  interface Props {
    title: string;
    children: Snippet;
  }
  let { title, children }: Props = $props();
</script>

<section>
  <h2>{title}</h2>
  <div class="card">{@render children()}</div>
</section>

<style>
  section {
    margin-bottom: var(--space-5);
  }
  h2 {
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    margin: 0 0 var(--space-2) var(--space-1);
  }
  .card {
    background: var(--surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
  }
</style>
