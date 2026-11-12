<script lang="ts">
  import { Crop, AppWindow, Monitor } from '@lucide/svelte';
  import { commands } from '$lib/ipc/commands';
  import ActionGrid, { type Action } from './ActionGrid.svelte';

  let { disabled = false }: { disabled?: boolean } = $props();

  const actions: Action[] = [
    { id: 'region', icon: Crop, title: 'capture.region', hint: 'capture.region.hint', run: () => commands.takeScreenshot({ kind: 'select', mode: 'region' }) },
    { id: 'window', icon: AppWindow, title: 'capture.window', hint: 'capture.window.hint', run: () => commands.takeScreenshot({ kind: 'select', mode: 'window' }) },
    { id: 'screen', icon: Monitor, title: 'capture.screen', hint: 'capture.screen.hint', run: () => commands.takeScreenshot({ kind: 'display', displayId: null }) },
  ];
</script>

<ActionGrid {actions} {disabled} />
