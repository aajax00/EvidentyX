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
pub mod investigation;

pub use error::CoreError;
pub use investigation::{
    Investigation, InvestigationDescription, InvestigationId, InvestigationName,
    InvestigationStatus, Timestamp,
};

/// Version du moteur, exposée à l'interface et inscrite dans les rapports.
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");
