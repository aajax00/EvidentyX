-- Migration 0002 : réglages de l'application (clé → valeur).
--
-- Les valeurs sont interprétées et validées par evidentyx-core (AppSettings) :
-- une clé inconnue ou une valeur invalide est ignorée à la lecture.

CREATE TABLE settings (
    key   TEXT NOT NULL PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
