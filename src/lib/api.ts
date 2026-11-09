import { invoke } from "@tauri-apps/api/core";
import { z } from "zod";
import { DEFAULT_SETTINGS } from "./types";

export const accountSchema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  email: z.email(),
  color: z.string().min(1).nullish(),
  createdAt: z.string().min(1),
});

export const providerSchema = z.object({
  id: z.string().min(1),
  name: z.string().min(1),
  url: z.url(),
  category: z.enum(["image", "video", "both"]),
  engine: z.enum(["webview", "chrome"]),
  custom: z.boolean(),
});

export const promptSchema = z.object({
  id: z.string().min(1),
  title: z.string().min(1),
  body: z.string().min(1),
  category: z.enum(["image", "video", "both"]),
  pinned: z.boolean(),
});

export const settingsSchema = z.object({
  language: z.enum(["system", "id", "en"]),
  theme: z.enum(["system", "light", "dark"]),
  closeBehavior: z.enum(["exit", "hide"]),
  lruSize: z.number().int().min(1).max(10),
});

export const openPersonaResultSchema = z.object({
  label: z.string(),
  openOrder: z.array(z.string()),
});

export type OpenPersonaResult = z.infer<typeof openPersonaResultSchema>;

export function listAccounts() {
  return invoke("list_accounts").then(z.array(accountSchema).parse);
}

export function saveAccounts(accounts: unknown) {
  return invoke("save_accounts", { accounts: z.array(accountSchema).parse(accounts) });
}

export function listProviders() {
  return invoke("list_providers").then(z.array(providerSchema).parse);
}

export function saveProviders(providers: unknown) {
  return invoke("save_providers", { providers: z.array(providerSchema).parse(providers) });
}

export function listPrompts() {
  return invoke("list_prompts").then(z.array(promptSchema).parse);
}

export function savePrompts(prompts: unknown) {
  return invoke("save_prompts", { prompts: z.array(promptSchema).parse(prompts) });
}

export function getSettings() {
  return invoke("get_settings").then((raw) => settingsSchema.parse(raw));
}

export function saveSettings(settings: unknown) {
  return invoke("save_settings", { settings: settingsSchema.parse(settings) });
}

export function openPersona(personaId: string, url: string, engine: "webview" | "chrome") {
  return invoke("open_persona", { personaId, url, engine }).then(openPersonaResultSchema.parse);
}

export const recentEntrySchema = z.object({
  providerId: z.string().min(1),
  personaId: z.string().min(1),
  personaName: z.string().min(1),
  providerName: z.string().min(1),
  ts: z.number(),
});

export type RecentEntryParsed = z.infer<typeof recentEntrySchema>;

export function listRecent() {
  return invoke("list_recent").then(z.array(recentEntrySchema).parse);
}

export function logActivity(input: {
  providerId: string;
  personaId: string;
  personaName: string;
  providerName: string;
}) {
  return invoke("log_activity", { input }).then(z.array(recentEntrySchema).parse);
}

export function clearPersonaData(personaId: string) {
  return invoke("clear_persona_data", { personaId });
}

export const clearAllResultSchema = z.object({
  cleared: z.array(z.string()),
  skipped: z.array(z.string()),
});

export type ClearAllResult = z.infer<typeof clearAllResultSchema>;

export function clearAllPersonaData() {
  return invoke("clear_all_persona_data").then(clearAllResultSchema.parse);
}

export function listOpenPersonas() {
  return invoke("list_open_personas").then(z.array(z.string()).parse);
}

/** Invalid URL scheme check shared with the Rust boundary (http/https only). */
export function isValidProviderUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === "http:" || parsed.protocol === "https:";
  } catch {
    return false;
  }
}

export { DEFAULT_SETTINGS };
