// Données de DÉMONSTRATION de la vue d'ensemble d'une enquête.
//
// L'import de preuves arrive en phase 2 : en attendant, la vue d'ensemble est
// illustrée par des chiffres fictifs, générés de façon déterministe à partir de
// l'identifiant de l'enquête (même enquête → mêmes chiffres) et toujours
// affichés avec la mention « données de démonstration ».

export interface DemoOverview {
  events: number;
  sources: number;
  entities: number;
  alerts: number;
  /** Événements par heure sur 24 h (index 0 = 00 h). */
  hourly: number[];
  /** Répartition des alertes par gravité. */
  severity: { low: number; medium: number; high: number; critical: number };
  /** Principales sources, du plus grand volume au plus petit. */
  topSources: Array<{ name: string; events: number }>;
}

/** Générateur pseudo-aléatoire déterministe (mulberry32). */
function seeded(seed: number): () => number {
  let a = seed >>> 0;
  return () => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const SOURCE_NAMES = [
  "nginx/access.log",
  "auth.log",
  "pare-feu.csv",
  "capture-dmz.pcap",
  "syslog",
  "journal-windows.evtx",
] as const;

export function demoOverview(investigationId: number): DemoOverview {
  const rand = seeded(investigationId * 2654435761);
  const between = (min: number, max: number) => Math.round(min + rand() * (max - min));

  // Activité : creux la nuit, pic d'incident à une heure tirée au sort.
  const peak = between(9, 20);
  const hourly = Array.from({ length: 24 }, (_, h) => {
    const daytime = h >= 7 && h <= 21 ? 1 : 0.3;
    const burst = Math.max(0, 1 - Math.abs(h - peak) / 3) * between(300, 900);
    return Math.round(daytime * between(40, 140) + burst);
  });
  const events = hourly.reduce((a, b) => a + b, 0);

  const sourceCount = between(3, SOURCE_NAMES.length);
  const weights = Array.from({ length: sourceCount }, () => 0.2 + rand());
  const total = weights.reduce((a, b) => a + b, 0);
  const topSources = weights
    .map((w, i) => ({ name: SOURCE_NAMES[i] ?? `source-${i}`, events: Math.round((w / total) * events) }))
    .sort((a, b) => b.events - a.events);

  const severity = {
    low: between(4, 20),
    medium: between(2, 12),
    high: between(0, 6),
    critical: between(0, 2),
  };

  return {
    events,
    sources: sourceCount,
    entities: between(12, 140),
    alerts: severity.low + severity.medium + severity.high + severity.critical,
    hourly,
    severity,
    topSources,
  };
}
