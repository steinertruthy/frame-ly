<script lang="ts" module>
  import type { Component } from 'svelte';
  export interface MenuItem {
    label: string;
    icon?: Component<{ size?: number }>;
    danger?: boolean;
    run: () => void;
  }
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  interface Props {
    x: number;
    y: number;
    items: MenuItem[];
    onclose: () => void;
  }
  let { x, y, items, onclose }: Props = $props();
  let menu: HTMLDivElement | undefined = $state();
  let pos = $state({ x: 0, y: 0 });

  onMount(() => {
    // Keep the menu inside the window.
    const r = menu!.getBoundingClientRect();
    pos = { x: Math.min(x, window.innerWidth - r.width - 8), y: Math.min(y, window.innerHeight - r.height - 8) };
    menu?.querySelector('button')?.focus();
  });
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} onblur={onclose} />

<div class="backdrop" role="presentation" onpointerdown={onclose} oncontextmenu={(e) => (e.preventDefault(), onclose())}></div>
<div class="menu" role="menu" bind:this={menu} style="left:{pos.x}px;top:{pos.y}px">
  {#each items as item (item.label)}
    <button
      role="menuitem"
      class:danger={item.danger}
      onclick={() => {
        onclose();
        item.run();
      }}
    >
      {#if item.icon}<item.icon size={14} />{/if}
      <span>{item.label}</span>
    </button>
  {/each}
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 90;
  }
  .menu {
    position: fixed;
    z-index: 91;
    min-width: 190px;
    padding: 5px;
    border-radius: 10px;
    background: var(--surface);
    box-shadow:
      0 0 0 1px var(--border-strong),
      0 16px 36px -12px rgba(5, 7, 12, 0.5);
    animation: pop 120ms var(--ease-out);
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: scale(0.97);
    }
  }
  button {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    height: 30px;
    padding: 0 10px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text);
    font-size: var(--text-sm);
    text-align: left;
  }
  button :global(svg) {
    color: var(--text-2);
  }
  button:hover,
  button:focus-visible {
    background: var(--surface-hover);
    box-shadow: none;
  }
  button:hover :global(svg),
  button:focus-visible :global(svg) {
    color: var(--icon-accent);
  }
  .danger,
  .danger :global(svg) {
    color: var(--rec-icon);
  }
  .danger:hover,
  .danger:focus-visible {
    background: color-mix(in srgb, var(--rec-500) 14%, transparent);
  }
  .danger:hover :global(svg),
  .danger:focus-visible :global(svg) {
    color: var(--rec-icon);
  }
</style>
