//! Erreurs du stockage.

use evidentyx_core::{CoreError, InvestigationId};
use std::fmt;

/// Erreur renvoyée par le stockage.
#[derive(Debug)]
pub enum StorageError {
    /// Erreur SQLite (ouverture, requête, contrainte…).
    Sqlite(rusqlite::Error),
    /// Règle métier refusée (validation, transition, lecture seule).
    Domain(CoreError),
    /// Aucune enquête ne porte cet identifiant.
    NotFound(InvestigationId),
    /// La base a été créée par une version plus récente d'EVIDENTYX.
    SchemaTooNew { found: i64, supported: i64 },
    /// Une ligne enregistrée ne respecte pas les règles du domaine.
    CorruptRow {
        table: &'static str,
        id: i64,
        detail: String,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(e) => write!(f, "erreur de la base de données : {e}"),
            Self::Domain(e) => e.fmt(f),
            Self::NotFound(id) => write!(f, "enquête introuvable (identifiant {id})"),
            Self::SchemaTooNew { found, supported } => write!(
                f,
                "la base de données a été créée par une version plus récente d'EVIDENTYX \
                 (schéma {found}, cette version gère jusqu'au schéma {supported}) ; \
                 mettez l'application à jour"
            ),
            Self::CorruptRow { table, id, detail } => write!(
                f,
                "donnée invalide dans la table « {table} » (ligne {id}) : {detail}"
            ),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(e) => Some(e),
            Self::Domain(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e)
    }
}

impl From<CoreError> for StorageError {
    fn from(e: CoreError) -> Self {
        Self::Domain(e)
    }
}

pub type Result<T> = std::result::Result<T, StorageError>;
