<script lang="ts">
  import { onMount } from 'svelte';
  import { Cpu, Globe, KeyRound, Check, Trash2, PlugZap } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Section from '$lib/components/Section.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import SettingRow from '$lib/components/SettingRow.svelte';
  import Select from '$lib/components/Select.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type AiProfile, type Detail, type Precision, type ProviderId } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import type { Settings } from '$lib/ipc/types';
  import { formatBytes } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { TRANSCRIPTION_LANGUAGES } from '$lib/ai';
  import ModelRow from './ModelRow.svelte';
  import { languageName, offeredProvider, providerName } from './format';
  import CloudState from './CloudState.svelte';

  interface Props {
    platform: string;
    /** Saves through the Settings page (shows its "Saved" feedback). */
    set: (mutate: (d: Settings) => void) => Promise<void>;
  }
  let { platform, set }: Props = $props();
  const mac = $derived(platform === 'macos');

  onMount(() => void aiStore.init());
  const s = $derived(settings.value!.intelligence);
  const o = $derived(aiStore.overview);
  const info = (id: ProviderId) => o?.providers.find((p) => p.id === id);

  async function save(mutate: (d: Settings) => void) {
    await set(mutate);
    await aiStore.loadOverview();
  }

  const detailOptions = $derived((['brief', 'balanced', 'detailed'] as const).map((d) => ({ value: d, label: t(`ai.detail.${d}` as MessageKey) })));
  const providerOptions = $derived(
    (o?.providers ?? [])
      .filter((p) => offeredProvider(p, s.externalEnabled))
      .map((p) => ({ value: p.id, label: providerName(p.id) })),
  );

  // ── External providers ───────────────────────────────────────────────
  type External = 'anthropic' | 'openai';
  let keys = $state<Record<External, string>>({ anthropic: '', openai: '' });
  let models = $state<Record<External, string[]>>({ anthropic: [], openai: [] });
  let testing = $state<ProviderId | null>(null);
  async function saveKey(p: External) {
    try {
      await ai.setKey(p, keys[p]);
      keys[p] = '';
      toast('success', t('ai.set.key.saved', { store: t(mac ? 'ai.store.mac' : 'ai.store.win') }));
      await aiStore.loadOverview();
    } catch (e) {
      toast('error', t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'));
    }
  }
  async function test(p: External) {
    testing = p;
    try {
      models[p] = await ai.testProvider(p);
      toast('success', t('ai.set.testOk', { n: models[p].length }));
    } catch (e) {
      toast('error', isAppError(e) && e.code === 'provider' ? e.message : t(`error.${isAppError(e) ? e.code : 'generic'}` as MessageKey) || t('error.generic'), 6000);
    } finally {
      testing = null;
    }
  }
  async function removeProvider(p: External) {
    try {
      settings.adopt(await ai.removeProvider(p));
      models[p] = [];
      await aiStore.loadOverview();
    } catch {
      toast('error', t('error.generic'));
    }
  }

  async function clearTemp() {
    const freed = await ai.clearTemp();
    toast('success', t('ai.set.tempCleared', { size: formatBytes(freed) }));
    await aiStore.loadOverview();
  }

  const profileName = (p: Exclude<AiProfile, 'auto'>) => t(`ai.profile.${p}` as MessageKey);
</script>

{#if o}
  <Section title={t('ai.set.framely')}>
    <SettingRow title={t('ai.set.profile')} hint={t('ai.set.profile.hint', { profile: profileName(o.hardware.recommended) })} stacked>
      <Segmented
        label={t('ai.set.profile')}
        value={s.profile}
        options={[
          { value: 'auto', label: t('ai.profile.auto', { profile: profileName(o.hardware.recommended) }) },
          ...(['basic', 'balanced', 'advanced'] as const).map((p) => ({ value: p, label: profileName(p) })),
        ]}
        onchange={(v) => save((d) => (d.intelligence.profile = v))}
      />
    </SettingRow>
    <SettingRow
      title={t('ai.set.hardware')}
      hint="{o.hardware.cpu} · {t('ai.setup.machine', { ram: formatBytes(o.hardware.ramBytes), accel: t(`ai.accel.${o.hardware.accel}` as MessageKey) })}{o.hardware.gpu && o.hardware.accel === 'cpu' ? ` · ${o.hardware.gpu}` : ''}"
    >
      <Cpu size={16} />
    </SettingRow>
    <SettingRow title={t('ai.set.storage')} hint={t('ai.set.storage.hint', { models: formatBytes(o.storage.modelsBytes), temp: formatBytes(o.storage.tempBytes) })}>
      <Button size="sm" disabled={!o.storage.tempBytes} onclick={clearTemp}>{t('ai.set.clearTemp')}</Button>
    </SettingRow>
  </Section>

  <Section title={t('ai.set.models')}>
    {#each o.models as m (m.id)}<ModelRow model={m} />{/each}
  </Section>

  <Section title={t('ai.set.transcription')}>
    <SettingRow title={t('ai.new.precision')} hint={t('ai.set.precision.hint')} stacked>
      <Segmented
        label={t('ai.new.precision')}
        value={s.precision}
        options={(['auto', 'fast', 'balanced', 'max'] as Precision[]).map((p) => ({ value: p, label: t(`ai.precision.${p}` as MessageKey) }))}
        onchange={(v) => save((d) => (d.intelligence.precision = v))}
      />
    </SettingRow>
    <SettingRow title={t('ai.new.language')} hint={t('ai.set.language.hint')}>
      <Select
        label={t('ai.new.language')}
        value={s.language}
        options={[{ value: 'auto', label: t('ai.language.auto') }, ...TRANSCRIPTION_LANGUAGES.map((c) => ({ value: c, label: languageName(c) }))]}
        onchange={(v) => save((d) => (d.intelligence.language = v))}
      />
    </SettingRow>
    <SettingRow title={t('ai.set.speakers')} hint={t('ai.set.speakers.hint')}>
      <Toggle checked={false} disabled label={t('ai.set.speakers')} onchange={() => {}} />
    </SettingRow>
  </Section>

  {#each [{ key: 'summary', title: 'ai.set.summary' }, { key: 'topics', title: 'ai.set.topics' }] as const as g (g.key)}
    {@const detailKey = g.key === 'summary' ? 'summaryDetail' : 'topicsDetail'}
    {@const autoKey = g.key === 'summary' ? 'summaryAuto' : 'topicsAuto'}
    {@const providerKey = g.key === 'summary' ? 'summaryProvider' : 'topicsProvider'}
    <Section title={t(g.title)}>
      <SettingRow title={t('ai.gen.detail')}>
        <Segmented label={t('ai.gen.detail')} value={s[detailKey]} options={detailOptions} onchange={(v: Detail) => save((d) => (d.intelligence[detailKey] = v))} />
      </SettingRow>
      <SettingRow title={t('ai.gen.provider')} hint={info(s[providerKey])?.reason && info(s[providerKey])?.reason !== 'no_consent' ? t(`ai.reason.${info(s[providerKey])!.reason}` as MessageKey) : undefined}>
        <Select label={t('ai.gen.provider')} value={s[providerKey]} options={providerOptions} onchange={(v) => save((d) => (d.intelligence[providerKey] = v))} />
      </SettingRow>
      <SettingRow title={t('ai.set.auto')} hint={t('ai.set.auto.hint')}>
        <Toggle checked={s[autoKey]} label={t('ai.set.auto')} onchange={(v) => save((d) => (d.intelligence[autoKey] = v))} />
      </SettingRow>
    </Section>
  {/each}

  {@const cloud = aiStore.cloud}
  <Section title={t('ai.provider.cloud')}>
    <SettingRow
      title={t('ai.provider.cloud')}
      hint={cloud?.dataUse ? `${t('ai.cloud.description')} ${cloud.dataUse === 'no_training' ? t('ai.cloud.noTraining') : t('ai.cloud.mayTrain')}` : t('ai.cloud.description')}
    >
      <CloudState />
    </SettingRow>
    <SettingRow
      title={t('ai.set.consent')}
      hint={s.consents.includes('cloud') ? t('ai.set.consent.on', { provider: t('ai.provider.cloud') }) : t('ai.set.consent.off')}
    >
      {#if s.consents.includes('cloud')}
        <Button size="sm" variant="ghost" onclick={() => save((d) => (d.intelligence.consents = d.intelligence.consents.filter((c) => c !== 'cloud')))}>{t('ai.set.revoke')}</Button>
      {/if}
    </SettingRow>
  </Section>

  {#if mac}
    {@const apple = info('apple')}
    <Section title={t('ai.set.apple')}>
      <SettingRow
        title={t('ai.set.apple.enabled')}
        hint={apple?.available ? t('ai.set.apple.available') : apple?.reason ? t(`ai.reason.${apple.reason}` as MessageKey) : undefined}
      >
        <Toggle checked={s.appleEnabled} label={t('ai.set.apple.enabled')} onchange={(v) => save((d) => (d.intelligence.appleEnabled = v))} />
      </SettingRow>
    </Section>
  {/if}

  <Section title={t('ai.set.privacy')}>
    <SettingRow title={t('ai.set.localOnly')} hint={t('ai.set.localOnly.hint')}>
      <Toggle checked={s.localOnly} label={t('ai.set.localOnly')} onchange={(v) => save((d) => (d.intelligence.localOnly = v))} />
    </SettingRow>
    <SettingRow title={t('ai.set.external.enabled')} hint={t('ai.set.external.hint')}>
      <Toggle checked={s.externalEnabled} label={t('ai.set.external.enabled')} onchange={(v) => save((d) => (d.intelligence.externalEnabled = v))} />
    </SettingRow>
  </Section>

  {#if s.externalEnabled}
    {#each ['anthropic', 'openai'] as const as p (p)}
      {@const pi = info(p)}
      {@const consented = s.consents.includes(p)}
      <Section title={`${providerName(p)} · ${t('ai.set.paid')}`}>
        <SettingRow title={t('ai.set.key')} hint={pi?.keyConfigured ? t('ai.set.key.saved', { store: t(mac ? 'ai.store.mac' : 'ai.store.win') }) : undefined}>
          {#if pi?.keyConfigured}
            <span class="ok"><Check size={14} /></span>
            <Button size="sm" busy={testing === p} disabled={s.localOnly} onclick={() => test(p)}><PlugZap size={13} />{t('ai.set.test')}</Button>
            <Button size="sm" variant="ghost" onclick={() => removeProvider(p)}><Trash2 size={13} />{t('ai.set.remove')}</Button>
          {:else}
            <label class="key">
              <KeyRound size={14} />
              <input type="password" autocomplete="off" spellcheck="false" placeholder={t('ai.set.key.placeholder')} bind:value={keys[p]} />
            </label>
            <Button size="sm" variant="primary" disabled={keys[p].trim().length < 8} onclick={() => saveKey(p)}>{t('ai.set.save')}</Button>
          {/if}
        </SettingRow>
        {#if pi?.keyConfigured}
          {@const current = p === 'anthropic' ? s.anthropicModel : s.openaiModel}
          {@const list = p === 'anthropic' ? [...new Set([...o.anthropicModels, ...models[p].filter((m) => o.anthropicModels.includes(m))])] : models[p]}
          <SettingRow title={t('ai.set.model')} hint={list.length ? undefined : t('ai.set.model.pick')}>
            <Select
              label={t('ai.set.model')}
              value={current}
              disabled={!list.length && !current}
              options={(list.includes(current) || !current ? list : [current, ...list]).map((m) => ({ value: m, label: m }))}
              onchange={(v) => save((d) => (p === 'anthropic' ? (d.intelligence.anthropicModel = v) : (d.intelligence.openaiModel = v)))}
            />
          </SettingRow>
          <SettingRow title={t('ai.set.consent')} hint={consented ? t('ai.set.consent.on', { provider: providerName(p) }) : t('ai.set.consent.off')}>
            <Globe size={16} />
            {#if consented}
              <Button size="sm" variant="ghost" onclick={() => save((d) => (d.intelligence.consents = d.intelligence.consents.filter((c) => c !== p)))}>{t('ai.set.revoke')}</Button>
            {/if}
          </SettingRow>
        {/if}
      </Section>
    {/each}
  {/if}
{/if}

<style>
  .ok {
    display: grid;
    place-items: center;
    color: var(--ok-500);
  }
  .key {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    width: 260px;
    padding: 0 10px;
    border-radius: var(--radius-md);
    background: var(--bg);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--text-3);
  }
  .key:focus-within {
    box-shadow: var(--focus-ring);
  }
  .key input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: none;
    color: var(--text);
    font: inherit;
    font-size: var(--text-sm);
    user-select: text;
  }
  .key input:focus {
    outline: none;
  }
</style>
