<script lang="ts">
  import { onMount, untrack } from 'svelte';
  import { Settings, Maximize2, SquareDashed, AppWindow, MonitorPlay, Crop, Monitor, Mic, MicOff, Volume2, VolumeX, Pause, Play, Square } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import LogoMark from '$lib/components/LogoMark.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Select from '$lib/components/Select.svelte';
  import LevelMeter from '$lib/components/LevelMeter.svelte';
  import { commands, events, isAppError } from '$lib/ipc/commands';
  import { logUi } from '$lib/ipc/log';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { AppInfo, SelectMode } from '$lib/ipc/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { recording, formatDuration } from '$lib/stores/recording.svelte';
  import { formatAccelerator, type ShortcutAction } from '$lib/shortcuts';

  let { info }: { info: AppInfo } = $props();

  type Tab = 'record' | 'capture';
  // Initial tab only; the user switches freely afterwards.
  let tab = $state<Tab>(untrack(() => (info.supportsRecording ? 'record' : 'capture')));
  let display = $state(''); // '' = display under the pointer
  let error = $state<string | null>(null);
  let root: HTMLDivElement | undefined = $state();

  const s = $derived(settings.value);
  const rec = $derived(recording.status);
  const live = $derived(rec.phase !== 'idle');

  onMount(() => {
    const un = events.onPanelShown(() => {
      error = null;
      // Focus the panel itself, not its first button (no stray focus ring on open). Deferred:
      // the event arrives just before the window becomes key.
      setTimeout(() => {
        (document.activeElement as HTMLElement | null)?.blur();
        root?.focus();
      }, 60);
      void system.refresh();
    });
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && void commands.panelHide();
    window.addEventListener('keydown', onKey);
    return () => {
      void un.then((u) => u());
      window.removeEventListener('keydown', onKey);
    };
  });

  interface Tile {
    id: SelectMode | 'screen';
    icon: Component<{ size?: number; strokeWidth?: number }>;
    label: MessageKey;
    shortcut: ShortcutAction;
  }
  const tiles = $derived<Tile[]>(
    tab === 'record'
      ? [
          { id: 'region', icon: SquareDashed, label: 'record.region', shortcut: 'recordRegion' },
          { id: 'window', icon: AppWindow, label: 'record.window', shortcut: 'recordWindow' },
          { id: 'screen', icon: MonitorPlay, label: 'record.screen', shortcut: 'recordScreen' },
        ]
      : [
          { id: 'region', icon: Crop, label: 'capture.region', shortcut: 'captureRegion' },
          { id: 'window', icon: AppWindow, label: 'capture.window', shortcut: 'captureWindow' },
          { id: 'screen', icon: Monitor, label: 'capture.screen', shortcut: 'captureScreen' },
        ],
  );

  async function start(id: Tile['id']) {
    error = null;
    // Hide first: the panel must not sit on top of the selection overlay or countdown.
    await commands.panelHide();
    try {
      if (tab === 'record') {
        await commands.startRecording(id === 'screen' ? { kind: 'display', displayId: display || null } : { kind: 'select', mode: id });
      } else {
        await commands.takeScreenshot(id === 'screen' ? { kind: 'display', displayId: display || null } : { kind: 'select', mode: id });
      }
    } catch (e) {
      error = t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic');
      logUi('warn', `panel action failed: ${JSON.stringify(e)}`);
    }
  }

  async function control(fn: () => Promise<unknown>) {
    try {
      await fn();
    } catch (e) {
      logUi('warn', `panel control failed: ${JSON.stringify(e)}`);
    }
  }

  const phaseLabel = $derived<MessageKey>(
    ({ countdown: 'record.countdown', recording: 'record.recording', paused: 'record.paused', finalizing: 'record.finalizing', idle: 'record.recording' } as const)[
      rec.phase
    ],
  );
  const keys = (a: ShortcutAction) => formatAccelerator(s?.shortcuts[a] ?? '', info.platform);
</script>

<div class="panel" class:mac={info.platform === 'macos'} bind:this={root} tabindex="-1" role="dialog" aria-label="FrameLy">
  <header>
    <LogoMark size={22} />
    <span class="brand">FrameLy</span>
    <button class="icon" title={t('panel.settings')} onclick={() => commands.showMainWindow('settings')}><Settings size={15} /></button>
    <button class="icon" title={t('panel.open')} onclick={() => commands.showMainWindow('home')}><Maximize2 size={15} /></button>
  </header>

  {#if live}
    <section class="live" class:paused={rec.phase === 'paused'}>
      <div class="state">
        <span class="dot"></span>
        <span>{t(phaseLabel)}{rec.target ? ` · ${t(`record.target.${rec.target}`)}` : ''}</span>
      </div>
      <div class="clock">{formatDuration(rec.elapsedMs)}</div>
      {#if rec.audio}
        <div class="meters">
          {#if rec.audio.microphone}<span><Mic size={13} /><LevelMeter level={rec.audio.levels.microphone} segments={16} /></span>{/if}
          {#if rec.audio.system}<span><Volume2 size={13} /><LevelMeter level={rec.audio.levels.system} segments={16} /></span>{/if}
        </div>
      {/if}
      <div class="controls">
        {#if rec.phase === 'recording'}
          <button class="ctl" onclick={() => control(commands.pauseRecording)}><Pause size={16} />{t('record.pause')}</button>
        {:else if rec.phase === 'paused'}
          <button class="ctl" onclick={() => control(commands.resumeRecording)}><Play size={16} />{t('record.resume')}</button>
        {/if}
        <button class="ctl stop" disabled={rec.phase === 'finalizing'} onclick={() => control(commands.stopRecording)}>
          <Square size={14} fill="currentColor" />{t('record.stop')}
        </button>
      </div>
      <div class="hint">
        {#each [['stopRecording', 'record.stop'], ['togglePause', 'shortcuts.action.togglePause']] as const as [a, label] (a)}
          {#if keys(a).length}<span>{t(label)} <kbd>{keys(a).join('')}</kbd></span>{/if}
        {/each}
      </div>
    </section>
  {:else}
    {#if info.supportsRecording}
      <Segmented
        label={t('panel.record')}
        value={tab}
        options={[
          { value: 'record' as Tab, label: t('panel.record') },
          { value: 'capture' as Tab, label: t('panel.capture') },
        ]}
        onchange={(v) => (tab = v)}
      />
    {/if}

    {#if !system.screenReady}
      <div class="notice">
        <span>{t('panel.needsPermission')}</span>
        <button class="link" onclick={() => commands.showMainWindow('home')}>{t('panel.setup')}</button>
      </div>
    {/if}

    <div class="tiles" class:rec={tab === 'record'}>
      {#each tiles as tile (tab + tile.id)}
        <button class="tile" disabled={!system.screenReady} onclick={() => start(tile.id)}>
          <span class="glyph"><tile.icon size={20} strokeWidth={1.8} /></span>
          <span class="label">{t(tile.label)}</span>
          <span class="keys">{keys(tile.shortcut).join('')}</span>
        </button>
      {/each}
    </div>

    {#if system.displays.length > 1}
      <div class="row">
        <span class="row-label"><Monitor size={14} />{t('panel.monitor')}</span>
        <Select
          label={t('panel.monitor')}
          value={display}
          options={[{ value: '', label: t('panel.monitor.auto') }, ...system.displays.map((d) => ({ value: d.id, label: d.name }))]}
          onchange={(v) => (display = v)}
        />
      </div>
    {/if}

    {#if tab === 'record' && s}
      <div class="audio">
        <button
          class="toggle-row"
          class:on={s.audio.microphone}
          aria-pressed={s.audio.microphone}
          onclick={() => settings.update((d) => (d.audio.microphone = !d.audio.microphone))}
        >
          {#if s.audio.microphone}<Mic size={15} />{:else}<MicOff size={15} />{/if}
          <span>{t('audio.microphone')}</span>
          <span class="switch"></span>
        </button>
        {#if s.audio.microphone}
          <div class="device">
            <Select
              label={t('audio.device')}
              value={s.audio.microphoneDevice ?? ''}
              options={[{ value: '', label: t('audio.device.default') }, ...system.audioInputs.map((d) => ({ value: d.id, label: d.name }))]}
              onchange={(v) => settings.update((d) => (d.audio.microphoneDevice = v || null))}
            />
          </div>
        {/if}
        <button
          class="toggle-row"
          class:on={s.audio.systemAudio}
          aria-pressed={s.audio.systemAudio}
          onclick={() => settings.update((d) => (d.audio.systemAudio = !d.audio.systemAudio))}
        >
          {#if s.audio.systemAudio}<Volume2 size={15} />{:else}<VolumeX size={15} />{/if}
          <span>{t('audio.system')}</span>
          <span class="switch"></span>
        </button>
      </div>
    {/if}

    {#if error}<div class="error">{error}</div>{/if}

    <footer>
      {#if keys('openPanel').length}
        <span>{t('shortcuts.action.openPanel')} <kbd>{keys('openPanel').join(info.platform === 'macos' ? '' : '+')}</kbd></span>
      {/if}
    </footer>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    overflow: hidden;
  }
  .panel:focus {
    outline: none;
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .panel {
    position: fixed;
    inset: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px 14px 12px;
    border-radius: 14px;
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border-strong);
    overflow: hidden;
    animation: open 180ms var(--ease-out);
  }
  /* macOS: the native popover material shows through. */
  .panel.mac {
    background: color-mix(in srgb, var(--surface) 62%, transparent);
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.985);
    }
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .brand {
    flex: 1;
    font-family: var(--font-display);
    font-weight: 700;
    font-size: var(--text-lg);
    letter-spacing: -0.02em;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-2);
    transition: background var(--dur-fast);
  }
  .icon:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .tiles {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 14px 6px 10px;
    border: 0;
    border-radius: 12px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    transition:
      transform var(--dur-med) var(--ease-out),
      background var(--dur-fast);
  }
  .tile:hover:not(:disabled) {
    background: var(--surface-hover);
    transform: translateY(-1px);
  }
  .tile:active:not(:disabled) {
    transform: scale(0.97);
  }
  .tile:disabled {
    opacity: 0.45;
  }
  .glyph {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 12px;
    background: var(--hero);
    color: #fff;
    box-shadow: 0 6px 14px -6px rgba(22, 48, 122, 0.7);
  }
  .rec .glyph {
    background: linear-gradient(135deg, #ff6b78, #e5394b);
    box-shadow: 0 6px 14px -6px rgba(229, 57, 75, 0.65);
  }
  .label {
    font-weight: 600;
    font-size: var(--text-sm);
  }
  .keys {
    min-height: 14px;
    color: var(--text-3);
    font: 500 11px var(--font-mono);
    letter-spacing: 0.04em;
  }
  .row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }
  .row-label {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--text-2);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .row :global(select) {
    min-width: 0;
    width: 190px;
  }
  .audio {
    display: flex;
    flex-direction: column;
    border-radius: 12px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    overflow: hidden;
  }
  .toggle-row {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 40px;
    padding: 0 12px;
    border: 0;
    background: transparent;
    color: var(--text-2);
    font-weight: 600;
    text-align: left;
  }
  .toggle-row + .toggle-row,
  .device + .toggle-row {
    border-top: 1px solid var(--border);
  }
  .toggle-row span:nth-child(2) {
    flex: 1;
  }
  .toggle-row.on {
    color: var(--text);
  }
  .switch {
    position: relative;
    width: 30px;
    height: 18px;
    border-radius: 999px;
    background: var(--border-strong);
    transition: background var(--dur-med);
  }
  .switch::after {
    content: '';
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 2px rgba(6, 13, 38, 0.3);
    transition: transform var(--dur-med) var(--ease-spring);
  }
  .on .switch {
    background: var(--accent);
  }
  .on .switch::after {
    transform: translateX(12px);
  }
  .device {
    padding: 0 12px 10px 37px;
  }
  .device :global(select) {
    width: 100%;
    min-width: 0;
    max-width: none;
  }
  .device :global(.select) {
    display: flex;
  }
  .notice,
  .error {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    border-radius: 10px;
    font-size: var(--text-sm);
  }
  .notice {
    background: color-mix(in srgb, var(--warn-500) 14%, transparent);
  }
  .notice span {
    flex: 1;
  }
  .error {
    background: color-mix(in srgb, var(--rec-500) 14%, transparent);
  }
  .link {
    border: 0;
    background: none;
    color: var(--accent);
    font-weight: 650;
  }
  footer {
    margin-top: auto;
    display: flex;
    justify-content: center;
    color: var(--text-3);
    font-size: var(--text-xs);
  }
  kbd {
    padding: 1px 5px;
    border-radius: 5px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    font: 600 11px var(--font-mono);
  }
  .live {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 18px 10px 6px;
    flex: 1;
  }
  .state {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 600;
    color: var(--text-2);
  }
  .dot {
    width: 10px;
    height: 10px;
    border-radius: 50%;
    background: var(--rec-500);
    animation: pulse 1.6s infinite;
  }
  .paused .dot {
    background: var(--warn-500);
    animation: none;
  }
  @keyframes pulse {
    50% {
      opacity: 0.35;
    }
  }
  .clock {
    font: 600 44px/1 var(--font-mono);
    font-variant-numeric: tabular-nums;
    letter-spacing: -0.02em;
  }
  .meters {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: var(--text-3);
  }
  .meters span {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .controls {
    display: flex;
    gap: 8px;
    width: 100%;
    margin-top: 6px;
  }
  .ctl {
    flex: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 40px;
    border: 0;
    border-radius: 11px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text);
    font-weight: 650;
    transition: transform var(--dur-fast);
  }
  .ctl:active {
    transform: scale(0.97);
  }
  .ctl.stop {
    background: var(--rec-500);
    box-shadow: none;
    color: #fff;
  }
  .hint {
    display: flex;
    gap: 14px;
    margin-top: auto;
    color: var(--text-3);
    font-size: var(--text-xs);
  }
</style>
