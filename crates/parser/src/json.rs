//! JSON : JSON Lines, tableau d'objets, objets concaténés.
//!
//! - JSON Lines : lecture ligne à ligne ; une ligne invalide est signalée et la
//!   lecture continue à la ligne suivante.
//! - Tableau et objets concaténés : un découpeur à états (chaînes, échappements,
//!   profondeur) isole chaque valeur de premier niveau en flux, avec son
//!   décalage et sa ligne ; chaque valeur est ensuite analysée par `serde_json`.

use crate::fields::{map_fields, Scalar};
use crate::{Format, ParseContext, ParseError, ParseReport, MAX_RECORD_BYTES};
use evidentyx_core::{NewEvent, SourceRef};
use serde_json::Value;
use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::Ordering;

/// Profondeur d'aplatissement des objets imbriqués.
const MAX_DEPTH: usize = 8;

pub fn detect(file_name: &str, head: &[u8]) -> Option<Format> {
    let lower = file_name.to_lowercase();
    if lower.ends_with(".jsonl") || lower.ends_with(".ndjson") {
        return Some(Format::JsonLines);
    }
    let text = String::from_utf8_lossy(head);
    let trimmed = text.trim_start_matches('\u{feff}').trim_start();
    match trimmed.chars().next()? {
        '[' => Some(Format::JsonArray),
        '{' => {
            // Une ligne par objet ? Sinon, objets répartis sur plusieurs lignes.
            let lines: Vec<&str> = trimmed
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect();
            let complete: Vec<&&str> = lines
                .iter()
                .take(lines.len().saturating_sub(1).max(1))
                .collect();
            let one_per_line = complete
                .iter()
                .all(|l| l.starts_with('{') && l.ends_with('}'));
            Some(if one_per_line {
                Format::JsonLines
            } else {
                Format::JsonConcatenated
            })
        }
        _ => None,
    }
}

/// Aplatit une valeur JSON en champs `a.b.c` → scalaire ; les tableaux sont
/// conservés en JSON compact.
fn flatten(prefix: &str, value: &Value, depth: usize, out: &mut Vec<(String, Scalar)>) {
    let key = |k: &str| {
        if prefix.is_empty() {
            k.to_owned()
        } else {
            format!("{prefix}.{k}")
        }
    };
    match value {
        Value::Object(map) if depth < MAX_DEPTH => {
            for (k, v) in map {
                flatten(&key(k), v, depth + 1, out);
            }
        }
        Value::Null => {}
        Value::Bool(b) => out.push((prefix.to_owned(), Scalar::Bool(*b))),
        Value::Number(n) => out.push((
            prefix.to_owned(),
            Scalar::Number(n.as_f64().unwrap_or(f64::NAN), n.to_string()),
        )),
        Value::String(s) => out.push((prefix.to_owned(), Scalar::Text(s.clone()))),
        other => out.push((prefix.to_owned(), Scalar::Text(other.to_string()))),
    }
}

struct Emitter<'a, 'b> {
    ctx: ParseContext<'a>,
    sink: &'b mut dyn FnMut(NewEvent) -> Result<(), String>,
    report: ParseReport,
    record_no: u64,
}

impl Emitter<'_, '_> {
    fn cancelled(&self) -> bool {
        self.ctx.cancel.load(Ordering::Relaxed)
    }

    /// Analyse un enregistrement (texte d'une valeur JSON) et transmet l'événement.
    fn record(&mut self, bytes: &[u8], line: u64, offset: u64) -> Result<(), ParseError> {
        self.report.records += 1;
        self.record_no += 1;
        let text = String::from_utf8_lossy(bytes);
        let value: Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                self.report
                    .error(line, offset, format!("JSON invalide : {e}"));
                return Ok(());
            }
        };
        if !value.is_object() {
            self.report.error(line, offset, "objet JSON attendu");
            return Ok(());
        }
        let mut fields = Vec::new();
        flatten("", &value, 0, &mut fields);
        let source = SourceRef {
            evidence_id: self.ctx.evidence_id,
            byte_offset: offset,
            line: Some(line),
            record: Some(self.record_no),
        };
        let event = map_fields(source, &text, fields);
        if let Err(e) = event.validate() {
            self.report.error(line, offset, e.to_string());
            return Ok(());
        }
        (self.sink)(event).map_err(ParseError::Sink)?;
        self.report.events += 1;
        Ok(())
    }
}

pub fn parse(
    format: Format,
    reader: impl Read,
    ctx: ParseContext<'_>,
    sink: &mut dyn FnMut(NewEvent) -> Result<(), String>,
) -> Result<crate::ParseReport, ParseError> {
    let mut em = Emitter {
        ctx,
        sink,
        report: ParseReport::new(format),
        record_no: 0,
    };
    let mut reader = BufReader::with_capacity(1 << 16, reader);
    match format {
        Format::JsonLines => lines(&mut reader, &mut em)?,
        Format::JsonArray | Format::JsonConcatenated => {
            values(&mut reader, &mut em, format == Format::JsonArray)?
        }
        Format::Csv | Format::Syslog | Format::WebAccess => {
            unreachable!("format lu par un autre module")
        }
    }
    Ok(em.report)
}

/// JSON Lines : une valeur par ligne.
fn lines(reader: &mut impl BufRead, em: &mut Emitter<'_, '_>) -> Result<(), ParseError> {
    let mut offset = 0u64;
    let mut line_no = 0u64;
    let mut buf = Vec::new();
    loop {
        if em.cancelled() {
            return Err(ParseError::Cancelled);
        }
        buf.clear();
        // Lecture bornée : une ligne démesurée est signalée puis sautée.
        let n = read_line_bounded(reader, &mut buf)?;
        if n == 0 {
            break;
        }
        line_no += 1;
        let start = offset;
        offset += n as u64;
        em.report.bytes = offset;
        if line_no.is_multiple_of(256) {
            (em.ctx.on_progress)(offset);
        }
        if buf.len() > MAX_RECORD_BYTES {
            em.report.records += 1;
            em.report.error(
                line_no,
                start,
                format!("ligne trop longue (plus de {MAX_RECORD_BYTES} octets)"),
            );
            continue;
        }
        let mut content: &[u8] = &buf;
        if line_no == 1 && content.starts_with(&[0xEF, 0xBB, 0xBF]) {
            content = &content[3..];
        }
        let trimmed = trim_ascii(content);
        if trimmed.is_empty() {
            continue;
        }
        em.record(trimmed, line_no, start)?;
    }
    (em.ctx.on_progress)(offset);
    Ok(())
}

/// Lit une ligne ; au-delà de `MAX_RECORD_BYTES`, consomme le reste sans le garder.
/// Renvoie le nombre d'octets consommés (fin de ligne comprise).
pub(crate) fn read_line_bounded(
    reader: &mut impl BufRead,
    buf: &mut Vec<u8>,
) -> std::io::Result<usize> {
    let mut total = 0;
    loop {
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Ok(total);
        }
        let (chunk, done) = match available.iter().position(|&b| b == b'\n') {
            Some(i) => (&available[..=i], true),
            None => (available, false),
        };
        let len = chunk.len();
        if buf.len() <= MAX_RECORD_BYTES {
            buf.extend_from_slice(chunk);
        }
        reader.consume(len);
        total += len;
        if done {
            return Ok(total);
        }
    }
}

pub(crate) fn trim_ascii(b: &[u8]) -> &[u8] {
    let start = b
        .iter()
        .position(|c| !c.is_ascii_whitespace())
        .unwrap_or(b.len());
    let end = b
        .iter()
        .rposition(|c| !c.is_ascii_whitespace())
        .map_or(start, |i| i + 1);
    &b[start..end]
}

/// Tableau JSON (éléments) ou valeurs concaténées, découpés en flux.
fn values(
    reader: &mut impl BufRead,
    em: &mut Emitter<'_, '_>,
    array: bool,
) -> Result<(), ParseError> {
    let mut offset = 0u64; // octets lus
    let mut line = 1u64;
    let mut current: Vec<u8> = Vec::new();
    let (mut start_off, mut start_line) = (0u64, 1u64);
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escape = false;
    let mut in_value = false;
    let mut scalar = false; // valeur sans délimiteurs (nombre, littéral)
    let mut seen_open = !array; // pour un tableau : '[' de premier niveau rencontré
    let mut closed = false;
    let mut first_byte = true;
    let mut chunk_count = 0u64;

    loop {
        if em.cancelled() {
            return Err(ParseError::Cancelled);
        }
        let available = reader.fill_buf()?;
        if available.is_empty() {
            break;
        }
        let data = available.to_vec();
        reader.consume(data.len());
        chunk_count += 1;
        let mut i = 0;
        if first_byte && data.starts_with(&[0xEF, 0xBB, 0xBF]) {
            i = 3;
            offset += 3;
        }
        first_byte = false;
        while i < data.len() {
            let b = data[i];
            let pos = offset;
            offset += 1;
            i += 1;
            if !in_value {
                if b == b'\n' {
                    line += 1;
                }
                if b.is_ascii_whitespace() || closed {
                    if closed && !b.is_ascii_whitespace() {
                        return Err(ParseError::Malformed {
                            line,
                            byte_offset: pos,
                            message: "contenu après la fin du tableau".into(),
                        });
                    }
                    continue;
                }
                if !seen_open {
                    if b != b'[' {
                        return Err(ParseError::Malformed {
                            line,
                            byte_offset: pos,
                            message: "« [ » attendu au début du tableau".into(),
                        });
                    }
                    seen_open = true;
                    continue;
                }
                if array && b == b',' {
                    continue;
                }
                if array && b == b']' {
                    closed = true;
                    continue;
                }
                in_value = true;
                start_off = pos;
                start_line = line;
                current.clear();
                current.push(b);
                match b {
                    b'{' | b'[' => {
                        depth = 1;
                        scalar = false;
                    }
                    b'"' => {
                        in_string = true;
                        scalar = false;
                        depth = 0;
                    }
                    _ => scalar = true,
                }
                continue;
            }
            // Dans une valeur.
            if scalar && (b.is_ascii_whitespace() || b == b',' || b == b']') {
                in_value = false;
                em.record(&current, start_line, start_off)?;
                if b == b'\n' {
                    line += 1;
                }
                if array && b == b']' {
                    closed = true;
                }
                continue;
            }
            if b == b'\n' {
                line += 1;
            }
            if current.len() > MAX_RECORD_BYTES {
                return Err(ParseError::Malformed {
                    line: start_line,
                    byte_offset: start_off,
                    message: format!("enregistrement de plus de {MAX_RECORD_BYTES} octets"),
                });
            }
            current.push(b);
            if in_string {
                if escape {
                    escape = false;
                } else if b == b'\\' {
                    escape = true;
                } else if b == b'"' {
                    in_string = false;
                    if depth == 0 {
                        in_value = false;
                        em.record(&current, start_line, start_off)?;
                    }
                }
                continue;
            }
            match b {
                b'"' => in_string = true,
                b'{' | b'[' => depth += 1,
                b'}' | b']' => {
                    depth -= 1;
                    if depth == 0 {
                        in_value = false;
                        em.record(&current, start_line, start_off)?;
                    }
                }
                _ => {}
            }
        }
        em.report.bytes = offset;
        if chunk_count.is_multiple_of(16) {
            (em.ctx.on_progress)(offset);
        }
    }
    if in_value {
        if scalar {
            em.record(&current, start_line, start_off)?;
        } else {
            return Err(ParseError::Malformed {
                line: start_line,
                byte_offset: start_off,
                message: "enregistrement non terminé en fin de fichier".into(),
            });
        }
    }
    if array && !closed {
        return Err(ParseError::Malformed {
            line,
            byte_offset: offset,
            message: "tableau JSON non fermé (« ] » manquant)".into(),
        });
    }
    em.report.bytes = offset;
    (em.ctx.on_progress)(offset);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{detect, parse as parse_any, RecordError};
    use evidentyx_core::{EventKind, EvidenceId, Outcome};
    use std::sync::atomic::AtomicBool;

    fn run(format: Format, input: &str) -> (Result<ParseReport, ParseError>, Vec<NewEvent>) {
        let cancel = AtomicBool::new(false);
        let mut progress = |_| {};
        let mut events = Vec::new();
        let mut sink = |e: NewEvent| {
            events.push(e);
            Ok(())
        };
        let r = parse_any(
            format,
            input.as_bytes(),
            ParseContext {
                evidence_id: EvidenceId::new(9),
                reference_time: None,
                on_progress: &mut progress,
                cancel: &cancel,
            },
            &mut sink,
        );
        (r, events)
    }

    const JSONL: &str = "{\"ts\":\"2026-11-16T00:00:21Z\",\"src_ip\":\"198.51.100.40\",\"user\":\"admin\",\"action\":\"ssh_login\",\"status\":\"failed\"}\n\
\n\
pas du json\n\
{\"ts\":1763251300,\"msg\":\"Connexion\",\"host\":{\"name\":\"srv-01\"},\"tags\":[\"a\",\"b\"]}\r\n\
\"une chaîne\"\n";

    #[test]
    fn json_lines_avec_erreurs_par_ligne() {
        let (r, ev) = run(Format::JsonLines, JSONL);
        let r = r.unwrap();
        assert_eq!((r.records, r.events, r.errors), (4, 2, 2));
        assert_eq!(r.first_errors[0].line, 3);
        assert!(r.first_errors[0].message.starts_with("JSON invalide"));
        assert_eq!(
            r.first_errors[1],
            RecordError {
                line: 5,
                byte_offset: (JSONL.len() - "\"une chaîne\"\n".len()) as u64,
                message: "objet JSON attendu".into()
            }
        );

        let a = &ev[0];
        assert_eq!(a.source.byte_offset, 0);
        assert_eq!(a.source.line, Some(1));
        assert_eq!(a.kind, EventKind::Authentication);
        assert_eq!(a.outcome, Some(Outcome::Failure));
        let b = &ev[1];
        assert_eq!(b.source.line, Some(4));
        assert_eq!(
            &JSONL.as_bytes()[b.source.byte_offset as usize..][..1],
            b"{"
        );
        assert_eq!(
            b.host.as_deref(),
            Some("srv-01"),
            "champ imbriqué host.name"
        );
        assert_eq!(
            b.attributes.get("tags").map(String::as_str),
            Some("[\"a\",\"b\"]")
        );
        assert_eq!(b.message, "Connexion");
        assert!(b.time.is_some());
    }

    #[test]
    fn tableau_json_multiligne_et_decalages() {
        let input = "[\n  {\"message\": \"a }{ \\\" ] [\", \"n\": 1},\n  {\n    \"message\": \"b\"\n  }\n]\n";
        let (r, ev) = run(Format::JsonArray, input);
        let r = r.unwrap();
        assert_eq!((r.records, r.events, r.errors), (2, 2, 0));
        assert_eq!(ev[0].message, "a }{ \" ] [");
        assert_eq!(ev[0].source.line, Some(2));
        assert_eq!(ev[1].source.line, Some(3));
        assert_eq!(
            &input.as_bytes()[ev[1].source.byte_offset as usize..][..1],
            b"{"
        );
        assert_eq!(ev[1].source.record, Some(2));
    }

    #[test]
    fn tableau_mal_forme() {
        let (r, ev) = run(Format::JsonArray, "[{\"message\":\"a\"}, {\"message\": ");
        assert!(matches!(r, Err(ParseError::Malformed { .. })));
        assert_eq!(ev.len(), 1, "les événements déjà lus restent transmis");
        assert!(matches!(
            run(Format::JsonArray, "{}").0,
            Err(ParseError::Malformed { .. })
        ));
    }

    #[test]
    fn objets_concatenes() {
        let input = "{\n \"message\": \"a\"\n}\n{\"message\":\"b\"}";
        let (r, ev) = run(Format::JsonConcatenated, input);
        assert_eq!(r.unwrap().events, 2);
        assert_eq!(ev[1].source.line, Some(4));
    }

    #[test]
    fn detection() {
        assert_eq!(detect("x.ndjson", b""), Some(Format::JsonLines));
        assert_eq!(detect("x.json", b"  [ {"), Some(Format::JsonArray));
        assert_eq!(
            detect("x.log", b"{\"a\":1}\n{\"a\":2}\n{\"a\""),
            Some(Format::JsonLines)
        );
        assert_eq!(
            detect("x.json", b"{\n  \"a\": 1\n}\n"),
            Some(Format::JsonConcatenated)
        );
        assert_eq!(detect("x.csv", b"a,b,c\n1,2,3"), Some(Format::Csv));
        assert_eq!(
            detect("x.json", "\u{feff}[".as_bytes()),
            Some(Format::JsonArray)
        );
    }

    #[test]
    fn annulation() {
        let cancel = AtomicBool::new(true);
        let mut progress = |_| {};
        let mut sink = |_e: NewEvent| Ok(());
        let r = parse_any(
            Format::JsonLines,
            JSONL.as_bytes(),
            ParseContext {
                evidence_id: EvidenceId::new(1),
                reference_time: None,
                on_progress: &mut progress,
                cancel: &cancel,
            },
            &mut sink,
        );
        assert!(matches!(r, Err(ParseError::Cancelled)));
    }

    #[test]
    fn ligne_trop_longue_sautee() {
        let long = format!(
            "{{\"message\":\"{}\"}}\n{{\"message\":\"ok\"}}\n",
            "x".repeat(MAX_RECORD_BYTES + 10)
        );
        let (r, ev) = run(Format::JsonLines, &long);
        let r = r.unwrap();
        assert_eq!((r.errors, r.events), (1, 1));
        assert_eq!(ev[0].source.line, Some(2));
    }
}
