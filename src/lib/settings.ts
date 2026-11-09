import type { Category, LanguageSetting, Provider } from "./types";

/** Resolve a settings value against the host preference. Default locale is `id`. */
export function resolveLocale(setting: LanguageSetting, systemLocale: string): "id" | "en" {
  if (setting === "system") {
    return systemLocale.toLowerCase().startsWith("id") ? "id" : "en";
  }
  return setting;
}

export function categoryLabel(
  category: Category,
  t: { image: string; video: string; both: string },
): string {
  if (category === "image") return t.image;
  if (category === "video") return t.video;
  return t.both;
}

/** Providers usable from a tab: exact category plus "both". */
export function filterProviders(providers: Provider[], tab: Exclude<Category, "both">): Provider[] {
  return providers.filter((p) => p.category === tab || p.category === "both");
}
