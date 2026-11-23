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

## Complément — CSV (étape 2.4)

- Crate [`csv`](https://crates.io/crates/csv) (MIT / Unlicense) : conforme RFC 4180, rapide, positions d'enregistrement.
- **Séparateur** deviné sur les 50 premiers enregistrements (découpage respectant les guillemets) : celui qui donne au moins 2 colonnes et le nombre de colonnes le plus constant (≥ 80 % des enregistrements).
- **En-tête** reconnu si la première ligne ne contient que des noms (non vides, non numériques, non datés, distincts) ; sinon colonnes `colonne_1`, `colonne_2`…
- **Heure** choisie une fois par fichier sur la première ligne de données : alias connus, puis colonnes date et heure séparées (combinées), puis toute colonne au nom évocateur. La colonne retenue figure dans le bilan.
- **CRLF** : la crate `csv` donne alors des numéros de ligne faux et un décalage posé sur le `\n` précédent ; un lecteur intermédiaire note les fins de ligne lues et recalcule ligne et décalage exacts (mémoire bornée).
- **Texte brut** : enregistrement reconstitué (mêmes valeurs, séparateur d'origine, guillemets normalisés) ; le décalage et la ligne désignent l'original.
- **EZ Tools** : les colonnes d'EvtxECmd (`TimeCreated`, `Computer`, `UserName`, `MapDescription`…) sont reconnues par les alias communs ; les sorties à plusieurs horodatages par ligne (MFTECmd) relèvent d'une évolution ultérieure.
- **Performances mesurées** (compilation optimisée, lecture seule) : 70 000 lignes de journaux en 2 s ; fichiers de flux CIC-IDS2017 à 79 colonnes ≈ 5 700 lignes/s — optimisation prévue en phase 5.

## Complément — Syslog (étape 2.5)

- Trois écritures, reconnues **ligne par ligne** (un fichier peut les mélanger) : RFC 5424, RFC 3164 (`<PRI>` facultatif, absent des fichiers Linux), format fichier de rsyslog (heure ISO 8601, étiquette `app:` exigée pour ne pas confondre avec un journal daté quelconque). Détection : au moins 80 % des premières lignes ; testée avant le CSV.
- **Année (RFC 3164)** : la plus récente qui ne place pas l'événement plus d'un jour après la date de modification du fichier ; base `InferredYear`. Sans date de fichier, l'événement reste sans heure. Limites : date de modification altérée par une copie, fichier couvrant plus d'un an ; fuseau inconnu, lu comme UTC.
- **Champs repris des messages** (sshd, sudo, pare-feu du noyau), mot pour mot. Les noms d'utilisateur de sshd viennent de l'attaquant : l'adresse est lue à partir de la fin du message, pour qu'un nom piégé (« x from 1.2.3.4 ») ne la falsifie pas. sudo n'indique pas la réussite d'une commande : aucune issue n'est déduite, sauf refus explicite.
- Catégorie d'après l'application, puis le service (`facility`). Pas de nouvelle dépendance.

## Conséquences

- Les étapes 2.4 à 2.7 ajoutent un module par format en réutilisant le cadre et `map_fields`.
- Progression détaillée et annulation dans l'interface : étape 2.8 (le cadre les gère déjà).
- Performances mesurées : ~30 000 événements/s sur 70 000 enregistrements JSON Lines réels (31,6 Mo, compilation optimisée) ; optimisation prévue en phase 5 si nécessaire.
