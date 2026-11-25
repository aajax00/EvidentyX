//! Chronologie d'une enquête : événements par position, pour une liste virtualisée.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{Event, EvidenceId, InvestigationId};
use evidentyx_storage::EventFilter;
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
    pub ip: Option<String>,
    pub user: Option<String>,
    pub host: Option<String>,
    pub text: Option<String>,
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
            outcome: f
                .outcome
                .map(|o| o.parse())
                .transpose()
                .map_err(CommandError::from)?,
            ip: f.ip,
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
