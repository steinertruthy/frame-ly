<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import { ArrowRight, Camera, Mic, MicOff, SlidersHorizontal, Video, Volume2, VolumeX } from '@lucide/svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import Viewer from '$lib/components/library/Viewer.svelte';
  import PermissionRow from '$lib/components/PermissionRow.svelte';
  import RecordSetup from '$lib/components/RecordSetup.svelte';
  import RecordingBanner from '$lib/components/RecordingBanner.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import { commands, isAppError } from '$lib/ipc/commands';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { recording } from '$lib/stores/recording.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { audioMode, recordRequest, summaryKeys } from '$lib/recordSetup';
  import { formatAccelerator, type ShortcutAction } from '$lib/shortcuts';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import type { AppInfo, CaptureRequest, LibraryItem } from '$lib/ipc/types';

  let { info, onnavigate }: { info: AppInfo; onnavigate: (page: 'library' | 'settings') => void } = $props();
  let viewing = $state<{ items: LibraryItem[]; id: number } | null>(null);
  let customizing = $state(false);

  onMount(() => void library.init());

  const s = $derived(settings.value);
  const keys = (a: ShortcutAction) => formatAccelerator(s?.shortcuts[a] ?? '', info.platform);
  const shotAction = $derived(
    ({ region: 'captureRegion', window: 'captureWindow', screen: 'captureScreen' } as const)[recordPrefs.shotMode],
  );
  const recAction = $derived(
    s ? ({ region: 'recordRegion', window: 'recordWindow', screen: 'recordScreen' } as const)[s.recording.target] : 'recordScreen',
  );
  const summary = $derived(s ? summaryKeys(s.recording.target, audioMode(s.audio)) : null);
  const busy = $derived(!system.screenReady || recording.active);

  const shotModes = $derived([
    { value: 'region' as const, label: t('home.shot.mode.region') },
    { value: 'window' as const, label: t('home.shot.mode.window') },
    { value: 'screen' as const, label: t('home.shot.mode.screen') },
  ]);

  function fail(e: unknown) {
    toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
  }

  async function screenshot(mode = recordPrefs.shotMode) {
    const request: CaptureRequest =
      mode === 'screen' ? { kind: 'display', displayId: recordPrefs.displayId || null } : { kind: 'select', mode };
    try {
      await commands.takeScreenshot(request);
    } catch (e) {
      fail(e);
    }
  }

  async function record() {
    if (!s) return;
    try {
      await commands.startRecording(recordRequest(s.recording.target, recordPrefs.displayId || null));
    } catch (e) {
      fail(e);
    }
  }

  function toggleAudio(source: 'microphone' | 'systemAudio') {
    void settings.update((d) => (d.audio[source] = !d.audio[source]));
  }

  function seeAll(tab: 'image' | 'video') {
    library.showTab(tab);
    onnavigate('library');
  }
</script>

{#snippet shortcut(list: string[])}
  {#if list.length}<kbd class="keys">{#each list as k, i (i)}<span>{k}</span>{/each}</kbd>{/if}
{/snippet}

{#snippet emptyBody(withKeys: MessageKey, plain: MessageKey, list: string[])}
  {#if list.length}
    {@const [before, after] = t(withKeys).split('{keys}')}
    {before}{@render shortcut(list)}{after}
  {:else}
    {t(plain)}
  {/if}
{/snippet}

{#if system.permissions && !system.screenReady}
  <section class="setup">
    <h2>{t('perm.title')}</h2>
    <p>{t('perm.subtitle')}</p>
    <div class="card">
      <PermissionRow permission="screen" status={system.permissions.screen} platform={info.platform} />
      <PermissionRow permission="microphone" status={system.permissions.microphone} platform={info.platform} />
    </div>
  </section>
{/if}

<!-- Capture studio: screenshot and recording side by side -->
<section class="studio" class:single={!info.supportsRecording}>
  <div class="block shot">
    <header>
      <h2>{t('home.shot.title')}</h2>
      {@render shortcut(keys(shotAction))}
    </header>
    <p class="desc">{t('home.shot.desc')}</p>
    <div class="actions">
      <Segmented options={shotModes} value={recordPrefs.shotMode} label={t('home.shot.mode')} onchange={(v) => (recordPrefs.shotMode = v)} />
      <button class="btn primary" disabled={busy} onclick={() => screenshot()}>
        <Camera size={16} strokeWidth={1.9} /><span>{t('home.shot.capture')}</span>
      </button>
    </div>
  </div>

  {#if info.supportsRecording}
    <div class="block rec">
      <header>
        <h2>{t('home.rec.title')}</h2>
        {@render shortcut(keys(recAction))}
      </header>
      {#if summary && s}
        <p class="desc">{t(summary[0])} · {t(summary[1])} · {t('home.rec.fps', { n: s.recording.fps })}</p>
      {/if}
      <div class="actions">
        <div class="chips">
          <button class="chip" aria-pressed={s?.audio.microphone ?? false} onclick={() => toggleAudio('microphone')} title={t('audio.microphone')}>
            {#if s?.audio.microphone}<Mic size={14} />{:else}<MicOff size={14} />{/if}<span>{t('home.rec.mic')}</span>
          </button>
          <button class="chip" aria-pressed={s?.audio.systemAudio ?? false} onclick={() => toggleAudio('systemAudio')} title={t('audio.system')}>
            {#if s?.audio.systemAudio}<Volume2 size={14} />{:else}<VolumeX size={14} />{/if}<span>{t('home.rec.system')}</span>
          </button>
          <button class="ghost" class:on={customizing} onclick={() => (customizing = !customizing)} aria-expanded={customizing} title={t('home.rec.customize')}>
            <SlidersHorizontal size={14} /><span>{t('home.rec.customize')}</span>
          </button>
        </div>
        <button class="btn record" disabled={busy} onclick={record}>
          <span class="dot" aria-hidden="true"></span><span>{t('home.rec.start')}</span>
        </button>
      </div>
    </div>
    {#if customizing}
      <div class="setup-body" transition:slide={{ duration: 240 }}>
        <RecordSetup />
      </div>
    {/if}
  {/if}
</section>

{#if recording.active}
  <div class="rec-live"><RecordingBanner /></div>
{/if}

<!-- Recents -->
<section class="recent">
  <header>
    <h2>{t('home.recent.shots')}</h2>
    {#if library.overview}<span class="count">{library.overview.images}</span>{/if}
    {#if library.recentImages.length}<button class="link" onclick={() => seeAll('image')}>{t('home.seeAll')}<ArrowRight size={14} /></button>{/if}
  </header>
  {#if library.recentImages.length}
    <div class="grid images">
      {#each library.recentImages as item (item.id)}
        <MediaTile {item} card onopen={(i) => (viewing = { items: library.recentImages, id: i.id })} />
      {/each}
    </div>
  {:else if library.loaded}
    <div class="empty">
      <span class="empty-icon" aria-hidden="true"><Camera size={18} /></span>
      <div class="empty-text">
        <span class="empty-title">{t('home.recent.shots.emptyTitle')}</span>
        <span class="empty-body">{@render emptyBody('home.recent.shots.emptyBody', 'home.recent.shots.emptyBodyPlain', keys('captureRegion'))}</span>
      </div>
      <button class="btn primary sm" disabled={busy} onclick={() => screenshot('region')}>
        <Camera size={15} strokeWidth={1.9} /><span>{t('home.recent.shots.cta')}</span>
      </button>
    </div>
  {/if}
</section>

{#if info.supportsRecording}
  <section class="recent">
    <header>
      <h2>{t('home.recent.videos')}</h2>
      {#if library.overview}<span class="count">{library.overview.videos}</span>{/if}
      {#if library.recentVideos.length}<button class="link" onclick={() => seeAll('video')}>{t('home.seeAll')}<ArrowRight size={14} /></button>{/if}
    </header>
    {#if library.recentVideos.length}
      <div class="grid videos">
        {#each library.recentVideos as item (item.id)}
          <MediaTile {item} card onopen={(i) => (viewing = { items: library.recentVideos, id: i.id })} />
        {/each}
      </div>
    {:else if library.loaded}
      <div class="empty">
        <span class="empty-icon rec-tone" aria-hidden="true"><Video size={18} /></span>
        <div class="empty-text">
          <span class="empty-title">{t('home.recent.videos.emptyTitle')}</span>
          <span class="empty-body">{@render emptyBody('home.recent.videos.emptyBody', 'home.recent.videos.emptyBodyPlain', keys(recAction))}</span>
        </div>
        <button class="btn record sm" disabled={busy} onclick={record}>
          <span class="dot" aria-hidden="true"></span><span>{t('home.rec.start')}</span>
        </button>
      </div>
    {/if}
  </section>
{/if}

{#if viewing}
  <Viewer items={viewing.items} bind:id={viewing.id} onclose={() => (viewing = null)} />
{/if}

<style>
  .setup {
    margin-bottom: var(--space-5);
  }
  .setup h2 {
    font-size: var(--text-lg);
  }
  .setup p {
    margin: 2px 0 var(--space-3);
    color: var(--text-2);
  }
  .card {
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
  }

  /* Studio */
  .studio {
    display: grid;
    grid-template-columns: 1fr 1fr;
    border-radius: 14px;
    overflow: hidden;
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .studio.single {
    grid-template-columns: 1fr;
  }
  .block {
    display: flex;
    flex-direction: column;
    min-width: 0;
    padding: 22px 24px 24px;
  }
  .shot {
    background: var(--hero);
  }
  .rec {
    box-shadow: inset 1px 0 0 var(--border);
    container-type: inline-size;
  }
  /* Narrow column: "Customize" keeps its icon (and tooltip), the labels of the rest stay. */
  @container (max-width: 500px) {
    .ghost span {
      display: none;
    }
    .ghost {
      padding: 0 9px;
    }
  }
  @container (max-width: 400px) {
    .chip span {
      display: none;
    }
    .chip {
      padding: 0 9px;
    }
  }
  .block header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }
  .block h2 {
    font-size: 17px;
    font-weight: 650;
  }
  .desc {
    margin-top: 6px;
    color: var(--text-2);
    font-size: var(--text-md);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-top: auto;
    padding-top: 26px;
  }
  .actions :global(.seg) {
    min-width: 0;
  }
  .setup-body {
    grid-column: 1 / -1;
    padding: 18px 24px 22px;
    box-shadow: inset 0 1px 0 var(--border);
  }

  /* Keyboard shortcut chip: "⌥ ⇧ ⌘ 4" */
  .keys {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 2px 7px;
    border-radius: 6px;
    background: var(--kbd-bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-2);
    font: 500 11px/18px var(--font-ui);
    white-space: nowrap;
    vertical-align: 1px;
  }

  /* Buttons */
  .btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    height: 36px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--radius-md);
    font-weight: 600;
    white-space: nowrap;
    flex-shrink: 0;
    transition:
      background var(--dur-fast),
      transform var(--dur-fast);
  }
  .btn.sm {
    height: 32px;
    padding: 0 13px;
    font-size: var(--text-sm);
  }
  .btn:active:not(:disabled) {
    transform: scale(0.98);
  }
  .btn:disabled {
    opacity: 0.45;
  }
  .primary {
    background: var(--primary-fill);
    box-shadow: inset 0 0 0 1px var(--primary-border);
    color: var(--accent-text);
  }
  .primary :global(svg) {
    color: var(--primary-icon);
  }
  .primary:hover:not(:disabled) {
    background: var(--primary-hover);
  }
  .record {
    background: var(--rec-500);
    color: #fff;
  }
  .record:hover:not(:disabled) {
    background: var(--rec-hover);
  }
  .record .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: currentColor;
  }
  .chips {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
  }
  .chip,
  .ghost {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 11px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-3);
    font-size: var(--text-sm);
    font-weight: 600;
    white-space: nowrap;
    transition:
      background var(--dur-fast),
      color var(--dur-fast),
      box-shadow var(--dur-fast);
  }
  .chip {
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .chip[aria-pressed='true'] {
    background: var(--bg);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .chip[aria-pressed='false'] span {
    text-decoration: line-through;
    text-decoration-color: var(--text-3);
  }
  .chip:hover,
  .ghost:hover,
  .ghost.on {
    background: var(--surface-hover);
    color: var(--text);
  }
  .ghost {
    color: var(--text-2);
  }

  .rec-live {
    margin-top: var(--space-4);
  }

  /* Recents */
  .recent {
    margin-top: 34px;
  }
  .recent header {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 14px;
  }
  .recent h2 {
    font-size: var(--text-lg);
    font-weight: 650;
  }
  .count {
    color: var(--text-3);
    font-size: var(--text-sm);
    font-variant-numeric: tabular-nums;
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    margin-left: auto;
    border: 0;
    background: none;
    color: var(--link);
    font-size: var(--text-md);
    font-weight: 550;
  }
  .link:hover {
    color: var(--link-hover);
  }
  .grid {
    display: grid;
    gap: 14px;
  }
  .grid.images {
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  }
  .grid.videos {
    grid-template-columns: repeat(auto-fill, minmax(170px, 1fr));
  }
  .empty {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 18px 20px;
    border-radius: 14px;
    background: var(--surface);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    border-radius: 10px;
    flex-shrink: 0;
    background: var(--icon-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  .empty-icon.rec-tone {
    background: var(--rec-tile);
    color: var(--rec-icon);
  }
  .empty-text {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }
  .empty-title {
    font-weight: 600;
    font-size: var(--text-md);
  }
  .empty-body {
    color: var(--text-2);
    font-size: var(--text-sm);
  }

  @media (max-width: 1000px) {
    .studio {
      grid-template-columns: 1fr;
    }
    .rec {
      box-shadow: inset 0 1px 0 var(--border);
    }
  }
  @media (max-width: 560px) {
    .actions,
    .empty {
      flex-wrap: wrap;
    }
  }
</style>
