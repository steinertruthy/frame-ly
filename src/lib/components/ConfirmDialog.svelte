<script lang="ts">
  import { onMount } from 'svelte';
  import Button from './Button.svelte';
  interface Props {
    title: string;
    body?: string;
    confirm: string;
    cancel: string;
    danger?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }
  let { title, body, confirm, cancel, danger = false, onconfirm, oncancel }: Props = $props();
  let dialog: HTMLDivElement | undefined = $state();
  onMount(() => dialog?.querySelector<HTMLButtonElement>('.primary-action button')?.focus());
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && (e.stopPropagation(), oncancel())} />

<div class="scrim" role="presentation" onpointerdown={oncancel}></div>
<div class="dialog" role="alertdialog" aria-modal="true" aria-label={title} bind:this={dialog}>
  <h3>{title}</h3>
  {#if body}<p>{body}</p>{/if}
  <div class="actions">
    <Button variant="ghost" onclick={oncancel}>{cancel}</Button>
    <span class="primary-action" class:danger><Button variant="primary" onclick={onconfirm}>{confirm}</Button></span>
  </div>
</div>

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 120;
    background: rgba(5, 7, 12, 0.4);
    animation: fade var(--dur-med);
  }
  .dialog {
    position: fixed;
    z-index: 121;
    left: 50%;
    top: 50%;
    width: min(380px, calc(100vw - 32px));
    padding: 20px;
    border-radius: var(--radius-lg);
    background: var(--surface);
    box-shadow:
      0 0 0 1px var(--border-strong),
      0 24px 60px -20px rgba(5, 7, 12, 0.6);
    transform: translate(-50%, -50%);
    animation: pop var(--dur-med) var(--ease-spring);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translate(-50%, -48%) scale(0.97);
    }
  }
  h3 {
    font-size: var(--text-lg);
    font-weight: 650;
  }
  p {
    margin-top: 6px;
    color: var(--text-2);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 18px;
  }
  .danger :global(.primary) {
    background: var(--rec-500);
    box-shadow: none;
  }
</style>
