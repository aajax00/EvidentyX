# ADR 0002 — Stockage SQLite et version minimale de Rust

- **Statut** : acceptée
- **Date** : 2026-11-19

## Contexte

L'étape 1.5 introduit la persistance des enquêtes. EVIDENTYX fonctionne hors ligne, sur un seul poste, et doit conserver des données traçables sur la durée. Par ailleurs, l'ADR 0001 fixait la version minimale de Rust (MSRV) à 1.80, alors que Tauri 2.11, ajouté à l'étape 1.2, exige déjà Rust 1.90 : cette valeur n'était plus respectée.

## Décisions

1. **SQLite, une base locale unique**, via le crate [`rusqlite`](https://crates.io/crates/rusqlite) (licence MIT) avec la fonctionnalité `bundled` : SQLite est compilé avec l'application, sans bibliothèque système, en version identique sur les trois plateformes. Pas d'ORM ni de générateur de requêtes : le SQL reste lisible et maîtrisé.
2. **Crate `evidentyx-storage`** : seul crate à parler SQL. Il n'accepte que des valeurs déjà validées par `evidentyx-core` et **revalide ce qu'il relit** ; une ligne invalide produit une erreur explicite (`CorruptRow`) plutôt qu'une donnée silencieusement fausse.
3. **Migrations versionnées** : scripts SQL numérotés dans `crates/storage/migrations/`, embarqués à la compilation, version suivie par `PRAGMA user_version`. Chaque migration s'applique dans une transaction. Une migration publiée n'est jamais modifiée. Une base créée par une version plus récente est refusée (`SchemaTooNew`) plutôt qu'ouverte au risque de la corrompre.
4. **Défense en profondeur** : tables `STRICT`, contraintes `CHECK` (statut connu, nom non vide, chronologie des dates), clés étrangères activées, journal WAL.
5. **Identifiants** : entiers `AUTOINCREMENT`, jamais réutilisés après suppression, pour qu'un identifiant cité dans un rapport ne désigne jamais une autre enquête.
6. **Horodatages** : millisecondes Unix UTC (`Timestamp` dans `evidentyx-core`), sans dépendance de date. L'horloge est injectable (`Clock`) pour des tests déterministes ; une date de modification ne recule jamais, même si l'horloge système recule.
7. **MSRV relevée à Rust 1.90** (remplace le point 7 de l'ADR 0001), pour refléter les exigences réelles de Tauri 2.11 et de `rusqlite` 0.43.

## Conséquences

- La compilation de SQLite (code C) allonge la première compilation d'environ 20 secondes ; elle est ensuite mise en cache.
- Toute évolution du schéma passe par une nouvelle migration et ses tests.
- L'hôte Tauri devra protéger la connexion par un verrou (`Storage` est `Send`, pas `Sync`) ; ce branchement est prévu à l'étape 1.6.
