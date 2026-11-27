#!/usr/bin/env bash
# Linux : déclare l'application lancée depuis les sources (`npm run tauri dev`)
# auprès du bureau, pour que la barre des tâches affiche le logo d'EVIDENTYX.
#
# La fenêtre s'annonce sous le nom du binaire (`evidentyx-desktop`) : le bureau
# cherche un fichier `evidentyx-desktop.desktop`. Les paquets (.deb, .rpm)
# installent le leur ; ce script ne sert qu'aux versions compilées localement.
#
# L'icône est copiée sous un nom qui contient son empreinte : quand elle change,
# le chemin change et le bureau ne peut pas resservir l'ancienne image en cache.
#
# Retirer : rm ~/.local/share/applications/evidentyx-desktop.desktop \
#   ~/.local/share/icons/evidentyx-desktop-*.png
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
data="${XDG_DATA_HOME:-$HOME/.local/share}"
apps="$data/applications"
icons="$data/icons"
src="$root/apps/desktop/src-tauri/icons/icon.png"
mkdir -p "$apps" "$icons"

hash="$(sha256sum "$src" | cut -c1-12)"
icon="$icons/evidentyx-desktop-$hash.png"
find "$icons" -maxdepth 1 -name 'evidentyx-desktop-*.png' ! -name "evidentyx-desktop-$hash.png" -delete
cp "$src" "$icon"

cat > "$apps/evidentyx-desktop.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=EVIDENTYX (développement)
Comment=Investigation numérique et reconstruction d'incidents
Exec="$root/target/debug/evidentyx-desktop"
Icon=$icon
StartupWMClass=evidentyx-desktop
Terminal=false
NoDisplay=true
Categories=Utility;
EOF
command -v update-desktop-database >/dev/null && update-desktop-database "$apps" >/dev/null 2>&1 || true
echo "Raccourci installé : $apps/evidentyx-desktop.desktop (icône $icon). Relancez l'application."
