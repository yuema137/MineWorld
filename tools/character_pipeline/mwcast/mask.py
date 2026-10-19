# pyright: basic, reportMissingImports=false, reportMissingModuleSource=false
"""Stage 3, the mask (port of `prep_inputs.py`; PIPELINE.md §2, §4.1 `mask`).

This module runs from the tool venv (`tools/character_pipeline/requirements.txt`), never from CI and never
from the root workspace. Its third-party imports are made inside the functions, so importing this module
needs no package. `alpha_bbox` is pure. The crop, the removal, the square pad and the resize follow
`prep_inputs.py`:

- the crop box is `fit.mask.crop_box` when given; otherwise the bounding box of the alpha found in a first
  full-image pass (the automatic crop, the default for new characters);
- `rembg` removes the background with the session named by `fit.mask.session`;
- the foreground is square-padded so that its longer side is `fg_ratio` of the frame;
- the result is resized to `size_px` with LANCZOS and saved as RGBA.
"""

from __future__ import annotations

from collections.abc import Sequence
from pathlib import Path
from typing import Any

from mwcast.records import FitParams, Mask


def alpha_bbox(alpha_rows: Sequence[bytes], threshold: int) -> tuple[int, int, int, int]:
    """The bounding box `(x0, y0, x1, y1)` of the alpha values above `threshold`; `x1` and `y1` are exclusive.

    Raises ValueError when no pixel is above the threshold (a fully transparent image has no foreground)."""
    xs: list[int] = []
    ys: list[int] = []
    for y, row in enumerate(alpha_rows):
        for x, value in enumerate(row):
            if value > threshold:
                xs.append(x)
                ys.append(y)
    if not xs:
        raise ValueError(f"no pixel has alpha above {threshold}; the image has no foreground")
    return min(xs), min(ys), max(xs) + 1, max(ys) + 1


def run_mask(fit: FitParams, reference: Path, out: Path) -> tuple[int, int, int, int]:
    """Stage 3: write the RGBA mask PNG at `out` and return the crop box that was used."""
    return run_mask_with(fit.mask, reference, out)


def run_mask_with(settings: Mask, reference: Path, out: Path) -> tuple[int, int, int, int]:
    """Stage 3 with the mask settings alone (the S32-p wardrobe stages have no `fit.yaml` for an image)."""
    import numpy as np
    from PIL import Image
    from rembg import new_session, remove

    session = new_session(settings.session)
    reference_image: Any = Image.open(reference).convert("RGB")
    if settings.crop_box is not None:
        box = settings.crop_box
    else:
        full = np.array(remove(reference_image, session=session))
        alpha_rows = [bytes(row) for row in full[..., 3].astype(np.uint8)]
        box = alpha_bbox(alpha_rows, settings.alpha_threshold)
    cut = remove(reference_image.crop(box), session=session)
    alpha = np.array(cut)[..., 3]
    print(f"crop {box}; alpha coverage above 127: {(alpha > 127).mean():.3f}")
    padded = _square_pad(cut, settings.fg_ratio, settings.alpha_threshold, settings.size_px)
    out.parent.mkdir(parents=True, exist_ok=True)
    padded.save(out)
    return box


def _square_pad(rgba: Any, fg_ratio: float, threshold: int, size_px: int) -> Any:
    """Square-pad the foreground of an RGBA image: its longer side becomes `fg_ratio` of the frame."""
    import numpy as np
    from PIL import Image

    assert isinstance(rgba, Image.Image)
    alpha = np.array(rgba)[..., 3]
    ys, xs = np.nonzero(alpha > threshold)
    x0, y0, x1, y1 = int(xs.min()), int(ys.min()), int(xs.max()) + 1, int(ys.max()) + 1
    print(f"   foreground bbox x {x0}..{x1} y {y0}..{y1}")
    obj = rgba.crop((x0, y0, x1, y1))
    side = int(max(obj.size) / fg_ratio)
    canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas.paste(obj, ((side - obj.size[0]) // 2, (side - obj.size[1]) // 2))
    return canvas.resize((size_px, size_px), Image.Resampling.LANCZOS)
