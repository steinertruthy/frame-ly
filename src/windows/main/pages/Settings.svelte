<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { FolderOpen } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Section from '$lib/components/Section.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import SettingRow from '$lib/components/SettingRow.svelte';
  import { t } from '$lib/i18n/index.svelte';
  import { commands } from '$lib/ipc/commands';
  import type { AppInfo, Folder, Language, Theme } from '$lib/ipc/types';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { system } from '$lib/stores/system.svelte';
  import Badge from '$lib/components/Badge.svelte';
  import PermissionRow from '$lib/components/PermissionRow.svelte';
  import Toggle from '$lib/components/Toggle.svelte';

  let { info }: { info: AppInfo } = $props();
  const s = $derived(settings.value);

  async function chooseFolder(kind: 'recordingsDir' | 'screenshotsDir') {
    if (!s) return;
    const picked = await openDialog({ directory: true, defaultPath: s.storage[kind], title: t('settings.chooseFolder') });
    if (typeof picked !== 'string' || picked === s.storage[kind]) return;
    if (await settings.update((d) => (d.storage[kind] = picked))) toast('success', t('settings.saved'));
  }

  async function reveal(folder: Folder) {
    try {
      await commands.openFolder(folder);
    } catch (e) {
      console.error(e);
      toast('error', t('error.io'));
    }
  }
</script>

<h1 class="title">{t('settings.title')}</h1>

{#if s}
  <Section title={t('settings.general')}>
    <SettingRow title={t('settings.language')} hint={t('settings.language.hint')}>
      <Segmented
        label={t('settings.language')}
        value={s.general.language}
        options={[
          { value: 'system' as Language, label: t('settings.language.system') },
          { value: 'pt-BR' as Language, label: 'Português' },
          { value: 'en' as Language, label: 'English' },
        ]}
        onchange={(v) => settings.update((d) => (d.general.language = v))}
      />
    </SettingRow>
  </Section>

  <Section title={t('settings.appearance')}>
    <SettingRow title={t('settings.theme')} hint={t('settings.theme.hint')}>
      <Segmented
        label={t('settings.theme')}
        value={s.appearance.theme}
        options={[
          { value: 'system' as Theme, label: t('settings.theme.system') },
          { value: 'light' as Theme, label: t('settings.theme.light') },
          { value: 'dark' as Theme, label: t('settings.theme.dark') },
        ]}
        onchange={(v) => settings.update((d) => (d.appearance.theme = v))}
      />
    </SettingRow>
  </Section>

  <Section title={t('settings.capture')}>
    <SettingRow title={t('settings.capture.copy')} hint={t('settings.capture.copy.hint')}>
      <Toggle
        label={t('settings.capture.copy')}
        checked={s.capture.copyToClipboard}
        disabled={s.capture.copyToClipboard && !s.capture.saveToDisk}
        onchange={(v) => settings.update((d) => (d.capture.copyToClipboard = v))}
      />
    </SettingRow>
    <SettingRow title={t('settings.capture.save')} hint={t('settings.capture.save.hint')}>
      <Toggle
        label={t('settings.capture.save')}
        checked={s.capture.saveToDisk}
        disabled={s.capture.saveToDisk && !s.capture.copyToClipboard}
        onchange={(v) => settings.update((d) => (d.capture.saveToDisk = v))}
      />
    </SettingRow>
    <SettingRow title={t('settings.capture.preview')} hint={t('settings.capture.preview.hint')}>
      <Toggle
        label={t('settings.capture.preview')}
        checked={s.capture.showPreview}
        onchange={(v) => settings.update((d) => (d.capture.showPreview = v))}
      />
    </SettingRow>
  </Section>

  <Section title={t('settings.storage')}>
    <SettingRow title={t('settings.recordingsDir')} hint={s.storage.recordingsDir}>
      <Button size="sm" variant="ghost" onclick={() => reveal('recordings')}><FolderOpen size={14} />{t('settings.show')}</Button>
      <Button size="sm" onclick={() => chooseFolder('recordingsDir')}>{t('settings.change')}</Button>
    </SettingRow>
    <SettingRow title={t('settings.screenshotsDir')} hint={s.storage.screenshotsDir}>
      <Button size="sm" variant="ghost" onclick={() => reveal('screenshots')}><FolderOpen size={14} />{t('settings.show')}</Button>
      <Button size="sm" onclick={() => chooseFolder('screenshotsDir')}>{t('settings.change')}</Button>
    </SettingRow>
  </Section>

  {#if system.permissions}
    <Section title={t('settings.permissions')}>
      <PermissionRow permission="screen" status={system.permissions.screen} platform={info.platform} />
      <PermissionRow permission="microphone" status={system.permissions.microphone} platform={info.platform} />
    </Section>
  {/if}

  <Section title={t('settings.devices')}>
    {#each system.displays as d (d.id)}
      <SettingRow
        title={d.name}
        hint={`${d.pixelWidth} × ${d.pixelHeight}${d.refreshRate ? ` · ${Math.round(d.refreshRate)} Hz` : ''}${d.scaleFactor !== 1 ? ` · ${d.scaleFactor}×` : ''}`}
      >
        <span class="kind">{t('settings.kind.display')}</span>
        {#if d.isPrimary}<Badge tone="accent">{t('settings.primary')}</Badge>{/if}
      </SettingRow>
    {/each}
    {#each system.audioInputs as m (m.id)}
      <SettingRow title={m.name}>
        <span class="kind">{t('settings.kind.microphone')}</span>
        {#if m.isDefault}<Badge tone="accent">{t('settings.default')}</Badge>{/if}
      </SettingRow>
    {:else}
      <SettingRow title={t('settings.noMicrophones')}><span></span></SettingRow>
    {/each}
  </Section>

  <Section title={t('settings.system')}>
    <SettingRow title={t('settings.version')}>
      <span class="value">FrameLy {info.version}</span>
    </SettingRow>
    <SettingRow title={t('settings.platform')}>
      <span class="value">{info.platform} · {info.arch}</span>
    </SettingRow>
    <SettingRow title={t('settings.logs')} hint={t('settings.logs.hint')}>
      <Button size="sm" onclick={() => reveal('logs')}><FolderOpen size={14} />{t('settings.show')}</Button>
    </SettingRow>
  </Section>
{/if}

<style>
  .title {
    font-size: var(--text-xl);
    font-weight: 700;
    margin-bottom: var(--space-5);
  }
  .kind {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .value {
    color: var(--text-2);
    font-variant-numeric: tabular-nums;
  }
</style>
