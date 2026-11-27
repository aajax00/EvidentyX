//! Moteur de détection : règles déclaratives et versionnées, évaluées de façon
//! déterministe sur les événements d'une enquête.
//!
//! Une règle sélectionne des événements (catégorie, issue), les regroupe par
//! adresse source et cherche une fenêtre de temps glissante où une mesure
//! (nombre d'événements, comptes distincts, cibles distinctes) atteint un
//! seuil. Chaque détection garde les événements qui la justifient.

use crate::error::Result;
use crate::Storage;
use evidentyx_core::{EventKind, InvestigationId, Outcome};
use std::collections::{HashMap, VecDeque};

/// Événements justificatifs conservés au plus par détection.
const MAX_EVIDENCE_EVENTS: usize = 50;

/// Gravité d'une détection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Ce qui est compté dans la fenêtre de temps.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Measure {
    /// Nombre d'événements.
    Events,
    /// Comptes utilisateurs distincts.
    DistinctUsers,
    /// Cibles distinctes (adresse et port de destination).
    DistinctTargets,
}

/// Règle de détection. Toute modification de sa logique change sa version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rule {
    pub id: &'static str,
    pub version: u32,
    pub title: &'static str,
    pub description: &'static str,
    pub severity: Severity,
    pub kind: EventKind,
    pub outcome: Option<Outcome>,
    pub measure: Measure,
    pub threshold: u32,
    pub window_secs: i64,
}

/// Règles fournies avec l'application.
pub const RULES: &[Rule] = &[
    Rule {
        id: "auth-echecs-repetes",
        version: 1,
        title: "Échecs d'authentification répétés",
        description:
            "Au moins 10 échecs d'authentification depuis une même adresse en 5 minutes : \
                      tentative de force brute probable.",
        severity: Severity::High,
        kind: EventKind::Authentication,
        outcome: Some(Outcome::Failure),
        measure: Measure::Events,
        threshold: 10,
        window_secs: 300,
    },
    Rule {
        id: "auth-pulverisation",
        version: 1,
        title: "Pulvérisation de mots de passe",
        description: "Échecs d'authentification sur au moins 5 comptes différents depuis une même \
                      adresse en 10 minutes : essai de mots de passe sur plusieurs comptes.",
        severity: Severity::High,
        kind: EventKind::Authentication,
        outcome: Some(Outcome::Failure),
        measure: Measure::DistinctUsers,
        threshold: 5,
        window_secs: 600,
    },
    Rule {
        id: "reseau-balayage",
        version: 1,
        title: "Balayage réseau",
        description: "Une même adresse contacte au moins 20 cibles différentes (adresse et port) \
                      en 1 minute : balayage de ports ou d'hôtes probable.",
        severity: Severity::Medium,
        kind: EventKind::Network,
        outcome: None,
        measure: Measure::DistinctTargets,
        threshold: 20,
        window_secs: 60,
    },
];

/// Détection : une règle déclenchée par une adresse sur une période.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Detection {
    pub rule: Rule,
    /// Adresse source en cause.
    pub source_ip: String,
    pub first_us: i64,
    pub last_us: i64,
    /// Événements de la période retenus par la règle.
    pub events: u64,
    /// Valeur maximale de la mesure atteinte dans une fenêtre.
    pub peak: u64,
    /// Premiers événements justificatifs (identifiants), au plus 50.
    pub event_ids: Vec<i64>,
}

struct Row {
    id: i64,
    key: String,
    time: i64,
    item: Option<String>,
}

/// Évaluation d'une règle sur des événements triés par adresse puis par heure.
/// Une détection couvre une période continue où le seuil est atteint.
fn evaluate(rule: &Rule, rows: impl IntoIterator<Item = Row>) -> Vec<Detection> {
    let window_us = rule.window_secs * 1_000_000;
    let mut out: Vec<Detection> = Vec::new();
    let mut key = String::new();
    let mut win: VecDeque<Row> = VecDeque::new();
    let mut distinct: HashMap<String, u32> = HashMap::new();
    let mut open = false;
    for row in rows {
        if row.key != key {
            key.clone_from(&row.key);
            win.clear();
            distinct.clear();
            open = false;
        }
        while win.front().is_some_and(|f| row.time - f.time > window_us) {
            let f = win.pop_front().expect("fenêtre non vide");
            if let Some(item) = f.item {
                if let Some(n) = distinct.get_mut(&item) {
                    *n -= 1;
                    if *n == 0 {
                        distinct.remove(&item);
                    }
                }
            }
        }
        if let Some(item) = &row.item {
            *distinct.entry(item.clone()).or_default() += 1;
        }
        let (time, id) = (row.time, row.id);
        win.push_back(row);
        let value = match rule.measure {
            Measure::Events => win.len(),
            Measure::DistinctUsers | Measure::DistinctTargets => distinct.len(),
        } as u64;
        if value < u64::from(rule.threshold) {
            open = false;
            continue;
        }
        if open {
            let d = out.last_mut().expect("détection ouverte");
            d.last_us = time;
            d.events += 1;
            d.peak = d.peak.max(value);
            if d.event_ids.len() < MAX_EVIDENCE_EVENTS {
                d.event_ids.push(id);
            }
        } else {
            open = true;
            out.push(Detection {
                rule: *rule,
                source_ip: key.clone(),
                first_us: win.front().map_or(time, |f| f.time),
                last_us: time,
                events: win.len() as u64,
                peak: value,
                event_ids: win.iter().take(MAX_EVIDENCE_EVENTS).map(|r| r.id).collect(),
            });
        }
    }
    out
}

impl Storage {
    /// Applique toutes les règles fournies aux événements d'une enquête.
    /// Les détections sont triées par gravité décroissante puis par heure.
    pub fn detect(&self, investigation_id: InvestigationId) -> Result<Vec<Detection>> {
        self.investigation(investigation_id)?;
        let mut all = Vec::new();
        for rule in RULES {
            let item = match rule.measure {
                Measure::Events => "NULL",
                Measure::DistinctUsers => "user_name",
                Measure::DistinctTargets => {
                    "destination_ip || ':' || coalesce(destination_port, '')"
                }
            };
            let outcome = rule.outcome.map_or(String::new(), |o| {
                format!(" AND outcome = '{}'", o.as_str())
            });
            // Valeurs insérées issues de constantes du programme (catégorie, issue), pas d'une saisie.
            let mut stmt = self.conn.prepare(&format!(
                "SELECT id, source_ip, time_us, {item} FROM events
                 WHERE investigation_id = ?1 AND kind = '{}'{outcome}
                   AND source_ip IS NOT NULL AND time_us IS NOT NULL
                 ORDER BY source_ip, time_us, id",
                rule.kind.as_str()
            ))?;
            let rows = stmt
                .query_map([investigation_id.get()], |r| {
                    Ok(Row {
                        id: r.get(0)?,
                        key: r.get(1)?,
                        time: r.get(2)?,
                        item: r.get(3)?,
                    })
                })?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            all.extend(evaluate(rule, rows));
        }
        all.sort_by(|a, b| {
            b.rule
                .severity
                .cmp(&a.rule.severity)
                .then(a.first_us.cmp(&b.first_us))
        });
        Ok(all)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(spec: &[(&str, i64, Option<&str>)]) -> Vec<Row> {
        spec.iter()
            .enumerate()
            .map(|(i, (k, s, item))| Row {
                id: i as i64 + 1,
                key: (*k).into(),
                time: s * 1_000_000,
                item: item.map(Into::into),
            })
            .collect()
    }

    #[test]
    fn echecs_repetes_au_seuil_et_dans_la_fenetre() {
        let rule = RULES[0];
        // 10 échecs en 90 s : une détection ; 9 échecs : aucune.
        let ten: Vec<_> = (0..10).map(|i| ("1.2.3.4", i * 10, None)).collect();
        let d = evaluate(&rule, rows(&ten));
        assert_eq!(d.len(), 1);
        assert_eq!(
            (d[0].events, d[0].first_us, d[0].last_us),
            (10, 0, 90_000_000)
        );
        assert_eq!(d[0].event_ids.len(), 10);
        assert!(evaluate(&rule, rows(&ten[..9])).is_empty());
        // 10 échecs étalés sur plus de 5 minutes : jamais 10 dans la fenêtre.
        let spread: Vec<_> = (0..10).map(|i| ("1.2.3.4", i * 40, None)).collect();
        assert!(evaluate(&rule, rows(&spread)).is_empty());
        // Deux adresses ne s'additionnent pas.
        let mixed: Vec<_> = (0..10)
            .map(|i| (if i < 5 { "a" } else { "b" }, i, None))
            .collect();
        assert!(evaluate(&rule, rows(&mixed)).is_empty());
    }

    #[test]
    fn une_periode_continue_donne_une_seule_detection() {
        let rule = RULES[0];
        let burst: Vec<_> = (0..25).map(|i| ("1.2.3.4", i, None)).collect();
        let d = evaluate(&rule, rows(&burst));
        assert_eq!((d.len(), d[0].events, d[0].peak), (1, 25, 25));
        // Une nouvelle rafale une heure plus tard : deuxième détection.
        let mut two = burst.clone();
        two.extend((0..10).map(|i| ("1.2.3.4", 3600 + i, None)));
        assert_eq!(evaluate(&rule, rows(&two)).len(), 2);
    }

    #[test]
    fn pulverisation_compte_les_comptes_distincts() {
        let rule = RULES[1];
        let users = ["root", "admin", "deploy", "www", "oracle"];
        let spray: Vec<_> = users
            .iter()
            .enumerate()
            .map(|(i, u)| ("1.2.3.4", i as i64 * 30, Some(*u)))
            .collect();
        assert_eq!(evaluate(&rule, rows(&spray)).len(), 1);
        // Même compte répété : pas de pulvérisation.
        let same: Vec<_> = (0..20).map(|i| ("1.2.3.4", i, Some("root"))).collect();
        assert!(evaluate(&rule, rows(&same)).is_empty());
    }

    #[test]
    fn balayage_compte_les_cibles_distinctes() {
        let rule = RULES[2];
        let targets: Vec<String> = (0..20).map(|p| format!("10.0.0.9:{p}")).collect();
        let scan: Vec<_> = targets
            .iter()
            .enumerate()
            .map(|(i, t)| ("1.2.3.4", i as i64, Some(t.as_str())))
            .collect();
        let d = evaluate(&rule, rows(&scan));
        assert_eq!((d.len(), d[0].peak), (1, 20));
        // Les mêmes cibles sur 20 minutes : jamais 20 en 1 minute.
        let slow: Vec<_> = targets
            .iter()
            .enumerate()
            .map(|(i, t)| ("1.2.3.4", i as i64 * 60, Some(t.as_str())))
            .collect();
        assert!(evaluate(&rule, rows(&slow)).is_empty());
    }

    #[test]
    fn detection_sur_une_vraie_base() {
        use crate::NewEvidence;
        use evidentyx_core::{
            EventTime, EvidenceId, InvestigationDescription, InvestigationName, NewEvent,
            Sha256Digest, SourceRef, TimeBasis, TimePrecision, Timestamp,
        };
        let mut db = Storage::open_in_memory().unwrap();
        let inv = db
            .create_investigation(
                &InvestigationName::parse("T").unwrap(),
                &InvestigationDescription::default(),
            )
            .unwrap()
            .id;
        let ev = db
            .add_evidence(
                inv,
                &NewEvidence {
                    file_name: "auth.log".into(),
                    path: "/p/auth.log".into(),
                    size_bytes: 1,
                    sha256: Sha256Digest::from_bytes([1; 32]),
                    file_modified_at: Some(Timestamp::from_unix_millis(0)),
                },
            )
            .unwrap()
            .id;
        let events: Vec<NewEvent> = (0..12u64)
            .map(|i| {
                let source = SourceRef {
                    evidence_id: EvidenceId::new(ev.get()),
                    byte_offset: i,
                    line: Some(i + 1),
                    record: None,
                };
                let mut e =
                    NewEvent::new(source, EventKind::Authentication, "Failed password", "brut");
                e.time = Some(EventTime {
                    utc_micros: i as i64 * 5_000_000,
                    precision: TimePrecision::Second,
                    basis: TimeBasis::Explicit,
                });
                e.source_ip = Some("203.0.113.7".parse().unwrap());
                e.outcome = Some(Outcome::Failure);
                e.user = Some(format!("compte{}", i % 6));
                e
            })
            .collect();
        db.insert_events(ev, &events).unwrap();
        let found = db.detect(inv).unwrap();
        let ids: Vec<_> = found.iter().map(|d| d.rule.id).collect();
        assert_eq!(ids, vec!["auth-echecs-repetes", "auth-pulverisation"]);
        assert_eq!(
            (found[0].source_ip.as_str(), found[0].events),
            ("203.0.113.7", 12)
        );
    }

    #[test]
    fn identifiants_de_regles_uniques() {
        let mut ids: Vec<_> = RULES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), RULES.len());
    }
}
