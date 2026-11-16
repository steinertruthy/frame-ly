<script lang="ts">
  import { Sparkles, AudioLines, ListTree, FileText, HardDrive, ShieldCheck, Check, WifiOff } from '@lucide/svelte';
  import Button from '$lib/components/Button.svelte';
  import Segmented from '$lib/components/Segmented.svelte';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { formatBytes } from '$lib/stores/format';
  import { aiStore } from '$lib/stores/ai.svelte';
  import { settings } from '$lib/stores/settings.svelte';
  import { isReady, languageModelId, setupPlan, speechModelId, usesLocalGeneration } from '$lib/ai';
  import type { AiProfile } from '$lib/ipc/ai';
  import ModelRow from './ModelRow.svelte';

  interface Props {
    onlater?: () => void;
  }
  let { onlater }: Props = $props();

  const o = $derived(aiStore.overview);
  const plan = $derived(
    o ? setupPlan(o.models, speechModelId(o.defaultPrecision), usesLocalGeneration(settings.value?.intelligence) ? languageModelId(o.profile) : null) : { models: [], remainingBytes: 0, done: false },
  );
  const started = $derived(plan.models.some((m) => m.state !== 'missing' || m.received > 0));
  const localOnly = $derived(settings.value?.intelligence.localOnly ?? false);
  const noSpace = $derived(o?.storage.freeBytes != null && o.storage.freeBytes < plan.remainingBytes + 512 * 1024 * 1024);
  const profileLabel = (p: Exclude<AiProfile, 'auto'>) => t(`ai.profile.${p}` as MessageKey);

  async function setProfile(p: Exclude<AiProfile, 'auto'>) {
    // Picking the recommended one keeps following the computer ("automatic").
    const value: AiProfile = p === o?.hardware.recommended ? 'auto' : p;
    if (await settings.update((d) => (d.intelligence.profile = value))) await aiStore.loadOverview();
  }

  async function downloadAll() {
    for (const m of plan.models) if (!isReady(m) && m.state !== 'downloading') await aiStore.modelAction(m.id, m.state === 'corrupt' ? 'repair' : 'download');
  }
</script>

{#if o}
  <section class="setup">
    <div class="hero">
      <span class="mark"><Sparkles size={22} /></span>
      <div>
        <h2>{t('ai.setup.title')}</h2>
        <p>{t('ai.setup.intro')}</p>
      </div>
    </div>

    <ul class="features">
      <li><AudioLines size={15} />{t('ai.setup.f.transcribe')}</li>
      <li><FileText size={15} />{t('ai.setup.f.summary')}</li>
      <li><ListTree size={15} />{t('ai.setup.f.topics')}</li>
    </ul>

    <div class="profile">
      <div>
        <div class="label">{t('ai.setup.profile', { profile: profileLabel(o.hardware.recommended) })}</div>
        <div class="hint">
          {t('ai.setup.machine', { ram: formatBytes(o.hardware.ramBytes), accel: t(`ai.accel.${o.hardware.accel}` as MessageKey) })} ·
          {t(`ai.profile.${o.profile}.hint` as MessageKey)}
        </div>
      </div>
      <Segmented
        label={t('ai.setup.change')}
        size="sm"
        value={o.profile}
        options={(['basic', 'balanced', 'advanced'] as const).map((p) => ({ value: p, label: profileLabel(p) }))}
        onchange={setProfile}
      />
    </div>

    <div class="models">
      {#each plan.models as m (m.id)}<ModelRow model={m} simple />{/each}
    </div>

    <div class="facts">
      <span><HardDrive size={14} />{t('ai.setup.space', { size: formatBytes(plan.remainingBytes), free: o.storage.freeBytes != null ? formatBytes(o.storage.freeBytes) : '—' })}</span>
      <span><ShieldCheck size={14} />{t('ai.setup.storage')}</span>
    </div>

    {#if localOnly}
      <p class="warn"><WifiOff size={14} />{t('ai.setup.offline')}</p>
    {:else if noSpace && !plan.done}
      <p class="warn">{t('ai.setup.noSpace', { size: formatBytes(plan.remainingBytes) })}</p>
    {/if}

    <div class="actions">
      {#if plan.done}
        <span class="ready"><Check size={15} />{t('ai.setup.ready')}</span>
        {#if onlater}<Button variant="primary" onclick={onlater}>{t('ai.done')}</Button>{/if}
      {:else}
        {#if onlater}<Button variant="ghost" onclick={onlater}>{t('ai.setup.later')}</Button>{/if}
        <Button variant="primary" size="lg" disabled={localOnly || noSpace || plan.models.every((m) => m.state === 'downloading' || m.state === 'verifying' || isReady(m))} onclick={downloadAll}>
          {started ? t('ai.setup.resume') : t('ai.setup.download')}
        </Button>
      {/if}
    </div>
  </section>
{/if}

<style>
  .setup {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: 24px;
    border-radius: var(--radius-xl);
    background: var(--hero), var(--surface);
    box-shadow: var(--shadow-md);
    animation: rise var(--dur-slow) var(--ease-out);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  .hero {
    display: flex;
    gap: 16px;
    align-items: flex-start;
  }
  .mark {
    display: grid;
    place-items: center;
    flex-shrink: 0;
    width: 46px;
    height: 46px;
    border-radius: 13px;
    background: var(--brand-tile);
    box-shadow: inset 0 0 0 1px var(--border);
    color: var(--icon-accent);
  }
  h2 {
    margin: 2px 0 6px;
    font-size: var(--text-xl);
    font-weight: 650;
    letter-spacing: -0.01em;
  }
  p {
    margin: 0;
    color: var(--text-2);
    line-height: 1.5;
    max-width: 640px;
  }
  .features {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 20px;
    margin: 0;
    padding: 0;
    list-style: none;
    color: var(--text);
    font-size: var(--text-sm);
  }
  .features li {
    display: flex;
    align-items: center;
    gap: 7px;
  }
  .features :global(svg) {
    color: var(--icon-accent);
  }
  .profile {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    flex-wrap: wrap;
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .label {
    font-weight: 600;
  }
  .hint {
    color: var(--text-2);
    font-size: var(--text-sm);
    margin-top: 2px;
  }
  .models {
    border-radius: var(--radius-lg);
    background: var(--bg-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .facts {
    display: flex;
    flex-direction: column;
    gap: 6px;
    color: var(--text-2);
    font-size: var(--text-sm);
  }
  .facts span {
    display: flex;
    align-items: flex-start;
    gap: 8px;
    line-height: 1.45;
  }
  .facts :global(svg) {
    flex-shrink: 0;
    margin-top: 2px;
    color: var(--text-3);
  }
  .warn {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--warn-500);
    font-size: var(--text-sm);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 8px;
  }
  .ready {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--ok-500);
    font-weight: 600;
  }
</style>
