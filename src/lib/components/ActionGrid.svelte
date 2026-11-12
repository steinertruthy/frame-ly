<script lang="ts" module>
  import type { Component } from 'svelte';
  import type { MessageKey } from '$lib/i18n/index.svelte';
  export interface Action {
    id: string;
    icon: Component<{ size?: number; strokeWidth?: number }>;
    title: MessageKey;
    hint: MessageKey;
    run: () => Promise<unknown>;
  }
</script>

<script lang="ts">
  import { isAppError } from '$lib/ipc/commands';
  import { t } from '$lib/i18n/index.svelte';
  import { toast } from '$lib/stores/toasts.svelte';

  let { actions, tone = 'blue', disabled = false }: { actions: Action[]; tone?: 'blue' | 'red'; disabled?: boolean } = $props();

  async function run(a: Action) {
    try {
      await a.run();
    } catch (e) {
      const key = (isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey;
      toast('error', t(key) ?? t('error.generic'));
    }
  }
</script>

<div class="grid {tone}">
  {#each actions as a (a.id)}
    <button class="action" {disabled} onclick={() => run(a)}>
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
    color: #fff;
  }
  .blue .icon {
    background: var(--hero);
    box-shadow: 0 6px 16px -6px rgba(22, 48, 122, 0.7);
  }
  .red .icon {
    background: linear-gradient(135deg, #ff6b78, #e5394b);
    box-shadow: 0 6px 16px -6px rgba(229, 57, 75, 0.65);
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
