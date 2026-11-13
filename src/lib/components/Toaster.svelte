<script lang="ts">
  import { fly } from 'svelte/transition';
  import { CircleCheck, CircleAlert, Info } from '@lucide/svelte';
  import { toasts, dismiss } from '$lib/stores/toasts.svelte';
</script>

<div class="stack" aria-live="polite">
  {#each toasts as item (item.id)}
    <button class="toast {item.kind}" transition:fly={{ y: 12, duration: 220 }} onclick={() => dismiss(item.id)}>
      {#if item.kind === 'success'}<CircleCheck size={16} />{:else if item.kind === 'error'}<CircleAlert
          size={16}
        />{:else}<Info size={16} />{/if}
      <span>{item.text}</span>
    </button>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    bottom: var(--space-5);
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    /* Topmost layer: feedback must show above the media viewer (100) and dialogs (120). */
    z-index: 200;
    pointer-events: none;
  }
  .toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    border: 0;
    border-radius: 999px;
    background: var(--navy-900);
    color: #fff;
    font-size: var(--text-sm);
    font-weight: 550;
    box-shadow: 0 10px 30px -8px rgba(5, 7, 12, 0.6);
  }
  .success :global(svg) {
    color: var(--ok-500);
  }
  .error :global(svg) {
    color: var(--rec-500);
  }
  .info :global(svg) {
    color: var(--signal-400);
  }
</style>
