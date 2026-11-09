import { z } from "zod";
import { REPO_URL } from "./types";

const LATEST_RELEASE_API = `${REPO_URL.replace("https://github.com", "https://api.github.com")}/releases/latest`;

const releaseSchema = z.object({ tag_name: z.string().min(1) });

/**
 * True when `candidate` is strictly newer than `current`. Tags are bare
 * semver (release.yml validates MAJOR.MINOR.PATCH); a prerelease suffix
 * compares equal to the plain release, so it never offers an update.
 */
export function isNewerVersion(candidate: string, current: string): boolean {
  const parts = (v: string) =>
    v
      .replace(/^v/, "")
      .split(/[.-]/)
      .slice(0, 3)
      .map((n) => Number(n) || 0);
  const next = parts(candidate);
  const cur = parts(current);
  for (let i = 0; i < 3; i++) {
    if (next[i] !== cur[i]) return (next[i] ?? 0) > (cur[i] ?? 0);
  }
  return false;
}

/**
 * Ask GitHub whether a newer release exists; return its tag, or null when
 * up to date or unreachable (offline, rate limit) — never throws.
 */
export async function checkForUpdate(currentVersion: string): Promise<string | null> {
  try {
    const res = await fetch(LATEST_RELEASE_API, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) return null;
    const { tag_name } = releaseSchema.parse(await res.json());
    return isNewerVersion(tag_name, currentVersion) ? tag_name : null;
  } catch {
    return null;
  }
}
