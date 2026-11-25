// Mise en forme des événements (libellés, heures) partagée par la vue d'ensemble.

import type { TimelineEvent } from "../api";

export const KIND_LABELS: Record<string, string> = {
  authentication: "Authentification",
  network: "Réseau",
  web: "Web",
  process: "Processus",
  file: "Fichier",
  system: "Système",
  other: "Autre",
};

export const kindLabel = (kind: string) => KIND_LABELS[kind] ?? kind;

const timeFmt = new Intl.DateTimeFormat("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit" });
const dateFmt = new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric" });

/** « 14:32:17 » (heure locale). */
export function eventClock(e: TimelineEvent): string {
  return e.timeUs === null ? "—" : timeFmt.format(Math.floor(e.timeUs / 1000));
}

/** « 12/04/2026 » (date locale). */
export function eventDate(e: TimelineEvent): string {
  return e.timeUs === null ? "sans heure" : dateFmt.format(Math.floor(e.timeUs / 1000));
}

/** Heure complète, avec la fraction que la source fournit réellement. */
export function eventTimeFull(e: TimelineEvent): string {
  if (e.timeUs === null) return "Aucune heure dans la source";
  const ms = Math.floor(e.timeUs / 1000);
  if (e.timePrecision === "day") return dateFmt.format(ms);
  const frac =
    e.timePrecision === "millisecond"
      ? `,${String(Math.floor((e.timeUs / 1000) % 1000)).padStart(3, "0")}`
      : e.timePrecision === "microsecond"
        ? `,${String(e.timeUs % 1_000_000).padStart(6, "0")}`
        : "";
  return `${dateFmt.format(ms)} ${timeFmt.format(ms)}${frac}`;
}

export const BASIS_LABELS: Record<string, string> = {
  explicit: "fuseau indiqué par la source",
  assumed_utc: "fuseau absent : lu comme UTC",
  inferred_year: "année déduite de la date du fichier, lu comme UTC",
};

export function endpoint(ip: string | null, port: number | null): string | null {
  if (!ip) return null;
  if (port === null) return ip;
  return ip.includes(":") ? `[${ip}]:${port}` : `${ip}:${port}`;
}

/** Ligne secondaire d'un événement : qui, d'où, vers où. */
export function eventSubline(e: TimelineEvent): string {
  const src = endpoint(e.sourceIp, e.sourcePort);
  const dst = endpoint(e.destinationIp, e.destinationPort);
  const parts = [src && dst ? `${src} → ${dst}` : (src ?? dst), e.user && `utilisateur ${e.user}`, e.host];
  return parts.filter(Boolean).join(" · ");
}
