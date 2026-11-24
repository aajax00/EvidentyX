# Feuille de route

Chaque étape correspond à un ou plusieurs commits ; chaque phase se termine par une version publiée.

## Phase 1 — Fondations → `v0.1.0`

- [x] **Étape 1.1** — Initialisation du dépôt, workspace Cargo, crate `core`, CI, documentation
- [x] **Étape 1.2** — Application Tauri 2 + React + TypeScript (Vite, TypeScript strict)
- [x] **Étape 1.3** — Système de design : variables CSS, typographie, composants de base
- [x] **Étape 1.4** — Coque applicative : barre latérale, en-tête, navigation, palette de commandes
- [x] **Étape 1.5** — Crate `storage` : SQLite, migrations, persistance des enquêtes
- [x] **Étape 1.6** — Gestion des enquêtes : création, ouverture, renommage, archivage, suppression, recherche, tri
- [x] **Étape 1.7** — Vue d'ensemble (données de démonstration clairement étiquetées) et paramètres
- [x] **Étape 1.8** — Empaquetage : workflow de publication GitHub (AppImage, DEB, MSI/EXE, DMG non signés)
- [x] **Version `v0.1.0`** — 2026-11-19

## Phase 2 — Ingestion → `v0.2.0`

Objectif : importer des preuves dans une enquête et en extraire des événements traçables, sans jamais modifier les fichiers sources.

- [x] **Étape 2.1** — Crate `evidence` : enregistrement d'une preuve dans une enquête (chemin, taille, dates), empreinte SHA-256 calculée en flux, vérification d'intégrité ; fichiers ouverts en lecture seule, jamais copiés en base
- [x] **Étape 2.2** — Modèle d'événement canonique : horodatage UTC avec précision et incertitude, référence à la source (preuve, décalage, ligne ou paquet), champs normalisés (hôte, adresses, utilisateur, action, résultat), champs bruts conservés ; stockage des événements (ADR)
- [x] **Étape 2.3** — Crate `parser` : cadre commun (détection du format, lecture en flux, erreurs par ligne sans interrompre l'import), JSON et JSON Lines
- [x] **Étape 2.4** — CSV : séparateur et en-têtes détectés, choix des colonnes d'horodatage
- [x] **Étape 2.5** — Syslog RFC 3164 et RFC 5424
- [x] **Étape 2.6** — Journaux d'accès Apache et Nginx (formats common et combined)
- [x] **Étape 2.7** — PCAP et PCAPNG : paquets, flux réseau de base (adresses, ports, protocoles)
- [ ] **Étape 2.8** — Import dans l'application : sélection ou glisser-déposer, progression, annulation, rapport d'import (lignes lues, retenues, ignorées, erreurs)
- [ ] **Étape 2.9** — Vue d'ensemble réelle : les statistiques de l'enquête remplacent les données de démonstration
- [ ] **Version `v0.2.0`**

## Phase 3 — Moteur d'enquête → `v0.3.0`

Normalisation · horodatages et incertitude · chronologie virtualisée · filtres et recherche · corrélation · graphe d'attaque.

## Phase 4 — Détection et rapports → `v0.4.0`

Règles de détection versionnées · coffre de preuves · journal d'audit · exports PDF, JSON, CSV.

## Phase 5 — Stabilisation → `v1.0.0`

Tests d'intégration et de sécurité · bancs de performance · paquets pour les trois plateformes · documentation utilisateur.
