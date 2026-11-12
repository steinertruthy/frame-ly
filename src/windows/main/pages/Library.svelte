<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, ExternalLink, FolderOpen, Copy, Pencil, Trash2, Images } from '@lucide/svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import Select from '$lib/components/Select.svelte';
  import ContextMenu, { type MenuItem } from '$lib/components/ContextMenu.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import MediaTile from '$lib/components/library/MediaTile.svelte';
  import Viewer from '$lib/components/library/Viewer.svelte';
  import { itemActions } from '$lib/components/library/actions';
  import { t, locale, type MessageKey } from '$lib/i18n/index.svelte';
  import { library } from '$lib/stores/library.svelte';
  import { groupByDate } from '$lib/library';
  import type { LibraryFilter, LibraryItem, LibrarySort } from '$lib/ipc/types';

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
  <div>
    <h1>{t('library.title')}</h1>
    {#if library.loaded}<p class="count">{t('library.count', { n: library.items.length })}</p>{/if}
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
    <Segmented
      label={t('library.title')}
      value={library.query.filter}
      options={(['all', 'video', 'image'] as LibraryFilter[]).map((f) => ({ value: f, label: t(`library.filter.${f}`) }))}
      onchange={(v) => library.setQuery({ filter: v })}
    />
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
      <h2>{t('library.empty.title')}</h2>
      <p>{t('library.empty.body')}</p>
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
      confirmTrash = null;
      void itemActions.trash(item);
    }}
    oncancel={() => (confirmTrash = null)}
  />
{/if}

{#if viewing}
  <Viewer items={library.items} bind:id={viewing.id} focusName={viewing.focusName} onclose={() => (viewing = null)} />
{/if}

<style>
  .head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-4);
    flex-wrap: wrap;
    margin-bottom: var(--space-5);
  }
  h1 {
    font-size: var(--text-xl);
    font-weight: 700;
  }
  .count {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .tools {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }
  .tools :global(.seg) {
    min-width: 0;
  }
  .tools :global(select) {
    min-width: 150px;
  }
  .search {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
  }
  .search:focus-within {
    box-shadow: var(--focus-ring);
  }
  .search input {
    width: 170px;
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
  section + section {
    margin-top: var(--space-5);
  }
  .group {
    margin: 0 0 var(--space-3) var(--space-1);
    font-size: var(--text-sm);
    font-weight: 650;
    color: var(--text-2);
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(190px, 1fr));
    gap: var(--space-4) var(--space-3);
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
    border-radius: 18px;
    background: var(--accent-soft);
    color: var(--accent);
  }
  .empty h2 {
    color: var(--text);
    font-size: var(--text-lg);
  }
</style>
