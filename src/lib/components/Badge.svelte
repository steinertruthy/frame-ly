<script lang="ts">
  import type { Snippet } from 'svelte';
  let { tone = 'neutral', children }: { tone?: 'ok' | 'warn' | 'error' | 'neutral' | 'accent'; children: Snippet } = $props();
</script>

<span class="badge {tone}">{@render children()}</span>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    height: 22px;
    padding: 0 9px;
    border-radius: 999px;
    font-size: var(--text-xs);
    font-weight: 600;
    white-space: nowrap;
  }
  .badge::before {
    content: '';
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
  }
  .ok {
    color: var(--ok-500);
    background: color-mix(in srgb, var(--ok-500) 14%, transparent);
  }
  .warn {
    color: var(--warn-500);
    background: color-mix(in srgb, var(--warn-500) 14%, transparent);
  }
  .error {
    color: var(--rec-500);
    background: color-mix(in srgb, var(--rec-500) 14%, transparent);
  }
  .accent {
    color: var(--accent);
    background: var(--accent-soft);
  }
  .neutral {
    color: var(--text-2);
    background: var(--surface-hover);
  }
</style>
