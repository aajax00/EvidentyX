# EVIDENTYX

**Plateforme d'investigation numérique et de reconstruction d'incidents**

> *Chaque trace. Chaque connexion. Chaque vérité.*

EVIDENTYX est une application de bureau open source, multiplateforme (Linux, Windows, macOS) et fonctionnant entièrement hors ligne. Elle transforme des preuves numériques fragmentées (journaux, captures réseau) en enquêtes structurées, chronologiques et explicables.

- **Déterministe** : aucune IA générative, aucun service cloud. Une même entrée et les mêmes règles produisent toujours le même résultat.
- **Explicable** : chaque corrélation et chaque détection renvoie aux événements sources qui la justifient.
- **Respectueuse des preuves** : les fichiers sources ne sont jamais modifiés ; leur intégrité est garantie par SHA-256.
- **Locale** : aucune télémétrie, aucun compte, aucun envoi de données.

## État du projet

⚠️ **Développement précoce — version 0.0.1.** Aucune fonctionnalité d'analyse n'est encore disponible.

| Phase | Contenu | État |
|---|---|---|
| 1 — Fondations | Dépôt, workspace Rust, Tauri 2, React/TypeScript, système de design, gestion des enquêtes | 🟡 En cours |
| 2 — Ingestion | Modèle canonique, JSON, CSV, Syslog, Apache/Nginx, PCAP, hachage SHA-256 | ⚪ Planifiée |
| 3 — Moteur d'enquête | Normalisation, SQLite, chronologie, corrélation, graphe | ⚪ Planifiée |
| 4 — Détection et rapports | Règles de détection, coffre de preuves, exports PDF/JSON/CSV | ⚪ Planifiée |
| 5 — Stabilisation | Tests, sécurité, performances, paquets signés | ⚪ Planifiée |

Le détail est dans [`docs/feuille-de-route.md`](docs/feuille-de-route.md).

## Démarrage rapide (développeurs)

Prérequis actuels : [Rust](https://rustup.rs) ≥ 1.80.

```bash
git clone https://github.com/CHANGER-MOI/evidentyx.git
cd evidentyx
cargo test --workspace
```

Les prérequis Node.js et Tauri seront ajoutés avec l'application de bureau.

## Documentation

- [Architecture](docs/architecture.md)
- [Décisions techniques (ADR)](docs/decisions/)
- [Contribuer](CONTRIBUTING.md)
- [Politique de sécurité](SECURITY.md)
- [Journal des modifications](CHANGELOG.md)

## Licence

Distribué sous licence [Apache 2.0](LICENSE).
