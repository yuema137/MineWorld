"""D-impl-22: the cover query `assemble` runs for skin poking through a worn part (mwcast.wardrobe.cover)."""

from __future__ import annotations

from pathlib import Path

import numpy as np
from contact_shapes import open_box

from mwcast.wardrobe import cover


def test_skin_just_under_an_open_sleeve_is_flagged_and_skin_deep_inside_or_outside_is_not(
    tmp_path: Path,
) -> None:
    # A 100 mm square sleeve along z, open at both ends (sides -z and +z dropped). Skin 5 mm under its wall is
    # under the garment; skin at the axis is 50 mm from every wall (beyond the 15 mm reach, left to the ray
    # rules); skin 3 mm outside a wall and skin past the open end are not under it.
    vertices, faces = open_box((0.0, 0.0, 0.0), (0.1, 0.1, 0.4), drop=(0, 1))
    points = np.array(
        [
            [0.095, 0.05, 0.2],  # 5 mm under the +x wall
            [0.05, 0.05, 0.2],  # the axis
            [0.103, 0.05, 0.2],  # 3 mm outside the +x wall
            [0.095, 0.05, 0.5],  # beyond the open end
        ]
    )
    query = tmp_path / "query.npz"
    answer = tmp_path / "flags.npy"
    np.savez(query, points=points, reach_m=np.float64(0.015), v0=vertices, f0=faces)
    assert cover.main([str(query), str(answer)]) == 0
    assert np.load(answer).tolist() == [True, False, False, False]
