# ADR 0003 — Preuves : références, empreinte SHA-256 et intégrité

- **Statut** : acceptée
- **Date** : 2026-11-19

## Contexte

La phase 2 commence par l'enregistrement des fichiers de preuve (journaux, captures réseau) dans une enquête. Ces fichiers peuvent peser plusieurs gigaoctets et doivent rester inaltérés : leur valeur probante repose sur la certitude qu'ils n'ont pas changé depuis leur collecte.

## Décisions

1. **Références, pas de copie** : la base enregistre le chemin absolu canonique, le nom, la taille, la date de modification du fichier et son empreinte. Le fichier n'est jamais copié, déplacé ni modifié ; retirer une preuve ou supprimer une enquête ne supprime que la référence.
2. **Lecture seule et en flux** : crate `evidentyx-evidence`, fichier ouvert en lecture seule, lu par blocs de 1 Mio (mémoire constante), avec suivi de progression et annulation entre deux blocs.
3. **SHA-256** via le crate [`sha2`](https://crates.io/crates/sha2) (RustCrypto, licence MIT / Apache-2.0, sans code C). Algorithme standard de l'investigation numérique ; testé sur les vecteurs du NIST (FIPS 180-2).
4. **Un contenu par enquête** : contrainte d'unicité (enquête, empreinte). Un second fichier au contenu identique est refusé avec un message explicite (`duplicate`) plutôt que dupliqué.
5. **Vérification d'intégrité à la demande** : l'empreinte est recalculée et comparée ; trois issues — *intègre*, *modifiée*, *introuvable* — enregistrées avec leur date. La vérification est permise même sur une enquête archivée (elle ne modifie pas la preuve) ; l'ajout et le retrait ne le sont pas.
6. **Hors du fil de l'interface** : le calcul d'empreinte s'exécute dans un fil dédié (`spawn_blocking`), sans tenir le verrou de la base, pour que l'application reste réactive.
7. **Sélection des fichiers** : plugin officiel [`tauri-plugin-dialog`](https://v2.tauri.app/plugin/dialog/) (licence MIT / Apache-2.0), permission limitée à l'ouverture (`dialog:allow-open`) ; glisser-déposer natif de Tauri.

## Conséquences

- Si un fichier est déplacé, il apparaît *introuvable* : il faudra une action « relocaliser » (vérifier l'empreinte au nouvel emplacement) — prévue avec l'import (étape 2.8).
- Les chemins non représentables en UTF-8 sont refusés pour l'instant, avec un message clair.
- La progression détaillée et l'annulation dans l'interface arrivent avec l'import (étape 2.8) ; le cœur les prend déjà en charge.
