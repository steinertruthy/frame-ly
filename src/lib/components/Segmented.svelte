<script lang="ts" generics="T extends string">
  interface Option {
    value: T;
    label: string;
  }
  interface Props {
    options: Option[];
    value: T;
    label: string;
    onchange: (value: T) => void;
  }
  let { options, value, label, onchange }: Props = $props();

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
  class="seg"
  role="radiogroup"
  aria-label={label}
  tabindex="0"
  {onkeydown}
  style="--count:{options.length};--index:{index}"
>
  <span class="thumb" aria-hidden="true"></span>
  {#each options as o (o.value)}
    <button
      role="radio"
      aria-checked={o.value === value}
      tabindex="-1"
      class:active={o.value === value}
      onclick={() => onchange(o.value)}>{o.label}</button
    >
  {/each}
</div>

<style>
  .seg {
    position: relative;
    display: grid;
    grid-template-columns: repeat(var(--count), 1fr);
    padding: 3px;
    border-radius: var(--radius-md);
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    min-width: 240px;
  }
  .thumb {
    position: absolute;
    top: 3px;
    bottom: 3px;
    left: 3px;
    width: calc((100% - 6px) / var(--count));
    transform: translateX(calc(100% * var(--index)));
    border-radius: 7px;
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    transition: transform var(--dur-med) var(--ease-out);
  }
  button {
    position: relative;
    height: 26px;
    border: 0;
    background: none;
    color: var(--text-2);
    font-size: var(--text-sm);
    font-weight: 550;
    transition: color var(--dur-fast);
  }
  button.active {
    color: var(--text);
  }
</style>
