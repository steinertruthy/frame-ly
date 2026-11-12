import '../../app.css';
import { mount } from 'svelte';
import RecBar from './RecBar.svelte';
import { installErrorForwarding } from '$lib/ipc/log';
import { settings } from '$lib/stores/settings.svelte';
import { recording } from '$lib/stores/recording.svelte';

installErrorForwarding();

async function boot() {
  await Promise.all([settings.init(), recording.init()]);
  mount(RecBar, { target: document.getElementById('app')! });
}

void boot();
