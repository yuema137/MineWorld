"""`fit` crops a candidate to its opaque pixels and scales it to the pack's density, never up.

Runs offline on synthetic images, in CI's `python` layer:
    uv run --locked pytest tools/asset_generation/tests
"""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from mineworld_gen.fit import fit


def _canvas(size: tuple[int, int], box: tuple[int, int, int, int], faint_box=None) -> Image.Image:
    image = Image.new("RGBA", size, (0, 0, 0, 0))
    image.paste((200, 120, 80, 255), box)
    if faint_box is not None:
        image.paste((200, 120, 80, 10), faint_box)  # below the 0.06 alpha floor: not part of the sprite
    return image


class FitTest(unittest.TestCase):
    def test_a_tall_subject_is_cropped_and_scaled_down_to_64_px_per_metre(self) -> None:
        # 600 px tall at 4.6 m is 130 px/m native; the pack's density is 64.
        fitted = fit(_canvas((1024, 1024), (100, 200, 800, 800), (0, 0, 1024, 50)), height_m=4.6)
        self.assertEqual(fitted.px_per_m, 64.0)
        self.assertEqual(fitted.image.size, (343, 294))
        self.assertEqual(fitted.anchor, (171.5, 294.0))
        self.assertEqual(fitted.binding("art/generated/x.png", 4.6)["scale"], 0.5)

    def test_a_subject_too_wide_for_64_px_per_metre_keeps_its_native_density(self) -> None:
        # 512 px tall at 30 m is 17 px/m native: kept, and drawn at scale 32/17.07.
        fitted = fit(_canvas((1536, 1024), (0, 256, 1536, 768)), height_m=30.0)
        self.assertAlmostEqual(fitted.px_per_m, 512 / 30.0)
        self.assertEqual(fitted.image.size, (1536, 512))
        self.assertAlmostEqual(fitted.binding("f", 30.0)["scale"], round(32 / (512 / 30.0), 6))

    def test_a_flat_sprite_is_measured_by_its_length(self) -> None:
        # 800 px long at 4 m is 200 px/m; scaled to 64 px/m by width: 256 px.
        fitted = fit(_canvas((1024, 1024), (112, 450, 912, 610)), height_m=0.3, width_m=4.0)
        self.assertEqual(fitted.image.size[0], 256)

    def test_the_door_lies_on_the_bottom_edge_at_its_fraction(self) -> None:
        fitted = fit(_canvas((1024, 1024), (0, 0, 640, 640)), height_m=10.0, door_x_frac=0.25)
        self.assertEqual(fitted.door, (160.0, 640.0))

    def test_an_empty_candidate_is_refused(self) -> None:
        with self.assertRaises(ValueError):
            fit(Image.new("RGBA", (64, 64), (0, 0, 0, 0)), height_m=1.0)


if __name__ == "__main__":
    unittest.main()
