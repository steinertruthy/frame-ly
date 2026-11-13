<script lang="ts">
  /** Recording setup in the tray panel: what to record, sound sources, and "More options". */
  import { slide } from 'svelte/transition';
  import { ChevronDown, Mic, MicOff, Volume2, VolumeX } from '@lucide/svelte';
  import Segmented from './Segmented.svelte';
  import Select from './Select.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import type { Fps, RecordTarget, VideoQuality, VideoResolution } from '$lib/ipc/types';

  let advanced = $state(false);
  const s = $derived(settings.value);

  function toggle(source: 'microphone' | 'systemAudio') {
    void settings.update((d) => (d.audio[source] = !d.audio[source]));
  }
</script>

{#if s}
  <div class="setup">
    <Segmented
      stretch
      label={t('rec.options.area')}
      value={s.recording.target}
      options={[
        { value: 'screen' as RecordTarget, label: t('home.shot.mode.screen') },
        { value: 'window' as RecordTarget, label: t('home.shot.mode.window') },
        { value: 'region' as RecordTarget, label: t('home.shot.mode.region') },
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

    <div class="sources">
      <button class="source" aria-pressed={s.audio.microphone} onclick={() => toggle('microphone')}>
        {#if s.audio.microphone}<Mic size={16} />{:else}<MicOff size={16} />{/if}
        <span>{t('audio.microphone')}</span>
      </button>
      <button class="source" aria-pressed={s.audio.systemAudio} onclick={() => toggle('systemAudio')} title={t('audio.system')}>
        {#if s.audio.systemAudio}<Volume2 size={16} />{:else}<VolumeX size={16} />{/if}
        <span>{t('home.rec.system')}</span>
      </button>
    </div>
    {#if s.audio.microphone}
      <div class="device" transition:slide={{ duration: 180 }}>
        <Select
          label={t('audio.device')}
          value={s.audio.microphoneDevice ?? ''}
          options={[{ value: '', label: t('audio.device.default') }, ...system.audioInputs.map((d) => ({ value: d.id, label: d.name }))]}
          onchange={(v) => settings.update((d) => (d.audio.microphoneDevice = v || null))}
        />
      </div>
    {/if}

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
            options={['24', '30', '60'].map((v) => ({ value: v, label: v }))}
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
    gap: 10px;
  }
  .device :global(.select),
  .device :global(select) {
    width: 100%;
    max-width: none;
  }
  .sources {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .source {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    min-width: 0;
    height: 38px;
    padding: 0 10px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
    font-weight: 600;
    transition:
      background var(--dur-fast),
      color var(--dur-fast),
      box-shadow var(--dur-fast);
  }
  .source span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .source[aria-pressed='true'] {
    background: var(--nav-active);
    box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--primary-border) 70%, transparent);
    color: var(--text);
  }
  .source[aria-pressed='true'] :global(svg) {
    color: var(--icon-accent);
  }
  .source:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .source[aria-pressed='true']:hover {
    background: color-mix(in srgb, var(--nav-active) 80%, var(--surface-hover));
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
