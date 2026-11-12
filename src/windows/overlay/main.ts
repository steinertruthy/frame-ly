import '../../app.css';
import { mount } from 'svelte';
import Overlay from './Overlay.svelte';
import { installErrorForwarding } from '$lib/ipc/log';

installErrorForwarding();
const displayId = new URLSearchParams(location.search).get('d') ?? '';
mount(Overlay, { target: document.getElementById('app')!, props: { displayId } });
