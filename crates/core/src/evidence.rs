//! Preuves : fichiers sources rattachés à une enquête.
//!
//! Une preuve n'est jamais copiée ni modifiée : EVIDENTYX en conserve le
//! chemin, la taille et l'empreinte SHA-256, qui permet de vérifier à tout
//! moment que le fichier n'a pas changé depuis son enregistrement.

use crate::error::CoreError;
use crate::investigation::{InvestigationId, Timestamp};
use std::fmt;
use std::str::FromStr;

/// Identifiant d'une preuve, attribué par le stockage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EvidenceId(i64);

impl EvidenceId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

impl fmt::Display for EvidenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Empreinte SHA-256 : 64 caractères hexadécimaux en minuscules.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Sha256Digest(String);

impl Sha256Digest {
    /// Construit l'empreinte à partir des 32 octets bruts.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut out = String::with_capacity(64);
        for b in bytes {
            out.push(HEX[usize::from(b >> 4)] as char);
            out.push(HEX[usize::from(b & 0x0f)] as char);
        }
        Self(out)
    }

    /// Valide une empreinte textuelle (majuscules acceptées, normalisées en minuscules).
    pub fn parse(raw: &str) -> Result<Self, CoreError> {
        let value = raw.trim().to_ascii_lowercase();
        if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(CoreError::InvalidDigest);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Résultat de la dernière vérification d'intégrité d'une preuve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntegrityStatus {
    /// Le fichier existe et son empreinte est identique.
    Intact,
    /// Le fichier existe mais son empreinte a changé.
    Modified,
    /// Le fichier est introuvable ou illisible.
    Missing,
}

impl IntegrityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Intact => "intact",
            Self::Modified => "modified",
            Self::Missing => "missing",
        }
    }

    pub const fn label_fr(self) -> &'static str {
        match self {
            Self::Intact => "Intègre",
            Self::Modified => "Modifiée",
            Self::Missing => "Introuvable",
        }
    }

    /// Compare l'empreinte enregistrée à celle recalculée (`None` : fichier inaccessible).
    pub fn compare(recorded: &Sha256Digest, current: Option<&Sha256Digest>) -> Self {
        match current {
            None => Self::Missing,
            Some(d) if d == recorded => Self::Intact,
            Some(_) => Self::Modified,
        }
    }
}

impl FromStr for IntegrityStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "intact" => Ok(Self::Intact),
            "modified" => Ok(Self::Modified),
            "missing" => Ok(Self::Missing),
            other => Err(CoreError::UnknownIntegrityStatus(other.to_owned())),
        }
    }
}

/// Preuve enregistrée dans une enquête.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    pub id: EvidenceId,
    pub investigation_id: InvestigationId,
    /// Nom du fichier (dernier composant du chemin).
    pub file_name: String,
    /// Chemin absolu du fichier au moment de l'enregistrement.
    pub path: String,
    pub size_bytes: u64,
    pub sha256: Sha256Digest,
    /// Date de dernière modification du fichier selon le système, si disponible.
    pub file_modified_at: Option<Timestamp>,
    pub added_at: Timestamp,
    pub integrity: IntegrityStatus,
    pub verified_at: Timestamp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empreinte_depuis_octets() {
        let d = Sha256Digest::from_bytes([0xab; 32]);
        assert_eq!(d.as_str(), "ab".repeat(32));
    }

    #[test]
    fn empreinte_textuelle_validee_et_normalisee() {
        let upper = "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855";
        assert_eq!(
            Sha256Digest::parse(upper).unwrap().as_str(),
            upper.to_ascii_lowercase()
        );
        assert!(Sha256Digest::parse("abc").is_err());
        assert!(Sha256Digest::parse(&"g".repeat(64)).is_err());
    }

    #[test]
    fn comparaison_d_integrite() {
        let a = Sha256Digest::from_bytes([1; 32]);
        let b = Sha256Digest::from_bytes([2; 32]);
        assert_eq!(
            IntegrityStatus::compare(&a, Some(&a)),
            IntegrityStatus::Intact
        );
        assert_eq!(
            IntegrityStatus::compare(&a, Some(&b)),
            IntegrityStatus::Modified
        );
        assert_eq!(IntegrityStatus::compare(&a, None), IntegrityStatus::Missing);
    }

    #[test]
    fn statut_aller_retour_texte() {
        for s in [
            IntegrityStatus::Intact,
            IntegrityStatus::Modified,
            IntegrityStatus::Missing,
        ] {
            assert_eq!(s.as_str().parse::<IntegrityStatus>().unwrap(), s);
        }
        assert!("autre".parse::<IntegrityStatus>().is_err());
    }
}
