<script lang="ts">
  interface Props {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    label: string;
    disabled?: boolean;
    format?: (v: number) => string;
    onchange: (value: number) => void;
  }
  let { value, min = 0, max = 2, step = 0.05, label, disabled = false, format = (v) => `${Math.round(v * 100)}%`, onchange }: Props =
    $props();
  // Live value while dragging; committed on release to avoid saving on every pixel.
  let live = $state<number | null>(null);
  const shown = $derived(live ?? value);
  const pct = $derived(((shown - min) / (max - min)) * 100);
</script>

<div class="slider" class:disabled>
  <input
    type="range"
    aria-label={label}
    {min}
    {max}
    {step}
    {disabled}
    value={shown}
    style="--pct:{pct}%"
    oninput={(e) => (live = Number(e.currentTarget.value))}
    onchange={(e) => {
      onchange(Number(e.currentTarget.value));
      live = null;
    }}
  />
  <span class="value">{format(shown)}</span>
</div>

<style>
  .slider {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .slider.disabled {
    opacity: 0.5;
  }
  input {
    appearance: none;
    width: 180px;
    height: 4px;
    border-radius: 999px;
    background: linear-gradient(to right, var(--accent) var(--pct), var(--border-strong) var(--pct));
  }
  input::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #fff;
    box-shadow:
      0 1px 3px rgba(5, 7, 12, 0.35),
      0 0 0 0.5px rgba(5, 7, 12, 0.2);
    transition: transform var(--dur-fast);
  }
  input:active::-webkit-slider-thumb {
    transform: scale(1.12);
  }
  .value {
    width: 42px;
    text-align: right;
    color: var(--text-2);
    font: 600 var(--text-sm) var(--font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>
