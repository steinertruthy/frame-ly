import { commands, events, isAppError } from '$lib/ipc/commands';
import type { AppError, Settings, Theme } from '$lib/ipc/types';
import { setLanguage, t, type MessageKey } from '$lib/i18n/index.svelte';
import { toast } from './toasts.svelte';

class SettingsStore {
  value = $state<Settings | null>(null);

  async init(): Promise<void> {
    this.apply(await commands.getSettings());
    // Other windows (tray panel, editor…) may change settings; stay in sync.
    await events.onSettingsChanged((s) => this.apply(s));
  }

  /** Applies a change through the backend; the UI only reflects what was actually saved. */
  /** `describe` can turn a backend error into a specific, translated message. */
  async update(mutate: (draft: Settings) => void, describe?: (e: AppError) => string | null): Promise<boolean> {
    if (!this.value) return false;
    const draft = $state.snapshot(this.value) as Settings;
    mutate(draft);
    try {
      this.apply(await commands.updateSettings(draft));
      return true;
    } catch (e) {
      const specific = isAppError(e) && describe ? describe(e) : null;
      const key = (isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey;
      toast('error', specific ?? t(key) ?? t('error.generic'));
      console.error('update_settings failed', e);
      return false;
    }
  }

  private apply(s: Settings): void {
    this.value = s;
    setLanguage(s.general.language);
    applyTheme(s.appearance.theme);
  }
}

function applyTheme(theme: Theme): void {
  const root = document.documentElement;
  if (theme === 'system') delete root.dataset.theme;
  else root.dataset.theme = theme;
}

export const settings = new SettingsStore();
