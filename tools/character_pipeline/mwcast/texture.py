# pyright: basic, reportMissingImports=false, reportMissingModuleSource=false
"""Image comparisons for the Gate (PIPELINE.md §7; design §5.7).

Runs from the tool venv (numpy and Pillow), never from CI. Imports are made inside the functions.

- `compare_texels`: the share of texels whose every channel differs by at most 2/255
  (`PARITY_TEXEL_SHARE`, on the same bake device only).
- `compare_masks`: the alpha IoU at alpha > 127, and the RGB mean absolute difference over the union
  foreground
  (the stage 3 acceptance, AB1-13).
"""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from mwcast.standard import PARITY_TEXEL_TOLERANCE as TEXEL_TOLERANCE


@dataclass(frozen=True)
class TexelReport:
    texels: int
    within_tolerance: int

    @property
    def share(self) -> float:
        return self.within_tolerance / self.texels


@dataclass(frozen=True)
class MaskReport:
    alpha_iou: float
    rgb_mean_absolute_difference: float


def _load_rgba(path: Path) -> object:
    from PIL import Image

    return Image.open(path).convert("RGBA")


def compare_texels(candidate: Path, accepted: Path) -> TexelReport:
    """Texel comparison of two images of the same size. Raises ValueError on a size mismatch."""
    import numpy as np

    a = np.array(_load_rgba(candidate), dtype=np.int16)
    b = np.array(_load_rgba(accepted), dtype=np.int16)
    if a.shape != b.shape:
        raise ValueError(f"size differs: {a.shape[:2]} against {b.shape[:2]}")
    within = np.all(np.abs(a - b) <= TEXEL_TOLERANCE, axis=-1)
    return TexelReport(texels=int(within.size), within_tolerance=int(within.sum()))


def compare_masks(candidate: Path, accepted: Path) -> MaskReport:
    """Alpha IoU at alpha > 127 and RGB mean absolute difference over the union foreground."""
    import numpy as np

    a = np.array(_load_rgba(candidate), dtype=np.int16)
    b = np.array(_load_rgba(accepted), dtype=np.int16)
    if a.shape != b.shape:
        raise ValueError(f"size differs: {a.shape[:2]} against {b.shape[:2]}")
    fa = a[..., 3] > 127
    fb = b[..., 3] > 127
    union = fa | fb
    intersection = fa & fb
    iou = float(intersection.sum()) / float(union.sum()) if union.any() else 1.0
    if union.any():
        difference = float(np.abs(a[..., :3] - b[..., :3])[union].mean())
    else:
        difference = 0.0
    return MaskReport(alpha_iou=iou, rgb_mean_absolute_difference=difference)
