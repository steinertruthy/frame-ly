<script lang="ts" generics="T extends string">
  import type { Component } from 'svelte';
  interface Option {
    value: T;
    label: string;
    icon?: Component<{ size?: number; strokeWidth?: number }>;
  }
  interface Props {
    options: Option[];
    value: T;
    label: string;
    size?: 'sm' | 'md';
    stretch?: boolean;
    onchange: (value: T) => void;
  }
  let { options, value, label, size = 'md', stretch = false, onchange }: Props = $props();

  const index = $derived(Math.max(0, options.findIndex((o) => o.value === value)));

  function onkeydown(e: KeyboardEvent) {
    const dir = e.key === 'ArrowRight' ? 1 : e.key === 'ArrowLeft' ? -1 : 0;
    if (!dir) return;
    e.preventDefault();
    const next = options[(index + dir + options.length) % options.length];
    if (next) onchange(next.value);
  }
</script>

<div
  class="seg {size}"
  class:stretch
  role="radiogroup"
  aria-label={label}
  tabindex="0"
  {onkeydown}
  style="--count:{options.length};--index:{index}"
>
  <span class="thumb" aria-hidden="true"></span>
  {#each options as o (o.value)}
    <button role="radio" aria-checked={o.value === value} tabindex="-1" class:active={o.value === value} onclick={() => onchange(o.value)} title={o.label}>
      {#if o.icon}<o.icon size={14} strokeWidth={2} />{/if}
      <span>{o.label}</span>
    </button>
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: inline-grid;
    grid-template-columns: repeat(var(--count), minmax(0, 1fr));
    padding: 3px;
    border-radius: 9px;
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
    min-width: 220px;
  }
  .seg.stretch {
    display: grid;
    width: 100%;
    min-width: 0;
  }
  .thumb {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 3px;
    width: calc((100% - 6px) / var(--count));
    transform: translateX(calc(100% * var(--index)));
    border-radius: 7px;
    background: var(--surface-raised);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.05),
      0 0 0 1px var(--border-strong),
      0 4px 14px -6px var(--glow);
    transition: transform var(--dur-med) var(--ease-out);
  }
  button {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-width: 0;
    height: 28px;
    padding: 0 10px;
    border: 0;
    background: none;
    color: var(--text-2);
    font-size: var(--text-sm);
    font-weight: 600;
    transition: color var(--dur-fast);
  }
  .sm button {
    height: 24px;
    padding: 0 8px;
    font-size: var(--text-xs);
  }
  button span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  button:hover {
    color: var(--text);
  }
  button.active {
    color: var(--text);
  }
</style>
