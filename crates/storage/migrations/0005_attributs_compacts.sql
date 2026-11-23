-- Migration 0005 : attributs d'événement regroupés en une colonne JSON.
--
-- Une ligne par attribut donnait des dizaines de millions de lignes sur les
-- fichiers à nombreuses colonnes (suppression et ouverture très lentes).
-- Les attributs existants sont conservés à l'identique.

ALTER TABLE events ADD COLUMN attributes TEXT NOT NULL DEFAULT '{}';

UPDATE events
   SET attributes = (SELECT json_group_object(key, value)
                       FROM event_attributes a WHERE a.event_id = events.id)
 WHERE EXISTS (SELECT 1 FROM event_attributes a WHERE a.event_id = events.id);

DROP TABLE event_attributes;
