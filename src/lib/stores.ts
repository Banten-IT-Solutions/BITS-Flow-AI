import { getCurrentWindow } from "@tauri-apps/api/window";
import { get, writable } from "svelte/store";
import { getSettings, listAccounts, listPrompts, listProviders, listRecent } from "./api";
import { m } from "./paraglide/messages";
import { resolveLocale, resolveTheme } from "./settings";
import { toast } from "./toast";
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
/** Labels of persona windows that are currently open (Home badge). */
export const openLabels = writable<string[]>([]);
/** Recent "persona opened provider" activity, newest first. */
export const recent = writable<RecentEntry[]>([]);
/** Bumped whenever the UI locale changes so views re-render translated strings. */
export const uiLocale = writable<"id" | "en">("id");

export async function loadAll(): Promise<void> {
  let loaded: [Settings, Account[], Provider[], Prompt[], RecentEntry[]];
  try {
    loaded = await Promise.all([
      getSettings(),
      listAccounts(),
      listProviders(),
      listPrompts(),
      listRecent().catch(() => []),
    ]);
  } catch (e) {
    console.error("init failed", e);
    toast(m.errorsLoadFailed());
    throw e;
  }
  const [s, a, p, pr, r] = loaded;
  settings.set(s);
  accounts.set(a);
  providers.set(p);
  prompts.set(pr);
  recent.set(r);
  applyTheme();
  uiLocale.set(applyLocale());
}

/**
 * System theme, resolved from the Tauri window (backed by the XDG
 * color-scheme portal). The prefers-color-scheme media query is not used:
 * on WebKitGTK it is derived from GTK settings that two writers keep
 * flipping (the theme-name heuristic reads Mint-Y-Dark-Aqua as light while
 * the portal says prefer-dark), so following it makes the whole window
 * flicker light<->dark. ponytail: revisit if WebKitGTK ships a stable
 * color-scheme source.
 */
let systemThemeDark: boolean | null = null;

async function resolveSystemThemeDark(): Promise<void> {
  try {
    systemThemeDark = (await getCurrentWindow().theme()) === "dark";
  } catch {
    // Outside Tauri (or theme API unavailable): fall back to a one-shot
    // matchMedia read; it is only a hint, never a live signal we follow.
    systemThemeDark =
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches;
  }
  applyTheme();
}

export function applyTheme() {
  const { theme } = get(settings);
  // Keep stylesheet's initial light value until portal-backed preference is
  // ready; otherwise loadAll() can paint light before async theme resolution.
  if (theme === "system" && systemThemeDark === null) return;
  const resolved = resolveTheme(theme, systemThemeDark ?? false);
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
  // Snapshot the portal-backed theme once at startup. On the affected
  // WebKitGTK/GTK stack, runtime theme notifications trigger contradictory
  // GTK preference updates and repeatedly repaint the entire webview. Do not
  // subscribe to either WebKit's media query or GTK/Tauri theme events.
  void resolveSystemThemeDark();
  return () => {};
}
