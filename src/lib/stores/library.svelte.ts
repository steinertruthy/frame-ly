import { commands, events } from '$lib/ipc/commands';
import type { LibraryItem, LibraryOverview, LibraryQuery } from '$lib/ipc/types';
import { logUi } from '$lib/ipc/log';

/** Library contents + overview, kept fresh by `library://changed` events. */
class LibraryStore {
  query = $state<LibraryQuery>({ search: '', filter: 'all', sort: 'newest' });
  items = $state<LibraryItem[]>([]);
  recent = $state<LibraryItem[]>([]);
  recentImages = $state<LibraryItem[]>([]);
  recentVideos = $state<LibraryItem[]>([]);
  /** Which library tab to show (Home's "See all" links set it). */
  tab = $state<'recent' | 'image' | 'video'>('recent');
  overview = $state<LibraryOverview | null>(null);
  loaded = $state(false);
  private timer: ReturnType<typeof setTimeout> | undefined;
  private started = false;

  async init(): Promise<void> {
    if (this.started) return;
    this.started = true;
    await events.onLibraryChanged(() => this.scheduleReload());
    await this.reload();
  }

  /** Coalesces bursts of changes (e.g. a folder sync) into one reload. */
  scheduleReload(): void {
    clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.reload(), 120);
  }

  async reload(): Promise<void> {
    try {
      const [items, recent, recentImages, recentVideos, overview] = await Promise.all([
        commands.libraryList($state.snapshot(this.query)),
        commands.libraryList({ search: '', filter: 'all', sort: 'newest', limit: 4 }),
        commands.libraryList({ search: '', filter: 'image', sort: 'newest', limit: 8 }),
        commands.libraryList({ search: '', filter: 'video', sort: 'newest', limit: 6 }),
        commands.libraryOverview(),
      ]);
      this.items = items;
      this.recent = recent;
      this.recentImages = recentImages;
      this.recentVideos = recentVideos;
      this.overview = overview;
      this.loaded = true;
    } catch (e) {
      logUi('warn', `library reload failed: ${JSON.stringify(e)}`);
    }
  }

  showTab(tab: 'recent' | 'image' | 'video'): void {
    this.tab = tab;
    this.setQuery({ filter: tab === 'recent' ? 'all' : tab });
  }

  setQuery(patch: Partial<LibraryQuery>): void {
    this.query = { ...this.query, ...patch };
    this.scheduleReload();
  }

  /** Picks up files added or removed outside FrameLy. */
  sync(): void {
    void commands.libraryRefresh();
  }
}

export const library = new LibraryStore();
