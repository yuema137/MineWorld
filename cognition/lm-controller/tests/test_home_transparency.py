"""Sharing is invisible (pr-s28a C6; AC-a-2, AC-a-3; step-28 I-S28-2), frames of 256 and 4 096 facts.

Twelve seats of the synthetic world (seed 7, 10 days), each through `absorb` under `consolidate`: their seat-view
dumps and the 200 rendered sections of every pass point and of the end are byte-identical whether the seats share
one home (`home`) or each has a home of its own (`none`), for three seeded interleavings of the seats' frames.
Frames of 1 and 7 are `test_home_transparency_frames.py`, the real stream `test_home_transparency_real.py`; the
harness ran as one file of ≈ 280 s locally, over §12's 120 s per file (R-a-2). The oracle is always the `none`
arm, computed here (`home_harness`).
"""

from __future__ import annotations

import pytest
from home_harness import SYNTHETIC, alone, consolidate, home_arm, synthetic
from home_streams import SEATS

INTERLEAVINGS = (1, 2, 3)


def check_transparent(frame: int, seed: int) -> str:
    """AC-a-2, AC-a-3 for one frame size and interleaving; returns the shared home's dump digest."""
    oracle = alone(frame)
    shared = home_arm(synthetic(frame), frame, seed, SYNTHETIC, consolidate)
    assert set(shared.snapshots) == set(oracle.snapshots) == set(SEATS)
    for seat in SEATS:
        # Pass points were taken: a day's pass for every day after the first, and the end (two days of
        # frames of 1 hold one pass).
        assert len(oracle.snapshots[seat]) >= (2 if frame == 1 else 9), seat
        assert shared.snapshots[seat] == oracle.snapshots[seat], seat
    return shared.home


@pytest.mark.parametrize("seed", INTERLEAVINGS)
@pytest.mark.parametrize("frame", [256, 4096])
def test_every_seat_view_and_section_is_the_same_shared_or_alone(frame: int, seed: int) -> None:
    check_transparent(frame, seed)
