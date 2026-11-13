import { commands, isAppError } from '$lib/ipc/commands';
import { logUi } from '$lib/ipc/log';
import { t, type MessageKey } from '$lib/i18n/index.svelte';
import { toast } from '$lib/stores/toasts.svelte';
import type { LibraryItem } from '$lib/ipc/types';
import type { MenuItem } from '$lib/components/ContextMenu.svelte';
import { Copy, ExternalLink, FolderOpen, Pencil, PenLine, Trash2 } from '@lucide/svelte';

function fail(e: unknown) {
  logUi('warn', `library action failed: ${isAppError(e) ? `${e.code}: ${e.message}` : String(e)}`);
  toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
}

/** Library item actions with consistent feedback, shared by the Library page and Home. */
export const itemActions = {
  async open(item: LibraryItem) {
    try {
      await commands.libraryOpen(item.id);
    } catch (e) {
      fail(e);
    }
  },
  async edit(item: LibraryItem) {
    try {
      await commands.editorOpen(item.path);
    } catch (e) {
      fail(e);
    }
  },
  async reveal(item: LibraryItem) {
    try {
      await commands.libraryReveal(item.id);
    } catch (e) {
      fail(e);
    }
  },
  async copy(item: LibraryItem) {
    try {
      await commands.libraryCopy(item.id);
      toast('success', t('library.copied'));
    } catch (e) {
      fail(e);
    }
  },
  async rename(item: LibraryItem, name: string): Promise<LibraryItem | null> {
    try {
      const updated = await commands.libraryRename(item.id, name);
      toast('success', t('library.renamed'));
      return updated;
    } catch (e) {
      fail(e);
      return null;
    }
  },
  async trash(item: LibraryItem): Promise<boolean> {
    try {
      await commands.libraryTrash(item.id);
      toast('success', t('library.trashed'));
      return true;
    } catch (e) {
      fail(e);
      return false;
    }
  },
};

/**
 * The item menu (right-click or the card's "⋯" button), the same on Home and in the Library.
 * Renaming and trashing need the page's viewer and confirmation dialog, so the page supplies them.
 */
export function itemMenu(item: LibraryItem, page: { rename: (item: LibraryItem) => void; trash: (item: LibraryItem) => void }): MenuItem[] {
  return [
    ...(item.kind === 'image' ? [{ label: t('editor.edit'), icon: PenLine, run: () => itemActions.edit(item) }] : []),
    { label: t('library.open'), icon: ExternalLink, run: () => itemActions.open(item) },
    { label: t('library.show'), icon: FolderOpen, run: () => itemActions.reveal(item) },
    { label: t('library.copy'), icon: Copy, run: () => itemActions.copy(item) },
    { label: t('library.rename'), icon: Pencil, run: () => page.rename(item) },
    { label: t('library.trash'), icon: Trash2, danger: true, run: () => page.trash(item) },
  ];
}
