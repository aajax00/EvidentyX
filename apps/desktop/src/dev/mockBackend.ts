// Moteur de démonstration — développement dans un navigateur uniquement.
//
// Permet de travailler sur l'interface avec `npm run dev` sans l'hôte Tauri.
// Données en mémoire, perdues au rechargement, clairement signalées à l'écran.
// Les règles reproduites ici sont volontairement minimales : la référence est
// le moteur Rust (evidentyx-core, evidentyx-storage). Exclu de la version publiée.

import type {
  AnalysisReport,
  Evidence,
  Investigation,
  InvestigationQuery,
  Overview,
  ProgressHandler,
  Settings,
  StorageInfo,
  EventFilter,
  TimelineEvent,
} from "../api";

const DAY = 86_400_000;
const now = Date.now();
let nextId = 4;
const investigations: Investigation[] = [
  {
    id: 1,
    name: "Intrusion serveur web (démo)",
    description: "Compromission présumée d'un serveur nginx exposé.",
    status: "active",
    createdAt: now - 6 * DAY,
    updatedAt: now - 2 * 3_600_000,
  },
  {
    id: 2,
    name: "Hameçonnage service RH (démo)",
    description: "Campagne ciblée, trois postes concernés.",
    status: "active",
    createdAt: now - 3 * DAY,
    updatedAt: now - DAY,
  },
  {
    id: 3,
    name: "Fuite de données 2025 (démo)",
    description: "Exfiltration via DNS — dossier clos.",
    status: "archived",
    createdAt: now - 90 * DAY,
    updatedAt: now - 40 * DAY,
  },
];

let settings: Settings = {
  splashScreen: true,
  backgroundAnimation: true,
  sidebarCollapsed: false,
  defaultSort: "updated_desc",
  defaultStatusFilter: "active",
  displayUtc: false,
};

const evidence: Evidence[] = [
  {
    id: 1,
    investigationId: 1,
    fileName: "access.log",
    path: "/preuves/demo/nginx/access.log",
    sizeBytes: 18_734_112,
    sha256: "3f8a1c0d9be24f6a7d51e0c2b9a4f7e81d6c3b0a5e9f2d4c7b1a8e6f0d3c5b29",
    fileModifiedAt: now - 7 * DAY,
    addedAt: now - 6 * DAY,
    integrity: "intact",
    verifiedAt: now - 3_600_000,
    eventCount: 0,
  },
];
let nextEvidenceId = 2;
let demoPick = 0;
const DEMO_FILES = ["/preuves/demo/auth.jsonl", "/preuves/demo/capture-dmz.pcap", "/preuves/demo/pare-feu.csv"];

/** Sélection fictive de fichiers (mode démonstration). */
export function demoPickFiles(): string[] {
  const file = DEMO_FILES[demoPick++ % DEMO_FILES.length]!;
  return [file];
}

function fakeDigest(text: string): string {
  let h = 2166136261;
  let out = "";
  for (let i = 0; i < 8; i++) {
    for (const ch of `${text}#${i}`) h = Math.imul(h ^ ch.charCodeAt(0), 16777619) >>> 0;
    out += h.toString(16).padStart(8, "0");
  }
  return out;
}

const fail = (code: string, message: string, field?: string): never => {
  throw { code, message, ...(field ? { field } : {}) };
};

const fold = (s: string) =>
  s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase().replace(/\s+/g, " ").trim();

function validName(raw: unknown): string {
  const name = String(raw ?? "").trim();
  if (!name) fail("validation", "Le nom de l'enquête ne peut pas être vide", "name");
  if ([...name].length > 120) fail("validation", "Le nom de l'enquête est trop long (maximum 120)", "name");
  return name;
}

function validDescription(raw: unknown): string {
  const d = String(raw ?? "").trim();
  if ([...d].length > 4000) fail("validation", "La description est trop longue (maximum 4000)", "description");
  return d;
}

function find(id: unknown): Investigation {
  return investigations.find((i) => i.id === id) ?? fail("not_found", `Enquête introuvable (identifiant ${String(id)})`);
}

function editable(i: Investigation) {
  if (i.status === "archived") fail("archived_read_only", "L'enquête est archivée : réactivez-la avant de la modifier");
}

const touch = (i: Investigation) => {
  i.updatedAt = Math.max(Date.now(), i.updatedAt);
  return { ...i };
};

const cancelled = new Set<string>();
const running = new Set<string>();

/** Tâche longue simulée : progression sur environ 2 s, annulable. */
async function simulateTask(args: Record<string, unknown>, total: number, cancelMessage: string) {
  const taskId = String(args.taskId);
  const onProgress = args.onProgress as ProgressHandler;
  running.add(taskId);
  try {
    for (let i = 0; i <= 20; i++) {
      if (cancelled.delete(taskId)) fail("cancelled", cancelMessage);
      onProgress({ done: Math.round((total * i) / 20), total });
      await new Promise((r) => setTimeout(r, 100));
    }
  } finally {
    running.delete(taskId);
  }
}

/** Événements de démonstration : connexions SSH simulées, sans rapport avec un vrai fichier. */
function demoEvents(investigationId: number): TimelineEvent[] {
  const list = evidence.filter((e) => e.investigationId === investigationId && e.eventCount > 0);
  const out: TimelineEvent[] = [];
  const t0 = (Date.now() - 2 * DAY) * 1000;
  const users = ["root", "admin", "deploy", "www-data"];
  for (const ev of list) {
    for (let i = 0; i < ev.eventCount; i++) {
      const ip = `203.0.113.${(i * 7) % 23}`;
      const ok = i % 5 === 0;
      const user = users[i % users.length]!;
      out.push({
        id: ev.id * 100_000 + i,
        evidenceId: ev.id,
        byteOffset: i * 96,
        line: i + 1,
        record: null,
        timeUs: t0 + i * 432_000_000,
        timePrecision: "second",
        timeBasis: i % 7 === 3 ? "assumed_utc" : "explicit",
        kind: "authentication",
        host: "srv-web-01",
        sourceIp: ip,
        sourcePort: 40000 + i,
        destinationIp: "10.0.0.5",
        destinationPort: 22,
        user,
        action: "ssh-login",
        outcome: ok ? "success" : "failure",
        message: `${ok ? "Accepted" : "Failed"} password for ${user} from ${ip}`,
        raw: `{"ts":"…","user":"${user}","src":"${ip}","status":"${ok ? "ok" : "failed"}"}`,
        rawTruncated: false,
        attributes: { process: "sshd", pid: String(1000 + i) },
      });
    }
  }
  return out;
}

/** Événements démo retenus par le filtre (mêmes règles que le moteur). */
function demoFiltered(args: Record<string, unknown>): TimelineEvent[] {
  const f = args.filter as EventFilter;
  return demoEvents(args.investigationId as number).filter(
    (e) =>
      (f.fromUs === undefined || (e.timeUs !== null && e.timeUs >= f.fromUs)) &&
      (f.toUs === undefined || (e.timeUs !== null && e.timeUs <= f.toUs)) &&
      (!f.kind || e.kind === f.kind) &&
      (f.evidenceId === undefined || e.evidenceId === f.evidenceId) &&
      (!f.outcome || e.outcome === f.outcome) &&
      (!f.time ||
        (f.time === "missing" ? e.timeUs === null : e.timeBasis === "assumed_utc" || e.timeBasis === "inferred_year")) &&
      (!f.ip || !!e.sourceIp?.startsWith(f.ip) || !!e.destinationIp?.startsWith(f.ip)) &&
      (!f.user || !!e.user?.includes(f.user)) &&
      (!f.host || !!e.host?.includes(f.host)) &&
      (!f.text || e.message.toLowerCase().includes(f.text.toLowerCase())),
  );
}

export async function mockInvoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  await new Promise((r) => setTimeout(r, 80));
  if (command === "cancel_task") {
    const id = String(args.taskId);
    if (!running.has(id)) return false as T;
    cancelled.add(id);
    return true as T;
  }
  if (command === "add_evidence") await simulateTask(args, 48_000_000, "Lecture annulée");
  if (command === "analyze_evidence") {
    const e = evidence.find((x) => x.id === args.evidenceId);
    if (e && /\.(jsonl?|ndjson|csv|tsv)$/i.test(e.fileName)) await simulateTask(args, e.sizeBytes, "Analyse annulée : les événements de cette analyse ont été retirés.");
  }
  const result = ((): unknown => {
    switch (command) {
      case "list_evidence":
        find(args.investigationId);
        return evidence.filter((e) => e.investigationId === args.investigationId).map((e) => ({ ...e }));
      case "add_evidence": {
        const inv = find(args.investigationId);
        editable(inv);
        const path = String(args.path);
        const sha256 = fakeDigest(path);
        const dup = evidence.find((e) => e.investigationId === inv.id && e.sha256 === sha256);
        if (dup) fail("duplicate", `Ce contenu est déjà enregistré dans l'enquête (« ${dup.fileName} », même empreinte SHA-256)`);
        const t = Date.now();
        const created: Evidence = {
          id: nextEvidenceId++,
          investigationId: inv.id,
          fileName: path.split("/").pop() ?? path,
          path,
          sizeBytes: 1_000_000 + (parseInt(sha256.slice(0, 6), 16) % 50_000_000),
          sha256,
          fileModifiedAt: t - DAY,
          addedAt: t,
          integrity: "intact",
          verifiedAt: t,
          eventCount: 0,
        };
        evidence.push(created);
        touch(inv);
        return { ...created };
      }
      case "verify_evidence": {
        const e = evidence.find((x) => x.id === args.id) ?? fail("not_found", "Preuve introuvable");
        e.verifiedAt = Date.now();
        return { ...e };
      }
      case "remove_evidence": {
        const e = evidence.find((x) => x.id === args.id) ?? fail("not_found", "Preuve introuvable");
        editable(find(e.investigationId));
        evidence.splice(evidence.indexOf(e), 1);
        return null;
      }
      case "analyze_evidence": {
        const e = evidence.find((x) => x.id === args.evidenceId) ?? fail("not_found", "Preuve introuvable");
        editable(find(e.investigationId));
        if (!/\.(jsonl?|ndjson|csv|tsv)$/i.test(e.fileName))
          fail("unsupported_format", "Format non reconnu pour l'instant : JSON, JSON Lines, CSV, Syslog, journaux d'accès web et captures réseau (PCAP, PCAPNG) sont lus.");
        const isCsv = /\.(csv|tsv)$/i.test(e.fileName);
        const before = e.eventCount;
        e.eventCount = 1240;
        return {
          format: isCsv ? "CSV" : "JSON Lines",
          timeField: isCsv ? "timestamp_utc" : null,
          layout: isCsv ? "séparateur : virgule · 17 colonnes · en-tête" : null,
          records: 1243,
          inserted: e.eventCount - before,
          duplicates: before,
          errors: 3,
          firstErrors: [{ line: 57, byteOffset: 9120, message: "JSON invalide : EOF while parsing an object" }],
        } satisfies AnalysisReport;
      }
      case "events_count":
        return demoFiltered(args).length;
      case "events_window": {
        const offset = args.offset as number;
        return demoFiltered(args).slice(offset, offset + (args.limit as number));
      }
      case "investigation_overview": {
        find(args.investigationId);
        const list = evidence.filter((e) => e.investigationId === args.investigationId);
        const sources = list
          .filter((e) => e.eventCount > 0)
          .map((e) => ({ evidenceId: e.id, fileName: e.fileName, events: e.eventCount }))
          .sort((a, b) => b.events - a.events);
        const events = sources.reduce((a, s) => a + s.events, 0);
        // Répartition régulière fictive : la démonstration ne lit aucun fichier.
        const histogram = events === 0 ? [] : Array.from({ length: 48 }, (_, i) => Math.round((events / 48) * (1 + Math.sin(i / 4))));
        const t = Date.now();
        return {
          events,
          timed: events,
          firstAt: events ? t - 2 * DAY : null,
          lastAt: events ? t : null,
          histogram,
          byKind: events ? [{ kind: "authentication", events }] : [],
          ips: events ? 12 : 0,
          users: events ? 4 : 0,
          hosts: events ? 2 : 0,
          evidenceTotal: list.length,
          sources,
        } satisfies Overview;
      }
      case "get_settings":
        return { ...settings };
      case "update_settings":
        settings = { ...(args.settings as Settings) };
        return { ...settings };
      case "storage_info":
        return { path: "mémoire du navigateur (démonstration)", schemaVersion: 3, error: null } satisfies StorageInfo;
      case "list_investigations": {
        const q = (args.query ?? {}) as InvestigationQuery;
        const terms = fold(q.text ?? "").split(" ").filter(Boolean);
        return investigations
          .filter((i) => !q.status || i.status === q.status)
          .filter((i) => terms.every((t) => fold(`${i.name} ${i.description}`).includes(t)))
          .sort((a, b) => {
            const primary =
              q.sort === "name_asc"
                ? fold(a.name).localeCompare(fold(b.name))
                : q.sort === "created_desc"
                  ? b.createdAt - a.createdAt
                  : b.updatedAt - a.updatedAt;
            return primary || b.id - a.id;
          })
          .map((i) => ({ ...i }));
      }
      case "get_investigation":
        return { ...find(args.id) };
      case "create_investigation": {
        const t = Date.now();
        const created: Investigation = {
          id: nextId++,
          name: validName(args.name),
          description: validDescription(args.description),
          status: "active",
          createdAt: t,
          updatedAt: t,
        };
        investigations.push(created);
        return { ...created };
      }
      case "rename_investigation": {
        const i = find(args.id);
        editable(i);
        i.name = validName(args.name);
        return touch(i);
      }
      case "update_investigation_description": {
        const i = find(args.id);
        editable(i);
        i.description = validDescription(args.description);
        return touch(i);
      }
      case "set_investigation_status": {
        const i = find(args.id);
        if (args.status !== "active" && args.status !== "archived") fail("validation", "Statut inconnu");
        if (i.status === args.status) fail("invalid_transition", "Transition de statut interdite");
        i.status = args.status as Investigation["status"];
        return touch(i);
      }
      case "delete_investigation": {
        const index = investigations.indexOf(find(args.id));
        investigations.splice(index, 1);
        return null;
      }
      default:
        return fail("unavailable", `Commande non disponible en mode démonstration : ${command}`);
    }
  })();
  return result as T;
}
