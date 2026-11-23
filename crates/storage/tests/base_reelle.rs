//! Mesures sur une copie d'une vraie base, hors dépôt. Modifie la base indiquée :
//! toujours lancer sur une COPIE.
//!
//! `EVIDENTYX_BASE_TEST=/copie.sqlite cargo test -p evidentyx-storage --test base_reelle -- --ignored --nocapture`

use evidentyx_storage::Storage;
use std::time::Instant;

#[test]
#[ignore = "nécessite EVIDENTYX_BASE_TEST (une copie)"]
fn suppression() {
    let Ok(path) = std::env::var("EVIDENTYX_BASE_TEST") else {
        return;
    };
    let t = Instant::now();
    let mut db = Storage::open(&path).unwrap();
    println!("ouverture + migration {:.2?}", t.elapsed());
    for inv in db.list_investigations(None).unwrap() {
        let t = Instant::now();
        let counts = db.event_counts_by_evidence(inv.id).unwrap();
        let list = db.list_evidence(inv.id).unwrap();
        println!("chargement enquête {:.2?} : {counts:?}", t.elapsed());
        if let Some(e) = list.last() {
            let t = Instant::now();
            db.remove_evidence(e.id).unwrap();
            println!("retrait preuve {:.2?}", t.elapsed());
        }
        let t = Instant::now();
        db.delete_investigation(inv.id).unwrap();
        println!("suppression enquête {:.2?}", t.elapsed());
    }
}
