//! Commandes des preuves : ajout (empreinte SHA-256), liste, vérification, retrait.
//!
//! La lecture des fichiers se fait hors du fil d'exécution de l'interface
//! (`spawn_blocking`) et sans tenir le verrou de la base, pour que
//! l'application reste réactive pendant le calcul d'empreinte d'un gros fichier.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{Evidence, EvidenceId, IntegrityStatus, InvestigationId};
use evidentyx_evidence::{current_digest, fingerprint, EvidenceError, Fingerprint};
use evidentyx_storage::{NewEvidence, Storage};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

/// Miroir TypeScript : `Evidence` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceDto {
    pub id: i64,
    pub investigation_id: i64,
    pub file_name: String,
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    /// Millisecondes Unix UTC.
    pub file_modified_at: Option<i64>,
    pub added_at: i64,
    /// "intact", "modified" ou "missing".
    pub integrity: &'static str,
    pub verified_at: i64,
    /// Événements extraits de la preuve (0 tant qu'elle n'est pas analysée).
    pub event_count: u64,
}

impl From<Evidence> for EvidenceDto {
    fn from(e: Evidence) -> Self {
        Self {
            id: e.id.get(),
            investigation_id: e.investigation_id.get(),
            file_name: e.file_name,
            path: e.path,
            size_bytes: e.size_bytes,
            sha256: e.sha256.as_str().to_owned(),
            file_modified_at: e.file_modified_at.map(|t| t.unix_millis()),
            added_at: e.added_at.unix_millis(),
            integrity: e.integrity.as_str(),
            verified_at: e.verified_at.unix_millis(),
            event_count: 0,
        }
    }
}

type CommandResult<T> = Result<T, CommandError>;

fn new_evidence(f: Fingerprint) -> CommandResult<NewEvidence> {
    let path = f
        .path
        .to_str()
        .ok_or_else(|| CommandError::from(EvidenceError::NonUtf8Path(f.path.clone())))?
        .to_owned();
    Ok(NewEvidence {
        file_name: f.file_name,
        path,
        size_bytes: f.size_bytes,
        sha256: f.sha256,
        file_modified_at: f.modified_at,
    })
}

/// Enregistre une preuve dont l'empreinte vient d'être calculée.
fn record_new(
    storage: &mut Storage,
    investigation_id: i64,
    fp: Fingerprint,
) -> CommandResult<EvidenceDto> {
    let new = new_evidence(fp)?;
    Ok(storage
        .add_evidence(InvestigationId::new(investigation_id), &new)?
        .into())
}

/// Enregistre le résultat d'une vérification (`None` : fichier inaccessible).
fn record_verification(
    storage: &mut Storage,
    evidence: &Evidence,
    digest: Option<&evidentyx_core::Sha256Digest>,
) -> CommandResult<EvidenceDto> {
    let status = IntegrityStatus::compare(&evidence.sha256, digest);
    Ok(storage.record_integrity(evidence.id, status)?.into())
}

pub fn list(storage: &Storage, investigation_id: i64) -> CommandResult<Vec<EvidenceDto>> {
    let id = InvestigationId::new(investigation_id);
    let counts = storage.event_counts_by_evidence(id)?;
    Ok(storage
        .list_evidence(id)?
        .into_iter()
        .map(|e| {
            let event_count = counts.get(&e.id).copied().unwrap_or(0);
            EvidenceDto {
                event_count,
                ..e.into()
            }
        })
        .collect())
}

/// Calcule l'empreinte hors du fil de l'interface.
async fn fingerprint_in_background(path: PathBuf) -> CommandResult<Fingerprint> {
    tauri::async_runtime::spawn_blocking(move || {
        fingerprint(&path, |_, _| {}, &AtomicBool::new(false))
    })
    .await
    .map_err(|e| CommandError::new("io", format!("Lecture interrompue : {e}")))?
    .map_err(Into::into)
}

// ——— Commandes Tauri ———

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn list_evidence(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
) -> CommandResult<Vec<EvidenceDto>> {
    state.with_storage(|s| list(s, investigation_id))
}

#[tauri::command]
pub async fn add_evidence(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
    path: String,
) -> CommandResult<EvidenceDto> {
    // Vérification rapide avant une lecture potentiellement longue.
    state.with_storage(|s| {
        Ok(s.investigation(InvestigationId::new(investigation_id))?
            .ensure_editable()?)
    })?;
    let fp = fingerprint_in_background(PathBuf::from(path)).await?;
    state.with_storage(|s| record_new(s, investigation_id, fp))
}

#[tauri::command]
pub async fn verify_evidence(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> CommandResult<EvidenceDto> {
    let evidence = state.with_storage(|s| Ok(s.evidence(EvidenceId::new(id))?))?;
    let path = PathBuf::from(&evidence.path);
    let digest = tauri::async_runtime::spawn_blocking(move || {
        current_digest(&path, &AtomicBool::new(false))
    })
    .await
    .map_err(|e| CommandError::new("io", format!("Lecture interrompue : {e}")))??;
    state.with_storage(|s| record_verification(s, &evidence, digest.as_ref()))
}

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn remove_evidence(state: tauri::State<'_, AppState>, id: i64) -> CommandResult<()> {
    state.with_storage(|s| Ok(s.remove_evidence(EvidenceId::new(id))?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Équivalents synchrones des commandes, avec les mêmes fonctions d'enregistrement.
    fn add(s: &mut Storage, inv: i64, path: &str) -> CommandResult<EvidenceDto> {
        let fp = fingerprint(&PathBuf::from(path), |_, _| {}, &AtomicBool::new(false))?;
        record_new(s, inv, fp)
    }

    fn verify(s: &mut Storage, id: i64) -> CommandResult<EvidenceDto> {
        let evidence = s.evidence(EvidenceId::new(id))?;
        let digest = current_digest(&PathBuf::from(&evidence.path), &AtomicBool::new(false))?;
        record_verification(s, &evidence, digest.as_ref())
    }

    fn setup() -> (Storage, i64, tempfile::TempDir) {
        let mut s = Storage::open_in_memory().unwrap();
        let inv = crate::investigations::create(&mut s, "Enquête", "").unwrap();
        (s, inv.id, tempfile::tempdir().unwrap())
    }

    fn write(dir: &tempfile::TempDir, name: &str, content: &[u8]) -> String {
        let path = dir.path().join(name);
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content)
            .unwrap();
        path.to_str().unwrap().to_owned()
    }

    #[test]
    fn ajout_verification_et_alteration() {
        let (mut s, inv, dir) = setup();
        let path = write(&dir, "auth.log", b"abc");
        let e = add(&mut s, inv, &path).unwrap();
        assert_eq!(e.file_name, "auth.log");
        assert_eq!(
            e.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(e.integrity, "intact");
        assert_eq!(list(&s, inv).unwrap().len(), 1);

        assert_eq!(verify(&mut s, e.id).unwrap().integrity, "intact");
        std::fs::write(&path, b"abd").unwrap();
        assert_eq!(verify(&mut s, e.id).unwrap().integrity, "modified");
        std::fs::remove_file(&path).unwrap();
        assert_eq!(verify(&mut s, e.id).unwrap().integrity, "missing");
    }

    #[test]
    fn erreurs_typees() {
        let (mut s, inv, dir) = setup();
        let path = write(&dir, "a.log", b"x");
        add(&mut s, inv, &path).unwrap();
        let copy = write(&dir, "copie.log", b"x");
        assert_eq!(add(&mut s, inv, &copy).unwrap_err().code, "duplicate");
        let absent = dir.path().join("absent.log");
        assert_eq!(
            add(&mut s, inv, absent.to_str().unwrap()).unwrap_err().code,
            "file_not_found"
        );
        assert_eq!(
            add(&mut s, inv, dir.path().to_str().unwrap())
                .unwrap_err()
                .code,
            "not_a_file"
        );
    }
}
