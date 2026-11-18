//! Hôte de bureau Tauri d'EVIDENTYX.
//!
//! Ce crate se limite à exposer le moteur à l'interface via des commandes
//! typées. Aucune règle métier ne doit y vivre : elle appartient aux crates
//! du workspace (`evidentyx-core`, etc.).

use serde::Serialize;

/// Informations de version affichées par l'interface.
///
/// Miroir TypeScript : `EngineInfo` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineInfo {
    pub engine_version: &'static str,
    pub app_version: &'static str,
    pub platform: &'static str,
}

impl EngineInfo {
    pub fn current() -> Self {
        Self {
            engine_version: evidentyx_core::ENGINE_VERSION,
            app_version: env!("CARGO_PKG_VERSION"),
            platform: std::env::consts::OS,
        }
    }
}

#[tauri::command]
fn engine_info() -> EngineInfo {
    EngineInfo::current()
}

/// Point d'entrée de l'application de bureau.
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![engine_info])
        .run(tauri::generate_context!())
        .expect("échec du démarrage de l'application EVIDENTYX");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_info_reprend_la_version_du_moteur() {
        let info = EngineInfo::current();
        assert_eq!(info.engine_version, evidentyx_core::ENGINE_VERSION);
        assert!(!info.platform.is_empty());
    }

    #[test]
    fn engine_info_serialise_en_camel_case() {
        let json = serde_json::to_value(EngineInfo::current()).unwrap();
        for key in ["engineVersion", "appVersion", "platform"] {
            assert!(json.get(key).is_some(), "clé manquante : {key}");
        }
    }
}
