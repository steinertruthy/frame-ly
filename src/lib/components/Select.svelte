<script lang="ts" generics="T extends string">
  import type { Component } from 'svelte';
  import { ChevronDown } from '@lucide/svelte';
  interface Props {
    value: T;
    options: { value: T; label: string }[];
    label: string;
    disabled?: boolean;
    /** Leading icon; with `prefix`, the field reads "[icon] Microphone  MacBook Mic ▾". */
    icon?: Component<{ size?: number; strokeWidth?: number }>;
    /** What the field controls, shown before the current choice. */
    prefix?: string;
    onchange: (value: T) => void;
  }
  let { value, options, label, disabled = false, icon: Icon, prefix, onchange }: Props = $props();
  const rich = $derived(!!Icon || !!prefix);
  const current = $derived(options.find((o) => o.value === value)?.label ?? '');
</script>

{#if rich}
  <!-- The native select covers the whole field (invisible), so a click anywhere opens the system list. -->
  <label class="field" class:disabled class:plain={!prefix} title={prefix ? `${prefix}: ${current}` : current}>
    {#if Icon}<span class="icon"><Icon size={15} strokeWidth={1.9} /></span>{/if}
    {#if prefix}<span class="prefix">{prefix}</span>{/if}
    <span class="value">{current}</span>
    <span class="chev"><ChevronDown size={14} /></span>
    <select aria-label={label} {disabled} {value} onchange={(e) => onchange(e.currentTarget.value as T)}>
      {#each options as o (o.value)}
        <option value={o.value}>{o.label}</option>
      {/each}
    </select>
  </label>
{:else}
  <label class="select" class:disabled>
    <select aria-label={label} {disabled} {value} onchange={(e) => onchange(e.currentTarget.value as T)}>
      {#each options as o (o.value)}
        <option value={o.value}>{o.label}</option>
      {/each}
    </select>
    <ChevronDown size={14} />
  </label>
{/if}

<style>
  .select {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .select.disabled,
  .field.disabled {
    opacity: 0.5;
  }
  .select select {
    appearance: none;
    height: 32px;
    min-width: 200px;
    max-width: 280px;
    padding: 0 32px 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text);
    transition: box-shadow var(--dur-fast);
    font-size: var(--text-sm);
    font-weight: 550;
    text-overflow: ellipsis;
  }
  .select select:hover:not(:disabled) {
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .select select:focus-visible {
    box-shadow: var(--focus-ring);
  }
  .select :global(svg) {
    position: absolute;
    right: 10px;
    color: var(--text-3);
    pointer-events: none;
  }

  /* Rich field: icon + what it controls + current choice */
  .field {
    position: relative;
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    height: 32px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    font-size: var(--text-sm);
    transition: box-shadow var(--dur-fast);
  }
  .field:hover:not(.disabled) {
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .field:has(select:focus-visible) {
    box-shadow: var(--focus-ring);
  }
  .icon {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    color: var(--icon-accent);
  }
  .prefix {
    flex-shrink: 0;
    color: var(--text-3);
  }
  .value {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    color: var(--text);
    font-weight: 550;
    text-align: right;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  /* Without a prefix the choice reads left to right next to its icon. */
  .plain .value {
    flex: 0 1 auto;
    text-align: left;
  }
  .plain .chev {
    margin-left: auto;
  }
  .chev {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    color: var(--text-3);
  }
  .field select {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    opacity: 0;
    appearance: none;
    border: 0;
    font: inherit;
  }
</style>
