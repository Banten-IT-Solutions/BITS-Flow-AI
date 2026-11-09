import { describe, expect, it } from "vitest";
import { isNewerVersion } from "./updates";

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
