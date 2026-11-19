# Feuille de route

Chaque étape correspond à un ou plusieurs commits ; chaque phase se termine par une version publiée.

## Phase 1 — Fondations → `v0.1.0`

- [x] **Étape 1.1** — Initialisation du dépôt, workspace Cargo, crate `core`, CI, documentation
- [x] **Étape 1.2** — Application Tauri 2 + React + TypeScript (Vite, TypeScript strict)
- [x] **Étape 1.3** — Système de design : variables CSS, typographie, composants de base
- [x] **Étape 1.4** — Coque applicative : barre latérale, en-tête, navigation, palette de commandes
- [x] **Étape 1.5** — Crate `storage` : SQLite, migrations, persistance des enquêtes
- [x] **Étape 1.6** — Gestion des enquêtes : création, ouverture, renommage, archivage, suppression, recherche, tri
- [ ] **Étape 1.7** — Vue d'ensemble (données de démonstration clairement étiquetées) et paramètres
- [ ] **Étape 1.8** — Empaquetage : workflow de publication GitHub (AppImage, DEB, MSI/EXE, DMG non signés)
- [ ] **Version `v0.1.0`**

## Phase 2 — Ingestion → `v0.2.0`

Modèle d'événement canonique · JSON · CSV · Syslog RFC 3164/5424 · Apache/Nginx · PCAP/PCAPNG · progression, annulation, rapport d'import · SHA-256.

## Phase 3 — Moteur d'enquête → `v0.3.0`

Normalisation · horodatages et incertitude · chronologie virtualisée · filtres et recherche · corrélation · graphe d'attaque.

## Phase 4 — Détection et rapports → `v0.4.0`

Règles de détection versionnées · coffre de preuves · journal d'audit · exports PDF, JSON, CSV.

## Phase 5 — Stabilisation → `v1.0.0`

Tests d'intégration et de sécurité · bancs de performance · paquets pour les trois plateformes · documentation utilisateur.
