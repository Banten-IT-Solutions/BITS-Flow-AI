import { m } from "./paraglide/messages";

/**
 * Map a rejected command's raw error string to a localized message.
 * Rust returns i18n keys (e.g. "error.close_first") for user-facing failures;
 * anything else is rendered as a generic command failure.
 */
export function commandError(e: unknown): string {
  const raw = e instanceof Error ? e.message : String(e);
  if (raw === "error.close_first") return m.errorsCloseFirst();
  if (raw === "error.no_data_dir") return m.errorNoDataDir();
  return m.errorCommandFailed({ message: raw });
}
