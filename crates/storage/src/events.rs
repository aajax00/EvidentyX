//! Persistance des événements canoniques.

use crate::error::{Result, StorageError};
use crate::Storage;
use evidentyx_core::{
    Event, EventId, EventKind, EventTime, EvidenceId, InvestigationId, NewEvent, SourceRef,
};
use rusqlite::{params, Row, Transaction};
use std::collections::BTreeMap;
use std::net::IpAddr;

/// Bilan d'une insertion par lot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InsertReport {
    /// Événements enregistrés.
    pub inserted: u64,
    /// Événements ignorés car leur enregistrement source l'était déjà (réimport).
    pub duplicates: u64,
}

/// Page de la chronologie d'une enquête.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventPage {
    pub events: Vec<Event>,
    /// Curseur à passer pour la page suivante ; `None` en fin de chronologie.
    pub next: Option<EventId>,
}

const COLUMNS: &str = "id, evidence_id, byte_offset, line, record, time_us, time_precision, \
                       time_basis, kind, host, source_ip, source_port, destination_ip, \
                       destination_port, user_name, action, outcome, message, raw, raw_truncated, attributes";

fn to_i64(value: u64, what: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| StorageError::CorruptRow {
        table: "events",
        id: 0,
        detail: format!("{what} hors limites : {value}"),
    })
}

/// Lit une ligne et revalide ses champs.
fn from_row(row: &Row<'_>) -> rusqlite::Result<std::result::Result<Event, StorageError>> {
    let id: i64 = row.get(0)?;
    let corrupt = |detail: String| StorageError::CorruptRow {
        table: "events",
        id,
        detail,
    };
    let get_u64 = |idx: usize| -> rusqlite::Result<Option<u64>> {
        Ok(row
            .get::<_, Option<i64>>(idx)?
            .and_then(|v| u64::try_from(v).ok()))
    };
    let evidence_id: i64 = row.get(1)?;
    let byte_offset = get_u64(2)?;
    let line = get_u64(3)?;
    let record = get_u64(4)?;
    let time_us: Option<i64> = row.get(5)?;
    let precision: Option<String> = row.get(6)?;
    let basis: Option<String> = row.get(7)?;
    let kind: String = row.get(8)?;
    let host: Option<String> = row.get(9)?;
    let source_ip: Option<String> = row.get(10)?;
    let source_port: Option<i64> = row.get(11)?;
    let destination_ip: Option<String> = row.get(12)?;
    let destination_port: Option<i64> = row.get(13)?;
    let user: Option<String> = row.get(14)?;
    let action: Option<String> = row.get(15)?;
    let outcome: Option<String> = row.get(16)?;
    let message: String = row.get(17)?;
    let raw: String = row.get(18)?;
    let raw_truncated: i64 = row.get(19)?;
    let attributes: String = row.get(20)?;

    Ok((|| {
        let ip = |v: Option<String>| -> std::result::Result<Option<IpAddr>, StorageError> {
            v.map(|s| {
                s.parse::<IpAddr>()
                    .map_err(|_| corrupt(format!("adresse IP invalide : {s}")))
            })
            .transpose()
        };
        let port = |v: Option<i64>| v.and_then(|p| u16::try_from(p).ok());
        let time = match (time_us, precision, basis) {
            (Some(utc_micros), Some(p), Some(b)) => Some(EventTime {
                utc_micros,
                precision: p
                    .parse()
                    .map_err(|e: evidentyx_core::CoreError| corrupt(e.to_string()))?,
                basis: b
                    .parse()
                    .map_err(|e: evidentyx_core::CoreError| corrupt(e.to_string()))?,
            }),
            (None, None, None) => None,
            _ => return Err(corrupt("horodatage incomplet".into())),
        };
        Ok(Event {
            id: EventId::new(id),
            event: NewEvent {
                source: SourceRef {
                    evidence_id: EvidenceId::new(evidence_id),
                    byte_offset: byte_offset.ok_or_else(|| corrupt("décalage invalide".into()))?,
                    line,
                    record,
                },
                time,
                kind: kind
                    .parse::<EventKind>()
                    .map_err(|e| corrupt(e.to_string()))?,
                host,
                source_ip: ip(source_ip)?,
                source_port: port(source_port),
                destination_ip: ip(destination_ip)?,
                destination_port: port(destination_port),
                user,
                action,
                outcome: outcome
                    .map(|o| {
                        o.parse()
                            .map_err(|e: evidentyx_core::CoreError| corrupt(e.to_string()))
                    })
                    .transpose()?,
                message,
                raw,
                raw_truncated: raw_truncated != 0,
                attributes: serde_json::from_str(&attributes)
                    .map_err(|e| corrupt(format!("attributs illisibles : {e}")))?,
            },
        })
    })())
}

fn insert_one(
    tx: &Transaction<'_>,
    investigation_id: InvestigationId,
    e: &NewEvent,
) -> Result<bool> {
    let (time_us, precision, basis) = match e.time {
        Some(t) => (
            Some(t.utc_micros),
            Some(t.precision.as_str()),
            Some(t.basis.as_str()),
        ),
        None => (None, None, None),
    };
    let attributes = serde_json::to_string(&e.attributes).map_err(|err| {
        StorageError::Domain(evidentyx_core::CoreError::InvalidEvent(err.to_string()))
    })?;
    let mut stmt = tx.prepare_cached(
        "INSERT INTO events (investigation_id, evidence_id, byte_offset, line, record, time_us,
                             time_precision, time_basis, kind, host, source_ip, source_port,
                             destination_ip, destination_port, user_name, action, outcome,
                             message, raw, raw_truncated, attributes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
         ON CONFLICT (evidence_id, byte_offset) DO NOTHING",
    )?;
    let inserted = stmt.execute(params![
        investigation_id.get(),
        e.source.evidence_id.get(),
        to_i64(e.source.byte_offset, "décalage")?,
        e.source.line.map(|v| to_i64(v, "ligne")).transpose()?,
        e.source
            .record
            .map(|v| to_i64(v, "enregistrement"))
            .transpose()?,
        time_us,
        precision,
        basis,
        e.kind.as_str(),
        e.host,
        e.source_ip.map(|ip| ip.to_string()),
        e.source_port,
        e.destination_ip.map(|ip| ip.to_string()),
        e.destination_port,
        e.user,
        e.action,
        e.outcome.map(|o| o.as_str()),
        e.message,
        e.raw,
        e.raw_truncated,
        attributes,
    ])?;
    Ok(inserted > 0)
}

impl Storage {
    /// Enregistre un lot d'événements d'une même preuve, en une transaction.
    /// Chaque événement est validé ; un enregistrement source déjà importé
    /// (même preuve, même décalage) est ignoré et compté comme doublon.
    pub fn insert_events(
        &mut self,
        evidence_id: EvidenceId,
        events: &[NewEvent],
    ) -> Result<InsertReport> {
        let evidence = self.evidence(evidence_id)?;
        self.investigation(evidence.investigation_id)?
            .ensure_editable()?;
        for e in events {
            e.validate()?;
            if e.source.evidence_id != evidence_id {
                return Err(StorageError::Domain(
                    evidentyx_core::CoreError::InvalidEvent(
                        "événement rattaché à une autre preuve".into(),
                    ),
                ));
            }
        }
        let tx = self.conn.transaction()?;
        let mut report = InsertReport::default();
        for e in events {
            if insert_one(&tx, evidence.investigation_id, e)? {
                report.inserted += 1;
            } else {
                report.duplicates += 1;
            }
        }
        tx.commit()?;
        Ok(report)
    }

    /// Nombre d'événements d'une enquête.
    pub fn count_events(&self, investigation_id: InvestigationId) -> Result<u64> {
        self.investigation(investigation_id)?;
        let n: i64 = self.conn.query_row(
            "SELECT count(*) FROM events WHERE investigation_id = ?1",
            [investigation_id.get()],
            |r| r.get(0),
        )?;
        Ok(u64::try_from(n).unwrap_or(0))
    }

    /// Nombre d'événements extraits de chaque preuve d'une enquête.
    pub fn event_counts_by_evidence(
        &self,
        investigation_id: InvestigationId,
    ) -> Result<BTreeMap<EvidenceId, u64>> {
        self.investigation(investigation_id)?;
        let mut stmt = self.conn.prepare(
            "SELECT evidence_id, count(*) FROM events
             WHERE evidence_id IN (SELECT id FROM evidence WHERE investigation_id = ?1)
             GROUP BY evidence_id",
        )?;
        let rows = stmt.query_map([investigation_id.get()], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?))
        })?;
        let mut counts = BTreeMap::new();
        for row in rows {
            let (id, n) = row?;
            counts.insert(EvidenceId::new(id), u64::try_from(n).unwrap_or(0));
        }
        Ok(counts)
    }

    /// Chronologie d'une enquête, par pages : ordre chronologique, événements
    /// sans heure à la fin, puis ordre d'insertion (déterministe).
    pub fn list_events(
        &self,
        investigation_id: InvestigationId,
        after: Option<EventId>,
        limit: u32,
    ) -> Result<EventPage> {
        self.investigation(investigation_id)?;
        let limit = limit.clamp(1, 5000);
        // Clé de tri : (sans heure ?, heure, id). Le curseur reprend après l'événement donné.
        let cursor = match after {
            None => None,
            Some(id) => Some(self.conn.query_row(
                "SELECT time_us IS NULL, coalesce(time_us, 0), id FROM events
                 WHERE id = ?1 AND investigation_id = ?2",
                params![id.get(), investigation_id.get()],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, i64>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                },
            )?),
        };
        let (c_null, c_time, c_id) = cursor.unwrap_or((-1, 0, 0));
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM events
             WHERE investigation_id = ?1
               AND (?2 < 0 OR ((time_us IS NULL), coalesce(time_us, 0), id) > (?2, ?3, ?4))
             ORDER BY time_us IS NULL, time_us, id
             LIMIT ?5"
        ))?;
        let rows = stmt.query_map(
            params![
                investigation_id.get(),
                c_null,
                c_time,
                c_id,
                i64::from(limit) + 1
            ],
            from_row,
        )?;
        let mut events = rows.map(|r| r?).collect::<Result<Vec<_>>>()?;
        let next = if events.len() > limit as usize {
            events.truncate(limit as usize);
            events.last().map(|e| e.id)
        } else {
            None
        };
        Ok(EventPage { events, next })
    }

    /// Supprime les événements extraits d'une preuve (avant une réanalyse).
    pub fn delete_events_for_evidence(&mut self, evidence_id: EvidenceId) -> Result<u64> {
        let evidence = self.evidence(evidence_id)?;
        self.investigation(evidence.investigation_id)?
            .ensure_editable()?;
        let n = self.conn.execute(
            "DELETE FROM events WHERE evidence_id = ?1",
            [evidence_id.get()],
        )?;
        Ok(n as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::NewEvidence;
    use evidentyx_core::{
        CoreError, InvestigationDescription, InvestigationName, InvestigationStatus, Outcome,
        Sha256Digest, TimeBasis, TimePrecision, Timestamp,
    };

    fn setup() -> (Storage, InvestigationId, EvidenceId) {
        let mut db = Storage::open_in_memory().unwrap();
        let inv = db
            .create_investigation(
                &InvestigationName::parse("Test").unwrap(),
                &InvestigationDescription::default(),
            )
            .unwrap();
        let ev = db
            .add_evidence(
                inv.id,
                &NewEvidence {
                    file_name: "auth.log".into(),
                    path: "/preuves/auth.log".into(),
                    size_bytes: 10,
                    sha256: Sha256Digest::from_bytes([7; 32]),
                    file_modified_at: Some(Timestamp::from_unix_millis(0)),
                },
            )
            .unwrap();
        (db, inv.id, ev.id)
    }

    fn event(ev: EvidenceId, offset: u64, time: Option<i64>) -> NewEvent {
        let mut e = NewEvent::new(
            SourceRef {
                evidence_id: ev,
                byte_offset: offset,
                line: Some(offset + 1),
                record: None,
            },
            EventKind::Authentication,
            format!("événement {offset}"),
            &format!("ligne brute {offset}"),
        );
        e.time = time.map(|t| EventTime {
            utc_micros: t,
            precision: TimePrecision::Second,
            basis: TimeBasis::InferredYear,
        });
        e
    }

    #[test]
    fn aller_retour_complet() {
        let (mut db, inv, ev) = setup();
        let mut e = event(ev, 0, Some(1_700_000_000_000_000));
        e.host = Some("srv-web-01".into());
        e.source_ip = Some("203.0.113.7".parse().unwrap());
        e.source_port = Some(51234);
        e.destination_ip = Some("2001:db8::1".parse().unwrap());
        e.destination_port = Some(22);
        e.user = Some("root".into());
        e.action = Some("ssh-login".into());
        e.outcome = Some(Outcome::Failure);
        e.attributes.insert("pid".into(), "4242".into());
        e.attributes.insert("process".into(), "sshd".into());

        assert_eq!(
            db.insert_events(ev, std::slice::from_ref(&e))
                .unwrap()
                .inserted,
            1
        );
        let page = db.list_events(inv, None, 10).unwrap();
        assert_eq!(page.events.len(), 1);
        assert_eq!(page.events[0].event, e);
        assert_eq!(page.next, None);
    }

    #[test]
    fn reimport_sans_doublon() {
        let (mut db, inv, ev) = setup();
        let batch = [event(ev, 0, Some(10)), event(ev, 50, Some(20))];
        assert_eq!(
            db.insert_events(ev, &batch).unwrap(),
            InsertReport {
                inserted: 2,
                duplicates: 0
            }
        );
        assert_eq!(
            db.insert_events(ev, &batch).unwrap(),
            InsertReport {
                inserted: 0,
                duplicates: 2
            }
        );
        assert_eq!(db.count_events(inv).unwrap(), 2);
        assert_eq!(db.event_counts_by_evidence(inv).unwrap().get(&ev), Some(&2));
    }

    #[test]
    fn chronologie_paginee_et_deterministe() {
        let (mut db, inv, ev) = setup();
        // Insérés dans le désordre ; deux sans heure ; deux au même instant.
        let batch = [
            event(ev, 0, Some(300)),
            event(ev, 10, None),
            event(ev, 20, Some(100)),
            event(ev, 30, Some(300)),
            event(ev, 40, None),
            event(ev, 50, Some(200)),
        ];
        db.insert_events(ev, &batch).unwrap();
        let mut offsets = Vec::new();
        let mut after = None;
        loop {
            let page = db.list_events(inv, after, 2).unwrap();
            offsets.extend(page.events.iter().map(|e| e.event.source.byte_offset));
            match page.next {
                Some(n) => after = Some(n),
                None => break,
            }
        }
        assert_eq!(offsets, [20, 50, 0, 30, 10, 40]);
    }

    #[test]
    fn validation_et_lecture_seule() {
        let (mut db, inv, ev) = setup();
        let mut bad = event(ev, 0, None);
        bad.message = " ".into();
        assert!(matches!(
            db.insert_events(ev, &[bad]),
            Err(StorageError::Domain(CoreError::InvalidEvent(_)))
        ));
        assert_eq!(db.count_events(inv).unwrap(), 0, "lot refusé en entier");

        db.set_investigation_status(inv, InvestigationStatus::Archived)
            .unwrap();
        assert!(matches!(
            db.insert_events(ev, &[event(ev, 0, None)]),
            Err(StorageError::Domain(CoreError::ArchivedReadOnly))
        ));
    }

    #[test]
    fn suppression_en_cascade() {
        let (mut db, inv, ev) = setup();
        let mut e = event(ev, 0, None);
        e.attributes.insert("k".into(), "v".into());
        db.insert_events(ev, &[e, event(ev, 5, None)]).unwrap();
        assert_eq!(db.delete_events_for_evidence(ev).unwrap(), 2);
        db.insert_events(ev, &[event(ev, 0, None)]).unwrap();
        db.remove_evidence(ev).unwrap();
        assert_eq!(db.count_events(inv).unwrap(), 0);
    }
}
