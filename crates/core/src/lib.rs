//! # evidentyx-core
//!
//! Modèle de domaine central d'EVIDENTYX.
//!
//! Ce crate ne dépend d'aucune bibliothèque externe et ne réalise aucune
//! entrée/sortie : il définit les types métier partagés par les autres crates
//! (stockage, analyse, interface Tauri) ainsi que leurs règles de validation.
//!
//! Convention : les identifiants de code sont en anglais (usage Rust), la
//! documentation et les messages destinés à l'analyste sont en français.

pub mod error;
pub mod event;
pub mod evidence;
pub mod investigation;
pub mod search;
pub mod settings;
pub mod text;

pub use error::CoreError;
pub use event::{
    Event, EventId, EventKind, EventTime, NewEvent, Outcome, SourceRef, TimeBasis, TimePrecision,
};
pub use evidence::{Evidence, EvidenceId, IntegrityStatus, Sha256Digest};
pub use investigation::{
    Investigation, InvestigationDescription, InvestigationId, InvestigationName,
    InvestigationStatus, Timestamp,
};
pub use search::{InvestigationQuery, InvestigationSort};
pub use settings::{AppSettings, StatusFilter};

/// Version du moteur, exposée à l'interface et inscrite dans les rapports.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
