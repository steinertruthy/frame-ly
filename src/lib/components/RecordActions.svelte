<script lang="ts">
  import { SquareDashed, AppWindow, MonitorPlay } from '@lucide/svelte';
  import { commands } from '$lib/ipc/commands';
  import ActionGrid, { type Action } from './ActionGrid.svelte';

  let { disabled = false }: { disabled?: boolean } = $props();

  const actions: Action[] = [
    { id: 'region', icon: SquareDashed, title: 'record.region', hint: 'record.region.hint', run: () => commands.startRecording({ kind: 'select', mode: 'region' }) },
    { id: 'window', icon: AppWindow, title: 'record.window', hint: 'record.window.hint', run: () => commands.startRecording({ kind: 'select', mode: 'window' }) },
    { id: 'screen', icon: MonitorPlay, title: 'record.screen', hint: 'record.screen.hint', run: () => commands.startRecording({ kind: 'display', displayId: null }) },
  ];
</script>

<ActionGrid {actions} {disabled} tone="red" />
