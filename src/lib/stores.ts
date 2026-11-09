import { get, writable } from "svelte/store";
import { getSettings, listAccounts, listPrompts, listProviders, listRecent } from "./api";
import { m } from "./paraglide/messages";
import { resolveLocale } from "./settings";
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
  uiLocale.set(applyLocale());
}

export function applyLocale() {
  const { language } = get(settings);
  const systemLocale = typeof navigator !== "undefined" ? navigator.language : "id";
  return resolveLocale(language, systemLocale);
}
