-- Migration 0004 : événements canoniques extraits des preuves.
--
-- Chaque événement désigne son enregistrement source exact (preuve + décalage
-- en octets, unique) : réimporter une preuve ne crée pas de doublon.
-- Horodatage en microsecondes UTC, avec sa précision et sa base (fuseau
-- explicite, supposé UTC, année déduite). Un champ absent de la source reste NULL.
-- Les attributs d'origine non normalisés sont conservés dans event_attributes.

CREATE TABLE events (
    id               INTEGER PRIMARY KEY AUTOINCREMENT,
    investigation_id INTEGER NOT NULL REFERENCES investigations (id) ON DELETE CASCADE,
    evidence_id      INTEGER NOT NULL REFERENCES evidence (id) ON DELETE CASCADE,
    byte_offset      INTEGER NOT NULL CHECK (byte_offset >= 0),
    line             INTEGER CHECK (line >= 1),
    record           INTEGER CHECK (record >= 1),
    time_us          INTEGER,
    time_precision   TEXT CHECK (time_precision IN ('day', 'second', 'millisecond', 'microsecond')),
    time_basis       TEXT CHECK (time_basis IN ('explicit', 'assumed_utc', 'inferred_year')),
    kind             TEXT NOT NULL CHECK (kind IN ('authentication', 'network', 'web', 'process', 'file', 'system', 'other')),
    host             TEXT,
    source_ip        TEXT,
    source_port      INTEGER CHECK (source_port BETWEEN 0 AND 65535),
    destination_ip   TEXT,
    destination_port INTEGER CHECK (destination_port BETWEEN 0 AND 65535),
    user_name        TEXT,
    action           TEXT,
    outcome          TEXT CHECK (outcome IN ('success', 'failure')),
    message          TEXT NOT NULL CHECK (length(trim(message)) > 0),
    raw              TEXT NOT NULL,
    raw_truncated    INTEGER NOT NULL CHECK (raw_truncated IN (0, 1)),
    -- L'heure est complète ou absente, jamais partielle.
    CHECK ((time_us IS NULL) = (time_precision IS NULL) AND (time_us IS NULL) = (time_basis IS NULL)),
    UNIQUE (evidence_id, byte_offset)
) STRICT;

-- Chronologie d'une enquête : par heure (les événements sans heure à la fin), puis ordre d'insertion.
CREATE INDEX events_timeline ON events (investigation_id, time_us, id);

CREATE TABLE event_attributes (
    event_id INTEGER NOT NULL REFERENCES events (id) ON DELETE CASCADE,
    key      TEXT    NOT NULL CHECK (length(key) > 0),
    value    TEXT    NOT NULL,
    PRIMARY KEY (event_id, key)
) STRICT, WITHOUT ROWID;
