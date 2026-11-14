<script lang="ts">
  /**
   * Floating recording controls: elapsed time, sound levels, pause/resume and stop.
   * Lives on the recorded display, can be dragged anywhere, and is never part of the video.
   */
  import { Pause, Play, Square, EyeOff } from '@lucide/svelte';
  import MuteMeter from '$lib/components/MuteMeter.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { commands } from '$lib/ipc/commands';
  import { logUi } from '$lib/ipc/log';
  import { recording, formatDuration } from '$lib/stores/recording.svelte';

  const rec = $derived(recording.status);
  const paused = $derived(rec.phase === 'paused');

  async function control(action: () => Promise<void>) {
    try {
      await action();
    } catch (e) {
      logUi('warn', `recording control failed: ${JSON.stringify(e)}`);
    }
  }
</script>

<div class="bar" class:paused data-tauri-drag-region role="toolbar" aria-label={t('record.recording')}>
  <span class="dot" data-tauri-drag-region></span>
  <span class="clock" data-tauri-drag-region>{formatDuration(rec.elapsedMs)}</span>

  {#if rec.audio && (rec.audio.microphone || rec.audio.system)}
    <span class="sep" data-tauri-drag-region></span>
    <div class="meters" data-tauri-drag-region>
      {#if rec.audio.microphone}
        <MuteMeter source="microphone" level={rec.audio.levels.microphone} muted={rec.audio.microphoneMuted} />
      {/if}
      {#if rec.audio.system}
        <MuteMeter source="system" level={rec.audio.levels.system} muted={rec.audio.systemMuted} />
      {/if}
    </div>
  {/if}

  <span class="sep" data-tauri-drag-region></span>
  {#if paused}
    <button class="btn" title={t('record.resume')} aria-label={t('record.resume')} onclick={() => control(commands.resumeRecording)}>
      <Play size={15} />
    </button>
  {:else}
    <button
      class="btn"
      title={t('record.pause')}
      aria-label={t('record.pause')}
      disabled={rec.phase !== 'recording'}
      onclick={() => control(commands.pauseRecording)}
    >
      <Pause size={15} />
    </button>
  {/if}
  <button
    class="btn stop"
    title={t('record.stop')}
    aria-label={t('record.stop')}
    disabled={rec.phase === 'finalizing'}
    onclick={() => control(commands.stopRecording)}
  >
    <Square size={12} fill="currentColor" />
  </button>
  <button
    class="btn ghost"
    title={t('record.hideControls.hint')}
    aria-label={t('record.hideControls')}
    onclick={() => control(() => commands.setRecordingControlsVisible(false))}
  >
    <EyeOff size={14} />
  </button>
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    overflow: hidden;
  }
  :global(#app) {
    height: 100vh;
    display: grid;
    place-items: center;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 44px;
    padding: 0 6px 0 16px;
    border-radius: 22px;
    background: rgb(12 20 35 / 0.92);
    border: 1px solid rgb(255 255 255 / 0.08);
    /* Fits inside the window's transparent margin (a clipped shadow shows as a rectangle). */
    box-shadow:
      0 6px 14px -4px rgb(0 0 0 / 0.55),
      0 0 0 0.5px rgb(0 0 0 / 0.6);
    color: #edf0f7;
    cursor: grab;
    user-select: none;
    -webkit-user-select: none;
  }
  .dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: var(--rec-500, #d94656);
    box-shadow: 0 0 0 4px rgb(255 84 104 / 0.18);
    animation: pulse 1.6s ease-in-out infinite;
  }
  .paused .dot {
    background: #f5b14c;
    box-shadow: 0 0 0 4px rgb(245 177 76 / 0.18);
    animation: none;
  }
  @keyframes pulse {
    50% {
      opacity: 0.45;
    }
  }
  .clock {
    font: 600 15px/1 var(--font-mono, ui-monospace, monospace);
    font-variant-numeric: tabular-nums;
    letter-spacing: 0.02em;
    min-width: 4.6ch;
  }
  .paused .clock {
    color: #8b95a8;
  }
  .sep {
    width: 1px;
    height: 20px;
    background: rgb(255 255 255 / 0.1);
  }
  .meters {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: #8b95a8;
  }
  .btn {
    display: grid;
    place-items: center;
    width: 32px;
    height: 32px;
    border: 0;
    border-radius: 50%;
    background: rgb(255 255 255 / 0.06);
    color: #edf0f7;
    cursor: pointer;
    transition: background 0.15s;
  }
  .btn:hover:not(:disabled) {
    background: rgb(255 255 255 / 0.14);
  }
  .btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .btn.ghost {
    width: 28px;
    background: transparent;
    color: rgb(237 240 247 / 0.55);
  }
  .btn.ghost:hover {
    background: rgb(255 255 255 / 0.08);
    color: #edf0f7;
  }
  .btn.stop {
    background: var(--rec-500, #d94656);
    color: #fff;
  }
  .btn.stop:hover:not(:disabled) {
    background: #e25565;
  }
  .btn:focus-visible {
    outline: 2px solid #4c7bd9;
    outline-offset: 2px;
  }
  @media (prefers-reduced-motion: reduce) {
    .dot {
      animation: none;
    }
  }
</style>
