import { describe, expect, it } from "vitest";
import { z } from "zod";
import { mergeById, parseItems } from "./transfer";

const item = z.object({ id: z.string().min(1) });

describe("parseItems", () => {
  const raw = (body: unknown) => JSON.stringify(body);

  it("accepts a matching payload", () => {
    const body = { kind: "accounts", version: 1, items: [{ id: "a" }, { id: "b" }] };
    expect(parseItems("accounts", item, raw(body))).toEqual([{ id: "a" }, { id: "b" }]);
  });

  it("rejects the other kind, wrong versions, and broken JSON", () => {
    const wrong = { kind: "prompts", version: 1, items: [{ id: "a" }] };
    const old = { kind: "accounts", version: 2, items: [] };
    expect(parseItems("accounts", item, raw(wrong))).toBeNull();
    expect(parseItems("accounts", item, raw(old))).toBeNull();
    expect(parseItems("accounts", item, "not json")).toBeNull();
    expect(
      parseItems("accounts", item, raw({ kind: "accounts", version: 1, items: [{}] })),
    ).toBeNull();
  });
});

describe("mergeById", () => {
  it("appends only unseen ids and counts skipped duplicates", () => {
    const current = [{ id: "a" }, { id: "b" }];
    const incoming = [{ id: "b" }, { id: "c" }, { id: "c" }];
    const { merged, skipped } = mergeById(current, incoming);
    expect(merged.map((i) => i.id)).toEqual(["a", "b", "c"]);
    expect(skipped).toBe(2);
  });

  it("keeps current order and returns zero skipped for disjoint sets", () => {
    const { merged, skipped } = mergeById([{ id: "x" }], [{ id: "y" }]);
    expect(merged.map((i) => i.id)).toEqual(["x", "y"]);
    expect(skipped).toBe(0);
  });
});
