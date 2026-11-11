<script lang="ts">
  import { Film, Image, FolderOpen } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { commands } from '$lib/ipc/commands';
  import type { AppInfo, Folder } from '$lib/ipc/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';

  let { info }: { info: AppInfo } = $props();
  const storage = $derived(settings.value?.storage);

  async function open(folder: Folder) {
    try {
      await commands.openFolder(folder);
    } catch (e) {
      console.error(e);
      toast('error', t('error.io'));
    }
  }
</script>

<header class="hero">
  <div class="glow" aria-hidden="true"></div>
  <h1>{t('home.greeting')}</h1>
  <p>{t('home.subtitle')}</p>
</header>

{#if storage}
  <h2 class="label">{t('home.folders')}</h2>
  <div class="grid">
    {#each [{ id: 'recordings' as const, icon: Film, title: t('home.folders.recordings'), path: storage.recordingsDir }, { id: 'screenshots' as const, icon: Image, title: t('home.folders.screenshots'), path: storage.screenshotsDir }] as f (f.id)}
      <article class="folder">
        <div class="icon"><f.icon size={18} /></div>
        <div class="meta">
          <div class="name">{f.title}</div>
          <div class="path" title={f.path}><span>{f.path}</span></div>
        </div>
        <Button size="sm" onclick={() => open(f.id)}><FolderOpen size={14} />{t('home.open')}</Button>
      </article>
    {/each}
  </div>
{/if}

<p class="foot">FrameLy {info.version} · {info.platform} {info.arch}</p>

<style>
  .hero {
    position: relative;
    overflow: hidden;
    padding: 36px 32px;
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
    font-size: var(--text-2xl);
    font-weight: 700;
  }
  .hero p {
    position: relative;
    margin-top: 6px;
    color: rgba(233, 238, 252, 0.78);
    font-size: var(--text-lg);
  }
  .label {
    margin: var(--space-6) 0 var(--space-3) var(--space-1);
    font-size: var(--text-sm);
    font-weight: 600;
    color: var(--text-3);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--space-3);
  }
  .folder {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-4);
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out);
  }
  .folder:hover {
    transform: translateY(-1px);
    box-shadow: var(--shadow-md);
  }
  .icon {
    display: grid;
    place-items: center;
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--accent);
    flex-shrink: 0;
  }
  .meta {
    flex: 1;
    min-width: 0;
  }
  .name {
    font-weight: 600;
  }
  .path {
    color: var(--text-3);
    font-size: var(--text-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    /* Truncate from the left so the meaningful end of the path stays visible. */
    direction: rtl;
    text-align: left;
  }
  .path span {
    direction: ltr;
    unicode-bidi: isolate;
  }
  .foot {
    margin-top: var(--space-6);
    color: var(--text-3);
    font-size: var(--text-xs);
  }
</style>
