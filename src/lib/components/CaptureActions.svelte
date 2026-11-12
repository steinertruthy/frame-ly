<script lang="ts">
  import { Crop, AppWindow, Monitor } from '@lucide/svelte';
  import { commands, isAppError } from '$lib/ipc/commands';
  import type { CaptureRequest } from '$lib/ipc/types';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { toast } from '$lib/stores/toasts.svelte';

  let { disabled = false }: { disabled?: boolean } = $props();

  const actions: { id: string; icon: typeof Crop; title: MessageKey; hint: MessageKey; request: CaptureRequest }[] = [
    { id: 'region', icon: Crop, title: 'capture.region', hint: 'capture.region.hint', request: { kind: 'select', mode: 'region' } },
    { id: 'window', icon: AppWindow, title: 'capture.window', hint: 'capture.window.hint', request: { kind: 'select', mode: 'window' } },
    { id: 'screen', icon: Monitor, title: 'capture.screen', hint: 'capture.screen.hint', request: { kind: 'display', displayId: null } },
  ];

  async function run(request: CaptureRequest) {
    try {
      await commands.takeScreenshot(request);
    } catch (e) {
      toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
    }
  }
</script>

<div class="grid">
  {#each actions as a (a.id)}
    <button class="action" {disabled} onclick={() => run(a.request)}>
      <span class="icon"><a.icon size={20} strokeWidth={1.8} /></span>
      <span class="title">{t(a.title)}</span>
      <span class="hint">{t(a.hint)}</span>
    </button>
  {/each}
</div>

<style>
  .grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--space-3);
  }
  .action {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: var(--space-4);
    border: 0;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow: var(--shadow-sm);
    text-align: left;
    transition:
      transform var(--dur-med) var(--ease-out),
      box-shadow var(--dur-med) var(--ease-out);
  }
  .action:hover:not(:disabled) {
    transform: translateY(-2px);
    box-shadow: var(--shadow-md);
  }
  .action:active:not(:disabled) {
    transform: translateY(0) scale(0.98);
  }
  .action:disabled {
    opacity: 0.5;
  }
  .icon {
    display: grid;
    place-items: center;
    width: 40px;
    height: 40px;
    margin-bottom: 10px;
    border-radius: 12px;
    background: var(--hero);
    color: #fff;
    box-shadow: 0 6px 16px -6px rgba(22, 48, 122, 0.7);
  }
  .title {
    font-weight: 650;
    font-size: var(--text-lg);
  }
  .hint {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
</style>
