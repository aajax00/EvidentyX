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
