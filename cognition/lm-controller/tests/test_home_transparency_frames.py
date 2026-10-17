"""Sharing is invisible in small frames (pr-s28a C6; AC-a-2, AC-a-3): frames of 7, and of 1 fact (the first two
days, R-a-2), three seeded interleavings each. The checks are `test_home_transparency.check_transparent`'s.
Frames of 1 are `slow`: they run each night in the `stability` layer, not per PR (D-a-15)."""

from __future__ import annotations

import pytest
from test_home_transparency import INTERLEAVINGS, check_transparent


@pytest.mark.parametrize("seed", INTERLEAVINGS)
@pytest.mark.parametrize("frame", [pytest.param(1, marks=pytest.mark.slow), 7])
def test_every_seat_view_and_section_is_the_same_shared_or_alone(frame: int, seed: int) -> None:
    check_transparent(frame, seed)
