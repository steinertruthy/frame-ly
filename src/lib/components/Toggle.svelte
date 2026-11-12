<script lang="ts">
  interface Props {
    checked: boolean;
    label: string;
    disabled?: boolean;
    onchange: (value: boolean) => void;
  }
  let { checked, label, disabled = false, onchange }: Props = $props();
</script>

<button
  class="toggle"
  class:on={checked}
  role="switch"
  aria-checked={checked}
  aria-label={label}
  {disabled}
  onclick={() => onchange(!checked)}
>
  <span class="knob"></span>
</button>

<style>
  .toggle {
    position: relative;
    width: 38px;
    height: 22px;
    box-shadow: inset 0 0 0 1px var(--border);
    padding: 0;
    border: 0;
    border-radius: 999px;
    background: var(--border-strong);
    transition: background var(--dur-med) var(--ease-out);
    flex-shrink: 0;
  }
  .toggle.on {
    background: var(--primary-fill);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.2),
      0 0 14px -4px var(--glow);
  }
  .toggle:disabled {
    opacity: 0.45;
  }
  .knob {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 1px 3px rgba(6, 13, 38, 0.3);
    transition: transform var(--dur-med) var(--ease-spring);
  }
  .on .knob {
    transform: translateX(16px);
  }
</style>
