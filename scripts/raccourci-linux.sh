#!/usr/bin/env bash
# Linux : déclare l'application lancée depuis les sources (`npm run tauri dev`)
# auprès du bureau, pour que la barre des tâches affiche le logo d'EVIDENTYX.
#
# La fenêtre s'annonce sous le nom du binaire (`evidentyx-desktop`) : le bureau
# cherche un fichier `evidentyx-desktop.desktop`. Les paquets (.deb, .rpm)
# installent le leur ; ce script ne sert qu'aux versions compilées localement.
#
# Retirer : rm ~/.local/share/applications/evidentyx-desktop.desktop
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
dir="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
file="$dir/evidentyx-desktop.desktop"
mkdir -p "$dir"
cat > "$file" <<EOF
[Desktop Entry]
Type=Application
Name=EVIDENTYX (développement)
Comment=Investigation numérique et reconstruction d'incidents
Exec="$root/target/debug/evidentyx-desktop"
Icon=$root/apps/desktop/src-tauri/icons/icon.png
StartupWMClass=evidentyx-desktop
Terminal=false
NoDisplay=true
Categories=Utility;
EOF
command -v update-desktop-database >/dev/null && update-desktop-database "$dir" >/dev/null 2>&1 || true
echo "Raccourci créé : $file (relancez l'application)."
