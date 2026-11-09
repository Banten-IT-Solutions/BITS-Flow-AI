/** Relative timestamps via Intl.RelativeTimeFormat (locked design). */

const LOCALE_TAGS = { id: "id-ID", en: "en-US" } as const;

/** "5 minutes ago" / "5 menit yang lalu" style, bucketed minute→month. */
export function formatRelative(ts: number, locale: "id" | "en", now = Date.now()): string {
  const rtf = new Intl.RelativeTimeFormat(LOCALE_TAGS[locale], { numeric: "auto" });
  const diff = ts - now;
  // Compute on the magnitude so rounding is symmetric around zero.
  const past = diff < 0;
  const sign = past ? -1 : 1;
  const minutes = Math.round(Math.abs(diff) / 60_000);
  if (minutes < 60) return rtf.format(sign * minutes, "minute");
  const hours = Math.round(Math.abs(diff) / 3_600_000);
  if (hours < 24) return rtf.format(sign * hours, "hour");
  const days = Math.round(Math.abs(diff) / 86_400_000);
  if (days < 30) return rtf.format(sign * days, "day");
  return rtf.format(sign * Math.round(days / 30), "month");
}
