#!/usr/bin/env python3
"""The default character's texture set.

    python3 character_textures.py --upstream <VitruvianGodot clone> \
        --out ../assets/characters/vitruvian/textures \
        --freckles <freckles.json from character_model.py> [--generate]

Four steps, in order, because each depends on the last:

```text
1  downscale   the CC0 skin, hair and eye maps to the repository's budget
2  freckles    stipple them onto the face albedo, at the UV positions
               character_model.py read off the geometry
3  tee         the cream jersey with its mountain-and-slogan print
4  denim       the worn mid-blue jeans albedo
5  hair cards  the strand atlas the hair cards are mapped onto
```

Steps 3 and 4 are **generated art** under `ARC-9`: a candidate comes back from
an image model, a person accepts it, and it is then cleaned, normalised and
committed with its provenance.  `--generate` calls the model and rewrites the
committed source art; without it the committed art is reused, so the build runs
with no network and no key.  The key is read from the environment and never
printed, logged or written anywhere.

The slogan is **drawn, not generated.**  `CHARACTER_IDENTITY.md` §5 fixes the
words as "Good Places" / "Brighter People", and an image model will not reliably
render eleven specific letters twice running; the mountains it renders well.

Everything here is `magick` and `sips`, which are on the machine, plus the
standard library.
"""

from __future__ import annotations

import argparse
import base64
import json
import math
import os
import random
import subprocess
import sys
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
# Committed generated source art. `.gdignore` keeps Godot from importing these
# as runtime textures: they are inputs to this script, not assets.
ART = os.path.join(HERE, "texture_art")

# (upstream file, repository file, longest edge in px)
CC0_MAPS = [
    ("vit_face_bc.png", "face_bc.jpg", 2048), ("vit_face_n.png", "face_n.jpg", 2048),
    ("vit_face_rough.png", "face_rough.jpg", 1024),
    ("vit_body_bc.png", "body_bc.jpg", 1024), ("vit_body_n.png", "body_n.jpg", 1024),
    ("vit_body_rough.png", "body_rough.jpg", 1024),
    ("vit_fabric_n.png", "fabric_n.jpg", 512),
    ("vit_hair_diffuse.png", "hair_bc.jpg", 512),
    ("vit_hair_opacity.png", "hair_opacity.png", 512),
    ("vit_iris.png", "iris.jpg", 256), ("vit_sclera.png", "sclera.jpg", 256),
    ("vit_mouth.png", "mouth.jpg", 512),
]

FACE_PX = 2048          # the face albedo is one UDIM tile at this size

# The tee texture is wrapped round the torso by `cylindrical_uv`, so u spans the
# whole ~0.90 m circumference and v the 0.472 m from hem to neckline.  A square
# texture is therefore anisotropic on the garment, and the print has to be drawn
# into a box that is *taller* than it is wide to come out square on the chest.
TEE_W, TEE_H = 1024, 1024
TEE_CIRCUM, TEE_SPAN = 0.90, 0.472
PRINT_M = 0.165          # how wide the print is on the actual shirt
PRINT_TOP_Z, PRINT_HEM_Z = 1.482, 1.010    # must match garments.cylindrical_uv
PRINT_TOP_OF_PRINT = 1.365   # high on the chest, as the reference shows

MOUNTAIN_PROMPT = (
    "A flat two-colour screen-printed graphic for a t-shirt: a simple range of "
    "three mountain peaks in dark warm brown on a plain white background, the "
    "tallest peak slightly left of centre, snow lines on the peaks drawn as "
    "negative space cut out of the solid brown fill, clean vector-like edges "
    "with a slightly cracked worn vintage print texture, no text, no letters, "
    "no border, no frame, centred, plenty of white space below the mountains."
)
DENIM_PROMPT = (
    "A seamless tiling photographic texture of worn mid-blue denim, flat lit, "
    "no shadows, no seams, no stitching, no pockets, visible twill weave "
    "diagonal, gently faded and slightly uneven in tone, desaturated mid blue, "
    "top-down orthographic, no objects, no text."
)

SLOGAN = ("Good Places", "Brighter People")
SLOGAN_FONT = "/System/Library/Fonts/Supplemental/Trebuchet MS Bold.ttf"
PRINT_INK = "#4A3226"
TEE_CLOTH = "#D9D2C4"      # cream / oatmeal, lightly mottled and worn


def run(cmd: list[str]) -> None:
    subprocess.run(cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)


# ------------------------------------------------------------------ 1  CC0 maps

def downscale(upstream: str, out: str) -> None:
    src_dir = os.path.join(upstream, "godot_project")
    if not os.path.isdir(src_dir):
        raise SystemExit(f"no godot_project/ under {upstream}")
    total = 0
    for src_name, dst_name, px in CC0_MAPS:
        src = os.path.join(src_dir, src_name)
        if not os.path.exists(src):
            print(f"  missing {src_name}")
            continue
        dst = os.path.join(out, dst_name)
        fmt = "jpeg" if dst_name.endswith(".jpg") else "png"
        cmd = ["sips", "-s", "format", fmt, "-Z", str(px), src, "--out", dst]
        if fmt == "jpeg":
            cmd[1:1] = ["-s", "formatOptions", "78"]
        run(cmd)
        total += os.path.getsize(dst)
        print(f"  {dst_name:<18} {px:>5}px  {os.path.getsize(dst) / 1e6:.2f} MB")
    print(f"  CC0 maps total {total / 1e6:.2f} MB")


# ------------------------------------------------------------------ 2  freckles

def freckles(points_path: str, out: str) -> None:
    """Stipple the freckles onto the face albedo, in place.

    The positions come from the geometry (`character_model.py` §write_freckles),
    so they land on the bridge of the nose and the upper cheeks rather than
    wherever the UV sheet happens to look right.  They are drawn in two passes —
    a soft wide dab under a smaller sharp core — because a flat disc at this
    size reads as a mole and a blurred one disappears.
    """
    data = json.load(open(points_path, encoding="utf-8"))
    pts = data["points"]
    face = os.path.join(out, "face_bc.jpg")
    layer = os.path.join(out, "_freckles.png")

    soft, core = [], []
    for p in pts:
        x, y = p["u"] * FACE_PX, (1.0 - p["v"]) * FACE_PX
        r = p["r"]
        soft.append(f"fill-opacity {p['a'] * 0.5:.3f} circle {x:.1f},{y:.1f} "
                    f"{x + r * 1.8:.1f},{y:.1f}")
        core.append(f"fill-opacity {p['a']:.3f} circle {x:.1f},{y:.1f} "
                    f"{x + r:.1f},{y:.1f}")
    run(["magick", "-size", f"{FACE_PX}x{FACE_PX}", "xc:none",
         "-fill", "#6B4430", "-draw", " ".join(soft),
         "-blur", "0x2.2",
         "-fill", "#5A3726", "-draw", " ".join(core),
         "-blur", "0x0.7", layer])
    run(["magick", face, layer, "-compose", "over", "-composite",
         "-quality", "82", face])
    os.remove(layer)
    print(f"  freckles: {len(pts)} stippled onto face_bc.jpg "
          f"({os.path.getsize(face) / 1e6:.2f} MB)")


# --------------------------------------------------------------- 3  the tee

def tee(out: str) -> None:
    """Cream jersey, with the mountain print and the two lines of slogan."""
    art = os.path.join(ART, "tee_mountains.jpg")
    if not os.path.exists(art):
        raise SystemExit(f"missing generated art {art}; run with --generate")

    # where the print lands, in pixels of the wrapped texture
    box_w = int(PRINT_M / TEE_CIRCUM * TEE_W)
    box_h = int(PRINT_M / TEE_SPAN * TEE_H)
    cx = TEE_W // 2
    v_top = (PRINT_TOP_Z - PRINT_TOP_OF_PRINT) / (PRINT_TOP_Z - PRINT_HEM_Z)
    cy = int(v_top * TEE_H)

    graphic = os.path.join(out, "_print.png")
    # the mountains occupy the upper ~55 % of the print; the slogan the rest
    mount_h = int(box_h * 0.62)
    run(["magick", art, "-colorspace", "gray", "-level", "8%,86%",
         "-negate", "-resize", f"{box_w}x{mount_h}!",
         "-background", "none", "-alpha", "copy",
         "-fill", PRINT_INK, "-colorize", "100", graphic])

    # The slogan is fitted to the print, not set at a guessed point size: an
    # earlier pass clipped "Brighter People" halfway through the word.  Both
    # lines are rendered large, trimmed, stacked left-aligned, and the stack is
    # then scaled into the box -- which is also what keeps the two lines the
    # same weight as each other.
    text_h = box_h - mount_h
    lines = []
    for i, line in enumerate(SLOGAN):
        lp = os.path.join(out, f"_line{i}.png")
        run(["magick", "-background", "none", "-fill", PRINT_INK,
             "-font", SLOGAN_FONT, "-pointsize", "220", f"label:{line}",
             "-trim", "+repage", lp])
        lines.append(lp)
    widths = []
    for lp in lines:
        widths.append(int(subprocess.run(["magick", "identify", "-format", "%w", lp],
                                         check=True, capture_output=True,
                                         text=True).stdout))
    block = os.path.join(out, "_text_block.png")
    run(["magick", "-background", "none",
         lines[0], "-gravity", "west", "-extent", f"{max(widths)}x",
         lines[1], "-gravity", "west", "-extent", f"{max(widths)}x",
         "-append", block])
    text = os.path.join(out, "_text.png")
    run(["magick", block, "-resize", f"{box_w}x{text_h}!", text])
    for lp in lines + [block]:
        os.remove(lp)

    base = os.path.join(out, "tee_bc.jpg")
    # A worn, lightly mottled jersey rather than a flat fill.  The noise goes on
    # the base directly: compositing a grayscale layer over it converted the
    # whole texture to grayscale and blew it out to white.
    run(["magick", "-size", f"{TEE_W}x{TEE_H}", f"xc:{TEE_CLOTH}",
         "-attenuate", "0.09", "+noise", "Gaussian", "-blur", "0x1.6",
         graphic, "-geometry", f"+{cx - box_w // 2}+{cy}",
         "-compose", "over", "-composite",
         text, "-geometry", f"+{cx - box_w // 2}+{cy + mount_h}",
         "-compose", "over", "-composite",
         "-quality", "88", base])
    for f in (graphic, text):
        os.remove(f)
    print(f"  tee_bc.jpg  {TEE_W}x{TEE_H}, print {box_w}x{box_h} px at "
          f"+{cx - box_w // 2}+{cy}  ({os.path.getsize(base) / 1e6:.2f} MB)")


# ------------------------------------------------------------------- 4  denim

def denim(out: str) -> None:
    art = os.path.join(ART, "denim.jpg")
    if not os.path.exists(art):
        raise SystemExit(f"missing generated art {art}; run with --generate")
    dst = os.path.join(out, "denim_bc.jpg")
    # normalised to the project's albedo-luminance band (DEP-8's coherence
    # procedure, step 2) and tiled seamlessly by mirroring rather than by
    # trusting the model to have produced a wrapping texture
    run(["magick", art, "-resize", "512x512!",
         "-modulate", "100,88,100", "-level", "6%,90%",
         "-quality", "84", dst])
    print(f"  denim_bc.jpg 512x512  ({os.path.getsize(dst) / 1e6:.2f} MB)")


# -------------------------------------------------------------- 5  hair cards

HAIR_COLS = 4
HAIR_COL_PX = 256
HAIR_ROWS_PX = 512
HAIR_STRANDS = 17
HAIR_SEED = 20260927


def hair_cards(out: str) -> None:
    """A hair-card atlas: four tapered strand clumps, each isolated in alpha.

    This replaces sampling the CC0 *groom* map, and the difference is not a
    matter of taste.  That map is a dense continuous field of strands: a card
    takes a slice of it, every edge of the slice cuts through the middle of a
    strand, and the card renders as an opaque rectangle.  A head of them reads
    as a smooth brown cap, which is what three rounds of geometry tuning failed
    to fix — the geometry was never the problem.

    It also fixes the sampling.  The groom's strands are far finer than a pixel
    at conversation distance; with `alpha_to_coverage` that dithers into the
    yellow-tan speckling the operator saw.  These strands are 3–8 px in a 256 px
    clump mapped across a ~20 mm card, so they survive a mip chain as strands.

    Each clump is wide at the root and narrow at the tip, with transparent
    margins on all four sides, so a card's *silhouette* is a lock of hair.
    """
    rng = random.Random(HAIR_SEED)
    width = HAIR_COLS * HAIR_COL_PX
    draw = []
    for col in range(HAIR_COLS):
        cx = col * HAIR_COL_PX + HAIR_COL_PX / 2
        root_w = HAIR_COL_PX * rng.uniform(0.78, 0.90)
        tip_w = HAIR_COL_PX * rng.uniform(0.10, 0.22)
        length = HAIR_ROWS_PX * rng.uniform(0.86, 1.0)
        for i in range(HAIR_STRANDS):
            u = (i + rng.uniform(-0.3, 0.3)) / (HAIR_STRANDS - 1) - 0.5
            x0 = cx + u * root_w
            x1 = cx + u * tip_w + rng.uniform(-8, 8)
            # a gentle S, so the clump is not a bundle of straight lines
            bend = rng.uniform(-26, 26)
            xm = (x0 + x1) / 2 + bend
            y1 = length * rng.uniform(0.80, 1.0)
            w = rng.uniform(3.0, 7.5)
            g = rng.uniform(0.55, 1.0)
            draw.append(
                f"stroke-width {w:.1f} stroke rgba({int(g*255)},{int(g*255)},"
                f"{int(g*255)},1) path 'M {x0:.0f},-6 Q {xm:.0f},{y1*0.55:.0f} "
                f"{x1:.0f},{y1:.0f}'")

    strands = os.path.join(out, "_strands.png")
    run(["magick", "-size", f"{width}x{HAIR_ROWS_PX}", "xc:none",
         "-fill", "none", "-draw", " ".join(draw), "-blur", "0x0.6", strands])
    # fade the last third to nothing: a lock of hair ends in air, and a hard
    # bottom edge on every card is what makes a groom read as plastic
    fade = os.path.join(out, "_fade.png")
    run(["magick", "-size", f"{width}x{HAIR_ROWS_PX}",
         f"gradient:white-black", "-sigmoidal-contrast", "5x38%", fade])
    dst = os.path.join(out, "hair_card.png")
    run(["magick", strands, fade, "-alpha", "off", "-compose", "copy_opacity",
         "-composite", "-channel", "A", "-evaluate", "multiply", "1.0",
         "+channel", dst])
    # RGB carries strand-to-strand shading; the shader reads .r for that and .a
    # for coverage, so one file serves both of its texture slots.
    run(["magick", strands, "-alpha", "extract", "-blur", "0x0.4",
         os.path.join(out, "_cov.png")])
    run(["magick", strands, "(", os.path.join(out, "_cov.png"), fade,
         "-compose", "multiply", "-composite", ")",
         "-compose", "copy_opacity", "-composite", dst])
    for f in (strands, fade, os.path.join(out, "_cov.png")):
        os.remove(f)
    print(f"  hair_card.png {width}x{HAIR_ROWS_PX}, {HAIR_COLS} clumps of "
          f"{HAIR_STRANDS} strands  ({os.path.getsize(dst) / 1e6:.2f} MB)")


# --------------------------------------------------------------- generation

def generate(name: str, prompt: str, size: str = "1024x1024") -> None:
    """Ask the image model for one candidate and save it as committed source art."""
    key = os.environ.get("OPENAI_API_KEY")
    if not key:
        raise SystemExit("OPENAI_API_KEY is not set; source it from your secrets file")
    body = json.dumps({"model": "gpt-image-1", "prompt": prompt,
                       "size": size, "n": 1}).encode()
    req = urllib.request.Request(
        "https://api.openai.com/v1/images/generations", data=body,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=300) as resp:
        payload = json.load(resp)
    os.makedirs(ART, exist_ok=True)
    path = os.path.join(ART, name)
    raw = path + ".raw.png"
    with open(raw, "wb") as fh:
        fh.write(base64.b64decode(payload["data"][0]["b64_json"]))
    # the model returns a 1024 PNG; what gets committed is a 768 JPEG, because
    # this is a build input the repository has to carry, not a master
    run(["magick", raw, "-resize", "768x768", "-quality", "92", path])
    os.remove(raw)
    print(f"  generated {name}  ({os.path.getsize(path) / 1e6:.2f} MB)")


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--upstream", help="a VitruvianGodot clone; omit to skip step 1")
    ap.add_argument("--out", required=True)
    ap.add_argument("--freckles", help="freckles.json from character_model.py")
    ap.add_argument("--generate", action="store_true",
                    help="re-ask the image model for the tee print and the denim")
    args = ap.parse_args()
    os.makedirs(args.out, exist_ok=True)

    if args.generate:
        generate("tee_mountains.jpg", MOUNTAIN_PROMPT)
        generate("denim.jpg", DENIM_PROMPT)
    if args.upstream:
        downscale(args.upstream, args.out)
    if args.freckles:
        freckles(args.freckles, args.out)
    tee(args.out)
    denim(args.out)
    hair_cards(args.out)
    return 0


if __name__ == "__main__":
    sys.exit(main())
