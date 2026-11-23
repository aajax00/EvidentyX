// Accès typé aux commandes exposées par l'hôte Tauri.
// Toute logique métier reste côté Rust : ce module ne fait que relayer.

import { invoke } from "@tauri-apps/api/core";

/** Vrai lorsque l'interface tourne dans la fenêtre Tauri (et non dans un navigateur seul). */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}

/**
 * Vrai lorsque l'interface tourne dans un navigateur en développement : les
 * commandes sont alors servies par un moteur de démonstration en mémoire
 * (src/dev/mockBackend.ts), signalé à l'écran. Jamais actif en version publiée.
 */
export const isDemoMode = (): boolean => import.meta.env.DEV && !isTauri();

/** Miroir de `CommandError` (src-tauri/src/error.rs). */
export interface CommandError {
  code:
    | "validation"
    | "invalid_argument"
    | "invalid_transition"
    | "archived_read_only"
    | "not_found"
    | "schema_too_new"
    | "storage"
    | "duplicate"
    | "file_not_found"
    | "not_a_file"
    | "cancelled"
    | "io"
    | "evidence_changed"
    | "unsupported_format"
    | "parse"
    | "unavailable";
  message: string;
  field?: "name" | "description";
}

/** Convertit toute erreur reçue en `CommandError` exploitable par l'interface. */
export function toCommandError(error: unknown): CommandError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    return error as CommandError;
  }
  return { code: "unavailable", message: typeof error === "string" ? error : String(error) };
}

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    if (isTauri()) return await invoke<T>(command, args);
    if (import.meta.env.DEV) {
      const { mockInvoke } = await import("./dev/mockBackend");
      return await mockInvoke<T>(command, args);
    }
    throw { code: "unavailable", message: "Moteur non joignable hors de l'application de bureau." };
  } catch (error) {
    throw toCommandError(error);
  }
}

// ——— Moteur ———

/** Miroir de `EngineInfo` (src-tauri/src/lib.rs). */
export interface EngineInfo {
  engineVersion: string;
  appVersion: string;
  platform: string;
}

export function getEngineInfo(): Promise<EngineInfo> {
  return call<EngineInfo>("engine_info");
}

/** Miroir de `StorageInfo` (src-tauri/src/state.rs). */
export interface StorageInfo {
  path: string;
  schemaVersion: number | null;
  error: string | null;
}

export function getStorageInfo(): Promise<StorageInfo> {
  return call<StorageInfo>("storage_info");
}

// ——— Enquêtes ———

export type InvestigationStatus = "active" | "archived";
export type InvestigationSort = "updated_desc" | "created_desc" | "name_asc";

/** Miroir de `InvestigationDto` (src-tauri/src/investigations.rs). Dates en ms Unix UTC. */
export interface Investigation {
  id: number;
  name: string;
  description: string;
  status: InvestigationStatus;
  createdAt: number;
  updatedAt: number;
}

/** Miroir de `QueryDto` (src-tauri/src/investigations.rs). */
export interface InvestigationQuery {
  status?: InvestigationStatus;
  text?: string;
  sort?: InvestigationSort;
}

export function listInvestigations(query: InvestigationQuery = {}): Promise<Investigation[]> {
  return call("list_investigations", { query: { text: "", ...query } });
}

export function getInvestigation(id: number): Promise<Investigation> {
  return call("get_investigation", { id });
}

export function createInvestigation(name: string, description: string): Promise<Investigation> {
  return call("create_investigation", { name, description });
}

export function renameInvestigation(id: number, name: string): Promise<Investigation> {
  return call("rename_investigation", { id, name });
}

export function updateInvestigationDescription(id: number, description: string): Promise<Investigation> {
  return call("update_investigation_description", { id, description });
}

export function setInvestigationStatus(id: number, status: InvestigationStatus): Promise<Investigation> {
  return call("set_investigation_status", { id, status });
}

export function deleteInvestigation(id: number): Promise<void> {
  return call("delete_investigation", { id });
}

// ——— Réglages ———

export type StatusFilter = InvestigationStatus | "all";

/** Miroir de `SettingsDto` (src-tauri/src/settings.rs). */
export interface Settings {
  splashScreen: boolean;
  backgroundAnimation: boolean;
  sidebarCollapsed: boolean;
  defaultSort: InvestigationSort;
  defaultStatusFilter: StatusFilter;
}

export function getSettings(): Promise<Settings> {
  return call("get_settings");
}

export function updateSettings(settings: Settings): Promise<Settings> {
  return call("update_settings", { settings });
}

// ——— Preuves ———

export type IntegrityStatus = "intact" | "modified" | "missing";

/** Miroir de `EvidenceDto` (src-tauri/src/evidence.rs). Dates en ms Unix UTC. */
export interface Evidence {
  id: number;
  investigationId: number;
  fileName: string;
  path: string;
  sizeBytes: number;
  sha256: string;
  fileModifiedAt: number | null;
  addedAt: number;
  integrity: IntegrityStatus;
  verifiedAt: number;
  /** Événements extraits (0 tant que la preuve n'est pas analysée). */
  eventCount: number;
}

export function listEvidence(investigationId: number): Promise<Evidence[]> {
  return call("list_evidence", { investigationId });
}

/** Lit le fichier (empreinte SHA-256, en lecture seule) et l'enregistre dans l'enquête. */
export function addEvidence(investigationId: number, path: string): Promise<Evidence> {
  return call("add_evidence", { investigationId, path });
}

/** Recalcule l'empreinte et enregistre le résultat de la vérification. */
export function verifyEvidence(id: number): Promise<Evidence> {
  return call("verify_evidence", { id });
}

/** Retire la référence ; le fichier source n'est jamais supprimé. */
export function removeEvidence(id: number): Promise<void> {
  return call("remove_evidence", { id });
}

/**
 * Ouvre le sélecteur de fichiers natif. En mode démonstration (navigateur),
 * renvoie des chemins fictifs.
 */
export async function pickEvidenceFiles(): Promise<string[]> {
  if (isTauri()) {
    const { open } = await import("@tauri-apps/plugin-dialog");
    const selected = await open({ multiple: true, directory: false, title: "Ajouter des preuves" });
    if (selected === null) return [];
    return Array.isArray(selected) ? selected : [selected];
  }
  if (import.meta.env.DEV) {
    const { demoPickFiles } = await import("./dev/mockBackend");
    return demoPickFiles();
  }
  return [];
}

// ——— Analyse ———

/** Miroir de `AnalysisReportDto` (src-tauri/src/analysis.rs). */
export interface AnalysisReport {
  format: string;
  records: number;
  inserted: number;
  duplicates: number;
  errors: number;
  firstErrors: Array<{ line: number; byteOffset: number; message: string }>;
  /** Colonne retenue pour l'heure (CSV). */
  timeField: string | null;
  /** Structure reconnue (CSV : séparateur, colonnes, en-tête). */
  layout: string | null;
}

/** Lit la preuve (format reconnu automatiquement) et enregistre ses événements. */
export function analyzeEvidence(evidenceId: number): Promise<AnalysisReport> {
  return call("analyze_evidence", { evidenceId });
}
