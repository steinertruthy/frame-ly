<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { emit, listen } from '@tauri-apps/api/event';
  import { MoveUpRight, Type, Square, Circle, Grid3x3, Highlighter, Crop, Undo2, Redo2, X, Copy, Download } from '@lucide/svelte';
  import type { Component } from 'svelte';
  import { commands } from '$lib/ipc/commands';
  import { framelyUrl } from '$lib/ipc/protocol';
  import { logUi } from '$lib/ipc/log';
  import { applyAppearance } from '$lib/stores/settings.svelte';
  import { setLanguage, t, type MessageKey } from '$lib/i18n/index.svelte';
  import type { OverlayInit, Rect as CssRect, SelectMode } from '$lib/ipc/types';
  import { clampRect as clampCss, hitTest, rectFromPoints, windowToLocal } from './geometry';
  import { COLORS, HIGHLIGHT, History, baseStroke, isMeaningful, normRect, type Doc, type Point, type Rect, type Shape } from '../editor/model';
  import { drawDoc, exportPng, font, measureText } from '../editor/render';

  let { displayId }: { displayId: string } = $props();

  const MIN_SIZE = 4;
  const LOUPE = 128;
  const LOUPE_PIXELS = 15;

  type Phase = 'select' | 'edit' | 'busy';
  type EditTool = 'arrow' | 'text' | 'rect' | 'ellipse' | 'blur' | 'highlight' | 'crop';

  let init = $state<OverlayInit | null>(null);
  let mode = $state<SelectMode>('region');
  let phase = $state<Phase>('select');
  let locked = $state(false); // another display is being edited
  let vw = $state(window.innerWidth);
  let vh = $state(window.innerHeight);
  let pointer = $state<{ x: number; y: number } | null>(null);
  let dragStart = $state<{ x: number; y: number } | null>(null);
  let showHints = $state(true);
  let img: HTMLImageElement;
  let loupe: HTMLCanvasElement | undefined = $state();

  // Edit phase (all geometry in frame pixels; `k` = frame px per CSS px).
  let doc = $state<Doc>({ shapes: [], crop: { x: 0, y: 0, w: 1, h: 1 }, output: null });
  const history = new History();
  let historyTick = $state(0);
  let tool = $state<EditTool>('arrow');
  let color = $state<string>(COLORS[0]);
  let level = $state<1 | 2 | 3>(2);
  let paletteOpen = $state(false);
  let draft = $state<Shape | null>(null);
  let textEdit = $state<{ x: number; y: number; value: string } | null>(null);
  let canvas: HTMLCanvasElement | undefined = $state();
  let textArea: HTMLTextAreaElement | undefined = $state();
  let toolbar: HTMLDivElement | undefined = $state();
  let nextId = 1;
  const dpr = window.devicePixelRatio || 1;

  const isRecord = $derived(init?.purpose === 'record');
  const k = $derived(init ? init.frameWidth / vw : 1);
  const frameSize = $derived(init ? { w: init.frameWidth, h: init.frameHeight } : { w: 1, h: 1 });
  const unit = $derived(baseStroke(doc.crop.w, doc.crop.h));
  const strokeWidth = $derived(unit * level);
  const textSize = $derived(unit * (5 + level * 3));
  const canUndo = $derived.by(() => (void historyTick, history.canUndo));
  const canRedo = $derived.by(() => (void historyTick, history.canRedo));

  const scale = $derived(init ? init.frameWidth / vw : 1);
  const selection = $derived(dragStart && pointer ? clampCss(rectFromPoints(dragStart, pointer), vw, vh) : null);
  const windows = $derived(init ? init.windows.map((w) => ({ info: w, rect: clampCss(windowToLocal(w, init!.display, vw), vw, vh) })) : []);
  const hovered = $derived(phase === 'select' && mode === 'window' && pointer ? hitTest(windows, pointer.x, pointer.y) : undefined);
  /** Selection in CSS pixels: while dragging, the drag; while editing, the crop. */
  const sel = $derived<CssRect | null>(
    phase === 'select' ? (mode === 'region' ? selection : (hovered?.rect ?? null)) : { x: doc.crop.x / k, y: doc.crop.y / k, width: doc.crop.w / k, height: doc.crop.h / k },
  );
  const px = (v: number) => Math.round(v * scale);

  onMount(() => {
    void (async () => {
      try {
        const [data, settings] = await Promise.all([commands.overlayInit(displayId), commands.getSettings()]);
        setLanguage(settings.general.language);
        applyAppearance(settings.appearance);
        init = data;
        mode = data.mode;
        img.crossOrigin = 'anonymous'; // served with CORS: the annotated export stays possible
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
    const un = listen<string>('overlay://editing', (e) => {
      if (e.payload !== displayId) locked = true;
    });
    const onResize = () => {
      vw = window.innerWidth;
      vh = window.innerHeight;
    };
    window.addEventListener('resize', onResize);
    return () => {
      window.removeEventListener('resize', onResize);
      void un.then((u) => u());
    };
  });

  // ── Loupe (selection phase) ────────────────────────────────────────────
  $effect(() => {
    if (!loupe || !pointer || !init || mode !== 'region' || phase !== 'select') return;
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

  // ── Entering the edit phase ────────────────────────────────────────────
  function startEditing(css: CssRect) {
    const crop: Rect = {
      x: Math.round(css.x * k),
      y: Math.round(css.y * k),
      w: Math.max(1, Math.round(css.width * k)),
      h: Math.max(1, Math.round(css.height * k)),
    };
    doc = { shapes: [], crop, output: null };
    phase = 'edit';
    tool = 'arrow';
    void emit('overlay://editing', displayId);
  }

  async function finishRecordRegion(r: CssRect) {
    phase = 'busy';
    try {
      await commands.finishRegion(displayId, r, { width: vw, height: vh });
    } catch (e) {
      logUi('error', `finish_region failed: ${JSON.stringify(e)}`);
      void commands.cancelCapture();
    }
  }

  async function finishRecordWindow(id: string) {
    phase = 'busy';
    try {
      await commands.finishWindow(id);
    } catch (e) {
      logUi('error', `finish_window failed: ${JSON.stringify(e)}`);
      void commands.cancelCapture();
    }
  }

  // ── Selection-phase pointer handling ───────────────────────────────────
  function onpointerdown(e: PointerEvent) {
    if (e.button !== 0 || locked || phase !== 'select') return;
    if (mode === 'window') {
      if (!hovered) return;
      if (isRecord) void finishRecordWindow(hovered.info.id);
      else startEditing(hovered.rect);
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
    if (phase !== 'select' || mode !== 'region' || !selection) return;
    const r = selection;
    dragStart = null;
    if (r.width < MIN_SIZE || r.height < MIN_SIZE) return;
    if (isRecord) void finishRecordRegion(r);
    else startEditing(r);
  }

  // ── Edit phase: document changes ───────────────────────────────────────
  function change(mutate: (d: Doc) => void) {
    history.record($state.snapshot(doc) as Doc);
    const next = $state.snapshot(doc) as Doc;
    mutate(next);
    doc = next;
    historyTick++;
  }
  function undo() {
    const prev = history.undo($state.snapshot(doc) as Doc);
    if (prev) {
      doc = prev;
      historyTick++;
    }
  }
  function redo() {
    const next = history.redo($state.snapshot(doc) as Doc);
    if (next) {
      doc = next;
      historyTick++;
    }
  }

  // ── Edit phase: rendering annotations over the selection ───────────────
  $effect(() => {
    if (phase === 'select' || !canvas || !init) return;
    const c = doc.crop;
    const w = Math.round((c.w / k) * dpr);
    const h = Math.round((c.h / k) * dpr);
    if (canvas.width !== w) canvas.width = w;
    if (canvas.height !== h) canvas.height = h;
    const ctx = canvas.getContext('2d')!;
    ctx.clearRect(0, 0, w, h);
    drawDoc(ctx, img, frameSize, draft ? [...doc.shapes, draft] : doc.shapes, { region: c, scale: dpr / k });
  });

  // ── Edit phase: drawing ────────────────────────────────────────────────
  let editStart: Point | null = null;
  let cropDrag: { handle: string; origin: Rect; start: Point } | null = null;

  function toFrame(e: PointerEvent | MouseEvent): Point {
    return { x: e.clientX * k, y: e.clientY * k };
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
      case 'blur': {
        const c = doc.crop;
        const x0 = Math.max(c.x, r.x);
        const y0 = Math.max(c.y, r.y);
        return { id, type: 'blur', x: x0, y: y0, w: Math.max(0, Math.min(c.x + c.w, r.x + r.w) - x0), h: Math.max(0, Math.min(c.y + c.h, r.y + r.h) - y0) };
      }
      default:
        return null;
    }
  }

  function oncanvasdown(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault(); // keeps focus on a text box created by this click
    e.stopPropagation();
    paletteOpen = false;
    if (textEdit) return commitText();
    const p = toFrame(e);
    if (tool === 'text') {
      textEdit = { x: p.x, y: p.y - textSize * 0.6, value: '' };
      void tick().then(() => textArea?.focus());
      return;
    }
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    if (tool === 'crop') {
      cropDrag = { handle: 'move', origin: { ...doc.crop }, start: p };
      return;
    }
    editStart = p;
  }

  function oncanvasmove(e: PointerEvent) {
    const p = toFrame(e);
    if (cropDrag) return dragCrop(p);
    if (editStart) draft = draftFor(editStart, p);
  }

  function oncanvasup() {
    if (cropDrag) return endCropDrag();
    if (draft) {
      const s = draft;
      draft = null;
      if (isMeaningful(s, 4 * k)) {
        nextId++;
        change((d) => d.shapes.push(s));
      }
    }
    editStart = null;
  }

  // ── Crop: handles resize, dragging inside moves ────────────────────────
  function onhandledown(e: PointerEvent, handle: string) {
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    cropDrag = { handle, origin: { ...doc.crop }, start: toFrame(e) };
  }

  function dragCrop(p: Point) {
    if (!cropDrag) return;
    const o = cropDrag.origin;
    const dx = p.x - cropDrag.start.x;
    const dy = p.y - cropDrag.start.y;
    const min = 8 * k;
    let { x, y, w, h } = o;
    const hd = cropDrag.handle;
    if (hd === 'move') {
      x = Math.min(Math.max(0, o.x + dx), frameSize.w - o.w);
      y = Math.min(Math.max(0, o.y + dy), frameSize.h - o.h);
    } else {
      if (hd.includes('w')) {
        x = Math.min(Math.max(0, o.x + dx), o.x + o.w - min);
        w = o.x + o.w - x;
      }
      if (hd.includes('e')) w = Math.max(min, Math.min(frameSize.w - o.x, o.w + dx));
      if (hd.includes('n')) {
        y = Math.min(Math.max(0, o.y + dy), o.y + o.h - min);
        h = o.y + o.h - y;
      }
      if (hd.includes('s')) h = Math.max(min, Math.min(frameSize.h - o.y, o.h + dy));
    }
    doc.crop = { x: Math.round(x), y: Math.round(y), w: Math.round(w), h: Math.round(h) };
  }

  function endCropDrag() {
    if (!cropDrag) return;
    const before = { ...$state.snapshot(doc), crop: cropDrag.origin } as Doc;
    const changed = JSON.stringify(before.crop) !== JSON.stringify(doc.crop);
    if (changed) {
      history.record(before);
      historyTick++;
    }
    cropDrag = null;
  }

  // ── Text ───────────────────────────────────────────────────────────────
  function commitText() {
    if (!textEdit || !canvas) return;
    const { x, y, value } = textEdit;
    textEdit = null;
    const text = value.replace(/\s+$/, '');
    if (!text) return;
    const m = measureText(canvas.getContext('2d')!, text, textSize);
    const shape: Shape = { id: nextId++, type: 'text', x, y, w: m.w, h: m.h, text, color, size: textSize };
    change((d) => d.shapes.push(shape));
  }

  // ── Finish ─────────────────────────────────────────────────────────────
  async function finish(intent: 'copy' | 'save') {
    if (phase !== 'edit') return;
    if (textEdit) commitText();
    phase = 'busy';
    try {
      const png = await exportPng(img, $state.snapshot(doc.shapes) as Shape[], doc.crop, { w: doc.crop.w, h: doc.crop.h });
      await commands.finishAnnotated(png, intent);
    } catch (e) {
      logUi('error', `annotated capture failed: ${JSON.stringify(e)}`);
      phase = 'edit';
    }
  }

  const cancel = () => void commands.cancelCapture();

  // ── Keyboard ───────────────────────────────────────────────────────────
  const TOOL_KEYS: Record<string, EditTool> = { a: 'arrow', t: 'text', r: 'rect', e: 'ellipse', b: 'blur', h: 'highlight', c: 'crop' };

  function onkeydown(e: KeyboardEvent) {
    if (phase === 'busy') return;
    if (textEdit) return; // the text box handles its own keys
    const mod = e.metaKey || e.ctrlKey;
    if (e.key === 'Escape') {
      e.preventDefault();
      if (paletteOpen) paletteOpen = false;
      else if (dragStart) dragStart = null;
      else cancel();
      return;
    }
    if (phase === 'select') {
      if (e.key === ' ' && !locked) {
        e.preventDefault();
        dragStart = null;
        mode = mode === 'region' ? 'window' : 'region';
      }
      return;
    }
    if (mod && e.key.toLowerCase() === 'z') return (e.preventDefault(), e.shiftKey ? redo() : undo());
    if (mod && e.key.toLowerCase() === 'c') return (e.preventDefault(), void finish('copy'));
    if (mod && e.key.toLowerCase() === 's') return (e.preventDefault(), void finish('save'));
    if (e.key === 'Enter') return (e.preventDefault(), void finish('copy'));
    if (mod) return;
    const next = TOOL_KEYS[e.key.toLowerCase()];
    if (next) tool = next;
  }

  // ── Layout helpers ─────────────────────────────────────────────────────
  const loupePos = $derived.by(() => {
    if (!pointer) return { x: 0, y: 0 };
    const gap = 22;
    const x = pointer.x + gap + LOUPE > vw ? pointer.x - gap - LOUPE : pointer.x + gap;
    const y = pointer.y + gap + LOUPE + 28 > vh ? pointer.y - gap - LOUPE - 28 : pointer.y + gap;
    return { x, y };
  });

  /** Toolbar below the selection, else above, else inside its bottom edge; kept on screen. */
  let toolbarW = $state(560);
  $effect(() => {
    if (toolbar) toolbarW = toolbar.offsetWidth;
  });
  const toolbarPos = $derived.by(() => {
    if (!sel) return { x: 0, y: 0 };
    const H = 46;
    const gap = 12;
    let y = sel.y + sel.height + gap;
    if (y + H > vh - 8) y = sel.y - H - gap;
    if (y < 8) y = Math.min(vh - H - 8, sel.y + sel.height - H - gap);
    const x = Math.min(Math.max(8, sel.x + sel.width / 2 - toolbarW / 2), vw - toolbarW - 8);
    return { x, y };
  });

  type Icon = Component<{ size?: number; strokeWidth?: number }>;
  const tools: { id: EditTool; icon: Icon; key: string; label: MessageKey }[] = [
    { id: 'arrow', icon: MoveUpRight, key: 'A', label: 'editor.tool.arrow' },
    { id: 'text', icon: Type, key: 'T', label: 'editor.tool.text' },
    { id: 'rect', icon: Square, key: 'R', label: 'editor.tool.rect' },
    { id: 'ellipse', icon: Circle, key: 'E', label: 'editor.tool.ellipse' },
    { id: 'blur', icon: Grid3x3, key: 'B', label: 'editor.tool.blur' },
    { id: 'highlight', icon: Highlighter, key: 'H', label: 'editor.tool.highlight' },
    { id: 'crop', icon: Crop, key: 'C', label: 'editor.tool.crop' },
  ];
  const HANDLES = ['nw', 'n', 'ne', 'e', 'se', 's', 'sw', 'w'];
  const textStyle = $derived(textEdit ? `left:${textEdit.x / k}px;top:${textEdit.y / k}px;font:${font(textSize / k)};color:${color}` : '');
  const mac = navigator.userAgent.includes('Mac');
</script>

<svelte:window {onkeydown} oncontextmenu={(e) => e.preventDefault()} />

<div
  class="root"
  class:window-mode={mode === 'window' && phase === 'select'}
  class:editing={phase !== 'select'}
  class:record={isRecord}
  role="application"
  {onpointerdown}
  {onpointermove}
  {onpointerup}
  onpointerleave={() => !dragStart && (pointer = null)}
>
  <img bind:this={img} alt="" class="frame" draggable="false" />

  {#if sel && sel.width > 0 && sel.height > 0}
    <div class="focus" class:window={mode === 'window' && phase === 'select'} style="left:{sel.x}px;top:{sel.y}px;width:{sel.width}px;height:{sel.height}px">
      {#if phase === 'select'}
        <div class="size" class:inside={sel.y < 34}>
          {#if mode === 'window' && hovered}<strong>{hovered.info.appName}</strong>{/if}
          {px(sel.width)} × {px(sel.height)}
        </div>
      {:else}
        <canvas
          bind:this={canvas}
          class="ink tool-{tool}"
          style="width:{sel.width}px;height:{sel.height}px"
          onpointerdown={oncanvasdown}
          onpointermove={oncanvasmove}
          onpointerup={oncanvasup}
        ></canvas>
        {#if tool === 'crop'}
          {#each HANDLES as hd (hd)}
            <span class="handle h-{hd}" role="presentation" onpointerdown={(e) => onhandledown(e, hd)} onpointermove={(e) => cropDrag && dragCrop(toFrame(e))} onpointerup={endCropDrag}></span>
          {/each}
          <div class="size live">{doc.crop.w} × {doc.crop.h}</div>
        {/if}
      {/if}
    </div>
  {:else}
    <div class="dim"></div>
  {/if}

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
          e.stopPropagation();
          textEdit = null;
        }
      }}
    ></textarea>
  {/if}

  {#if phase === 'select' && mode === 'region' && pointer && !locked}
    {#if !dragStart}
      <div class="cross h" style="top:{pointer.y}px"></div>
      <div class="cross v" style="left:{pointer.x}px"></div>
    {/if}
    <div class="loupe" style="left:{loupePos.x}px;top:{loupePos.y}px">
      <canvas bind:this={loupe} style="width:{LOUPE}px;height:{LOUPE}px"></canvas>
      <div class="coords">{px(pointer.x)}, {px(pointer.y)}</div>
    </div>
  {/if}

  {#if phase !== 'select' && sel}
    <div
      class="toolbar"
      bind:this={toolbar}
      style="left:{toolbarPos.x}px;top:{toolbarPos.y}px"
      role="toolbar"
      tabindex="-1"
      onpointerdown={(e) => e.stopPropagation()}
    >
      <div class="group">
        {#each tools as item (item.id)}
          <button class="tb" class:active={tool === item.id} onclick={() => (tool = item.id)} title="{t(item.label)} ({item.key})" aria-label={t(item.label)}>
            <item.icon size={17} strokeWidth={1.9} />
          </button>
        {/each}
        <div class="palette-wrap">
          <button class="tb swatch-btn" onclick={() => (paletteOpen = !paletteOpen)} title={t('editor.stroke')} aria-label={t('editor.stroke')}>
            <span class="dot" style="--c:{color}"></span>
          </button>
          {#if paletteOpen}
            <div class="palette">
              <div class="colors">
                {#each COLORS as c (c)}
                  <button class="swatch" class:on={color === c} style="--c:{c}" onclick={() => (color = c)} aria-label={c}></button>
                {/each}
              </div>
              <div class="levels">
                {#each [1, 2, 3] as const as l (l)}
                  <button class="level" class:on={level === l} onclick={() => (level = l)} aria-label="{t('editor.stroke')} {l}">
                    <span style="height:{l * 2 + 1}px"></span>
                  </button>
                {/each}
              </div>
            </div>
          {/if}
        </div>
      </div>
      <span class="sep"></span>
      <div class="group">
        <button class="tb" disabled={!canUndo} onclick={undo} title="{t('editor.undo')} ({mac ? '⌘' : 'Ctrl+'}Z)" aria-label={t('editor.undo')}><Undo2 size={17} /></button>
        <button class="tb" disabled={!canRedo} onclick={redo} title={t('editor.redo')} aria-label={t('editor.redo')}><Redo2 size={17} /></button>
      </div>
      <span class="sep"></span>
      <div class="group actions">
        <button class="tb" onclick={cancel} title="{t('editor.cancel')} (Esc)" aria-label={t('editor.cancel')}><X size={17} /></button>
        <button class="act" disabled={phase === 'busy'} onclick={() => finish('copy')} title="{t('editor.copy')} (↩)">
          <Copy size={15} /><span>{t('editor.copy')}</span>
        </button>
        <button class="act primary" disabled={phase === 'busy'} onclick={() => finish('save')} title="{t('editor.save')} ({mac ? '⌘' : 'Ctrl+'}S)">
          <Download size={15} /><span>{t('editor.save')}</span>
        </button>
      </div>
    </div>
  {/if}

  {#if init && phase === 'select' && !locked}
    <div class="hints" class:hidden={!showHints}>
      {#if isRecord}<span class="rec-dot"></span>{/if}
      <span>
        {t(
          isRecord
            ? mode === 'region'
              ? 'overlay.hint.record.region'
              : 'overlay.hint.record.window'
            : mode === 'region'
              ? 'overlay.hint.region'
              : 'overlay.hint.window',
        )}
      </span>
      <span class="sep-dot"></span>
      <span>{t(mode === 'region' ? 'overlay.hint.toggle.region' : 'overlay.hint.toggle.window')}</span>
      <span class="sep-dot"></span>
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
  .root.window-mode,
  .root.editing {
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
    background: rgba(7, 9, 13, 0.42);
  }
  .focus {
    position: absolute;
    outline: 1.5px solid #8bb0ee;
    box-shadow:
      0 0 0 100vmax rgba(7, 9, 13, 0.5),
      0 0 18px 0 rgba(76, 123, 217, 0.25);
  }
  .editing .focus {
    outline-color: rgba(255, 255, 255, 0.9);
    animation: settle var(--dur-slow) var(--ease-out);
  }
  @keyframes settle {
    from {
      box-shadow:
        0 0 0 100vmax rgba(7, 9, 13, 0.42),
        0 0 0 0 rgba(76, 123, 217, 0.6);
    }
  }
  .focus.window {
    background: rgba(76, 123, 217, 0.12);
    outline-width: 3px;
    outline-offset: -3px;
    border-radius: 10px;
    transition:
      left 90ms var(--ease-out),
      top 90ms var(--ease-out),
      width 90ms var(--ease-out),
      height 90ms var(--ease-out);
  }
  .record .focus {
    outline-color: var(--rec-500);
    box-shadow:
      0 0 0 100vmax rgba(7, 9, 13, 0.5),
      0 0 18px 0 rgba(217, 70, 86, 0.22);
  }
  .record .focus.window {
    background: rgba(217, 70, 86, 0.1);
  }
  .ink {
    position: absolute;
    inset: 0;
    display: block;
    touch-action: none;
  }
  .ink:not(.tool-text):not(.tool-crop) {
    cursor: crosshair;
  }
  .tool-text {
    cursor: text;
  }
  .tool-crop {
    cursor: move;
  }
  .handle {
    position: absolute;
    width: 12px;
    height: 12px;
    margin: -6px 0 0 -6px;
    border-radius: 3px;
    background: #fff;
    box-shadow:
      0 0 0 1.5px #4c7bd9,
      0 2px 6px rgba(0, 0, 0, 0.4);
    z-index: 2;
  }
  .h-nw {
    left: 0;
    top: 0;
    cursor: nwse-resize;
  }
  .h-n {
    left: 50%;
    top: 0;
    cursor: ns-resize;
  }
  .h-ne {
    left: 100%;
    top: 0;
    cursor: nesw-resize;
  }
  .h-e {
    left: 100%;
    top: 50%;
    cursor: ew-resize;
  }
  .h-se {
    left: 100%;
    top: 100%;
    cursor: nwse-resize;
  }
  .h-s {
    left: 50%;
    top: 100%;
    cursor: ns-resize;
  }
  .h-sw {
    left: 0;
    top: 100%;
    cursor: nesw-resize;
  }
  .h-w {
    left: 0;
    top: 50%;
    cursor: ew-resize;
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
    background: rgba(11, 14, 20, 0.88);
    color: #fff;
    font: 600 12px/18px var(--font-mono);
    white-space: nowrap;
    backdrop-filter: blur(10px);
    box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.09);
  }
  .size strong {
    font-family: var(--font-ui);
  }
  .size.inside,
  .size.live {
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
      0 0 0 6px rgba(76, 123, 217, 0.25),
      0 10px 30px rgba(0, 0, 0, 0.5);
  }
  .coords {
    padding: 2px 8px;
    border-radius: 6px;
    background: rgba(11, 14, 20, 0.88);
    color: #fff;
    font: 600 11px/16px var(--font-mono);
  }
  .text-input {
    position: absolute;
    z-index: 3;
    min-width: 80px;
    padding: 0 0.25em;
    border: 1px dashed #8bb0ee;
    border-radius: 4px;
    background: rgba(11, 14, 20, 0.25);
    resize: none;
    overflow: hidden;
    field-sizing: content;
    line-height: 1.25;
    user-select: text;
  }
  .text-input:focus {
    outline: none;
  }

  /* Contextual toolbar */
  .toolbar {
    position: absolute;
    z-index: 5;
    display: flex;
    align-items: center;
    gap: 4px;
    height: 46px;
    padding: 0 6px;
    border-radius: 13px;
    background: rgba(17, 21, 31, 0.92);
    box-shadow:
      inset 0 1px 0 rgba(255, 255, 255, 0.05),
      0 0 0 1px rgba(255, 255, 255, 0.08),
      0 18px 50px -14px rgba(0, 0, 0, 0.8);
    backdrop-filter: blur(18px) saturate(1.3);
    color: #edf0f7;
    animation: rise var(--dur-slow) var(--ease-spring);
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(8px) scale(0.97);
    }
  }
  .group {
    display: flex;
    align-items: center;
    gap: 2px;
  }
  .tb {
    position: relative;
    display: grid;
    place-items: center;
    width: 34px;
    height: 34px;
    border: 0;
    border-radius: 9px;
    background: transparent;
    color: #9aa3b5;
    transition:
      background var(--dur-fast),
      color var(--dur-fast),
      transform var(--dur-fast) var(--ease-out);
  }
  .tb:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.07);
    color: #fff;
  }
  .tb:active:not(:disabled) {
    transform: scale(0.92);
  }
  .tb.active {
    background: #1b273d;
    color: #8bb0ee;
    box-shadow: inset 0 0 0 1px #325188;
  }
  .tb:disabled {
    opacity: 0.3;
  }
  .sep {
    width: 1px;
    height: 22px;
    margin: 0 4px;
    background: rgba(255, 255, 255, 0.1);
  }
  .dot {
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--c);
    box-shadow:
      0 0 0 2px rgba(17, 21, 31, 0.9),
      0 0 0 3.5px rgba(255, 255, 255, 0.5);
  }
  .palette-wrap {
    position: relative;
  }
  .palette {
    position: absolute;
    left: 50%;
    bottom: 46px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 10px;
    border-radius: 12px;
    background: rgba(17, 21, 31, 0.96);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.08),
      0 14px 40px -12px rgba(0, 0, 0, 0.8);
    transform: translateX(-50%);
    animation: rise var(--dur-med) var(--ease-out);
  }
  .colors,
  .levels {
    display: flex;
    gap: 7px;
  }
  .swatch {
    width: 20px;
    height: 20px;
    border: 0;
    border-radius: 50%;
    background: var(--c);
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.3);
    transition: transform var(--dur-fast) var(--ease-spring);
  }
  .swatch.on {
    box-shadow:
      0 0 0 2px rgba(17, 21, 31, 1),
      0 0 0 4px #4c7bd9;
    transform: scale(1.08);
  }
  .level {
    display: grid;
    place-items: center;
    flex: 1;
    height: 24px;
    border: 0;
    border-radius: 6px;
    background: transparent;
  }
  .level span {
    width: 18px;
    border-radius: 2px;
    background: #9aa3b5;
  }
  .level.on {
    background: rgba(76, 123, 217, 0.2);
  }
  .level.on span {
    background: #8bb0ee;
  }
  .act {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 12px;
    border: 0;
    border-radius: 9px;
    background: rgba(255, 255, 255, 0.07);
    color: #edf0f7;
    font-size: 12.5px;
    font-weight: 650;
    white-space: nowrap;
    transition:
      background var(--dur-fast),
      transform var(--dur-fast);
  }
  .act:hover:not(:disabled) {
    background: rgba(255, 255, 255, 0.11);
  }
  .act:active:not(:disabled) {
    transform: scale(0.96);
  }
  .act.primary {
    background: #1f3459;
    box-shadow: inset 0 0 0 1px #325188;
  }
  .act.primary:hover:not(:disabled) {
    background: #27416f;
  }
  .act:disabled {
    opacity: 0.6;
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
    background: rgba(11, 14, 20, 0.88);
    color: rgba(237, 240, 247, 0.92);
    font-size: 12px;
    font-weight: 550;
    pointer-events: none;
    backdrop-filter: blur(10px);
    box-shadow:
      0 0 0 1px rgba(255, 255, 255, 0.08),
      0 10px 30px rgba(0, 0, 0, 0.4);
    transition:
      opacity var(--dur-slow) var(--ease-out),
      transform var(--dur-slow) var(--ease-out);
  }
  .hints.hidden {
    opacity: 0;
    transform: translate(-50%, -8px);
  }
  .sep-dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.4);
  }
  .rec-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--rec-500);
  }
</style>
