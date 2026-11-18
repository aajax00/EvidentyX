// Accès typé aux commandes exposées par l'hôte Tauri.
// Toute logique métier reste côté Rust : ce module ne fait que relayer.

import { invoke } from "@tauri-apps/api/core";

/** Miroir de `EngineInfo` (apps/desktop/src-tauri/src/lib.rs). */
export interface EngineInfo {
  engineVersion: string;
  appVersion: string;
  platform: string;
}

export function getEngineInfo(): Promise<EngineInfo> {
  return invoke<EngineInfo>("engine_info");
}

/** Vrai lorsque l'interface tourne dans la fenêtre Tauri (et non dans un navigateur seul). */
export function isTauri(): boolean {
  return "__TAURI_INTERNALS__" in window;
}
