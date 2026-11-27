//! Chronologie d'une enquête : événements par position, pour une liste virtualisée.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{Event, EvidenceId, InvestigationId};
use evidentyx_storage::{EventFilter, FlowKey, GroupBy, TimeQuality};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Miroir TypeScript : `TimelineEvent` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventDto {
    pub id: i64,
    pub evidence_id: i64,
    pub byte_offset: u64,
    pub line: Option<u64>,
    pub record: Option<u64>,
    /// Microsecondes Unix UTC.
    pub time_us: Option<i64>,
    pub time_precision: Option<&'static str>,
    pub time_basis: Option<&'static str>,
    pub kind: &'static str,
    pub host: Option<String>,
    pub source_ip: Option<String>,
    pub source_port: Option<u16>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<u16>,
    pub user: Option<String>,
    pub action: Option<String>,
    pub outcome: Option<&'static str>,
    pub message: String,
    pub raw: String,
    pub raw_truncated: bool,
    pub attributes: BTreeMap<String, String>,
}

impl From<Event> for EventDto {
    fn from(e: Event) -> Self {
        let n = e.event;
        Self {
            id: e.id.get(),
            evidence_id: n.source.evidence_id.get(),
            byte_offset: n.source.byte_offset,
            line: n.source.line,
            record: n.source.record,
            time_us: n.time.map(|t| t.utc_micros),
            time_precision: n.time.map(|t| t.precision.as_str()),
            time_basis: n.time.map(|t| t.basis.as_str()),
            kind: n.kind.as_str(),
            host: n.host,
            source_ip: n.source_ip.map(|a| a.to_string()),
            source_port: n.source_port,
            destination_ip: n.destination_ip.map(|a| a.to_string()),
            destination_port: n.destination_port,
            user: n.user,
            action: n.action,
            outcome: n.outcome.map(|o| o.as_str()),
            message: n.message,
            raw: n.raw,
            raw_truncated: n.raw_truncated,
            attributes: n.attributes,
        }
    }
}

/// Miroir TypeScript : `EventFilter` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FilterDto {
    pub from_us: Option<i64>,
    pub to_us: Option<i64>,
    pub kind: Option<String>,
    pub evidence_id: Option<i64>,
    pub outcome: Option<String>,
    /// "uncertain" ou "missing".
    pub time: Option<String>,
    pub ip: Option<String>,
    pub flow: Option<FlowDto>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub text: Option<String>,
}

/// Flux réseau exact (pivot depuis un regroupement).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FlowDto {
    pub source_ip: String,
    pub destination_ip: String,
    pub destination_port: Option<u16>,
}

/// Miroir TypeScript : `EventGroup` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupDto {
    pub key: String,
    pub destination_ip: Option<String>,
    pub destination_port: Option<u16>,
    pub events: u64,
    pub failures: u64,
    pub first_us: Option<i64>,
    pub last_us: Option<i64>,
}

impl TryFrom<FilterDto> for EventFilter {
    type Error = CommandError;

    fn try_from(f: FilterDto) -> Result<Self, CommandError> {
        Ok(EventFilter {
            from_us: f.from_us,
            to_us: f.to_us,
            kind: f
                .kind
                .map(|k| k.parse())
                .transpose()
                .map_err(CommandError::from)?,
            evidence_id: f.evidence_id.map(EvidenceId::new),
            time: match f.time.as_deref() {
                None => None,
                Some("uncertain") => Some(TimeQuality::Uncertain),
                Some("missing") => Some(TimeQuality::Missing),
                Some(other) => {
                    return Err(CommandError::invalid_argument(format!(
                        "Critère d'heure inconnu : « {other} »"
                    )))
                }
            },
            outcome: f
                .outcome
                .map(|o| o.parse())
                .transpose()
                .map_err(CommandError::from)?,
            ip: f.ip,
            flow: f.flow.map(|f| FlowKey {
                source_ip: f.source_ip,
                destination_ip: f.destination_ip,
                destination_port: f.destination_port,
            }),
            user: f.user,
            host: f.host,
            text: f.text,
        })
    }
}

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn events_count(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
    filter: FilterDto,
) -> Result<u64, CommandError> {
    let filter = EventFilter::try_from(filter)?;
    state.with_storage(|s| Ok(s.count_filtered(InvestigationId::new(investigation_id), &filter)?))
}

#[tauri::command(async)]
pub fn events_window(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
    filter: FilterDto,
    offset: u64,
    limit: u32,
) -> Result<Vec<EventDto>, CommandError> {
    let filter = EventFilter::try_from(filter)?;
    state.with_storage(|s| {
        Ok(s.events_window(
            InvestigationId::new(investigation_id),
            &filter,
            offset,
            limit,
        )?
        .into_iter()
        .map(Into::into)
        .collect())
    })
}

/// Miroir TypeScript : `EntityLink` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntityLinkDto {
    pub from_kind: &'static str,
    pub from: String,
    pub to_kind: &'static str,
    pub to: String,
    pub events: u64,
}

/// Liens les plus fréquents entre entités, dans les filtres en cours.
#[tauri::command(async)]
pub fn entity_links(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
    filter: FilterDto,
    limit: u32,
) -> Result<Vec<EntityLinkDto>, CommandError> {
    let filter = EventFilter::try_from(filter)?;
    state.with_storage(|s| {
        Ok(
            s.entity_links(InvestigationId::new(investigation_id), &filter, limit)?
                .into_iter()
                .map(|l| EntityLinkDto {
                    from_kind: l.from_kind,
                    from: l.from,
                    to_kind: l.to_kind,
                    to: l.to,
                    events: l.events,
                })
                .collect(),
        )
    })
}

/// Regroupements : `by` vaut "ip", "user", "host" ou "flow".
#[tauri::command(async)]
pub fn event_groups(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
    filter: FilterDto,
    by: String,
    limit: u32,
) -> Result<Vec<GroupDto>, CommandError> {
    let filter = EventFilter::try_from(filter)?;
    let by = match by.as_str() {
        "ip" => GroupBy::Ip,
        "user" => GroupBy::User,
        "host" => GroupBy::Host,
        "flow" => GroupBy::Flow,
        other => {
            return Err(CommandError::invalid_argument(format!(
                "Regroupement inconnu : « {other} »"
            )))
        }
    };
    state.with_storage(|s| {
        Ok(
            s.groups(InvestigationId::new(investigation_id), &filter, by, limit)?
                .into_iter()
                .map(|g| GroupDto {
                    key: g.key,
                    destination_ip: g.destination_ip,
                    destination_port: g.destination_port,
                    events: g.events,
                    failures: g.failures,
                    first_us: g.first_us,
                    last_us: g.last_us,
                })
                .collect(),
        )
    })
}

/// Miroir TypeScript : `Detection` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectionDto {
    pub rule_id: &'static str,
    pub rule_version: u32,
    pub title: &'static str,
    pub description: &'static str,
    /// "low", "medium", "high" ou "critical".
    pub severity: &'static str,
    pub source_ip: String,
    /// Microsecondes Unix UTC.
    pub first_us: i64,
    pub last_us: i64,
    pub events: u64,
    pub peak: u64,
    pub event_ids: Vec<i64>,
}

/// Applique les règles de détection fournies aux événements d'une enquête.
#[tauri::command(async)]
pub fn run_detections(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
) -> Result<Vec<DetectionDto>, CommandError> {
    state.with_storage(|s| {
        Ok(s.detect(InvestigationId::new(investigation_id))?
            .into_iter()
            .map(|d| DetectionDto {
                rule_id: d.rule.id,
                rule_version: d.rule.version,
                title: d.rule.title,
                description: d.rule.description,
                severity: d.rule.severity.as_str(),
                source_ip: d.source_ip,
                first_us: d.first_us,
                last_us: d.last_us,
                events: d.events,
                peak: d.peak,
                event_ids: d.event_ids,
            })
            .collect())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtre_valide_a_la_frontiere() {
        let ok = FilterDto {
            kind: Some("network".into()),
            outcome: Some("failure".into()),
            ..Default::default()
        };
        let f = EventFilter::try_from(ok).unwrap();
        assert_eq!(f.kind, Some(evidentyx_core::EventKind::Network));
        let bad = FilterDto {
            kind: Some("inconnu".into()),
            ..Default::default()
        };
        assert_eq!(EventFilter::try_from(bad).unwrap_err().code, "validation");
        let bad = FilterDto {
            time: Some("parfois".into()),
            ..Default::default()
        };
        assert_eq!(
            EventFilter::try_from(bad).unwrap_err().code,
            "invalid_argument"
        );
        let json = serde_json::json!({ "ip": "10.", "inconnu": 1 });
        assert!(serde_json::from_value::<FilterDto>(json).is_err());
    }

    #[test]
    fn evenement_serialise_en_camel_case() {
        let e = Event {
            id: evidentyx_core::EventId::new(7),
            event: evidentyx_core::NewEvent::new(
                evidentyx_core::SourceRef {
                    evidence_id: evidentyx_core::EvidenceId::new(1),
                    byte_offset: 3,
                    line: Some(2),
                    record: None,
                },
                evidentyx_core::EventKind::Web,
                "GET / → 200",
                "brut",
            ),
        };
        let json = serde_json::to_value(EventDto::from(e)).unwrap();
        for key in [
            "evidenceId",
            "byteOffset",
            "timeUs",
            "sourceIp",
            "rawTruncated",
        ] {
            assert!(json.get(key).is_some(), "clé manquante : {key}");
        }
        assert_eq!(json["kind"], "web");
    }
}
