import { commands, isAppError } from '$lib/ipc/commands';
import { t, type MessageKey } from '$lib/i18n/index.svelte';
import { toast } from '$lib/stores/toasts.svelte';
import type { LibraryItem } from '$lib/ipc/types';

function fail(e: unknown) {
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
