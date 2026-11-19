//! Recherche et tri des enquêtes : règles uniques, partagées par toutes les vues.

use crate::investigation::{Investigation, InvestigationStatus};
use crate::text::fold;
use std::cmp::Ordering;

/// Ordre d'affichage d'une liste d'enquêtes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InvestigationSort {
    /// Dernière modification d'abord (par défaut).
    #[default]
    UpdatedDesc,
    /// Création la plus récente d'abord.
    CreatedDesc,
    /// Ordre alphabétique, sans tenir compte des accents ni de la casse.
    NameAsc,
}

impl InvestigationSort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UpdatedDesc => "updated_desc",
            Self::CreatedDesc => "created_desc",
            Self::NameAsc => "name_asc",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "updated_desc" => Some(Self::UpdatedDesc),
            "created_desc" => Some(Self::CreatedDesc),
            "name_asc" => Some(Self::NameAsc),
            _ => None,
        }
    }
}

/// Critères de recherche d'enquêtes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvestigationQuery {
    /// `None` : tous les statuts.
    pub status: Option<InvestigationStatus>,
    /// Texte libre : chaque mot doit apparaître dans le nom ou la description
    /// (sans tenir compte des accents ni de la casse).
    pub text: String,
    pub sort: InvestigationSort,
}

impl InvestigationQuery {
    /// Filtre et trie les enquêtes. Le résultat est déterministe : à critère
    /// égal, l'identifiant le plus grand (le plus récent) passe d'abord.
    pub fn apply(&self, investigations: Vec<Investigation>) -> Vec<Investigation> {
        let folded = fold(&self.text);
        let terms: Vec<&str> = folded.split(' ').filter(|t| !t.is_empty()).collect();

        let mut kept: Vec<(String, Investigation)> = investigations
            .into_iter()
            .filter(|i| self.status.is_none_or(|s| i.status == s))
            .filter_map(|i| {
                let name = fold(i.name.as_str());
                if terms.is_empty() {
                    return Some((name, i));
                }
                let haystack = format!("{name} {}", fold(i.description.as_str()));
                terms
                    .iter()
                    .all(|t| haystack.contains(t))
                    .then_some((name, i))
            })
            .collect();

        kept.sort_by(|(name_a, a), (name_b, b)| {
            let primary = match self.sort {
                InvestigationSort::UpdatedDesc => b.updated_at.cmp(&a.updated_at),
                InvestigationSort::CreatedDesc => b.created_at.cmp(&a.created_at),
                InvestigationSort::NameAsc => name_a.cmp(name_b),
            };
            primary.then_with(|| b.id.cmp(&a.id)).then(Ordering::Equal)
        });
        kept.into_iter().map(|(_, i)| i).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::investigation::{
        InvestigationDescription, InvestigationId, InvestigationName, Timestamp,
    };

    fn inv(
        id: i64,
        name: &str,
        desc: &str,
        status: InvestigationStatus,
        created: i64,
        updated: i64,
    ) -> Investigation {
        Investigation {
            id: InvestigationId::new(id),
            name: InvestigationName::parse(name).unwrap(),
            description: InvestigationDescription::parse(desc).unwrap(),
            status,
            created_at: Timestamp::from_unix_millis(created),
            updated_at: Timestamp::from_unix_millis(updated),
        }
    }

    fn sample() -> Vec<Investigation> {
        use InvestigationStatus::*;
        vec![
            inv(
                1,
                "Intrusion serveur web",
                "Nginx compromis",
                Active,
                100,
                500,
            ),
            inv(
                2,
                "Fuite de données",
                "Exfiltration via DNS",
                Archived,
                200,
                300,
            ),
            inv(3, "élévation de privilèges", "Poste RH", Active, 300, 400),
            inv(4, "Hameçonnage", "Campagne ciblée", Active, 400, 400),
        ]
    }

    fn ids(list: Vec<Investigation>) -> Vec<i64> {
        list.into_iter().map(|i| i.id.get()).collect()
    }

    #[test]
    fn tri_par_defaut_derniere_modification() {
        // 3 et 4 modifiées au même instant : l'identifiant le plus grand d'abord.
        assert_eq!(
            ids(InvestigationQuery::default().apply(sample())),
            [1, 4, 3, 2]
        );
    }

    #[test]
    fn tri_par_creation_et_par_nom() {
        let q = |sort| InvestigationQuery {
            sort,
            ..Default::default()
        };
        assert_eq!(
            ids(q(InvestigationSort::CreatedDesc).apply(sample())),
            [4, 3, 2, 1]
        );
        // « élévation » se classe à « e », malgré l'accent et la minuscule.
        assert_eq!(
            ids(q(InvestigationSort::NameAsc).apply(sample())),
            [3, 2, 4, 1]
        );
    }

    #[test]
    fn filtre_par_statut() {
        let q = InvestigationQuery {
            status: Some(InvestigationStatus::Archived),
            ..Default::default()
        };
        assert_eq!(ids(q.apply(sample())), [2]);
    }

    #[test]
    fn recherche_insensible_aux_accents_dans_nom_et_description() {
        let q = |text: &str| InvestigationQuery {
            text: text.into(),
            ..Default::default()
        };
        assert_eq!(ids(q("ELEVATION").apply(sample())), [3]);
        assert_eq!(ids(q("dns").apply(sample())), [2]);
        assert_eq!(ids(q("  serveur   nginx ").apply(sample())), [1]);
        assert_eq!(ids(q("serveur dns").apply(sample())), Vec::<i64>::new());
    }

    #[test]
    fn tri_aller_retour_texte() {
        for s in [
            InvestigationSort::UpdatedDesc,
            InvestigationSort::CreatedDesc,
            InvestigationSort::NameAsc,
        ] {
            assert_eq!(InvestigationSort::parse(s.as_str()), Some(s));
        }
        assert_eq!(InvestigationSort::parse("autre"), None);
    }
}
