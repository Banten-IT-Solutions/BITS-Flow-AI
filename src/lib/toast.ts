import { writable } from "svelte/store";

export interface ToastAction {
  label: string;
  run: () => void;
}

export interface Toast {
  id: number;
  message: string;
  action?: ToastAction;
}

export const toasts = writable<Toast[]>([]);

let nextId = 1;

/** Auto-dismiss 4s; 5s when the toast carries an action (Undo window). */
export function toast(message: string, action?: ToastAction): void {
  const id = nextId++;
  toasts.update((list) => [...list, { id, message, action }]);
  setTimeout(() => dismiss(id), action ? 5000 : 4000);
}

export function dismiss(id: number): void {
  toasts.update((list) => list.filter((t) => t.id !== id));
}
