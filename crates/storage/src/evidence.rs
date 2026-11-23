//! Persistance des preuves (références aux fichiers sources).

use crate::error::{Result, StorageError};
use crate::Storage;
use evidentyx_core::{
    Evidence, EvidenceId, IntegrityStatus, InvestigationId, Sha256Digest, Timestamp,
};
use rusqlite::{params, OptionalExtension, Row};

/// Données d'une preuve à enregistrer, issues de la lecture du fichier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEvidence {
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub sha256: Sha256Digest,
    pub file_modified_at: Option<Timestamp>,
}

const COLUMNS: &str = "id, investigation_id, file_name, path, size_bytes, sha256, \
                       file_modified_at, added_at, integrity, verified_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<std::result::Result<Evidence, StorageError>> {
    let id: i64 = row.get(0)?;
    let investigation_id: i64 = row.get(1)?;
    let file_name: String = row.get(2)?;
    let path: String = row.get(3)?;
    let size_bytes: i64 = row.get(4)?;
    let sha256: String = row.get(5)?;
    let file_modified_at: Option<i64> = row.get(6)?;
    let added_at: i64 = row.get(7)?;
    let integrity: String = row.get(8)?;
    let verified_at: i64 = row.get(9)?;

    let corrupt = |detail: String| StorageError::CorruptRow {
        table: "evidence",
        id,
        detail,
    };
    Ok((|| {
        Ok(Evidence {
            id: EvidenceId::new(id),
            investigation_id: InvestigationId::new(investigation_id),
            file_name,
            path,
            size_bytes: u64::try_from(size_bytes)
                .map_err(|_| corrupt(format!("taille négative : {size_bytes}")))?,
            sha256: Sha256Digest::parse(&sha256).map_err(|e| corrupt(e.to_string()))?,
            file_modified_at: file_modified_at.map(Timestamp::from_unix_millis),
            added_at: Timestamp::from_unix_millis(added_at),
            integrity: integrity
                .parse::<IntegrityStatus>()
                .map_err(|e| corrupt(e.to_string()))?,
            verified_at: Timestamp::from_unix_millis(verified_at),
        })
    })())
}

impl Storage {
    /// Met à jour la date de modification d'une enquête (ajout ou retrait de preuve).
    fn touch_investigation(&self, id: InvestigationId, previous: Timestamp) -> Result<()> {
        let now = self.clock.now().max(previous);
        self.conn.execute(
            "UPDATE investigations SET updated_at = ?1 WHERE id = ?2",
            params![now.unix_millis(), id.get()],
        )?;
        Ok(())
    }

    /// Enregistre une preuve dans une enquête active. Un contenu déjà présent
    /// (même empreinte) est refusé avec `DuplicateEvidence`.
    pub fn add_evidence(
        &mut self,
        investigation_id: InvestigationId,
        new: &NewEvidence,
    ) -> Result<Evidence> {
        let investigation = self.investigation(investigation_id)?;
        investigation.ensure_editable()?;

        let existing: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT id, file_name FROM evidence WHERE investigation_id = ?1 AND sha256 = ?2",
                params![investigation_id.get(), new.sha256.as_str()],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((id, file_name)) = existing {
            return Err(StorageError::DuplicateEvidence {
                existing: EvidenceId::new(id),
                file_name,
            });
        }

        let now = self.clock.now();
        let size = i64::try_from(new.size_bytes).map_err(|_| StorageError::CorruptRow {
            table: "evidence",
            id: 0,
            detail: format!("taille hors limites : {}", new.size_bytes),
        })?;
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO evidence (investigation_id, file_name, path, size_bytes, sha256,
                                   file_modified_at, added_at, integrity, verified_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?7)",
            params![
                investigation_id.get(),
                new.file_name,
                new.path,
                size,
                new.sha256.as_str(),
                new.file_modified_at.map(Timestamp::unix_millis),
                now.unix_millis(),
                IntegrityStatus::Intact.as_str(),
            ],
        )?;
        let id = EvidenceId::new(tx.last_insert_rowid());
        tx.commit()?;
        self.touch_investigation(investigation_id, investigation.updated_at)?;
        self.evidence(id)
    }

    /// Renvoie la preuve demandée, ou `StorageError::EvidenceNotFound`.
    pub fn evidence(&self, id: EvidenceId) -> Result<Evidence> {
        self.conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM evidence WHERE id = ?1"),
                [id.get()],
                from_row,
            )
            .optional()?
            .ok_or(StorageError::EvidenceNotFound(id))?
    }

    /// Preuves d'une enquête, dans l'ordre d'enregistrement.
    pub fn list_evidence(&self, investigation_id: InvestigationId) -> Result<Vec<Evidence>> {
        self.investigation(investigation_id)?;
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM evidence WHERE investigation_id = ?1 ORDER BY added_at, id"
        ))?;
        let rows = stmt.query_map([investigation_id.get()], from_row)?;
        rows.map(|r| r?).collect()
    }

    /// Enregistre le résultat d'une vérification d'intégrité (permise même si
    /// l'enquête est archivée : la vérification ne modifie pas la preuve).
    pub fn record_integrity(
        &mut self,
        id: EvidenceId,
        status: IntegrityStatus,
    ) -> Result<Evidence> {
        let current = self.evidence(id)?;
        let now = self.clock.now().max(current.verified_at);
        self.conn.execute(
            "UPDATE evidence SET integrity = ?1, verified_at = ?2 WHERE id = ?3",
            params![status.as_str(), now.unix_millis(), id.get()],
        )?;
        self.evidence(id)
    }

    /// Retire une preuve d'une enquête active. Seule la référence est
    /// supprimée : le fichier source n'est jamais touché.
    pub fn remove_evidence(&mut self, id: EvidenceId) -> Result<()> {
        let evidence = self.evidence(id)?;
        let investigation = self.investigation(evidence.investigation_id)?;
        investigation.ensure_editable()?;
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM events WHERE evidence_id = ?1", [id.get()])?;
        tx.execute("DELETE FROM evidence WHERE id = ?1", [id.get()])?;
        tx.commit()?;
        self.touch_investigation(investigation.id, investigation.updated_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidentyx_core::{
        CoreError, InvestigationDescription, InvestigationName, InvestigationStatus,
    };

    fn db_with_investigation() -> (Storage, InvestigationId) {
        let mut db = Storage::open_in_memory().unwrap();
        let inv = db
            .create_investigation(
                &InvestigationName::parse("Test").unwrap(),
                &InvestigationDescription::default(),
            )
            .unwrap();
        (db, inv.id)
    }

    fn new(name: &str, byte: u8) -> NewEvidence {
        NewEvidence {
            file_name: name.into(),
            path: format!("/preuves/{name}"),
            size_bytes: 1234,
            sha256: Sha256Digest::from_bytes([byte; 32]),
            file_modified_at: Some(Timestamp::from_unix_millis(42)),
        }
    }

    #[test]
    fn ajout_lecture_et_liste() {
        let (mut db, inv) = db_with_investigation();
        let a = db.add_evidence(inv, &new("a.log", 1)).unwrap();
        let b = db.add_evidence(inv, &new("b.pcap", 2)).unwrap();
        assert_eq!(a.integrity, IntegrityStatus::Intact);
        assert_eq!(a.size_bytes, 1234);
        assert_eq!(a.file_modified_at, Some(Timestamp::from_unix_millis(42)));
        assert_eq!(db.evidence(a.id).unwrap(), a);
        let ids: Vec<_> = db
            .list_evidence(inv)
            .unwrap()
            .into_iter()
            .map(|e| e.id)
            .collect();
        assert_eq!(ids, [a.id, b.id]);
    }

    #[test]
    fn contenu_en_double_refuse() {
        let (mut db, inv) = db_with_investigation();
        let a = db.add_evidence(inv, &new("a.log", 1)).unwrap();
        assert!(matches!(
            db.add_evidence(inv, &new("copie.log", 1)),
            Err(StorageError::DuplicateEvidence { existing, .. }) if existing == a.id
        ));
    }

    #[test]
    fn enquete_archivee_lecture_seule_mais_verifiable() {
        let (mut db, inv) = db_with_investigation();
        let a = db.add_evidence(inv, &new("a.log", 1)).unwrap();
        db.set_investigation_status(inv, InvestigationStatus::Archived)
            .unwrap();
        assert!(matches!(
            db.add_evidence(inv, &new("b.log", 2)),
            Err(StorageError::Domain(CoreError::ArchivedReadOnly))
        ));
        assert!(matches!(
            db.remove_evidence(a.id),
            Err(StorageError::Domain(CoreError::ArchivedReadOnly))
        ));
        let checked = db
            .record_integrity(a.id, IntegrityStatus::Modified)
            .unwrap();
        assert_eq!(checked.integrity, IntegrityStatus::Modified);
    }

    #[test]
    fn retrait_et_suppression_en_cascade() {
        let (mut db, inv) = db_with_investigation();
        let a = db.add_evidence(inv, &new("a.log", 1)).unwrap();
        let b = db.add_evidence(inv, &new("b.log", 2)).unwrap();
        db.remove_evidence(a.id).unwrap();
        assert!(matches!(
            db.evidence(a.id),
            Err(StorageError::EvidenceNotFound(_))
        ));
        db.delete_investigation(inv).unwrap();
        assert!(matches!(
            db.evidence(b.id),
            Err(StorageError::EvidenceNotFound(_))
        ));
    }

    #[test]
    fn enquete_inexistante() {
        let (mut db, _) = db_with_investigation();
        let missing = InvestigationId::new(99);
        assert!(matches!(
            db.list_evidence(missing),
            Err(StorageError::NotFound(_))
        ));
        assert!(matches!(
            db.add_evidence(missing, &new("a.log", 1)),
            Err(StorageError::NotFound(_))
        ));
    }
}
