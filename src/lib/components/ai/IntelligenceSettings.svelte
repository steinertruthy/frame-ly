<script lang="ts">
  import { onMount } from 'svelte';
  import { Cpu, Globe, KeyRound, Check, Trash2, PlugZap, ShieldCheck, Cloud } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Section from '$lib/components/Section.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import SettingRow from '$lib/components/SettingRow.svelte';
  import Select from '$lib/components/Select.svelte';
  import Toggle from '$lib/components/Toggle.svelte';
  import ConfirmDialog from '$lib/components/ConfirmDialog.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { ai, type AiProfile, type Detail, type Precision, type ProviderId, type ProviderInfo } from '$lib/ipc/ai';
  import { isAppError } from '$lib/ipc/commands';
  import type { Settings } from '$lib/ipc/types';
  import { formatBytes } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import { TRANSCRIPTION_LANGUAGES } from '$lib/ai';
  import ModelRow from './ModelRow.svelte';
  import { consentPoints, languageName, offeredProvider, providerName, roughTime } from './format';
  import CloudState from './CloudState.svelte';

  interface Props {
    platform: string;
    /** Saves through the Settings page (shows its "Saved" feedback). */
    set: (mutate: (d: Settings) => void) => Promise<void>;
  }
  let { platform, set }: Props = $props();
  const mac = $derived(platform === 'macos');

  onMount(() => {
    void aiStore.init();
    // Fresh usage numbers every time Settings opens.
    void aiStore.checkCloud(true);
  });
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

  // Consent can also be given here, after reading the same notice shown before the first send.
  let granting = $state<ProviderId | null>(null);
  async function grant() {
    const p = granting!;
    granting = null;
    await save((d) => void (d.intelligence.consents.includes(p) || d.intelligence.consents.push(p)));
  }

  /** Toggles that change where content may go: save, then say plainly what changed. */
  async function toggle(key: 'localOnly' | 'appleEnabled' | 'externalEnabled', v: boolean) {
    await save((d) => (d.intelligence[key] = v));
    const name = key === 'localOnly' ? 'localOnly' : key === 'appleEnabled' ? 'apple' : 'external';
    toast('info', t(`ai.toast.${name}.${v ? 'on' : 'off'}` as MessageKey), 4000);
    if (key === 'localOnly') void aiStore.checkCloud(true);
  }

  /** One line per provider for the overview: where it runs and whether it's ready. */
  function statusOf(p: ProviderInfo): { tone: 'ok' | 'wait' | 'off' | 'warn'; text: string } {
    if (p.id === 'cloud' && !s.localOnly) {
      const c = aiStore.cloud;
      if (c && c.state !== 'available') return { tone: c.state === 'not_configured' ? 'off' : 'warn', text: t(`ai.cloud.state.${c.state}` as MessageKey) };
    }
    switch (p.reason ?? null) {
      case null:
        return { tone: 'ok', text: t('ai.status.ready') };
      case 'no_consent':
        return { tone: 'wait', text: t('ai.status.consent') };
      case 'local_only':
        return { tone: 'off', text: t('ai.status.localOnly') };
      case 'disabled':
      case 'external_disabled':
        return { tone: 'off', text: t('ai.status.off') };
      case 'no_key':
        return { tone: 'wait', text: t('ai.status.needsKey') };
      case 'no_model':
        return { tone: 'wait', text: t('ai.status.needsModel') };
      case 'model_missing':
        return { tone: 'wait', text: t('ai.status.needsDownload') };
      default:
        return { tone: 'warn', text: t(`ai.reason.${p.reason}` as MessageKey) };
    }
  }
  const where = (p: ProviderInfo) => t(p.local ? 'ai.where.local' : p.paid ? 'ai.where.paid' : 'ai.where.cloud');

  const profileName = (p: Exclude<AiProfile, 'auto'>) => t(`ai.profile.${p}` as MessageKey);
</script>

{#if o}
  <Section title={t('ai.set.overview')}>
    <div class="overview">
      <p class="lead" class:local={s.localOnly}>
        {#if s.localOnly}<ShieldCheck size={16} />{:else}<Cloud size={16} />{/if}
        <span>{s.localOnly ? t('ai.set.overview.local') : t('ai.set.overview.mixed')}</span>
      </p>
      <ul class="providers">
        {#each o.providers.filter((p) => mac || p.id !== 'apple') as p (p.id)}
          {@const st = statusOf(p)}
          <li>
            <span class="picon">{#if p.local}<Cpu size={15} />{:else}<Globe size={15} />{/if}</span>
            <span class="pmeta">
              <span class="pname">{providerName(p.id)}</span>
              <span class="where">{where(p)}</span>
            </span>
            <span class="badge tone-{st.tone}"><i></i>{st.text}</span>
          </li>
        {/each}
      </ul>
    </div>
  </Section>

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

  <Section title={t('ai.set.models')} anchor="models">
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
      {:else}
        <Button size="sm" disabled={s.localOnly} onclick={() => (granting = 'cloud')}>{t('ai.set.allow')}</Button>
      {/if}
    </SettingRow>
    {#if cloud?.requestsPerDay}
      {@const used = cloud.usedToday}
      {@const max = cloud.requestsPerDay}
      <SettingRow
        title={t('ai.cloud.limit')}
        hint={`${t('ai.cloud.limit.hint', { n: max })}${cloud.resetsIn ? ` ${t('ai.cloud.limit.resets', { time: roughTime(cloud.resetsIn * 1000) })}` : ''}`}
      >
        {#if used != null}
          <span class="usage" class:full={used >= max}>
            <span class="meter"><span style:width="{Math.min(100, (used / max) * 100)}%"></span></span>
            <span class="value">{t('ai.cloud.limit.used', { used, max })}</span>
          </span>
        {:else}
          <span class="value">{max}</span>
        {/if}
      </SettingRow>
    {/if}
  </Section>

  {#if mac}
    {@const apple = info('apple')}
    <Section title={t('ai.set.apple')}>
      <SettingRow
        title={t('ai.set.apple.enabled')}
        hint={s.appleEnabled
          ? `${t('ai.set.apple.on')}${apple?.reason && apple.reason !== 'disabled' ? ` ${t(`ai.reason.${apple.reason}` as MessageKey)}` : ''}`
          : t('ai.set.apple.off')}
      >
        <Toggle checked={s.appleEnabled} label={t('ai.set.apple.enabled')} onchange={(v) => toggle('appleEnabled', v)} />
      </SettingRow>
    </Section>
  {/if}

  <Section title={t('ai.set.privacy')}>
    <SettingRow title={t('ai.set.localOnly')} hint={s.localOnly ? t('ai.set.localOnly.on') : t('ai.set.localOnly.off')}>
      <Toggle checked={s.localOnly} label={t('ai.set.localOnly')} onchange={(v) => toggle('localOnly', v)} />
    </SettingRow>
    <SettingRow title={t('ai.set.external.enabled')} hint={`${s.externalEnabled ? t('ai.set.external.on') : t('ai.set.external.off')} ${t('ai.set.external.hint')}`}>
      <Toggle checked={s.externalEnabled} label={t('ai.set.external.enabled')} onchange={(v) => toggle('externalEnabled', v)} />
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
            {:else}
              <Button size="sm" disabled={s.localOnly} onclick={() => (granting = p)}>{t('ai.set.allow')}</Button>
            {/if}
          </SettingRow>
        {/if}
      </Section>
    {/each}
  {/if}
{/if}

{#if granting}
  <ConfirmDialog
    title={t('ai.set.allow.title', { provider: providerName(granting) })}
    points={consentPoints(granting, aiStore.cloud)}
    note={t('ai.consent.p.revoke')}
    confirm={t('ai.set.allow')}
    cancel={t('ai.consent.cancel')}
    onconfirm={grant}
    oncancel={() => (granting = null)}
  />
{/if}

<style>
  .overview {
    display: grid;
    gap: 12px;
    padding: 14px 16px;
  }
  .lead {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    margin: 0;
    color: var(--text-2);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .lead :global(svg) {
    flex-shrink: 0;
    margin-top: 1px;
    color: var(--icon-accent);
  }
  .lead.local :global(svg) {
    color: var(--ok-500);
  }
  .providers {
    display: grid;
    gap: 2px;
    margin: 0;
    padding: 0;
    list-style: none;
  }
  .providers li {
    display: grid;
    grid-template-columns: 30px minmax(0, 1fr) auto;
    align-items: center;
    gap: 12px;
    padding: 9px 0;
    border-top: 1px solid var(--border);
  }
  .picon {
    display: grid;
    place-items: center;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    background: var(--icon-tile);
    color: var(--icon-accent);
  }
  .pmeta {
    display: grid;
    gap: 1px;
    min-width: 0;
  }
  .pname {
    font-weight: 600;
  }
  .where {
    color: var(--text-3);
    font-size: var(--text-xs);
  }
  .badge {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: 99px;
    background: var(--surface-2);
    color: var(--text-2);
    font-size: var(--text-xs);
    font-weight: 600;
    white-space: nowrap;
  }
  .badge i {
    width: 6px;
    height: 6px;
    flex-shrink: 0;
    border-radius: 50%;
    background: var(--text-3);
  }
  .tone-ok {
    color: var(--ok-500);
  }
  .tone-ok i {
    background: var(--ok-500);
  }
  .tone-wait {
    color: var(--link);
  }
  .tone-wait i {
    background: var(--accent);
  }
  .tone-warn {
    color: var(--warn-500);
  }
  .tone-warn i {
    background: var(--warn-500);
  }
  .usage {
    display: inline-flex;
    align-items: center;
    gap: 10px;
  }
  .meter {
    position: relative;
    width: 96px;
    height: 6px;
    border-radius: 99px;
    background: var(--surface-2);
    overflow: hidden;
  }
  .meter span {
    position: absolute;
    inset: 0 auto 0 0;
    border-radius: inherit;
    background: var(--accent);
  }
  .usage.full .meter span {
    background: var(--warn-500);
  }
  .usage.full .value {
    color: var(--warn-500);
  }
  .value {
    color: var(--text-2);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
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
