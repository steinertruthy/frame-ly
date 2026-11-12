import '../../app.css';
import { mount, tick } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from './App.svelte';
import { commands } from '$lib/ipc/commands';
import { settings } from '$lib/stores/settings.svelte';
import { system } from '$lib/stores/system.svelte';
import { recording } from '$lib/stores/recording.svelte';
import { installErrorForwarding, logUi } from '$lib/ipc/log';
import { events } from '$lib/ipc/commands';
import { toast } from '$lib/stores/toasts.svelte';
import { t, type MessageKey } from '$lib/i18n/index.svelte';

installErrorForwarding();

async function boot() {
  const [info] = await Promise.all([commands.appInfo(), settings.init(), system.init(), recording.init()]);
  document.documentElement.classList.add(`platform-${info.platform}`);
  mount(App, { target: document.getElementById('app')!, props: { info } });
  // Show only once the DOM is rendered: no blank flash on launch. (rAF would never fire
  // here: WebKit doesn't schedule frames for a hidden window.)
  // Captures started from the tray report failures here (e.g. missing permission).
  await events.onCaptureFailed((e) => {
    toast('error', t(`error.${e.code}` as MessageKey) ?? t('error.generic'));
    void system.refresh();
  });
  await events.onRecordingFailed((e) => toast('error', t(`error.${e.code}` as MessageKey) ?? t('error.generic')));
  await events.onRecordingWarning((code) => toast('info', t(`audio.warning.${code}`), 6000));
  const recovered = await commands.recoveredRecordings();
  if (recovered.length) toast('info', t('record.recovered'), 6000);
  await tick();
  await getCurrentWindow().show();
  logUi('info', `main window ready in ${Math.round(performance.now())} ms`);
}

boot().catch((e) => {
  logUi('error', `UI failed to start: ${e instanceof Error ? e.stack : String(e)}`);
  void getCurrentWindow().show();
});
