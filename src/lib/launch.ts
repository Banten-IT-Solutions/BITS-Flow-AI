import { listOpenPersonas, logActivity, type OpenPersonaResult, openPersona } from "./api";
import { openLabels, recent } from "./stores";
import type { Account, Provider, RecentEntry } from "./types";

/** Most recently used provider for a persona, or undefined if never used. */
export function lastUsedProvider(
  account: Account,
  recents: RecentEntry[],
  available: Provider[],
): Provider | undefined {
  const latest = recents.find((r) => r.personaId === account.id);
  if (!latest) return undefined;
  return available.find((p) => p.id === latest.providerId);
}

/**
 * Open the isolated persona window for account+provider, sync stores and
 * append the activity row (best-effort: a failed log never fails the launch).
 */
export async function launchPersona(
  account: Account,
  provider: Provider,
): Promise<OpenPersonaResult> {
  const result = await openPersona(account.id, provider.url, provider.engine);
  void refreshOpenPersonas();
  try {
    recent.set(
      await logActivity({
        providerId: provider.id,
        personaId: account.id,
        personaName: account.name,
        providerName: provider.name,
      }),
    );
  } catch {
    // Activity log is best-effort.
  }
  return result;
}

/** Mirror the Rust-side open persona window labels into the store. */
export async function refreshOpenPersonas(): Promise<void> {
  try {
    openLabels.set(await listOpenPersonas());
  } catch {
    // Badge refresh is best-effort.
  }
}
