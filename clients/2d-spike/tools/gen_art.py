#!/usr/bin/env python3
"""Generate the 2D spike's sprite art as SVG.

Everything under ``art/svg`` is produced by this script. Nothing is hand-drawn
and nothing is downloaded: the shapes are flat vector geometry in the palette
sampled from ``presentation/mineworld-default/2D/references``.

Run from anywhere:  python3 clients/2d-spike/tools/gen_art.py

Conventions
-----------
* Everything is authored at 2x the intended on-screen size; the Godot side
  draws the sprites at scale 0.5 so curves stay crisp.
* Each prop records a *ground anchor* (ax, ay) in ``art/props.json``. That is
  the point that sits on the ground plane, which is what the scene positions
  and what the painter's-algorithm sort uses.
* Projection is 2:1 isometric. One world unit is TILE_W x TILE_H screen pixels
  at 1x, so 2 * TILE_W x 2 * TILE_H here.
"""

from __future__ import annotations

import json
import math
import os
import random

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.normpath(os.path.join(HERE, "..", "art", "svg"))
MANIFEST = os.path.normpath(os.path.join(HERE, "..", "art", "props.json"))

# 1x tile footprint; art is authored at 2x.
TILE_W = 128.0
TILE_H = 64.0
SX = TILE_W * 2.0  # world unit width at 2x  (half-width = SX/2 = 128)
SY = TILE_H * 2.0  # world unit height at 2x  (half-height = SY/2 = 64)

# ---------------------------------------------------------------------------
# palette, eyeballed off the four reference plates
# ---------------------------------------------------------------------------
P = {
    "stone_hi": "#EDE6D6",
    "stone": "#DCD4C1",
    "stone_lo": "#C8BEA6",
    "joint": "#C2B69C",
    "quay": "#D3CAB4",
    "quay_lo": "#B6AB90",
    "quay_line": "#9C9075",

    "grass": "#8CC260",
    "grass_lo": "#6CA648",
    "grass_hi": "#A9D579",

    "leaf_lo": "#4C8A3B",
    "leaf": "#6FB544",
    "leaf_hi": "#95CF60",
    "leaf_top": "#B8E282",
    "leaf_out": "#57903A",

    "trunk": "#96683F",
    "trunk_lo": "#795334",

    "wood": "#C48C52",
    "wood_lo": "#9A6A36",
    "wood_hi": "#DCAE74",
    "wood_out": "#9C7040",

    "water": "#3FA8D6",
    "water_lo": "#2B85B6",
    "water_hi": "#7FCBE8",

    "cream": "#F7EFDF",
    "cream_lo": "#E4D8C0",
    "cream_out": "#CFC2A8",

    "rose": "#E0938A",
    "rose_lo": "#C97A70",

    "wall_blue": "#53749E",
    "wall_blue_lo": "#3F5B7F",
    "wall_green": "#437F6B",
    "wall_green_lo": "#33654F",
    "wall_sand": "#E7D9BC",
    "wall_sand_lo": "#CEBD9A",

    "roof_terra": "#C4674A",
    "roof_terra_lo": "#9F4F37",
    "roof_slate": "#6E7D95",
    "roof_slate_lo": "#566379",

    "iron": "#2E3742",
    "iron_hi": "#4A5563",
    "glow": "#FFD98A",
    "glow_soft": "#FFE9B8",

    "glass": "#F3D9A6",
    "glass_lo": "#DFBE85",
    "glass_cool": "#BFD6DE",

    "sign": "#C9A06A",
    "sign_lo": "#A07B49",
    "board": "#3A3A38",
    "board_edge": "#8A6A42",

    "shadow": "#3A3226",
    "white": "#FFFFFF",
}

def _hex(c):
    c = c.lstrip("#")
    return tuple(int(c[i:i + 2], 16) for i in (0, 2, 4))


def _rgb(t):
    return "#%02X%02X%02X" % tuple(max(0, min(255, int(round(v)))) for v in t)


SHADOW_TONE = (74, 56, 40)     # warm brown; shade here is never neutral grey
SUN_TONE = (255, 247, 226)     # warm white


def _mix(c, t, k):
    r, g, b = _hex(c)
    return _rgb((r + (t[0] - r) * k, g + (t[1] - g) * k, b + (t[2] - b) * k))


def sh(c, k):
    """Into shade: darker, and warmer rather than greyer."""
    return _mix(c, SHADOW_TONE, k)


def lt(c, k):
    """Into sunlight."""
    return _mix(c, SUN_TONE, k)


FLOWERS = ["#F2A0B4", "#F4E07A", "#FFFFFF", "#C79BE0", "#F08C6A", "#8FB8EA"]

# ---------------------------------------------------------------------------
# tiny svg helpers
# ---------------------------------------------------------------------------

parts: list[str] = []
props: dict[str, dict] = {}


def e(tag: str, **kw) -> str:
    out = []
    for k, v in kw.items():
        if v is None:
            continue
        out.append('%s="%s"' % (k.replace("_", "-").rstrip("-"), v))
    return "<%s %s/>" % (tag, " ".join(out))


def g(body: str, transform: str | None = None, opacity=None) -> str:
    a = ""
    if transform:
        a += ' transform="%s"' % transform
    if opacity is not None:
        a += ' opacity="%s"' % opacity
    return "<g%s>%s</g>" % (a, body)


def rect(x, y, w, h, fill, stroke=None, sw=3, rx=None, opacity=None):
    return e("rect", x=r2(x), y=r2(y), width=r2(w), height=r2(h), rx=rx,
             fill=fill, stroke=stroke, stroke_width=sw if stroke else None,
             opacity=opacity)


def circ(cx, cy, r, fill, stroke=None, sw=3, opacity=None):
    return e("circle", cx=r2(cx), cy=r2(cy), r=r2(r), fill=fill, stroke=stroke,
             stroke_width=sw if stroke else None, opacity=opacity)


def ell(cx, cy, rx, ry, fill, stroke=None, sw=3, opacity=None):
    return e("ellipse", cx=r2(cx), cy=r2(cy), rx=r2(rx), ry=r2(ry), fill=fill,
             stroke=stroke, stroke_width=sw if stroke else None, opacity=opacity)


def poly(pts, fill, stroke=None, sw=3, opacity=None):
    s = " ".join("%s,%s" % (r2(p[0]), r2(p[1])) for p in pts)
    return e("polygon", points=s, fill=fill, stroke=stroke,
             stroke_width=sw if stroke else None, stroke_linejoin="round" if stroke else None,
             opacity=opacity)


def line(x1, y1, x2, y2, stroke, sw=3, opacity=None, cap="round"):
    return e("line", x1=r2(x1), y1=r2(y1), x2=r2(x2), y2=r2(y2), stroke=stroke,
             stroke_width=sw, stroke_linecap=cap, opacity=opacity)


def path(d, fill="none", stroke=None, sw=3, opacity=None):
    return e("path", d=d, fill=fill, stroke=stroke,
             stroke_width=sw if stroke else None,
             stroke_linecap="round" if stroke else None,
             stroke_linejoin="round" if stroke else None, opacity=opacity)


def r2(v):
    return ("%.2f" % float(v)).rstrip("0").rstrip(".")


def soft_shadow(cx, cy, rx, ry):
    """Three stacked ellipses stand in for a blur ThorVG may not support."""
    out = []
    for k, a in ((1.34, 0.05), (1.14, 0.07), (1.0, 0.10)):
        out.append(ell(cx, cy, rx * k, ry * k, P["shadow"], opacity=a))
    return "".join(out)


def write(name, w, h, ax, ay, body):
    doc = (
        '<svg xmlns="http://www.w3.org/2000/svg" width="%d" height="%d" '
        'viewBox="0 0 %d %d">%s</svg>' % (int(w), int(h), int(w), int(h), body)
    )
    with open(os.path.join(OUT, name + ".svg"), "w") as f:
        f.write(doc)
    props[name] = {"w": int(w), "h": int(h), "ax": round(ax, 2), "ay": round(ay, 2)}


# ---------------------------------------------------------------------------
# vegetation
# ---------------------------------------------------------------------------

def blob_cluster(blobs, base, mid, hi, top, out_col, out_w=9):
    """Draw a union-outlined clump of circles, then light it from upper-left."""
    s = []
    for (cx, cy, r) in blobs:                       # pass 1: silhouette
        s.append(circ(cx, cy, r, out_col, stroke=out_col, sw=out_w))
    for (cx, cy, r) in blobs:                       # pass 2: body
        s.append(circ(cx, cy, r, base))
    ordered = sorted(blobs, key=lambda b: b[1] - b[0] * 0.25)
    n = len(ordered)
    for i, (cx, cy, r) in enumerate(ordered):       # pass 3: light
        if i < n * 0.34:
            s.append(circ(cx - r * 0.16, cy - r * 0.20, r * 0.78, top))
            s.append(circ(cx - r * 0.24, cy - r * 0.30, r * 0.46, hi))
        elif i < n * 0.62:
            s.append(circ(cx - r * 0.14, cy - r * 0.16, r * 0.66, mid))
    return "".join(s)


def make_tree(name, height=430, spread=1.0, seed=1, kind="round"):
    rnd = random.Random(seed)
    w = int(height * 0.96 * spread)
    h = int(height)
    cx = w / 2.0
    ground = h - 14
    trunk_h = h * (0.30 if kind == "round" else 0.34)
    canopy_cy = ground - trunk_h - h * 0.24
    R = h * 0.27 * spread

    s = [soft_shadow(cx + 6, ground + 2, R * 0.72, R * 0.26)]
    # trunk
    tw = h * 0.055
    s.append(path("M %s %s C %s %s, %s %s, %s %s L %s %s C %s %s, %s %s, %s %s Z" % (
        r2(cx - tw), r2(ground), r2(cx - tw * 0.72), r2(ground - trunk_h * 0.6),
        r2(cx - tw * 0.58), r2(ground - trunk_h * 0.8), r2(cx - tw * 0.5), r2(ground - trunk_h),
        r2(cx + tw * 0.5), r2(ground - trunk_h),
        r2(cx + tw * 0.58), r2(ground - trunk_h * 0.8),
        r2(cx + tw * 0.72), r2(ground - trunk_h * 0.6), r2(cx + tw), r2(ground)),
        fill=P["trunk"], stroke=P["trunk_lo"], sw=2.8))
    s.append(path("M %s %s L %s %s" % (r2(cx + tw * 0.35), r2(ground - 6),
                                       r2(cx + tw * 0.2), r2(ground - trunk_h * 0.85)),
                  stroke=P["trunk_lo"], sw=3.3, opacity=0.5))
    # a couple of limbs into the canopy
    for dx in (-1, 1):
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(cx + dx * tw * 0.3), r2(ground - trunk_h * 0.85),
            r2(cx + dx * R * 0.34), r2(ground - trunk_h * 1.05),
            r2(cx + dx * R * 0.5), r2(canopy_cy + R * 0.42)),
            stroke=P["trunk"], sw=6.1))

    blobs = []
    if kind == "round":
        ring = 7
        for i in range(ring):
            a = -math.pi / 2 + i * (2 * math.pi / ring) + rnd.uniform(-0.16, 0.16)
            rr = R * rnd.uniform(0.50, 0.62)
            blobs.append((cx + math.cos(a) * R * 0.62,
                          canopy_cy + math.sin(a) * R * 0.46,
                          rr))
        blobs.append((cx + rnd.uniform(-8, 8), canopy_cy + rnd.uniform(-6, 6), R * 0.66))
        blobs.append((cx - R * 0.26, canopy_cy - R * 0.34, R * 0.44))
    else:  # billowy, taller
        for i in range(9):
            a = rnd.uniform(0, math.tau)
            d = rnd.uniform(0.15, 0.78)
            blobs.append((cx + math.cos(a) * R * 0.70 * d,
                          canopy_cy + math.sin(a) * R * 0.62 * d - R * 0.1,
                          R * rnd.uniform(0.40, 0.60)))
        blobs.append((cx, canopy_cy - R * 0.18, R * 0.60))

    s.append(blob_cluster(blobs, P["leaf"], P["leaf_hi"], P["leaf_top"],
                          P["leaf_top"], P["leaf_out"], out_w=10))
    # a little depth at the base of the canopy
    s.append(ell(cx + R * 0.16, canopy_cy + R * 0.40, R * 0.46, R * 0.20,
                 P["leaf_lo"], opacity=0.35))
    write(name, w, h, cx, ground, "".join(s))


def make_bush(name, size=150, seed=5, flowers=True):
    rnd = random.Random(seed)
    w = int(size * 1.35)
    h = int(size)
    cx, ground = w / 2.0, h - 10
    R = h * 0.42
    s = [soft_shadow(cx + 3, ground, R * 1.0, R * 0.26)]
    blobs = []
    for i in range(6):
        a = math.pi + i * (math.pi / 5.0)
        blobs.append((cx + math.cos(a) * R * 0.86, ground - R * 0.52 + math.sin(a) * R * 0.30,
                      R * rnd.uniform(0.46, 0.60)))
    blobs.append((cx, ground - R * 0.74, R * 0.58))
    s.append(blob_cluster(blobs, P["leaf"], P["leaf_hi"], P["leaf_top"],
                          P["leaf_top"], P["leaf_out"], out_w=8))
    if flowers:
        for i in range(9):
            fx = cx + rnd.uniform(-R * 1.15, R * 1.15)
            fy = ground - R * 0.5 + rnd.uniform(-R * 0.55, R * 0.30)
            c = FLOWERS[rnd.randrange(len(FLOWERS))]
            s.append(circ(fx, fy, R * 0.085, c))
            s.append(circ(fx - R * 0.02, fy - R * 0.02, R * 0.04, P["white"], opacity=0.7))
    write(name, w, h, cx, ground, "".join(s))


def make_hedge(name, size=120, seed=9):
    make_bush(name, size=size, seed=seed, flowers=False)


# ---------------------------------------------------------------------------
# street furniture
# ---------------------------------------------------------------------------

def make_lamppost(name="lamppost", h=470):
    w = 150
    cx, ground = w / 2.0, h - 10
    s = [soft_shadow(cx + 10, ground, 34, 12)]
    # base
    s.append(ell(cx, ground - 6, 24, 10, P["iron"]))
    s.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
        r2(cx - 20), r2(ground - 6), r2(cx - 12), r2(ground - 44),
        r2(cx + 12), r2(ground - 44), r2(cx + 20), r2(ground - 6)),
        fill=P["iron"]))
    # column
    s.append(rect(cx - 6.5, ground - h * 0.80, 13, h * 0.80 - 30, P["iron"], rx=5))
    s.append(rect(cx - 6.5, ground - h * 0.80, 4.5, h * 0.80 - 30, P["iron_hi"], rx=3, opacity=0.8))
    # collar
    s.append(rect(cx - 12, ground - h * 0.80 - 10, 24, 14, P["iron"], rx=5))
    # lantern
    ly = ground - h * 0.80 - 10
    s.append(poly([(cx - 22, ly), (cx + 22, ly), (cx + 14, ly - 56), (cx - 14, ly - 56)],
                  P["glow"], stroke=P["iron"], sw=3.9))
    s.append(poly([(cx - 22, ly), (cx + 22, ly), (cx + 14, ly - 56), (cx - 14, ly - 56)],
                  P["glow_soft"], opacity=0.55))
    s.append(line(cx, ly - 2, cx, ly - 54, P["iron"], sw=2.2, opacity=0.55))
    s.append(poly([(cx - 17, ly - 56), (cx + 17, ly - 56), (cx, ly - 82)], P["iron"]))
    s.append(circ(cx, ly - 86, 5, P["iron"]))
    # halo
    s.append(circ(cx, ly - 28, 46, P["glow"], opacity=0.10))
    s.append(circ(cx, ly - 28, 30, P["glow"], opacity=0.12))
    write(name, w, h, cx, ground, "".join(s))


def _iso_slab(x, y, wu, du, th, top, side_l, side_r, hw=52.0, hh=26.0):
    """A box lying on the ground: wu across +X, du across +Y, th tall (2x px)."""
    a = (x, y)                                   # back
    b = (x + wu * hw, y + wu * hh)               # right
    c = (x + wu * hw - du * hw, y + wu * hh + du * hh)  # front
    d = (x - du * hw, y + du * hh)               # left
    s = []
    s.append(poly([(p[0], p[1] + th) for p in (a, b, c, d)], side_l, opacity=1))
    s.append(poly([b, c, (c[0], c[1] + th), (b[0], b[1] + th)], side_r))
    s.append(poly([d, c, (c[0], c[1] + th), (d[0], d[1] + th)], side_l))
    s.append(poly([a, b, c, d], top))
    return "".join(s), (a, b, c, d)


def make_bench(name="bench", flip=False):
    """Bench seen in iso, back to the up-screen side."""
    w, h = 300, 210
    ground = h - 12
    s = [soft_shadow(w / 2 + 6, ground, 96, 22)]
    L = 250.0
    dx, dy = (L * 0.92, L * 0.36)
    x0 = (w - dx) / 2.0
    y0 = ground - dy / 2.0 - 34
    sgn = -1 if flip else 1
    if flip:
        x0 = w - x0
    # legs
    for t in (0.10, 0.88):
        lx = x0 + sgn * dx * t
        ly = y0 + dy * t
        s.append(rect(lx - 7, ly + 4, 14, 44, P["iron"], rx=4))
        s.append(rect(lx + sgn * 28 - 7, ly + 4 - 11, 14, 42, P["iron"], rx=4))
    # seat slats
    for i in range(3):
        off = i * 13
        s.append(poly([(x0 + sgn * 0, y0 + 6 + off), (x0 + sgn * dx, y0 + dy + 6 + off),
                       (x0 + sgn * dx + sgn * 26, y0 + dy - 4 + off),
                       (x0 + sgn * 26, y0 - 4 + off)],
                      P["wood"] if i % 2 == 0 else P["wood_hi"],
                      stroke=P["wood_out"], sw=1.7))
    # back
    for i in range(3):
        off = i * 15
        s.append(poly([(x0, y0 - 4 - off), (x0 + sgn * dx, y0 + dy - 4 - off),
                       (x0 + sgn * dx, y0 + dy - 15 - off), (x0, y0 - 15 - off)],
                      P["wood_hi"] if i % 2 else P["wood"], stroke=P["wood_out"], sw=1.7))
    for t in (0.06, 0.94):
        lx = x0 + sgn * dx * t
        ly = y0 + dy * t
        s.append(rect(lx - 6, ly - 52, 12, 58, P["iron"], rx=4))
    write(name, w, h, w / 2.0, ground, "".join(s))


def make_planter(name="planter", seed=3, tall=False):
    rnd = random.Random(seed)
    w, h = 210, 200 if tall else 170
    ground = h - 8
    s = [soft_shadow(w / 2 + 4, ground, 70, 18)]
    box, corners = _iso_slab(w / 2.0, ground - 96 - (14 if tall else 0), 1.0, 1.0, 58,
                             P["wood_hi"], P["wood"], P["wood_lo"])
    s.append(box)
    a, b, c, d = corners
    s.append(poly([a, b, c, d], "none", stroke=P["wood_out"], sw=2.2))
    for t in (0.3, 0.7):
        s.append(line(d[0] + (c[0] - d[0]) * t, d[1] + (c[1] - d[1]) * t + 4,
                      d[0] + (c[0] - d[0]) * t, d[1] + (c[1] - d[1]) * t + 54,
                      P["wood_out"], sw=1.7, opacity=0.4))
    # soil + planting
    cx = w / 2.0
    cy = (a[1] + c[1]) / 2.0
    s.append(ell(cx, cy, 52, 24, "#6B5233"))
    blobs = []
    for i in range(6):
        ang = i * (math.tau / 6)
        blobs.append((cx + math.cos(ang) * 34, cy - 12 + math.sin(ang) * 15,
                      rnd.uniform(18, 26)))
    s.append(blob_cluster(blobs, P["leaf"], P["leaf_hi"], P["leaf_top"],
                          P["leaf_top"], P["leaf_out"], out_w=6))
    for i in range(11):
        fx = cx + rnd.uniform(-52, 52)
        fy = cy - 16 + rnd.uniform(-18, 16)
        col = FLOWERS[rnd.randrange(len(FLOWERS))]
        s.append(circ(fx, fy, 6.5, col))
        s.append(circ(fx - 1.6, fy - 1.6, 2.8, P["white"], opacity=0.75))
    write(name, w, h, cx, ground, "".join(s))


def make_pot(name="pot", seed=11):
    rnd = random.Random(seed)
    w, h = 130, 160
    cx, ground = w / 2.0, h - 6
    s = [soft_shadow(cx + 3, ground, 40, 13)]
    s.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
        r2(cx - 33), r2(ground - 62), r2(cx + 33), r2(ground - 62),
        r2(cx + 25), r2(ground - 2), r2(cx - 25), r2(ground - 2)),
        fill=P["roof_terra"], stroke=P["roof_terra_lo"], sw=2.2))
    s.append(rect(cx - 36, ground - 72, 72, 16, P["roof_terra"], stroke=P["roof_terra_lo"], sw=2.2, rx=4))
    s.append(ell(cx, ground - 64, 30, 8, "#6B5233"))
    blobs = [(cx + rnd.uniform(-20, 20), ground - 84 + rnd.uniform(-14, 6), rnd.uniform(15, 22))
             for _ in range(5)]
    s.append(blob_cluster(blobs, P["leaf"], P["leaf_hi"], P["leaf_top"],
                          P["leaf_top"], P["leaf_out"], out_w=6))
    for i in range(5):
        fx = cx + rnd.uniform(-24, 24)
        fy = ground - 88 + rnd.uniform(-12, 10)
        s.append(circ(fx, fy, 5.5, FLOWERS[rnd.randrange(len(FLOWERS))]))
    write(name, w, h, cx, ground, "".join(s))


def make_chalkboard(name="chalkboard"):
    w, h = 170, 220
    cx, ground = w / 2.0, h - 8
    s = [soft_shadow(cx + 4, ground, 48, 14)]
    s.append(poly([(cx - 6, ground - 140), (cx + 34, ground - 130),
                   (cx + 30, ground - 4), (cx - 8, ground - 8)],
                  P["wood_lo"], stroke=P["wood_out"], sw=2.2))          # rear leg
    s.append(poly([(cx - 52, ground - 132), (cx + 8, ground - 144),
                   (cx + 12, ground - 6), (cx - 46, ground - 2)],
                  P["board_edge"], stroke=P["wood_out"], sw=2.8))       # frame
    s.append(poly([(cx - 44, ground - 126), (cx + 2, ground - 136),
                   (cx + 5, ground - 20), (cx - 39, ground - 14)],
                  P["board"]))
    for i in range(4):
        y = ground - 112 + i * 24
        s.append(line(cx - 36, y, cx - 4 - i * 2, y - 8, "#DCD6C8", sw=2.8, opacity=0.75))
    write(name, w, h, cx, ground, "".join(s))


def make_signpost(name="signpost"):
    w, h = 240, 330
    cx, ground = w / 2.0 - 20, h - 8
    s = [soft_shadow(cx + 6, ground, 26, 10)]
    s.append(rect(cx - 8, ground - 300, 16, 300, P["wood"], stroke=P["wood_out"], sw=2.2, rx=4))
    for i in range(3):
        y = ground - 286 + i * 44
        s.append(poly([(cx + 6, y), (cx + 150, y + 6), (cx + 168, y + 22),
                       (cx + 150, y + 38), (cx + 6, y + 32)],
                      P["wood_hi"], stroke=P["wood_out"], sw=2.2))
        s.append(line(cx + 22, y + 18, cx + 120, y + 22, P["wood_lo"], sw=2.2, opacity=0.35))
    s.append(circ(cx, ground - 306, 9, P["wood_lo"]))
    write(name, w, h, cx, ground, "".join(s))


def make_umbrella_set(name="cafeset"):
    """Parasol with a bistro table and two chairs, as in plates 02 and 03."""
    w, h = 420, 470
    cx, ground = w / 2.0, h - 10
    s = [soft_shadow(cx + 14, ground, 130, 34)]
    # chairs behind
    for sgn, ox in ((-1, -108), (1, 112)):
        bx = cx + ox
        s.append(rect(bx - 30, ground - 96, 60, 12, P["wood"], stroke=P["wood_out"], sw=1.7, rx=4))
        s.append(rect(bx - 26, ground - 150, 52, 56, P["wood_hi"], stroke=P["wood_out"], sw=1.7, rx=8))
        s.append(rect(bx - 18, ground - 142, 36, 40, P["wood"], opacity=0.5, rx=6))
        for lx in (bx - 24, bx + 18):
            s.append(rect(lx, ground - 86, 8, 80, P["wood_lo"], rx=3))
    # table
    s.append(rect(cx - 10, ground - 116, 20, 108, P["iron"], rx=6))
    s.append(ell(cx, ground - 8, 46, 14, P["iron"]))
    s.append(ell(cx, ground - 122, 84, 30, P["wood_hi"], stroke=P["wood_out"], sw=2.2))
    s.append(ell(cx, ground - 128, 84, 30, P["wood"], stroke=P["wood_out"], sw=2.2))
    s.append(ell(cx - 18, ground - 136, 30, 11, P["wood_hi"], opacity=0.5))
    # cup
    s.append(rect(cx + 16, ground - 152, 20, 20, P["white"], stroke=P["cream_out"], sw=1.7, rx=4))
    s.append(ell(cx + 26, ground - 152, 15, 5, P["white"], stroke=P["cream_out"], sw=1.7))
    # pole + canopy
    s.append(rect(cx - 7, ground - 400, 14, 290, P["wood"], rx=5))
    scallops = []
    N = 8
    R = 168.0
    for i in range(N + 1):
        t = i / N
        ang = math.pi * t
        px = cx - math.cos(ang) * R
        py = ground - 330 + math.sin(ang) * 22 + 34
        scallops.append((px, py))
    d = "M %s %s " % (r2(cx), r2(ground - 400))
    d += "L %s %s " % (r2(scallops[0][0]), r2(scallops[0][1]))
    for i in range(N):
        a = scallops[i]
        b = scallops[i + 1]
        mx = (a[0] + b[0]) / 2.0
        d += "Q %s %s, %s %s " % (r2(mx), r2((a[1] + b[1]) / 2.0 + 20), r2(b[0]), r2(b[1]))
    d += "Z"
    s.append(path(d, fill=P["cream"], stroke=P["cream_out"], sw=2.8))
    s.append(path("M %s %s L %s %s L %s %s Z" % (
        r2(cx), r2(ground - 400), r2(scallops[-1][0]), r2(scallops[-1][1]),
        r2(cx + 20), r2(ground - 332)), fill=P["cream_lo"], opacity=0.55))
    s.append(circ(cx, ground - 404, 8, P["cream_out"]))
    write(name, w, h, cx, ground, "".join(s))


def make_bicycle(name="bicycle"):
    w, h = 320, 200
    cx, ground = w / 2.0, h - 8
    s = [soft_shadow(cx, ground, 110, 16)]
    for wx in (cx - 88, cx + 88):
        s.append(circ(wx, ground - 54, 52, "none", stroke=P["iron"], sw=5.0))
        s.append(circ(wx, ground - 54, 52, "none", stroke=P["iron_hi"], sw=1.7, opacity=0.6))
        for k in range(6):
            a = k * math.pi / 6
            s.append(line(wx - math.cos(a) * 48, ground - 54 - math.sin(a) * 48,
                          wx + math.cos(a) * 48, ground - 54 + math.sin(a) * 48,
                          P["iron_hi"], sw=1.4, opacity=0.75))
        s.append(circ(wx, ground - 54, 7, P["iron"]))
    frame = [(cx - 88, ground - 54), (cx - 18, ground - 54), (cx + 20, ground - 120),
             (cx - 46, ground - 120), (cx - 18, ground - 54), (cx + 88, ground - 54),
             (cx + 20, ground - 120)]
    for i in range(len(frame) - 1):
        s.append(line(frame[i][0], frame[i][1], frame[i + 1][0], frame[i + 1][1],
                      P["wall_blue"], sw=5.5))
    s.append(line(cx - 46, ground - 120, cx - 84, ground - 58, P["wall_blue"], sw=5.5))
    s.append(rect(cx - 44, ground - 138, 52, 14, P["iron"], rx=6))       # saddle
    s.append(line(cx + 6, ground - 128, cx + 34, ground - 152, P["iron"], sw=4.4))
    s.append(line(cx + 14, ground - 152, cx + 54, ground - 148, P["iron"], sw=4.4))
    s.append(rect(cx + 26, ground - 150, 58, 40, P["wood_hi"], stroke=P["wood_out"], sw=2.2, rx=6))
    for i in range(4):
        s.append(circ(cx + 36 + i * 12, ground - 154 + (i % 2) * 6, 7,
                      FLOWERS[i % len(FLOWERS)]))
    write(name, w, h, cx, ground, "".join(s))


def make_barrel(name="barrel"):
    w, h = 140, 170
    cx, ground = w / 2.0, h - 6
    s = [soft_shadow(cx + 3, ground, 42, 14)]
    s.append(path("M %s %s C %s %s, %s %s, %s %s L %s %s C %s %s, %s %s, %s %s Z" % (
        r2(cx - 36), r2(ground - 112), r2(cx - 46), r2(ground - 70),
        r2(cx - 46), r2(ground - 46), r2(cx - 34), r2(ground - 6),
        r2(cx + 34), r2(ground - 6), r2(cx + 46), r2(ground - 46),
        r2(cx + 46), r2(ground - 70), r2(cx + 36), r2(ground - 112)),
        fill=P["wood"], stroke=P["wood_out"], sw=2.2))
    for y in (ground - 98, ground - 58, ground - 20):
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(cx - 45), r2(y), r2(cx), r2(y + 9), r2(cx + 45), r2(y)),
            stroke=P["iron"], sw=4.4, opacity=0.85))
    s.append(ell(cx, ground - 112, 36, 12, P["wood_hi"], stroke=P["wood_out"], sw=2.2))
    write(name, w, h, cx, ground, "".join(s))


def make_fountain(name="fountain"):
    w, h = 720, 560
    cx = w / 2.0
    ground = h - 40
    s = [soft_shadow(cx + 12, ground + 6, 290, 88)]
    # outer basin
    s.append(ell(cx, ground, 290, 116, P["stone_lo"], stroke=P["joint"], sw=3.3))
    s.append(ell(cx, ground - 26, 290, 116, P["stone_hi"], stroke=P["joint"], sw=3.3))
    s.append(ell(cx, ground - 26, 252, 96, P["water_lo"]))
    s.append(ell(cx, ground - 30, 252, 96, P["water"]))
    s.append(ell(cx - 40, ground - 44, 120, 38, P["water_hi"], opacity=0.45))
    for i in range(3):
        s.append(ell(cx, ground - 30, 90 + i * 52, 34 + i * 20, "none",
                     stroke=P["water_hi"], sw=2.2, opacity=0.4))
    # pedestal + bowls
    s.append(ell(cx, ground - 40, 76, 30, P["stone_lo"]))
    s.append(ell(cx, ground - 52, 76, 30, P["stone_hi"], stroke=P["joint"], sw=2.2))
    s.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
        r2(cx - 30), r2(ground - 56), r2(cx + 30), r2(ground - 56),
        r2(cx + 20), r2(ground - 132), r2(cx - 20), r2(ground - 132)),
        fill=P["stone"], stroke=P["joint"], sw=2.2))
    s.append(ell(cx, ground - 128, 104, 40, P["stone_hi"], stroke=P["joint"], sw=2.8))
    s.append(ell(cx, ground - 136, 92, 34, P["water"], stroke=P["water_lo"], sw=1.7))
    s.append(ell(cx - 16, ground - 140, 44, 14, P["water_hi"], opacity=0.5))
    s.append(rect(cx - 13, ground - 208, 26, 78, P["stone"], stroke=P["joint"], sw=2.2, rx=6))
    s.append(ell(cx, ground - 206, 56, 22, P["stone_hi"], stroke=P["joint"], sw=2.2))
    s.append(ell(cx, ground - 212, 46, 17, P["water"], opacity=0.9))
    # jets
    for dx in (-1, 1):
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(cx + dx * 6), r2(ground - 236), r2(cx + dx * 54), r2(ground - 214),
            r2(cx + dx * 70), r2(ground - 150)),
            stroke=P["water_hi"], sw=4.4, opacity=0.8))
    s.append(path("M %s %s L %s %s" % (r2(cx), r2(ground - 214), r2(cx), r2(ground - 300)),
                  stroke=P["white"], sw=6.1, opacity=0.75))
    s.append(path("M %s %s L %s %s" % (r2(cx), r2(ground - 240), r2(cx), r2(ground - 296)),
                  stroke=P["white"], sw=2.8, opacity=0.9))
    s.append(ell(cx, ground - 302, 16, 9, P["white"], opacity=0.6))
    # droplets
    for (dx, dy, rr) in ((-46, -256, 6), (52, -262, 5), (-70, -210, 5), (74, -206, 6)):
        s.append(circ(cx + dx, ground + dy, rr, P["water_hi"], opacity=0.8))
    # flower ring at the foot
    rnd = random.Random(21)
    for i in range(26):
        a = i * (math.tau / 26) + 0.1
        fx = cx + math.cos(a) * 300
        fy = ground + 6 + math.sin(a) * 122
        if fy < ground - 80:
            continue
        s.append(circ(fx, fy, 13, P["leaf"], stroke=P["leaf_out"], sw=1.7))
        s.append(circ(fx - 2, fy - 3, 6, FLOWERS[rnd.randrange(len(FLOWERS))]))
    write(name, w, h, cx, ground + 6, "".join(s))


def make_railing(name, direction=1, span=1.0):
    """Iron railing along +X (direction 1) or +Y (direction -1), one world unit."""
    dx = SX * span
    dy = SY * span * direction * 0.5 * 2 / 2  # = SY/2 per unit, matching iso
    dy = (SY / 2.0) * span * (1 if direction > 0 else 1)
    down = 1
    w = int(dx + 40)
    h = int(abs(dy) + 150)
    x0 = 20.0
    if direction > 0:
        y0 = 110.0
    else:
        y0 = 110.0 + abs(dy)
    s = []
    def at(t):
        return (x0 + dx * t, y0 + (dy if direction > 0 else -dy) * t)
    s.append(soft_shadow((at(0)[0] + at(1)[0]) / 2, (at(0)[1] + at(1)[1]) / 2 + 4,
                         dx * 0.5, 10))
    for t in (0.0, 0.34, 0.67, 1.0):
        px, py = at(t)
        s.append(rect(px - 5, py - 96, 10, 96, P["iron"], rx=4))
        s.append(circ(px, py - 100, 7, P["iron"]))
    s.append(line(at(0)[0], at(0)[1] - 92, at(1)[0], at(1)[1] - 92, P["iron"], sw=5.0))
    s.append(line(at(0)[0], at(0)[1] - 48, at(1)[0], at(1)[1] - 48, P["iron"], sw=3.3))
    anchor_x = (at(0)[0] + at(1)[0]) / 2.0
    anchor_y = (at(0)[1] + at(1)[1]) / 2.0
    write(name, w, h, anchor_x, anchor_y, "".join(s))


def make_boat(name="boat"):
    w, h = 460, 250
    cx, ground = w / 2.0, h - 40
    s = []
    s.append(ell(cx, ground + 10, 190, 34, P["water_lo"], opacity=0.5))
    hull = "M %s %s C %s %s, %s %s, %s %s C %s %s, %s %s, %s %s Z" % (
        r2(cx - 200), r2(ground - 66), r2(cx - 150), r2(ground + 22),
        r2(cx + 150), r2(ground + 22), r2(cx + 200), r2(ground - 66),
        r2(cx + 120), r2(ground - 40), r2(cx - 120), r2(ground - 40),
        r2(cx - 200), r2(ground - 66))
    s.append(path(hull, fill=P["white"], stroke=P["cream_out"], sw=2.8))
    s.append(path("M %s %s C %s %s, %s %s, %s %s" % (
        r2(cx - 182), r2(ground - 20), r2(cx - 130), r2(ground + 18),
        r2(cx + 130), r2(ground + 18), r2(cx + 182), r2(ground - 20)),
        stroke=P["wall_blue"], sw=7.2))
    s.append(ell(cx, ground - 52, 138, 26, P["water_lo"], opacity=0.25))
    s.append(ell(cx, ground - 54, 138, 26, P["cream"], stroke=P["cream_out"], sw=2.2))
    for ox in (-60, 60):
        s.append(poly([(cx + ox - 54, ground - 58), (cx + ox + 54, ground - 58),
                       (cx + ox + 48, ground - 48), (cx + ox - 48, ground - 48)],
                      P["wood_hi"], stroke=P["wood_out"], sw=1.7))
    s.append(rect(cx - 26, ground - 96, 62, 44, P["wood"], stroke=P["wood_out"], sw=2.2, rx=5))
    s.append(rect(cx - 18, ground - 88, 46, 12, P["wood_hi"], opacity=0.6, rx=3))
    write(name, w, h, cx, ground, "".join(s))


def make_jetty(name="jetty", nx=3, ny=2):
    """Wooden deck, nx world units along +X, ny along +Y."""
    hw, hh = SX / 2.0, SY / 2.0
    th = 26.0
    a = (nx * hw, 0.0)
    b = (nx * hw + 0.0, 0.0)
    # compute corners with origin at the back corner
    A = (ny * hw, 0.0)
    B = (ny * hw + nx * hw, nx * hh)
    C = (nx * hw, nx * hh + ny * hh)
    D = (0.0, ny * hh)
    minx = min(p[0] for p in (A, B, C, D)) - 20
    maxx = max(p[0] for p in (A, B, C, D)) + 20
    miny = min(p[1] for p in (A, B, C, D)) - 20
    maxy = max(p[1] for p in (A, B, C, D)) + 110
    w, h = maxx - minx, maxy - miny
    ox, oy = -minx, -miny
    A, B, C, D = [(p[0] + ox, p[1] + oy) for p in (A, B, C, D)]
    s = []
    # piles
    for (px, py) in (B, C, D):
        s.append(rect(px - 11, py + th - 6, 22, 86, P["wood_lo"], stroke=P["wood_out"], sw=2.2, rx=4))
    # side faces
    s.append(poly([B, C, (C[0], C[1] + th), (B[0], B[1] + th)], P["wood_lo"]))
    s.append(poly([D, C, (C[0], C[1] + th), (D[0], D[1] + th)], P["wood"]))
    # deck planks along +X
    s.append(poly([A, B, C, D], P["wood_hi"], stroke=P["wood_out"], sw=2.2))
    steps = max(2, int(ny * 5))
    for i in range(1, steps):
        t = i / steps
        p1 = (A[0] + (D[0] - A[0]) * t, A[1] + (D[1] - A[1]) * t)
        p2 = (B[0] + (C[0] - B[0]) * t, B[1] + (C[1] - B[1]) * t)
        s.append(line(p1[0], p1[1], p2[0], p2[1], P["wood_lo"], sw=1.7, opacity=0.5, cap="butt"))
    s.append(poly([A, B, C, D], "none", stroke=P["wood_out"], sw=2.2))
    # bollards on the outer edge
    for t in (0.12, 0.88):
        px = B[0] + (C[0] - B[0]) * t
        py = B[1] + (C[1] - B[1]) * t
        s.append(rect(px - 13, py - 52, 26, 60, P["wood"], stroke=P["wood_out"], sw=2.2, rx=8))
        s.append(ell(px, py - 52, 13, 6, P["wood_hi"]))
    write(name, w, h, C[0], C[1], "".join(s))


# ---------------------------------------------------------------------------
# buildings
# ---------------------------------------------------------------------------

def _vine(x0, y0, length, seed, up=True):
    """Climbing greenery. The references run it up every facade."""
    rnd = random.Random(seed)
    out = []
    pts = []
    n = 9
    for i in range(n + 1):
        t = i / n
        pts.append((x0 + math.sin(t * 5.2 + seed) * 16, y0 - length * t if up else y0 + length * t))
    d = "M %s %s" % (r2(pts[0][0]), r2(pts[0][1]))
    for q in pts[1:]:
        d += " L %s %s" % (r2(q[0]), r2(q[1]))
    out.append(path(d, stroke=P["leaf_lo"], sw=2.5))
    for i in range(int(length / 7)):
        t = rnd.random()
        cx = x0 + math.sin(t * 5.2 + seed) * 16 + rnd.uniform(-19, 19)
        cy = (y0 - length * t) if up else (y0 + length * t)
        rr = rnd.uniform(5.0, 9.5)
        out.append(circ(cx, cy, rr, P["leaf_lo"]))
        out.append(circ(cx - rr * 0.2, cy - rr * 0.25, rr * 0.72, P["leaf"]))
        out.append(circ(cx - rr * 0.32, cy - rr * 0.38, rr * 0.38, P["leaf_hi"]))
        if rnd.random() < 0.18:
            out.append(circ(cx + rnd.uniform(-8, 8), cy + rnd.uniform(-8, 8), 3.4,
                            FLOWERS[rnd.randrange(len(FLOWERS))]))
    return "".join(out)


def _basket(cx, cy, seed):
    """Hanging flower basket."""
    rnd = random.Random(seed)
    out = [path("M %s %s L %s %s L %s %s" % (
        r2(cx - 13), r2(cy - 2), r2(cx), r2(cy - 26), r2(cx + 13), r2(cy - 2)),
        stroke=P["iron"], sw=2.0)]
    out.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
        r2(cx - 17), r2(cy), r2(cx + 17), r2(cy),
        r2(cx + 12), r2(cy + 18), r2(cx - 12), r2(cy + 18)),
        fill=P["wood"]))
    out.append(rect(cx - 17, cy, 34, 5, sh(P["wood"], 0.26)))
    for i in range(14):
        rr = rnd.uniform(5, 9)
        bx = cx + rnd.uniform(-19, 19)
        by = cy + rnd.uniform(-8, 20)
        out.append(circ(bx, by, rr, P["leaf_lo"]))
        out.append(circ(bx - 1.5, by - 2, rr * 0.68, P["leaf"]))
    for i in range(8):
        out.append(circ(cx + rnd.uniform(-19, 19), cy + rnd.uniform(-6, 18), 3.6,
                        FLOWERS[rnd.randrange(len(FLOWERS))]))
    return "".join(out)


def _interior(x, y, w, h, seed, kind):
    """Suggested goods behind the glass: a dark warm room with lit shapes."""
    rnd = random.Random(seed)
    out = [rect(x, y, w, h, "#4A3524")]                       # dark room
    out.append(rect(x, y + h * 0.42, w, h * 0.58, "#5E442C"))
    # back-wall lamp pools
    for i in range(max(2, int(w / 110))):
        lx = x + w * (i + 0.5) / max(2, int(w / 110))
        out.append(circ(lx, y + h * 0.16, h * 0.30, P["glow"], opacity=0.30))
        out.append(circ(lx, y + h * 0.16, h * 0.16, P["glow_soft"], opacity=0.75))
        out.append(path("M %s %s L %s %s" % (r2(lx), r2(y), r2(lx), r2(y + h * 0.08)),
                        stroke="#6B5236", sw=2.5))
    if kind == "books":
        for r in range(3):
            sy = y + h * (0.30 + r * 0.22)
            out.append(rect(x + 8, sy + 22, w - 16, 5, "#8A6A46"))
            bx = x + 14
            while bx < x + w - 20:
                bw = rnd.uniform(6, 11)
                bh = rnd.uniform(14, 22)
                out.append(rect(bx, sy + 22 - bh, bw, bh,
                                FLOWERS[rnd.randrange(len(FLOWERS))]))
                bx += bw + 2
    elif kind == "bakery":
        for r in range(2):
            sy = y + h * (0.42 + r * 0.30)
            out.append(rect(x + 8, sy + 20, w - 16, 5, "#8A6A46"))
            bx = x + 16
            while bx < x + w - 24:
                out.append(ell(bx + 9, sy + 12, 10, 7, "#D9A866"))
                out.append(ell(bx + 9, sy + 9, 7, 4, "#EFCB93"))
                bx += 24
    elif kind == "bloom":
        bx = x + 14
        while bx < x + w - 18:
            out.append(rect(bx, y + h * 0.66, 13, 18, P["roof_terra"]))
            for k in range(4):
                out.append(circ(bx + 6 + rnd.uniform(-7, 7), y + h * 0.60 + rnd.uniform(-10, 4),
                                5.5, FLOWERS[rnd.randrange(len(FLOWERS))]))
            bx += 22
    else:  # cafe: counter, cups, and someone behind it
        out.append(rect(x + 6, y + h * 0.60, w - 12, h * 0.40, "#7A5634"))
        out.append(rect(x + 6, y + h * 0.58, w - 12, 8, "#9A7346"))
        for i in range(int((w - 30) / 26)):
            out.append(rect(x + 18 + i * 26, y + h * 0.48, 11, 12, "#F0E6D2"))
        px = x + w * 0.30
        out.append(circ(px, y + h * 0.30, 13, "#EFC49E"))
        out.append(path("M %s %s A 13 13 0 0 1 %s %s Z" % (
            r2(px - 13), r2(y + h * 0.30), r2(px + 13), r2(y + h * 0.30)), fill="#4A3222"))
        out.append(rect(px - 14, y + h * 0.40, 28, h * 0.24, "#5C7EA8"))
    # glass sheen over the lot
    out.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
        r2(x), r2(y + h), r2(x + w * 0.34), r2(y),
        r2(x + w * 0.60), r2(y), r2(x + w * 0.26), r2(y + h)),
        fill="#FFFFFF", opacity=0.13))
    return "".join(out)


def vgrad(x, y, w, h, top_col, bot_col, n=10):
    """A vertical ramp as stacked bands. ThorVG-safe stand-in for a gradient."""
    out = []
    for i in range(n):
        t = i / float(n - 1)
        r0, g0, b0 = _hex(top_col)
        r1, g1, b1 = _hex(bot_col)
        c = _rgb((r0 + (r1 - r0) * t, g0 + (g1 - g0) * t, b0 + (b1 - b0) * t))
        out.append(rect(x, y + h * i / n, w, h / n + 1, c))
    return "".join(out)


def make_building(name, fx, fy, storeys, wall, wall_lo, roof, roof_lo,
                  awning_a, awning_b, sign_w=430, seed=2, shop=True, kind="cafe"):
    """A shop in 2:1 iso: front face along +X, return face along +Y.

    No outlines on the masonry. Separation is by value: the front face is
    sunlit and ramps into shade at its base, the return face is in shadow, and
    every junction where two surfaces meet gets an occlusion wash. The ground
    floor is laid out as a real shopfront elevation -- fascia, awning, glazing,
    plinth -- so nothing overlaps anything it should not.
    """
    rnd = random.Random(seed)
    hw, hh = SX / 2.0, SY / 2.0
    ground_h = 250.0
    upper_h = 200.0
    wall_h = ground_h + (storeys - 1) * upper_h
    eave = 26.0
    rise = 66.0
    gy = wall_h - ground_h

    A = (fy * hw, 0.0)
    B = (fy * hw + fx * hw, fx * hh)
    C = (fx * hw, fx * hh + fy * hh)
    D = (0.0, fy * hh)

    pad = 110.0
    minx = min(p[0] for p in (A, B, C, D)) - pad
    maxx = max(p[0] for p in (A, B, C, D)) + pad
    miny = -wall_h - rise - 130
    maxy = max(p[1] for p in (A, B, C, D)) + 70
    w, h = maxx - minx, maxy - miny
    ox, oy = -minx, -miny
    A, B, C, D = [(p[0] + ox, p[1] + oy) for p in (A, B, C, D)]

    s = []
    s.append(soft_shadow((A[0] + C[0]) / 2 + 26, (A[1] + C[1]) / 2 + 32,
                         (maxx - minx) * 0.33, (maxy - miny) * 0.085))

    Atop = (A[0], A[1] - wall_h)
    Btop = (B[0], B[1] - wall_h)
    Ctop = (C[0], C[1] - wall_h)
    Dtop = (D[0], D[1] - wall_h)

    # ---- return face: in shade, and darker still toward the ground --------
    side_len = fy * hw
    side = []
    side.append(vgrad(0, 0, side_len, wall_h, sh(wall, 0.40), sh(wall, 0.56)))
    side.append(rect(0, 0, side_len, 34, sh(wall, 0.62)))
    for st in range(storeys):
        wy = (wall_h - ground_h - st * upper_h + 56) if st else (gy + 70)
        if wy < 10 or wy + 92 > wall_h - 10:
            continue
        for k in range(max(1, int(fy))):
            wx = 60 + k * (side_len / max(1, int(fy)))
            side.append(rect(wx, wy, 72, 92, sh(P["glass_cool"], 0.42), rx=3))
            side.append(rect(wx + 4, wy + 4, 22, 84, "#FFFFFF", opacity=0.10))
    s.append(g("".join(side), "matrix(1,-0.5,0,1,%s,%s)" % (r2(C[0]), r2(C[1] - wall_h))))

    # ---- sunlit front face ------------------------------------------------
    front_len = fx * hw
    fw = []
    fw.append(vgrad(0, 0, front_len, wall_h, lt(wall, 0.26), sh(wall, 0.16)))
    fw.append(rect(0, 0, front_len, 30, sh(wall, 0.26)))            # eave shade
    fw.append(rect(0, 30, front_len, 18, sh(wall, 0.12)))
    for i in range(int(front_len / 30)):                            # plaster mottle
        fw.append(rect(rnd.uniform(0, front_len - 70), rnd.uniform(40, wall_h - 50),
                       rnd.uniform(28, 68), rnd.uniform(13, 28),
                       sh(wall, 0.055) if i % 2 else lt(wall, 0.07),
                       rx=9, opacity=0.28))

    # upper storeys
    for st in range(storeys - 1):
        wy = wall_h - ground_h - (st + 1) * upper_h + 42
        cols = max(2, int(fx))
        for k in range(cols):
            wx = front_len * (k + 0.5) / cols - 54
            fw.append(rect(wx - 9, wy - 11, 126, 150, sh(wall, 0.17), rx=5))
            fw.append(rect(wx - 9, wy - 11, 126, 9, sh(wall, 0.30), rx=4))
            fw.append(rect(wx, wy, 108, 124, "#34291E", rx=2))
            fw.append(vgrad(wx + 3, wy + 3, 102, 118, "#6E6455", "#4A3E30"))
            fw.append(rect(wx + 8, wy + 40, 92, 82, "#8A6A42", opacity=0.55))
            fw.append(circ(wx + 54, wy + 58, 26, P["glow"], opacity=0.24))
            fw.append(rect(wx + 3, wy + 3, 34, 118, "#DCE8EE", opacity=0.42))
            fw.append(rect(wx + 3, wy + 3, 102, 20, "#DCE8EE", opacity=0.22))
            fw.append(rect(wx + 52, wy + 2, 5, 122, lt(wall, 0.40)))
            fw.append(rect(wx + 2, wy + 60, 104, 5, lt(wall, 0.40)))
            # window box, with occlusion under it
            for i in range(20):
                fx2 = wx - 12 + i * 7.0 + rnd.uniform(-2, 2)
                fy2 = wy + 124 + rnd.uniform(-4, 2)
                fw.append(circ(fx2, fy2, rnd.uniform(4.5, 7.0), P["leaf_lo"]))
            for i in range(16):
                fx2 = wx - 10 + i * 8.6 + rnd.uniform(-2, 2)
                fw.append(circ(fx2, wy + 121 + rnd.uniform(-3, 2), rnd.uniform(3.4, 5.4),
                               P["leaf"]))
            for i in range(9):
                fw.append(circ(wx - 8 + i * 15.0 + rnd.uniform(-3, 3),
                               wy + 120 + rnd.uniform(-4, 3), 3.2,
                               FLOWERS[rnd.randrange(len(FLOWERS))]))
            fw.append(rect(wx - 15, wy + 128, 138, 26, P["wood"], rx=4))
            fw.append(rect(wx - 15, wy + 128, 138, 6, lt(P["wood"], 0.26), rx=4))
            fw.append(rect(wx - 15, wy + 148, 138, 6, sh(P["wood"], 0.30), rx=3))
            fw.append(rect(wx - 15, wy + 154, 138, 12, "#2A2018", opacity=0.26))

    if shop:
        # --- shopfront elevation, laid out top to bottom, no collisions ----
        fascia_y = gy + 6
        fascia_h = 62
        awn_y = gy + 74
        awn_h = 54
        glass_y = gy + 100
        glass_h = 118
        plinth_y = glass_y + glass_h
        margin = 26.0
        door_w = min(112.0, front_len * 0.24)
        gap = 16.0
        usable = front_len - 2 * margin
        rest = usable - door_w - 2 * gap
        w1 = rest * 0.55
        w2 = rest * 0.45
        x1 = margin
        xd = x1 + w1 + gap
        x2 = xd + door_w + gap

        # plinth
        fw.append(rect(0, plinth_y, front_len, wall_h - plinth_y, sh(wall, 0.22)))
        fw.append(rect(0, plinth_y, front_len, 7, sh(wall, 0.34)))
        fw.append(rect(0, wall_h - 22, front_len, 22, "#2A2018", opacity=0.22))

        def window(x, wd, sd):
            o = []
            o.append(rect(x - 8, glass_y - 8, wd + 16, glass_h + 16, sh(P["wood"], 0.30), rx=4))
            o.append(rect(x - 8, glass_y - 8, wd + 16, 6, sh(P["wood"], 0.46), rx=3))
            o.append(_interior(x, glass_y, wd, glass_h, sd, kind))
            n_m = max(1, int(wd / 78))
            for i in range(1, n_m + 1):
                o.append(rect(x + wd * i / (n_m + 1) - 3, glass_y, 6, glass_h,
                              sh(P["wood"], 0.30)))
            # the awning throws deep shade across the top of the glass
            o.append(rect(x, glass_y, wd, 34, "#241B12", opacity=0.42))
            o.append(rect(x, glass_y + 34, wd, 16, "#241B12", opacity=0.18))
            return "".join(o)

        fw.append(window(x1, w1, seed))
        fw.append(window(x2, w2, seed + 3))

        # door, recessed and darker
        fw.append(rect(xd - 7, glass_y - 8, door_w + 14, glass_h + 8 + (plinth_y - glass_y - glass_h) + 18,
                       sh(P["wood"], 0.34), rx=4))
        fw.append(rect(xd, glass_y - 2, door_w, glass_h + 22, sh(P["wood"], 0.16), rx=3))
        fw.append(rect(xd + 10, glass_y + 8, door_w - 20, 62, "#3E2E1E", rx=2))
        fw.append(circ(xd + door_w * 0.5, glass_y + 40, 30, P["glow"], opacity=0.30))
        fw.append(circ(xd + door_w * 0.5, glass_y + 40, 16, P["glow_soft"], opacity=0.45))
        fw.append(rect(xd + 10, glass_y + 80, door_w - 20, glass_h - 74, sh(P["wood"], 0.04), rx=2))
        fw.append(rect(xd + 10, glass_y + 80, 12, glass_h - 74, lt(P["wood"], 0.16), rx=2))
        fw.append(circ(xd + door_w - 20, glass_y + 62, 6, lt(P["sign"], 0.3)))
        fw.append(rect(xd - 7, glass_y - 14, door_w + 14, 12, "#241B12", opacity=0.34))

        # --- awning: lit top, shaded underside, scalloped valance ----------
        aw = front_len - 16
        ax = 8
        n = max(6, int(aw / 58))
        for i in range(n):
            col = awning_a if i % 2 == 0 else awning_b
            fw.append(rect(ax + aw * i / n, awn_y, aw / n + 1, awn_h, col))
        fw.append(rect(ax, awn_y, aw, 16, "#FFFFFF", opacity=0.32))        # sunlit crest
        fw.append(rect(ax, awn_y + awn_h - 18, aw, 18, "#241B12", opacity=0.16))
        segs = max(5, int(aw / 64))
        val = "M %s %s " % (r2(ax), r2(awn_y + awn_h))
        for i in range(segs):
            xa = ax + aw * i / segs
            xb = ax + aw * (i + 1) / segs
            val += "Q %s %s, %s %s " % (r2((xa + xb) / 2), r2(awn_y + awn_h + 34),
                                        r2(xb), r2(awn_y + awn_h))
        val += "L %s %s L %s %s Z" % (r2(ax + aw), r2(awn_y + awn_h - 8), r2(ax), r2(awn_y + awn_h - 8))
        fw.append(path(val, fill=awning_b))
        fw.append(path(val, fill="#241B12", opacity=0.13))
        fw.append(rect(ax, awn_y - 7, aw, 13, lt(awning_a, 0.3), rx=3))

        # --- fascia and sign ------------------------------------------------
        fw.append(rect(0, fascia_y, front_len, fascia_h, sh(P["sign"], 0.30)))
        fw.append(rect(0, fascia_y, front_len, 8, sh(P["sign"], 0.48)))
        sgw = min(sign_w, front_len - 70)
        sx = (front_len - sgw) / 2.0
        fw.append(rect(sx, fascia_y + 6, sgw, fascia_h - 12, P["sign"], rx=4))
        fw.append(rect(sx, fascia_y + 6, sgw, 16, lt(P["sign"], 0.20), rx=4))
        fw.append(rect(sx, fascia_y + fascia_h - 18, sgw, 12, sh(P["sign"], 0.14)))
        fw.append(rect(0, fascia_y + fascia_h, front_len, 10, "#241B12", opacity=0.22))
        for lx in (sx - 40, sx + sgw + 40):
            if 24 < lx < front_len - 24:
                fw.append(rect(lx - 4, fascia_y - 34, 8, 32, P["iron"], rx=3))
                fw.append(path("M %s %s L %s %s L %s %s L %s %s Z" % (
                    r2(lx - 22), r2(fascia_y - 2), r2(lx + 22), r2(fascia_y - 2),
                    r2(lx + 12), r2(fascia_y - 30), r2(lx - 12), r2(fascia_y - 30)),
                    fill=P["iron"]))
                fw.append(ell(lx, fascia_y - 1, 20, 7, P["glow"]))
                fw.append(circ(lx, fascia_y + 16, 46, P["glow"], opacity=0.14))
                fw.append(circ(lx, fascia_y + 8, 26, P["glow"], opacity=0.16))

        # --- facade greenery, kept inside the wall so it actually reads ----
        fw.append(_vine(front_len - 13, plinth_y - 6, wall_h * 0.80, seed + 11))
        fw.append(_vine(11, plinth_y - 6, wall_h * 0.58, seed + 21))
    s.append(g("".join(fw), "matrix(1,0.5,0,1,%s,%s)" % (r2(D[0]), r2(D[1] - wall_h))))

    # ---- hip roof ---------------------------------------------------------
    inset = 0.34
    def wpt(pt):
        return (ox + (pt[0] - pt[1]) * hw + fy * hw,
                oy + (pt[0] + pt[1]) * hh - wall_h - rise)
    R1 = wpt((0.0 + inset, fy / 2.0))
    R2 = wpt((fx - inset, fy / 2.0))
    def outp(pt, dx, dy):
        return (pt[0] + dx, pt[1] + dy)
    Deav = outp(Dtop, -eave, eave * 0.5)
    Ceav = outp(Ctop, 0, eave)
    Beav = outp(Btop, eave, eave * 0.5)
    Aeav = outp(Atop, 0, -eave * 0.5)
    s.append(poly([Deav, Ceav, R2, R1], lt(roof, 0.12)))
    s.append(poly([Ceav, Beav, R2], sh(roof, 0.30)))
    s.append(poly([Aeav, Deav, R1], sh(roof, 0.40)))
    for i in range(1, 7):
        t = i / 7.0
        p1 = (Deav[0] + (R1[0] - Deav[0]) * t, Deav[1] + (R1[1] - Deav[1]) * t)
        p2 = (Ceav[0] + (R2[0] - Ceav[0]) * t, Ceav[1] + (R2[1] - Ceav[1]) * t)
        s.append(line(p1[0], p1[1], p2[0], p2[1], sh(roof, 0.14), sw=3, opacity=0.35, cap="butt"))
    s.append(line(R1[0], R1[1] - 2, R2[0], R2[1] - 2, lt(roof, 0.34), sw=5))
    s.append(poly([Deav, Ceav, Ctop, Dtop], "#241B12", opacity=0.26))   # eave shadow
    chx = R1[0] + (R2[0] - R1[0]) * 0.74
    chy = R1[1] + (R2[1] - R1[1]) * 0.74
    s.append(rect(chx - 21, chy - 88, 42, 92, P["wall_sand"], rx=2))
    s.append(rect(chx - 21, chy - 88, 14, 92, lt(P["wall_sand"], 0.20)))
    s.append(rect(chx - 27, chy - 98, 54, 17, sh(P["wall_sand"], 0.24), rx=2))

    write(name, w, h, C[0], C[1], "".join(s))
    if shop:
        u = front_len / 2.0
        v = gy + 6 + 62 / 2.0
        props[name]["sign"] = [round(D[0] + u, 2), round(D[1] - wall_h + 0.5 * u + v, 2)]
        props[name]["sign_w"] = min(sign_w, front_len - 70)


def make_house(name, fx, fy, wall, wall_lo, roof, roof_lo, seed=7):
    make_building(name, fx, fy, 1, wall, wall_lo, roof, roof_lo,
                  P["cream"], P["cream"], seed=seed, shop=False)


# ---------------------------------------------------------------------------
# characters
# ---------------------------------------------------------------------------

def _char_body(H, skin, hair, top_c, bottom_c, shoe_c, back, hat=None, bag=None,
               hair_style="short"):
    """H is total pixel height at 2x. Returns svg body drawn with feet at y=H."""
    s = []
    head_r = H * 0.148
    head_cy = head_r + H * 0.055
    neck_y = head_cy + head_r * 0.86
    torso_h = H * 0.30
    torso_w = H * 0.20
    leg_h = H - (neck_y + torso_h) - H * 0.035
    out = "#4A3A30"

    # legs
    lw = torso_w * 0.36
    for dx in (-1, 1):
        s.append(rect(H * 0.5 + dx * torso_w * 0.26 - lw / 2, neck_y + torso_h - 4,
                      lw, leg_h + 6, bottom_c, stroke=out, sw=6.1, rx=lw * 0.42))
    for dx in (-1, 1):
        s.append(ell(H * 0.5 + dx * torso_w * 0.26, H - H * 0.018,
                     lw * 0.72, H * 0.026, shoe_c, stroke=out, sw=6.1))
    # torso
    s.append(rect(H * 0.5 - torso_w / 2, neck_y - 2, torso_w, torso_h + 6,
                  top_c, stroke=out, sw=6.9, rx=torso_w * 0.30))
    if not back:
        s.append(rect(H * 0.5 - torso_w * 0.10, neck_y + 2, torso_w * 0.20, torso_h * 0.62,
                      "#FFFFFF", opacity=0.22, rx=6))
    # arms
    aw = torso_w * 0.27
    for dx in (-1, 1):
        s.append(rect(H * 0.5 + dx * (torso_w / 2 + aw * 0.18) - aw / 2, neck_y + 4,
                      aw, torso_h * 0.86, top_c, stroke=out, sw=6.1, rx=aw * 0.5))
        s.append(circ(H * 0.5 + dx * (torso_w / 2 + aw * 0.18), neck_y + 4 + torso_h * 0.86,
                      aw * 0.44, skin, stroke=out, sw=5.4))
    if bag:
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(H * 0.5 - torso_w * 0.34), r2(neck_y + 4),
            r2(H * 0.5), r2(neck_y + torso_h * 0.36),
            r2(H * 0.5 + torso_w * 0.42), r2(neck_y + torso_h * 0.30)),
            stroke=bag, sw=12.2))
        s.append(rect(H * 0.5 + torso_w * 0.30, neck_y + torso_h * 0.30, torso_w * 0.34,
                      torso_h * 0.42, bag, stroke=out, sw=6.1, rx=5))
    # head
    s.append(circ(H * 0.5, head_cy, head_r, skin, stroke=out, sw=6.9))
    if not back:
        ey = head_cy + head_r * 0.12
        s.append(ell(H * 0.5 - head_r * 0.34, ey, head_r * 0.10, head_r * 0.15, "#3A2F28"))
        s.append(ell(H * 0.5 + head_r * 0.34, ey, head_r * 0.10, head_r * 0.15, "#3A2F28"))
        s.append(ell(H * 0.5 - head_r * 0.56, ey + head_r * 0.24, head_r * 0.15,
                     head_r * 0.10, "#F0A08E", opacity=0.55))
        s.append(ell(H * 0.5 + head_r * 0.56, ey + head_r * 0.24, head_r * 0.15,
                     head_r * 0.10, "#F0A08E", opacity=0.55))
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(H * 0.5 - head_r * 0.16), r2(ey + head_r * 0.42),
            r2(H * 0.5), r2(ey + head_r * 0.58), r2(H * 0.5 + head_r * 0.16),
            r2(ey + head_r * 0.42)), stroke="#8A5A4A", sw=5.9))
    # hair
    if hair_style == "bun":
        s.append(circ(H * 0.5 + head_r * (0.0 if back else 0.02), head_cy - head_r * 1.02,
                      head_r * 0.42, hair, stroke=out, sw=6.1))
    if back:
        s.append(circ(H * 0.5, head_cy, head_r, hair, stroke=out, sw=6.9))
        if hair_style == "long":
            s.append(rect(H * 0.5 - head_r * 0.86, head_cy, head_r * 1.72, head_r * 1.5,
                          hair, stroke=out, sw=6.1, rx=head_r * 0.5))
    else:
        if hair_style == "long":
            s.append(path("M %s %s A %s %s 0 0 1 %s %s L %s %s Q %s %s, %s %s L %s %s Z" % (
                r2(H * 0.5 - head_r), r2(head_cy + head_r * 0.10),
                r2(head_r), r2(head_r),
                r2(H * 0.5 + head_r), r2(head_cy + head_r * 0.10),
                r2(H * 0.5 + head_r * 0.98), r2(head_cy + head_r * 1.30),
                r2(H * 0.5 + head_r * 0.60), r2(head_cy + head_r * 0.60),
                r2(H * 0.5 + head_r * 0.62), r2(head_cy - head_r * 0.10),
                r2(H * 0.5 - head_r * 0.62), r2(head_cy - head_r * 0.10)),
                fill=hair, stroke=out, sw=6.1))
            s.append(path("M %s %s Q %s %s, %s %s L %s %s Z" % (
                r2(H * 0.5 - head_r * 0.98), r2(head_cy + head_r * 0.10),
                r2(H * 0.5 - head_r * 0.70), r2(head_cy + head_r * 0.80),
                r2(H * 0.5 - head_r * 0.92), r2(head_cy + head_r * 1.30),
                r2(H * 0.5 - head_r * 0.60), r2(head_cy + head_r * 0.20)),
                fill=hair, stroke=out, sw=6.1))
        else:
            s.append(path("M %s %s A %s %s 0 0 1 %s %s Q %s %s, %s %s Q %s %s, %s %s Z" % (
                r2(H * 0.5 - head_r * 1.02), r2(head_cy + head_r * 0.04),
                r2(head_r * 1.02), r2(head_r * 1.02),
                r2(H * 0.5 + head_r * 1.02), r2(head_cy + head_r * 0.04),
                r2(H * 0.5 + head_r * 0.7), r2(head_cy - head_r * 0.22),
                r2(H * 0.5 + head_r * 0.10), r2(head_cy - head_r * 0.30),
                r2(H * 0.5 - head_r * 0.50), r2(head_cy - head_r * 0.10),
                r2(H * 0.5 - head_r * 1.02), r2(head_cy + head_r * 0.04)),
                fill=hair, stroke=out, sw=6.1))
        s.append(path("M %s %s Q %s %s, %s %s" % (
            r2(H * 0.5 - head_r * 0.72), r2(head_cy - head_r * 0.52),
            r2(H * 0.5 - head_r * 0.30), r2(head_cy - head_r * 0.86),
            r2(H * 0.5 + head_r * 0.20), r2(head_cy - head_r * 0.74)),
            stroke="#FFFFFF", sw=7.4, opacity=0.22))
    if hat:
        s.append(ell(H * 0.5, head_cy - head_r * 0.52, head_r * 1.62, head_r * 0.42,
                     hat, stroke=out, sw=6.1))
        s.append(path("M %s %s A %s %s 0 0 1 %s %s Z" % (
            r2(H * 0.5 - head_r * 0.80), r2(head_cy - head_r * 0.56),
            r2(head_r * 0.80), r2(head_r * 0.80),
            r2(H * 0.5 + head_r * 0.80), r2(head_cy - head_r * 0.56)),
            fill=hat, stroke=out, sw=6.1))
        s.append(rect(H * 0.5 - head_r * 0.80, head_cy - head_r * 0.74, head_r * 1.60,
                      head_r * 0.20, "#C98B6A", rx=3))
    return "".join(s)


def make_character(name, H=210, skin="#F0C39C", hair="#4A3222", top_c="#5C7EA8",
                   bottom_c="#3E4A5C", shoe_c="#4A3A30", hat=None, bag=None,
                   hair_style="short", ring=None):
    w = int(H * 1.0)
    h = int(H + H * 0.10)
    for back in (False, True):
        s = [soft_shadow(w / 2 + H * 0.03, H + 2, H * 0.155, H * 0.052)]
        if ring:
            s.append(ell(w / 2, H + 2, H * 0.225, H * 0.078, ring, opacity=0.30))
            s.append(ell(w / 2, H + 2, H * 0.225, H * 0.078, "none", stroke=ring,
                         sw=9.8, opacity=0.55))
            s.append(ell(w / 2, H + 2, H * 0.165, H * 0.056, ring, opacity=0.22))
        s.append(g(_char_body(H, skin, hair, top_c, bottom_c, shoe_c, back,
                              hat=hat, bag=bag, hair_style=hair_style),
                   "translate(%s,0)" % r2(w / 2 - H * 0.5)))
        write(name + ("_back" if back else "_front"), w, h, w / 2.0, H + 2, "".join(s))


def make_seated(name, H=210, skin="#F0C39C", hair="#4A3222", top_c="#8C7FA8",
                bottom_c="#3E4A5C", hair_style="short"):
    """Someone sitting: the Sit affordance the references show."""
    w = int(H * 0.95)
    h = int(H * 0.86)
    ground = h - 6
    out = "#4A3A30"
    head_r = H * 0.148
    cx = w / 2.0
    seat_y = ground - H * 0.20
    s = [soft_shadow(cx + 5, ground, H * 0.17, H * 0.05)]
    # lower legs
    for dx in (-1, 1):
        s.append(rect(cx + dx * H * 0.05 - H * 0.035, seat_y, H * 0.07, H * 0.20,
                      bottom_c, stroke=out, sw=6.1, rx=H * 0.03))
        s.append(ell(cx + dx * H * 0.05, ground - H * 0.012, H * 0.05, H * 0.026,
                     "#4A3A30", stroke=out, sw=6.1))
    # thighs
    s.append(rect(cx - H * 0.10, seat_y - H * 0.075, H * 0.24, H * 0.085,
                  bottom_c, stroke=out, sw=6.1, rx=H * 0.035))
    torso_h = H * 0.26
    torso_w = H * 0.20
    ty = seat_y - H * 0.075 - torso_h
    s.append(rect(cx - torso_w / 2, ty, torso_w, torso_h + H * 0.02,
                  top_c, stroke=out, sw=6.9, rx=torso_w * 0.30))
    aw = torso_w * 0.26
    for dx in (-1, 1):
        s.append(rect(cx + dx * (torso_w / 2 + aw * 0.14) - aw / 2, ty + 6, aw,
                      torso_h * 0.72, top_c, stroke=out, sw=6.1, rx=aw * 0.5))
        s.append(circ(cx + dx * (torso_w / 2 + aw * 0.14), ty + 6 + torso_h * 0.72,
                      aw * 0.44, skin, stroke=out, sw=5.4))
    head_cy = ty - head_r * 0.80
    s.append(circ(cx, head_cy, head_r, skin, stroke=out, sw=6.9))
    ey = head_cy + head_r * 0.12
    s.append(ell(cx - head_r * 0.34, ey, head_r * 0.10, head_r * 0.15, "#3A2F28"))
    s.append(ell(cx + head_r * 0.34, ey, head_r * 0.10, head_r * 0.15, "#3A2F28"))
    s.append(path("M %s %s A %s %s 0 0 1 %s %s Q %s %s, %s %s Z" % (
        r2(cx - head_r * 1.02), r2(head_cy + head_r * 0.04),
        r2(head_r * 1.02), r2(head_r * 1.02),
        r2(cx + head_r * 1.02), r2(head_cy + head_r * 0.04),
        r2(cx), r2(head_cy - head_r * 0.34),
        r2(cx - head_r * 1.02), r2(head_cy + head_r * 0.04)),
        fill=hair, stroke=out, sw=6.1))
    if hair_style == "bun":
        s.append(circ(cx, head_cy - head_r * 1.04, head_r * 0.40, hair, stroke=out, sw=6.1))
    # coffee cup in hand
    s.append(rect(cx + torso_w * 0.46, ty + torso_h * 0.60, H * 0.045, H * 0.045,
                  P["white"], stroke=P["cream_out"], sw=6.1, rx=3))
    write(name, w, h, cx, ground, "".join(s))


def make_dog(name="dog"):
    H = 96.0
    w, h = 190, 130
    ground = h - 8
    out = "#4A3A30"
    body = "#E8C79A"
    patch = "#C98A52"
    s = [soft_shadow(w / 2, ground, 56, 12)]
    for lx in (58, 76, 118, 136):
        s.append(rect(lx - 7, ground - 34, 14, 34, body, stroke=out, sw=6.1, rx=6))
    s.append(rect(50, ground - 74, 100, 46, body, stroke=out, sw=7.4, rx=22))
    s.append(ell(112, ground - 70, 30, 20, patch))
    s.append(path("M %s %s Q %s %s, %s %s" % (148, ground - 70, 178, ground - 96,
                                              166, ground - 52),
             stroke=body, sw=27.0))
    s.append(circ(52, ground - 86, 25, body, stroke=out, sw=7.4))
    s.append(ell(38, ground - 80, 15, 11, "#F4E3CB", stroke=out, sw=6.1))
    s.append(circ(28, ground - 80, 5, "#3A2F28"))
    s.append(circ(48, ground - 92, 4, "#3A2F28"))
    s.append(path("M %s %s Q %s %s, %s %s Z" % (62, ground - 100, 78, ground - 116,
                                                74, ground - 88),
             fill=patch, stroke=out, sw=6.1))
    write(name, w, h, w / 2.0, ground, "".join(s))


def make_bird(name="bird"):
    w, h = 100, 80
    ground = h - 6
    out = "#7A6A58"
    s = [soft_shadow(w / 2, ground, 24, 7)]
    s.append(ell(52, ground - 26, 30, 20, P["white"], stroke=out, sw=6.1))
    s.append(circ(28, ground - 40, 14, P["white"], stroke=out, sw=6.1))
    s.append(path("M %s %s L %s %s L %s %s Z" % (16, ground - 40, 4, ground - 36,
                                                 16, ground - 33),
             fill="#E8A23C", stroke=out, sw=4.9))
    s.append(circ(24, ground - 43, 3.2, "#3A2F28"))
    s.append(ell(60, ground - 30, 18, 11, "#D8D2C6", stroke=out, sw=4.9))
    for lx in (46, 58):
        s.append(line(lx, ground - 10, lx, ground - 2, "#E8A23C", sw=7.4))
    write(name, w, h, w / 2.0, ground, "".join(s))


# ---------------------------------------------------------------------------

def main():
    os.makedirs(OUT, exist_ok=True)
    for f in os.listdir(OUT):
        if f.endswith(".svg"):
            os.remove(os.path.join(OUT, f))

    make_tree("tree_a", height=520, spread=1.06, seed=11, kind="round")
    make_tree("tree_b", height=470, spread=0.96, seed=23, kind="billow")
    make_tree("tree_c", height=400, spread=0.88, seed=37, kind="round")
    make_bush("bush_a", size=160, seed=5)
    make_bush("bush_b", size=130, seed=8)
    make_hedge("hedge", size=118, seed=15)

    make_lamppost()
    make_bench("bench")
    make_bench("bench_r", flip=True)
    make_planter("planter", seed=3)
    make_planter("planter_b", seed=6, tall=True)
    make_pot("pot", seed=11)
    make_pot("pot_b", seed=19)
    make_chalkboard()
    make_signpost()
    make_umbrella_set()
    make_bicycle()
    make_barrel()
    make_fountain()
    make_railing("rail_x", direction=1, span=1.0)
    make_railing("rail_y", direction=-1, span=1.0)
    make_boat()
    make_jetty("jetty", nx=3, ny=2)

    make_building("shop_cafe", fx=4.0, fy=2.6, storeys=2,
                  wall=P["wall_sand"], wall_lo=P["wall_sand_lo"],
                  roof=P["roof_terra"], roof_lo=P["roof_terra_lo"],
                  awning_a=P["cream"], awning_b=P["rose"], sign_w=470, seed=2, kind="cafe")
    make_building("shop_bakery", fx=3.4, fy=2.6, storeys=2,
                  wall=P["wall_blue"], wall_lo=P["wall_blue_lo"],
                  roof=P["roof_slate"], roof_lo=P["roof_slate_lo"],
                  awning_a=P["cream"], awning_b=P["cream_lo"], sign_w=400, seed=5, kind="bakery")
    make_building("shop_books", fx=3.4, fy=2.6, storeys=2,
                  wall=P["wall_green"], wall_lo=P["wall_green_lo"],
                  roof=P["roof_terra"], roof_lo=P["roof_terra_lo"],
                  awning_a=P["cream"], awning_b="#7FB6A2", sign_w=400, seed=8, kind="books")
    make_building("shop_bloom", fx=3.0, fy=2.4, storeys=2,
                  wall="#6E8FBE", wall_lo="#53719B",
                  roof=P["roof_slate"], roof_lo=P["roof_slate_lo"],
                  awning_a=P["cream"], awning_b="#8FB4DC", sign_w=360, seed=12, kind="bloom")

    make_character("player", H=228, skin="#F3C9A2", hair="#6B4225",
                   top_c="#E8A23C", bottom_c="#37697A", shoe_c="#4A3A30",
                   bag=None, hair_style="short", ring="#FFC963")
    make_character("npc_a", H=206, skin="#EEC49F", hair="#3A2A20",
                   top_c="#5C7EA8", bottom_c="#C9BFA6", hair_style="short",
                   bag="#6B5540")
    make_character("npc_b", H=200, skin="#F2CBA6", hair="#5A3A24",
                   top_c="#EFE6D4", bottom_c="#7C8A58", hair_style="bun",
                   bag="#9A8464")
    make_character("npc_c", H=204, skin="#D8A277", hair="#2F2320",
                   top_c="#7E9E7A", bottom_c="#46546A", hair_style="short")
    make_character("npc_d", H=198, skin="#F0C7A4", hair="#4A3222",
                   top_c="#D7E0E8", bottom_c="#6E7A8C", hair_style="long",
                   hat="#E4C98C")
    make_character("npc_e", H=192, skin="#E8BC94", hair="#8A8A88",
                   top_c="#B0843F", bottom_c="#4B5560", hair_style="short")

    make_seated("seated_a", H=206, skin="#EEC49F", hair="#3A2A20", top_c="#5C7EA8",
                bottom_c="#3E4A5C")
    make_seated("seated_b", H=200, skin="#F2CBA6", hair="#6B4225", top_c="#D98C7E",
                bottom_c="#5A6472", hair_style="bun")
    make_dog()
    make_bird()

    with open(MANIFEST, "w") as f:
        json.dump(props, f, indent=1, sort_keys=True)
    print("wrote %d svg props -> %s" % (len(props), OUT))


if __name__ == "__main__":
    main()
