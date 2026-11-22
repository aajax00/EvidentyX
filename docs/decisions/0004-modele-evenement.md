# ADR 0004 — Modèle d'événement canonique

- **Statut** : acceptée
- **Date** : 2026-11-23

## Contexte

La phase 2 convertit des formats très différents (JSON, CSV, Syslog, journaux web, PCAP) en événements analysables ensemble (chronologie, corrélation, détection). Il faut un modèle unique, traçable jusqu'à la source et honnête sur ce que la source dit réellement — en particulier sur l'heure, souvent incomplète (pas de fuseau, pas d'année en Syslog RFC 3164, précision variable).

## Décisions

1. **Un modèle unique** (`NewEvent` / `Event` dans `evidentyx-core`) : catégorie (`authentication`, `network`, `web`, `process`, `file`, `system`, `other`), hôte, adresses et ports source/destination (`IpAddr`, `u16`), utilisateur, action, issue (`success` / `failure`), message lisible, texte brut, attributs d'origine.
2. **Traçabilité** : `SourceRef` = preuve + **décalage en octets** (toujours présent) + numéro de ligne et/ou d'enregistrement (à partir de 1). Le couple (preuve, décalage) est **unique** : réimporter une preuve ne crée pas de doublon (les événements déjà présents sont comptés comme doublons, pas réinsérés).
3. **Heure honnête** : `EventTime` = microsecondes UTC + **précision** (`day`, `second`, `millisecond`, `microsecond`) + **base** (`explicit` : fuseau donné par la source ; `assumed_utc` : pas de fuseau, heure lue comme UTC ; `inferred_year` : ni année ni fuseau). `window()` donne l'intervalle dû à la précision ; l'incertitude de fuseau est signalée par la base, pas fondue dans l'intervalle (elle peut atteindre plusieurs heures). Un événement sans heure exploitable a `time = None` : il n'est jamais daté arbitrairement.
4. **Rien d'inventé** : un champ absent de la source reste vide (`NULL` en base) ; une chaîne vide est refusée (on utilise l'absence de valeur). Le texte brut est conservé (tronqué à 64 Kio, troncature signalée) ainsi que les attributs non normalisés (table `event_attributes`, clé → valeur, triés).
5. **Stockage** (migration 0004) : table `events` (`STRICT`, contraintes de domaine, heure complète ou absente), index de chronologie `(enquête, heure, id)`. Insertion **par lot en une transaction**, lot entier refusé si un événement est invalide. Suppression en cascade avec la preuve ou l'enquête.
6. **Chronologie déterministe** : ordre (heure, puis id d'insertion), événements sans heure à la fin ; pagination par **curseur** (stable, sans `OFFSET`).
7. **Aucune dépendance ajoutée** : adresses IP via `std::net`, horodatages en entiers.

## Conséquences

- Chaque analyseur de format (étapes 2.3 à 2.7) produit des `NewEvent` et choisit honnêtement précision et base de l'heure.
- La normalisation plus poussée (fuseaux configurables, déduction d'année guidée par l'utilisateur) relève de la phase 3 ; le modèle porte déjà l'information nécessaire.
