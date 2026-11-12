import '../../app.css';
import { mount, tick } from 'svelte';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Editor from './Editor.svelte';
import { commands } from '$lib/ipc/commands';
import { installErrorForwarding, logUi } from '$lib/ipc/log';
import { settings } from '$lib/stores/settings.svelte';

installErrorForwarding();

async function boot() {
  const [info] = await Promise.all([commands.appInfo(), settings.init()]);
  document.documentElement.classList.add(`platform-${info.platform}`);
  mount(Editor, { target: document.getElementById('app')!, props: { platform: info.platform } });
  await tick();
  await getCurrentWindow().show();
}

boot().catch((e) => {
  logUi('error', `editor failed to start: ${e instanceof Error ? e.stack : String(e)}`);
  void getCurrentWindow().show();
});
