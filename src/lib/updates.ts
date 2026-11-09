import { z } from "zod";
import { REPO_URL } from "./types";

/** api.github.com counterpart of a github.com repo URL. */
export function latestReleaseApi(repoUrl: string): string {
  return `${repoUrl.replace("https://github.com", "https://api.github.com/repos")}/releases/latest`;
}

const LATEST_RELEASE_API = latestReleaseApi(REPO_URL);

const releaseSchema = z.object({
  tag_name: z.string().min(1),
  assets: z.array(
    z.object({
      name: z.string().min(1),
      browser_download_url: z.url(),
    }),
  ),
});

export interface ReleaseUpdate {
  tag: string;
  debUrl: string;
}

/**
 * Pick the release asset for this app's Debian package: it must carry the
 * release tag in its name, be the amd64 `.deb`, and point back at this
 * repo's release downloads. Returns null when no asset qualifies.
 */
export function pickDebAsset(
  assets: { name: string; browser_download_url: string }[],
  tag: string,
): string | null {
  for (const asset of assets) {
    if (
      asset.name.includes(tag) &&
      asset.name.endsWith("amd64.deb") &&
      asset.browser_download_url.startsWith(`${REPO_URL}/releases/download/`)
    ) {
      return asset.browser_download_url;
    }
  }
  return null;
}

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
 * Ask GitHub whether a newer release exists and return its tag plus the
 * `.deb` download URL, or null when up to date, unreachable, or the release
 * ships no installable package (offline, rate limit) — never throws.
 */
export async function checkForUpdate(currentVersion: string): Promise<ReleaseUpdate | null> {
  try {
    const res = await fetch(LATEST_RELEASE_API, {
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) return null;
    const release = releaseSchema.parse(await res.json());
    if (!isNewerVersion(release.tag_name, currentVersion)) return null;
    const debUrl = pickDebAsset(release.assets, release.tag_name);
    return debUrl ? { tag: release.tag_name, debUrl } : null;
  } catch {
    return null;
  }
}
