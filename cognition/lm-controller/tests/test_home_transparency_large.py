"""Sharing is invisible in large frames (pr-s28a C6; AC-a-2, AC-a-3): frames of 4 096 facts (each seat's whole
ten days in one frame), three seeded interleavings. The checks are `test_home_transparency.check_transparent`'s.
`slow`: it runs each night in the `stability` layer, not per PR (D-a-15)."""

from __future__ import annotations

import pytest
from test_home_transparency import INTERLEAVINGS, check_transparent

pytestmark = pytest.mark.slow


@pytest.mark.parametrize("seed", INTERLEAVINGS)
def test_every_seat_view_and_section_is_the_same_shared_or_alone(seed: int) -> None:
    check_transparent(4096, seed)
