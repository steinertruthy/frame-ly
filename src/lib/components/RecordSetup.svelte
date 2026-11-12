<script lang="ts">
  import { slide } from 'svelte/transition';
  import { Monitor, AppWindow, SquareDashed, Mic, Volume2, AudioLines, VolumeX, ChevronDown } from '@lucide/svelte';
  import Segmented from './Segmented.svelte';
  import Select from './Select.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { applyAudioMode, audioMode, type AudioMode } from '$lib/recordSetup';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import type { Fps, RecordTarget, VideoQuality, VideoResolution } from '$lib/ipc/types';

  /** `compact` = quick panel width. */
  let { compact = false }: { compact?: boolean } = $props();
  let advanced = $state(false);

  const s = $derived(settings.value);
  const mode = $derived(s ? audioMode(s.audio) : 'none');
  const usesMic = $derived(mode === 'both' || mode === 'mic');
</script>

{#if s}
  <div class="setup" class:compact>
    <div class="field">
      <span class="label">{t('rec.what')}</span>
      <Segmented
        stretch
        size={compact ? 'sm' : 'md'}
        label={t('rec.what')}
        value={s.recording.target}
        options={[
          { value: 'screen' as RecordTarget, label: t('rec.target.screen'), icon: Monitor },
          { value: 'window' as RecordTarget, label: t('rec.target.window'), icon: AppWindow },
          { value: 'region' as RecordTarget, label: t('rec.target.region'), icon: SquareDashed },
        ]}
        onchange={(v) => settings.update((d) => (d.recording.target = v))}
      />
      {#if s.recording.target === 'screen' && system.displays.length > 1}
        <div class="device" transition:slide={{ duration: 180 }}>
          <Select
            label={t('panel.monitor')}
            value={recordPrefs.displayId}
            options={[{ value: '', label: t('panel.monitor.auto') }, ...system.displays.map((d) => ({ value: d.id, label: d.name }))]}
            onchange={(v) => (recordPrefs.displayId = v)}
          />
        </div>
      {/if}
    </div>

    <div class="field">
      <span class="label">{t('rec.sound')}</span>
      <Segmented
        stretch
        size={compact ? 'sm' : 'md'}
        label={t('rec.sound')}
        value={mode}
        options={[
          { value: 'both' as AudioMode, label: t('rec.audio.short.both'), icon: AudioLines },
          { value: 'mic' as AudioMode, label: t('rec.audio.short.mic'), icon: Mic },
          { value: 'system' as AudioMode, label: t('rec.audio.short.system'), icon: Volume2 },
          { value: 'none' as AudioMode, label: t('rec.audio.short.none'), icon: VolumeX },
        ]}
        onchange={(v) => settings.update((d) => applyAudioMode(d.audio, v))}
      />
      {#if usesMic}
        <div class="device" transition:slide={{ duration: 180 }}>
          <Select
            label={t('audio.device')}
            value={s.audio.microphoneDevice ?? ''}
            options={[{ value: '', label: t('audio.device.default') }, ...system.audioInputs.map((d) => ({ value: d.id, label: d.name }))]}
            onchange={(v) => settings.update((d) => (d.audio.microphoneDevice = v || null))}
          />
        </div>
      {/if}
    </div>

    <button class="more" onclick={() => (advanced = !advanced)} aria-expanded={advanced}>
      <ChevronDown size={14} />
      <span>{t('rec.more')}</span>
    </button>

    {#if advanced}
      <div class="advanced" transition:slide={{ duration: 220 }}>
        <div class="row">
          <span>{t('rec.smoothness')}</span>
          <Segmented
            size="sm"
            label={t('rec.smoothness')}
            value={String(s.recording.fps)}
            options={[
              { value: '24', label: '24' },
              { value: '30', label: '30' },
              { value: '60', label: '60' },
            ]}
            onchange={(v) => settings.update((d) => (d.recording.fps = Number(v) as Fps))}
          />
        </div>
        <div class="row">
          <span>{t('rec.quality')}</span>
          <Segmented
            size="sm"
            label={t('rec.quality')}
            value={s.recording.quality}
            options={(['medium', 'high', 'max'] as VideoQuality[]).map((q) => ({ value: q, label: t(`settings.recording.quality.${q}`) }))}
            onchange={(v) => settings.update((d) => (d.recording.quality = v))}
          />
        </div>
        <div class="row">
          <span>{t('rec.resolution')}</span>
          <Segmented
            size="sm"
            label={t('rec.resolution')}
            value={s.recording.resolution}
            options={[
              { value: 'native' as VideoResolution, label: t('settings.recording.resolution.native') },
              { value: '1080p' as VideoResolution, label: '1080p' },
              { value: '720p' as VideoResolution, label: '720p' },
            ]}
            onchange={(v) => settings.update((d) => (d.recording.resolution = v))}
          />
        </div>
        <div class="row">
          <span>{t('rec.countdown')}</span>
          <Segmented
            size="sm"
            label={t('rec.countdown')}
            value={String(s.recording.countdown)}
            options={[
              { value: '0', label: t('settings.recording.countdown.off') },
              { value: '3', label: '3 s' },
              { value: '5', label: '5 s' },
            ]}
            onchange={(v) => settings.update((d) => (d.recording.countdown = Number(v)))}
          />
        </div>
      </div>
    {/if}
  </div>
{/if}

<style>
  .setup {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .compact {
    gap: 10px;
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
  }
  .label {
    font-size: var(--text-xs);
    font-weight: 650;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .device :global(.select),
  .device :global(select) {
    width: 100%;
    max-width: none;
  }
  .more {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    align-self: flex-start;
    padding: 2px 0;
    border: 0;
    background: none;
    color: var(--text-2);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .more:hover {
    color: var(--text);
  }
  .more :global(svg) {
    transition: transform var(--dur-med) var(--ease-out);
  }
  .more[aria-expanded='true'] :global(svg) {
    transform: rotate(180deg);
  }
  .advanced {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .row :global(.seg) {
    min-width: 0;
    flex-shrink: 0;
  }
</style>
