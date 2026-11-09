import { describe, expect, it } from "vitest";
import { formatRelative } from "./time";

const NOW = 1_000_000_000_000;

describe("formatRelative", () => {
  it("buckets minutes, hours, days, months", () => {
    const min = 60_000;
    expect(formatRelative(NOW - 2 * min, "en", NOW)).toBe("2 minutes ago");
    expect(formatRelative(NOW - 3 * 60 * min, "en", NOW)).toBe("3 hours ago");
    expect(formatRelative(NOW - 2 * 24 * 60 * min, "en", NOW)).toBe("2 days ago");
    expect(formatRelative(NOW - 45 * 24 * 60 * min, "en", NOW)).toBe("2 months ago");
  });

  it("renders the Indonesian locale", () => {
    expect(formatRelative(NOW - 5 * 60_000, "id", NOW)).toBe("5 menit yang lalu");
    expect(formatRelative(NOW + 60_000, "id", NOW)).toBe("dalam 1 menit");
  });

  it("uses auto numeric phrasing for near-now", () => {
    expect(formatRelative(NOW - 30_000, "en", NOW)).toBe("1 minute ago");
  });
});
