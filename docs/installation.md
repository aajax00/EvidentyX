# Installer EVIDENTYX

Les paquets sont joints à chaque [publication GitHub](https://github.com/aajax00/EvidentyX/releases). EVIDENTYX fonctionne entièrement hors ligne : aucune connexion n'est nécessaire après l'installation.

> ⚠️ Les versions 0.x sont des versions de développement. Les paquets ne sont **pas encore signés** (signature prévue en phase 5) : Windows et macOS affichent un avertissement à la première ouverture.

## Linux

| Fichier | Distributions | Installation |
|---|---|---|
| `EVIDENTYX_X.Y.Z_amd64.AppImage` | Toutes | `chmod +x EVIDENTYX_*.AppImage` puis lancer le fichier |
| `EVIDENTYX_X.Y.Z_amd64.deb` | Debian, Ubuntu, Mint… | `sudo apt install ./EVIDENTYX_*.deb` |
| `EVIDENTYX-X.Y.Z-1.x86_64.rpm` | Fedora, openSUSE… | `sudo dnf install ./EVIDENTYX-*.rpm` |

Le paquet `.deb` installe automatiquement les bibliothèques nécessaires (WebKitGTK). L'AppImage les attend déjà présentes sur le système, ce qui est le cas sur la plupart des bureaux GNOME et KDE récents.

## Windows

- `EVIDENTYX_X.Y.Z_x64-setup.exe` : installation pour l'utilisateur courant, sans droits administrateur.
- `EVIDENTYX_X.Y.Z_x64_fr-FR.msi` : installation pour tout le poste (déploiement en entreprise).

Au premier lancement, SmartScreen peut afficher « Windows a protégé votre ordinateur » : cliquer sur **Informations complémentaires**, puis **Exécuter quand même**.

## macOS

`EVIDENTYX_X.Y.Z_universal.dmg` fonctionne sur les Mac Apple (M1 et suivants) et Intel, macOS 11 ou plus récent. Ouvrir le `.dmg` et glisser EVIDENTYX dans **Applications**.

Au premier lancement, Gatekeeper refuse une application non signée : faire un clic droit sur EVIDENTYX dans **Applications**, choisir **Ouvrir**, puis confirmer.

## Données

La base locale `evidentyx.sqlite` est créée au premier lancement dans le dossier de données de l'application :

| Système | Dossier |
|---|---|
| Linux | `~/.local/share/org.evidentyx.desktop/` |
| Windows | `%APPDATA%\org.evidentyx.desktop\` |
| macOS | `~/Library/Application Support/org.evidentyx.desktop/` |

Désinstaller l'application ne supprime pas ce dossier : vos enquêtes sont conservées.
