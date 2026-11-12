#!/usr/bin/env python3
"""Draws the app icons from the same shapes as assets/mark.svg: the mark as frosted glass lit from
behind, on a dark tile. Needs Pillow."""

from pathlib import Path

from PIL import Image, ImageChops, ImageDraw, ImageFilter

ROOT = Path(__file__).parent.parent
ASSETS = ROOT / "crates/solferry-desktop/assets"
IOS = ROOT / "ios/Solferry/Assets.xcassets/AppIcon.appiconset/icon.png"
ANDROID = ROOT / "android/app/src/main/res"
SCALE = 4
SIZE = 1024
GROUND_TOP = (29, 29, 34, 255)
GROUND_BOTTOM = (19, 19, 22, 255)
GLOW = (123, 116, 255)
HALO = (133, 128, 255)
GLASS_TOP = (204, 201, 255)
GLASS_BOTTOM = (112, 104, 246)
# The mark's drawn bounds in its 64 unit box are x 8..56 and y 14..60, so its center sits below the box's.
CENTER = (32, 37)
SUN = (12, 14, 52, 54)
# Left, top and width of each bar. Lower bars are fainter, like a reflection fading into the water.
BARS = ((8, 40, 48, 0.9), (16, 48, 32, 0.66), (24, 56, 16, 0.44))
# Android draws a launcher icon's layers on a 108dp square and masks it to the inner 72dp.
ANDROID_SIDE = 108
ANDROID_MARK = 43.2
ANDROID_DENSITIES = {"mdpi": 1, "hdpi": 1.5, "xhdpi": 2, "xxhdpi": 3, "xxxhdpi": 4}


def vertical(size, top, bottom):
    """A box of the given size fading from the top color to the bottom one."""
    mask = Image.linear_gradient("L").resize(size)
    return Image.composite(Image.new("RGBA", size, bottom), Image.new("RGBA", size, top), mask)


def scaled(mask, factor):
    return mask.point(lambda value: round(value * factor))


class Canvas:
    """Maps the mark's 64 unit box onto a square of `side` pixels, `unit` pixels per mark unit."""

    def __init__(self, side, unit):
        self.side = side
        self.unit = unit

    def point(self, x, y):
        return (self.side / 2 + (x - CENTER[0]) * self.unit, self.side / 2 + (y - CENTER[1]) * self.unit)

    def box(self, left, top, right, bottom):
        return (*self.point(left, top), *self.point(right, bottom))

    def blank(self, mode="L"):
        return Image.new(mode, (self.side, self.side), 0)

    def shapes(self, outline=0):
        """Each part of the mark as a mask with its opacity, filled or drawn as an outline."""
        sun = self.blank()
        style = {"outline": 255, "width": round(outline)} if outline else {"fill": 255}
        ImageDraw.Draw(sun).pieslice(self.box(*SUN), 180, 360, **style)
        parts = [(sun, self.box(*SUN[:2], SUN[2], 34), 1.0)]
        for left, top, width, alpha in BARS:
            bar = self.blank()
            box = self.box(left, top, left + width, top + 4)
            ImageDraw.Draw(bar).rounded_rectangle(box, 2 * self.unit, **style)
            parts.append((bar, box, alpha))
        return parts

    def shaded(self, outline, top, bottom):
        """Fills every part with a gradient running down that part alone."""
        layer = self.blank("RGBA")
        for mask, box, alpha in self.shapes(outline):
            box = tuple(round(edge) for edge in box)
            size = (box[2] - box[0], box[3] - box[1])
            fill = vertical(size, top, bottom)
            fill.putalpha(ImageChops.multiply(fill.getchannel("A"), scaled(mask.crop(box), alpha)))
            layer.alpha_composite(fill, box[:2])
        return layer

    def ground(self, inset, corner, hairline):
        """The tile, with violet light pooled where the mark sits."""
        tile = self.blank()
        ImageDraw.Draw(tile).rounded_rectangle(
            (inset, inset, self.side - inset, self.side - inset), corner, fill=255
        )
        layer = vertical((self.side, self.side), GROUND_TOP, GROUND_BOTTOM)
        pool = self.blank()
        ImageDraw.Draw(pool).ellipse(self.box(10, 14, 54, 50), fill=round(255 * 0.5))
        layer.alpha_composite(self.colored(pool.filter(ImageFilter.GaussianBlur(9 * self.unit)), GLOW))
        if hairline:
            ring = self.blank()
            ImageDraw.Draw(ring).rounded_rectangle(
                (inset, inset, self.side - inset, self.side - inset), corner, outline=255, width=round(hairline)
            )
            # Light catches only the top of the rim.
            fade = self.blank()
            light = vertical((self.side, self.side * 2 // 5), (255, 255, 255, 30), (255, 255, 255, 0))
            fade.paste(light.getchannel("A"), (0, inset))
            layer.alpha_composite(self.colored(ImageChops.multiply(ring, fade), (255, 255, 255)))
        layer.putalpha(tile)
        return layer

    def mark(self):
        """The mark as frosted glass: a soft halo, a translucent fill and a bright top edge."""
        union = self.blank()
        for mask, _, alpha in self.shapes():
            union = ImageChops.lighter(union, scaled(mask, alpha))
        halo = scaled(union.filter(ImageFilter.GaussianBlur(2.5 * self.unit)), 0.5)
        layer = self.colored(halo, HALO)
        layer.alpha_composite(self.shaded(0, (*GLASS_TOP, 235), (*GLASS_BOTTOM, 160)))
        edge = self.shaded(0.3 * self.unit, (255, 255, 255, 220), (255, 255, 255, 70))
        layer.alpha_composite(edge)
        return layer

    def colored(self, alpha, rgb):
        layer = Image.new("RGBA", (self.side, self.side), (*rgb, 0))
        layer.putalpha(alpha)
        return layer


def tile_icon(margin, corner, fill):
    """A finished icon: `fill` is the share of the tile's width the mark spans."""
    side = SIZE * SCALE
    inset = margin * SCALE
    canvas = Canvas(side, fill * (side - 2 * inset) / 48)
    image = canvas.ground(inset, corner * SCALE, hairline=1.5 * SCALE if margin else 0)
    image.alpha_composite(canvas.mark())
    return image.resize((SIZE, SIZE), Image.Resampling.LANCZOS)


def android_layers():
    """The adaptive icon's background and foreground at each density. The themed icon keeps the
    flat vector mark."""
    for density, factor in ANDROID_DENSITIES.items():
        side = round(ANDROID_SIDE * factor)
        canvas = Canvas(side * SCALE, ANDROID_MARK / 48 * factor * SCALE)
        folder = ANDROID / f"mipmap-{density}"
        folder.mkdir(exist_ok=True)
        for name, layer in (("background", canvas.ground(0, 0, 0)), ("foreground", canvas.mark())):
            layer.resize((side, side), Image.Resampling.LANCZOS).save(folder / f"ic_launcher_{name}.png")


def main():
    desktop = tile_icon(margin=100, corner=186, fill=0.6)
    desktop.save(ASSETS / "icon.png")
    sizes = [(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)]
    desktop.save(ASSETS / "icon.ico", sizes=sizes)
    # iOS masks the corners itself and wants an opaque square.
    tile_icon(margin=0, corner=0, fill=0.6).convert("RGB").save(IOS)
    android_layers()


if __name__ == "__main__":
    main()
