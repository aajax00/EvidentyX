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
- Système de design inspiré de COSINT (graphite neutre, angles droits, bordures de 1 px, thème sombre par défaut, polices IBM Plex Sans et Mono embarquées) : jetons CSS (couleurs, typographie, espacements, thèmes sombre et clair), composants de base `Button`, `Badge`, `Panel`, `TextField`, `TextAreaField`, `Spinner`, et catalogue de développement (`#catalogue`).
- Coque applicative : barre latérale repliable (Ctrl+B / ⌘B) avec sections planifiées signalées comme telles, en-tête avec état du moteur, pages Accueil et À propos, palette de commandes (Ctrl+K / ⌘K) avec recherche insensible aux accents et navigation au clavier.
- Tests de l'interface avec Vitest (filtrage des commandes, raccourcis, navigation), exécutés en intégration continue.
- Crate `evidentyx-storage` : base SQLite locale (embarquée), migrations versionnées, création, lecture, liste, renommage, description, archivage, réactivation et suppression des enquêtes ; revalidation des données relues (ADR 0002).
- `evidentyx-core` : entité `Investigation`, identifiant, horodatage UTC et règle « enquête archivée en lecture seule ».

### Modifié

- Version minimale de Rust relevée de 1.80 à 1.90, exigée par Tauri 2.11 et `rusqlite` (ADR 0002).
