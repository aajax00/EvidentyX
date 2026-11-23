//! Persistance des enquêtes.

use crate::error::{Result, StorageError};
use crate::Storage;
use evidentyx_core::{
    Investigation, InvestigationDescription, InvestigationId, InvestigationName,
    InvestigationStatus, Timestamp,
};
use rusqlite::{params, OptionalExtension, Row};

const COLUMNS: &str = "id, name, description, status, created_at, updated_at";

/// Lit une ligne et revalide chaque champ avec les règles du domaine.
fn from_row(row: &Row<'_>) -> rusqlite::Result<std::result::Result<Investigation, StorageError>> {
    let id: i64 = row.get(0)?;
    let name: String = row.get(1)?;
    let description: String = row.get(2)?;
    let status: String = row.get(3)?;
    let created_at: i64 = row.get(4)?;
    let updated_at: i64 = row.get(5)?;

    let corrupt = |detail: String| StorageError::CorruptRow {
        table: "investigations",
        id,
        detail,
    };
    Ok((|| {
        Ok(Investigation {
            id: InvestigationId::new(id),
            name: InvestigationName::parse(&name).map_err(|e| corrupt(e.to_string()))?,
            description: InvestigationDescription::parse(&description)
                .map_err(|e| corrupt(e.to_string()))?,
            status: status
                .parse()
                .map_err(|e: evidentyx_core::CoreError| corrupt(e.to_string()))?,
            created_at: Timestamp::from_unix_millis(created_at),
            updated_at: Timestamp::from_unix_millis(updated_at),
        })
    })())
}

impl Storage {
    /// Horodatage d'une modification : jamais antérieur au précédent, même si
    /// l'horloge système recule (changement d'heure manuel, synchronisation).
    fn modification_time(&self, previous: Timestamp) -> Timestamp {
        self.clock.now().max(previous)
    }

    /// Crée une enquête active.
    pub fn create_investigation(
        &mut self,
        name: &InvestigationName,
        description: &InvestigationDescription,
    ) -> Result<Investigation> {
        let now = self.clock.now();
        self.conn.execute(
            "INSERT INTO investigations (name, description, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?4)",
            params![
                name.as_str(),
                description.as_str(),
                InvestigationStatus::Active.as_str(),
                now.unix_millis()
            ],
        )?;
        self.investigation(InvestigationId::new(self.conn.last_insert_rowid()))
    }

    /// Renvoie l'enquête demandée, ou `StorageError::NotFound`.
    pub fn investigation(&self, id: InvestigationId) -> Result<Investigation> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM investigations WHERE id = ?1"),
                [id.get()],
                from_row,
            )
            .optional()?
            .ok_or(StorageError::NotFound(id))?
    }

    /// Liste les enquêtes, éventuellement filtrées par statut, de la plus
    /// récemment modifiée à la plus ancienne (ordre déterministe : à date
    /// égale, l'identifiant le plus grand d'abord).
    pub fn list_investigations(
        &self,
        status: Option<InvestigationStatus>,
    ) -> Result<Vec<Investigation>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM investigations
             WHERE ?1 IS NULL OR status = ?1
             ORDER BY updated_at DESC, id DESC"
        ))?;
        let rows = stmt.query_map([status.map(InvestigationStatus::as_str)], from_row)?;
        rows.map(|r| r?).collect()
    }

    /// Renomme une enquête active.
    pub fn rename_investigation(
        &mut self,
        id: InvestigationId,
        name: &InvestigationName,
    ) -> Result<Investigation> {
        let current = self.investigation(id)?;
        current.ensure_editable()?;
        let now = self.modification_time(current.updated_at);
        self.conn.execute(
            "UPDATE investigations SET name = ?1, updated_at = ?2 WHERE id = ?3",
            params![name.as_str(), now.unix_millis(), id.get()],
        )?;
        self.investigation(id)
    }

    /// Remplace la description d'une enquête active.
    pub fn update_investigation_description(
        &mut self,
        id: InvestigationId,
        description: &InvestigationDescription,
    ) -> Result<Investigation> {
        let current = self.investigation(id)?;
        current.ensure_editable()?;
        let now = self.modification_time(current.updated_at);
        self.conn.execute(
            "UPDATE investigations SET description = ?1, updated_at = ?2 WHERE id = ?3",
            params![description.as_str(), now.unix_millis(), id.get()],
        )?;
        self.investigation(id)
    }

    /// Archive ou réactive une enquête, selon les transitions autorisées par le domaine.
    pub fn set_investigation_status(
        &mut self,
        id: InvestigationId,
        target: InvestigationStatus,
    ) -> Result<Investigation> {
        let current = self.investigation(id)?;
        let status = current.status.transition_to(target)?;
        let now = self.modification_time(current.updated_at);
        self.conn.execute(
            "UPDATE investigations SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.unix_millis(), id.get()],
        )?;
        self.investigation(id)
    }

    /// Supprime définitivement une enquête.
    pub fn delete_investigation(&mut self, id: InvestigationId) -> Result<()> {
        let tx = self.conn.transaction()?;
        // Événements d'abord : la cascade ligne à ligne serait plus lente.
        tx.execute("DELETE FROM events WHERE investigation_id = ?1", [id.get()])?;
        let deleted = tx.execute("DELETE FROM investigations WHERE id = ?1", [id.get()])?;
        if deleted == 0 {
            return Err(StorageError::NotFound(id));
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Clock;
    use evidentyx_core::CoreError;
    use std::sync::atomic::{AtomicI64, Ordering};
    use std::sync::Arc;

    /// Horloge réglable à la main, partagée entre le test et le stockage.
    #[derive(Clone, Default)]
    struct ManualClock(Arc<AtomicI64>);

    impl ManualClock {
        fn set(&self, millis: i64) {
            self.0.store(millis, Ordering::SeqCst);
        }
    }

    impl Clock for ManualClock {
        fn now(&self) -> Timestamp {
            Timestamp::from_unix_millis(self.0.load(Ordering::SeqCst))
        }
    }

    fn storage() -> (Storage, ManualClock) {
        let clock = ManualClock::default();
        clock.set(1_000);
        let storage = Storage::open_in_memory_with_clock(Box::new(clock.clone())).unwrap();
        (storage, clock)
    }

    fn name(s: &str) -> InvestigationName {
        InvestigationName::parse(s).unwrap()
    }

    fn desc(s: &str) -> InvestigationDescription {
        InvestigationDescription::parse(s).unwrap()
    }

    #[test]
    fn creation_puis_relecture() {
        let (mut db, _) = storage();
        let created = db
            .create_investigation(&name("Intrusion web"), &desc("Serveur nginx"))
            .unwrap();
        assert_eq!(created.name.as_str(), "Intrusion web");
        assert_eq!(created.description.as_str(), "Serveur nginx");
        assert_eq!(created.status, InvestigationStatus::Active);
        assert_eq!(created.created_at, Timestamp::from_unix_millis(1_000));
        assert_eq!(created.updated_at, created.created_at);
        assert_eq!(db.investigation(created.id).unwrap(), created);
    }

    #[test]
    fn enquete_inexistante() {
        let (db, _) = storage();
        let id = InvestigationId::new(42);
        assert!(matches!(db.investigation(id), Err(StorageError::NotFound(i)) if i == id));
    }

    #[test]
    fn liste_triee_par_derniere_modification_et_filtrable() {
        let (mut db, clock) = storage();
        let a = db.create_investigation(&name("A"), &desc("")).unwrap();
        clock.set(2_000);
        let b = db.create_investigation(&name("B"), &desc("")).unwrap();
        clock.set(3_000);
        db.rename_investigation(a.id, &name("A bis")).unwrap();
        clock.set(4_000);
        let c = db.create_investigation(&name("C"), &desc("")).unwrap();
        db.set_investigation_status(b.id, InvestigationStatus::Archived)
            .unwrap();

        let ids = |list: Vec<Investigation>| list.into_iter().map(|i| i.id).collect::<Vec<_>>();
        // b et c modifiés à 4000 : l'identifiant le plus grand départage.
        assert_eq!(
            ids(db.list_investigations(None).unwrap()),
            [c.id, b.id, a.id]
        );
        assert_eq!(
            ids(db
                .list_investigations(Some(InvestigationStatus::Active))
                .unwrap()),
            [c.id, a.id]
        );
        assert_eq!(
            ids(db
                .list_investigations(Some(InvestigationStatus::Archived))
                .unwrap()),
            [b.id]
        );
    }

    #[test]
    fn renommage_et_description_mettent_a_jour_la_date() {
        let (mut db, clock) = storage();
        let inv = db.create_investigation(&name("Avant"), &desc("")).unwrap();
        clock.set(5_000);
        let renamed = db.rename_investigation(inv.id, &name("Après")).unwrap();
        assert_eq!(renamed.name.as_str(), "Après");
        assert_eq!(renamed.created_at, inv.created_at);
        assert_eq!(renamed.updated_at, Timestamp::from_unix_millis(5_000));

        clock.set(6_000);
        let described = db
            .update_investigation_description(inv.id, &desc("Ligne 1\nLigne 2"))
            .unwrap();
        assert_eq!(described.description.as_str(), "Ligne 1\nLigne 2");
        assert_eq!(described.updated_at, Timestamp::from_unix_millis(6_000));
    }

    #[test]
    fn date_de_modification_jamais_anterieure_si_l_horloge_recule() {
        let (mut db, clock) = storage();
        clock.set(10_000);
        let inv = db.create_investigation(&name("X"), &desc("")).unwrap();
        clock.set(3_000);
        let renamed = db.rename_investigation(inv.id, &name("Y")).unwrap();
        assert_eq!(renamed.updated_at, Timestamp::from_unix_millis(10_000));
    }

    #[test]
    fn archivage_lecture_seule_puis_reactivation() {
        let (mut db, _) = storage();
        let inv = db.create_investigation(&name("X"), &desc("")).unwrap();

        let archived = db
            .set_investigation_status(inv.id, InvestigationStatus::Archived)
            .unwrap();
        assert_eq!(archived.status, InvestigationStatus::Archived);

        assert!(matches!(
            db.rename_investigation(inv.id, &name("Y")),
            Err(StorageError::Domain(CoreError::ArchivedReadOnly))
        ));
        assert!(matches!(
            db.update_investigation_description(inv.id, &desc("d")),
            Err(StorageError::Domain(CoreError::ArchivedReadOnly))
        ));
        assert!(matches!(
            db.set_investigation_status(inv.id, InvestigationStatus::Archived),
            Err(StorageError::Domain(CoreError::InvalidTransition { .. }))
        ));

        db.set_investigation_status(inv.id, InvestigationStatus::Active)
            .unwrap();
        assert_eq!(
            db.rename_investigation(inv.id, &name("Y"))
                .unwrap()
                .name
                .as_str(),
            "Y"
        );
    }

    #[test]
    fn suppression_et_identifiant_jamais_reutilise() {
        let (mut db, _) = storage();
        let a = db.create_investigation(&name("A"), &desc("")).unwrap();
        db.delete_investigation(a.id).unwrap();
        assert!(matches!(
            db.investigation(a.id),
            Err(StorageError::NotFound(_))
        ));
        assert!(matches!(
            db.delete_investigation(a.id),
            Err(StorageError::NotFound(_))
        ));
        let b = db.create_investigation(&name("B"), &desc("")).unwrap();
        assert!(b.id > a.id);
    }

    #[test]
    fn ligne_corrompue_detectee_a_la_relecture() {
        let (mut db, _) = storage();
        let inv = db.create_investigation(&name("X"), &desc("")).unwrap();
        // Contourne la validation : caractère de contrôle interdit dans un nom.
        db.conn
            .execute(
                "UPDATE investigations SET name = 'a' || char(7) || 'b' WHERE id = ?1",
                [inv.id.get()],
            )
            .unwrap();
        assert!(matches!(
            db.investigation(inv.id),
            Err(StorageError::CorruptRow {
                table: "investigations",
                ..
            })
        ));
        assert!(db.list_investigations(None).is_err());
    }

    #[test]
    fn contraintes_sql_en_dernier_rempart() {
        let (db, _) = storage();
        let insert = |status: &str, name: &str| {
            db.conn.execute(
                "INSERT INTO investigations (name, status, created_at, updated_at)
                 VALUES (?1, ?2, 0, 0)",
                params![name, status],
            )
        };
        assert!(insert("inconnu", "X").is_err());
        assert!(insert("active", "   ").is_err());
        assert!(insert("active", "X").is_ok());
    }

    #[test]
    fn persistance_dans_un_fichier() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("evidentyx.sqlite");
        let id = {
            let mut db = Storage::open(&path).unwrap();
            db.create_investigation(&name("Persistante"), &desc(""))
                .unwrap()
                .id
        };
        let db = Storage::open(&path).unwrap();
        assert_eq!(db.schema_version().unwrap(), crate::LATEST_SCHEMA_VERSION);
        assert_eq!(db.investigation(id).unwrap().name.as_str(), "Persistante");
    }

    #[test]
    fn messages_d_erreur_en_francais() {
        assert_eq!(
            StorageError::NotFound(InvestigationId::new(7)).to_string(),
            "enquête introuvable (identifiant 7)"
        );
    }
}
