# Contribuer à EVIDENTYX

Merci de votre intérêt pour EVIDENTYX. Ce document décrit les conventions du projet.

## Langue

- Interface, documentation, messages d'erreur destinés à l'utilisateur, messages de commit : **français**.
- Identifiants de code (types, fonctions, variables) : **anglais**, conformément aux usages Rust et TypeScript.

## Branches

- `main` : toujours stable, compilable et testée. Protégée.
- `feat/<sujet>`, `fix/<sujet>`, `docs/<sujet>`, `chore/<sujet>` : branches de travail, fusionnées par pull request.

## Messages de commit — Conventional Commits

Format : `<type>(<portée>): <description à l'impératif, en français>`

| Type | Usage |
|---|---|
| `feat` | Nouvelle fonctionnalité |
| `fix` | Correction de bogue |
| `docs` | Documentation uniquement |
| `refactor` | Restructuration sans changement de comportement |
| `test` | Ajout ou correction de tests |
| `perf` | Amélioration de performances |
| `build` | Système de build, dépendances, empaquetage |
| `ci` | Intégration continue |
| `chore` | Maintenance diverse |

Exemples :

```
feat(core): ajouter la validation des noms d'enquête
fix(parser): ne plus ignorer les lignes syslog sans PRI
docs: décrire l'architecture du moteur de corrélation
```

Un changement incompatible est signalé par `!` (`feat(core)!: …`) et une ligne `BREAKING CHANGE:` dans le corps.

## Versionnage — SemVer

EVIDENTYX suit le [versionnage sémantique](https://semver.org/lang/fr/) : `MAJEUR.MINEUR.CORRECTIF`.

- Avant la 1.0, une version mineure (`0.x.0`) peut contenir des changements incompatibles.
- Chaque version est marquée par une étiquette Git annotée `vX.Y.Z`.
- La fin de chaque phase de la feuille de route donne lieu à une version mineure (`v0.1.0` = fin de la phase 1, etc.).
- Le fichier [`CHANGELOG.md`](CHANGELOG.md) suit le format [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/).

## Avant d'ouvrir une pull request

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/desktop && npm test && npm run build
```

## Dépendances

Toute nouvelle dépendance doit être justifiée dans la pull request : utilité, état de maintenance, licence (compatible Apache 2.0). Les décisions structurantes sont consignées dans `docs/decisions/`.

## Règles du projet

- Aucun bouton sans action, aucune page factice présentée comme terminée.
- Une fonctionnalité non implémentée est explicitement signalée comme « planifiée » ou « indisponible ».
- Aucune donnée inventée : un champ absent de la source reste vide.
- Les données de démonstration sont clairement étiquetées comme telles.
