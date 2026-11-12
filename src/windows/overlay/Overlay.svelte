<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { commands } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { logUi } from '$lib/ipc/log';
  import { setLanguage, t } from '$lib/i18n/index.svelte';
  import type { OverlayInit, Rect, SelectMode } from '$lib/ipc/types';
  import { clampRect, hitTest, rectFromPoints, windowToLocal } from './geometry';

  let { displayId }: { displayId: string } = $props();

  const MIN_SIZE = 4;
  const LOUPE = 128; // CSS px
  const LOUPE_PIXELS = 15; // frame pixels across the loupe

  let init = $state<OverlayInit | null>(null);
  let mode = $state<SelectMode>('region');
  let vw = $state(window.innerWidth);
  let vh = $state(window.innerHeight);
  let pointer = $state<{ x: number; y: number } | null>(null);
  let dragStart = $state<{ x: number; y: number } | null>(null);
  let busy = $state(false);
  let showHints = $state(true);
  let img: HTMLImageElement;
  let loupe: HTMLCanvasElement | undefined = $state();

  // Frame pixels per CSS pixel (2 on Retina, 1.5 at 150% on Windows…).
  const scale = $derived(init ? init.frameWidth / vw : 1);
  const selection = $derived(dragStart && pointer ? clampRect(rectFromPoints(dragStart, pointer), vw, vh) : null);
  const windows = $derived(
    init ? init.windows.map((w) => ({ info: w, rect: clampRect(windowToLocal(w, init!.display, vw), vw, vh) })) : [],
  );
  const hovered = $derived(mode === 'window' && pointer ? hitTest(windows, pointer.x, pointer.y) : undefined);
  const focusRect = $derived<Rect | null>(mode === 'region' ? selection : (hovered?.rect ?? null));

  const px = (v: number) => Math.round(v * scale);

  onMount(() => {
    void (async () => {
      try {
        const [data, settings] = await Promise.all([commands.overlayInit(displayId), commands.getSettings()]);
        setLanguage(settings.general.language);
        init = data;
        mode = data.mode;
        img.src = framelyUrl(`frame/${encodeURIComponent(displayId)}`);
        await img.decode();
        const win = getCurrentWindow();
        await win.show();
        await win.setFocus();
        setTimeout(() => (showHints = false), 2800);
      } catch (e) {
        logUi('error', `overlay init failed: ${JSON.stringify(e)}`);
        void commands.cancelCapture();
      }
    })();
    const onResize = () => {
      vw = window.innerWidth;
      vh = window.innerHeight;
    };
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  });

  $effect(() => {
    if (!loupe || !pointer || !init || mode !== 'region') return;
    const dpr = window.devicePixelRatio || 1;
    loupe.width = LOUPE * dpr;
    loupe.height = LOUPE * dpr;
    const ctx = loupe.getContext('2d');
    if (!ctx) return;
    ctx.imageSmoothingEnabled = false;
    const cx = Math.floor(pointer.x * scale);
    const cy = Math.floor(pointer.y * scale);
    const half = Math.floor(LOUPE_PIXELS / 2);
    const cell = (LOUPE * dpr) / LOUPE_PIXELS;
    ctx.fillStyle = '#000';
    ctx.fillRect(0, 0, loupe.width, loupe.height);
    ctx.drawImage(img, cx - half, cy - half, LOUPE_PIXELS, LOUPE_PIXELS, 0, 0, loupe.width, loupe.height);
    ctx.strokeStyle = 'rgba(255,255,255,0.08)';
    ctx.lineWidth = 1;
    for (let i = 1; i < LOUPE_PIXELS; i++) {
      ctx.beginPath();
      ctx.moveTo(i * cell, 0);
      ctx.lineTo(i * cell, loupe.height);
      ctx.moveTo(0, i * cell);
      ctx.lineTo(loupe.width, i * cell);
      ctx.stroke();
    }
    ctx.strokeStyle = '#fff';
    ctx.lineWidth = 2 * dpr;
    ctx.strokeRect(half * cell, half * cell, cell, cell);
  });

  async function finishRegion(r: Rect) {
    busy = true;
    try {
      await commands.finishRegion(displayId, r, { width: vw, height: vh });
    } catch (e) {
      logUi('error', `finish_region failed: ${JSON.stringify(e)}`);
      void commands.cancelCapture();
    }
  }

  async function finishWindow(id: string) {
    busy = true;
    try {
      await commands.finishWindow(id);
    } catch (e) {
      logUi('error', `finish_window failed: ${JSON.stringify(e)}`);
      void commands.cancelCapture();
    }
  }

  function onpointerdown(e: PointerEvent) {
    if (busy || e.button !== 0) return;
    if (mode === 'window') {
      if (hovered) void finishWindow(hovered.info.id);
      return;
    }
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragStart = { x: e.clientX, y: e.clientY };
    pointer = { x: e.clientX, y: e.clientY };
  }

  function onpointermove(e: PointerEvent) {
    pointer = { x: e.clientX, y: e.clientY };
  }

  function onpointerup() {
    if (busy || mode !== 'region' || !selection) return;
    const r = selection;
    dragStart = null;
    if (r.width >= MIN_SIZE && r.height >= MIN_SIZE) void finishRegion(r);
  }

  function onkeydown(e: KeyboardEvent) {
    if (busy) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      if (dragStart) dragStart = null;
      else void commands.cancelCapture();
    } else if (e.key === ' ') {
      e.preventDefault();
      dragStart = null;
      mode = mode === 'region' ? 'window' : 'region';
    }
  }

  const loupePos = $derived.by(() => {
    if (!pointer) return { x: 0, y: 0 };
    const gap = 22;
    const x = pointer.x + gap + LOUPE > vw ? pointer.x - gap - LOUPE : pointer.x + gap;
    const y = pointer.y + gap + LOUPE + 28 > vh ? pointer.y - gap - LOUPE - 28 : pointer.y + gap;
    return { x, y };
  });
</script>

<svelte:window {onkeydown} oncontextmenu={(e) => e.preventDefault()} />

<div
  class="root"
  class:window-mode={mode === 'window'}
  class:record={init?.purpose === 'record'}
  role="application"
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  onpointerleave={() => !dragStart && (pointer = null)}
>
  <img bind:this={img} alt="" class="frame" draggable="false" />

  {#if focusRect && focusRect.width > 0 && focusRect.height > 0}
    <div
      class="focus"
      class:window={mode === 'window'}
      style="left:{focusRect.x}px;top:{focusRect.y}px;width:{focusRect.width}px;height:{focusRect.height}px"
    >
      {#if mode === 'region'}
        <span class="handle tl"></span><span class="handle tr"></span><span class="handle bl"></span><span class="handle br"></span>
      {/if}
      <div class="size" class:inside={focusRect.y < 34}>
        {#if mode === 'window' && hovered}
          <strong>{hovered.info.appName}</strong>
        {/if}
        {px(focusRect.width)} × {px(focusRect.height)}
      </div>
    </div>
  {:else}
    <div class="dim"></div>
  {/if}

  {#if mode === 'region' && pointer && !busy}
    {#if !dragStart}
      <div class="cross h" style="top:{pointer.y}px"></div>
      <div class="cross v" style="left:{pointer.x}px"></div>
    {/if}
    <div class="loupe" style="left:{loupePos.x}px;top:{loupePos.y}px">
      <canvas bind:this={loupe} style="width:{LOUPE}px;height:{LOUPE}px"></canvas>
      <div class="coords">{px(pointer.x)}, {px(pointer.y)}</div>
    </div>
  {/if}

  {#if init}
    <div class="hints" class:hidden={!showHints}>
      {#if init.purpose === 'record'}<span class="rec-dot"></span>{/if}
      <span>
        {t(
          init.purpose === 'record'
            ? mode === 'region'
              ? 'overlay.hint.record.region'
              : 'overlay.hint.record.window'
            : mode === 'region'
              ? 'overlay.hint.region'
              : 'overlay.hint.window',
        )}
      </span>
      <span class="sep"></span>
      <span>{t(mode === 'region' ? 'overlay.hint.toggle.region' : 'overlay.hint.toggle.window')}</span>
      <span class="sep"></span>
      <span>{t('overlay.hint.cancel')}</span>
    </div>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    background: #000 !important;
  }
  .root {
    position: fixed;
    inset: 0;
    overflow: hidden;
    cursor: crosshair;
    touch-action: none;
  }
  .root.window-mode {
    cursor: default;
  }
  .frame {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    pointer-events: none;
    user-select: none;
  }
  .dim {
    position: absolute;
    inset: 0;
    background: rgba(4, 10, 30, 0.38);
  }
  .focus {
    position: absolute;
    outline: 1.5px solid var(--signal-400);
    box-shadow: 0 0 0 100vmax rgba(4, 10, 30, 0.38);
  }
  .focus.window {
    background: rgba(74, 123, 255, 0.14);
    outline-width: 3px;
    outline-offset: -3px;
    border-radius: 10px;
    transition:
      left 90ms var(--ease-out),
      top 90ms var(--ease-out),
      width 90ms var(--ease-out),
      height 90ms var(--ease-out);
  }
  .handle {
    position: absolute;
    width: 9px;
    height: 9px;
    border-radius: 50%;
    background: #fff;
    box-shadow: 0 0 0 1.5px var(--signal-500);
  }
  .tl {
    left: -5px;
    top: -5px;
  }
  .tr {
    right: -5px;
    top: -5px;
  }
  .bl {
    left: -5px;
    bottom: -5px;
  }
  .br {
    right: -5px;
    bottom: -5px;
  }
  .size {
    position: absolute;
    left: 0;
    top: -30px;
    display: flex;
    gap: 6px;
    align-items: center;
    padding: 3px 9px;
    border-radius: 7px;
    background: rgba(6, 13, 38, 0.86);
    color: #fff;
    font: 600 12px/18px var(--font-mono);
    white-space: nowrap;
    backdrop-filter: blur(8px);
  }
  .size strong {
    font-family: var(--font-ui);
  }
  .size.inside {
    top: 8px;
    left: 8px;
  }
  .cross {
    position: absolute;
    background: rgba(255, 255, 255, 0.55);
    pointer-events: none;
    mix-blend-mode: difference;
  }
  .cross.h {
    left: 0;
    right: 0;
    height: 1px;
  }
  .cross.v {
    top: 0;
    bottom: 0;
    width: 1px;
  }
  .loupe {
    position: absolute;
    pointer-events: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }
  .loupe canvas {
    border-radius: 50%;
    box-shadow:
      0 0 0 2px #fff,
      0 10px 30px rgba(0, 0, 0, 0.45);
  }
  .coords {
    padding: 2px 8px;
    border-radius: 6px;
    background: rgba(6, 13, 38, 0.86);
    color: #fff;
    font: 600 11px/16px var(--font-mono);
  }
  .hints {
    position: absolute;
    top: 28px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 16px;
    border-radius: 999px;
    background: rgba(6, 13, 38, 0.86);
    color: rgba(255, 255, 255, 0.92);
    font-size: 12px;
    font-weight: 550;
    pointer-events: none;
    backdrop-filter: blur(10px);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.35);
    transition:
      opacity var(--dur-slow) var(--ease-out),
      transform var(--dur-slow) var(--ease-out);
  }
  .hints.hidden {
    opacity: 0;
    transform: translate(-50%, -8px);
  }
  /* Recording selections use the recording color instead of the brand blue. */
  .record .focus {
    outline-color: var(--rec-500);
  }
  .record .focus.window {
    background: rgba(255, 75, 92, 0.12);
  }
  .record .handle {
    box-shadow: 0 0 0 1.5px var(--rec-500);
  }
  .rec-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--rec-500);
  }
  .sep {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.4);
  }
</style>
