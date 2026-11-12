import '../../app.css';
import { mount } from 'svelte';
import Countdown from './Countdown.svelte';
import { installErrorForwarding } from '$lib/ipc/log';

installErrorForwarding();
const n = Number(new URLSearchParams(location.search).get('n') ?? '3');
mount(Countdown, { target: document.getElementById('app')!, props: { seconds: n } });
