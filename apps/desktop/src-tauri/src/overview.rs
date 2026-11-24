//! Vue d'ensemble d'une enquête : statistiques réelles des événements extraits.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::InvestigationId;
use evidentyx_storage::Storage;
use serde::Serialize;

/// Intervalles de l'histogramme d'activité.
const BUCKETS: u32 = 48;

/// Miroir TypeScript : `Overview` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDto {
    pub events: u64,
    pub timed: u64,
    /// Premier et dernier événement daté, en ms Unix UTC.
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
    /// Événements datés, en intervalles égaux de `first_at` à `last_at`.
    pub histogram: Vec<u64>,
    pub by_kind: Vec<KindCount>,
    pub ips: u64,
    pub users: u64,
    pub hosts: u64,
    pub evidence_total: u64,
    /// Preuves ayant produit des événements, du plus grand nombre au plus petit.
    pub sources: Vec<SourceCount>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindCount {
    pub kind: &'static str,
    pub events: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCount {
    pub evidence_id: i64,
    pub file_name: String,
    pub events: u64,
}

pub fn overview(storage: &Storage, investigation_id: i64) -> Result<OverviewDto, CommandError> {
    let id = InvestigationId::new(investigation_id);
    let o = storage.overview(id, BUCKETS)?;
    let counts = storage.event_counts_by_evidence(id)?;
    let evidence = storage.list_evidence(id)?;
    let mut sources: Vec<SourceCount> = evidence
        .iter()
        .filter_map(|e| {
            let events = counts.get(&e.id).copied().filter(|&n| n > 0)?;
            Some(SourceCount {
                evidence_id: e.id.get(),
                file_name: e.file_name.clone(),
                events,
            })
        })
        .collect();
    sources.sort_by(|a, b| {
        b.events
            .cmp(&a.events)
            .then(a.evidence_id.cmp(&b.evidence_id))
    });
    Ok(OverviewDto {
        events: o.events,
        timed: o.timed,
        first_at: o.first_us.map(|t| t.div_euclid(1000)),
        last_at: o.last_us.map(|t| t.div_euclid(1000)),
        histogram: o.histogram,
        by_kind: o
            .by_kind
            .into_iter()
            .map(|(k, n)| KindCount {
                kind: k.as_str(),
                events: n,
            })
            .collect(),
        ips: o.ips,
        users: o.users,
        hosts: o.hosts,
        evidence_total: evidence.len() as u64,
        sources,
    })
}

/// Miroir TypeScript : `Totals` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotalsDto {
    pub events: u64,
    pub evidence: u64,
    pub evidence_intact: u64,
}

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn totals(state: tauri::State<'_, AppState>) -> Result<TotalsDto, CommandError> {
    state.with_storage(|s| {
        let t = s.totals()?;
        Ok(TotalsDto {
            events: t.events,
            evidence: t.evidence,
            evidence_intact: t.evidence_intact,
        })
    })
}

// Hors du fil de la fenêtre : calcul long sur une grosse enquête.
#[tauri::command(async)]
pub fn investigation_overview(
    state: tauri::State<'_, AppState>,
    investigation_id: i64,
) -> Result<OverviewDto, CommandError> {
    state.with_storage(|s| overview(s, investigation_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enquete_vide_puis_sources_triees() {
        let mut s = Storage::open_in_memory().unwrap();
        let inv = crate::investigations::create(&mut s, "Enquête", "").unwrap();
        let o = overview(&s, inv.id).unwrap();
        assert_eq!((o.events, o.evidence_total, o.first_at), (0, 0, None));
        assert!(o.sources.is_empty() && o.histogram.is_empty());

        let json = serde_json::to_value(&o).unwrap();
        for key in ["firstAt", "byKind", "evidenceTotal", "histogram"] {
            assert!(json.get(key).is_some(), "clé manquante : {key}");
        }
    }
}
