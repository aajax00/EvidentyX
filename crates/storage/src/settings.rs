//! Persistance des réglages.

use crate::error::Result;
use crate::Storage;
use evidentyx_core::AppSettings;

impl Storage {
    /// Lit les réglages ; les valeurs absentes ou invalides prennent leur valeur par défaut.
    pub fn settings(&self) -> Result<AppSettings> {
        let mut stmt = self.conn.prepare("SELECT key, value FROM settings")?;
        let pairs = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        Ok(AppSettings::from_pairs(
            pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        ))
    }

    /// Enregistre tous les réglages en une transaction.
    pub fn save_settings(&mut self, settings: &AppSettings) -> Result<AppSettings> {
        let tx = self.conn.transaction()?;
        for (key, value) in settings.to_pairs() {
            tx.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
                [key, value],
            )?;
        }
        tx.commit()?;
        self.settings()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use evidentyx_core::{InvestigationSort, StatusFilter};

    #[test]
    fn valeurs_par_defaut_puis_enregistrement() {
        let mut db = Storage::open_in_memory().unwrap();
        assert_eq!(db.settings().unwrap(), AppSettings::default());

        let changed = AppSettings {
            splash_screen: false,
            default_sort: InvestigationSort::NameAsc,
            default_status_filter: StatusFilter::All,
            display_utc: true,
            ..AppSettings::default()
        };
        assert_eq!(db.save_settings(&changed).unwrap(), changed);
        assert_eq!(db.settings().unwrap(), changed);

        // Un second enregistrement remplace les valeurs.
        assert_eq!(
            db.save_settings(&AppSettings::default()).unwrap(),
            AppSettings::default()
        );
    }

    #[test]
    fn valeur_corrompue_ignoree() {
        let db = Storage::open_in_memory().unwrap();
        db.conn
            .execute(
                "INSERT INTO settings VALUES ('splash_screen', 'n''importe quoi')",
                [],
            )
            .unwrap();
        assert_eq!(db.settings().unwrap(), AppSettings::default());
    }
}
