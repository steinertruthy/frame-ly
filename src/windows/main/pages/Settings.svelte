<script lang="ts">
  import { onMount } from 'svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { Settings2, Scan, Video, AudioLines, Keyboard, Palette, HardDrive, Info, Check, FolderOpen } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import Button from '$lib/components/Button.svelte';
  import Section from '$lib/components/Section.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import SettingRow from '$lib/components/SettingRow.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import Select from '$lib/components/Select.svelte';
  import Slider from '$lib/components/Slider.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import MicTest from '$lib/components/MicTest.svelte';
  import PermissionRow from '$lib/components/PermissionRow.svelte';
  import ShortcutRecorder from '$lib/components/ShortcutRecorder.svelte';
  import LogoMark from '$lib/components/LogoMark.svelte';
  import StorageCard from '$lib/components/library/StorageCard.svelte';
  import RecordingEstimate from '$lib/components/RecordingEstimate.svelte';
  import { outputSize } from '$lib/videoEstimate';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { LOCALES, LOCALE_NAMES } from '$lib/i18n/translate';
  import { commands, events } from '$lib/ipc/commands';
  import type { AppError, AppInfo, Folder, Fps, ImageFormat, Language, Motion, RecordTarget, Settings, ShortcutStatus, Theme, VideoCodec, VideoQuality, VideoResolution } from '$lib/ipc/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { formatBytes } from '$lib/stores/format';
  import { applyAudioMode, audioMode, type AudioMode } from '$lib/recordSetup';
  import { RECORDING_ACTIONS, SHORTCUT_ACTIONS, type ShortcutAction } from '$lib/shortcuts';

  let { info }: { info: AppInfo } = $props();
  const s = $derived(settings.value);

  type Category = 'general' | 'captures' | 'recording' | 'audio' | 'shortcuts' | 'appearance' | 'storage' | 'about';
  const categories = $derived<{ id: Category; icon: Component<{ size?: number }>; label: MessageKey }[]>(
    [
      { id: 'general' as const, icon: Settings2, label: 'settings.cat.general' as const },
      { id: 'captures' as const, icon: Scan, label: 'settings.cat.captures' as const },
      { id: 'recording' as const, icon: Video, label: 'settings.cat.recording' as const },
      { id: 'audio' as const, icon: AudioLines, label: 'settings.cat.audio' as const },
      { id: 'shortcuts' as const, icon: Keyboard, label: 'settings.cat.shortcuts' as const },
      { id: 'appearance' as const, icon: Palette, label: 'settings.cat.appearance' as const },
      { id: 'storage' as const, icon: HardDrive, label: 'settings.cat.storage' as const },
      { id: 'about' as const, icon: Info, label: 'settings.cat.about' as const },
    ].filter((c) => info.supportsRecording || (c.id !== 'recording' && c.id !== 'audio')),
  );
  let category = $state<Category>('general');

  // ── Saving with feedback ───────────────────────────────────────────────
  let savedFlash = $state(0);
  async function set(mutate: (d: Settings) => void, describe?: (e: AppError) => string | null) {
    if (await settings.update(mutate, describe)) {
      savedFlash++;
      const mine = savedFlash;
      setTimeout(() => mine === savedFlash && (savedFlash = 0), 1600);
    }
  }

  // ── Shortcuts ──────────────────────────────────────────────────────────
  let shortcutStatus = $state<ShortcutStatus>({});
  onMount(() => {
    void commands.shortcutStatus().then((st) => (shortcutStatus = st));
    const un = events.onShortcutStatus((st) => (shortcutStatus = st));
    void library.init();
    void refreshCache();
    return () => void un.then((u) => u());
  });
  /** Pixel size of the main screen, for the recording size estimate and presets. */
  const screen = $derived.by((): [number, number] => {
    const d = system.displays.find((x) => x.isPrimary) ?? system.displays[0];
    return d ? [d.pixelWidth, d.pixelHeight] : [1920, 1080];
  });
  const shortcutActions = $derived(SHORTCUT_ACTIONS.filter((a) => info.supportsRecording || !RECORDING_ACTIONS.includes(a)));
  function describeShortcutError(e: AppError): string | null {
    if (e.message.includes('reserved')) return t('shortcuts.reserved');
    if (e.message.includes('needs')) return t('shortcuts.needsModifier');
    if (e.message.includes('same shortcut')) return t('shortcuts.duplicate');
    return null;
  }

  // ── Folders and storage ────────────────────────────────────────────────
  async function chooseFolder(kind: 'recordingsDir' | 'screenshotsDir') {
    if (!s) return;
    const picked = await openDialog({ directory: true, defaultPath: s.storage[kind], title: t('settings.chooseFolder') });
    if (typeof picked !== 'string' || picked === s.storage[kind]) return;
    await set((d) => (d.storage[kind] = picked));
  }

  async function reveal(folder: Folder) {
    try {
      await commands.openFolder(folder);
    } catch {
      toast('error', t('error.io'));
    }
  }

  let cacheBytes = $state<number | null>(null);
  async function refreshCache() {
    try {
      cacheBytes = (await commands.storageCache()).bytes;
    } catch {
      cacheBytes = null;
    }
  }
  async function cleanup() {
    try {
      const { bytes } = await commands.storageCleanup();
      toast('success', t('settings.storage.cleaned', { size: formatBytes(bytes) }));
      await refreshCache();
    } catch {
      toast('error', t('error.io'));
    }
  }

  const mode = $derived(s ? audioMode(s.audio) : 'none');
  const shortPath = (p: string) => p.replace(/^\/Users\/[^/]+/, '~').replace(/^C:\\Users\\[^\\]+/i, '~');
</script>

<div class="settings">
  <nav class="cats" aria-label={t('settings.title')}>
    <h1>{t('settings.title')}</h1>
    {#each categories as c (c.id)}
      <button class="cat" class:active={category === c.id} onclick={() => (category = c.id)}>
        <c.icon size={15} />
        <span>{t(c.label)}</span>
      </button>
    {/each}
  </nav>

  {#if s}
    <div class="content">
      <div class="content-head">
        <h2>{t(categories.find((c) => c.id === category)?.label ?? 'settings.cat.general')}</h2>
        {#if savedFlash}<span class="saved"><Check size={13} />{t('settings.saved')}</span>{/if}
      </div>

      {#key category}
        <div class="pane">
          {#if category === 'general'}
            <Section>
              <SettingRow title={t('settings.language')} hint={t('settings.language.hint')}>
                <Select
                  label={t('settings.language')}
                  value={s.general.language}
                  options={[
                    { value: 'system' as Language, label: t('settings.language.system') },
                    ...LOCALES.map((l) => ({ value: l as Language, label: LOCALE_NAMES[l] })),
                  ]}
                  onchange={(v) => set((d) => (d.general.language = v))}
                />
              </SettingRow>
            </Section>
            <Section title={t('settings.group.startup')}>
              <SettingRow title={t('settings.general.login')} hint={t('settings.general.login.hint')}>
                <Toggle label={t('settings.general.login')} checked={s.general.launchAtLogin} onchange={(v) => set((d) => (d.general.launchAtLogin = v))} />
              </SettingRow>
              <SettingRow title={t('settings.general.showOnLaunch')} hint={t('settings.general.showOnLaunch.hint')}>
                <Toggle label={t('settings.general.showOnLaunch')} checked={s.general.showOnLaunch} onchange={(v) => set((d) => (d.general.showOnLaunch = v))} />
              </SettingRow>
              <SettingRow title={t('settings.general.closeToTray')} hint={t(info.platform === 'macos' ? 'settings.general.closeToTray.hint.mac' : 'settings.general.closeToTray.hint.win')}>
                <Toggle label={t('settings.general.closeToTray')} checked={s.general.closeToTray} onchange={(v) => set((d) => (d.general.closeToTray = v))} />
              </SettingRow>
              <SettingRow title={t('settings.general.notifications')} hint={t('settings.general.notifications.hint')}>
                <Toggle label={t('settings.general.notifications')} checked={s.general.notifications} onchange={(v) => set((d) => (d.general.notifications = v))} />
              </SettingRow>
              <SettingRow title={t('settings.general.sounds')} hint={t('settings.general.sounds.hint')}>
                <Toggle
                  label={t('settings.general.sounds')}
                  checked={s.general.sounds}
                  onchange={(v) => {
                    set((d) => (d.general.sounds = v));
                    if (v) void commands.previewSound(); // let people hear what they just turned on
                  }}
                />
              </SettingRow>
            </Section>
            {#if system.permissions}
              <Section title={t('settings.permissions')}>
                <PermissionRow permission="screen" status={system.permissions.screen} platform={info.platform} />
                <PermissionRow permission="microphone" status={system.permissions.microphone} platform={info.platform} />
              </Section>
            {/if}
          {:else if category === 'captures'}
            <Section>
              <SettingRow title={t('settings.captures.folder')} hint={shortPath(s.storage.screenshotsDir)}>
                <Button size="sm" variant="ghost" onclick={() => reveal('screenshots')}><FolderOpen size={14} />{t('settings.show')}</Button>
                <Button size="sm" onclick={() => chooseFolder('screenshotsDir')}>{t('settings.change')}</Button>
              </SettingRow>
              <SettingRow title={t('settings.captures.format')} hint={t(s.capture.format === 'png' ? 'settings.captures.format.png' : 'settings.captures.format.jpeg')}>
                <Segmented
                  label={t('settings.captures.format')}
                  value={s.capture.format}
                  options={[
                    { value: 'png' as ImageFormat, label: 'PNG' },
                    { value: 'jpeg' as ImageFormat, label: 'JPEG' },
                  ]}
                  onchange={(v) => set((d) => (d.capture.format = v))}
                />
              </SettingRow>
              {#if s.capture.format === 'jpeg'}
                <SettingRow title={t('settings.captures.quality')} hint={t('settings.captures.quality.hint')}>
                  <Slider
                    label={t('settings.captures.quality')}
                    value={s.capture.jpegQuality}
                    min={50}
                    max={100}
                    step={5}
                    format={(v) => `${Math.round(v)}%`}
                    onchange={(v) => set((d) => (d.capture.jpegQuality = Math.round(v)))}
                  />
                </SettingRow>
              {/if}
            </Section>
            <Section title={t('settings.group.afterShot')}>
              <SettingRow title={t('settings.capture.copy')} hint={t('settings.capture.copy.hint')}>
                <Toggle label={t('settings.capture.copy')} checked={s.capture.copyToClipboard} onchange={(v) => set((d) => (d.capture.copyToClipboard = v))} />
              </SettingRow>
              <SettingRow title={t('settings.capture.save')} hint={t('settings.capture.save.hint')}>
                <Toggle label={t('settings.capture.save')} checked={s.capture.saveToDisk} onchange={(v) => set((d) => (d.capture.saveToDisk = v))} />
              </SettingRow>
              <SettingRow title={t('settings.capture.preview')} hint={t('settings.capture.preview.hint')}>
                <Toggle label={t('settings.capture.preview')} checked={s.capture.showPreview} onchange={(v) => set((d) => (d.capture.showPreview = v))} />
              </SettingRow>
            </Section>
          {:else if category === 'recording'}
            <Section>
              <SettingRow title={t('settings.recording.folder')} hint={shortPath(s.storage.recordingsDir)}>
                <Button size="sm" variant="ghost" onclick={() => reveal('recordings')}><FolderOpen size={14} />{t('settings.show')}</Button>
                <Button size="sm" onclick={() => chooseFolder('recordingsDir')}>{t('settings.change')}</Button>
              </SettingRow>
              <SettingRow title={t('settings.recording.target')} hint={t('settings.recording.target.hint')}>
                <Segmented
                  label={t('settings.recording.target')}
                  value={s.recording.target}
                  options={[
                    { value: 'screen' as RecordTarget, label: t('rec.target.screen') },
                    { value: 'window' as RecordTarget, label: t('rec.target.window') },
                    { value: 'region' as RecordTarget, label: t('rec.target.region') },
                  ]}
                  onchange={(v) => set((d) => (d.recording.target = v))}
                />
              </SettingRow>
            </Section>
            <Section title={t('settings.group.video')}>
              <RecordingEstimate
                {screen}
                quality={s.recording.quality}
                resolution={s.recording.resolution}
                codec={s.recording.codec}
                fps={s.recording.fps}
              />
              <SettingRow title={t('settings.recording.quality')} hint={t(`settings.recording.quality.hint.${s.recording.quality}`)}>
                <Segmented
                  label={t('settings.recording.quality')}
                  value={s.recording.quality}
                  options={(['low', 'medium', 'high', 'max'] as VideoQuality[]).map((q) => ({ value: q, label: t(`settings.recording.quality.${q}`) }))}
                  onchange={(v) => set((d) => (d.recording.quality = v))}
                />
              </SettingRow>
              <SettingRow title={t('settings.recording.fps')} hint={t(`settings.recording.fps.hint.${s.recording.fps}`)}>
                <Segmented
                  label={t('settings.recording.fps')}
                  value={String(s.recording.fps)}
                  options={[
                    { value: '24', label: '24 fps' },
                    { value: '30', label: '30 fps' },
                    { value: '60', label: '60 fps' },
                  ]}
                  onchange={(v) => set((d) => (d.recording.fps = Number(v) as Fps))}
                />
              </SettingRow>
              <SettingRow
                title={t('settings.recording.resolution')}
                hint={t(`settings.recording.resolution.hint.${s.recording.resolution}`, {
                  size: outputSize(screen, s.recording.resolution, s.recording.codec).join(' × '),
                })}
              >
                <Segmented
                  label={t('settings.recording.resolution')}
                  value={s.recording.resolution}
                  options={[
                    { value: 'native' as VideoResolution, label: t('settings.recording.resolution.native') },
                    { value: '1080p' as VideoResolution, label: '1080p' },
                    { value: '720p' as VideoResolution, label: '720p' },
                  ]}
                  onchange={(v) => set((d) => (d.recording.resolution = v))}
                />
              </SettingRow>
              <SettingRow title={t('settings.recording.codec')} hint={t(s.recording.codec === 'h264' ? 'settings.recording.codec.h264' : 'settings.recording.codec.hevc')}>
                <Segmented
                  label={t('settings.recording.codec')}
                  value={s.recording.codec}
                  options={[
                    { value: 'h264' as VideoCodec, label: t('settings.recording.codec.compatible') },
                    { value: 'hevc' as VideoCodec, label: t('settings.recording.codec.smaller') },
                  ]}
                  onchange={(v) => set((d) => (d.recording.codec = v))}
                />
              </SettingRow>
            </Section>
            <Section title={t('settings.group.whileRecording')}>
              <SettingRow title={t('settings.recording.controls')} hint={t('settings.recording.controls.hint')}>
                <Toggle label={t('settings.recording.controls')} checked={s.recording.showControls} onchange={(v) => set((d) => (d.recording.showControls = v))} />
              </SettingRow>
              <SettingRow title={t('settings.recording.countdown')} hint={t('settings.recording.countdown.hint')}>
                <Segmented
                  label={t('settings.recording.countdown')}
                  value={String(s.recording.countdown)}
                  options={[
                    { value: '0', label: t('settings.recording.countdown.off') },
                    { value: '3', label: '3 s' },
                    { value: '5', label: '5 s' },
                  ]}
                  onchange={(v) => set((d) => (d.recording.countdown = Number(v)))}
                />
              </SettingRow>
              <SettingRow title={t('settings.recording.cursor')} hint={t('settings.recording.cursor.hint')}>
                <Toggle label={t('settings.recording.cursor')} checked={s.recording.showCursor} onchange={(v) => set((d) => (d.recording.showCursor = v))} />
              </SettingRow>
              <SettingRow title={t('settings.recording.preview')} hint={t('settings.recording.preview.hint')}>
                <Toggle label={t('settings.recording.preview')} checked={s.recording.showPreview} onchange={(v) => set((d) => (d.recording.showPreview = v))} />
              </SettingRow>
            </Section>
          {:else if category === 'audio'}
            <Section>
              <SettingRow title={t('settings.audio.default')} hint={t('settings.audio.default.hint')} stacked>
                <Segmented
                  stretch
                  label={t('settings.audio.default')}
                  value={mode}
                  options={[
                    { value: 'both' as AudioMode, label: t('rec.audio.both') },
                    { value: 'mic' as AudioMode, label: t('rec.audio.mic') },
                    { value: 'system' as AudioMode, label: t('rec.audio.system') },
                    { value: 'none' as AudioMode, label: t('rec.audio.none') },
                  ]}
                  onchange={(v) => set((d) => applyAudioMode(d.audio, v))}
                />
              </SettingRow>
            </Section>
            <Section title={t('settings.group.microphone')}>
              <SettingRow title={t('audio.device')} hint={t('audio.device.hint')}>
                <Select
                  label={t('audio.device')}
                  value={s.audio.microphoneDevice ?? ''}
                  options={[{ value: '', label: t('audio.device.default') }, ...system.audioInputs.map((d) => ({ value: d.id, label: d.name }))]}
                  onchange={(v) => set((d) => (d.audio.microphoneDevice = v || null))}
                />
              </SettingRow>
              <SettingRow title={t('audio.micVolume')} hint={t('audio.test.hint')}>
                <MicTest deviceId={s.audio.microphoneDevice} />
                <Slider label={t('audio.micVolume')} value={s.audio.microphoneGain} onchange={(v) => set((d) => (d.audio.microphoneGain = v))} />
              </SettingRow>
              <SettingRow title={t('settings.audio.noise')} hint={t('settings.audio.noise.hint')}>
                <Toggle label={t('settings.audio.noise')} checked={s.audio.noiseSuppression} onchange={(v) => set((d) => (d.audio.noiseSuppression = v))} />
              </SettingRow>
            </Section>
            <Section title={t('settings.group.computerSound')}>
              <SettingRow title={t('settings.audio.systemSource')} hint={t('settings.audio.systemSource.hint')}>
                <Badge tone="ok">{t('settings.audio.systemSource.value')}</Badge>
              </SettingRow>
              <SettingRow title={t('audio.systemVolume')}>
                <Slider label={t('audio.systemVolume')} value={s.audio.systemGain} onchange={(v) => set((d) => (d.audio.systemGain = v))} />
              </SettingRow>
            </Section>
          {:else if category === 'shortcuts'}
            <p class="lead">{t('shortcuts.hint')}</p>
            <Section>
              {#each shortcutActions as action (action)}
                <SettingRow title={t(`shortcuts.action.${action}`)}>
                  {#if s.shortcuts[action] && shortcutStatus[action]}<Badge tone="warn">{t('shortcuts.inUse')}</Badge>{/if}
                  <ShortcutRecorder
                    label={t(`shortcuts.action.${action}`)}
                    value={s.shortcuts[action] ?? ''}
                    platform={info.platform}
                    onchange={(v) => set((d) => (d.shortcuts[action] = v), describeShortcutError)}
                  />
                </SettingRow>
              {/each}
            </Section>
            <div class="foot-row">
              <span class="muted">{t('shortcuts.recordingHint')}</span>
              <Button size="sm" variant="ghost" onclick={() => commands.resetShortcuts()}>{t('shortcuts.reset')}</Button>
            </div>
          {:else if category === 'appearance'}
            <Section>
              <SettingRow title={t('settings.theme')} hint={t('settings.theme.hint')}>
                <Segmented
                  label={t('settings.theme')}
                  value={s.appearance.theme}
                  options={[
                    { value: 'dark' as Theme, label: t('settings.theme.dark') },
                    { value: 'light' as Theme, label: t('settings.theme.light') },
                    { value: 'system' as Theme, label: t('settings.theme.system') },
                  ]}
                  onchange={(v) => set((d) => (d.appearance.theme = v))}
                />
              </SettingRow>
              <SettingRow title={t('settings.appearance.ambient')} hint={t('settings.appearance.ambient.hint')}>
                <Toggle label={t('settings.appearance.ambient')} checked={s.appearance.ambientLight} onchange={(v) => set((d) => (d.appearance.ambientLight = v))} />
              </SettingRow>
              <SettingRow title={t('settings.appearance.motion')} hint={t('settings.appearance.motion.hint')}>
                <Segmented
                  label={t('settings.appearance.motion')}
                  value={s.appearance.motion}
                  options={[
                    { value: 'system' as Motion, label: t('settings.appearance.motion.system') },
                    { value: 'full' as Motion, label: t('settings.appearance.motion.full') },
                    { value: 'reduced' as Motion, label: t('settings.appearance.motion.reduced') },
                  ]}
                  onchange={(v) => set((d) => (d.appearance.motion = v))}
                />
              </SettingRow>
            </Section>
          {:else if category === 'storage'}
            {#if library.overview}<div class="storage-card"><StorageCard overview={library.overview} /></div>{/if}
            <Section title={t('settings.group.locations')}>
              <SettingRow title={t('settings.captures.folder')} hint={shortPath(s.storage.screenshotsDir)}>
                <Button size="sm" variant="ghost" onclick={() => reveal('screenshots')}><FolderOpen size={14} />{t('settings.show')}</Button>
                <Button size="sm" onclick={() => chooseFolder('screenshotsDir')}>{t('settings.change')}</Button>
              </SettingRow>
              {#if info.supportsRecording}
                <SettingRow title={t('settings.recording.folder')} hint={shortPath(s.storage.recordingsDir)}>
                  <Button size="sm" variant="ghost" onclick={() => reveal('recordings')}><FolderOpen size={14} />{t('settings.show')}</Button>
                  <Button size="sm" onclick={() => chooseFolder('recordingsDir')}>{t('settings.change')}</Button>
                </SettingRow>
              {/if}
            </Section>
            <Section title={t('settings.group.maintenance')}>
              <SettingRow title={t('settings.storage.temp')} hint={t('settings.storage.temp.hint', { size: cacheBytes == null ? '—' : formatBytes(cacheBytes) })}>
                <Button size="sm" onclick={cleanup}>{t('settings.storage.clean')}</Button>
              </SettingRow>
            </Section>
          {:else}
            <div class="about">
              <LogoMark size={64} />
              <div>
                <h3>FrameLy</h3>
                <p class="muted">{t('settings.about.version', { version: info.version })} · {info.platform} {info.arch}</p>
              </div>
            </div>
            <Section>
              <SettingRow title={t('settings.about.updates')} hint={t('settings.about.updates.hint')}><span></span></SettingRow>
              <SettingRow title={t('settings.logs')} hint={t('settings.logs.hint')}>
                <Button size="sm" onclick={() => reveal('logs')}><FolderOpen size={14} />{t('settings.show')}</Button>
              </SettingRow>
            </Section>
            <Section title={t('settings.about.credits')}>
              <SettingRow title={t('settings.about.builtWith')} hint="Tauri · Rust · Svelte · Swift · ScreenCaptureKit · AVFoundation · SQLite"><span></span></SettingRow>
            </Section>
          {/if}
        </div>
      {/key}
    </div>
  {/if}
</div>

<style>
  .settings {
    display: grid;
    grid-template-columns: 200px 1fr;
    gap: var(--space-6);
    align-items: start;
  }
  .cats {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .cats h1 {
    margin: 0 0 var(--space-4) 10px;
    font-size: 22px;
    font-weight: 650;
  }
  .cat {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 34px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-2);
    font-weight: 550;
    text-align: left;
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .cat:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .cat.active {
    background: var(--nav-active);
    box-shadow: inset 0 0 0 1px var(--nav-active-border);
    color: var(--text);
  }
  .cat.active :global(svg) {
    color: var(--icon-accent);
  }
  .content {
    min-width: 0;
  }
  .content-head {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: 42px;
    margin-bottom: var(--space-3);
  }
  .content-head h2 {
    font-size: 17px;
    font-weight: 650;
  }
  .saved {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--ok-500);
    font-size: var(--text-sm);
    font-weight: 600;
    animation: saved var(--dur-slow) var(--ease-out);
  }
  @keyframes saved {
    from {
      opacity: 0;
      transform: translateX(-4px);
    }
  }
  .pane {
    animation: pane var(--dur-slow) var(--ease-out);
  }
  @keyframes pane {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  .lead,
  .muted {
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .lead {
    margin: -4px 0 var(--space-4) 4px;
  }
  .foot-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: -8px 4px 0;
  }
  .storage-card {
    margin-bottom: var(--space-5);
  }
  .about {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin-bottom: var(--space-5);
  }
  .about h3 {
    font-size: var(--text-xl);
  }
  @media (max-width: 860px) {
    .settings {
      grid-template-columns: 1fr;
      gap: var(--space-4);
    }
    .cats {
      position: static;
      flex-direction: row;
      flex-wrap: wrap;
    }
    .cats h1 {
      width: 100%;
    }
  }
</style>
