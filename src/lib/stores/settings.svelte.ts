import { commands, events, isAppError } from '$lib/ipc/commands';
import type { AppError, Settings } from '$lib/ipc/types';
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

  /** Takes settings the backend already saved (commands that change them as a side effect). */
  adopt(s: Settings): void {
    this.apply(s);
  }

  private apply(s: Settings): void {
    this.value = s;
    setLanguage(s.general.language);
    applyAppearance(s.appearance);
  }
}

/** Theme, motion and ambient light as attributes on <html>, read by the design tokens. */
export function applyAppearance(a: Settings['appearance']): void {
  const root = document.documentElement;
  root.dataset.theme = a.theme;
  root.dataset.motion = a.motion;
  root.dataset.ambient = a.ambientLight ? 'on' : 'off';
}

export const settings = new SettingsStore();
