# Architecture d'EVIDENTYX

> Document vivant : il décrit l'état **actuel** du code et signale ce qui est planifié.

## Vue d'ensemble

```
┌───────────────────────────────────────────────────────────┐
│  Interface (React + TypeScript, système de design maison)  │  planifié
└──────────────────────────┬────────────────────────────────┘
                           │ commandes Tauri typées
┌──────────────────────────▼────────────────────────────────┐
│  Hôte de bureau (Tauri 2, Rust)                           │  planifié
└──────────────────────────┬────────────────────────────────┘
                           │ appels de bibliothèque
┌──────────────────────────▼────────────────────────────────┐
│  Moteur d'analyse (workspace Cargo)                       │
│  core · parser · normalizer · timeline · correlation      │
│  graph · detection · evidence · storage · report          │
└──────────────────────────┬────────────────────────────────┘
                           │
                ┌──────────▼──────────┐
                │  SQLite (local)     │  planifié
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
| `evidentyx-storage` | Persistance SQLite, migrations, journal d'audit | ⚪ Planifié (phase 1) |
| `evidentyx-parser` | Analyseurs de formats (JSON, CSV, Syslog, Apache, Nginx, PCAP, EVTX) | ⚪ Phase 2 |
| `evidentyx-normalizer` | Conversion vers le modèle d'événement canonique | ⚪ Phase 2-3 |
| `evidentyx-evidence` | Hachage SHA-256, vérification d'intégrité | ⚪ Phase 2 |
| `evidentyx-timeline` | Requêtes chronologiques, pagination, filtres | ⚪ Phase 3 |
| `evidentyx-correlation` | Règles de corrélation déterministes | ⚪ Phase 3 |
| `evidentyx-graph` | Construction du graphe d'entités et de relations | ⚪ Phase 3 |
| `evidentyx-detection` | Moteur de règles de détection versionnées | ⚪ Phase 4 |
| `evidentyx-report` | Exports PDF, JSON, CSV | ⚪ Phase 4 |

Les crates ne sont créés qu'au moment où ils contiennent du code réel.

## Conventions linguistiques

Code en anglais, documentation et messages utilisateur en français. Les messages d'erreur du domaine (`CoreError`) sont formulés en français car ils remontent jusqu'à l'analyste.
