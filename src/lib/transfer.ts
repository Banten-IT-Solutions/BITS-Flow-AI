import { z } from "zod";

/**
 * Export file shape: {"kind":"accounts"|"prompts","version":1,"items":[...]}.
 * `kind` guards against importing an accounts file into prompts and back.
 */
function payloadSchema<T>(kind: "accounts" | "prompts", item: z.ZodType<T>) {
  return z.object({
    kind: z.literal(kind),
    version: z.literal(1),
    items: z.array(item),
  });
}

/** Parse an export file body; null when it is not valid JSON of this kind. */
export function parseItems<T>(
  kind: "accounts" | "prompts",
  itemSchema: z.ZodType<T>,
  raw: string,
): T[] | null {
  try {
    const parsed = payloadSchema(kind, itemSchema).parse(JSON.parse(raw));
    return parsed.items;
  } catch {
    return null;
  }
}

/** Append entries whose ids are not present yet; returns how many were skipped. */
export function mergeById<T extends { id: string }>(
  current: T[],
  incoming: T[],
): { merged: T[]; skipped: number } {
  const ids = new Set(current.map((item) => item.id));
  const fresh: T[] = [];
  for (const item of incoming) {
    if (!ids.has(item.id)) {
      ids.add(item.id);
      fresh.push(item);
    }
  }
  return { merged: [...current, ...fresh], skipped: incoming.length - fresh.length };
}
