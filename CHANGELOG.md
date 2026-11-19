# Journal des modifications

Toutes les modifications notables de ce projet sont consignées dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le projet respecte le [versionnage sémantique](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté

- Structure initiale du dépôt, licence Apache 2.0, documentation de contribution et de sécurité.
- Workspace Cargo et crate `evidentyx-core` : validation des noms et descriptions d'enquête, cycle de vie des statuts (active / archivée), erreurs en français.
- Intégration continue GitHub Actions : formatage, Clippy et tests sur Linux, Windows et macOS.
- Documentation d'architecture, feuille de route et première décision technique (ADR 0001).
- Application de bureau Tauri 2 + React 19 + TypeScript strict (Vite) dans `apps/desktop`, reliée à `evidentyx-core` par une première commande typée (`engine_info`) ; politique de sécurité du contenu stricte.
- Système de design inspiré de COSINT (graphite neutre, angles droits, bordures de 1 px, polices IBM Plex Sans et Mono embarquées) : jetons CSS (couleurs, typographie, espacements), composants de base `Button`, `Badge`, `Panel`, `TextField`, `TextAreaField`, `Spinner`, et catalogue de développement (`#catalogue`).
- Coque applicative : barre latérale repliable (Ctrl+B / ⌘B) avec sections planifiées signalées comme telles, en-tête avec état du moteur, pages Accueil et À propos, palette de commandes (Ctrl+K / ⌘K) avec recherche insensible aux accents et navigation au clavier.
- Tests de l'interface avec Vitest (filtrage des commandes, raccourcis, navigation), exécutés en intégration continue.
- Crate `evidentyx-storage` : base SQLite locale (embarquée), migrations versionnées, création, lecture, liste, renommage, description, archivage, réactivation et suppression des enquêtes ; revalidation des données relues (ADR 0002).
- `evidentyx-core` : entité `Investigation`, identifiant, horodatage UTC et règle « enquête archivée en lecture seule ».
- Gestion des enquêtes dans l'application : création, ouverture, renommage, modification de la description, archivage, réactivation et suppression (avec confirmation) ; liste avec recherche (insensible aux accents, nom et description), filtre par statut et tri. Données enregistrées dans une base SQLite locale (dossier de données de l'application).
- Commandes Tauri des enquêtes et de l'état du stockage ; erreurs typées avec champ concerné ; l'application démarre même si la base est inaccessible et l'indique.
- `evidentyx-core` : recherche et tri des enquêtes (`InvestigationQuery`), normalisation de texte sans accents (`text::fold`).
- Interface : page Enquêtes, page de détail d'une enquête, accueil avec statistiques et enquêtes récentes, fil d'Ariane, barre d'état (moteur, base, nombre d'enquêtes), enquêtes récentes dans la barre latérale, palette enrichie (nouvelle enquête, ouverture d'une enquête), raccourci Ctrl+N / ⌘N.
- Composants : `Dialog`, `IconButton`, boutons avec icône, `SegmentedControl`, `Select`, `SearchField`, `EmptyState`, notifications (`ToastProvider`).
- Identité visuelle : logo intégré en bleu (barre latérale, accueil, À propos, icônes de l'application) ; palette sombre et sobre, accent unique au bleu du logo ; thème sombre uniquement (le thème clair est retiré).
- Mode démonstration pour le développement dans un navigateur (moteur en mémoire, signalé à l'écran, exclu de la version publiée).

### Modifié

- Version minimale de Rust relevée de 1.80 à 1.90, exigée par Tauri 2.11 et `rusqlite` (ADR 0002).
