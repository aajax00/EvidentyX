//! Analyse d'une preuve : lecture du format et extraction des événements.
//!
//! Garanties :
//! - le fichier est lu en lecture seule et en flux ;
//! - il doit avoir la taille et l'empreinte SHA-256 enregistrées : sinon les
//!   événements extraits sont retirés et l'analyse est refusée ;
//! - les événements sont enregistrés par lots ; une réanalyse n'ajoute que
//!   ce qui manque (unicité preuve + décalage) ;
//! - la lecture se fait hors du fil de l'interface.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{EvidenceId, IntegrityStatus, NewEvent};
use evidentyx_evidence::{open_read_only, EvidenceError, HashingReader};
use evidentyx_parser::{detect, parse, Format, ParseContext, ParseError};
use evidentyx_storage::Storage;
use serde::Serialize;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use tauri::Manager;

/// Taille des lots d'événements enregistrés en une transaction.
const BATCH: usize = 2000;
/// Octets lus pour reconnaître le format.
const HEAD_BYTES: usize = 64 * 1024;

/// Miroir TypeScript : `AnalysisReport` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReportDto {
    /// Format reconnu (libellé lisible).
    pub format: &'static str,
    pub records: u64,
    /// Événements nouvellement enregistrés.
    pub inserted: u64,
    /// Événements déjà présents (réanalyse).
    pub duplicates: u64,
    pub errors: u64,
    pub first_errors: Vec<RecordErrorDto>,
    /// Colonne retenue pour l'heure (CSV), si une colonne a été reconnue.
    pub time_field: Option<String>,
    /// Structure reconnue (CSV : séparateur, colonnes, en-tête).
    pub layout: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordErrorDto {
    pub line: u64,
    pub byte_offset: u64,
    pub message: String,
}

fn unsupported() -> CommandError {
    CommandError::new(
        "unsupported_format",
        "Format non reconnu pour l'instant : JSON, JSON Lines et CSV sont lus. Syslog, journaux web et PCAP arrivent aux étapes suivantes.",
    )
}

fn parse_error(e: ParseError) -> CommandError {
    match e {
        ParseError::Cancelled => CommandError::new("cancelled", "Analyse annulée."),
        ParseError::Sink(m) => CommandError::new("storage", m),
        other => CommandError::new("parse", capitalize(&other.to_string())),
    }
}

fn capitalize(text: &str) -> String {
    let mut c = text.chars();
    c.next()
        .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
}

/// Opération exécutée sur le stockage, sous verrou.
type StorageOp<'a> = dyn FnMut(&mut Storage) -> Result<(), CommandError> + 'a;
/// Accès au stockage : exécute une opération sous verrou (testable sans Tauri).
type WithStorage<'a> = dyn Fn(&mut StorageOp<'_>) -> Result<(), CommandError> + 'a;

/// Analyse complète d'une preuve.
pub fn analyze(
    with_storage: &WithStorage<'_>,
    evidence_id: i64,
) -> Result<AnalysisReportDto, CommandError> {
    let id = EvidenceId::new(evidence_id);
    let mut evidence = None;
    with_storage(&mut |s| {
        let e = s.evidence(id)?;
        s.investigation(e.investigation_id)?.ensure_editable()?;
        evidence = Some(e);
        Ok(())
    })?;
    let evidence = evidence.ok_or_else(|| CommandError::new("not_found", "Preuve introuvable"))?;
    let path = Path::new(&evidence.path);

    // Reconnaissance du format sur les premiers octets.
    let mut head = vec![0u8; HEAD_BYTES];
    let n = open_read_only(path, evidence.size_bytes)?
        .read(&mut head)
        .map_err(|e| {
            CommandError::from(EvidenceError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        })?;
    head.truncate(n);
    let format: Format = detect(&evidence.file_name, &head).ok_or_else(unsupported)?;

    // Lecture complète avec empreinte, enregistrement par lots.
    let mut reader = HashingReader::new(open_read_only(path, evidence.size_bytes)?);
    let cancel = AtomicBool::new(false);
    let mut progress = |_: u64| {};
    let mut batch: Vec<NewEvent> = Vec::with_capacity(BATCH);
    let (mut inserted, mut duplicates) = (0u64, 0u64);
    let mut flush = |batch: &mut Vec<NewEvent>| -> Result<(), String> {
        if batch.is_empty() {
            return Ok(());
        }
        let mut result = Ok(());
        with_storage(&mut |s| {
            let r = s.insert_events(id, batch)?;
            inserted += r.inserted;
            duplicates += r.duplicates;
            Ok(())
        })
        .unwrap_or_else(|e| result = Err(e.message));
        batch.clear();
        result
    };
    let report = {
        let mut sink = |e: NewEvent| {
            batch.push(e);
            if batch.len() >= BATCH {
                flush(&mut batch)?;
            }
            Ok(())
        };
        parse(
            format,
            &mut reader,
            ParseContext {
                evidence_id: id,
                on_progress: &mut progress,
                cancel: &cancel,
            },
            &mut sink,
        )
    };
    let outcome = report.map_err(parse_error).and_then(|r| {
        flush(&mut batch).map_err(|m| CommandError::new("storage", m))?;
        Ok(r)
    });

    // Le contenu lu doit être exactement la preuve enregistrée. Si la lecture
    // s'est arrêtée avant la fin (structure illisible), on lit le reste pour
    // que l'empreinte porte sur le fichier entier.
    std::io::copy(&mut reader, &mut std::io::sink()).map_err(|e| {
        CommandError::from(EvidenceError::Io {
            path: path.to_path_buf(),
            source: e,
        })
    })?;
    let digest = reader.finish();
    if digest != evidence.sha256 {
        with_storage(&mut |s| {
            s.delete_events_for_evidence(id)?;
            s.record_integrity(id, IntegrityStatus::Modified)?;
            Ok(())
        })?;
        return Err(EvidenceError::Changed(path.to_path_buf()).into());
    }
    let report = outcome?;
    with_storage(&mut |s| {
        s.record_integrity(id, IntegrityStatus::Intact)?;
        Ok(())
    })?;
    Ok(AnalysisReportDto {
        format: report.format.label_fr(),
        records: report.records,
        inserted,
        duplicates,
        errors: report.errors,
        time_field: report.time_field,
        layout: report.layout,
        first_errors: report
            .first_errors
            .into_iter()
            .take(10)
            .map(|e| RecordErrorDto {
                line: e.line,
                byte_offset: e.byte_offset,
                message: e.message,
            })
            .collect(),
    })
}

#[tauri::command]
pub async fn analyze_evidence(
    app: tauri::AppHandle,
    evidence_id: i64,
) -> Result<AnalysisReportDto, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        analyze(&|f| state.with_storage(|s| f(s)), evidence_id)
    })
    .await
    .map_err(|e| CommandError::new("io", format!("Analyse interrompue : {e}")))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::io::Write;

    fn setup(
        content: &[u8],
        name: &str,
    ) -> (RefCell<Storage>, i64, tempfile::TempDir, std::path::PathBuf) {
        let mut s = Storage::open_in_memory().unwrap();
        let inv = crate::investigations::create(&mut s, "Enquête", "").unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(name);
        std::fs::File::create(&path)
            .unwrap()
            .write_all(content)
            .unwrap();
        let fp =
            evidentyx_evidence::fingerprint(&path, |_, _| {}, &AtomicBool::new(false)).unwrap();
        let ev = s
            .add_evidence(
                evidentyx_core::InvestigationId::new(inv.id),
                &evidentyx_storage::NewEvidence {
                    file_name: fp.file_name,
                    path: fp.path.to_str().unwrap().into(),
                    size_bytes: fp.size_bytes,
                    sha256: fp.sha256,
                    file_modified_at: fp.modified_at,
                },
            )
            .unwrap();
        (RefCell::new(s), ev.id.get(), dir, path)
    }

    fn run(db: &RefCell<Storage>, id: i64) -> Result<AnalysisReportDto, CommandError> {
        analyze(&|f| f(&mut db.borrow_mut()), id)
    }

    const LOG: &[u8] = b"{\"ts\":\"2026-11-16T00:00:21Z\",\"user\":\"admin\",\"action\":\"login\",\"status\":\"failed\"}\n{broken\n{\"ts\":\"2026-11-16T00:01:00Z\",\"message\":\"ok\"}\n";

    #[test]
    fn analyse_puis_reanalyse_sans_doublon() {
        let (db, id, _dir, _) = setup(LOG, "auth.jsonl");
        let r = run(&db, id).unwrap();
        assert_eq!(
            (r.format, r.records, r.inserted, r.duplicates, r.errors),
            ("JSON Lines", 3, 2, 0, 1)
        );
        assert_eq!(r.first_errors[0].line, 2);
        let r = run(&db, id).unwrap();
        assert_eq!((r.inserted, r.duplicates), (0, 2));
        let inv = db
            .borrow()
            .evidence(EvidenceId::new(id))
            .unwrap()
            .investigation_id;
        assert_eq!(db.borrow().count_events(inv).unwrap(), 2);
    }

    #[test]
    fn fichier_modifie_refuse_sans_laisser_d_evenements() {
        let (db, id, _dir, path) = setup(LOG, "auth.jsonl");
        // Même taille, contenu différent : seule l'empreinte le révèle.
        let mut altered = LOG.to_vec();
        altered[30] = b'X';
        std::fs::write(&path, &altered).unwrap();
        assert_eq!(run(&db, id).unwrap_err().code, "evidence_changed");
        let e = db.borrow().evidence(EvidenceId::new(id)).unwrap();
        assert_eq!(e.integrity, IntegrityStatus::Modified);
        assert_eq!(db.borrow().count_events(e.investigation_id).unwrap(), 0);
        // Taille différente : refus immédiat.
        std::fs::write(&path, b"{}").unwrap();
        assert_eq!(run(&db, id).unwrap_err().code, "evidence_changed");
    }

    #[test]
    fn structure_illisible_sans_fausse_alerte_d_integrite() {
        let (db, id, _dir, _) = setup(b"[{\"message\":\"a\"}, {\"message\": \"b\"", "export.json");
        let err = run(&db, id).unwrap_err();
        assert_eq!(err.code, "parse");
        let e = db.borrow().evidence(EvidenceId::new(id)).unwrap();
        assert_eq!(e.integrity, IntegrityStatus::Intact);
    }

    #[test]
    fn format_non_reconnu() {
        let (db, id, _dir, _) = setup(b"Nov 16 10:00:00 srv sshd[42]: Accepted\n", "auth.log");
        assert_eq!(run(&db, id).unwrap_err().code, "unsupported_format");
    }
}
