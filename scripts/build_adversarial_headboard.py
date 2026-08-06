#!/usr/bin/env python3
"""Compose the adversarial-benchmark blog headboard.

A flat-color Venetian poster: racing galleys crossing the Adriatic toward
Rovinj at dusk, led by the Bucintoro flying the banner of St Mark, with
MARCIANA lettered on the flagship's side. Rendered from layered SVG so the
scene is reproducible without any external art asset.

Run with: uv run --with cairosvg -- python3 scripts/build_adversarial_headboard.py
"""

from __future__ import annotations

import math
from pathlib import Path

import cairosvg

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs" / "blog" / "adversarial-benchmark" / "headboard.png"
WIDTH, HEIGHT = 1672, 941
HORIZON = 560

SKY_TOP = "#1c2440"
SKY_MID = "#4c3f63"
SKY_GLOW = "#e8a24a"
SKY_LOW = "#f3c46b"
SEA_TOP = "#2b3a52"
SEA_MID = "#1f2e42"
SEA_DEEP = "#141f30"
LIGHT_PATH = "#e7a94f"
INK = "#10131f"
HULL = "#1a1626"
GOLD = "#d9a74a"
GOLD_DEEP = "#a97a2e"
BANNER = "#8f1f24"
BANNER_DEEP = "#6f171c"
CREAM = "#f4e9c9"
HAZE = "#c78a52"


def defs() -> str:
    return f"""
  <defs>
    <linearGradient id="sky" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{SKY_TOP}"/>
      <stop offset="0.45" stop-color="{SKY_MID}"/>
      <stop offset="0.8" stop-color="{SKY_GLOW}"/>
      <stop offset="1" stop-color="{SKY_LOW}"/>
    </linearGradient>
    <linearGradient id="sea" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="{SEA_TOP}"/>
      <stop offset="0.35" stop-color="{SEA_MID}"/>
      <stop offset="1" stop-color="{SEA_DEEP}"/>
    </linearGradient>
    <radialGradient id="sun" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#f7dfa0" stop-opacity="0.95"/>
      <stop offset="0.55" stop-color="#eeb45e" stop-opacity="0.55"/>
      <stop offset="1" stop-color="#eeb45e" stop-opacity="0"/>
    </radialGradient>
  </defs>"""


def sky_and_sea() -> str:
    sun_x, sun_y = 1210, HORIZON - 12
    return f"""
  <rect x="0" y="0" width="{WIDTH}" height="{HORIZON}" fill="url(#sky)"/>
  <circle cx="{sun_x}" cy="{sun_y}" r="270" fill="url(#sun)"/>
  <circle cx="{sun_x}" cy="{sun_y}" r="46" fill="#f8e7b0"/>
  <rect x="0" y="{HORIZON}" width="{WIDTH}" height="{HEIGHT - HORIZON}" fill="url(#sea)"/>
"""


def light_path() -> str:
    # Scattered low-sun glitter widening toward the viewer, aimed at Rovinj.
    dashes = []
    y = HORIZON + 12.0
    spread = 40.0
    seed = 7
    while y < HEIGHT - 30:
        for _ in range(3):
            seed = (seed * 1103515245 + 12345) % (2**31)
            jitter = (seed % 1000) / 1000 - 0.5
            seed = (seed * 1103515245 + 12345) % (2**31)
            length = 8 + (seed % 100) / 100 * min(60, spread * 0.6)
            x = 1210 + jitter * spread - length / 2
            opacity = max(0.08, 0.4 - (y - HORIZON) / 900)
            dashes.append(
                f'<rect x="{x:.0f}" y="{y:.0f}" width="{length:.0f}" height="4" '
                f'rx="2" fill="{LIGHT_PATH}" opacity="{opacity:.2f}"/>'
            )
        y += 22
        spread *= 1.18
    return "\n".join(dashes)


def rovinj() -> str:
    """The Rovinj peninsula: a mound of old-town houses under the campanile."""

    base = HORIZON
    tower_x = 1210
    # The old town climbs its hill: a jagged clustered roofline, not a skyline.
    hill = (
        f"M 1030 {base} "
        f"L 1058 {base - 18} L 1072 {base - 30} L 1090 {base - 26} "
        f"L 1104 {base - 44} L 1122 {base - 38} L 1136 {base - 56} "
        f"L 1154 {base - 50} L 1166 {base - 66} L 1186 {base - 60} "
        f"L 1196 {base - 74} L 1226 {base - 74} L 1238 {base - 62} "
        f"L 1258 {base - 68} L 1272 {base - 52} L 1292 {base - 58} "
        f"L 1306 {base - 42} L 1326 {base - 46} L 1340 {base - 30} "
        f"L 1360 {base - 34} L 1376 {base - 18} L 1398 {base} Z"
    )
    windows = []
    seed = 3
    for x in range(1066, 1380, 18):
        seed = (seed * 48271) % (2**31 - 1)
        if seed % 3:
            height = 30 + seed % 32
            windows.append(
                f'<rect x="{x}" y="{base - height}" width="3.5" height="7" '
                f'fill="{SKY_LOW}" opacity="0.5"/>'
            )
    return f"""
  <g>
    <path d="{hill}" fill="{HAZE}" opacity="0.85"/>
    <path d="{hill}" fill="{INK}" opacity="0.18"/>
    <rect x="{tower_x - 10}" y="{base - 152}" width="20" height="82" fill="{HAZE}"/>
    <rect x="{tower_x - 6}" y="{base - 144}" width="4" height="16" fill="{SKY_LOW}" opacity="0.6"/>
    <rect x="{tower_x + 2}" y="{base - 144}" width="4" height="16" fill="{SKY_LOW}" opacity="0.6"/>
    <rect x="{tower_x - 13}" y="{base - 160}" width="26" height="8" fill="{HAZE}"/>
    <path d="M {tower_x - 13} {base - 160} L {tower_x} {base - 206} L {tower_x + 13} {base - 160} Z"
          fill="{HAZE}"/>
    <line x1="{tower_x}" y1="{base - 218}" x2="{tower_x}" y2="{base - 206}"
          stroke="{HAZE}" stroke-width="3"/>
    <circle cx="{tower_x}" cy="{base - 220}" r="3" fill="{HAZE}"/>
    {''.join(windows)}
  </g>"""


def galley(x: float, y: float, scale: float, opacity: float, flip: bool = False) -> str:
    """A racing galley silhouette with a lateen sail and a bank of oars."""

    direction = -1 if flip else 1
    oars = []
    for index in range(7):
        ox = -70 + index * 22
        oars.append(
            f'<line x1="{ox}" y1="6" x2="{ox + 12 * direction}" y2="30" '
            f'stroke="{INK}" stroke-width="3"/>'
        )
    return f"""
  <g transform="translate({x} {y}) scale({scale * direction} {scale})" opacity="{opacity}">
    <path d="M -95 0 Q 0 26 95 0 L 78 14 Q 0 34 -70 16 Z" fill="{INK}"/>
    <path d="M 95 0 Q 112 -10 118 -26 L 104 -6 Z" fill="{INK}"/>
    <line x1="-8" y1="0" x2="30" y2="-108" stroke="{INK}" stroke-width="4"/>
    <path d="M 34 -112 Q -20 -96 -58 -18 L 30 -34 Z" fill="{INK}"/>
    <path d="M 34 -112 L 44 -30 L 30 -34 Z" fill="{INK}"/>
    {''.join(oars)}
    <path d="M -95 16 Q -140 22 -180 18 L -95 26 Z" fill="{INK}" opacity="0.35"/>
  </g>"""


def lion_of_st_mark(x: float, y: float, scale: float) -> str:
    """A stylized golden winged lion passant with halo and gospel book."""

    mane = []
    seed = 11
    # Mane spikes cover the back of the head only, leaving the face clear.
    for degrees in range(60, 301, 24):
        seed = (seed * 48271) % (2**31 - 1)
        reach = 24 + seed % 6
        tip = math.radians(degrees)
        left = math.radians(degrees + 11)
        right = math.radians(degrees - 11)
        mane.append(
            f'<path d="M {44 + 15 * math.cos(left):.1f} {-16 + 15 * math.sin(left):.1f} '
            f'L {44 + reach * math.cos(tip):.1f} {-16 + reach * math.sin(tip):.1f} '
            f'L {44 + 15 * math.cos(right):.1f} {-16 + 15 * math.sin(right):.1f} Z"/>'
        )
    feathers = (
        '<path d="M -6 -8 Q -50 -72 -88 -60 Q -56 -48 -6 -8 Z"/>'
        '<path d="M -6 -8 Q -56 -52 -80 -38 Q -50 -32 -6 -8 Z"/>'
        '<path d="M -6 -8 Q -52 -32 -68 -18 Q -44 -14 -6 -8 Z"/>'
        '<path d="M -6 -8 Q -44 -16 -54 -2 Q -36 0 -6 -8 Z"/>'
    )
    return f"""
  <g transform="translate({x} {y}) scale({scale})" fill="{GOLD}">
    <circle cx="48" cy="-22" r="34" fill="none" stroke="{GOLD}" stroke-width="3.5"
            opacity="0.9"/>
    {feathers}
    <path d="M -54 6 C -70 2 -74 -12 -66 -24" fill="none"
          stroke="{GOLD}" stroke-width="5" stroke-linecap="round"/>
    <path d="M -66 -24 Q -78 -32 -72 -18 Q -70 -12 -60 -18 Z"/>
    <ellipse cx="-10" cy="8" rx="45" ry="18"/>
    <rect x="-48" y="16" width="8" height="24" rx="3"/>
    <rect x="-28" y="19" width="8" height="21" rx="3"/>
    <rect x="2" y="19" width="8" height="21" rx="3"/>
    <rect x="26" y="12" width="8" height="20" rx="3"/>
    {''.join(mane)}
    <circle cx="44" cy="-16" r="15"/>
    <path d="M 48 -27 L 52 -38 L 58 -25 Z"/>
    <path d="M 33 -28 L 36 -39 L 43 -27 Z"/>
    <path d="M 54 -23 Q 76 -22 77 -13 Q 76 -5 64 -4 Q 55 -5 52 -11 Z"/>
    <path d="M 62 -7 Q 70 -5 73 -10" fill="none" stroke="{BANNER}"
          stroke-width="1.8" stroke-linecap="round"/>
    <circle cx="52" cy="-22" r="2.6" fill="{BANNER}"/>
    <rect x="32" y="28" width="30" height="13" rx="2"/>
    <line x1="47" y1="28" x2="47" y2="41" stroke="{BANNER}" stroke-width="1.6"/>
  </g>"""


def bucintoro() -> str:
    """The ducal flagship: gilded double-deck hull, red canopy, the banner."""

    x, y = 470, 760
    return f"""
  <g transform="translate({x} {y})">
    <path d="M -250 0 Q 0 44 250 0 L 224 30 Q 0 62 -218 34 Z" fill="{HULL}"/>
    <path d="M -250 0 Q 0 44 250 0 L 244 8 Q 0 50 -244 8 Z" fill="{GOLD_DEEP}"/>
    <rect x="-210" y="-46" width="420" height="46" rx="8" fill="{HULL}"/>
    <g fill="{GOLD}">
      {' '.join(f'<path d="M {ox} -12 L {ox} -30 Q {ox + 9} -40 {ox + 18} -30 L {ox + 18} -12 Z"/>' for ox in range(-192, 180, 36))}
    </g>
    <rect x="-210" y="-52" width="420" height="8" fill="{GOLD}"/>
    <rect x="-186" y="-88" width="372" height="36" rx="6" fill="{BANNER_DEEP}"/>
    <rect x="-186" y="-94" width="372" height="8" fill="{GOLD}"/>
    <path d="M 250 0 Q 286 -20 292 -62 Q 268 -30 246 -14 Z" fill="{GOLD}"/>
    <path d="M -250 0 Q -282 -16 -286 -50 Q -266 -24 -246 -10 Z" fill="{GOLD}"/>
    <text x="0" y="24" text-anchor="middle" font-family="Georgia, serif"
          font-size="30" font-weight="bold" letter-spacing="6"
          fill="{GOLD}">MARCIANA</text>
    <line x1="120" y1="-94" x2="120" y2="-300" stroke="{HULL}" stroke-width="7"/>
    <path d="M 124 -300 L 124 -186 L 330 -186 Q 312 -243 330 -300 Z" fill="{BANNER}"/>
    <path d="M 124 -300 L 124 -292 L 322 -292 L 324 -300 Z" fill="{GOLD}" opacity="0.6"/>
    {lion_of_st_mark(234, -237, 0.82)}
    <path d="M -250 34 Q -320 44 -390 38 L -250 52 Z" fill="{INK}" opacity="0.3"/>
  </g>"""


def waves() -> str:
    rows = []
    for index in range(14):
        y = HORIZON + 30 + index * 26
        amplitude = 3 + index * 0.4
        points = []
        for x in range(0, WIDTH + 40, 40):
            offset = amplitude * math.sin(x / 90 + index * 1.7)
            points.append(f"{x},{y + offset:.1f}")
        rows.append(
            f'<polyline points="{" ".join(points)}" fill="none" '
            f'stroke="{CREAM}" stroke-width="1.4" opacity="{0.05 + index * 0.008:.3f}"/>'
        )
    return "\n".join(rows)


def scene() -> str:
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}"
     viewBox="0 0 {WIDTH} {HEIGHT}">
  {defs()}
  {sky_and_sea()}
  {light_path()}
  {rovinj()}
  {waves()}
  {galley(1010, 646, 0.52, 0.85)}
  {galley(1210, 688, 0.66, 0.9)}
  {galley(860, 700, 0.78, 0.95)}
  {bucintoro()}
</svg>"""


def main() -> None:
    OUTPUT.parent.mkdir(parents=True, exist_ok=True)
    cairosvg.svg2png(
        bytestring=scene().encode("utf-8"),
        write_to=str(OUTPUT),
        output_width=WIDTH,
        output_height=HEIGHT,
    )
    print(f"wrote {OUTPUT}")


if __name__ == "__main__":
    main()
