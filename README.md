<p align="center"><img src="docs/assets/logo.png" alt="Logo EVIDENTYX" width="280"></p>

# EVIDENTYX

**Plateforme d'investigation numérique et de reconstruction d'incidents**

> *Chaque trace. Chaque connexion. Chaque vérité.*

EVIDENTYX est une application de bureau open source, multiplateforme (Linux, Windows, macOS) et fonctionnant entièrement hors ligne. Elle transforme des preuves numériques fragmentées (journaux, captures réseau) en enquêtes structurées, chronologiques et explicables.

- **Déterministe** : aucune IA générative, aucun service cloud. Une même entrée et les mêmes règles produisent toujours le même résultat.
- **Explicable** : chaque corrélation et chaque détection renvoie aux événements sources qui la justifient.
- **Respectueuse des preuves** : les fichiers sources ne sont jamais modifiés ; leur intégrité est garantie par SHA-256.
- **Locale** : aucune télémétrie, aucun compte, aucun envoi de données.

## État du projet

⚠️ **Développement précoce — version 0.1.0.** La gestion des enquêtes est disponible ; l'import et l'analyse des preuves arrivent avec la phase 2.

| Phase | Contenu | État |
|---|---|---|
| 1 — Fondations | Dépôt, workspace Rust, Tauri 2, React/TypeScript, système de design, gestion des enquêtes | ✅ Terminée (`v0.1.0`) |
| 2 — Ingestion | Modèle canonique, JSON, CSV, Syslog, Apache/Nginx, PCAP, hachage SHA-256 | 🟡 Prochaine |
| 3 — Moteur d'enquête | Normalisation, SQLite, chronologie, corrélation, graphe | ⚪ Planifiée |
| 4 — Détection et rapports | Règles de détection, coffre de preuves, exports PDF/JSON/CSV | ⚪ Planifiée |
| 5 — Stabilisation | Tests, sécurité, performances, paquets signés | ⚪ Planifiée |

Le détail est dans [`docs/feuille-de-route.md`](docs/feuille-de-route.md).

## Installation

Les paquets (Linux, Windows, macOS) sont joints aux [publications](https://github.com/aajax00/EvidentyX/releases). Voir le [guide d'installation](docs/installation.md).

## Démarrage rapide (développeurs)

Prérequis :

- [Rust](https://rustup.rs) ≥ 1.90 ;
- [Node.js](https://nodejs.org) ≥ 20.19 ;
- les [dépendances système de Tauri](https://v2.tauri.app/start/prerequisites/). Sous Debian/Ubuntu :

```bash
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev pkg-config
```

Lancer l'application en mode développement :

```bash
git clone https://github.com/aajax00/EvidentyX.git
cd EvidentyX/apps/desktop
npm install
npm run tauri dev
```

Lancer les tests (l'interface doit avoir été construite une fois avec `npm run build`, car l'hôte Tauri l'embarque à la compilation) :

```bash
cargo test --workspace
```

## Documentation

- [Architecture](docs/architecture.md)
- [Système de design](docs/systeme-de-design.md)
- [Installation](docs/installation.md)
- [Publier une version](docs/publication.md)
- [Décisions techniques (ADR)](docs/decisions/)
- [Contribuer](CONTRIBUTING.md)
- [Politique de sécurité](SECURITY.md)
- [Journal des modifications](CHANGELOG.md)

## Licence

Distribué sous licence [Apache 2.0](LICENSE).
