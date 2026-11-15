/** Main window navigation, shared so any page can send the user elsewhere (e.g. the library
 * viewer's "Transcribe" opens the Transcriptions page with that file). */
export type Page = 'home' | 'library' | 'transcripts' | 'settings';

class Nav {
  page = $state<Page>('home');
  /** File the Transcriptions page should open (consumed once). */
  transcribePath = $state<string | null>(null);
  /** Settings category to open (consumed once). */
  settingsCategory = $state<string | null>(null);

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
}

export const nav = new Nav();
