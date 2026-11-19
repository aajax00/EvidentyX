-- Migration 0001 : table des enquêtes.
--
-- AUTOINCREMENT empêche la réutilisation d'un identifiant après suppression :
-- un identifiant cité dans un rapport ne désignera jamais une autre enquête.
-- Les horodatages sont des millisecondes Unix UTC (type Timestamp du crate core).

CREATE TABLE investigations (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT    NOT NULL CHECK (length(trim(name)) > 0),
    description TEXT    NOT NULL DEFAULT '',
    status      TEXT    NOT NULL CHECK (status IN ('active', 'archived')),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL CHECK (updated_at >= created_at)
) STRICT;

CREATE INDEX investigations_by_status_and_update
    ON investigations (status, updated_at DESC, id DESC);
