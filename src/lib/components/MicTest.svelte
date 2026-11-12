<script lang="ts">
  import { onDestroy } from 'svelte';
  import { Mic } from '@lucide/svelte';
  import Button from './Button.svelte';
  import LevelMeter from './LevelMeter.svelte';
  import { commands, isAppError } from '$lib/ipc/commands';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { system } from '$lib/stores/system.svelte';

  let { deviceId }: { deviceId: string | null } = $props();
  let running = $state(false);
  let level = $state(0);
  let timer: ReturnType<typeof setInterval> | undefined;

  async function start() {
    try {
      await commands.micTestStart(deviceId);
      running = true;
      timer = setInterval(async () => (level = await commands.micTestLevel()), 80);
    } catch (e) {
      toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
      void system.refresh();
    }
  }

  function stop() {
    clearInterval(timer);
    running = false;
    level = 0;
    void commands.micTestStop();
  }

  // Switching device while testing restarts on the new one.
  $effect(() => {
    void deviceId;
    if (running) {
      stop();
      void start();
    }
  });

  onDestroy(() => running && stop());
</script>

<div class="test">
  {#if running}<LevelMeter {level} label={t('audio.test')} />{/if}
  <Button size="sm" variant={running ? 'secondary' : 'ghost'} onclick={() => (running ? stop() : start())}>
    <Mic size={14} />{t(running ? 'audio.test.stop' : 'audio.test')}
  </Button>
</div>

<style>
  .test {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }
</style>
