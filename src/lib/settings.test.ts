import { describe, expect, it } from "vitest";
import { filterProviders, languageOptions } from "./settings";
import type { Provider } from "./types";

const providers: Provider[] = [
  {
    id: "flow",
    name: "Flow",
    url: "https://labs.google/fx/tools/flow",
    category: "both",
    engine: "webview",
    custom: false,
  },
  {
    id: "gemini",
    name: "Gemini",
    url: "https://gemini.google.com",
    category: "image",
    engine: "webview",
    custom: false,
  },
  {
    id: "pixverse",
    name: "PixVerse",
    url: "https://app.pixverse.ai",
    category: "video",
    engine: "webview",
    custom: false,
  },
];

describe("languageOptions", () => {
  it("offers exactly Indonesia and English as default", () => {
    expect(languageOptions()).toEqual([
      { value: "id", label: "Bahasa Indonesia" },
      { value: "en", label: "English (Default)" },
    ]);
  });
});

describe("filterProviders", () => {
  it("shows exact category plus both", () => {
    const image = filterProviders(providers, "image").map((p) => p.id);
    const video = filterProviders(providers, "video").map((p) => p.id);
    expect(image).toEqual(["flow", "gemini"]);
    expect(video).toEqual(["flow", "pixverse"]);
  });
});
