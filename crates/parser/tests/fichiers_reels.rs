//! Essais sur de vrais fichiers, hors dépôt (taille, licence).
//!
//! Lancer : `EVIDENTYX_FICHIER_TEST=/chemin/vers/fichier.jsonl cargo test -p evidentyx-parser --release -- --ignored --nocapture`

use evidentyx_core::{EvidenceId, NewEvent};
use evidentyx_parser::{detect, parse, ParseContext};
use std::io::Read;
use std::sync::atomic::AtomicBool;
use std::time::Instant;

#[test]
#[ignore = "nécessite EVIDENTYX_FICHIER_TEST"]
fn fichier_reel() {
    let Ok(path) = std::env::var("EVIDENTYX_FICHIER_TEST") else {
        eprintln!("EVIDENTYX_FICHIER_TEST non défini : essai ignoré");
        return;
    };
    let mut head = vec![0u8; 65536];
    let n = std::fs::File::open(&path).unwrap().read(&mut head).unwrap();
    let format = detect(&path, &head[..n]).expect("format reconnu");
    let started = Instant::now();
    let (mut with_time, mut with_ip, mut with_user) = (0u64, 0u64, 0u64);
    let mut sink = |e: NewEvent| {
        with_time += u64::from(e.time.is_some());
        with_ip += u64::from(e.source_ip.is_some());
        with_user += u64::from(e.user.is_some());
        Ok(())
    };
    let cancel = AtomicBool::new(false);
    let mut progress = |_| {};
    let report = parse(
        format,
        std::fs::File::open(&path).unwrap(),
        ParseContext {
            evidence_id: EvidenceId::new(1),
            on_progress: &mut progress,
            cancel: &cancel,
        },
        &mut sink,
    )
    .unwrap();
    let secs = started.elapsed().as_secs_f64();
    println!(
        "{} · {} enregistrements · {} événements · {} erreurs · {:.1} Mo en {:.2} s ({:.0} év./s) · heure {} · IP {} · utilisateur {}",
        format.label_fr(), report.records, report.events, report.errors,
        report.bytes as f64 / 1e6, secs, report.events as f64 / secs, with_time, with_ip, with_user
    );
    assert_eq!(report.records, report.events + report.errors);
}
