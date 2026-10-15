"""Sharing is invisible in small frames (pr-s28a C6; AC-a-2, AC-a-3): frames of 1 fact (the first two days, R-a-2)
and of 7, three seeded interleavings each. The checks are `test_home_transparency.check_transparent`'s."""

from __future__ import annotations

import pytest
from test_home_transparency import INTERLEAVINGS, check_transparent


@pytest.mark.parametrize("seed", INTERLEAVINGS)
@pytest.mark.parametrize("frame", [1, 7])
def test_every_seat_view_and_section_is_the_same_shared_or_alone(frame: int, seed: int) -> None:
    check_transparent(frame, seed)
