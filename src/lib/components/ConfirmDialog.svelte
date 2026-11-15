<script lang="ts">
  import { onMount } from 'svelte';
  import { Check, Info, TriangleAlert } from '@lucide/svelte';
  import Button from './Button.svelte';
  interface Props {
    title: string;
    body?: string;
    /** Short points under the title (icon by tone), for notices that must be scannable. */
    points?: { tone: 'ok' | 'warn' | 'info'; text: string }[];
    /** Small closing line under the points. */
    note?: string;
    confirm: string;
    cancel: string;
    danger?: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }
  let { title, body, points, note, confirm, cancel, danger = false, onconfirm, oncancel }: Props = $props();
  let dialog: HTMLDivElement | undefined = $state();
  onMount(() => dialog?.querySelector<HTMLButtonElement>('.primary-action button')?.focus());
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && (e.stopPropagation(), oncancel())} />

<div class="scrim" role="presentation" onpointerdown={oncancel}></div>
<div class="dialog" class:wide={!!points?.length} role="alertdialog" aria-modal="true" aria-label={title} bind:this={dialog}>
  <h3>{title}</h3>
  {#if body}<p>{body}</p>{/if}
  {#if points?.length}
    <ul class="points">
      {#each points as pt (pt.text)}
        <li class={pt.tone}>
          {#if pt.tone === 'ok'}<Check size={15} />{:else if pt.tone === 'warn'}<TriangleAlert size={15} />{:else}<Info size={15} />{/if}
          <span>{pt.text}</span>
        </li>
      {/each}
    </ul>
  {/if}
  {#if note}<p class="note">{note}</p>{/if}
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
  .dialog.wide {
    width: min(440px, calc(100vw - 32px));
  }
  .points {
    display: grid;
    gap: 10px;
    margin: 14px 0 0;
    padding: 0;
    list-style: none;
  }
  .points li {
    display: grid;
    grid-template-columns: 16px 1fr;
    gap: 10px;
    align-items: start;
    color: var(--text);
    font-size: var(--text-sm);
    line-height: 1.45;
  }
  .points li > :global(svg) {
    margin-top: 2px;
  }
  .points .ok > :global(svg) {
    color: var(--ok-500);
  }
  .points .warn > :global(svg) {
    color: var(--warn-500);
  }
  .points .info > :global(svg) {
    color: var(--icon-accent);
  }
  .note {
    margin-top: 14px;
    color: var(--text-3);
    font-size: var(--text-xs);
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
