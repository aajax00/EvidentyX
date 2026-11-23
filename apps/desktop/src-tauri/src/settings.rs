//! Commandes des réglages de l'application.

use crate::error::CommandError;
use crate::state::AppState;
use evidentyx_core::{AppSettings, InvestigationSort, StatusFilter};
use evidentyx_storage::Storage;
use serde::{Deserialize, Serialize};

/// Miroir TypeScript : `Settings` dans `apps/desktop/src/api.ts`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub splash_screen: bool,
    pub background_animation: bool,
    pub sidebar_collapsed: bool,
    /// "updated_desc", "created_desc" ou "name_asc".
    pub default_sort: String,
    /// "active", "archived" ou "all".
    pub default_status_filter: String,
}

impl From<AppSettings> for SettingsDto {
    fn from(s: AppSettings) -> Self {
        Self {
            splash_screen: s.splash_screen,
            background_animation: s.background_animation,
            sidebar_collapsed: s.sidebar_collapsed,
            default_sort: s.default_sort.as_str().to_owned(),
            default_status_filter: s.default_status_filter.as_str().to_owned(),
        }
    }
}

impl TryFrom<SettingsDto> for AppSettings {
    type Error = CommandError;

    fn try_from(dto: SettingsDto) -> Result<Self, CommandError> {
        Ok(Self {
            splash_screen: dto.splash_screen,
            background_animation: dto.background_animation,
            sidebar_collapsed: dto.sidebar_collapsed,
            default_sort: InvestigationSort::parse(&dto.default_sort).ok_or_else(|| {
                CommandError::invalid_argument(format!("Tri inconnu : « {} »", dto.default_sort))
            })?,
            default_status_filter: StatusFilter::parse(&dto.default_status_filter).ok_or_else(
                || {
                    CommandError::invalid_argument(format!(
                        "Filtre de statut inconnu : « {} »",
                        dto.default_status_filter
                    ))
                },
            )?,
        })
    }
}

pub fn get(storage: &Storage) -> Result<SettingsDto, CommandError> {
    Ok(storage.settings()?.into())
}

pub fn update(storage: &mut Storage, dto: SettingsDto) -> Result<SettingsDto, CommandError> {
    let settings = AppSettings::try_from(dto)?;
    Ok(storage.save_settings(&settings)?.into())
}

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn get_settings(state: tauri::State<'_, AppState>) -> Result<SettingsDto, CommandError> {
    state.with_storage(|s| get(s))
}

// Hors du fil de la fenêtre : la base peut être occupée.
#[tauri::command(async)]
pub fn update_settings(
    state: tauri::State<'_, AppState>,
    settings: SettingsDto,
) -> Result<SettingsDto, CommandError> {
    state.with_storage(|s| update(s, settings))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lecture_et_modification() {
        let mut s = Storage::open_in_memory().unwrap();
        let mut dto = get(&s).unwrap();
        assert!(dto.splash_screen);
        assert_eq!(dto.default_sort, "updated_desc");

        dto.splash_screen = false;
        dto.default_status_filter = "all".into();
        let saved = update(&mut s, dto.clone()).unwrap();
        assert_eq!(saved, dto);
        assert_eq!(get(&s).unwrap(), dto);
    }

    #[test]
    fn valeurs_inconnues_refusees() {
        let mut s = Storage::open_in_memory().unwrap();
        let mut dto = get(&s).unwrap();
        dto.default_sort = "hasard".into();
        assert_eq!(update(&mut s, dto).unwrap_err().code, "invalid_argument");
    }
}
