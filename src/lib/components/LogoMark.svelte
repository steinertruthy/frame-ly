<script lang="ts">
  /** `tile`: the mark drawn in theme colors on a quiet tile (sidebar), instead of the app-icon gradient. */
  let { size = 28, tile = false }: { size?: number; tile?: boolean } = $props();
</script>

{#if tile}
  <span class="tile" style="--s:{size}px" aria-hidden="true">
    <svg width={Math.round(size * 0.5)} height={Math.round(size * 0.5)} viewBox="250 250 524 524">
      <g fill="none" stroke="currentColor" stroke-width="64" stroke-linecap="round" stroke-linejoin="round">
        <path d="M282 410 V336 a54 54 0 0 1 54 -54 H410" />
        <path d="M614 282 H688 a54 54 0 0 1 54 54 V410" />
        <path d="M742 614 V688 a54 54 0 0 1 -54 54 H614" />
        <path d="M410 742 H336 a54 54 0 0 1 -54 -54 V614" />
      </g>
      <circle cx="512" cy="512" r="80" fill="var(--accent)" />
    </svg>
  </span>
{:else}
  <svg width={size} height={size} viewBox="100 100 824 824" aria-hidden="true">
    <defs>
      <linearGradient id="lm-bg" x1="0" y1="0" x2="1" y2="1">
        <stop offset="0" stop-color="#2148A8" />
        <stop offset="0.5" stop-color="#0F2259" />
        <stop offset="1" stop-color="#060D26" />
      </linearGradient>
    </defs>
    <rect x="100" y="100" width="824" height="824" rx="186" fill="url(#lm-bg)" />
    <g fill="none" stroke="#fff" stroke-width="56" stroke-linecap="round" stroke-linejoin="round">
      <path d="M282 410 V336 a54 54 0 0 1 54 -54 H410" />
      <path d="M614 282 H688 a54 54 0 0 1 54 54 V410" />
      <path d="M742 614 V688 a54 54 0 0 1 -54 54 H614" />
      <path d="M410 742 H336 a54 54 0 0 1 -54 -54 V614" />
    </g>
    <circle cx="512" cy="512" r="96" fill="#5B8CFF" />
  </svg>
{/if}

<style>
  .tile {
    display: grid;
    place-items: center;
    width: var(--s);
    height: var(--s);
    flex-shrink: 0;
    border-radius: calc(var(--s) * 0.28);
    background: var(--brand-tile);
    box-shadow: inset 0 0 0 1px var(--nav-active-border);
    color: var(--icon-accent);
  }
</style>
