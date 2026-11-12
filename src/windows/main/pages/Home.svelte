<script lang="ts">
  import { onMount } from 'svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import Viewer from '$lib/components/library/Viewer.svelte';
  import StorageCard from '$lib/components/library/StorageCard.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import type { AppInfo } from '$lib/ipc/types';
  import { system } from '$lib/stores/system.svelte';
  import PermissionRow from '$lib/components/PermissionRow.svelte';
  import CaptureActions from '$lib/components/CaptureActions.svelte';
  import RecordActions from '$lib/components/RecordActions.svelte';
  import RecordingBanner from '$lib/components/RecordingBanner.svelte';
  import AudioBar from '$lib/components/AudioBar.svelte';
  import { recording } from '$lib/stores/recording.svelte';

  let { info, onnavigate }: { info: AppInfo; onnavigate: (page: 'library') => void } = $props();
  let viewing = $state<number | null>(null);
  onMount(() => void library.init());
</script>

<header class="hero">
  <div class="glow" aria-hidden="true"></div>
  <h1>{t('home.greeting')}</h1>
  <p>{t('home.subtitle')}</p>
</header>

{#if system.permissions && !system.screenReady}
  <section class="setup">
    <div class="setup-head">
      <h2>{t('perm.title')}</h2>
      <p>{t('perm.subtitle')}</p>
    </div>
    <div class="card">
      <PermissionRow permission="screen" status={system.permissions.screen} platform={info.platform} />
      <PermissionRow permission="microphone" status={system.permissions.microphone} platform={info.platform} />
    </div>
  </section>
{/if}

{#if info.supportsRecording}
  <h2 class="label">{t('record.title')}</h2>
  {#if recording.active}
    <RecordingBanner />
  {:else}
    <RecordActions disabled={!system.screenReady} />
    <AudioBar />
  {/if}
{/if}

<h2 class="label">{t('capture.title')}</h2>
<CaptureActions disabled={!system.screenReady || recording.active} />

{#if library.recent.length}
  <div class="label-row">
    <h2 class="label">{t('home.recent')}</h2>
    <button class="see-all" onclick={() => onnavigate('library')}>{t('home.seeAll')}</button>
  </div>
  <div class="recent">
    {#each library.recent as item (item.id)}
      <MediaTile {item} onopen={(i) => (viewing = i.id)} />
    {/each}
  </div>
{/if}

{#if library.overview}
  <h2 class="label">{t('home.storage')}</h2>
  <StorageCard overview={library.overview} />
{/if}

{#if viewing !== null}
  <Viewer items={library.recent} bind:id={viewing} onclose={() => (viewing = null)} />
{/if}

<p class="foot">FrameLy {info.version} · {info.platform} {info.arch}</p>

<style>
  .hero {
    position: relative;
    overflow: hidden;
    padding: 22px 28px;
    border-radius: var(--radius-xl);
    background: var(--hero);
    color: #fff;
    box-shadow: var(--shadow-md);
  }
  .glow {
    position: absolute;
    right: -80px;
    top: -120px;
    width: 340px;
    height: 340px;
    border-radius: 50%;
    background: radial-gradient(circle, rgba(91, 140, 255, 0.55), transparent 65%);
    pointer-events: none;
  }
  .hero h1 {
    position: relative;
    font-size: var(--text-xl);
    font-weight: 700;
  }
  .hero p {
    position: relative;
    margin-top: 6px;
    color: rgba(233, 238, 252, 0.78);
    font-size: var(--text-md);
  }
  .setup {
    margin-top: var(--space-5);
    animation: rise var(--dur-slow) var(--ease-out);
  }
  .setup-head {
    margin: 0 0 var(--space-3) var(--space-1);
  }
  .setup-head h2 {
    font-size: var(--text-lg);
    font-weight: 650;
  }
  .setup-head p {
    color: var(--text-2);
    margin-top: 2px;
  }
  .card {
    background: var(--surface);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-sm);
    overflow: hidden;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px);
    }
  }
  .label {
    margin: var(--space-6) 0 var(--space-3) var(--space-1);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .label-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }
  .see-all {
    border: 0;
    background: none;
    color: var(--accent);
    font-size: var(--text-sm);
    font-weight: 600;
  }
  .recent {
    display: grid;
    grid-template-columns: repeat(6, minmax(0, 1fr));
    gap: var(--space-3);
  }
  .foot {
    margin-top: var(--space-6);
    color: var(--text-3);
    font-size: var(--text-xs);
  }
</style>
