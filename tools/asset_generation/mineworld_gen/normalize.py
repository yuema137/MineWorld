"""Normalization applied to every transparent candidate.

Two defects show up consistently in returned RGBA and both bite later rather than immediately,
which is why they are fixed on the way in rather than left for whoever integrates the asset:

1. **No fully-opaque pixel.** Returned alpha tops out at 254, so a renderer that treats 255 as
   "solid" sees the whole sprite as slightly translucent. Rescaled so the maximum becomes 255.

2. **Colour garbage in fully transparent pixels.** Transparent regions carry arbitrary RGB.
   The moment the sprite is scaled or mipmapped, bilinear filtering mixes that RGB in and the
   asset grows a dark fringe. Fixed by bleeding edge colour outwards into the transparent
   region, which is standard practice for sprite atlases and changes no visible pixel.

Both are lossless with respect to what is actually visible. Anything subjective — whether an
asset is *good* — is explicitly not done here; that is the operator's call under ARC-9.
"""

from __future__ import annotations

from pathlib import Path

__all__ = ["normalize_rgba"]

#: How far to bleed colour past the sprite edge, in pixels. Comfortably beyond any mip level a
#: 2D client will sample at, and cheap.
BLEED_RADIUS = 8


def normalize_rgba(path: Path) -> list[str]:
    """Normalize in place. Returns a list of notes describing what changed, for provenance."""
    from PIL import Image, ImageFilter

    image = Image.open(path)
    if image.mode != "RGBA":
        return []
    notes: list[str] = []

    alpha = image.getchannel("A")
    low, high = alpha.getextrema()
    if high == 0:
        return ["alpha channel is entirely empty - asset is blank"]

    if high < 255:
        alpha = alpha.point(lambda v, h=high: min(255, round(v * 255 / h)))
        image.putalpha(alpha)
        notes.append(f"alpha rescaled so peak {high} -> 255")

    if low == 0:
        rgb = image.convert("RGB")
        mask = alpha.point(lambda v: 255 if v > 0 else 0)
        filled = rgb
        grown = mask
        for _ in range(BLEED_RADIUS):
            blurred = filled.filter(ImageFilter.BoxBlur(1))
            expanded = grown.filter(ImageFilter.MaxFilter(3))
            # Keep already-covered pixels exactly; only newly covered ones take the blurred colour.
            filled = Image.composite(filled, blurred, grown)
            grown = expanded
        image = Image.merge("RGBA", (*filled.split(), alpha))
        notes.append(f"edge colour bled {BLEED_RADIUS}px into transparent region")

    if notes:
        image.save(path)
    return notes
