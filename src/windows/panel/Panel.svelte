<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import { Settings, Images, Camera, ChevronDown, Pause, Play, Square, Eye } from '@lucide/svelte';
  import Aurora from '$lib/components/Aurora.svelte';
  import LogoMark from '$lib/components/LogoMark.svelte';
  import MuteMeter from '$lib/components/MuteMeter.svelte';
  import RecordSetup from '$lib/components/RecordSetup.svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import { itemActions } from '$lib/components/library/actions';
  import { commands, events, isAppError } from '$lib/ipc/commands';
  import { logUi } from '$lib/ipc/log';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { AppInfo } from '$lib/ipc/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import { recording, formatDuration } from '$lib/stores/recording.svelte';
  import { audioMode, recordRequest } from '$lib/recordSetup';
  import { formatAccelerator, type ShortcutAction } from '$lib/shortcuts';

  let { info }: { info: AppInfo } = $props();

  let error = $state<string | null>(null);
  let options = $state(false);
  /** The floating controls were hidden during this recording (offer to bring them back). */
  let pillHidden = $state(false);
  async function refreshPill() {
    pillHidden = live ? await commands.recordingControlsHidden().catch(() => false) : false;
  }
  async function showPill() {
    await commands.setRecordingControlsVisible(true).catch(() => {});
    pillHidden = false;
  }
  let root: HTMLDivElement | undefined = $state();

  const s = $derived(settings.value);
  const rec = $derived(recording.status);
  const live = $derived(rec.phase !== 'idle');
  const summary = $derived(
    s ? `${t(`rec.target.${s.recording.target}` as MessageKey)} · ${t(`panel.audio.${audioMode(s.audio)}` as MessageKey)}` : '',
  );
  const keys = (a: ShortcutAction) => formatAccelerator(s?.shortcuts[a] ?? '', info.platform);

  onMount(() => {
    void library.init();
    const un = events.onPanelShown(() => {
      error = null;
      // Focus the panel itself, not its first button (no stray focus ring on open). Deferred:
      // the event arrives just before the window becomes key.
      setTimeout(() => {
        (document.activeElement as HTMLElement | null)?.blur();
        root?.focus();
      }, 60);
      void system.refresh();
      void refreshPill();
      library.sync();
    });
    const onKey = (e: KeyboardEvent) => e.key === 'Escape' && void commands.panelHide();
    window.addEventListener('keydown', onKey);
    // The window always fits the content (options expanded, recording state, recents).
    const ro = new ResizeObserver(() => {
      if (root) void commands.panelResize(Math.ceil(root.scrollHeight));
    });
    if (root) ro.observe(root);
    return () => {
      ro.disconnect();
      void un.then((u) => u());
      window.removeEventListener('keydown', onKey);
    };
  });

  function fail(e: unknown) {
    error = t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic');
    logUi('warn', `panel action failed: ${JSON.stringify(e)}`);
  }

  async function screenshot() {
    error = null;
    // Hide first: the panel must not sit on top of the selection overlay.
    await commands.panelHide();
    try {
      await commands.takeScreenshot({ kind: 'select', mode: 'region' });
    } catch (e) {
      fail(e);
    }
  }

  async function record() {
    if (!s) return;
    error = null;
    await commands.panelHide();
    try {
      await commands.startRecording(recordRequest(s.recording.target, recordPrefs.displayId || null));
    } catch (e) {
      fail(e);
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
    ({ countdown: 'record.countdown', recording: 'record.recording', paused: 'record.paused', finalizing: 'record.finalizing', idle: 'record.recording' } as const)[rec.phase],
  );
</script>

{#snippet chip(list: string[], cls = '')}
  {#if list.length}<kbd class="keys {cls}">{#each list as k, i (i)}<span>{k}</span>{/each}</kbd>{/if}
{/snippet}

<Aurora intensity={0.7} />

<div class="panel" class:mac={info.platform === 'macos'} bind:this={root} tabindex="-1" role="dialog" aria-label="FrameLy">
  <header>
    <LogoMark size={24} tile />
    <span class="brand">FrameLy</span>
    <button class="icon" title={t('nav.library')} aria-label={t('nav.library')} onclick={() => commands.showMainWindow('library')}><Images size={15} /></button>
    <button class="icon" title={t('panel.settings')} aria-label={t('panel.settings')} onclick={() => commands.showMainWindow('settings')}><Settings size={15} /></button>
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
          {#if rec.audio.microphone}
            <MuteMeter source="microphone" level={rec.audio.levels.microphone} muted={rec.audio.microphoneMuted} segments={16} labeled />
          {/if}
          {#if rec.audio.system}
            <MuteMeter source="system" level={rec.audio.levels.system} muted={rec.audio.systemMuted} segments={16} labeled />
          {/if}
        </div>
      {/if}
      <div class="controls">
        {#if rec.phase === 'recording'}
          <button class="ctl" onclick={() => control(commands.pauseRecording)}><Pause size={16} />{t('record.pause')}</button>
        {:else if rec.phase === 'paused'}
          <button class="ctl" onclick={() => control(commands.resumeRecording)}><Play size={16} />{t('record.resume')}</button>
        {/if}
        <button class="ctl stop" disabled={rec.phase === 'finalizing'} onclick={() => control(commands.stopRecording)}>
          <Square size={13} fill="currentColor" />{t('record.stop')}
        </button>
      </div>
      {#if pillHidden}
        <button class="link show-pill" onclick={showPill}><Eye size={14} />{t('record.showControls')}</button>
      {/if}
    </section>
  {:else}
    {#if system.permissions && !system.screenReady}
      <div class="notice">
        <span>{t('panel.needsPermission')}</span>
        <button class="link" onclick={() => commands.showMainWindow('home')}>{t('panel.setup')}</button>
      </div>
    {/if}

    <!-- 1. Screenshot -->
    <button class="shot" disabled={!system.screenReady} onclick={screenshot}>
      <span class="shot-icon"><Camera size={19} strokeWidth={1.8} /></span>
      <span class="shot-text">
        <span class="shot-title">{t('home.shot.title')}</span>
        <span class="shot-sub">{t('panel.shot.sub')}</span>
      </span>
      {@render chip(keys('captureRegion'), 'on-primary')}
    </button>

    <!-- 2. Recording -->
    {#if info.supportsRecording}
      <section class="rec">
        <div class="rec-row">
          <button class="rec-start" disabled={!system.screenReady} onclick={record}><span class="rec-dot"></span>{t('panel.rec.start')}</button>
          <button class="rec-summary" onclick={() => (options = !options)} aria-expanded={options}>
            <span>{summary}</span>
            <ChevronDown size={14} />
          </button>
        </div>
        {#if options}
          <div class="rec-options" transition:slide={{ duration: 220 }}>
            <RecordSetup />
          </div>
        {/if}
      </section>
    {/if}

    <!-- 3. Recent files -->
    {#if library.recent.length}
      <section class="recent">
        <div class="recent-head">
          <span class="recent-label">{t('panel.recent')}</span>
          <button class="link" onclick={() => commands.showMainWindow('library')}>{t('home.seeAll')}</button>
        </div>
        <div class="recent-grid">
          {#each library.recent as item (item.id)}
            <MediaTile {item} compact onopen={(i) => itemActions.open(i)} />
          {/each}
        </div>
      </section>
    {/if}

    {#if error}<div class="error">{error}</div>{/if}

    <!-- 4. App -->
    <footer>
      <button class="link" onclick={() => commands.showMainWindow('home')}>{t('panel.open')}</button>
      {@render chip(keys('openPanel'))}
    </footer>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
    overflow: hidden;
  }
  /* The aurora lives inside the rounded panel only. */
  :global(.aurora) {
    border-radius: 16px;
  }
  .panel {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 14px 0;
    border-radius: 16px;
    background: color-mix(in srgb, var(--bg) 96%, transparent);
    box-shadow: inset 0 0 0 1px var(--border-strong);
    animation: open 200ms var(--ease-out);
  }
  .panel.mac {
    background: color-mix(in srgb, var(--bg) 92%, transparent);
  }
  .panel:focus {
    outline: none;
  }
  @keyframes open {
    from {
      opacity: 0;
      transform: translateY(-6px) scale(0.985);
    }
  }
  header {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  header {
    padding: 0 2px;
  }
  .brand {
    flex: 1;
    font: 650 var(--text-lg) var(--font-display);
    letter-spacing: -0.015em;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
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

  /* 1. Screenshot */
  .shot {
    position: relative;
    display: flex;
    align-items: center;
    gap: 12px;
    width: 100%;
    padding: 14px 12px 14px 14px;
    border: 0;
    border-radius: 14px;
    overflow: hidden;
    background: var(--primary-fill);
    color: var(--accent-text);
    text-align: left;
    box-shadow: inset 0 0 0 1px var(--primary-border);
    transition:
      transform var(--dur-med) var(--ease-out),
      background var(--dur-fast);
  }
  .shot:hover:not(:disabled) {
    background: var(--primary-hover);
  }
  .shot:active:not(:disabled) {
    transform: scale(0.985);
  }
  .shot:disabled {
    opacity: 0.5;
  }
  .shot-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    flex-shrink: 0;
    border-radius: 11px;
    background: rgba(139, 176, 238, 0.1);
    box-shadow: inset 0 0 0 1px rgba(139, 176, 238, 0.28);
    color: var(--primary-icon);
  }
  .shot-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  .shot-title {
    font: 650 var(--text-lg) var(--font-display);
  }
  .shot-sub {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: color-mix(in srgb, var(--accent-text) 72%, transparent);
    font-size: var(--text-sm);
  }

  /* 2. Recording */
  .rec {
    border-radius: 14px;
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .rec-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px;
  }
  .rec-start {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 30px;
    padding: 0 12px;
    border: 0;
    border-radius: 8px;
    background: var(--rec-500);
    color: #fff;
    font-size: var(--text-sm);
    font-weight: 600;
    white-space: nowrap;
    transition: background var(--dur-fast);
  }
  .rec-start:hover:not(:disabled) {
    background: var(--rec-hover);
  }
  .rec-start:disabled {
    opacity: 0.5;
  }
  .rec-dot {
    border-radius: 50%;
    width: 6px;
    height: 6px;
    background: currentColor;
  }
  .rec-summary {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    height: 30px;
    padding: 0 6px 0 8px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: var(--text-2);
    font-size: var(--text-sm);
    text-align: left;
  }
  .rec-summary span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .rec-summary:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .rec-summary :global(svg) {
    flex-shrink: 0;
    transition: transform var(--dur-med) var(--ease-out);
  }
  .rec-summary[aria-expanded='true'] :global(svg) {
    transform: rotate(180deg);
  }
  .rec-options {
    padding: 12px 12px 12px;
    box-shadow: inset 0 1px 0 var(--border);
  }

  /* 3. Recent */
  .recent-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .recent-label {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .recent-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 6px;
  }
  .link {
    border: 0;
    background: none;
    color: var(--link);
    font-size: var(--text-sm);
    font-weight: 550;
  }
  .link:hover {
    color: var(--link-hover);
  }

  /* 4. App */
  footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 0 -14px;
    padding: 12px 16px 14px;
    box-shadow: inset 0 1px 0 var(--border);
    color: var(--text-3);
  }
  /* Shortcut chip: "⌥ ⇧ ⌘ 4" */
  .keys {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 9px;
    border-radius: 7px;
    background: var(--kbd-bg);
    box-shadow: inset 0 0 0 1px var(--border-strong);
    color: var(--text-2);
    font: 500 11px/18px var(--font-ui);
    white-space: nowrap;
  }
  .keys.on-primary {
    flex-shrink: 0;
    gap: 4px;
    padding: 2px 7px;
    background: rgba(7, 9, 13, 0.3);
    box-shadow: inset 0 0 0 1px rgba(139, 176, 238, 0.22);
    color: color-mix(in srgb, var(--accent-text) 85%, transparent);
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

  /* Live recording */
  .live {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 14px 6px 16px;
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
    box-shadow: 0 0 12px var(--rec-500);
    animation: pulse 1.6s infinite;
  }
  .paused .dot {
    background: var(--warn-500);
    box-shadow: none;
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
  .show-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
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
    background: var(--surface-raised);
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
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.2);
    color: #fff;
  }
  .hint {
    display: flex;
    gap: 14px;
    color: var(--text-3);
    font-size: var(--text-xs);
  }
</style>
