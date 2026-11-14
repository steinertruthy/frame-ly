<script lang="ts">
  import { onMount } from 'svelte';
  import { commands } from '$lib/ipc/commands';
  import { setLanguage, t } from '$lib/i18n/index.svelte';

  let { seconds }: { seconds: number } = $props();
  let left = $state(0);

  onMount(() => {
    left = seconds;
    void commands.getSettings().then((s) => setLanguage(s.general.language));
    // Visual only: the backend owns the real timer and starts recording on its own.
    const started = performance.now();
    const id = setInterval(() => {
      left = Math.max(1, seconds - Math.floor((performance.now() - started) / 1000));
    }, 100);
    return () => clearInterval(id);
  });

  const cancel = () => void commands.cancelCountdown();
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && cancel()} />

<button class="wrap" onclick={cancel} aria-label={t('record.cancelHint')}>
  <span class="ring" style="--s:{seconds}s"></span>
  {#key left}
    <span class="num">{left}</span>
  {/key}
  <span class="hint">{t('record.cancelHint')}</span>
</button>

<style>
  :global(html),
  :global(body) {
    background: transparent !important;
  }
  .wrap {
    position: fixed;
    inset: 30px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: 50%;
    background: rgba(5, 7, 12, 0.82);
    backdrop-filter: blur(16px);
    color: #fff;
    box-shadow:
      inset 0 0 0 1px rgba(255, 255, 255, 0.08),
      0 10px 24px -6px rgba(0, 0, 0, 0.55);
    cursor: pointer;
  }
  .ring {
    position: absolute;
    inset: 8px;
    border-radius: 50%;
    background: conic-gradient(var(--rec-500) var(--p, 100%), transparent 0);
    mask: radial-gradient(farthest-side, transparent calc(100% - 4px), #000 0);
    animation: drain var(--s) linear forwards;
  }
  @property --p {
    syntax: '<percentage>';
    inherits: false;
    initial-value: 100%;
  }
  @keyframes drain {
    to {
      --p: 0%;
    }
  }
  .num {
    margin-top: -16px;
    font: 700 84px/1 var(--font-display);
    font-variant-numeric: tabular-nums;
    animation: pop 1s var(--ease-out);
  }
  @keyframes pop {
    from {
      transform: scale(1.35);
      opacity: 0;
    }
    30% {
      opacity: 1;
    }
  }
  .hint {
    position: absolute;
    bottom: 30px;
    max-width: 130px;
    font-size: 11px;
    line-height: 1.3;
    text-align: center;
    color: rgba(255, 255, 255, 0.6);
  }
</style>
