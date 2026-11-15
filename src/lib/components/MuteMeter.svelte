<script lang="ts">
  /**
   * A captured source during a recording: its level meter, and its icon as a mute button.
   * Muting silences that source in the recording from now on; the recording continues.
   */
  import { Mic, MicOff, Volume2, VolumeX } from '@lucide/svelte';
  import LevelMeter from './LevelMeter.svelte';
  import { commands } from '$lib/ipc/commands';
  import { logUi } from '$lib/ipc/log';
  import { t } from '$lib/i18n/index.svelte';

  let {
    source,
    level,
    muted,
    segments = 8,
    size = 13,
    compact = false,
    labeled = false,
  }: {
    source: 'system' | 'microphone';
    level: number;
    muted: boolean;
    segments?: number;
    size?: number;
    /** Smaller button, for the floating pill. */
    compact?: boolean;
    /** Shows the source's name on the button, so it is clear which one gets muted. */
    labeled?: boolean;
  } = $props();

  const name = $derived(t(source === 'microphone' ? 'audio.microphone' : 'audio.system'));
  const action = $derived(t(muted ? 'record.unmute' : 'record.mute', { source: name }));

  function toggle() {
    commands.setRecordingAudioMuted(source, !muted).catch((e) => logUi('warn', `mute failed: ${JSON.stringify(e)}`));
  }
</script>

<span class="source" class:muted class:compact class:labeled>
  <button class="toggle" onclick={toggle} aria-pressed={muted} aria-label={action} title={action}>
    {#if source === 'microphone'}
      {#if muted}<MicOff {size} />{:else}<Mic {size} />{/if}
    {:else if muted}<VolumeX {size} />{:else}<Volume2 {size} />{/if}
    {#if labeled}<span class="name">{name}</span>{/if}
  </button>
  <LevelMeter level={muted ? 0 : level} {segments} label={name} />
</span>

<style>
  .source {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
  .toggle {
    display: grid;
    place-items: center;
    width: 24px;
    height: 24px;
    padding: 0;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: inherit;
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .compact {
    gap: 4px;
  }
  .compact .toggle {
    width: 20px;
    height: 20px;
    border-radius: 5px;
  }
  .toggle:hover {
    background: rgb(255 255 255 / 0.1);
  }
  .muted .toggle {
    background: color-mix(in srgb, var(--rec-500, #d94656) 18%, transparent);
    color: var(--rec-icon, #e26d7a);
  }
  .labeled .toggle {
    display: inline-flex;
    gap: 6px;
    width: auto;
    min-width: 0;
    padding: 0 8px 0 6px;
    font-size: var(--text-xs, 12px);
    white-space: nowrap;
  }
  .labeled .name {
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .muted .name {
    text-decoration: line-through;
  }
  .muted :global(.meter) {
    opacity: 0.35;
  }
</style>
