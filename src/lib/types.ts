/** Mirror of the Rust data model (src-tauri/src/commands.rs). Keep in sync. */

export type Category = "image" | "video" | "both";
export type Engine = "webview" | "chrome";
export type LanguageSetting = "system" | "id" | "en";
export type ThemeSetting = "system" | "light" | "dark";
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
  theme: ThemeSetting;
  closeBehavior: CloseBehavior;
  lruSize: number;
}

export const DEFAULT_SETTINGS: Settings = {
  language: "system",
  theme: "system",
  closeBehavior: "exit",
  lruSize: 3,
};

export type MenuId = "home" | "accounts" | "providers" | "generate" | "prompts" | "settings";
