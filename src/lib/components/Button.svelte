<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    variant?: 'primary' | 'secondary' | 'ghost';
    size?: 'sm' | 'md';
    busy?: boolean;
    children: Snippet;
  }
  let { variant = 'secondary', size = 'md', busy = false, disabled, children, ...rest }: Props = $props();
</script>

<button class="btn {variant} {size}" class:busy disabled={disabled || busy} aria-busy={busy} {...rest}>
  {@render children()}
</button>

<style>
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    border: 0;
    border-radius: var(--radius-md);
    font-weight: 550;
    white-space: nowrap;
    transition:
      background var(--dur-fast) var(--ease-out),
      transform var(--dur-fast) var(--ease-out),
      box-shadow var(--dur-fast) var(--ease-out),
      opacity var(--dur-fast);
  }
  .md {
    height: 32px;
    padding: 0 14px;
  }
  .sm {
    height: 26px;
    padding: 0 10px;
    font-size: var(--text-sm);
    border-radius: var(--radius-sm);
  }
  .btn:active:not(:disabled) {
    transform: scale(0.97);
  }
  .btn:disabled {
    opacity: 0.5;
  }
  .primary {
    background: var(--accent);
    color: var(--accent-text);
    box-shadow:
      0 1px 0 rgba(255, 255, 255, 0.18) inset,
      0 4px 12px -4px rgba(47, 95, 230, 0.6);
  }
  .primary:hover:not(:disabled) {
    background: var(--accent-hover);
  }
  .secondary {
    background: var(--surface);
    color: var(--text);
    box-shadow: var(--shadow-sm);
  }
  .secondary:hover:not(:disabled) {
    background: var(--surface-hover);
  }
  .ghost {
    background: transparent;
    color: var(--text-2);
  }
  .ghost:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text);
  }
  .busy {
    cursor: progress;
  }
</style>
