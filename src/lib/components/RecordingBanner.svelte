<script lang="ts">
  import { Pause, Play, Square } from '@lucide/svelte';
  import { commands } from '$lib/ipc/commands';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { recording, formatDuration } from '$lib/stores/recording.svelte';
  import { logUi } from '$lib/ipc/log';
  import { Mic, Volume2 } from '@lucide/svelte';
  import LevelMeter from './LevelMeter.svelte';

  const s = $derived(recording.status);
  const label = $derived<MessageKey>(
    ({ countdown: 'record.countdown', recording: 'record.recording', paused: 'record.paused', finalizing: 'record.finalizing', idle: 'record.recording' } as const)[s.phase],
  );

  async function act(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (e) {
      logUi('error', `recording control failed: ${JSON.stringify(e)}`);
    }
  }
</script>

<div class="banner" class:paused={s.phase === 'paused'}>
  <span class="dot" aria-hidden="true"></span>
  <div class="text">
    <div class="state">{t(label)}{s.target ? ` · ${t(`record.target.${s.target}`)}` : ''}</div>
    <div class="time">{formatDuration(s.elapsedMs)}</div>
  </div>
  {#if s.audio}
    <div class="levels">
      {#if s.audio.microphone}<span class="lvl"><Mic size={13} /><LevelMeter level={s.audio.levels.microphone} segments={10} label={t('audio.microphone')} /></span>{/if}
      {#if s.audio.system}<span class="lvl"><Volume2 size={13} /><LevelMeter level={s.audio.levels.system} segments={10} label={t('audio.system')} /></span>{/if}
    </div>
  {/if}
  <div class="controls">
    {#if s.phase === 'recording'}
      <button class="ctl" onclick={() => act(commands.pauseRecording)} title={t('record.pause')}><Pause size={16} /></button>
    {:else if s.phase === 'paused'}
      <button class="ctl" onclick={() => act(commands.resumeRecording)} title={t('record.resume')}><Play size={16} /></button>
    {/if}
    <button class="ctl stop" disabled={s.phase === 'finalizing'} onclick={() => act(commands.stopRecording)}>
      <Square size={14} fill="currentColor" />
      <span>{t('record.stop')}</span>
    </button>
  </div>
</div>

<style>
  .banner {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 14px var(--space-4);
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--rec-500) 10%, var(--surface));
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--rec-500) 30%, transparent),
      var(--shadow-sm);
    animation: rise var(--dur-slow) var(--ease-out);
  }
  .banner.paused {
    background: color-mix(in srgb, var(--warn-500) 10%, var(--surface));
    box-shadow:
      inset 0 0 0 1px color-mix(in srgb, var(--warn-500) 30%, transparent),
      var(--shadow-sm);
  }
  .dot {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--rec-500);
    box-shadow: 0 0 0 0 color-mix(in srgb, var(--rec-500) 60%, transparent);
    animation: pulse 1.6s infinite;
  }
  .paused .dot {
    background: var(--warn-500);
    animation: none;
  }
  .text {
    flex: 1;
  }
  .state {
    font-weight: 600;
  }
  .time {
    font: 600 var(--text-xl) / 1.2 var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
  .levels {
    display: flex;
    flex-direction: column;
    gap: 4px;
    color: var(--text-3);
  }
  .lvl {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .controls {
    display: flex;
    gap: var(--space-2);
  }
  .ctl {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 34px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--surface);
    color: var(--text);
    font-weight: 600;
    box-shadow: var(--shadow-sm);
    transition: transform var(--dur-fast);
  }
  .ctl:active {
    transform: scale(0.96);
  }
  .ctl.stop {
    background: var(--rec-500);
    color: #fff;
  }
  @keyframes pulse {
    70% {
      box-shadow: 0 0 0 10px transparent;
    }
    100% {
      box-shadow: 0 0 0 0 transparent;
    }
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
</style>
