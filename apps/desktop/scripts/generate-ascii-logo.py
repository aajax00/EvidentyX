#!/usr/bin/env python3
"""Génère la version ASCII du logo (src/assets/logoAscii.ts) depuis docs/assets/logo.png.

Usage : python3 scripts/generate-ascii-logo.py [colonnes]
Chaque cellule reçoit un caractère selon la densité d'encre et un niveau de
couleur (1 = bleu nuit, 2 = bleu, 3 = bleu vif) d'après la luminosité du logo.
Nécessite Pillow.
"""
import colorsys
import sys
from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parents[3]
SRC = ROOT / "docs/assets/logo.png"
OUT = Path(__file__).resolve().parents[1] / "src/assets/logoAscii.ts"
COLS = int(sys.argv[1]) if len(sys.argv) > 1 else 84
RAMP = " .:-=+*#%@"
CELL_ASPECT = 2.0  # une cellule de texte est environ deux fois plus haute que large

im = Image.open(SRC).convert("RGBA")
alpha = im.getchannel("A").point(lambda v: 255 if v > 40 else 0)
im = im.crop(alpha.getbbox())
w, h = im.size
rows = max(1, round(COLS * h / w / CELL_ASPECT))
small = im.resize((COLS, rows), Image.LANCZOS)

chars, levels = [], []
for y in range(rows):
    line, lv = [], []
    for x in range(COLS):
        r, g, b, a = small.getpixel((x, y))
        if a < 60:
            line.append(" ")
            lv.append("0")
            continue
        _, s, v = colorsys.rgb_to_hsv(r / 255, g / 255, b / 255)
        density = a / 255 * (0.55 + 0.45 * (1 - v if s > 0.25 else v))
        line.append(RAMP[min(len(RAMP) - 1, max(1, round(density * (len(RAMP) - 1))))])
        lv.append("1" if v < 0.35 else "3" if v > 0.7 else "2")
    chars.append("".join(line).rstrip())
    levels.append("".join(lv)[: len(chars[-1])])

def ts_array(lines):
    return "\n".join(f"  {line!r}," for line in lines)

OUT.write_text(
    "// Fichier généré par scripts/generate-ascii-logo.py — ne pas modifier à la main.\n\n"
    "/** Logo en caractères ASCII, ligne par ligne. */\n"
    f"export const LOGO_ASCII: readonly string[] = [\n{ts_array(chars)}\n];\n\n"
    "/** Niveau de couleur de chaque caractère : 0 vide, 1 bleu nuit, 2 bleu, 3 bleu vif. */\n"
    f"export const LOGO_ASCII_LEVELS: readonly string[] = [\n{ts_array(levels)}\n];\n",
    encoding="utf-8",
)
print(f"{COLS}×{rows} → {OUT}")
