//! Modèle d'événement canonique.
//!
//! Chaque format de preuve (JSON, CSV, Syslog, journaux web, PCAP…) est
//! converti vers ce modèle unique. Principes :
//! - **traçabilité** : chaque événement désigne l'enregistrement source exact
//!   (preuve, décalage en octets, ligne ou numéro de paquet) ;
//! - **honnêteté temporelle** : l'horodatage indique sa précision et la façon
//!   dont il a été établi (fuseau explicite, supposé UTC, année déduite) ;
//! - **rien d'inventé** : un champ absent de la source reste vide ; le texte
//!   brut et les attributs d'origine sont conservés.

use crate::error::CoreError;
use crate::evidence::EvidenceId;
use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;

/// Taille maximale du texte brut conservé (64 Kio) ; au-delà, il est tronqué et signalé.
pub const MAX_RAW_BYTES: usize = 64 * 1024;
/// Longueur maximale d'une clé d'attribut, en caractères.
pub const MAX_ATTRIBUTE_KEY_CHARS: usize = 128;
/// Longueur maximale d'une valeur d'attribut, en caractères.
pub const MAX_ATTRIBUTE_VALUE_CHARS: usize = 8 * 1024;
/// Nombre maximal d'attributs par événement.
pub const MAX_ATTRIBUTES: usize = 256;

macro_rules! text_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident => $text:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum $name {
            $($(#[$vmeta])* $variant),+
        }

        impl $name {
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $text),+
                }
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    other => Err(CoreError::UnknownEventValue {
                        field: stringify!($name),
                        value: other.to_owned(),
                    }),
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }
    };
}

text_enum! {
    /// Précision de l'horodatage indiquée par la source.
    TimePrecision {
        /// Date seule (jour).
        Day => "day",
        Second => "second",
        Millisecond => "millisecond",
        Microsecond => "microsecond",
    }
}

impl TimePrecision {
    /// Durée couverte par une unité de cette précision, en microsecondes.
    pub const fn unit_micros(self) -> i64 {
        match self {
            Self::Day => 86_400_000_000,
            Self::Second => 1_000_000,
            Self::Millisecond => 1_000,
            Self::Microsecond => 1,
        }
    }
}

text_enum! {
    /// Façon dont l'horodatage a été établi à partir de la source.
    TimeBasis {
        /// La source donne un fuseau ou un décalage explicite (ou une heure UTC).
        Explicit => "explicit",
        /// La source n'indique pas de fuseau : heure interprétée comme UTC.
        AssumedUtc => "assumed_utc",
        /// La source n'indique ni année ni fuseau (Syslog RFC 3164) : année
        /// déduite du contexte, heure interprétée comme UTC.
        InferredYear => "inferred_year",
    }
}

text_enum! {
    /// Catégorie générale d'un événement.
    EventKind {
        Authentication => "authentication",
        Network => "network",
        Web => "web",
        Process => "process",
        File => "file",
        System => "system",
        Other => "other",
    }
}

text_enum! {
    /// Issue d'une action, quand la source l'indique.
    Outcome {
        Success => "success",
        Failure => "failure",
    }
}

/// Horodatage d'un événement, en microsecondes UTC depuis l'époque Unix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventTime {
    pub utc_micros: i64,
    pub precision: TimePrecision,
    pub basis: TimeBasis,
}

impl EventTime {
    /// Intervalle `[début, fin)` dans lequel l'instant réel se situe, compte
    /// tenu de la précision de la source. L'incertitude liée au fuseau
    /// (`TimeBasis::AssumedUtc`, `InferredYear`) n'est pas incluse : elle est
    /// signalée à part, car elle peut atteindre plusieurs heures.
    pub const fn window(self) -> (i64, i64) {
        (
            self.utc_micros,
            self.utc_micros.saturating_add(self.precision.unit_micros()),
        )
    }

    /// Vrai si l'heure dépend d'une hypothèse (fuseau supposé, année déduite).
    pub const fn is_assumed(self) -> bool {
        !matches!(self.basis, TimeBasis::Explicit)
    }
}

/// Enregistrement source exact d'un événement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SourceRef {
    pub evidence_id: EvidenceId,
    /// Décalage en octets du début de l'enregistrement dans le fichier.
    pub byte_offset: u64,
    /// Numéro de ligne (à partir de 1), pour les formats texte.
    pub line: Option<u64>,
    /// Numéro d'enregistrement (à partir de 1) : paquet, entrée…
    pub record: Option<u64>,
}

/// Événement extrait d'une preuve, avant enregistrement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewEvent {
    pub source: SourceRef,
    /// `None` si la source ne contient pas d'horodatage exploitable.
    pub time: Option<EventTime>,
    pub kind: EventKind,
    pub host: Option<String>,
    pub source_ip: Option<IpAddr>,
    pub source_port: Option<u16>,
    pub destination_ip: Option<IpAddr>,
    pub destination_port: Option<u16>,
    pub user: Option<String>,
    pub action: Option<String>,
    pub outcome: Option<Outcome>,
    /// Résumé lisible, non vide.
    pub message: String,
    /// Texte brut de l'enregistrement (tronqué à `MAX_RAW_BYTES` si nécessaire).
    pub raw: String,
    pub raw_truncated: bool,
    /// Champs d'origine non normalisés, triés par clé.
    pub attributes: BTreeMap<String, String>,
}

impl NewEvent {
    /// Événement minimal : source, catégorie, message et texte brut ; les
    /// champs normalisés sont à renseigner ensuite, uniquement s'ils existent
    /// dans la source.
    pub fn new(source: SourceRef, kind: EventKind, message: impl Into<String>, raw: &str) -> Self {
        let (raw, raw_truncated) = truncate_raw(raw);
        Self {
            source,
            time: None,
            kind,
            host: None,
            source_ip: None,
            source_port: None,
            destination_ip: None,
            destination_port: None,
            user: None,
            action: None,
            outcome: None,
            message: message.into(),
            raw,
            raw_truncated,
            attributes: BTreeMap::new(),
        }
    }

    /// Vérifie les règles du modèle avant enregistrement.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.message.trim().is_empty() {
            return Err(CoreError::InvalidEvent("message vide".into()));
        }
        if self.raw.len() > MAX_RAW_BYTES {
            return Err(CoreError::InvalidEvent("texte brut trop long".into()));
        }
        if self.source.line == Some(0) || self.source.record == Some(0) {
            return Err(CoreError::InvalidEvent(
                "numéros de ligne et d'enregistrement : à partir de 1".into(),
            ));
        }
        for text in [&self.host, &self.user, &self.action].into_iter().flatten() {
            if text.trim().is_empty() {
                return Err(CoreError::InvalidEvent(
                    "champ texte vide : utiliser l'absence de valeur".into(),
                ));
            }
        }
        if self.attributes.len() > MAX_ATTRIBUTES {
            return Err(CoreError::InvalidEvent(format!(
                "trop d'attributs ({}, maximum {MAX_ATTRIBUTES})",
                self.attributes.len()
            )));
        }
        for (key, value) in &self.attributes {
            if key.is_empty() || key.chars().count() > MAX_ATTRIBUTE_KEY_CHARS {
                return Err(CoreError::InvalidEvent(format!(
                    "clé d'attribut invalide : « {key} »"
                )));
            }
            if value.chars().count() > MAX_ATTRIBUTE_VALUE_CHARS {
                return Err(CoreError::InvalidEvent(format!(
                    "valeur d'attribut trop longue : « {key} »"
                )));
            }
        }
        Ok(())
    }
}

/// Tronque le texte brut à `MAX_RAW_BYTES` sur une frontière de caractère.
fn truncate_raw(raw: &str) -> (String, bool) {
    if raw.len() <= MAX_RAW_BYTES {
        return (raw.to_owned(), false);
    }
    let mut end = MAX_RAW_BYTES;
    while !raw.is_char_boundary(end) {
        end -= 1;
    }
    (raw[..end].to_owned(), true)
}

/// Identifiant d'un événement enregistré.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EventId(i64);

impl EventId {
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> i64 {
        self.0
    }
}

/// Événement enregistré.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub id: EventId,
    pub event: NewEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> SourceRef {
        SourceRef {
            evidence_id: EvidenceId::new(1),
            byte_offset: 0,
            line: Some(1),
            record: None,
        }
    }

    #[test]
    fn fenetre_selon_la_precision() {
        let t = EventTime {
            utc_micros: 1_000_000,
            precision: TimePrecision::Second,
            basis: TimeBasis::Explicit,
        };
        assert_eq!(t.window(), (1_000_000, 2_000_000));
        assert!(!t.is_assumed());
        let t = EventTime {
            precision: TimePrecision::Microsecond,
            basis: TimeBasis::AssumedUtc,
            ..t
        };
        assert_eq!(t.window(), (1_000_000, 1_000_001));
        assert!(t.is_assumed());
    }

    #[test]
    fn valeurs_textuelles_aller_retour() {
        for k in EventKind::ALL {
            assert_eq!(k.as_str().parse::<EventKind>().unwrap(), *k);
        }
        for p in TimePrecision::ALL {
            assert_eq!(p.as_str().parse::<TimePrecision>().unwrap(), *p);
        }
        for b in TimeBasis::ALL {
            assert_eq!(b.as_str().parse::<TimeBasis>().unwrap(), *b);
        }
        assert!("inconnu".parse::<Outcome>().is_err());
    }

    #[test]
    fn texte_brut_tronque_sur_une_frontiere_de_caractere() {
        let long = "é".repeat(MAX_RAW_BYTES); // 2 octets par caractère
        let e = NewEvent::new(source(), EventKind::Other, "x", &long);
        assert!(e.raw_truncated);
        assert!(e.raw.len() <= MAX_RAW_BYTES);
        assert!(e.validate().is_ok());
        let court = NewEvent::new(source(), EventKind::Other, "x", "ligne");
        assert!(!court.raw_truncated);
    }

    #[test]
    fn validation() {
        let ok = NewEvent::new(
            source(),
            EventKind::Authentication,
            "Échec de connexion",
            "raw",
        );
        assert!(ok.validate().is_ok());

        let mut e = ok.clone();
        e.message = "  ".into();
        assert!(e.validate().is_err());

        let mut e = ok.clone();
        e.user = Some(String::new());
        assert!(e.validate().is_err());

        let mut e = ok.clone();
        e.source.line = Some(0);
        assert!(e.validate().is_err());

        let mut e = ok.clone();
        e.attributes.insert(String::new(), "v".into());
        assert!(e.validate().is_err());

        let mut e = ok;
        e.attributes
            .insert("k".into(), "v".repeat(MAX_ATTRIBUTE_VALUE_CHARS + 1));
        assert!(e.validate().is_err());
    }
}
