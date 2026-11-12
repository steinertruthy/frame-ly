<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask } from '@tauri-apps/plugin-dialog';
  import {
    MousePointer2,
    MoveUpRight,
    Square,
    Circle,
    Type,
    Highlighter,
    Grid3x3,
    Crop,
    Undo2,
    Redo2,
    Copy,
    Scaling,
  } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import Toaster from '$lib/components/Toaster.svelte';
  import Button from '$lib/components/Button.svelte';
  import { commands, events, isAppError } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { logUi } from '$lib/ipc/log';
  import { t, type MessageKey } from '$lib/i18n/index.svelte';
  import { toast } from '$lib/stores/toasts.svelte';
  import {
    COLORS,
    HIGHLIGHT,
    History,
    baseStroke,
    bounds,
    clampRect,
    isMeaningful,
    moved,
    newDoc,
    normRect,
    outputSize,
    resizeKeepingAspect,
    shapeAt,
    type Doc,
    type Point,
    type Rect,
    type Shape,
    type Tool,
  } from './model';
  import { drawDoc, exportPng, font, measureText } from './render';

  let { platform }: { platform: string } = $props();

  // ── State ──────────────────────────────────────────────────────────────
  let name = $state('');
  let image = $state<HTMLImageElement | null>(null);
  let doc = $state<Doc>(newDoc(1, 1));
  let history = new History();
  let historyTick = $state(0); // bumps so undo/redo buttons re-evaluate
  let dirty = $state(false);
  let tool = $state<Tool>('arrow');
  let color = $state<string>(COLORS[0]);
  let level = $state<1 | 2 | 3>(2);
  let selectedId = $state<number | null>(null);
  let draft = $state<Shape | null>(null);
  let cropDraft = $state<Rect | null>(null);
  let textEdit = $state<{ x: number; y: number; value: string; id: number | null } | null>(null);
  let resizeOpen = $state(false);
  let busy = $state(false);
  let nextId = 1;

  let stage: HTMLDivElement | undefined = $state();
  let canvas: HTMLCanvasElement | undefined = $state();
  let textArea: HTMLTextAreaElement | undefined = $state();
  let stageW = $state(800);
  let stageH = $state(600);
  const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;

  const imageSize = $derived(image ? { w: image.naturalWidth, h: image.naturalHeight } : { w: 1, h: 1 });
  const full = $derived<Rect>({ x: 0, y: 0, w: imageSize.w, h: imageSize.h });
  const region = $derived(tool === 'crop' ? full : doc.crop);
  const PAD = 32;
  const scale = $derived(Math.min((stageW - PAD * 2) / region.w, (stageH - PAD * 2) / region.h, 1));
  const unit = $derived(baseStroke(imageSize.w, imageSize.h));
  const strokeWidth = $derived(unit * level);
  const textSize = $derived(unit * (5 + level * 3));
  const selected = $derived(doc.shapes.find((s) => s.id === selectedId) ?? null);
  const out = $derived(outputSize(doc));
  // `history` is a plain object; `historyTick` makes these re-evaluate after each change.
  const canUndo = $derived.by(() => (void historyTick, history.canUndo));
  const canRedo = $derived.by(() => (void historyTick, history.canRedo));

  // ── Loading ────────────────────────────────────────────────────────────
  async function load() {
    try {
      const src = await commands.editorSource();
      const img = new Image();
      img.crossOrigin = 'anonymous'; // served with CORS so the canvas stays exportable
      img.src = framelyUrl(`edit-source?v=${Date.now()}`);
      await img.decode();
      name = src.name;
      image = img;
      doc = newDoc(img.naturalWidth, img.naturalHeight);
      history = new History();
      historyTick++;
      dirty = false;
      selectedId = null;
      tool = 'arrow';
      void getCurrentWindow().setTitle(`${src.name} — FrameLy`);
      // Health check: if the source were cross-origin without CORS the canvas would be
      // "tainted" and Save/Copy would fail later; surface that in the log right away.
      try {
        const probe = new OffscreenCanvas(2, 2);
        probe.getContext('2d')!.drawImage(img, 0, 0, 2, 2);
        await probe.convertToBlob();
        logUi('info', `editor ready: ${img.naturalWidth}x${img.naturalHeight}, export ok`);
      } catch (e) {
        logUi('error', `editor export unavailable (tainted canvas?): ${e}`);
      }
    } catch (e) {
      logUi('error', `editor load failed: ${JSON.stringify(e)}`);
    }
  }

  onMount(() => {
    void load();
    const unLoad = events.onEditorLoad(() => {
      void confirmDiscard().then((ok) => {
        if (ok) void load();
      });
    });
    const ro = new ResizeObserver(([entry]) => {
      stageW = entry!.contentRect.width;
      stageH = entry!.contentRect.height;
    });
    if (stage) ro.observe(stage);
    const unClose = getCurrentWindow().onCloseRequested(async (e) => {
      if (!(await confirmDiscard())) e.preventDefault();
    });
    return () => {
      ro.disconnect();
      void unLoad.then((u) => u());
      void unClose.then((u) => u());
    };
  });

  async function confirmDiscard(): Promise<boolean> {
    if (!dirty) return true;
    return ask(t('editor.discard'), { title: name, kind: 'warning' });
  }

  // ── Document changes ───────────────────────────────────────────────────
  function change(mutate: (d: Doc) => void) {
    history.record($state.snapshot(doc) as Doc);
    const next = $state.snapshot(doc) as Doc;
    mutate(next);
    doc = next;
    dirty = true;
    historyTick++;
  }

  function undo() {
    const prev = history.undo($state.snapshot(doc) as Doc);
    if (prev) {
      doc = prev;
      selectedId = null;
      dirty = true;
      historyTick++;
    }
  }

  function redo() {
    const next = history.redo($state.snapshot(doc) as Doc);
    if (next) {
      doc = next;
      dirty = true;
      historyTick++;
    }
  }

  // ── Rendering ──────────────────────────────────────────────────────────
  $effect(() => {
    if (!canvas || !image) return;
    const ctx = canvas.getContext('2d')!;
    const w = Math.round(region.w * scale * dpr);
    const h = Math.round(region.h * scale * dpr);
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;
    ctx.clearRect(0, 0, w, h);
    const shapes = draft ? [...doc.shapes, draft] : doc.shapes;
    const visible = textEdit?.id != null ? shapes.filter((s) => s.id !== textEdit!.id) : shapes;
    drawDoc(ctx, image, imageSize, visible, { region, scale: scale * dpr });

    ctx.save();
    ctx.scale(scale * dpr, scale * dpr);
    ctx.translate(-region.x, -region.y);
    const px = 1 / (scale * dpr); // one device pixel in image units
    if (selected && tool === 'select') {
      const b = bounds(selected);
      const pad = 6 * px * dpr;
      ctx.strokeStyle = '#4A7BFF';
      ctx.lineWidth = 1.5 * px * dpr;
      ctx.setLineDash([5 * px * dpr, 4 * px * dpr]);
      ctx.strokeRect(b.x - pad, b.y - pad, b.w + pad * 2, b.h + pad * 2);
    }
    if (tool === 'crop') {
      const c = cropDraft ?? doc.crop;
      ctx.fillStyle = 'rgba(4,10,30,0.55)';
      ctx.beginPath();
      ctx.rect(0, 0, imageSize.w, imageSize.h);
      ctx.rect(c.x, c.y, c.w, c.h);
      ctx.fill('evenodd');
      ctx.strokeStyle = '#fff';
      ctx.lineWidth = 2 * px * dpr;
      ctx.setLineDash([]);
      ctx.strokeRect(c.x, c.y, c.w, c.h);
      ctx.strokeStyle = 'rgba(255,255,255,0.35)';
      ctx.lineWidth = 1 * px * dpr;
      for (const f of [1 / 3, 2 / 3]) {
        ctx.beginPath();
        ctx.moveTo(c.x + c.w * f, c.y);
        ctx.lineTo(c.x + c.w * f, c.y + c.h);
        ctx.moveTo(c.x, c.y + c.h * f);
        ctx.lineTo(c.x + c.w, c.y + c.h * f);
        ctx.stroke();
      }
    }
    ctx.restore();
  });

  // ── Pointer interaction ────────────────────────────────────────────────
  let dragStart: Point | null = null;
  let moveOrigin: Shape | null = null;
  let movedSinceDown = false;

  function toImage(e: PointerEvent | MouseEvent): Point {
    const r = canvas!.getBoundingClientRect();
    return { x: (e.clientX - r.left) / scale + region.x, y: (e.clientY - r.top) / scale + region.y };
  }

  function draftFor(a: Point, b: Point): Shape | null {
    const id = nextId;
    const r = normRect(a, b);
    switch (tool) {
      case 'arrow':
        return { id, type: 'arrow', x1: a.x, y1: a.y, x2: b.x, y2: b.y, color, width: strokeWidth };
      case 'rect':
      case 'ellipse':
        return { id, type: tool, ...r, color, width: strokeWidth };
      case 'highlight':
        return { id, type: 'highlight', ...r, color: HIGHLIGHT };
      case 'blur':
        return { id, type: 'blur', ...clampRect(r, doc.crop) };
      default:
        return null;
    }
  }

  function onpointerdown(e: PointerEvent) {
    if (e.button !== 0 || !image) return;
    // Suppresses the compatibility mousedown, whose default action would move focus to the
    // page and instantly blur (and discard) a text box created by this click.
    e.preventDefault();
    if (textEdit) return commitText();
    canvas!.setPointerCapture(e.pointerId);
    const p = toImage(e);
    dragStart = p;
    movedSinceDown = false;
    if (tool === 'select') {
      const hit = shapeAt(doc.shapes, p, 6 / scale);
      selectedId = hit?.id ?? null;
      moveOrigin = hit ?? null;
      if (hit) history.record($state.snapshot(doc) as Doc);
    } else if (tool === 'text') {
      textEdit = { x: p.x, y: p.y - textSize * 0.6, value: '', id: null };
      void tick().then(() => textArea?.focus());
    } else if (tool === 'crop') {
      cropDraft = null;
    }
  }

  function onpointermove(e: PointerEvent) {
    if (!dragStart) return;
    const p = toImage(e);
    movedSinceDown = true;
    if (tool === 'select' && moveOrigin) {
      const dx = p.x - dragStart.x;
      const dy = p.y - dragStart.y;
      doc.shapes = doc.shapes.map((s) => (s.id === moveOrigin!.id ? moved(moveOrigin!, dx, dy) : s));
    } else if (tool === 'crop') {
      cropDraft = clampRect(normRect(dragStart, p), full);
    } else {
      draft = draftFor(dragStart, p);
    }
  }

  function onpointerup() {
    if (!dragStart) return;
    if (tool === 'select' && moveOrigin) {
      if (movedSinceDown) {
        dirty = true;
        historyTick++;
      } else {
        // A click without movement: drop the speculative history entry.
        history.undo($state.snapshot(doc) as Doc);
        historyTick++;
      }
    } else if (draft) {
      const s = draft;
      draft = null;
      if (isMeaningful(s, 4 / scale)) {
        nextId++;
        change((d) => d.shapes.push(s));
      }
    }
    dragStart = null;
    moveOrigin = null;
  }

  function ondblclick(e: MouseEvent) {
    const hit = shapeAt(doc.shapes, toImage(e), 6 / scale);
    if (hit?.type === 'text') {
      tool = 'select';
      selectedId = hit.id;
      textEdit = { x: hit.x, y: hit.y, value: hit.text, id: hit.id };
      void tick().then(() => textArea?.select());
    }
  }

  // ── Text ───────────────────────────────────────────────────────────────
  function commitText() {
    if (!textEdit || !canvas) return;
    const { x, y, value, id } = textEdit;
    textEdit = null;
    const existing = id != null ? doc.shapes.find((s) => s.id === id) : null;
    const size = existing?.type === 'text' ? existing.size : textSize;
    const c = existing?.type === 'text' ? existing.color : color;
    const text = value.replace(/\s+$/, '');
    if (!text) {
      if (id != null) change((d) => (d.shapes = d.shapes.filter((s) => s.id !== id)));
      return;
    }
    const m = measureText(canvas.getContext('2d')!, text, size);
    const shape: Shape = { id: id ?? nextId++, type: 'text', x, y, w: m.w, h: m.h, text, color: c, size };
    change((d) => {
      d.shapes = id != null ? d.shapes.map((s) => (s.id === id ? shape : s)) : [...d.shapes, shape];
    });
  }

  // ── Crop & resize ──────────────────────────────────────────────────────
  function setTool(next: Tool) {
    if (textEdit) commitText();
    if (tool === 'crop' && next !== 'crop') cropDraft = null;
    tool = next;
    draft = null;
    if (next !== 'select') selectedId = null;
  }

  function applyCrop() {
    const c = cropDraft;
    if (c && c.w >= 4 && c.h >= 4) {
      change((d) => {
        d.crop = { x: Math.round(c.x), y: Math.round(c.y), w: Math.round(c.w), h: Math.round(c.h) };
        d.output = null;
      });
    }
    cropDraft = null;
    tool = 'select';
  }

  function resetCrop() {
    cropDraft = { ...full };
  }

  function setOutput(value: number, edited: 'w' | 'h') {
    if (!Number.isFinite(value) || value < 1) return;
    change((d) => (d.output = resizeKeepingAspect(d.crop, Math.min(value, 16384), edited)));
  }

  // ── Style changes apply to the selection too ───────────────────────────
  function pickColor(c: string) {
    color = c;
    if (selected && 'color' in selected && selected.type !== 'highlight') {
      change((d) => (d.shapes = d.shapes.map((s) => (s.id === selected.id && 'color' in s ? { ...s, color: c } : s))));
    }
  }

  function pickLevel(l: 1 | 2 | 3) {
    level = l;
    if (!selected) return;
    const w = unit * l;
    const size = unit * (5 + l * 3);
    change((d) => {
      d.shapes = d.shapes.map((s) => {
        if (s.id !== selected.id) return s;
        if (s.type === 'arrow' || s.type === 'rect' || s.type === 'ellipse') return { ...s, width: w };
        if (s.type === 'text' && canvasCtx()) {
          const m = measureText(canvasCtx()!, s.text, size);
          return { ...s, size, w: m.w, h: m.h };
        }
        return s;
      });
    });
  }

  const canvasCtx = () => canvas?.getContext('2d') ?? null;

  function deleteSelected() {
    if (!selected) return;
    const id = selected.id;
    selectedId = null;
    change((d) => (d.shapes = d.shapes.filter((s) => s.id !== id)));
  }

  // ── Export ─────────────────────────────────────────────────────────────
  async function render(): Promise<Uint8Array | null> {
    if (!image) return null;
    if (textEdit) commitText();
    return exportPng(image, $state.snapshot(doc.shapes) as Shape[], doc.crop, out);
  }

  function fail(e: unknown) {
    logUi('error', `editor export failed: ${JSON.stringify(e)}`);
    toast('error', t((isAppError(e) ? `error.${e.code}` : 'error.generic') as MessageKey) ?? t('error.generic'));
  }

  async function save() {
    if (busy) return;
    busy = true;
    try {
      const png = await render();
      if (!png) return;
      const saved = await commands.editorSave(png);
      dirty = false;
      toast('success', t('editor.saved', { name: saved.name }), 3500);
    } catch (e) {
      fail(e);
    } finally {
      busy = false;
    }
  }

  async function copy() {
    if (busy) return;
    busy = true;
    try {
      const png = await render();
      if (!png) return;
      await commands.editorCopy(png);
      toast('success', t('editor.copied'));
    } catch (e) {
      fail(e);
    } finally {
      busy = false;
    }
  }

  // ── Keyboard ───────────────────────────────────────────────────────────
  const TOOL_KEYS: Record<string, Tool> = { v: 'select', a: 'arrow', r: 'rect', e: 'ellipse', t: 'text', h: 'highlight', b: 'blur', c: 'crop' };

  function onkeydown(e: KeyboardEvent) {
    const mod = platform === 'macos' ? e.metaKey : e.ctrlKey;
    if (textEdit || (e.target as HTMLElement).tagName === 'INPUT') {
      if (e.key === 'Escape') textEdit = null;
      return;
    }
    if (mod && e.key.toLowerCase() === 'z') {
      e.preventDefault();
      return e.shiftKey ? redo() : undo();
    }
    if (mod && e.key.toLowerCase() === 'y') return (e.preventDefault(), redo());
    if (mod && e.key.toLowerCase() === 's') return (e.preventDefault(), void save());
    if (mod && e.key.toLowerCase() === 'c') return (e.preventDefault(), void copy());
    if (mod) return;
    if (tool === 'crop' && e.key === 'Enter') return applyCrop();
    if (e.key === 'Escape') {
      if (tool === 'crop') return setTool('select');
      selectedId = null;
      return;
    }
    if ((e.key === 'Delete' || e.key === 'Backspace') && selected) return (e.preventDefault(), deleteSelected());
    const next = TOOL_KEYS[e.key.toLowerCase()];
    if (next) setTool(next);
  }

  // ── Toolbar model ──────────────────────────────────────────────────────
  type Icon = Component<{ size?: number; strokeWidth?: number }>;
  const tools: { id: Tool; icon: Icon; key: string }[] = [
    { id: 'select', icon: MousePointer2, key: 'V' },
    { id: 'arrow', icon: MoveUpRight, key: 'A' },
    { id: 'rect', icon: Square, key: 'R' },
    { id: 'ellipse', icon: Circle, key: 'E' },
    { id: 'text', icon: Type, key: 'T' },
    { id: 'highlight', icon: Highlighter, key: 'H' },
    { id: 'blur', icon: Grid3x3, key: 'B' },
    { id: 'crop', icon: Crop, key: 'C' },
  ];
  const showStyle = $derived(
    ['arrow', 'rect', 'ellipse', 'text'].includes(tool) || (selected != null && ['arrow', 'rect', 'ellipse', 'text'].includes(selected.type)),
  );
  const textStyle = $derived(textEdit ? `left:${(textEdit.x - region.x) * scale}px;top:${(textEdit.y - region.y) * scale}px;font:${font(textSize * scale)};color:${color}` : '');
</script>

<svelte:window {onkeydown} />

<div class="editor" class:mac={platform === 'macos'}>
  <header class="bar" data-tauri-drag-region>
    <div class="title" data-tauri-drag-region>{name}</div>
    <div class="tools" role="toolbar">
      {#each tools as item (item.id)}
        <button class="tool" class:active={tool === item.id} onclick={() => setTool(item.id)} title={`${t(`editor.tool.${item.id}`)} (${item.key})`}>
          <item.icon size={17} strokeWidth={1.9} />
        </button>
      {/each}
    </div>
    <div class="actions">
      <button class="tool" disabled={!canUndo} onclick={undo} title={t('editor.undo')}><Undo2 size={17} /></button>
      <button class="tool" disabled={!canRedo} onclick={redo} title={t('editor.redo')}><Redo2 size={17} /></button>
      <span class="sep"></span>
      <div class="size-wrap">
        <button class="size" onclick={() => (resizeOpen = !resizeOpen)} title={t('editor.size')}>
          <Scaling size={14} />{out.w} × {out.h}
        </button>
        {#if resizeOpen}
          <div class="popover">
            <label>{t('editor.width')}<input type="number" min="1" value={out.w} onchange={(e) => setOutput(Number(e.currentTarget.value), 'w')} /></label>
            <label>{t('editor.height')}<input type="number" min="1" value={out.h} onchange={(e) => setOutput(Number(e.currentTarget.value), 'h')} /></label>
            <Button size="sm" variant="ghost" onclick={() => change((d) => (d.output = null))}>{t('editor.resetSize')}</Button>
          </div>
        {/if}
      </div>
      <Button size="sm" onclick={copy} disabled={busy}><Copy size={14} />{t('editor.copy')}</Button>
      <Button size="sm" variant="primary" onclick={save} busy={busy}>{t('editor.save')}</Button>
    </div>
  </header>

  <div class="sub">
    {#if tool === 'crop'}
      <span class="hint">{t('editor.crop.hint')}</span>
      <Button size="sm" variant="ghost" onclick={resetCrop}>{t('editor.crop.reset')}</Button>
      <Button size="sm" variant="ghost" onclick={() => setTool('select')}>{t('editor.cancel')}</Button>
      <Button size="sm" variant="primary" onclick={applyCrop}>{t('editor.crop.apply')}</Button>
    {:else if showStyle}
      <div class="swatches">
        {#each COLORS as c (c)}
          <button class="swatch" class:on={color === c} style="--c:{c}" onclick={() => pickColor(c)} aria-label={c}></button>
        {/each}
      </div>
      <span class="sep"></span>
      <div class="levels" title={t('editor.stroke')}>
        {#each [1, 2, 3] as const as l (l)}
          <button class="level" class:on={level === l} onclick={() => pickLevel(l)} aria-label={`${t('editor.stroke')} ${l}`}>
            <span style="height:{l * 2 + 1}px"></span>
          </button>
        {/each}
      </div>
    {:else}
      <span class="hint">{t(`editor.tool.${tool}`)}</span>
    {/if}
  </div>

  <div class="stage" bind:this={stage}>
    {#if image}
      <div class="canvas-wrap" style="width:{region.w * scale}px;height:{region.h * scale}px">
        <canvas
          bind:this={canvas}
          class="tool-{tool}"
          style="width:{region.w * scale}px;height:{region.h * scale}px"
          {onpointerdown}
          {onpointermove}
          {onpointerup}
          {ondblclick}
        ></canvas>
        {#if textEdit}
          <textarea
            bind:this={textArea}
            bind:value={textEdit.value}
            class="text-input"
            style={textStyle}
            placeholder={t('editor.text.placeholder')}
            rows="1"
            spellcheck="false"
            onblur={commitText}
            onkeydown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                commitText();
              } else if (e.key === 'Escape') {
                textEdit = null;
              }
            }}
          ></textarea>
        {/if}
      </div>
    {/if}
  </div>
</div>

<Toaster />

<style>
  :global(html),
  :global(body) {
    background: var(--bg) !important;
  }
  .editor {
    display: grid;
    grid-template-rows: auto auto 1fr;
    height: 100%;
  }
  .bar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    gap: 12px;
    height: 52px;
    padding: 0 12px 0 16px;
    background: var(--bg-sidebar);
    border-bottom: 1px solid var(--border);
  }
  .mac .bar {
    padding-left: 92px; /* traffic lights */
    background: transparent;
  }
  .title {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
    color: var(--text-2);
  }
  .tools,
  .actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .tools {
    padding: 3px;
    border-radius: 10px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
  }
  .actions {
    justify-content: flex-end;
    gap: 6px;
  }
  .tool {
    display: grid;
    place-items: center;
    width: 32px;
    height: 30px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--text-2);
    transition:
      background var(--dur-fast),
      color var(--dur-fast);
  }
  .tool:hover:not(:disabled) {
    background: var(--surface-hover);
    color: var(--text);
  }
  .tool.active {
    background: var(--accent);
    color: #fff;
  }
  .tool:disabled {
    opacity: 0.35;
  }
  .sep {
    width: 1px;
    height: 18px;
    margin: 0 4px;
    background: var(--border-strong);
  }
  .size-wrap {
    position: relative;
  }
  .size {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 26px;
    padding: 0 8px;
    border: 0;
    border-radius: 6px;
    background: transparent;
    color: var(--text-2);
    font: 600 var(--text-sm) var(--font-mono);
  }
  .size:hover {
    background: var(--surface-hover);
  }
  .popover {
    position: absolute;
    right: 0;
    top: 34px;
    z-index: 10;
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 190px;
    padding: 12px;
    border-radius: 12px;
    background: var(--surface);
    box-shadow:
      0 0 0 1px var(--border-strong),
      var(--shadow-md);
  }
  .popover label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: var(--text-sm);
    color: var(--text-2);
  }
  .popover input {
    width: 86px;
    height: 28px;
    padding: 0 8px;
    border: 0;
    border-radius: 6px;
    background: var(--surface-2);
    box-shadow: inset 0 0 0 1px var(--border);
    font-family: var(--font-mono);
    user-select: text;
  }
  .sub {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    height: 42px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .hint {
    color: var(--text-3);
    font-size: var(--text-sm);
  }
  .swatches,
  .levels {
    display: flex;
    gap: 6px;
  }
  .swatch {
    width: 20px;
    height: 20px;
    border: 0;
    border-radius: 50%;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgba(10, 21, 52, 0.25);
    transition: transform var(--dur-fast) var(--ease-spring);
  }
  .swatch.on {
    box-shadow:
      0 0 0 2px var(--surface),
      0 0 0 4px var(--accent);
    transform: scale(1.08);
  }
  .level {
    display: grid;
    place-items: center;
    width: 30px;
    height: 24px;
    border: 0;
    border-radius: 6px;
    background: transparent;
  }
  .level span {
    width: 16px;
    border-radius: 2px;
    background: var(--text-2);
  }
  .level.on {
    background: var(--accent-soft);
  }
  .level.on span {
    background: var(--accent);
  }
  .stage {
    position: relative;
    display: grid;
    place-items: center;
    min-height: 0;
    overflow: hidden;
    background:
      radial-gradient(circle at 50% 40%, color-mix(in srgb, var(--accent) 6%, transparent), transparent 70%),
      var(--bg);
  }
  .canvas-wrap {
    position: relative;
    box-shadow:
      0 0 0 1px var(--border),
      0 20px 50px -20px rgba(6, 13, 38, 0.55);
  }
  canvas {
    display: block;
    touch-action: none;
  }
  .tool-select {
    cursor: default;
  }
  .tool-text {
    cursor: text;
  }
  canvas:not(.tool-select):not(.tool-text) {
    cursor: crosshair;
  }
  .text-input {
    position: absolute;
    min-width: 80px;
    padding: 0 0.25em;
    border: 1px dashed var(--accent);
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.04);
    resize: none;
    overflow: hidden;
    field-sizing: content;
    line-height: 1.25;
    user-select: text;
  }
  .text-input:focus {
    outline: none;
  }
</style>
