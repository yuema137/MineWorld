"""Fit a chosen candidate to a 2D pack's scale: crop it to its opaque pixels, scale it, report its binding.

    python -m mineworld_gen.fit CANDIDATE.png OUT.png --height-m 4.6 [--width-m W] [--door-x-frac F]

A generated candidate comes out at the generator's canvas size with transparent margins. A pack's sprite
is drawn at a known density, so the cut is (pr-s29f §5.8, ported from the spike's `tools/normalize.gd`
on `vis/2d-generated-assets`, which never reached `main`):

1. crop to the bounding box of the pixels whose alpha is above ``ALPHA_FLOOR`` (the spike's 0.06);
2. scale with Lanczos so the sprite's real size maps to ``px_per_m`` pixels per metre, where
   ``px_per_m = min(64, native)`` and ``native`` is the cropped height in pixels / ``height_m`` (or the
   cropped width / ``width_m`` for a flat sprite measured by its length). It never upscales: a sprite
   wider than the generator can draw at 64 px/m keeps its native density (Q-S29f-3);
3. print the binding block: ``anchor`` at the bottom centre of the crop, ``door`` (the bottom centre, or
   the point ``door_x_frac`` along the bottom edge), ``scale = 32 / px_per_m``, ``height_m``, ``ink``.

Example: a 1024 x 1024 candidate whose house fills 700 x 600 px, at ``--height-m 4.6``, is cropped to
700 x 600 (130 px/m native) and scaled to 64 px/m, 343 x 294 px, with ``scale`` 0.5.

No network, no key: Pillow only.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

from PIL import Image

ALPHA_FLOOR = 0.06
PACK_PX_PER_M = 64.0
DRAW_PX_PER_M = 32.0


@dataclass(frozen=True)
class Fitted:
    image: Image.Image
    px_per_m: float
    anchor: tuple[float, float]
    door: tuple[float, float]

    def binding(self, file: str, height_m: float) -> dict[str, object]:
        return {
            "anchor": [self.anchor[0], self.anchor[1]],
            "door": [self.door[0], self.door[1]],
            "file": file,
            "height_m": height_m,
            "ink": "none",
            "scale": round(DRAW_PX_PER_M / self.px_per_m, 6),
        }


def fit(
    image: Image.Image,
    height_m: float,
    width_m: float | None = None,
    door_x_frac: float | None = None,
) -> Fitted:
    """Crops ``image`` to its opaque bounds and scales it to at most 64 px per metre."""
    if height_m <= 0 or (width_m is not None and width_m <= 0):
        raise ValueError("height_m and width_m must be positive")
    rgba = image.convert("RGBA")
    alpha = rgba.getchannel("A").point(lambda a: 255 if a > ALPHA_FLOOR * 255 else 0)
    box = alpha.getbbox()
    if box is None:
        raise ValueError("the image has no pixel above the alpha floor")
    cropped = rgba.crop(box)
    w, h = cropped.size
    native = (w / width_m) if width_m is not None else (h / height_m)
    px_per_m = min(PACK_PX_PER_M, native)
    k = px_per_m / native
    if k < 1.0:
        cropped = cropped.resize((max(1, round(w * k)), max(1, round(h * k))), Image.Resampling.LANCZOS)
    out_w, out_h = cropped.size
    frac = 0.5 if door_x_frac is None else door_x_frac
    return Fitted(
        image=cropped,
        px_per_m=px_per_m,
        anchor=(out_w / 2.0, float(out_h)),
        door=(round(out_w * frac, 1), float(out_h)),
    )


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="mineworld_gen.fit")
    p.add_argument("candidate", type=Path)
    p.add_argument("out", type=Path)
    p.add_argument("--height-m", type=float, required=True)
    p.add_argument("--width-m", type=float, default=None)
    p.add_argument("--door-x-frac", type=float, default=None)
    p.add_argument("--file", default=None, help="The binding's `file` (default: art/generated/<out name>).")
    args = p.parse_args(argv)
    fitted = fit(Image.open(args.candidate), args.height_m, args.width_m, args.door_x_frac)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    fitted.image.save(args.out, optimize=True)
    file = args.file or f"art/generated/{args.out.name}"
    block = {args.out.stem: fitted.binding(file, args.height_m)}
    json.dump(block, sys.stdout, indent=2)
    print()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
