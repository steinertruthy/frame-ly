import '../../app.css';
import { mount, tick } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import App from './App.svelte';
import { commands } from '$lib/ipc/commands';
import { settings } from '$lib/stores/settings.svelte';
import { system } from '$lib/stores/system.svelte';
import { installErrorForwarding, logUi } from '$lib/ipc/log';

installErrorForwarding();

async function boot() {
  const [info] = await Promise.all([commands.appInfo(), settings.init(), system.init()]);
  document.documentElement.classList.add(`platform-${info.platform}`);
  mount(App, { target: document.getElementById('app')!, props: { info } });
  // Show only once the DOM is rendered: no blank flash on launch. (rAF would never fire
  // here: WebKit doesn't schedule frames for a hidden window.)
  await tick();
  await getCurrentWindow().show();
  logUi('info', `main window ready in ${Math.round(performance.now())} ms`);
}

boot().catch((e) => {
  logUi('error', `UI failed to start: ${e instanceof Error ? e.stack : String(e)}`);
  void getCurrentWindow().show();
});
