/** Main window navigation, shared so any page can send the user elsewhere (e.g. the library
 * viewer's "Transcribe" opens the Transcriptions page with that file). */
export type Page = 'home' | 'library' | 'transcripts' | 'settings';
export type TranscriptTab = 'transcript' | 'summary' | 'topics';

class Nav {
  page = $state<Page>('home');
  /** File the Transcriptions page should open (consumed once). */
  transcribePath = $state<string | null>(null);
  /** Settings category to open (consumed once). */
  settingsCategory = $state<string | null>(null);
  /** Section inside that category to scroll to (consumed once). */
  settingsAnchor = $state<string | null>(null);
  /** Transcript (and tab) the Transcriptions page should open (consumed once). */
  transcriptTarget = $state<{ id: number; tab: TranscriptTab } | null>(null);

  go(page: Page): void {
    this.page = page;
  }

  transcribe(path: string): void {
    this.transcribePath = path;
    this.page = 'transcripts';
  }

  openSettings(category: string): void {
    this.settingsCategory = category;
    this.page = 'settings';
  }

  openTranscript(id: number, tab: TranscriptTab = 'transcript'): void {
    this.transcriptTarget = { id, tab };
    this.page = 'transcripts';
  }

  /** Targets from `framely --show`: a page, `settings:<category>` or `transcript:<id>[:tab]`. */
  show(target: string): void {
    const [kind, arg, sub] = target.split(':');
    if (kind === 'home' || kind === 'library' || kind === 'transcripts') this.go(kind);
    else if (kind === 'settings') {
      this.settingsAnchor = sub ?? null;
      if (arg) this.openSettings(arg);
      else this.go('settings');
    } else if (kind === 'transcript' && arg && /^\d+$/.test(arg)) {
      const tab = sub === 'summary' || sub === 'topics' ? sub : 'transcript';
      this.openTranscript(Number(arg), tab);
    }
  }
}

export const nav = new Nav();
