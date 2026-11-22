//! Migrations du schéma SQLite.
//!
//! La version du schéma est stockée dans `PRAGMA user_version`. Chaque
//! migration est appliquée dans sa propre transaction avec la mise à jour de
//! la version : une migration interrompue ne laisse jamais la base à moitié
//! migrée. Les migrations publiées ne doivent jamais être modifiées ; tout
//! changement passe par une nouvelle migration.

use crate::error::{Result, StorageError};
use rusqlite::Connection;

/// Migrations ordonnées : (version atteinte, script SQL).
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../migrations/0001_enquetes.sql")),
    (2, include_str!("../migrations/0002_reglages.sql")),
    (3, include_str!("../migrations/0003_preuves.sql")),
    (4, include_str!("../migrations/0004_evenements.sql")),
];

/// Version de schéma la plus récente connue de ce code.
pub const LATEST_SCHEMA_VERSION: i64 = MIGRATIONS[MIGRATIONS.len() - 1].0;

pub fn schema_version(conn: &Connection) -> Result<i64> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

/// Applique les migrations manquantes. Refuse une base plus récente que le code.
pub fn migrate(conn: &mut Connection) -> Result<()> {
    let current = schema_version(conn)?;
    if current > LATEST_SCHEMA_VERSION {
        return Err(StorageError::SchemaTooNew {
            found: current,
            supported: LATEST_SCHEMA_VERSION,
        });
    }
    for &(version, sql) in MIGRATIONS.iter().filter(|(v, _)| *v > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_strictement_croissantes() {
        assert!(MIGRATIONS.windows(2).all(|w| w[0].0 < w[1].0));
        assert_eq!(MIGRATIONS[0].0, 1);
    }

    #[test]
    fn migration_idempotente() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(schema_version(&conn).unwrap(), LATEST_SCHEMA_VERSION);
    }

    #[test]
    fn base_plus_recente_refusee() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "user_version", LATEST_SCHEMA_VERSION + 1)
            .unwrap();
        assert!(matches!(
            migrate(&mut conn),
            Err(StorageError::SchemaTooNew { .. })
        ));
    }
}
