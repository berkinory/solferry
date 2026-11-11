#!/usr/bin/env python3
"""Draws the app icons from the same shapes as assets/mark.svg. Needs Pillow."""

from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).parent.parent
ASSETS = ROOT / "crates/solferry-desktop/assets"
IOS = ROOT / "ios/Solferry/Assets.xcassets/AppIcon.appiconset/icon.png"
BRAND = "#8580FF"
GROUND = "#18181B"
SCALE = 4
SIZE = 1024
# Share of the tile's width the mark spans.
FILL = 0.6
# The mark's drawn bounds in its 64 unit box are x 8..56 and y 14..60, so its center sits below the box's.
CENTER = (32, 37)


def icon(margin, corner):
    side = SIZE * SCALE
    image = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    inset = margin * SCALE
    draw.rounded_rectangle((inset, inset, side - inset, side - inset), corner * SCALE, fill=GROUND)
    unit = FILL * (side - 2 * inset) / 48
    x = lambda value: side / 2 + (value - CENTER[0]) * unit
    y = lambda value: side / 2 + (value - CENTER[1]) * unit
    draw.pieslice((x(12), y(14), x(52), y(54)), 180, 360, fill=BRAND)
    for left, top, width in ((8, 40, 48), (16, 48, 32), (24, 56, 16)):
        bar = (x(left), y(top), x(left + width), y(top + 4))
        draw.rounded_rectangle(bar, 2 * unit, fill=BRAND)
    return image.resize((SIZE, SIZE), Image.Resampling.LANCZOS)


def main():
    desktop = icon(margin=100, corner=186)
    desktop.save(ASSETS / "icon.png")
    sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    desktop.save(ASSETS / "icon.ico", sizes=sizes)
    # iOS masks the corners itself and wants an opaque square.
    icon(margin=0, corner=0).convert("RGB").save(IOS)


if __name__ == "__main__":
    main()
