# ADR 0001 — Workspace Cargo, conventions de langue et de versionnage

- **Statut** : acceptée
- **Date** : 2026-11-18

## Contexte

EVIDENTYX combine un moteur d'analyse en Rust et une interface de bureau Tauri 2 + React. Le projet est open source, destiné à une communauté francophone en priorité, et doit rester maintenable sur la durée.

## Décisions

1. **Workspace Cargo unique** à la racine, avec un crate par responsabilité (`crates/<nom>`, paquet `evidentyx-<nom>`). L'hôte Tauri (`apps/desktop/src-tauri`) rejoindra ce workspace pour partager le verrouillage des dépendances.
2. **Création progressive des crates** : un crate n'existe que lorsqu'il contient du code réel et testé.
3. **`evidentyx-core` sans dépendance externe** au départ : types de domaine purs, sans entrée/sortie. Les dépendances (sérialisation, identifiants, dates) seront ajoutées quand un besoin concret apparaîtra, avec justification.
4. **`unsafe` interdit** dans tout le workspace (`unsafe_code = "forbid"`).
5. **Langue** : code en anglais ; documentation, interface, messages d'erreur utilisateur et messages de commit en français.
6. **Versionnage** : SemVer, Conventional Commits, Keep a Changelog, étiquettes annotées `vX.Y.Z`, une version mineure par phase.
7. **MSRV** : Rust 1.80, vérifiée en CI à terme.

## Conséquences

- Les contributeurs non francophones peuvent lire le code ; l'expérience utilisateur reste entièrement francophone.
- L'historique Git permettra de générer automatiquement le journal des modifications.
