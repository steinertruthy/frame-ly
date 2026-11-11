<script lang="ts">
  import { Monitor, Mic } from '@lucide/svelte';
  import Badge from './Badge.svelte';
  import Button from './Button.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { commands } from '$lib/ipc/commands';
  import type { Permission, PermissionState } from '$lib/ipc/types';
  import { system } from '$lib/stores/system.svelte';
  import { toast } from '$lib/stores/toasts.svelte';

  interface Props {
    permission: Permission;
    status: PermissionState;
    platform: string;
  }
  let { permission, status, platform }: Props = $props();

  let busy = $state(false);
  const tone = $derived(
    ({ granted: 'ok', denied: 'warn', notDetermined: 'neutral', restricted: 'error' } as const)[status],
  );
  // macOS only applies a newly granted Screen Recording permission to a fresh process.
  const needsRelaunch = $derived(platform === 'macos' && permission === 'screen' && status === 'denied');
  const canOpenSettings = $derived(status === 'denied' && !(platform === 'windows' && permission === 'screen'));

  async function allow() {
    busy = true;
    await system.request(permission);
    busy = false;
  }

  async function openSettings() {
    try {
      await commands.openPermissionSettings(permission);
    } catch {
      toast('error', t('error.platform'));
    }
  }
</script>

<div class="row">
  <div class="icon" class:ok={status === 'granted'}>
    {#if permission === 'screen'}<Monitor size={18} />{:else}<Mic size={18} />{/if}
  </div>
  <div class="text">
    <div class="title">
      {t(permission === 'screen' ? 'perm.screen' : 'perm.microphone')}
      <Badge {tone}>{t(`perm.state.${status}`)}</Badge>
    </div>
    <div class="hint">
      {#if status === 'restricted'}
        {t('perm.restrictedHint')}
      {:else if needsRelaunch}
        {t('perm.relaunchHint')}
      {:else}
        {t(permission === 'screen' ? 'perm.screen.hint' : 'perm.microphone.hint')}
      {/if}
    </div>
  </div>
  <div class="actions">
    {#if status === 'notDetermined'}
      <Button variant="primary" size="sm" {busy} onclick={allow}>{t('perm.allow')}</Button>
    {/if}
    {#if canOpenSettings}
      <Button size="sm" onclick={openSettings}>{t('perm.openSettings')}</Button>
    {/if}
    {#if needsRelaunch}
      <Button variant="primary" size="sm" onclick={() => commands.relaunch()}>{t('perm.relaunch')}</Button>
    {/if}
  </div>
</div>

<style>
  .row {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 14px var(--space-4);
  }
  .row + :global(.row) {
    border-top: 1px solid var(--border);
  }
  .icon {
    display: grid;
    place-items: center;
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: var(--surface-hover);
    color: var(--text-2);
    flex-shrink: 0;
    transition:
      background var(--dur-med),
      color var(--dur-med);
  }
  .icon.ok {
    background: color-mix(in srgb, var(--ok-500) 14%, transparent);
    color: var(--ok-500);
  }
  .text {
    flex: 1;
    min-width: 0;
  }
  .title {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    font-weight: 600;
  }
  .hint {
    margin-top: 2px;
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .actions {
    display: flex;
    gap: var(--space-2);
    flex-shrink: 0;
  }
</style>
