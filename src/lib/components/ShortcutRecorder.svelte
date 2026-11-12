<script lang="ts">
  import { onDestroy } from 'svelte';
  import { X } from '@lucide/svelte';
  import { commands } from '$lib/ipc/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { eventToAccelerator, formatAccelerator } from '$lib/shortcuts';

  interface Props {
    value: string;
    platform: string;
    label: string;
    onchange: (accel: string) => void;
  }
  let { value, platform, label, onchange }: Props = $props();
  let listening = $state(false);

  function onkeydown(e: KeyboardEvent) {
    e.preventDefault();
    e.stopPropagation();
    const plain = !e.metaKey && !e.ctrlKey && !e.altKey && !e.shiftKey;
    if (plain && e.code === 'Escape') return stop();
    if (plain && (e.code === 'Backspace' || e.code === 'Delete')) {
      stop();
      return onchange('');
    }
    const accel = eventToAccelerator(e, platform);
    if (!accel) return; // still holding modifiers
    stop();
    if (accel !== value) onchange(accel);
  }

  async function start() {
    listening = true;
    await commands.suspendShortcuts();
    window.addEventListener('keydown', onkeydown, true);
  }

  function stop() {
    if (!listening) return;
    listening = false;
    window.removeEventListener('keydown', onkeydown, true);
    void commands.resumeShortcuts();
  }

  onDestroy(stop);
  const chips = $derived(formatAccelerator(value, platform));
</script>

<div class="rec">
  <button class="field" class:listening aria-label={label} onclick={() => (listening ? stop() : start())} onblur={stop}>
    {#if listening}
      <span class="press">{t('shortcuts.press')}</span>
    {:else if chips.length}
      {#each chips as c, i (i)}<kbd>{c}</kbd>{/each}
    {:else}
      <span class="none">{t('shortcuts.none')}</span>
    {/if}
  </button>
  {#if value && !listening}
    <button class="clear" title={t('shortcuts.clear')} onclick={() => onchange('')}><X size={13} /></button>
  {/if}
</div>

<style>
  .rec {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .field {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 3px;
    min-width: 150px;
    height: 30px;
    padding: 0 8px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    transition: box-shadow var(--dur-fast);
  }
  .field:hover {
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .field.listening {
    box-shadow:
      inset 0 0 0 1.5px var(--accent),
      var(--focus-ring);
  }
  kbd {
    min-width: 20px;
    padding: 1px 5px;
    border-radius: 5px;
    background: var(--surface);
    box-shadow:
      inset 0 -1px 0 var(--border-strong),
      0 0 0 1px var(--border);
    font: 600 12px/18px var(--font-mono);
    text-align: center;
  }
  .press {
    color: var(--accent);
    font-size: var(--text-sm);
    font-weight: 600;
    animation: blink 1.2s ease-in-out infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0.45;
    }
  }
  .none {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .clear {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-3);
  }
  .clear:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
</style>
