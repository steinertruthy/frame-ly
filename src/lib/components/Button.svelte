<script lang="ts">
  import type { Snippet } from 'svelte';
  import type { HTMLButtonAttributes } from 'svelte/elements';

  interface Props extends HTMLButtonAttributes {
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger';
    size?: 'sm' | 'md' | 'lg';
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
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 7px;
    border: 0;
    border-radius: var(--radius-md);
    font-weight: 600;
    white-space: nowrap;
    transition:
      background var(--dur-fast) var(--ease-out),
      transform var(--dur-fast) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out),
      color var(--dur-fast),
      opacity var(--dur-fast);
  }
  .sm {
    height: 28px;
    padding: 0 11px;
    font-size: var(--text-sm);
    border-radius: 7px;
  }
  .md {
    height: 34px;
    padding: 0 15px;
  }
  .lg {
    height: 42px;
    padding: 0 20px;
    font-size: var(--text-lg);
    border-radius: 10px;
  }
  .btn:active:not(:disabled) {
    transform: translateY(1px) scale(0.985);
  }
  .btn:disabled {
    opacity: 0.45;
  }
  .busy {
    cursor: progress;
  }
  .primary {
    background: var(--primary-fill);
    color: var(--accent-text);
    box-shadow: inset 0 0 0 1px var(--primary-border);
  }
  .primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }
  .secondary {
    background: var(--surface-raised);
    color: var(--text);
    box-shadow:
      inset 0 0 0 1px var(--border),
      inset 0 1px 0 rgba(255, 255, 255, 0.03);
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
  .danger {
    background: var(--rec-500);
    color: #fff;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.2);
  }
  .danger:hover:not(:disabled) {
    background: var(--rec-hover);
  }
</style>
