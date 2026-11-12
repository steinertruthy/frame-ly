<script lang="ts">
  /**
   * Recording options in a popover anchored under its trigger (Home's "Customize").
   * Closes on Escape, on a click outside, or with its own Close button; focus returns to the trigger.
   */
  import { onMount } from 'svelte';
  import Segmented from './Segmented.svelte';
  import Select from './Select.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import type { Fps, RecordTarget, VideoQuality, VideoResolution } from '$lib/ipc/types';

  let { anchor, onclose }: { anchor: HTMLElement | undefined; onclose: () => void } = $props();
  let root: HTMLDivElement | undefined = $state();
  /** Shift left when the popover would spill past the window's right edge. */
  let shift = $state(0);
  const s = $derived(settings.value);

  function close() {
    onclose();
    anchor?.focus();
  }

  onMount(() => {
    if (root) shift = Math.min(0, window.innerWidth - 16 - root.getBoundingClientRect().right);
    root?.focus();
    const down = (e: PointerEvent) => {
      const target = e.target as Node;
      if (root?.contains(target) || anchor?.contains(target)) return;
      onclose();
    };
    window.addEventListener('pointerdown', down, true);
    return () => window.removeEventListener('pointerdown', down, true);
  });
</script>

<svelte:window
  onkeydown={(e) => {
    if (e.key === 'Escape') {
      e.stopPropagation();
      close();
    }
  }}
/>

{#if s}
  <div class="pop" bind:this={root} style:translate="{shift}px 0" role="dialog" aria-label={t('rec.options.title')} tabindex="-1">
    <header>
      <h3>{t('rec.options.title')}</h3>
      <button class="close" onclick={close}>{t('rec.options.close')}</button>
    </header>

    <div class="field">
      <span class="label">{t('rec.options.area')}</span>
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
        <Select
          label={t('panel.monitor')}
          value={recordPrefs.displayId}
          options={[{ value: '', label: t('panel.monitor.auto') }, ...system.displays.map((d) => ({ value: d.id, label: d.name }))]}
          onchange={(v) => (recordPrefs.displayId = v)}
        />
      {/if}
    </div>

    {#if s.audio.microphone}
      <div class="field">
        <span class="label">{t('audio.microphone')}</span>
        <Select
          label={t('audio.device')}
          value={s.audio.microphoneDevice ?? ''}
          options={[{ value: '', label: t('audio.device.default') }, ...system.audioInputs.map((d) => ({ value: d.id, label: d.name }))]}
          onchange={(v) => settings.update((d) => (d.audio.microphoneDevice = v || null))}
        />
      </div>
    {/if}

    <div class="pair">
      <div class="field">
        <span class="label">{t('rec.options.fps')}</span>
        <Segmented
          stretch
          label={t('rec.options.fps')}
          value={String(s.recording.fps)}
          options={['24', '30', '60'].map((v) => ({ value: v, label: v }))}
          onchange={(v) => settings.update((d) => (d.recording.fps = Number(v) as Fps))}
        />
      </div>
      <div class="field">
        <span class="label">{t('rec.countdown')}</span>
        <Segmented
          stretch
          label={t('rec.countdown')}
          value={String(s.recording.countdown)}
          options={['0', '3', '5'].map((v) => ({ value: v, label: `${v}s` }))}
          onchange={(v) => settings.update((d) => (d.recording.countdown = Number(v)))}
        />
      </div>
    </div>

    <div class="field">
      <span class="label">{t('rec.quality')}</span>
      <Segmented
        stretch
        label={t('rec.quality')}
        value={s.recording.quality}
        options={(['medium', 'high', 'max'] as VideoQuality[]).map((q) => ({ value: q, label: t(`settings.recording.quality.${q}`) }))}
        onchange={(v) => settings.update((d) => (d.recording.quality = v))}
      />
    </div>

    <div class="field">
      <span class="label">{t('rec.resolution')}</span>
      <Segmented
        stretch
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
  </div>
{/if}

<style>
  .pop {
    position: absolute;
    top: calc(100% + 8px);
    left: 0;
    z-index: 20;
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 320px;
    max-width: calc(100vw - 32px);
    padding: 14px 16px 16px;
    border-radius: 14px;
    background: var(--surface);
    box-shadow:
      inset 0 0 0 1px var(--border-strong),
      0 24px 50px -18px rgba(0, 0, 0, 0.7);
    cursor: default;
    transform-origin: top left;
    animation: pop var(--dur-med) var(--ease-out);
  }
  .pop:focus {
    outline: none;
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.98);
    }
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-bottom: 12px;
    box-shadow: inset 0 -1px 0 var(--border);
  }
  h3 {
    font-size: var(--text-md);
    font-weight: 650;
  }
  .close {
    padding: 2px 4px;
    border: 0;
    border-radius: 6px;
    background: none;
    color: var(--text-3);
    font-size: var(--text-sm);
    font-weight: 550;
  }
  .close:hover {
    color: var(--text);
  }
  .field {
    display: flex;
    flex-direction: column;
    gap: 7px;
    min-width: 0;
  }
  .label {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .pair {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }
  .field :global(.select),
  .field :global(select) {
    width: 100%;
    min-width: 0;
    max-width: none;
  }
</style>
