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

const sizeFormat = new Intl.NumberFormat("fr-FR", { maximumFractionDigits: 1 });

/** Taille lisible en unités binaires françaises : « 12,4 Mo », « 980 o ». */
export function formatBytes(bytes: number): string {
  const units = ["o", "Ko", "Mo", "Go", "To"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${unit === 0 ? value : sizeFormat.format(value)} ${units[unit]}`;
}

/** Empreinte abrégée pour l'affichage : « 3f8a1c0d9be2…3c5b29 ». */
export function shortDigest(sha256: string): string {
  return sha256.length > 20 ? `${sha256.slice(0, 12)}…${sha256.slice(-6)}` : sha256;
}
