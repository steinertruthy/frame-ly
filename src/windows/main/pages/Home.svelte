<script lang="ts">
  import { onMount } from 'svelte';
  import { slide } from 'svelte/transition';
  import { Scan, Circle, SlidersHorizontal, ArrowRight } from '@lucide/svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import Viewer from '$lib/components/library/Viewer.svelte';
  import PermissionRow from '$lib/components/PermissionRow.svelte';
  import RecordSetup from '$lib/components/RecordSetup.svelte';
  import RecordingBanner from '$lib/components/RecordingBanner.svelte';
  import { commands, isAppError } from '$lib/ipc/commands';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { recording } from '$lib/stores/recording.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { system } from '$lib/stores/system.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { audioMode, recordRequest, summaryKeys } from '$lib/recordSetup';
  import { formatAccelerator } from '$lib/shortcuts';
  import { recordPrefs } from '$lib/stores/recordPrefs.svelte';
  import type { AppInfo, LibraryItem } from '$lib/ipc/types';

  let { info, onnavigate }: { info: AppInfo; onnavigate: (page: 'library' | 'settings') => void } = $props();
  let viewing = $state<{ items: LibraryItem[]; id: number } | null>(null);
  let customizing = $state(false);

  onMount(() => void library.init());

  const s = $derived(settings.value);
  const shotKeys = $derived(formatAccelerator(s?.shortcuts.captureRegion ?? '', info.platform));
  const summary = $derived(s ? summaryKeys(s.recording.target, audioMode(s.audio)) : null);

  function fail(e: unknown) {
    toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
  }

  async function screenshot() {
    try {
      await commands.takeScreenshot({ kind: 'select', mode: 'region' });
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

  function seeAll(tab: 'image' | 'video') {
    library.showTab(tab);
    onnavigate('library');
  }
</script>

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

<!-- Primary: screenshot -->
<button class="shot" disabled={!system.screenReady || recording.active} onclick={screenshot}>
  <span class="shot-light" aria-hidden="true"></span>
  <span class="shot-text">
    <span class="shot-title">{t('home.shot.title')}</span>
    <span class="shot-desc">{t('home.shot.desc')}</span>
    {#if shotKeys.length}
      <span class="keys">{#each shotKeys as k, i (i)}<kbd>{k}</kbd>{/each}</span>
    {/if}
  </span>
  <span class="lens" aria-hidden="true"><Scan size={34} strokeWidth={1.6} /></span>
</button>

<!-- Secondary: recording -->
{#if info.supportsRecording}
  {#if recording.active}
    <div class="rec-live"><RecordingBanner /></div>
  {:else}
    <section class="rec">
      <div class="rec-head">
        <span class="rec-icon" aria-hidden="true"><Circle size={14} fill="currentColor" /></span>
        <div class="rec-text">
          <span class="rec-title">{t('home.rec.title')}</span>
          {#if summary}<span class="rec-summary">{t(summary[0])} · {t(summary[1])}</span>{/if}
        </div>
        <button class="ghost-btn" class:on={customizing} onclick={() => (customizing = !customizing)} aria-expanded={customizing}>
          <SlidersHorizontal size={14} /><span>{t('home.rec.customize')}</span>
        </button>
        <button class="rec-start" disabled={!system.screenReady} onclick={record}>{t('home.rec.start')}</button>
      </div>
      {#if customizing}
        <div class="rec-body" transition:slide={{ duration: 240 }}>
          <RecordSetup />
        </div>
      {/if}
    </section>
  {/if}
{/if}

<!-- Recents -->
<section class="recent">
  <header>
    <h2>{t('home.recent.shots')}</h2>
    {#if library.recentImages.length}<button class="link" onclick={() => seeAll('image')}>{t('home.seeAll')}<ArrowRight size={13} /></button>{/if}
  </header>
  {#if library.recentImages.length}
    <div class="strip images">
      {#each library.recentImages as item (item.id)}
        <MediaTile {item} compact onopen={(i) => (viewing = { items: library.recentImages, id: i.id })} />
      {/each}
    </div>
  {:else if library.loaded}
    <p class="empty">{t('home.recent.shots.empty')}</p>
  {/if}
</section>

{#if info.supportsRecording}
  <section class="recent">
    <header>
      <h2>{t('home.recent.videos')}</h2>
      {#if library.recentVideos.length}<button class="link" onclick={() => seeAll('video')}>{t('home.seeAll')}<ArrowRight size={13} /></button>{/if}
    </header>
    {#if library.recentVideos.length}
      <div class="strip videos">
        {#each library.recentVideos as item (item.id)}
          <MediaTile {item} compact onopen={(i) => (viewing = { items: library.recentVideos, id: i.id })} />
        {/each}
      </div>
    {:else if library.loaded}
      <p class="empty">{t('home.recent.videos.empty')}</p>
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

  /* Screenshot: the primary action */
  .shot {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    min-height: 168px;
    padding: 28px 32px;
    border: 0;
    border-radius: var(--radius-xl);
    overflow: hidden;
    background: var(--hero);
    color: #f1f4fa;
    text-align: left;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.08),
      inset 0 0 0 1px rgba(108, 157, 230, 0.16),
      0 24px 60px -30px rgba(57, 120, 215, 0.8);
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-slow) var(--ease-out);
  }
  .shot:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.1),
      inset 0 0 0 1px rgba(108, 157, 230, 0.28),
      0 30px 70px -28px rgba(57, 120, 215, 0.95);
  }
  .shot:active:not(:disabled) {
    transform: translateY(0) scale(0.995);
  }
  .shot:disabled {
    opacity: 0.55;
  }
  .shot-light {
    position: absolute;
    right: -10%;
    top: -60%;
    width: 70%;
    height: 220%;
    background: radial-gradient(closest-side, rgba(108, 157, 230, 0.5), transparent 70%);
    filter: blur(20px);
    transform: rotate(-12deg);
    transition: transform 1.2s var(--ease-out);
    pointer-events: none;
  }
  .shot:hover .shot-light {
    transform: rotate(-4deg) translateX(-6%);
  }
  .shot-text {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-width: 70%;
  }
  .shot-title {
    font: 700 var(--text-3xl) / 1.1 var(--font-display);
    letter-spacing: -0.025em;
  }
  .shot-desc {
    color: rgba(241, 244, 250, 0.72);
    font-size: var(--text-lg);
  }
  .keys {
    display: flex;
    gap: 4px;
    margin-top: 10px;
  }
  .keys kbd {
    min-width: 24px;
    padding: 2px 7px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.1);
    box-shadow: inset 0 -1px 0 rgba(0, 0, 0, 0.25);
    color: rgba(241, 244, 250, 0.9);
    font-size: 12px;
    text-align: center;
  }
  .lens {
    position: relative;
    display: grid;
    place-items: center;
    width: 84px;
    height: 84px;
    border-radius: 50%;
    background: radial-gradient(circle at 35% 30%, rgba(169, 198, 242, 0.35), rgba(57, 120, 215, 0.2) 55%, rgba(12, 20, 35, 0.2));
    box-shadow:
      inset 0 0 0 1px rgba(169, 198, 242, 0.3),
      0 0 40px -6px rgba(57, 120, 215, 0.8);
    flex-shrink: 0;
  }

  /* Recording: secondary, calmer */
  .rec,
  .rec-live {
    margin-top: var(--space-4);
  }
  .rec {
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
  }
  .rec-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 16px;
  }
  .rec-icon {
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: color-mix(in srgb, var(--rec-500) 14%, transparent);
    color: var(--rec-500);
    flex-shrink: 0;
  }
  .rec-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }
  .rec-title {
    font-weight: 650;
    font-size: var(--text-lg);
  }
  .rec-summary {
    color: var(--text-2);
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ghost-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-2);
    font-weight: 600;
    white-space: nowrap;
  }
  .ghost-btn:hover,
  .ghost-btn.on {
    background: var(--surface-hover);
    color: var(--text);
  }
  .rec-start {
    height: 34px;
    padding: 0 16px;
    border: 0;
    border-radius: var(--radius-md);
    background: var(--rec-500);
    color: #fff;
    font-weight: 650;
    white-space: nowrap;
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.22),
      0 6px 18px -8px rgba(255, 84, 104, 0.7);
    transition:
      transform var(--dur-fast),
      filter var(--dur-fast);
  }
  .rec-start:hover:not(:disabled) {
    filter: brightness(1.07);
  }
  .rec-start:active:not(:disabled) {
    transform: scale(0.97);
  }
  .rec-start:disabled {
    opacity: 0.5;
  }
  .rec-body {
    padding: 4px 16px 16px 62px;
  }

  /* Recents */
  .recent {
    margin-top: var(--space-6);
  }
  .recent header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--space-3);
  }
  .recent h2 {
    font-size: var(--text-lg);
    font-weight: 650;
  }
  .link {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    border: 0;
    background: none;
    color: var(--accent);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .link:hover {
    color: var(--accent-hover);
  }
  .strip {
    display: grid;
    gap: var(--space-3);
  }
  .strip.images {
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  }
  .strip.videos {
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  }
  .empty {
    padding: 18px;
    border-radius: var(--radius-lg);
    background: color-mix(in srgb, var(--surface) 55%, transparent);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  @media (max-width: 760px) {
    .shot {
      padding: 22px;
    }
    .lens {
      display: none;
    }
    .shot-text {
      max-width: 100%;
    }
    .rec-head {
      flex-wrap: wrap;
    }
    .rec-body {
      padding-left: 16px;
    }
  }
</style>
