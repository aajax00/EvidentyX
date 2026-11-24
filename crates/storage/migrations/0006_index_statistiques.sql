-- Migration 0006 : index des statistiques d'une enquête (vue d'ensemble).
--
-- Les lignes d'événements sont volumineuses (texte brut, attributs) : sans
-- index, chaque statistique relit toute la table (95 s pour 1,7 million
-- d'événements mesurés). Cet index couvre les colonnes comptées ; la création
-- est longue une seule fois sur une grosse base. Aucune donnée modifiée.

CREATE INDEX events_stats ON events (investigation_id, kind, source_ip, destination_ip, user_name, host);
