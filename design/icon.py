#!/usr/bin/env python3
"""Draws the app icon from the same shapes as assets/mark.svg. Needs Pillow."""

from pathlib import Path

from PIL import Image, ImageDraw

ASSETS = Path(__file__).parent.parent / "crates/solferry-desktop/assets"
BRAND = "#8580FF"
GROUND = "#18181B"
SCALE = 4
SIZE = 1024
MARGIN = 100
CORNER = 186


def icon():
    side = SIZE * SCALE
    image = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    inset = MARGIN * SCALE
    draw.rounded_rectangle((inset, inset, side - inset, side - inset), CORNER * SCALE, fill=GROUND)
    unit = (SIZE - 2 * MARGIN - 200) * SCALE / 64
    origin = (MARGIN + 100) * SCALE
    at = lambda value: origin + value * unit
    draw.pieslice((at(12), at(14), at(52), at(54)), 180, 360, fill=BRAND)
    for left, top, width in ((8, 40, 48), (16, 48, 32), (24, 56, 16)):
        bar = (at(left), at(top), at(left + width), at(top + 4))
        draw.rounded_rectangle(bar, 2 * unit, fill=BRAND)
    return image.resize((SIZE, SIZE), Image.Resampling.LANCZOS)


def main():
    image = icon()
    image.save(ASSETS / "icon.png")
    sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    image.save(ASSETS / "icon.ico", sizes=sizes)


if __name__ == "__main__":
    main()
