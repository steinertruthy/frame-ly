import '../../app.css';
import { mount } from 'svelte';
import Preview from './Preview.svelte';
import { installErrorForwarding } from '$lib/ipc/log';

installErrorForwarding();
mount(Preview, { target: document.getElementById('app')! });
