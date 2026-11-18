//! Erreurs du domaine central.

use std::fmt;

/// Erreur de validation ou de règle métier du domaine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// Le nom d'enquête est vide après suppression des espaces.
    EmptyName,
    /// Le nom d'enquête dépasse la longueur maximale autorisée.
    NameTooLong { max: usize, actual: usize },
    /// La description dépasse la longueur maximale autorisée.
    DescriptionTooLong { max: usize, actual: usize },
    /// Le texte contient des caractères de contrôle interdits.
    ControlCharacters { field: &'static str },
    /// Valeur de statut inconnue.
    UnknownStatus(String),
    /// Transition de statut non autorisée.
    InvalidTransition {
        from: &'static str,
        to: &'static str,
    },
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "le nom de l'enquête ne peut pas être vide"),
            Self::NameTooLong { max, actual } => write!(
                f,
                "le nom de l'enquête est trop long ({actual} caractères, maximum {max})"
            ),
            Self::DescriptionTooLong { max, actual } => write!(
                f,
                "la description est trop longue ({actual} caractères, maximum {max})"
            ),
            Self::ControlCharacters { field } => write!(
                f,
                "le champ « {field} » contient des caractères de contrôle interdits"
            ),
            Self::UnknownStatus(value) => write!(f, "statut d'enquête inconnu : « {value} »"),
            Self::InvalidTransition { from, to } => write!(
                f,
                "transition de statut interdite : « {from} » vers « {to} »"
            ),
        }
    }
}

impl std::error::Error for CoreError {}
