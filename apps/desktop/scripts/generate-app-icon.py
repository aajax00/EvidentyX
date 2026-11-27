#!/usr/bin/env python3
"""Icône de l'application : logo sur un carreau clair aux coins arrondis, avec relief.

Produit `src-tauri/icons/source.png` (1024 px) depuis `docs/assets/logo.png`.
Les tailles sont ensuite générées par `npx tauri icon src-tauri/icons/source.png`.

Lancer depuis `apps/desktop` : `python3 scripts/generate-app-icon.py` (Pillow).
"""

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

SIZE = 1024
MARGIN = 56  # place pour l'ombre portée du carreau
RADIUS = 190  # coins arrondis (environ 20 % du carreau)
ROOT = Path(__file__).resolve().parents[3]
LOGO = ROOT / "docs/assets/logo.png"
OUT = ROOT / "apps/desktop/src-tauri/icons/source.png"


def rounded_mask(box: tuple[int, int, int, int]) -> Image.Image:
    mask = Image.new("L", (SIZE, SIZE), 0)
    ImageDraw.Draw(mask).rounded_rectangle(box, RADIUS, fill=255)
    return mask


def vertical_gradient(top: tuple[int, int, int], bottom: tuple[int, int, int]) -> Image.Image:
    grad = Image.new("RGB", (1, SIZE))
    for y in range(SIZE):
        t = y / (SIZE - 1)
        grad.putpixel((0, y), tuple(round(a + (b - a) * t) for a, b in zip(top, bottom)))
    return grad.resize((SIZE, SIZE))


def main() -> None:
    box = (MARGIN, MARGIN, SIZE - MARGIN, SIZE - MARGIN)
    canvas = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))

    # Ombre portée douce sous le carreau.
    shadow = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle((box[0], box[1] + 18, box[2], box[3] + 18), RADIUS, fill=(6, 12, 24, 150))
    canvas.alpha_composite(shadow.filter(ImageFilter.GaussianBlur(22)))

    # Carreau : dégradé clair, plus lumineux en haut.
    mask = rounded_mask(box)
    tile = vertical_gradient((248, 251, 255), (206, 218, 234)).convert("RGBA")
    tile.putalpha(mask)
    canvas.alpha_composite(tile)

    # Biseau : reflet clair en haut, liseré plus sombre en bas.
    inner = rounded_mask((box[0] + 10, box[1] + 10, box[2] - 10, box[3] - 10))
    rim = ImageChops.subtract(mask, inner)
    light = Image.new("RGBA", (SIZE, SIZE), (255, 255, 255, 0))
    light.putalpha(ImageChops.multiply(rim, vertical_gradient((210, 210, 210), (0, 0, 0)).convert("L")))
    dark = Image.new("RGBA", (SIZE, SIZE), (70, 90, 120, 0))
    dark.putalpha(ImageChops.multiply(rim, vertical_gradient((0, 0, 0), (150, 150, 150)).convert("L")))
    canvas.alpha_composite(light)
    canvas.alpha_composite(dark)

    # Logo en relief : ombre portée sombre, reflet clair au-dessus, puis le logo.
    logo = Image.open(LOGO).convert("RGBA")
    width = box[2] - box[0] - 120
    logo = logo.resize((width, round(logo.height * width / logo.width)), Image.LANCZOS)
    x = (SIZE - logo.width) // 2
    y = (SIZE - logo.height) // 2
    alpha = logo.getchannel("A")
    for dx, dy, color, blur in ((0, 14, (8, 20, 45, 120), 10), (0, -3, (255, 255, 255, 170), 2)):
        layer = Image.new("RGBA", logo.size, color)
        layer.putalpha(ImageChops.multiply(alpha, Image.new("L", logo.size, color[3])))
        holder = Image.new("RGBA", (SIZE, SIZE), (0, 0, 0, 0))
        holder.alpha_composite(layer, (x + dx, y + dy))
        canvas.alpha_composite(holder.filter(ImageFilter.GaussianBlur(blur)))
    canvas.alpha_composite(logo, (x, y))

    canvas.save(OUT)
    print(f"Icône source écrite : {OUT}")


if __name__ == "__main__":
    main()
