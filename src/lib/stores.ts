import { get, writable } from "svelte/store";
import { getSettings, listAccounts, listPrompts, listProviders, listRecent } from "./api";
import { resolveLocale, resolveTheme } from "./settings";
import {
  type Account,
  DEFAULT_SETTINGS,
  type MenuId,
  type Prompt,
  type Provider,
  type RecentEntry,
  type Settings,
} from "./types";

/** Active sidebar menu — no router, views switch on this. */
export const activeMenu = writable<MenuId>("home");

export const settings = writable<Settings>(DEFAULT_SETTINGS);
export const accounts = writable<Account[]>([]);
export const providers = writable<Provider[]>([]);
export const prompts = writable<Prompt[]>([]);
export const activePersona = writable<string | null>(null);
/** Persona labels in open order (oldest first), mirrored from the Rust LRU. */
export const openOrder = writable<string[]>([]);
/** Labels of persona windows that are currently open (Home badge). */
export const openLabels = writable<string[]>([]);
/** Recent "persona opened provider" activity, newest first. */
export const recent = writable<RecentEntry[]>([]);
/** Bumped whenever the UI locale changes so views re-render translated strings. */
export const uiLocale = writable<"id" | "en">("id");

export async function loadAll(): Promise<void> {
  const [s, a, p, pr, r] = await Promise.all([
    getSettings(),
    listAccounts(),
    listProviders(),
    listPrompts(),
    listRecent().catch(() => []),
  ]);
  settings.set(s);
  accounts.set(a);
  providers.set(p);
  prompts.set(pr);
  recent.set(r);
  applyTheme();
  uiLocale.set(applyLocale());
}

const darkQuery =
  typeof window !== "undefined" && typeof window.matchMedia === "function"
    ? window.matchMedia("(prefers-color-scheme: dark)")
    : null;

export function applyTheme() {
  const { theme } = get(settings);
  const resolved = resolveTheme(theme, darkQuery?.matches ?? false);
  if (typeof document !== "undefined") {
    document.documentElement.dataset.theme = resolved;
  }
}

export function applyLocale() {
  const { language } = get(settings);
  const systemLocale = typeof navigator !== "undefined" ? navigator.language : "id";
  return resolveLocale(language, systemLocale);
}

export function initSystemListeners(): () => void {
  if (!darkQuery) return () => {};
  const onChange = () => applyTheme();
  darkQuery.addEventListener("change", onChange);
  return () => darkQuery.removeEventListener("change", onChange);
}
