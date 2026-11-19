// Moteur de démonstration — développement dans un navigateur uniquement.
//
// Permet de travailler sur l'interface avec `npm run dev` sans l'hôte Tauri.
// Données en mémoire, perdues au rechargement, clairement signalées à l'écran.
// Les règles reproduites ici sont volontairement minimales : la référence est
// le moteur Rust (evidentyx-core, evidentyx-storage). Exclu de la version publiée.

import type { Investigation, InvestigationQuery, Settings, StorageInfo } from "../api";

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
};

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

export async function mockInvoke<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  await new Promise((r) => setTimeout(r, 80));
  const result = ((): unknown => {
    switch (command) {
      case "get_settings":
        return { ...settings };
      case "update_settings":
        settings = { ...(args.settings as Settings) };
        return { ...settings };
      case "storage_info":
        return { path: "mémoire du navigateur (démonstration)", schemaVersion: 2, error: null } satisfies StorageInfo;
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
