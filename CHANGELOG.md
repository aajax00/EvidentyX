# Journal des modifications

Toutes les modifications notables de ce projet sont consignées dans ce fichier.

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/) et le projet respecte le [versionnage sémantique](https://semver.org/lang/fr/).

## [Non publié]

### Ajouté

- Filtres de la chronologie : recherche dans les messages (sans casse), catégorie, preuve, issue, début d'adresse IP (source ou destination), utilisateur, hôte, période (heure locale, bornes incluses) ; nombre d'événements retenus affiché ; bouton « Réinitialiser » ; commande `events_count`.
- Chronologie d'une enquête (bouton « Chronologie ») : tous les événements dans l'ordre chronologique, sans heure à la fin ; liste virtualisée qui reste fluide sur des millions d'événements (seules les lignes visibles sont chargées) ; détail d'un événement (champs normalisés, attributs d'origine, texte brut, preuve, ligne et octet) ; commande `events_window`.

## [0.2.0] - 2026-11-25

Fin de la phase 2 (Ingestion) : les preuves s'ajoutent à une enquête, leur intégrité est vérifiée et leurs événements sont extraits.

### Corrigé

- CSV : un en-tête contenant une colonne en double (CIC-IDS2017 répète `Fwd Header Length`) était lu comme une ligne de données ; colonnes alors mal nommées.
- CSV à nombreuses colonnes environ 4 fois plus rapide (correspondance des champs en une passe, analyse des noms de champs mémorisée).
- Fenêtre figée (« Forcer à quitter ») lors de la suppression d'une preuve ou d'une enquête et à l'ouverture d'une enquête volumineuse : les accès à la base se font hors du fil de la fenêtre, avec indicateur d'attente.
- Suppression très lente : attributs d'événement regroupés en une colonne (migration 0005, données conservées ; une base volumineuse peut demander une minute ou plus au premier lancement).

### Ajouté

- Vue d'ensemble réelle d'une enquête : les données de démonstration sont remplacées par les statistiques des événements extraits (événements, preuves analysées, adresses IP et utilisateurs distincts, période couverte, activité dans le temps, principales sources, catégories) ; recalculée après chaque ajout, analyse ou retrait. Migration 0006 : index des statistiques (création longue une seule fois sur une grosse base ; 1,7 million d'événements : 95 s → 1,9 s).
- Import suivi : barre de progression (pourcentage, octets lus) pendant le calcul d'empreinte et l'analyse, bouton d'annulation (une analyse annulée ne laisse aucun de ses événements), rapport d'analyse détaillé (format, structure, enregistrements lus, événements retenus, déjà présents, illisibles avec leur ligne ou paquet et leur décalage).
- Preuves : ajout de fichiers à une enquête (sélecteur ou glisser-déposer), empreinte SHA-256 calculée en flux et en lecture seule, taille et dates ; vérification d'intégrité (intègre, modifiée, introuvable) ; retrait sans jamais toucher au fichier ; contenu en double refusé ([ADR 0003](docs/decisions/0003-preuves-et-integrite.md)).
- Crate `evidentyx-evidence` (lecture en flux, progression, annulation) ; migration 0003 (table `evidence`) ; commandes `list_evidence`, `add_evidence`, `verify_evidence`, `remove_evidence`.
- Modèle d'événement canonique (`evidentyx-core`) : référence exacte à la source (preuve, décalage, ligne ou enregistrement), horodatage UTC avec précision et base (fuseau explicite, supposé UTC, année déduite), champs normalisés, texte brut et attributs d'origine conservés ([ADR 0004](docs/decisions/0004-modele-evenement.md)).
- Stockage des événements (migration 0004) : insertion par lot transactionnelle sans doublon au réimport, chronologie paginée déterministe, suppression en cascade ; nombre d'événements affiché pour chaque preuve.
- Analyse des preuves : bouton « Analyser » qui reconnaît le format et extrait les événements ; lecture refusée si la taille ou l'empreinte SHA-256 du fichier ne correspond plus (événements retirés, preuve marquée « modifiée ») ; réanalyse sans doublon ; bilan (événements, lignes illisibles avec leur numéro) ([ADR 0005](docs/decisions/0005-cadre-de-lecture.md)).
- Crate `evidentyx-parser` : cadre commun (détection, lecture en flux, erreurs par enregistrement, progression, annulation), correspondance des champs vers le modèle d'événement, lecture des heures ISO 8601 / RFC 3339 et époque Unix ; formats JSON Lines, tableau JSON et objets JSON concaténés.
- Lecture CSV (crate `csv`, RFC 4180) : séparateur deviné (virgule, point-virgule, tabulation, barre verticale), en-tête reconnu ou colonnes numérotées, champs entre guillemets et sur plusieurs lignes, fins de ligne CRLF (ligne et décalage exacts), colonne d'heure choisie automatiquement (colonnes date et heure séparées combinées), noms de colonnes français reconnus, sorties CSV d'EvtxECmd (EZ Tools) reconnues ; colonne d'heure et structure indiquées dans le bilan d'analyse.
- Lecture Syslog : RFC 5424 (données structurées), RFC 3164 (année déduite de la date du fichier et signalée) et format fichier de rsyslog ; utilisateur, adresse et issue repris des messages de sshd, sudo et du pare-feu du noyau quand ils y figurent.
- Lecture des journaux d'accès Apache et Nginx (formats common et combined, préfixe `vhost:port`, champs supplémentaires de Nginx) : adresse du client, utilisateur, méthode, chemin, code de réponse et issue, taille, référent et navigateur ; requêtes non standard (sondes) conservées telles quelles.
- Lecture des captures réseau PCAP et PCAPNG : un événement par paquet avec heure de capture, adresses, ports, protocole (TCP, UDP, ICMP, ARP…), drapeaux TCP, VLAN et clé de flux identique dans les deux sens ; capture interrompue lue jusqu'au dernier paquet complet.

## [0.1.0] - 2026-11-19

Première version : fin de la phase 1 (Fondations). L'application gère les enquêtes ; l'analyse des preuves arrive avec la phase 2.

### Ajouté

**Enquêtes**

- Création, ouverture, renommage, modification de la description, archivage (lecture seule), réactivation et suppression avec confirmation.
- Liste avec recherche dans le nom et la description (insensible aux accents et à la casse), filtre par statut et tri (dernière modification, création, nom).
- Page de détail : informations, description, vue d'ensemble illustrée par des **données de démonstration** clairement étiquetées, modules à venir signalés comme planifiés.
- Enregistrement dans une base SQLite locale, dans le dossier de données de l'application ; l'application démarre même si la base est inaccessible et l'indique.

**Interface**

- Application de bureau Tauri 2 + React 19 + TypeScript strict, entièrement hors ligne, politique de sécurité du contenu stricte.
- Accueil (statistiques, enquêtes récentes, feuille de route), barre latérale repliable, fil d'Ariane, barre d'état (moteur, base, nombre d'enquêtes), palette de commandes.
- Raccourcis : Ctrl+K / ⌘K (palette), Ctrl+N / ⌘N (nouvelle enquête), Ctrl+B / ⌘B (barre latérale), « / » (recherche).
- Paramètres enregistrés : écran d'ouverture, animation d'arrière-plan, barre latérale au démarrage, tri et statut par défaut de la liste.
- Identité visuelle : logo bleu, thème sombre sobre aux angles droits, polices IBM Plex Sans et Mono embarquées.
- Animations : écran d'ouverture (le logo se décode en ASCII), réseau de points discret en arrière-plan, ouverture des boîtes de dialogue et des pages en fondu ; désactivables, et coupées si le système demande moins d'animations.
- Système de design : jetons CSS, composants (`Button`, `IconButton`, `Badge`, `Panel`, `Dialog`, `TextField`, `TextAreaField`, `Select`, `SegmentedControl`, `SearchField`, `Switch`, `EmptyState`, `Spinner`, notifications) et catalogue de développement.
- Accessibilité : navigation au clavier, focus visible, libellés reliés aux champs, erreurs annoncées.

**Moteur**

- `evidentyx-core` : modèle des enquêtes et règles de validation, recherche et tri, normalisation de texte sans accents, réglages ; aucune dépendance externe ; messages d'erreur en français.
- `evidentyx-storage` : SQLite embarqué, migrations versionnées (schéma 2), contraintes en base, identifiants jamais réutilisés, revalidation des données relues ([ADR 0002](docs/decisions/0002-stockage-sqlite.md)).
- Hôte Tauri : commandes typées pour les enquêtes, les réglages et l'état du stockage ; erreurs typées avec le champ concerné.

**Distribution**

- Paquets Linux (AppImage, DEB, RPM), Windows (installateur, MSI) et macOS (DMG universel), construits par le workflow « Publication » ; **non signés** dans cette version (voir le [guide d'installation](docs/installation.md)).
- Version unique contrôlée par `scripts/version.mjs`.

**Projet**

- Licence Apache 2.0, guide de contribution, politique de sécurité, architecture, feuille de route, décisions techniques (ADR 0001, 0002), procédure de publication.
- Intégration continue : formatage, Clippy, tests Rust et de l'interface (Vitest) sur Linux, Windows et macOS.
- Mode démonstration pour développer l'interface dans un navigateur (signalé à l'écran, absent des paquets).
- Version minimale de Rust : 1.90.

[Non publié]: https://github.com/aajax00/EvidentyX/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/aajax00/EvidentyX/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/aajax00/EvidentyX/releases/tag/v0.1.0
