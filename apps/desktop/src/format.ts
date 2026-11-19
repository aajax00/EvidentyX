// Mise en forme des dates pour l'affichage (fuseau local de l'analyste).

const dateTime = new Intl.DateTimeFormat("fr-FR", { dateStyle: "medium", timeStyle: "short" });
const relative = new Intl.RelativeTimeFormat("fr-FR", { numeric: "auto" });

/** « 19 nov. 2026, 14:05 » */
export function formatDateTime(unixMillis: number): string {
  return dateTime.format(unixMillis);
}

const UNITS: ReadonlyArray<[Intl.RelativeTimeFormatUnit, number]> = [
  ["year", 365 * 86_400_000],
  ["month", 30 * 86_400_000],
  ["week", 7 * 86_400_000],
  ["day", 86_400_000],
  ["hour", 3_600_000],
  ["minute", 60_000],
];

/** « il y a 5 minutes », « hier », « à l'instant ». */
export function formatRelative(unixMillis: number, now: number = Date.now()): string {
  const diff = unixMillis - now;
  for (const [unit, size] of UNITS) {
    if (Math.abs(diff) >= size) return relative.format(Math.round(diff / size), unit);
  }
  return "à l'instant";
}
