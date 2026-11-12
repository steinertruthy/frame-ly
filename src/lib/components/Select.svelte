<script lang="ts" generics="T extends string">
  import { ChevronDown } from '@lucide/svelte';
  interface Props {
    value: T;
    options: { value: T; label: string }[];
    label: string;
    disabled?: boolean;
    onchange: (value: T) => void;
  }
  let { value, options, label, disabled = false, onchange }: Props = $props();
</script>

<label class="select" class:disabled>
  <select aria-label={label} {disabled} {value} onchange={(e) => onchange(e.currentTarget.value as T)}>
    {#each options as o (o.value)}
      <option value={o.value}>{o.label}</option>
    {/each}
  </select>
  <ChevronDown size={14} />
</label>

<style>
  .select {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .select.disabled {
    opacity: 0.5;
  }
  select {
    appearance: none;
    height: 30px;
    min-width: 200px;
    max-width: 280px;
    padding: 0 32px 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border-strong);
    color: var(--text);
    font-size: var(--text-sm);
    font-weight: 550;
    text-overflow: ellipsis;
  }
  select:focus-visible {
    box-shadow: var(--focus-ring);
  }
  .select :global(svg) {
    position: absolute;
    right: 10px;
    color: var(--text-3);
    pointer-events: none;
  }
</style>
