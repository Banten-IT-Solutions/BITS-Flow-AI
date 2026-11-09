/** Mirror of the Rust data model (src-tauri/src/commands.rs). Keep in sync. */

export type Category = "image" | "video" | "both";
export type Engine = "webview" | "chrome";
export type LanguageSetting = "id" | "en";
export type CloseBehavior = "exit" | "hide";

export interface Account {
  id: string;
  name: string;
  email: string;
  color?: string | null;
  createdAt: string;
}

export interface Provider {
  id: string;
  name: string;
  url: string;
  category: Category;
  engine: Engine;
  custom: boolean;
}

export interface Prompt {
  id: string;
  title: string;
  body: string;
  category: Category;
  pinned: boolean;
}

export interface Settings {
  language: LanguageSetting;
  closeBehavior: CloseBehavior;
  lruSize: number;
}

/** One "persona opened provider" activity row from recent.json. */
export interface RecentEntry {
  providerId: string;
  personaId: string;
  personaName: string;
  providerName: string;
  ts: number;
}

export const DEFAULT_SETTINGS: Settings = {
  language: "en",
  closeBehavior: "exit",
  lruSize: 3,
};

export type MenuId = "home" | "accounts" | "providers" | "generate" | "prompts" | "settings";

/** Fixed 8-color persona palette (no purple-blue per design language). */
export const PERSONA_COLORS = [
  "#0f766e",
  "#16a34a",
  "#d97706",
  "#ea580c",
  "#dc2626",
  "#db2777",
  "#0891b2",
  "#64748b",
] as const;

/** Repository URL shown in Settings → About. */
export const REPO_URL = "https://github.com/Banten-IT-Solutions/BITS-Flow-AI";
