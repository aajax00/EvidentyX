# Journal des modifications

Toutes les modifications notables de ce projet sont consignées dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le projet respecte le [versionnage sémantique](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté

- Preuves : ajout de fichiers à une enquête (sélecteur ou glisser-déposer), empreinte SHA-256 calculée en flux et en lecture seule, taille et dates ; vérification d'intégrité (intègre, modifiée, introuvable) ; retrait sans jamais toucher au fichier ; contenu en double refusé ([ADR 0003](docs/decisions/0003-preuves-et-integrite.md)).
- Crate `evidentyx-evidence` (lecture en flux, progression, annulation) ; migration 0003 (table `evidence`) ; commandes `list_evidence`, `add_evidence`, `verify_evidence`, `remove_evidence`.
- Modèle d'événement canonique (`evidentyx-core`) : référence exacte à la source (preuve, décalage, ligne ou enregistrement), horodatage UTC avec précision et base (fuseau explicite, supposé UTC, année déduite), champs normalisés, texte brut et attributs d'origine conservés ([ADR 0004](docs/decisions/0004-modele-evenement.md)).
- Stockage des événements (migration 0004) : insertion par lot transactionnelle sans doublon au réimport, chronologie paginée déterministe, suppression en cascade ; nombre d'événements affiché pour chaque preuve.
- Analyse des preuves : bouton « Analyser » qui reconnaît le format et extrait les événements ; lecture refusée si la taille ou l'empreinte SHA-256 du fichier ne correspond plus (événements retirés, preuve marquée « modifiée ») ; réanalyse sans doublon ; bilan (événements, lignes illisibles avec leur numéro) ([ADR 0005](docs/decisions/0005-cadre-de-lecture.md)).
- Crate `evidentyx-parser` : cadre commun (détection, lecture en flux, erreurs par enregistrement, progression, annulation), correspondance des champs vers le modèle d'événement, lecture des heures ISO 8601 / RFC 3339 et époque Unix ; formats JSON Lines, tableau JSON et objets JSON concaténés.

## [0.1.0] - 2026-11-19

Première version : fin de la phase 1 (Fondations). L'application gère les enquêtes ; l'analyse des preuves arrive avec la phase 2.

### Ajouté

**Enquêtes**

- Création, ouverture, renommage, modification de la description, archivage (lecture seule), réactivation et suppression avec confirmation.
- Liste avec recherche dans le nom et la description (insensible aux accents et à la casse), filtre par statut et tri (dernière modification, création, nom).
- Page de détail : informations, description, vue d'ensemble illustrée par des **données de démonstration** clairement étiquetées, modules à venir signalés comme planifiés.
- Enregistrement dans une base SQLite locale, dans le dossier de données de l'application ; l'application démarre même si la base est inaccessible et l'indique.

**Interface**

- Application de bureau Tauri 2 + React 19 + TypeScript strict, entièrement hors ligne, politique de sécurité du contenu stricte.
- Accueil (statistiques, enquêtes récentes, feuille de route), barre latérale repliable, fil d'Ariane, barre d'état (moteur, base, nombre d'enquêtes), palette de commandes.
- Raccourcis : Ctrl+K / ⌘K (palette), Ctrl+N / ⌘N (nouvelle enquête), Ctrl+B / ⌘B (barre latérale), « / » (recherche).
- Paramètres enregistrés : écran d'ouverture, animation d'arrière-plan, barre latérale au démarrage, tri et statut par défaut de la liste.
- Identité visuelle : logo bleu, thème sombre sobre aux angles droits, polices IBM Plex Sans et Mono embarquées.
- Animations : écran d'ouverture (le logo se décode en ASCII), réseau de points discret en arrière-plan, ouverture des boîtes de dialogue et des pages en fondu ; désactivables, et coupées si le système demande moins d'animations.
- Système de design : jetons CSS, composants (`Button`, `IconButton`, `Badge`, `Panel`, `Dialog`, `TextField`, `TextAreaField`, `Select`, `SegmentedControl`, `SearchField`, `Switch`, `EmptyState`, `Spinner`, notifications) et catalogue de développement.
- Accessibilité : navigation au clavier, focus visible, libellés reliés aux champs, erreurs annoncées.

**Moteur**

- `evidentyx-core` : modèle des enquêtes et règles de validation, recherche et tri, normalisation de texte sans accents, réglages ; aucune dépendance externe ; messages d'erreur en français.
- `evidentyx-storage` : SQLite embarqué, migrations versionnées (schéma 2), contraintes en base, identifiants jamais réutilisés, revalidation des données relues ([ADR 0002](docs/decisions/0002-stockage-sqlite.md)).
- Hôte Tauri : commandes typées pour les enquêtes, les réglages et l'état du stockage ; erreurs typées avec le champ concerné.

**Distribution**

- Paquets Linux (AppImage, DEB, RPM), Windows (installateur, MSI) et macOS (DMG universel), construits par le workflow « Publication » ; **non signés** dans cette version (voir le [guide d'installation](docs/installation.md)).
- Version unique contrôlée par `scripts/version.mjs`.

**Projet**

- Licence Apache 2.0, guide de contribution, politique de sécurité, architecture, feuille de route, décisions techniques (ADR 0001, 0002), procédure de publication.
- Intégration continue : formatage, Clippy, tests Rust et de l'interface (Vitest) sur Linux, Windows et macOS.
- Mode démonstration pour développer l'interface dans un navigateur (signalé à l'écran, absent des paquets).
- Version minimale de Rust : 1.90.

[Non publié]: https://github.com/aajax00/EvidentyX/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aajax00/EvidentyX/releases/tag/v0.1.0
