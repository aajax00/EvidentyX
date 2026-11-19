//! État partagé de l'application : accès au stockage.

use crate::error::CommandError;
use evidentyx_storage::Storage;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Mutex;

/// Stockage ouvert au démarrage. Si l'ouverture échoue (base d'une version
/// plus récente, disque inaccessible…), l'application démarre quand même et
/// chaque commande renvoie l'erreur au lieu de planter.
pub struct AppState {
    storage: Result<Mutex<Storage>, CommandError>,
    path: PathBuf,
}

impl AppState {
    pub fn open(path: PathBuf) -> Self {
        let storage = Storage::open(&path)
            .map(Mutex::new)
            .map_err(CommandError::from);
        Self { storage, path }
    }

    pub fn with_storage<T>(
        &self,
        f: impl FnOnce(&mut Storage) -> Result<T, CommandError>,
    ) -> Result<T, CommandError> {
        let mutex = self.storage.as_ref().map_err(Clone::clone)?;
        // Un verrou empoisonné signifie qu'une commande a paniqué en cours
        // d'opération : la connexion SQLite reste cohérente (transactions),
        // on peut donc continuer à l'utiliser.
        let mut guard = mutex.lock().unwrap_or_else(|p| p.into_inner());
        f(&mut guard)
    }

    pub fn info(&self) -> StorageInfo {
        let (schema_version, error) = match self.with_storage(|s| Ok(s.schema_version()?)) {
            Ok(v) => (Some(v), None),
            Err(e) => (None, Some(e.message)),
        };
        StorageInfo {
            path: self.path.display().to_string(),
            schema_version,
            error,
        }
    }
}

/// Miroir TypeScript : `StorageInfo` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    pub path: String,
    pub schema_version: Option<i64>,
    pub error: Option<String>,
}

#[tauri::command]
pub fn storage_info(state: tauri::State<'_, AppState>) -> StorageInfo {
    state.info()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_ouverte_dans_un_fichier() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState::open(dir.path().join("evidentyx.sqlite"));
        let info = state.info();
        assert_eq!(
            info.schema_version,
            Some(evidentyx_storage::LATEST_SCHEMA_VERSION)
        );
        assert!(info.error.is_none());
    }

    #[test]
    fn base_inaccessible_signalee_sans_plantage() {
        let dir = tempfile::tempdir().unwrap();
        // Un dossier ne peut pas être ouvert comme base SQLite.
        let state = AppState::open(dir.path().to_path_buf());
        let info = state.info();
        assert!(info.schema_version.is_none());
        assert!(info.error.is_some());
        assert_eq!(state.with_storage(|_| Ok(())).unwrap_err().code, "storage");
    }
}
