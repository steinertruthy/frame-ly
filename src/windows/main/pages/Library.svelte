<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, ExternalLink, FolderOpen, Copy, Pencil, Trash2, Images, PenLine } from '@lucide/svelte';
  import Select from '$lib/components/Select.svelte';
  import ContextMenu, { type MenuItem } from '$lib/components/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import Viewer from '$lib/components/library/Viewer.svelte';
  import { itemActions } from '$lib/components/library/actions';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { groupByDate } from '$lib/library';
  import type { LibraryItem, LibrarySort } from '$lib/ipc/types';

  let viewing = $state<{ id: number; focusName: boolean } | null>(null);
  let menu = $state<{ x: number; y: number; item: LibraryItem } | null>(null);
  let confirmTrash = $state<LibraryItem | null>(null);
  let searchInput: HTMLInputElement | undefined = $state();

  onMount(() => {
    void library.init();
    library.sync();
  });

  const byDate = $derived(library.query.sort === 'newest' || library.query.sort === 'oldest');
  const groups = $derived(byDate ? groupByDate(library.items, new Date()) : [{ id: 'all', key: 'all' as const, items: library.items }]);
  const monthFmt = $derived(new Intl.DateTimeFormat(locale(), { month: 'long', year: 'numeric' }));

  function groupLabel(g: (typeof groups)[number]): string {
    if (g.key === 'all') return '';
    if (g.key === 'month' && 'month' in g && g.month) {
      const s = monthFmt.format(g.month);
      return s.charAt(0).toUpperCase() + s.slice(1);
    }
    return t(`library.group.${g.key}` as MessageKey);
  }

  function menuItems(item: LibraryItem): MenuItem[] {
    return [
      ...(item.kind === 'image' ? [{ label: t('editor.edit'), icon: PenLine, run: () => itemActions.edit(item) }] : []),
      { label: t('library.open'), icon: ExternalLink, run: () => itemActions.open(item) },
      { label: t('library.show'), icon: FolderOpen, run: () => itemActions.reveal(item) },
      { label: t('library.copy'), icon: Copy, run: () => itemActions.copy(item) },
      { label: t('library.rename'), icon: Pencil, run: () => (viewing = { id: item.id, focusName: true }) },
      { label: t('library.trash'), icon: Trash2, danger: true, run: () => (confirmTrash = item) },
    ];
  }

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
      e.preventDefault();
      searchInput?.focus();
    }
  }
</script>

<svelte:window {onkeydown} />

<header class="head">
  <h1>{t('library.title')}</h1>
  <div class="tabs" role="tablist">
    {#each [['recent', 'library.tab.recent'], ['image', 'library.tab.images'], ['video', 'library.tab.videos']] as const as [id, label] (id)}
      {@const n = library.overview ? (id === 'image' ? library.overview.images : id === 'video' ? library.overview.videos : null) : null}
      <button role="tab" aria-selected={library.tab === id} class:active={library.tab === id} onclick={() => library.showTab(id)}>
        {t(label)}{#if n != null}<span class="n">{n}</span>{/if}
      </button>
    {/each}
  </div>
  <div class="tools">
    <label class="search">
      <Search size={14} />
      <input
        bind:this={searchInput}
        type="search"
        placeholder={t('library.search')}
        value={library.query.search}
        oninput={(e) => library.setQuery({ search: e.currentTarget.value })}
        spellcheck="false"
      />
    </label>
    <Select
      label={t('library.sort.newest')}
      value={library.query.sort}
      options={(['newest', 'oldest', 'name', 'size'] as LibrarySort[]).map((s) => ({ value: s, label: t(`library.sort.${s}`) }))}
      onchange={(v) => library.setQuery({ sort: v })}
    />
  </div>
</header>

{#if library.loaded && library.items.length === 0}
  <div class="empty">
    <span class="empty-icon"><Images size={28} /></span>
    {#if library.query.search}
      <p>{t('library.noResults', { q: library.query.search })}</p>
    {:else}
      <h2>{t(library.tab === 'video' ? 'library.empty.videos.title' : library.tab === 'image' ? 'library.empty.images.title' : 'library.empty.title')}</h2>
      <p>{t(library.tab === 'video' ? 'library.empty.videos.body' : library.tab === 'image' ? 'library.empty.images.body' : 'library.empty.body')}</p>
    {/if}
  </div>
{/if}

{#each groups as group (group.id)}
  <section>
    {#if groupLabel(group)}<h2 class="group">{groupLabel(group)}</h2>{/if}
    <div class="grid">
      {#each group.items as item (item.id)}
        <MediaTile
          {item}
          card
          onopen={(i) => (viewing = { id: i.id, focusName: false })}
          oncontext={(i, e) => (menu = { x: e.clientX, y: e.clientY, item: i })}
        />
      {/each}
    </div>
  </section>
{/each}

{#if menu}
  <ContextMenu x={menu.x} y={menu.y} items={menuItems(menu.item)} onclose={() => (menu = null)} />
{/if}

{#if confirmTrash}
  {@const item = confirmTrash}
  <ConfirmDialog
    title={t('library.trash.confirm', { name: item.name })}
    body={t('library.trash.body')}
    confirm={t('library.trash')}
    cancel={t('library.cancel')}
    danger
    onconfirm={() => {
      // `item` is derived from `confirmTrash`: read it before clearing, or it becomes null.
      const target = item;
      confirmTrash = null;
      void itemActions.trash(target);
    }}
    oncancel={() => (confirmTrash = null)}
  />
{/if}

{#if viewing}
  <Viewer items={library.items} bind:id={viewing.id} focusName={viewing.focusName} onclose={() => (viewing = null)} />
{/if}

<style>
  .head {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--space-5);
    margin-bottom: var(--space-5);
  }
  h1 {
    font-size: 22px;
    font-weight: 650;
  }
  .tabs {
    display: flex;
    gap: 4px;
  }
  .tabs button {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-2);
    font-weight: 600;
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .tabs button:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .tabs button.active {
    background: var(--nav-active);
    color: var(--text);
    box-shadow: inset 0 0 0 1px var(--nav-active-border);
  }
  .n {
    color: var(--text-3);
    font-size: var(--text-sm);
    font-weight: 550;
    font-variant-numeric: tabular-nums;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .tools :global(select) {
    min-width: 0;
    width: 160px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
    transition: box-shadow var(--dur-fast);
  }
  .search:hover {
    box-shadow: inset 0 0 0 1px var(--border-strong);
  }
  .search:focus-within {
    box-shadow: var(--focus-ring);
  }
  .search input {
    width: 180px;
    border: 0;
    background: none;
    color: var(--text);
    font-size: var(--text-sm);
    user-select: text;
  }
  .search input:focus {
    outline: none;
    box-shadow: none;
  }
  @media (max-width: 980px) {
    .head {
      grid-template-columns: 1fr auto;
    }
    .tools {
      grid-column: 1 / -1;
    }
    .search {
      flex: 1;
    }
    .search input {
      width: 100%;
    }
  }
  section + section {
    margin-top: var(--space-5);
  }
  .group {
    margin: 0 0 var(--space-3);
    font-size: var(--text-xs);
    font-weight: 650;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-3);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: 14px;
  }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    padding: 80px 20px;
    color: var(--text-2);
    text-align: center;
  }
  .empty-icon {
    display: grid;
    place-items: center;
    width: 60px;
    height: 60px;
    margin-bottom: 8px;
    border-radius: 16px;
    background: var(--icon-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  .empty h2 {
    color: var(--text);
    font-size: var(--text-lg);
  }
</style>
