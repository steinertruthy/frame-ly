import '../../app.css';
import { mount } from 'svelte';
import Panel from './Panel.svelte';
import { commands } from '$lib/ipc/commands';
import { installErrorForwarding } from '$lib/ipc/log';
import { settings } from '$lib/stores/settings.svelte';
import { system } from '$lib/stores/system.svelte';
import { recording } from '$lib/stores/recording.svelte';

installErrorForwarding();

async function boot() {
  const [info] = await Promise.all([commands.appInfo(), settings.init(), system.init(), recording.init()]);
  document.documentElement.classList.add(`platform-${info.platform}`);
  mount(Panel, { target: document.getElementById('app')!, props: { info } });
}

void boot();
