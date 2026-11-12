<script lang="ts">
  /**
   * Ambient light behind the interface: three large, blurred blue glows drifting very slowly,
   * plus a faint grain for depth. Transform-only animation (GPU-composited, no layout work);
   * frozen when motion is reduced, hidden when the user turns ambient light off.
   */
  let { intensity = 1 }: { intensity?: number } = $props();
</script>

<div class="aurora" aria-hidden="true" style="--k:{intensity}">
  <span class="glow g1"></span>
  <span class="glow g2"></span>
  <span class="glow g3"></span>
  <span class="grain"></span>
</div>

<style>
  .aurora {
    position: fixed;
    inset: 0;
    z-index: 0;
    overflow: hidden;
    pointer-events: none;
    opacity: var(--k);
    transition: opacity var(--dur-slow) var(--ease-out);
  }
  :global(:root[data-ambient='off']) .aurora {
    opacity: 0;
  }
  .glow {
    position: absolute;
    border-radius: 50%;
    filter: blur(70px);
    will-change: transform;
  }
  .g1 {
    width: 70vmax;
    height: 38vmax;
    left: -18vmax;
    top: 18%;
    background: radial-gradient(closest-side, var(--aurora-1), transparent);
    animation: drift-a 46s var(--ease-inout) infinite alternate;
  }
  .g2 {
    width: 50vmax;
    height: 26vmax;
    right: -14vmax;
    top: 30%;
    background: radial-gradient(closest-side, var(--aurora-2), transparent);
    animation: drift-b 38s var(--ease-inout) infinite alternate;
  }
  .g3 {
    width: 60vmax;
    height: 30vmax;
    left: 10%;
    bottom: -18vmax;
    background: radial-gradient(closest-side, var(--aurora-3), transparent);
    animation: drift-c 54s var(--ease-inout) infinite alternate;
  }
  @keyframes drift-a {
    to {
      transform: translate(14vmax, -6vmax) rotate(8deg) scale(1.1);
    }
  }
  @keyframes drift-b {
    to {
      transform: translate(-16vmax, 8vmax) rotate(-10deg) scale(1.15);
    }
  }
  @keyframes drift-c {
    to {
      transform: translate(10vmax, -8vmax) scale(0.9);
    }
  }
  .grain {
    position: absolute;
    inset: -50%;
    opacity: var(--grain-opacity);
    background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='160' height='160'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='3' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
  }
  :global(:root[data-motion='reduced']) .glow {
    animation: none;
  }
  @media (prefers-reduced-motion: reduce) {
    :global(:root:not([data-motion='full'])) .glow {
      animation: none;
    }
  }
</style>
