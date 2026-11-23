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
| `evidentyx-parser` | Cadre de lecture des formats vers le modèle d'événement ; JSON / JSON Lines, CSV (Syslog, Apache, Nginx, PCAP à venir) ([ADR 0005](decisions/0005-cadre-de-lecture.md)) | ✅ JSON, CSV |
| `evidentyx-normalizer` | Conversion vers le modèle d'événement canonique | ⚪ Phase 2-3 |
| `evidentyx-evidence` | Lecture en flux et en lecture seule des preuves, empreinte SHA-256, vérification d'intégrité ([ADR 0003](decisions/0003-preuves-et-integrite.md)) | ✅ Initialisé |
| `evidentyx-timeline` | Requêtes chronologiques, pagination, filtres | ⚪ Phase 3 |
| `evidentyx-correlation` | Règles de corrélation déterministes | ⚪ Phase 3 |
| `evidentyx-graph` | Construction du graphe d'entités et de relations | ⚪ Phase 3 |
| `evidentyx-detection` | Moteur de règles de détection versionnées | ⚪ Phase 4 |
| `evidentyx-report` | Exports PDF, JSON, CSV | ⚪ Phase 4 |

Les crates ne sont créés qu'au moment où ils contiennent du code réel.

## Modèle d'événement

Tous les formats de preuve sont convertis vers un **événement canonique** (`evidentyx-core::event`, [ADR 0004](decisions/0004-modele-evenement.md)) : référence exacte à la source (preuve, décalage en octets, ligne ou enregistrement), horodatage UTC en microsecondes avec sa précision et sa base, champs normalisés (hôte, adresses, ports, utilisateur, action, issue), message, texte brut et attributs d'origine. Stockage : tables `events` et `event_attributes` (migration 0004).

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
| `list_evidence`, `add_evidence`, `verify_evidence`, `remove_evidence` | Preuves d'une enquête : liste, ajout avec empreinte SHA-256, vérification d'intégrité, retrait (le fichier n'est jamais touché) |
| `analyze_evidence` | Reconnaît le format d'une preuve, extrait ses événements, vérifie l'empreinte pendant la lecture |
| `get_settings`, `update_settings` | Lecture et enregistrement des réglages (validation par `evidentyx-core`) |

Les erreurs sont renvoyées sous la forme `{ code, message, field? }` : `code` est stable (`validation`, `not_found`, `archived_read_only`…), `message` est en français, `field` désigne le champ de formulaire concerné. La base `evidentyx.sqlite` se trouve dans le dossier de données de l'application (par exemple `~/.local/share/org.evidentyx.desktop/` sous Linux).

## Conventions linguistiques

Code en anglais, documentation et messages utilisateur en français. Les messages d'erreur du domaine (`CoreError`) sont formulés en français car ils remontent jusqu'à l'analyste.
