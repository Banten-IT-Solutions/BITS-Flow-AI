import { describe, expect, it } from "vitest";
import { isNewerVersion, pickDebAsset } from "./updates";

describe("isNewerVersion", () => {
  it("detects newer patch, minor, and major versions", () => {
    expect(isNewerVersion("1.1.10", "1.1.9")).toBe(true);
    expect(isNewerVersion("1.2.0", "1.1.9")).toBe(true);
    expect(isNewerVersion("2.0.0", "1.9.9")).toBe(true);
  });

  it("rejects equal and older versions", () => {
    expect(isNewerVersion("1.1.9", "1.1.9")).toBe(false);
    expect(isNewerVersion("1.1.8", "1.1.9")).toBe(false);
    expect(isNewerVersion("1.0.0", "1.1.9")).toBe(false);
  });

  it("accepts a v prefix and ignores prerelease suffixes", () => {
    expect(isNewerVersion("v1.2.0", "1.1.9")).toBe(true);
    expect(isNewerVersion("1.2.0-rc.1", "1.2.0")).toBe(false);
  });

  it("treats missing parts as zero", () => {
    expect(isNewerVersion("1.2", "1.1.9")).toBe(true);
    expect(isNewerVersion("1", "1.0.0")).toBe(false);
  });
});

describe("pickDebAsset", () => {
  const repo = "https://github.com/Banten-IT-Solutions/BITS-Flow-AI/releases/download/1.3.0";
  const deb = {
    name: "BITS Flow AI_1.3.0_amd64.deb",
    browser_download_url: `${repo}/BITS%20Flow%20AI_1.3.0_amd64.deb`,
  };

  it("picks the matching amd64 deb asset", () => {
    expect(pickDebAsset([deb], "1.3.0")).toBe(deb.browser_download_url);
  });

  it("rejects other tags, formats, and foreign hosts", () => {
    expect(pickDebAsset([deb], "1.2.9")).toBeNull();
    expect(pickDebAsset([{ ...deb, name: "app.AppImage" }], "1.3.0")).toBeNull();
    expect(
      pickDebAsset([{ ...deb, browser_download_url: "https://evil.example/x.deb" }], "1.3.0"),
    ).toBeNull();
    expect(pickDebAsset([], "1.3.0")).toBeNull();
  });
});
