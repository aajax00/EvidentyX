//! # evidentyx-parser
//!
//! Lecture des formats de preuve vers le modèle d'événement canonique
//! (`evidentyx-core::event`).
//!
//! Cadre commun à tous les formats :
//! - **détection** du format (extension et premiers octets) ;
//! - **lecture en flux** : mémoire bornée quelle que soit la taille du fichier ;
//! - **erreurs par enregistrement** : une ligne illisible est signalée
//!   (ligne, décalage, raison) sans interrompre la lecture ;
//! - **progression** et **annulation** ;
//! - chaque événement garde sa référence exacte à la source.
//!
//! Formats pris en charge : JSON Lines, tableau JSON, objets JSON concaténés.

mod fields;
mod json;
pub mod time;

pub use fields::{map_fields, Scalar};

use evidentyx_core::{EvidenceId, NewEvent};
use std::fmt;
use std::io::{self, Read};
use std::sync::atomic::AtomicBool;

/// Taille maximale d'un enregistrement (4 Mio) : au-delà, il est signalé et ignoré.
pub const MAX_RECORD_BYTES: usize = 4 * 1024 * 1024;
/// Nombre d'erreurs détaillées conservées dans le bilan (les suivantes sont comptées).
pub const MAX_REPORTED_ERRORS: usize = 50;

/// Format de preuve reconnu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Format {
    /// Un objet JSON par ligne (`.jsonl`, `.ndjson`).
    JsonLines,
    /// Un tableau JSON d'objets.
    JsonArray,
    /// Objets JSON successifs, éventuellement sur plusieurs lignes.
    JsonConcatenated,
}

impl Format {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JsonLines => "json_lines",
            Self::JsonArray => "json_array",
            Self::JsonConcatenated => "json_concatenated",
        }
    }

    pub const fn label_fr(self) -> &'static str {
        match self {
            Self::JsonLines => "JSON Lines",
            Self::JsonArray => "Tableau JSON",
            Self::JsonConcatenated => "Objets JSON",
        }
    }
}

/// Détecte le format d'après le nom du fichier et ses premiers octets.
/// Renvoie `None` si aucun format pris en charge ne correspond.
pub fn detect(file_name: &str, head: &[u8]) -> Option<Format> {
    json::detect(file_name, head)
}

/// Enregistrement illisible, signalé sans interrompre la lecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordError {
    /// Ligne de début (à partir de 1).
    pub line: u64,
    /// Décalage en octets du début de l'enregistrement.
    pub byte_offset: u64,
    pub message: String,
}

/// Bilan d'une lecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseReport {
    pub format: Format,
    /// Enregistrements rencontrés (hors lignes vides).
    pub records: u64,
    /// Événements produits.
    pub events: u64,
    /// Enregistrements en erreur.
    pub errors: u64,
    /// Premières erreurs, détaillées (au plus `MAX_REPORTED_ERRORS`).
    pub first_errors: Vec<RecordError>,
    /// Octets lus.
    pub bytes: u64,
}

impl ParseReport {
    fn new(format: Format) -> Self {
        Self {
            format,
            records: 0,
            events: 0,
            errors: 0,
            first_errors: Vec::new(),
            bytes: 0,
        }
    }

    fn error(&mut self, line: u64, byte_offset: u64, message: impl Into<String>) {
        self.errors += 1;
        if self.first_errors.len() < MAX_REPORTED_ERRORS {
            self.first_errors.push(RecordError {
                line,
                byte_offset,
                message: message.into(),
            });
        }
    }
}

/// Erreur qui interrompt la lecture.
#[derive(Debug)]
pub enum ParseError {
    Io(io::Error),
    /// Lecture annulée à la demande de l'utilisateur.
    Cancelled,
    /// Structure globale illisible (ex. tableau JSON non fermé) : la lecture s'arrête.
    Malformed {
        line: u64,
        byte_offset: u64,
        message: String,
    },
    /// Le destinataire des événements a refusé un événement (ex. erreur de base).
    Sink(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "lecture impossible : {e}"),
            Self::Cancelled => write!(f, "analyse annulée"),
            Self::Malformed { line, message, .. } => {
                write!(f, "structure illisible à la ligne {line} : {message}")
            }
            Self::Sink(e) => write!(f, "enregistrement interrompu : {e}"),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// Paramètres d'une lecture.
pub struct ParseContext<'a> {
    pub evidence_id: EvidenceId,
    /// Appelé avec le nombre d'octets lus depuis le début.
    pub on_progress: &'a mut dyn FnMut(u64),
    /// Consulté régulièrement ; `true` interrompt la lecture.
    pub cancel: &'a AtomicBool,
}

/// Lit une preuve et transmet chaque événement à `sink`, dans l'ordre du fichier.
pub fn parse(
    format: Format,
    reader: impl Read,
    ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<ParseReport, ParseError> {
    json::parse(format, reader, ctx, sink)
}
