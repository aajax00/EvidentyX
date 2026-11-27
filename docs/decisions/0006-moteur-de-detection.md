# ADR 0006 — Moteur de détection

- Statut : accepté
- Date : 2026-11-27

## Contexte

La phase 4 doit signaler automatiquement des comportements suspects. Exigences du projet : déterministe (aucune IA générative), explicable (chaque détection renvoie aux événements qui la justifient), hors ligne.

## Décision

- **Règles déclaratives** (`crates/storage/src/detection.rs`, `RULES`) : identifiant, **version**, titre, description, gravité, sélection (catégorie, issue), mesure (nombre d'événements, comptes distincts, cibles distinctes), seuil et fenêtre de temps. Toute modification de la logique d'une règle incrémente sa version ; la détection porte l'identifiant et la version de la règle.
- **Évaluation** : événements sélectionnés en SQL, regroupés par adresse source et triés par heure ; fenêtre glissante en Rust. Une détection couvre une période continue où le seuil est atteint (une rafale = une détection). Mêmes données, mêmes règles : mêmes détections.
- **Justification** : chaque détection garde la période, le nombre d'événements, le pic de la mesure et les identifiants des 50 premiers événements.
- **Premières règles** : échecs d'authentification répétés (10 en 5 min), pulvérisation de mots de passe (5 comptes en 10 min), balayage réseau (20 cibles en 1 min).
- Détections **calculées à la demande** (commande `run_detections`), non enregistrées : rien à invalider quand une preuve est réanalysée ou retirée.

## Conséquences

- Seuils fixes pour l'instant ; règles modifiables par l'utilisateur et import de règles (Sigma) à envisager plus tard.
- Les événements sans heure ou sans adresse source ne sont pas évalués.
- Affichage des détections : étape 4.2.
