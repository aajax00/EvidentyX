//! Types et règles de validation liés aux enquêtes.

use crate::error::CoreError;
use std::fmt;
use std::str::FromStr;

/// Longueur maximale d'un nom d'enquête, en caractères Unicode.
pub const MAX_NAME_CHARS: usize = 120;
/// Longueur maximale d'une description d'enquête, en caractères Unicode.
pub const MAX_DESCRIPTION_CHARS: usize = 4000;

/// Nom d'enquête validé : sans espaces superflus, non vide, borné,
/// sans caractères de contrôle.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvestigationName(String);

impl InvestigationName {
    /// Valide et normalise un nom d'enquête.
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(CoreError::EmptyName);
        }
        if trimmed.chars().any(char::is_control) {
            return Err(CoreError::ControlCharacters { field: "nom" });
        }
        let count = trimmed.chars().count();
        if count > MAX_NAME_CHARS {
            return Err(CoreError::NameTooLong {
                max: MAX_NAME_CHARS,
                actual: count,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InvestigationName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Description d'enquête validée. Peut être vide ; les retours à la ligne
/// et tabulations sont autorisés, les autres caractères de contrôle non.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InvestigationDescription(String);

impl InvestigationDescription {
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let trimmed = raw.trim();
        if trimmed
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            return Err(CoreError::ControlCharacters {
                field: "description",
            });
        }
        let count = trimmed.chars().count();
        if count > MAX_DESCRIPTION_CHARS {
            return Err(CoreError::DescriptionTooLong {
                max: MAX_DESCRIPTION_CHARS,
                actual: count,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Cycle de vie d'une enquête.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InvestigationStatus {
    /// Enquête ouverte, modifiable.
    Active,
    /// Enquête archivée : conservée, en lecture seule, réactivable.
    Archived,
}

impl InvestigationStatus {
    /// Représentation stable utilisée en base de données et dans l'API.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Archived => "archived",
        }
    }

    /// Libellé français destiné à l'interface.
    pub const fn label_fr(self) -> &'static str {
        match self {
            Self::Active => "Active",
            Self::Archived => "Archivée",
        }
    }

    /// Vérifie qu'une transition de statut est autorisée.
    pub fn transition_to(self, target: Self) -> Result<Self, CoreError> {
        if self == target {
            return Err(CoreError::InvalidTransition {
                from: self.as_str(),
                to: target.as_str(),
            });
        }
        Ok(target)
    }
}

impl FromStr for InvestigationStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "archived" => Ok(Self::Archived),
            other => Err(CoreError::UnknownStatus(other.to_owned())),
        }
    }
}

impl fmt::Display for InvestigationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Identifiant d'une enquête, attribué par le stockage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InvestigationId(i64);

impl InvestigationId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl fmt::Display for InvestigationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Instant UTC, en millisecondes depuis l'époque Unix (1970-01-01T00:00:00Z).
///
/// Représentation volontairement simple et sans dépendance : tri, comparaison
/// et stockage SQLite directs. La mise en forme lisible relève de l'interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const fn from_unix_millis(millis: i64) -> Self {
        Self(millis)
    }

    pub const fn unix_millis(self) -> i64 {
        self.0
    }
}

/// Enquête : unité de travail regroupant les preuves d'un incident.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Investigation {
    pub id: InvestigationId,
    pub name: InvestigationName,
    pub description: InvestigationDescription,
    pub status: InvestigationStatus,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Investigation {
    /// Vérifie que l'enquête peut être modifiée : une enquête archivée est en
    /// lecture seule jusqu'à sa réactivation.
    pub fn ensure_editable(&self) -> Result<(), CoreError> {
        match self.status {
            InvestigationStatus::Active => Ok(()),
            InvestigationStatus::Archived => Err(CoreError::ArchivedReadOnly),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nom_normalise_les_espaces() {
        let name = InvestigationName::parse("  Intrusion serveur web  ").unwrap();
        assert_eq!(name.as_str(), "Intrusion serveur web");
    }

    #[test]
    fn nom_vide_refuse() {
        assert_eq!(InvestigationName::parse("   "), Err(CoreError::EmptyName));
    }

    #[test]
    fn nom_trop_long_refuse() {
        let long = "é".repeat(MAX_NAME_CHARS + 1);
        assert!(matches!(
            InvestigationName::parse(&long),
            Err(CoreError::NameTooLong { actual, .. }) if actual == MAX_NAME_CHARS + 1
        ));
    }

    #[test]
    fn nom_a_la_limite_accepte_en_caracteres_unicode() {
        let exact = "é".repeat(MAX_NAME_CHARS);
        assert!(InvestigationName::parse(&exact).is_ok());
    }

    #[test]
    fn nom_avec_caracteres_de_controle_refuse() {
        assert!(matches!(
            InvestigationName::parse("abc\u{0007}def"),
            Err(CoreError::ControlCharacters { .. })
        ));
        assert!(InvestigationName::parse("ligne1\nligne2").is_err());
    }

    #[test]
    fn description_multiligne_acceptee() {
        let d = InvestigationDescription::parse("Ligne 1\nLigne 2\tTab").unwrap();
        assert_eq!(d.as_str(), "Ligne 1\nLigne 2\tTab");
    }

    #[test]
    fn description_vide_acceptee() {
        assert_eq!(InvestigationDescription::parse("").unwrap().as_str(), "");
    }

    #[test]
    fn description_trop_longue_refusee() {
        let long = "a".repeat(MAX_DESCRIPTION_CHARS + 1);
        assert!(InvestigationDescription::parse(&long).is_err());
    }

    #[test]
    fn statut_aller_retour_texte() {
        for s in [InvestigationStatus::Active, InvestigationStatus::Archived] {
            assert_eq!(s.as_str().parse::<InvestigationStatus>().unwrap(), s);
        }
        assert!("inconnu".parse::<InvestigationStatus>().is_err());
    }

    #[test]
    fn transitions_de_statut() {
        use InvestigationStatus::*;
        assert_eq!(Active.transition_to(Archived), Ok(Archived));
        assert_eq!(Archived.transition_to(Active), Ok(Active));
        assert!(Active.transition_to(Active).is_err());
    }

    fn sample(status: InvestigationStatus) -> Investigation {
        Investigation {
            id: InvestigationId::new(1),
            name: InvestigationName::parse("Test").unwrap(),
            description: InvestigationDescription::default(),
            status,
            created_at: Timestamp::from_unix_millis(0),
            updated_at: Timestamp::from_unix_millis(0),
        }
    }

    #[test]
    fn enquete_archivee_en_lecture_seule() {
        assert_eq!(
            sample(InvestigationStatus::Active).ensure_editable(),
            Ok(())
        );
        assert_eq!(
            sample(InvestigationStatus::Archived).ensure_editable(),
            Err(CoreError::ArchivedReadOnly)
        );
    }

    #[test]
    fn messages_d_erreur_en_francais() {
        assert_eq!(
            CoreError::EmptyName.to_string(),
            "le nom de l'enquête ne peut pas être vide"
        );
    }
}
