import { describe, expect, it } from "vitest";
import { filterProviders, resolveLocale } from "./settings";
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

describe("resolveLocale", () => {
  it("follows the system locale and defaults to id", () => {
    expect(resolveLocale("system", "id-ID")).toBe("id");
    expect(resolveLocale("system", "en-US")).toBe("en");
    expect(resolveLocale("system", "ja-JP")).toBe("en");
  });

  it("uses the explicit override", () => {
    expect(resolveLocale("en", "id-ID")).toBe("en");
    expect(resolveLocale("id", "en-US")).toBe("id");
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
