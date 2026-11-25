//! Hôte de bureau Tauri d'EVIDENTYX.
//!
//! Ce crate se limite à exposer le moteur à l'interface via des commandes
//! typées. Aucune règle métier ne doit y vivre : elle appartient aux crates
//! du workspace (`evidentyx-core`, etc.).

mod analysis;
mod error;
mod evidence;
mod investigations;
mod overview;
mod settings;
mod state;
mod tasks;
mod timeline;

use serde::Serialize;
use state::AppState;
use tauri::Manager;

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
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Base locale dans le dossier de données de l'application
            // (ex. ~/.local/share/org.evidentyx.desktop sous Linux).
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            app.manage(AppState::open(dir.join("evidentyx.sqlite")));
            app.manage(tasks::Tasks::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            engine_info,
            state::storage_info,
            investigations::list_investigations,
            investigations::get_investigation,
            investigations::create_investigation,
            investigations::rename_investigation,
            investigations::update_investigation_description,
            investigations::set_investigation_status,
            investigations::delete_investigation,
            settings::get_settings,
            settings::update_settings,
            evidence::list_evidence,
            evidence::add_evidence,
            evidence::verify_evidence,
            evidence::remove_evidence,
            analysis::analyze_evidence,
            tasks::cancel_task,
            timeline::events_window,
            timeline::events_count,
            timeline::event_groups,
            timeline::entity_links,
            overview::investigation_overview,
        ])
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
