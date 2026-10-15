"""The stage 3 crop rule, pure: `alpha_bbox` on hand-made alpha grids (design §9 B6)."""

from __future__ import annotations

import pytest

from mwcast.mask import alpha_bbox


def _grid(rows: list[list[int]]) -> list[bytes]:
    return [bytes(row) for row in rows]


def test_alpha_bbox_returns_the_hand_computed_box_on_a_6_by_5_grid() -> None:
    # Five rows of six pixels. Two foreground pixels: (x=1, y=1) and (x=4, y=3), both well above 16.
    grid = _grid(
        [
            [0, 0, 0, 0, 0, 0],
            [0, 200, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 200, 0],
            [0, 0, 0, 0, 0, 0],
        ]
    )

    assert alpha_bbox(grid, 16) == (1, 1, 5, 4)


def test_alpha_at_the_threshold_is_not_foreground() -> None:
    # The threshold is strict: 16 is background, 17 is foreground (D+ `prep_inputs.py` tests alpha > 16).
    grid = _grid([[16, 0, 0], [0, 0, 0], [0, 0, 17]])

    assert alpha_bbox(grid, 16) == (2, 2, 3, 3)


def test_a_fully_transparent_image_has_no_foreground_and_is_refused() -> None:
    grid = _grid([[0, 0, 0], [0, 0, 0]])

    with pytest.raises(ValueError, match="no pixel"):
        alpha_bbox(grid, 16)
