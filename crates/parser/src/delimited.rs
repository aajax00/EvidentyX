//! CSV et formats délimités (`,` `;` tabulation `|`).
//!
//! - Séparateur deviné sur les premières lignes (nombre de colonnes constant,
//!   guillemets respectés) ; extension `.tsv` → tabulation.
//! - En-tête reconnu si la première ligne ne contient que des noms (non vides,
//!   non numériques, distincts) ; sinon colonnes nommées `colonne_1`, `colonne_2`…
//! - Lecture conforme RFC 4180 (crate `csv`) : guillemets, guillemets doublés,
//!   champs sur plusieurs lignes, fins de ligne CRLF ; nombre de colonnes variable
//!   toléré.
//! - Colonne d'heure choisie une fois pour le fichier : nom reconnu (ou contenant
//!   « time », « date », « heure »…) dont la valeur se lit sur la première ligne ;
//!   colonnes date et heure séparées combinées.
//! - Texte brut : enregistrement reconstitué (mêmes valeurs, séparateur d'origine,
//!   guillemets normalisés) ; le décalage et la ligne désignent l'original exact.

use crate::fields::{map_fields_with, norm, Hints, Scalar, TIME};
use crate::{time, Format, ParseContext, ParseError, ParseReport};
use evidentyx_core::{NewEvent, SourceRef, TimePrecision};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::io::{Cursor, Read};
use std::rc::Rc;
use std::sync::atomic::Ordering;

const CANDIDATES: [u8; 4] = *b",;\t|";
const SNIFF_BYTES: usize = 64 * 1024;
const SNIFF_LINES: usize = 50;
/// Nom du champ synthétique quand date et heure sont dans deux colonnes.
const COMBINED_TIME: &str = "evidentyx.horodatage";

/// Compte les lignes au fil de la lecture. Avec des fins de ligne CRLF, la
/// crate `csv` donne des numéros de ligne faux et un décalage qui pointe sur le
/// `\n` précédant l'enregistrement : ligne et décalage sont donc recalculés. Le lecteur
/// note la position de chaque `\n` lu ; le numéro de ligne d'un enregistrement
/// se déduit de son décalage. Mémoire bornée : seules les fins de ligne pas
/// encore dépassées par les enregistrements sont gardées.
#[derive(Default)]
struct Lines {
    pending: VecDeque<u64>,
    passed: u64,
}

impl Lines {
    /// Début réel et ligne (à partir de 1) de l'enregistrement annoncé à `offset`.
    fn locate(&mut self, offset: u64) -> (u64, u64) {
        while self.pending.front().is_some_and(|&nl| nl < offset) {
            self.pending.pop_front();
            self.passed += 1;
        }
        if self.pending.front() == Some(&offset) {
            // Décalage posé sur une fin de ligne : l'enregistrement commence juste après.
            self.pending.pop_front();
            self.passed += 1;
            return (offset + 1, self.passed + 1);
        }
        (offset, self.passed + 1)
    }
}

struct CountingReader<R> {
    inner: R,
    position: u64,
    lines: Rc<RefCell<Lines>>,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n = self.inner.read(buf)?;
        let mut lines = self.lines.borrow_mut();
        for (i, &b) in buf[..n].iter().enumerate() {
            if b == b'\n' {
                lines.pending.push_back(self.position + i as u64);
            }
        }
        self.position += n as u64;
        Ok(n)
    }
}

/// Découpe le texte en enregistrements (fins de ligne hors guillemets) et
/// compte les champs de chacun pour un séparateur.
fn field_counts(text: &str, delim: u8, max: usize) -> Vec<usize> {
    let mut counts = Vec::new();
    let mut n = 1;
    let mut quoted = false;
    let mut empty = true;
    for b in text.bytes() {
        match b {
            b'"' => {
                quoted = !quoted;
                empty = false;
            }
            b'\n' if !quoted => {
                if !empty {
                    counts.push(n);
                    if counts.len() >= max {
                        return counts;
                    }
                }
                n = 1;
                empty = true;
            }
            b'\r' => {}
            _ if b == delim && !quoted => {
                n += 1;
                empty = false;
            }
            _ => {
                if !b.is_ascii_whitespace() {
                    empty = false;
                }
            }
        }
    }
    if !empty {
        counts.push(n);
    }
    counts
}

/// Séparateur dont le nombre de colonnes (≥ 2) est le plus constant.
fn sniff(head: &[u8]) -> Option<(u8, usize)> {
    let text = String::from_utf8_lossy(head);
    let truncated = head.len() >= SNIFF_BYTES;
    let mut best: Option<(u8, usize, usize)> = None; // (séparateur, colonnes, enregistrements conformes)
    for &d in &CANDIDATES {
        let mut counts = field_counts(&text, d, SNIFF_LINES + 1);
        if truncated && counts.len() > 1 {
            counts.pop(); // dernier enregistrement possiblement tronqué
        }
        let Some(&first) = counts.first() else {
            continue;
        };
        if first < 2 {
            continue;
        }
        let consistent = counts.iter().filter(|&&c| c == first).count();
        // Au moins 80 % des enregistrements doivent avoir le même nombre de colonnes.
        if consistent * 5 < counts.len() * 4 {
            continue;
        }
        if best.is_none_or(|(_, cols, ok)| (consistent, first) > (ok, cols)) {
            best = Some((d, first, consistent));
        }
    }
    best.map(|(d, cols, _)| (d, cols))
}

pub fn detect(file_name: &str, head: &[u8]) -> Option<Format> {
    let lower = file_name.to_lowercase();
    let by_extension = lower.ends_with(".csv") || lower.ends_with(".tsv");
    match sniff(head) {
        Some(_) => Some(Format::Csv),
        None if by_extension && !head.is_empty() => Some(Format::Csv),
        None => None,
    }
}

fn looks_like_header(fields: &[String]) -> bool {
    if fields.is_empty() {
        return false;
    }
    let mut seen = std::collections::HashSet::new();
    fields.iter().all(|f| {
        let t = f.trim();
        !t.is_empty()
            && t.parse::<f64>().is_err()
            && time::parse_text(t).is_none()
            && seen.insert(t.to_lowercase())
    })
}

/// Noms de colonnes uniques (doublons suffixés `_2`, `_3`…).
fn unique_names(raw: &[String]) -> Vec<String> {
    let mut counts = std::collections::HashMap::<String, usize>::new();
    raw.iter()
        .map(|n| {
            let base = n.trim().trim_start_matches('\u{feff}').to_owned();
            let c = counts.entry(base.to_lowercase()).or_insert(0);
            *c += 1;
            if *c == 1 {
                base
            } else {
                format!("{base}_{c}")
            }
        })
        .collect()
}

fn column_name(names: &[String], i: usize) -> String {
    names
        .get(i)
        .cloned()
        .unwrap_or_else(|| format!("colonne_{}", i + 1))
}

/// Choix de l'heure pour le fichier, sur la première ligne de données.
#[derive(Debug, Clone, PartialEq, Eq)]
enum TimeChoice {
    Column(usize),
    /// Colonnes date et heure combinées.
    Combined(usize, usize),
    None,
}

fn choose_time(names: &[String], row: &[String]) -> TimeChoice {
    let normed: Vec<String> = names.iter().map(|n| norm(n)).collect();
    let parses = |i: usize| row.get(i).and_then(|v| time::parse_text(v));
    // 1. Alias connus, dans l'ordre de priorité.
    for alias in TIME {
        if let Some(i) = normed.iter().position(|n| n == alias) {
            if parses(i).is_some_and(|t| t.precision != TimePrecision::Day) {
                return TimeChoice::Column(i);
            }
        }
    }
    // 2. Colonne date + colonne heure séparées.
    let date_col = normed.iter().position(|n| n == "date" || n == "jour");
    let time_col = normed
        .iter()
        .position(|n| n == "time" || n == "heure" || n == "hour");
    if let (Some(d), Some(t)) = (date_col, time_col) {
        let combined = format!(
            "{} {}",
            row.get(d).map_or("", |s| s.trim()),
            row.get(t).map_or("", |s| s.trim())
        );
        if time::parse_text(&combined).is_some() {
            return TimeChoice::Combined(d, t);
        }
    }
    // 3. Toute colonne dont le nom évoque une heure, puis les alias en précision jour.
    let timeish = [
        "time",
        "date",
        "heure",
        "horodatage",
        "timestamp",
        "created",
        "modified",
    ];
    if let Some(i) = normed
        .iter()
        .enumerate()
        .position(|(i, n)| timeish.iter().any(|w| n.contains(w)) && parses(i).is_some())
    {
        return TimeChoice::Column(i);
    }
    TimeChoice::None
}

/// Reconstitue l'enregistrement avec le séparateur d'origine.
fn rebuild(fields: &[String], delim: u8) -> String {
    let d = delim as char;
    fields
        .iter()
        .map(|f| {
            if f.contains(d) || f.contains('"') || f.contains('\n') || f.contains('\r') {
                format!("\"{}\"", f.replace('"', "\"\""))
            } else {
                f.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(&d.to_string())
}

fn delim_label(d: u8) -> &'static str {
    match d {
        b',' => "virgule",
        b';' => "point-virgule",
        b'\t' => "tabulation",
        _ => "barre verticale",
    }
}

pub fn parse(
    mut reader: impl Read,
    ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<ParseReport, ParseError> {
    let mut report = ParseReport::new(Format::Csv);
    // Lecture de l'en-tête de fichier pour deviner le séparateur, puis relecture enchaînée.
    let mut head = Vec::with_capacity(SNIFF_BYTES);
    (&mut reader)
        .take(SNIFF_BYTES as u64)
        .read_to_end(&mut head)?;
    let (delim, _) = sniff(&head).unwrap_or((b',', 1));
    let lines = Rc::new(RefCell::new(Lines::default()));
    let chained = CountingReader {
        inner: Cursor::new(head).chain(reader),
        position: 0,
        lines: Rc::clone(&lines),
    };

    let mut rdr = ::csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .from_reader(chained);

    let mut names: Option<Vec<String>> = None;
    let mut hints = Hints::default();
    let mut choice: Option<TimeChoice> = None;
    let mut record = ::csv::ByteRecord::new();
    let mut data_no = 0u64;
    loop {
        if ctx.cancel.load(Ordering::Relaxed) {
            return Err(ParseError::Cancelled);
        }
        let pos_before = rdr.position().clone();
        match rdr.read_byte_record(&mut record) {
            Ok(false) => break,
            Ok(true) => {}
            Err(e) => {
                if let ::csv::ErrorKind::Io(_) = e.kind() {
                    return Err(ParseError::Io(std::io::Error::other(e.to_string())));
                }
                report.records += 1;
                report.error(
                    lines.borrow_mut().locate(pos_before.byte()).1,
                    pos_before.byte(),
                    format!("CSV illisible : {e}"),
                );
                continue;
            }
        }
        let pos = record.position().cloned().unwrap_or(pos_before);
        let (offset, line) = lines.borrow_mut().locate(pos.byte());
        let values: Vec<String> = record
            .iter()
            .map(|f| String::from_utf8_lossy(f).into_owned())
            .collect();
        if values.iter().all(|v| v.trim().is_empty()) {
            continue;
        }
        // Première ligne non vide : en-tête ou données ?
        if names.is_none() {
            if looks_like_header(&values) {
                names = Some(unique_names(&values));
                report.layout = Some(format!(
                    "séparateur : {} · {} colonnes · en-tête",
                    delim_label(delim),
                    values.len()
                ));
                continue;
            }
            names = Some(Vec::new());
            report.layout = Some(format!(
                "séparateur : {} · {} colonnes · sans en-tête",
                delim_label(delim),
                values.len()
            ));
        }
        let header = names.as_deref().unwrap_or(&[]);
        let choice = choice.get_or_insert_with(|| {
            let c = choose_time(header, &values);
            match &c {
                TimeChoice::Column(i) => {
                    hints.time_field = Some(column_name(header, *i));
                    report.time_field = hints.time_field.clone();
                }
                TimeChoice::Combined(d, t) => {
                    hints.time_field = Some(COMBINED_TIME.into());
                    report.time_field = Some(format!(
                        "{} + {}",
                        column_name(header, *d),
                        column_name(header, *t)
                    ));
                }
                TimeChoice::None => {}
            }
            c
        });

        report.records += 1;
        data_no += 1;
        let mut fields: Vec<(String, Scalar)> = values
            .iter()
            .enumerate()
            .map(|(i, v)| (column_name(header, i), Scalar::Text(v.clone())))
            .collect();
        if let TimeChoice::Combined(d, t) = choice {
            let combined = format!(
                "{} {}",
                values.get(*d).map_or("", |s| s.trim()),
                values.get(*t).map_or("", |s| s.trim())
            );
            fields.push((COMBINED_TIME.into(), Scalar::Text(combined)));
        }
        let source = SourceRef {
            evidence_id: ctx.evidence_id,
            byte_offset: offset,
            line: Some(line),
            record: Some(data_no),
        };
        let raw = rebuild(&values, delim);
        let mut event = map_fields_with(source, &raw, fields, &hints);
        // Colonnes date/heure d'origine : conservées telles quelles en attributs.
        event.attributes.remove(COMBINED_TIME);
        if let Err(e) = event.validate() {
            report.error(line, offset, e.to_string());
            continue;
        }
        sink(event).map_err(ParseError::Sink)?;
        report.events += 1;
        report.bytes = offset;
        if data_no.is_multiple_of(1024) {
            (ctx.on_progress)(offset);
        }
    }
    report.bytes = rdr.position().byte();
    (ctx.on_progress)(report.bytes);
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{detect, parse as parse_any};
    use evidentyx_core::{EventKind, EvidenceId, Outcome, TimeBasis};
    use std::sync::atomic::AtomicBool;

    fn run(input: &str) -> (ParseReport, Vec<NewEvent>) {
        let cancel = AtomicBool::new(false);
        let mut progress = |_| {};
        let mut events = Vec::new();
        let mut sink = |e: NewEvent| {
            events.push(e);
            Ok(())
        };
        let r = parse_any(
            Format::Csv,
            input.as_bytes(),
            ParseContext {
                evidence_id: EvidenceId::new(3),
                on_progress: &mut progress,
                cancel: &cancel,
            },
            &mut sink,
        )
        .unwrap();
        (r, events)
    }

    const SYNTH: &str = "timestamp_utc,event_id,source_ip,destination_host,user,service,action,severity,status,path_or_resource,message,simulation\n\
2026-11-16T00:00:21Z,EVT-0015144,198.51.100.40,vpn-01,admin,smb,http_request,INFO,blocked,-,HTTP request processed,true\n\
2026-11-16T00:00:37Z,EVT-0016312,10.0.2.235,dc-01,jdupont,ssh,login,WARN,failed,/admin,\"Échec, mot de passe \"\"incorrect\"\"\",true\n";

    #[test]
    fn journal_synthetique() {
        let (r, ev) = run(SYNTH);
        assert_eq!((r.records, r.events, r.errors), (2, 2, 0));
        assert_eq!(r.time_field.as_deref(), Some("timestamp_utc"));
        assert_eq!(
            r.layout.as_deref(),
            Some("séparateur : virgule · 12 colonnes · en-tête")
        );
        let b = &ev[1];
        assert_eq!(b.source.line, Some(3));
        assert_eq!(
            &SYNTH.as_bytes()[b.source.byte_offset as usize..][..4],
            b"2026"
        );
        assert_eq!(b.time.unwrap().basis, TimeBasis::Explicit);
        assert_eq!(b.source_ip.unwrap().to_string(), "10.0.2.235");
        assert_eq!(b.host.as_deref(), Some("dc-01"));
        assert_eq!(b.user.as_deref(), Some("jdupont"));
        assert_eq!(b.outcome, Some(Outcome::Failure));
        assert_eq!(b.kind, EventKind::Authentication);
        assert_eq!(b.message, "Échec, mot de passe \"incorrect\"");
        assert_eq!(
            b.attributes.get("event_id").map(String::as_str),
            Some("EVT-0016312")
        );
        assert!(b.raw.contains("\"Échec, mot de passe \"\"incorrect\"\"\""));
    }

    #[test]
    fn point_virgule_date_et_heure_separees_sans_en_tete_d_heure() {
        let input = "Date;Heure;Poste;Utilisateur;Évènement\r\n16/11/2026;10:00:00;PC-1;alice;ouverture\r\n2026-11-16;10:05:30;PC-2;bob;\"session\r\nfermée\"\r\n";
        let (r, ev) = run(input);
        assert_eq!(
            r.layout.as_deref(),
            Some("séparateur : point-virgule · 5 colonnes · en-tête")
        );
        // Première ligne : date au format français non reconnu → pas de choix d'heure.
        assert_eq!(r.time_field, None);
        assert_eq!(ev.len(), 2);
        // Colonnes françaises reconnues : poste → hôte, utilisateur, évènement → action.
        assert_eq!(ev[1].host.as_deref(), Some("PC-2"));
        assert_eq!(ev[1].user.as_deref(), Some("bob"));
        assert_eq!(ev[1].action.as_deref(), Some("session\r\nfermée"));
        assert_eq!(ev[1].source.line, Some(3));
        assert_eq!(
            &input.as_bytes()[ev[1].source.byte_offset as usize..][..4],
            b"2026"
        );
        assert_eq!(
            &input.as_bytes()[ev[0].source.byte_offset as usize..][..2],
            b"16"
        );
    }

    #[test]
    fn date_et_heure_combinees() {
        let input = "date\ttime\thost\tmsg\n2026-11-16\t10:00:00\tsrv\tdémarrage\n";
        let (r, ev) = run(input);
        assert_eq!(r.time_field.as_deref(), Some("date + time"));
        let t = ev[0].time.unwrap();
        assert_eq!(t.precision, TimePrecision::Second);
        assert_eq!(t.basis, TimeBasis::AssumedUtc);
        assert_eq!(
            ev[0].attributes.get("date").map(String::as_str),
            Some("2026-11-16")
        );
        assert!(!ev[0].attributes.contains_key(COMBINED_TIME));
    }

    #[test]
    fn sans_en_tete_et_colonnes_variables() {
        let input = "198.51.100.7,443,ok\n10.0.0.1,22\n";
        let (r, ev) = run(input);
        assert!(r.layout.unwrap().ends_with("sans en-tête"));
        assert_eq!(ev.len(), 2);
        assert_eq!(
            ev[0].attributes.get("colonne_1").map(String::as_str),
            Some("198.51.100.7")
        );
        assert_eq!(ev[1].attributes.get("colonne_3"), None);
    }

    #[test]
    fn flux_cic_ids_sans_heure() {
        let input = " Destination Port, Flow Duration, Total Fwd Packets, Label\n54865,3,2,BENIGN\n80,1293792,3,DDoS\n";
        let (r, ev) = run(input);
        assert_eq!(r.time_field, None);
        assert_eq!(ev[1].destination_port, Some(80));
        assert!(ev[1].time.is_none(), "aucune heure inventée");
        assert_eq!(ev[1].kind, EventKind::Network);
        assert_eq!(
            ev[1].attributes.get("Label").map(String::as_str),
            Some("DDoS")
        );
    }

    #[test]
    fn profil_evtxecmd() {
        let input = "RecordNumber,TimeCreated,EventId,Level,Provider,Channel,Computer,UserName,MapDescription,PayloadData1\n\
42,2026-11-16 08:15:02.1234567,4625,Info,Microsoft-Windows-Security-Auditing,Security,WS-07.corp.local,CORP\\alice,Failed logon,Target: CORP\\alice\n";
        let (r, ev) = run(input);
        assert_eq!(r.time_field.as_deref(), Some("TimeCreated"));
        let e = &ev[0];
        assert_eq!(e.host.as_deref(), Some("WS-07.corp.local"));
        assert_eq!(e.user.as_deref(), Some("CORP\\alice"));
        assert_eq!(e.message, "Failed logon");
        assert_eq!(e.kind, EventKind::Authentication);
        assert_eq!(e.time.unwrap().precision, TimePrecision::Microsecond);
        assert_eq!(
            e.attributes.get("EventId").map(String::as_str),
            Some("4625")
        );
    }

    #[test]
    fn detection() {
        assert_eq!(detect("a.csv", b"x,y\n1,2\n"), Some(Format::Csv));
        assert_eq!(detect("a.txt", b"a;b;c\n1;2;3\n4;5;6\n"), Some(Format::Csv));
        assert_eq!(
            detect(
                "a.log",
                b"Nov 16 10:00:00 srv sshd[42]: Accepted password\n"
            ),
            None
        );
        assert_eq!(detect("a.json", b"[{}]"), Some(Format::JsonArray));
        assert_eq!(sniff(b"a\tb\tc\n1\t2\t3\n"), Some((b'\t', 3)));
    }
}
