//! # evidentyx-storage
//!
//! Persistance locale d'EVIDENTYX dans une base SQLite unique : ouverture,
//! migrations versionnées du schéma et accès aux enquêtes.
//!
//! Les règles métier restent dans `evidentyx-core` : ce crate n'accepte que
//! des valeurs déjà validées (`InvestigationName`, etc.) et revalide ce qu'il
//! relit de la base, pour ne jamais propager une donnée corrompue.

mod clock;
mod error;
mod events;
mod evidence;
mod investigations;
mod migrations;
mod settings;

pub use clock::{Clock, SystemClock};
pub use error::{Result, StorageError};
pub use events::{
    EntityLink, EventFilter, EventPage, FlowKey, Group, GroupBy, InsertReport,
    InvestigationFigures, Overview, RecentEvidence, TimeQuality,
};
pub use evidence::NewEvidence;
pub use migrations::LATEST_SCHEMA_VERSION;

use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

/// Accès à la base de données d'EVIDENTYX.
///
/// Une instance possède une connexion SQLite ; elle est `Send` mais pas `Sync` :
/// l'hôte Tauri la protège par un verrou.
pub struct Storage {
    conn: Connection,
    clock: Box<dyn Clock>,
}

impl Storage {
    /// Ouvre (ou crée) la base au chemin indiqué et applique les migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::open_with_clock(path, Box::new(SystemClock))
    }

    pub fn open_with_clock(path: impl AsRef<Path>, clock: Box<dyn Clock>) -> Result<Self> {
        let conn = Connection::open(path)?;
        // WAL : lectures non bloquées par une écriture, meilleure résistance aux coupures.
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Self::init(conn, clock)
    }

    /// Base temporaire en mémoire (tests, démonstrations).
    pub fn open_in_memory() -> Result<Self> {
        Self::open_in_memory_with_clock(Box::new(SystemClock))
    }

    pub fn open_in_memory_with_clock(clock: Box<dyn Clock>) -> Result<Self> {
        Self::init(Connection::open_in_memory()?, clock)
    }

    fn init(mut conn: Connection, clock: Box<dyn Clock>) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        // Cache de 64 Mio : les suppressions et insertions massives touchent
        // beaucoup de pages d'index.
        conn.pragma_update(None, "cache_size", -65_536)?;
        // Le journal WAL reprend une taille raisonnable après une grosse opération.
        conn.pragma_update(None, "journal_size_limit", 67_108_864)?;
        migrations::migrate(&mut conn)?;
        Ok(Self { conn, clock })
    }

    /// Version actuelle du schéma de la base ouverte.
    pub fn schema_version(&self) -> Result<i64> {
        migrations::schema_version(&self.conn)
    }
}
