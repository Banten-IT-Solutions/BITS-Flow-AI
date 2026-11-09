import type { Category, LanguageSetting, Provider } from "./types";

/** Options for the language setting: Indonesia plus English as default. */
export function languageOptions(): { value: LanguageSetting; label: string }[] {
  return [
    { value: "id", label: "Bahasa Indonesia" },
    { value: "en", label: "English (Default)" },
  ];
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
