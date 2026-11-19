# Architecture d'EVIDENTYX

> Document vivant : il décrit l'état **actuel** du code et signale ce qui est planifié.

## Vue d'ensemble

```
┌───────────────────────────────────────────────────────────┐
│  Interface (React + TypeScript, système de design maison)  │  initialisée
└──────────────────────────┬────────────────────────────────┘
                           │ commandes Tauri typées
┌──────────────────────────▼────────────────────────────────┐
│  Hôte de bureau (Tauri 2, Rust)                           │  initialisé
└──────────────────────────┬────────────────────────────────┘
                           │ appels de bibliothèque
┌──────────────────────────▼────────────────────────────────┐
│  Moteur d'analyse (workspace Cargo)                       │
│  core · parser · normalizer · timeline · correlation      │
│  graph · detection · evidence · storage · report          │
└──────────────────────────┬────────────────────────────────┘
                           │
                ┌──────────▼──────────┐
                │  SQLite (local)     │  initialisé
                └─────────────────────┘
```

## Principes

1. **Séparation stricte** : l'interface affiche et pilote ; toute logique métier vit dans les crates Rust. Aucune règle métier dupliquée côté TypeScript.
2. **Déterminisme** : pas d'aléatoire dans l'analyse ; tris stables ; résultats reproductibles.
3. **Traçabilité** : chaque événement normalisé référence son enregistrement source (fichier, décalage, numéro de ligne ou de paquet).
4. **Immuabilité des preuves** : fichiers sources en lecture seule, référencés par chemin et SHA-256, jamais copiés en base.
5. **Hors ligne** : aucune dépendance réseau à l'exécution.

## Crates

| Crate | Rôle | État |
|---|---|---|
| `evidentyx-core` | Types de domaine partagés, validation, erreurs | ✅ Initialisé |
| `evidentyx-desktop` (`apps/desktop/src-tauri`) | Hôte Tauri : ouvre la base locale, expose le moteur et les enquêtes à l'interface par des commandes typées, sans logique métier | ✅ Enquêtes |
| `evidentyx-storage` | Persistance SQLite, migrations versionnées, enquêtes (journal d'audit : phase 4) | ✅ Initialisé |
| `evidentyx-parser` | Analyseurs de formats (JSON, CSV, Syslog, Apache, Nginx, PCAP, EVTX) | ⚪ Phase 2 |
| `evidentyx-normalizer` | Conversion vers le modèle d'événement canonique | ⚪ Phase 2-3 |
| `evidentyx-evidence` | Hachage SHA-256, vérification d'intégrité | ⚪ Phase 2 |
| `evidentyx-timeline` | Requêtes chronologiques, pagination, filtres | ⚪ Phase 3 |
| `evidentyx-correlation` | Règles de corrélation déterministes | ⚪ Phase 3 |
| `evidentyx-graph` | Construction du graphe d'entités et de relations | ⚪ Phase 3 |
| `evidentyx-detection` | Moteur de règles de détection versionnées | ⚪ Phase 4 |
| `evidentyx-report` | Exports PDF, JSON, CSV | ⚪ Phase 4 |

Les crates ne sont créés qu'au moment où ils contiennent du code réel.

## Commandes Tauri

| Commande | Rôle |
|---|---|
| `engine_info` | Versions du moteur et de l'application, plateforme |
| `storage_info` | Chemin de la base, version du schéma, erreur d'ouverture éventuelle |
| `list_investigations` | Liste filtrée (statut, texte) et triée |
| `get_investigation` | Détail d'une enquête |
| `create_investigation`, `rename_investigation`, `update_investigation_description` | Création et modification (validation par `evidentyx-core`) |
| `set_investigation_status` | Archivage, réactivation |
| `delete_investigation` | Suppression définitive |

Les erreurs sont renvoyées sous la forme `{ code, message, field? }` : `code` est stable (`validation`, `not_found`, `archived_read_only`…), `message` est en français, `field` désigne le champ de formulaire concerné. La base `evidentyx.sqlite` se trouve dans le dossier de données de l'application (par exemple `~/.local/share/org.evidentyx.desktop/` sous Linux).

## Conventions linguistiques

Code en anglais, documentation et messages utilisateur en français. Les messages d'erreur du domaine (`CoreError`) sont formulés en français car ils remontent jusqu'à l'analyste.
