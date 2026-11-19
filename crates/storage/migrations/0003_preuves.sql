-- Migration 0003 : preuves rattachées aux enquêtes.
--
-- Seules des références sont stockées (chemin, taille, empreinte SHA-256) :
-- les fichiers sources ne sont jamais copiés en base. Supprimer une enquête
-- supprime ses références, jamais les fichiers.
-- Un même contenu (même empreinte) n'est enregistré qu'une fois par enquête.

CREATE TABLE evidence (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    investigation_id INTEGER NOT NULL REFERENCES investigations (id) ON DELETE CASCADE,
    file_name        TEXT    NOT NULL CHECK (length(file_name) > 0),
    path             TEXT    NOT NULL CHECK (length(path) > 0),
    size_bytes       INTEGER NOT NULL CHECK (size_bytes >= 0),
    sha256           TEXT    NOT NULL CHECK (length(sha256) = 64),
    file_modified_at INTEGER,
    added_at         INTEGER NOT NULL,
    integrity        TEXT    NOT NULL CHECK (integrity IN ('intact', 'modified', 'missing')),
    verified_at      INTEGER NOT NULL CHECK (verified_at >= added_at),
    UNIQUE (investigation_id, sha256)
) STRICT;

CREATE INDEX evidence_by_investigation ON evidence (investigation_id, added_at, id);
