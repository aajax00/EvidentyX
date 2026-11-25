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

const formatters = (utc: boolean) => {
  const timeZone = utc ? { timeZone: "UTC" } : {};
  return {
    time: new Intl.DateTimeFormat("fr-FR", { hour: "2-digit", minute: "2-digit", second: "2-digit", ...timeZone }),
    date: new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "2-digit", year: "numeric", ...timeZone }),
    dateTime: new Intl.DateTimeFormat("fr-FR", { dateStyle: "medium", timeStyle: "short", ...timeZone }),
  };
};
const LOCAL = formatters(false);
const UTC = formatters(true);
const fmt = (utc: boolean) => (utc ? UTC : LOCAL);

/** Nom du fuseau d'affichage, pour les libellés. */
export const zoneLabel = (utc: boolean) => (utc ? "UTC" : "heure locale");

/** « 14:32:17 » ; « — » si la source ne donne que la date ou aucune heure. */
export function eventClock(e: TimelineEvent, utc: boolean): string {
  return e.timeUs === null || e.timePrecision === "day" ? "—" : fmt(utc).time.format(Math.floor(e.timeUs / 1000));
}

/** « 12/04/2026 ». */
export function eventDate(e: TimelineEvent, utc: boolean): string {
  return e.timeUs === null ? "sans heure" : fmt(utc).date.format(Math.floor(e.timeUs / 1000));
}

/** « 12 avr. 2026, 14:32 » pour une date en ms. */
export function formatMs(ms: number, utc: boolean): string {
  return fmt(utc).dateTime.format(ms);
}

/** Heure complète, avec la fraction que la source fournit réellement. */
export function eventTimeFull(e: TimelineEvent, utc: boolean): string {
  if (e.timeUs === null) return "Aucune heure dans la source";
  const ms = Math.floor(e.timeUs / 1000);
  const f = fmt(utc);
  if (e.timePrecision === "day") return f.date.format(ms);
  const frac =
    e.timePrecision === "millisecond"
      ? `,${String(Math.floor((e.timeUs / 1000) % 1000)).padStart(3, "0")}`
      : e.timePrecision === "microsecond"
        ? `,${String(e.timeUs % 1_000_000).padStart(6, "0")}`
        : "";
  return `${f.date.format(ms)} ${f.time.format(ms)}${frac} (${zoneLabel(utc)})`;
}

export const PRECISION_LABELS: Record<string, string> = {
  day: "jour (date seule)",
  second: "seconde",
  millisecond: "milliseconde",
  microsecond: "microseconde",
};

/** Heure à prendre avec précaution : fuseau supposé ou année déduite. */
export const isUncertain = (e: TimelineEvent) => e.timeBasis === "assumed_utc" || e.timeBasis === "inferred_year";

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
