<script lang="ts">
  /** Peak meter: `level` is linear 0…1; displayed on a dB scale (-60…0 dBFS). */
  let { level, segments = 14, label }: { level: number; segments?: number; label?: string } = $props();
  const db = $derived(level > 0 ? 20 * Math.log10(level) : -Infinity);
  const lit = $derived(Math.max(0, Math.min(segments, Math.round(((db + 60) / 60) * segments))));
</script>

<div class="meter" role="meter" aria-label={label} aria-valuemin={-60} aria-valuemax={0} aria-valuenow={Math.max(-60, Math.round(db))}>
  {#each { length: segments } as _, i (i)}
    <span class="seg" class:on={i < lit} class:hot={i >= segments - 2} class:warm={i >= segments - 5 && i < segments - 2}></span>
  {/each}
</div>

<style>
  .meter {
    display: flex;
    gap: 2px;
    align-items: center;
  }
  .seg {
    width: 4px;
    height: 12px;
    border-radius: 1.5px;
    background: var(--border-strong);
    transition: background 80ms linear;
  }
  .seg.on {
    background: var(--ok-500);
  }
  .seg.on.warm {
    background: var(--warn-500);
  }
  .seg.on.hot {
    background: var(--rec-500);
  }
</style>
