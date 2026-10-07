#!/usr/bin/env python3
"""The strand atlas the default character's hair cards are textured with.

    blender --background --python hair_atlas.py -- <out.png>

**What a hair-card texture has to be.** A card is a narrow ribbon that stands
for a lock of a few dozen hairs.  Its texture is what turns the ribbon into
hairs: many fine strands, each a few pixels wide, separated by transparency,
tapering and thinning out toward the tip, with per-strand brightness so a
lock is not one flat colour.  The previous atlas (`character_textures.py`
step 5) drew a dozen thick bars per lock; at portrait framing each bar was a
visible stripe, and a head of them read as a cap of flat stripes.

**Layout.** 2048 x 1024, eight columns of 256 px.  `v` runs root (0, the
bottom of the image in Blender's convention) to tip (1).  Columns 0-5 are
locks of decreasing density; 6 and 7 are sparse wisps for the loose,
face-framing strands and the flyaways.  The cards pick a column at random, so
neighbouring cards do not repeat.

RGB carries strand-level shading around 0.5-1.0; alpha carries coverage.
`shaders/hair_card.gdshader` reads both.  Our own procedural work; no third-
party input.  The CC0 OwlishMedia alphas (`fetch_hair_alphas.sh`) can replace
it once fetched; the card UV layout does not change.

Seeded: the bake is re-runnable byte for byte.
"""

from __future__ import annotations

import sys

import bpy  # pylint: disable=import-error
import numpy as np

W, H = 2048, 1024
COLS = 8
SEED = 20261006

# (strands, width px at root, wave amplitude px, length range) per column
COLUMN_STYLE = [
    (84, 3.4, 3.0, (0.80, 1.00)),
    (76, 3.2, 5.0, (0.70, 1.00)),
    (70, 3.6, 4.0, (0.75, 1.00)),
    (64, 3.0, 7.0, (0.60, 1.00)),
    (56, 3.2, 6.0, (0.55, 0.98)),
    (46, 3.0, 8.0, (0.50, 0.95)),
    (22, 2.8, 9.0, (0.65, 1.00)),
    (13, 2.6, 11.0, (0.55, 1.00)),
]


def draw_column(rgb, alpha, col, rng):
    count, w0, wave, (lmin, lmax) = COLUMN_STYLE[col]
    x_lo = col * (W // COLS)
    cw = W // COLS
    rows = np.arange(H, dtype=np.float64)
    t_all = rows / (H - 1)                         # 0 root .. 1 tip
    # the lock is gathered at the root and fans a little toward the tip
    for _ in range(count):
        length = rng.uniform(lmin, lmax)
        root_x = cw * 0.5 + rng.normal(0.0, cw * 0.13)
        fan = rng.normal(0.0, cw * 0.10)
        freq = rng.uniform(1.2, 2.6)
        phase = rng.uniform(0, 6.283)
        amp = wave * rng.uniform(0.5, 1.3)
        bright = rng.uniform(0.50, 1.0)
        width = w0 * rng.uniform(0.75, 1.25)
        t = t_all / length
        live = t <= 1.0
        xc = root_x + fan * t ** 1.4 + amp * np.sin(6.283 * freq * t + phase) * t
        xc = np.clip(xc, 3, cw - 4)
        wr = width * (1.0 - 0.65 * np.clip(t, 0, 1) ** 1.5)
        # fade in over the first 3 % (the root is under other hair) and out
        # over the last 22 %, where a real lock thins to single hairs
        fade = np.clip(t / 0.03, 0, 1) * np.clip((1.0 - t) / 0.22, 0, 1)
        for off in range(-4, 5):
            px = np.floor(xc).astype(int) + off
            d = np.abs(px + 0.5 - xc)
            cov = np.clip(wr * 0.5 - d + 0.5, 0.0, 1.0) * fade
            cov[~live] = 0.0
            sel = cov > 0.0
            r = rows[sel].astype(int)
            c = (px[sel] + x_lo)
            a_new = cov[sel]
            a_old = alpha[r, c]
            alpha[r, c] = 1.0 - (1.0 - a_old) * (1.0 - a_new)
            # over-composite the strand's brightness
            rgb[r, c] = rgb[r, c] * (1.0 - a_new) + bright * a_new


def main() -> int:
    out = sys.argv[sys.argv.index("--") + 1]
    rng = np.random.default_rng(SEED)
    rgb = np.full((H, W), 0.55, dtype=np.float64)
    alpha = np.zeros((H, W), dtype=np.float64)
    for col in range(COLS):
        draw_column(rgb, alpha, col, rng)
    px = np.zeros((H, W, 4), dtype=np.float32)
    px[..., 0] = rgb
    px[..., 1] = rgb
    px[..., 2] = rgb
    px[..., 3] = alpha
    img = bpy.data.images.new("hair_atlas", W, H, alpha=True)
    img.pixels.foreach_set(px.ravel())
    img.filepath_raw = out
    img.file_format = "PNG"
    img.save()
    cover = [float(alpha[:, c * 256:(c + 1) * 256].mean()) for c in range(COLS)]
    print("hair atlas", out, "mean coverage per column",
          " ".join(f"{c:.2f}" for c in cover))
    return 0


if __name__ == "__main__":
    main()
