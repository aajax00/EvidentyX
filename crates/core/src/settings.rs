//! Réglages de l'application, stockés localement.

use crate::investigation::InvestigationStatus;
use crate::search::InvestigationSort;

/// Filtre de statut appliqué par défaut à la liste des enquêtes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusFilter {
    #[default]
    Active,
    Archived,
    All,
}

impl StatusFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
            Self::All => "all",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "active" => Some(Self::Active),
            "archived" => Some(Self::Archived),
            "all" => Some(Self::All),
            _ => None,
        }
    }

    /// Statut correspondant, `None` pour « toutes ».
    pub const fn status(self) -> Option<InvestigationStatus> {
        match self {
            Self::Active => Some(InvestigationStatus::Active),
            Self::Archived => Some(InvestigationStatus::Archived),
            Self::All => None,
        }
    }
}

/// Réglages de l'application. Chaque champ a une valeur par défaut : un
/// réglage absent ou illisible en base retombe sur celle-ci.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppSettings {
    /// Écran d'ouverture animé au lancement.
    pub splash_screen: bool,
    /// Animation discrète en arrière-plan de l'interface.
    pub background_animation: bool,
    /// Barre latérale repliée au démarrage.
    pub sidebar_collapsed: bool,
    /// Tri par défaut de la liste des enquêtes.
    pub default_sort: InvestigationSort,
    /// Filtre de statut par défaut de la liste des enquêtes.
    pub default_status_filter: StatusFilter,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            splash_screen: true,
            background_animation: true,
            sidebar_collapsed: false,
            default_sort: InvestigationSort::default(),
            default_status_filter: StatusFilter::default(),
        }
    }
}

const KEY_SPLASH: &str = "splash_screen";
const KEY_BACKGROUND: &str = "background_animation";
const KEY_SIDEBAR: &str = "sidebar_collapsed";
const KEY_SORT: &str = "default_sort";
const KEY_STATUS: &str = "default_status_filter";

fn bool_str(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}

impl AppSettings {
    /// Représentation clé → valeur, ordonnée, pour le stockage.
    pub fn to_pairs(&self) -> [(&'static str, &'static str); 5] {
        [
            (KEY_SPLASH, bool_str(self.splash_screen)),
            (KEY_BACKGROUND, bool_str(self.background_animation)),
            (KEY_SIDEBAR, bool_str(self.sidebar_collapsed)),
            (KEY_SORT, self.default_sort.as_str()),
            (KEY_STATUS, self.default_status_filter.as_str()),
        ]
    }

    /// Reconstruit les réglages depuis des paires clé → valeur. Les clés
    /// inconnues et les valeurs invalides sont ignorées (valeur par défaut).
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        let mut settings = Self::default();
        for (key, value) in pairs {
            let flag = match value {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            };
            match key {
                KEY_SPLASH => settings.splash_screen = flag.unwrap_or(settings.splash_screen),
                KEY_BACKGROUND => {
                    settings.background_animation = flag.unwrap_or(settings.background_animation)
                }
                KEY_SIDEBAR => {
                    settings.sidebar_collapsed = flag.unwrap_or(settings.sidebar_collapsed)
                }
                KEY_SORT => {
                    settings.default_sort =
                        InvestigationSort::parse(value).unwrap_or(settings.default_sort)
                }
                KEY_STATUS => {
                    settings.default_status_filter =
                        StatusFilter::parse(value).unwrap_or(settings.default_status_filter)
                }
                _ => {}
            }
        }
        settings
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aller_retour_cle_valeur() {
        let settings = AppSettings {
            splash_screen: false,
            background_animation: false,
            sidebar_collapsed: true,
            default_sort: InvestigationSort::NameAsc,
            default_status_filter: StatusFilter::All,
        };
        assert_eq!(AppSettings::from_pairs(settings.to_pairs()), settings);
    }

    #[test]
    fn valeurs_invalides_et_cles_inconnues_ignorees() {
        let settings = AppSettings::from_pairs([
            ("splash_screen", "peut-être"),
            ("default_sort", "inconnu"),
            ("cle_future", "x"),
            ("sidebar_collapsed", "true"),
        ]);
        assert_eq!(
            settings,
            AppSettings {
                sidebar_collapsed: true,
                ..AppSettings::default()
            }
        );
    }

    #[test]
    fn filtre_de_statut() {
        assert_eq!(StatusFilter::All.status(), None);
        assert_eq!(
            StatusFilter::parse("archived").and_then(StatusFilter::status),
            Some(InvestigationStatus::Archived)
        );
    }
}
