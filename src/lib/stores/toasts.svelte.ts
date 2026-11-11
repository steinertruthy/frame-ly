export type ToastKind = 'success' | 'error' | 'info';
export interface Toast {
  id: number;
  kind: ToastKind;
  text: string;
}

let nextId = 1;
export const toasts = $state<Toast[]>([]);

export function toast(kind: ToastKind, text: string, ms = 2600): void {
  const id = nextId++;
  toasts.push({ id, kind, text });
  setTimeout(() => dismiss(id), ms);
}

export function dismiss(id: number): void {
  const i = toasts.findIndex((t) => t.id === id);
  if (i >= 0) toasts.splice(i, 1);
}
