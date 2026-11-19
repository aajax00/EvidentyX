# Publier une version

Procédure de mainteneur. Les versions suivent le [versionnage sémantique](https://semver.org/lang/fr/) ; chaque phase de la [feuille de route](feuille-de-route.md) se termine par une version mineure (`v0.1.0` = fin de la phase 1).

## Version unique

La version est déclarée dans `Cargo.toml` (`[workspace.package]`) et `apps/desktop/package.json` ; `tauri.conf.json` reprend celle de `package.json`. Le script `scripts/version.mjs` les garde identiques :

```bash
node scripts/version.mjs            # affiche et vérifie la cohérence
node scripts/version.mjs set 0.1.0  # fixe la version partout (lockfiles inclus)
```

La CI vérifie la cohérence à chaque pull request ; le workflow de publication vérifie en plus que l'étiquette correspond à la version.

## Étapes

1. **Préparer** sur une branche `chore/version-X.Y.Z` :
   - `node scripts/version.mjs set X.Y.Z` ;
   - dans `CHANGELOG.md`, renommer « Non publié » en `[X.Y.Z] - AAAA-MM-JJ`, ouvrir une nouvelle section « Non publié » vide et mettre à jour les liens de comparaison ;
   - cocher la version dans `docs/feuille-de-route.md` ;
   - commit `chore: publier la version X.Y.Z`, pull request, fusion.
2. **(Facultatif) Essai à blanc** : onglet *Actions* → *Publication* → *Run workflow* sur `main`. Les paquets sont construits et téléchargeables en artefacts, sans publication.
3. **Étiqueter** `main` :
   ```bash
   git switch main && git pull
   git tag -a vX.Y.Z -m "EVIDENTYX X.Y.Z"
   git push origin vX.Y.Z
   ```
4. **Workflow *Publication*** : vérifie la version, construit les paquets (Linux sur Ubuntu 22.04, Windows, macOS universel) et crée une **publication en brouillon** avec les fichiers joints. Comptez 15 à 25 minutes.
5. **Relire et publier** : sur GitHub, *Releases* → brouillon → vérifier la liste des fichiers et le texte → *Publish release*. Les versions `0.x` sont marquées « pré-version ».

## Paquets produits

| Plateforme | Fichiers |
|---|---|
| Linux | `.AppImage`, `.deb`, `.rpm` |
| Windows | `-setup.exe` (NSIS, par utilisateur), `.msi` (WiX, par poste) |
| macOS | `.dmg` et `.app` universels (Apple et Intel) |

## Limites actuelles

- **Paquets non signés** : avertissements SmartScreen (Windows) et Gatekeeper (macOS). Signature et notarisation prévues en phase 5.
- **Pas de mise à jour automatique** : l'utilisateur télécharge la nouvelle version.
