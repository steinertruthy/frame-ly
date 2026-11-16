<script lang="ts">
  import type { Snippet } from 'svelte';
  import { X } from '@lucide/svelte';
  import { t } from '$lib/i18n/index.svelte';
  interface Props {
    title: string;
    width?: number;
    onclose: () => void;
    children: Snippet;
    footer?: Snippet;
  }
  let { title, width = 560, onclose, children, footer }: Props = $props();
  /** Rendered at the top of the document so page transforms don't confine it. */
  function portal(node: HTMLElement) {
    document.body.appendChild(node);
    return { destroy: () => node.remove() };
  }
</script>

<!-- Escape closes a confirmation shown over the sheet first, not the sheet. -->
<svelte:window onkeydown={(e) => e.key === 'Escape' && !document.querySelector('[role="alertdialog"]') && (e.stopPropagation(), onclose())} />

<div class="layer" use:portal>
  <div class="scrim" role="presentation" onpointerdown={onclose}></div>
  <div class="sheet" role="dialog" aria-modal="true" aria-label={title} style:width="min({width}px, calc(100vw - 32px))">
    <header>
      <h3>{title}</h3>
      <button class="close" onclick={onclose} aria-label={t('rec.options.close')} title={t('rec.options.close')}><X size={16} /></button>
    </header>
    <div class="body">{@render children()}</div>
    {#if footer}<footer>{@render footer()}</footer>{/if}
  </div>
</div>

<style>
  .layer {
    position: fixed;
    inset: 0;
    z-index: 110;
    display: grid;
    place-items: center;
  }
  .scrim {
    position: absolute;
    inset: 0;
    background: rgba(5, 7, 12, 0.45);
    animation: fade var(--dur-med);
  }
  .sheet {
    position: relative;
    display: flex;
    flex-direction: column;
    max-height: calc(100vh - 64px);
    border-radius: var(--radius-xl);
    background: var(--surface);
    box-shadow:
      inset 0 0 0 1px var(--border-strong),
      0 30px 80px -30px rgba(0, 0, 0, 0.8);
    animation: pop var(--dur-med) var(--ease-out);
  }
  @keyframes fade {
    from {
      opacity: 0;
    }
  }
  @keyframes pop {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.985);
    }
  }
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 18px 6px 20px;
  }
  h3 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 650;
  }
  .close {
    display: grid;
    place-items: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-3);
  }
  .close:hover {
    background: var(--surface-hover);
    color: var(--text);
  }
  .body {
    min-height: 0;
    overflow-y: auto;
    padding: 8px 20px 18px;
  }
  footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding: 12px 18px 16px;
    box-shadow: inset 0 1px 0 var(--border);
  }
</style>
