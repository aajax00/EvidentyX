//! Commandes de gestion des enquêtes.
//!
//! Chaque commande Tauri délègue à une fonction testable qui reçoit le
//! stockage : validation par `evidentyx-core`, persistance par
//! `evidentyx-storage`. Ce module ne fait que convertir les données.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{
    Investigation, InvestigationDescription, InvestigationId, InvestigationName,
    InvestigationQuery, InvestigationSort, InvestigationStatus,
};
use evidentyx_storage::Storage;
use serde::{Deserialize, Serialize};

/// Miroir TypeScript : `Investigation` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvestigationDto {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub status: &'static str,
    /// Millisecondes Unix UTC.
    pub created_at: i64,
    pub updated_at: i64,
}

impl From<Investigation> for InvestigationDto {
    fn from(i: Investigation) -> Self {
        Self {
            id: i.id.get(),
            name: i.name.as_str().to_owned(),
            description: i.description.as_str().to_owned(),
            status: i.status.as_str(),
            created_at: i.created_at.unix_millis(),
            updated_at: i.updated_at.unix_millis(),
        }
    }
}

/// Miroir TypeScript : `InvestigationQuery` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryDto {
    /// "active", "archived" ou absent (tous).
    pub status: Option<String>,
    #[serde(default)]
    pub text: String,
    /// "updated_desc" (défaut), "created_desc", "name_asc".
    pub sort: Option<String>,
}

type CommandResult<T> = Result<T, CommandError>;

fn parse_status(value: &str) -> CommandResult<InvestigationStatus> {
    Ok(value.parse::<InvestigationStatus>()?)
}

pub fn list(storage: &Storage, query: QueryDto) -> CommandResult<Vec<InvestigationDto>> {
    let status = query.status.as_deref().map(parse_status).transpose()?;
    let sort = match query.sort.as_deref() {
        None => InvestigationSort::default(),
        Some(s) => InvestigationSort::parse(s)
            .ok_or_else(|| CommandError::invalid_argument(format!("Tri inconnu : « {s} »")))?,
    };
    let query = InvestigationQuery {
        status,
        text: query.text,
        sort,
    };
    let all = storage.list_investigations(status)?;
    Ok(query.apply(all).into_iter().map(Into::into).collect())
}

pub fn get(storage: &Storage, id: i64) -> CommandResult<InvestigationDto> {
    Ok(storage.investigation(InvestigationId::new(id))?.into())
}

pub fn create(
    storage: &mut Storage,
    name: &str,
    description: &str,
) -> CommandResult<InvestigationDto> {
    let name = InvestigationName::parse(name)?;
    let description = InvestigationDescription::parse(description)?;
    Ok(storage.create_investigation(&name, &description)?.into())
}

pub fn rename(storage: &mut Storage, id: i64, name: &str) -> CommandResult<InvestigationDto> {
    let name = InvestigationName::parse(name)?;
    Ok(storage
        .rename_investigation(InvestigationId::new(id), &name)?
        .into())
}

pub fn update_description(
    storage: &mut Storage,
    id: i64,
    description: &str,
) -> CommandResult<InvestigationDto> {
    let description = InvestigationDescription::parse(description)?;
    Ok(storage
        .update_investigation_description(InvestigationId::new(id), &description)?
        .into())
}

pub fn set_status(storage: &mut Storage, id: i64, status: &str) -> CommandResult<InvestigationDto> {
    let status = parse_status(status)?;
    Ok(storage
        .set_investigation_status(InvestigationId::new(id), status)?
        .into())
}

pub fn delete(storage: &mut Storage, id: i64) -> CommandResult<()> {
    Ok(storage.delete_investigation(InvestigationId::new(id))?)
}

// ——— Commandes Tauri ———

#[tauri::command]
pub fn list_investigations(
    state: tauri::State<'_, AppState>,
    query: QueryDto,
) -> CommandResult<Vec<InvestigationDto>> {
    state.with_storage(|s| list(s, query))
}

#[tauri::command]
pub fn get_investigation(
    state: tauri::State<'_, AppState>,
    id: i64,
) -> CommandResult<InvestigationDto> {
    state.with_storage(|s| get(s, id))
}

#[tauri::command]
pub fn create_investigation(
    state: tauri::State<'_, AppState>,
    name: String,
    description: String,
) -> CommandResult<InvestigationDto> {
    state.with_storage(|s| create(s, &name, &description))
}

#[tauri::command]
pub fn rename_investigation(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: String,
) -> CommandResult<InvestigationDto> {
    state.with_storage(|s| rename(s, id, &name))
}

#[tauri::command]
pub fn update_investigation_description(
    state: tauri::State<'_, AppState>,
    id: i64,
    description: String,
) -> CommandResult<InvestigationDto> {
    state.with_storage(|s| update_description(s, id, &description))
}

#[tauri::command]
pub fn set_investigation_status(
    state: tauri::State<'_, AppState>,
    id: i64,
    status: String,
) -> CommandResult<InvestigationDto> {
    state.with_storage(|s| set_status(s, id, &status))
}

#[tauri::command]
pub fn delete_investigation(state: tauri::State<'_, AppState>, id: i64) -> CommandResult<()> {
    state.with_storage(|s| delete(s, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Storage {
        Storage::open_in_memory().unwrap()
    }

    fn query(status: Option<&str>, text: &str, sort: Option<&str>) -> QueryDto {
        QueryDto {
            status: status.map(Into::into),
            text: text.into(),
            sort: sort.map(Into::into),
        }
    }

    #[test]
    fn cycle_de_vie_complet() {
        let mut s = db();
        let created = create(&mut s, "  Intrusion  ", "Serveur web").unwrap();
        assert_eq!(created.name, "Intrusion");
        assert_eq!(created.status, "active");

        let renamed = rename(&mut s, created.id, "Intrusion nginx").unwrap();
        assert_eq!(renamed.name, "Intrusion nginx");
        let described = update_description(&mut s, created.id, "Détails").unwrap();
        assert_eq!(described.description, "Détails");

        let archived = set_status(&mut s, created.id, "archived").unwrap();
        assert_eq!(archived.status, "archived");
        assert_eq!(
            rename(&mut s, created.id, "X").unwrap_err().code,
            "archived_read_only"
        );
        set_status(&mut s, created.id, "active").unwrap();

        delete(&mut s, created.id).unwrap();
        assert_eq!(get(&s, created.id).unwrap_err().code, "not_found");
    }

    #[test]
    fn erreurs_de_validation_avec_champ() {
        let mut s = db();
        let e = create(&mut s, "   ", "").unwrap_err();
        assert_eq!((e.code, e.field), ("validation", Some("name")));
        let e = create(&mut s, "Nom", &"a".repeat(5000)).unwrap_err();
        assert_eq!((e.code, e.field), ("validation", Some("description")));
        assert_eq!(
            set_status(&mut s, 1, "inconnu").unwrap_err().code,
            "validation"
        );
    }

    #[test]
    fn liste_filtree_et_triee() {
        let mut s = db();
        let a = create(&mut s, "Bêta", "").unwrap();
        let b = create(&mut s, "alpha", "dns").unwrap();
        set_status(&mut s, a.id, "archived").unwrap();

        let names = |q| {
            list(&s, q)
                .unwrap()
                .into_iter()
                .map(|i| i.name)
                .collect::<Vec<_>>()
        };
        assert_eq!(names(query(None, "", Some("name_asc"))), ["alpha", "Bêta"]);
        assert_eq!(names(query(Some("archived"), "", None)), ["Bêta"]);
        assert_eq!(names(query(None, "DNS", None)), ["alpha"]);
        assert!(list(&s, query(None, "", Some("autre"))).is_err());
        let _ = b;
    }
}
