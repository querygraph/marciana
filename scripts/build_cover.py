#!/usr/bin/env python3
"""Compose the Marciana cover from the generated scene and First Pair mask."""

from __future__ import annotations

from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont, ImageOps


ROOT = Path(__file__).resolve().parents[1]
COVER = ROOT / "docs" / "book" / "cover"
BACKGROUND = COVER / "marciana-background.png"
PUBLISHER_MASK = COVER / "publisher-mask.png"
OUTPUT = COVER / "marciana-cover.png"
SIZE = (1275, 1650)


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    name = "Georgia Bold.ttf" if bold else "Georgia.ttf"
    return ImageFont.truetype(f"/System/Library/Fonts/Supplemental/{name}", size)


def fit_background(image: Image.Image) -> Image.Image:
    image = ImageOps.fit(image.convert("RGB"), SIZE, method=Image.Resampling.LANCZOS)
    shade = Image.new("RGBA", SIZE, (8, 14, 22, 72))
    return Image.alpha_composite(image.convert("RGBA"), shade)


def publisher_mark() -> Image.Image:
    source = Image.open(PUBLISHER_MASK).convert("L")
    source.thumbnail((340, 235), Image.Resampling.LANCZOS)
    source = ImageOps.autocontrast(source)
    # The source is a monochrome mask: white becomes warm copper ink.
    ink = Image.new("RGBA", source.size, (221, 174, 102, 255))
    ink.putalpha(source.point(lambda value: int(value * 0.82)))
    return ink


def add_text(canvas: Image.Image) -> None:
    draw = ImageDraw.Draw(canvas)
    cream = (247, 239, 215, 255)
    copper = (221, 174, 102, 255)
    draw.text((110, 1120), "MARCIANA", font=font(112, bold=True), fill=cream)
    draw.text(
        (116, 1258),
        "Governed Memory for the QueryGraph Semantic Stack",
        font=font(31),
        fill=copper,
    )
    draw.text((118, 1340), "Alexy Khrabrov", font=font(35), fill=cream)
    draw.line((115, 1410, 1160, 1410), fill=copper, width=3)
    draw.text((118, 1445), "FIRST PAIR PRESS", font=font(26, bold=True), fill=cream)


def main() -> None:
    canvas = fit_background(Image.open(BACKGROUND))
    mark = publisher_mark()
    # A subtle halo makes the publisher mark readable against the dark field.
    halo = mark.getchannel("A").filter(ImageFilter.GaussianBlur(8))
    halo_layer = Image.new("RGBA", mark.size, (221, 174, 102, 0))
    halo_layer.putalpha(halo.point(lambda value: int(value * 0.24)))
    canvas.alpha_composite(halo_layer, (935, 70))
    canvas.alpha_composite(mark, (935, 70))
    add_text(canvas)
    canvas.convert("RGB").save(OUTPUT, "PNG", optimize=True)


if __name__ == "__main__":
    main()
