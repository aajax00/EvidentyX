# ADR 0005 — Cadre de lecture des formats et analyse d'une preuve

- **Statut** : acceptée
- **Date** : 2026-11-23

## Contexte

Chaque format de preuve (JSON, CSV, Syslog, journaux web, PCAP) doit produire des événements canoniques (ADR 0004). Les fichiers peuvent être volumineux, partiellement corrompus, et leur valeur probante exige que les événements proviennent exactement du fichier enregistré.

## Décisions

1. **Crate `evidentyx-parser`**, un module par format, derrière une interface commune : `detect(nom, premiers octets)`, `parse(format, lecteur, contexte, destinataire)` qui transmet chaque événement dans l'ordre du fichier et renvoie un **bilan** (enregistrements, événements, erreurs détaillées).
2. **Lecture en flux**, mémoire bornée ; enregistrement limité à 4 Mio.
3. **Erreurs par enregistrement** : une ligne illisible est comptée et décrite (ligne, décalage, raison ; 50 premières conservées) sans interrompre la lecture. Seule une structure globale illisible (tableau JSON non fermé) arrête la lecture ; les événements déjà lus sont conservés.
4. **Traçabilité** : décalage en octets et ligne de début de chaque enregistrement, numéro d'ordre.
5. **Correspondance des champs commune** (`map_fields`) : alias comparés sans casse ni séparateurs (`source_ip`, `sourceIp`, `source.ip`), valeurs validées (une « IP » invalide reste en attribut), heure lue en ISO 8601 / RFC 3339 ou époque Unix (secondes à nanosecondes) avec précision et base honnêtes, catégorie déduite par mots-clés (heuristique documentée), message de la source ou résumé composé uniquement de valeurs présentes ; tous les autres champs conservés en attributs.
6. **JSON** via `serde_json` (MIT / Apache-2.0) : JSON Lines (ligne à ligne, reprise à la ligne suivante), tableau JSON et objets concaténés (découpeur à états qui isole chaque valeur en flux avec sa position).
7. **Analyse d'une preuve** (commande `analyze_evidence`) : fichier ouvert en lecture seule, taille vérifiée, **empreinte SHA-256 recalculée pendant la lecture** ; si elle diffère de l'empreinte enregistrée, les événements extraits sont retirés, la preuve passe en « modifiée » et l'analyse est refusée. Événements enregistrés par lots de 2 000 ; une réanalyse n'ajoute que ce qui manque.
8. **Lecture des heures sans dépendance** (calendrier grégorien, décalages horaires) : bornes de plausibilité 1970–2100.

## Conséquences

- Les étapes 2.4 à 2.7 ajoutent un module par format en réutilisant le cadre et `map_fields`.
- Progression détaillée et annulation dans l'interface : étape 2.8 (le cadre les gère déjà).
- Performances mesurées : ~30 000 événements/s sur 70 000 enregistrements JSON Lines réels (31,6 Mo, compilation optimisée) ; optimisation prévue en phase 5 si nécessaire.
