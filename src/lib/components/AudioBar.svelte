<script lang="ts">
  import { Mic, MicOff, Volume2, VolumeX } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';

  const a = $derived(settings.value?.audio);
  const micName = $derived(
    a?.microphoneDevice ? (system.audioInputs.find((d) => d.id === a.microphoneDevice)?.name ?? t('audio.device.default')) : t('audio.device.default'),
  );
</script>

{#if a}
  <div class="bar">
    <button class="chip" class:on={a.microphone} onclick={() => settings.update((d) => (d.audio.microphone = !d.audio.microphone))} aria-pressed={a.microphone}>
      {#if a.microphone}<Mic size={14} />{:else}<MicOff size={14} />{/if}
      <span>{t('audio.microphone')}</span>
      {#if a.microphone}<span class="sub">{micName}</span>{/if}
    </button>
    <button class="chip" class:on={a.systemAudio} onclick={() => settings.update((d) => (d.audio.systemAudio = !d.audio.systemAudio))} aria-pressed={a.systemAudio}>
      {#if a.systemAudio}<Volume2 size={14} />{:else}<VolumeX size={14} />{/if}
      <span>{t('audio.system')}</span>
    </button>
  </div>
{/if}

<style>
  .bar {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }
  .chip {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 12px;
    border: 0;
    border-radius: 999px;
    background: var(--surface);
    color: var(--text-3);
    font-size: var(--text-sm);
    font-weight: 600;
    box-shadow: var(--shadow-sm);
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .chip.on {
    background: var(--accent-soft);
    color: var(--accent);
  }
  .sub {
    max-width: 180px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 500;
    opacity: 0.8;
  }
</style>
