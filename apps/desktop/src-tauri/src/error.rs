//! Erreurs renvoyées à l'interface.

use evidentyx_core::CoreError;
use evidentyx_evidence::EvidenceError;
use evidentyx_storage::StorageError;
use serde::Serialize;

/// Erreur sérialisée vers l'interface.
///
/// Miroir TypeScript : `CommandError` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandError {
    /// Code stable, exploitable par l'interface.
    pub code: &'static str,
    /// Message en français destiné à l'analyste.
    pub message: String,
    /// Champ de formulaire concerné, le cas échéant (`name`, `description`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<&'static str>,
}

impl CommandError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            field: None,
        }
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::new("invalid_argument", message)
    }
}

impl From<CoreError> for CommandError {
    fn from(e: CoreError) -> Self {
        let (code, field) = match &e {
            CoreError::EmptyName | CoreError::NameTooLong { .. } => ("validation", Some("name")),
            CoreError::DescriptionTooLong { .. } => ("validation", Some("description")),
            CoreError::ControlCharacters { field } => (
                "validation",
                Some(if *field == "nom" {
                    "name"
                } else {
                    "description"
                }),
            ),
            CoreError::UnknownStatus(_)
            | CoreError::InvalidDigest
            | CoreError::UnknownIntegrityStatus(_)
            | CoreError::InvalidEvent(_)
            | CoreError::UnknownEventValue { .. } => ("validation", None),
            CoreError::InvalidTransition { .. } => ("invalid_transition", None),
            CoreError::ArchivedReadOnly => ("archived_read_only", None),
        };
        Self {
            code,
            message: capitalize(&e.to_string()),
            field,
        }
    }
}

impl From<StorageError> for CommandError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::Domain(core) => core.into(),
            StorageError::NotFound(_) | StorageError::EvidenceNotFound(_) => {
                Self::new("not_found", capitalize(&e.to_string()))
            }
            StorageError::DuplicateEvidence { .. } => {
                Self::new("duplicate", capitalize(&e.to_string()))
            }
            StorageError::SchemaTooNew { .. } => {
                Self::new("schema_too_new", capitalize(&e.to_string()))
            }
            StorageError::Sqlite(_) | StorageError::CorruptRow { .. } => {
                Self::new("storage", capitalize(&e.to_string()))
            }
        }
    }
}

impl From<EvidenceError> for CommandError {
    fn from(e: EvidenceError) -> Self {
        let code = match e {
            EvidenceError::NotFound(_) => "file_not_found",
            EvidenceError::NotAFile(_) => "not_a_file",
            EvidenceError::Cancelled => "cancelled",
            EvidenceError::NonUtf8Path(_) | EvidenceError::Io { .. } => "io",
        };
        Self::new(code, capitalize(&e.to_string()))
    }
}

/// Les messages du domaine commencent par une minuscule (convention Rust) ;
/// l'interface les affiche comme des phrases.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn erreur_de_validation_associee_au_champ() {
        let e = CommandError::from(CoreError::EmptyName);
        assert_eq!(e.code, "validation");
        assert_eq!(e.field, Some("name"));
        assert_eq!(e.message, "Le nom de l'enquête ne peut pas être vide");

        let e = CommandError::from(CoreError::ControlCharacters {
            field: "description",
        });
        assert_eq!(e.field, Some("description"));
    }

    #[test]
    fn erreurs_du_stockage() {
        let e = CommandError::from(StorageError::NotFound(
            evidentyx_core::InvestigationId::new(3),
        ));
        assert_eq!(e.code, "not_found");
        let e = CommandError::from(StorageError::Domain(CoreError::ArchivedReadOnly));
        assert_eq!(e.code, "archived_read_only");
    }
}
