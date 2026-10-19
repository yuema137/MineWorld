"""Sharing is invisible on the real stream (pr-s28a C6; AC-a-2, AC-a-3's real half): social-cafe's twelve people
(seed 7, 2 days, `conftest.social_cafe_12`), frames of 256, one interleaving, under `consolidate`. Every seat's
dump and 200 sections at every pass point and at the end are the same shared or alone."""

from __future__ import annotations

import pytest
from home_harness import Cast, consolidate, home_arm, none_arm
from home_streams import Cafe12

pytestmark = pytest.mark.real_binary


def test_every_real_seat_is_the_same_shared_or_alone(social_cafe_12: Cafe12) -> None:
    cast = Cast(social_cafe_12.me, social_cafe_12.instance, sorted(social_cafe_12.me.values()))
    streams = social_cafe_12.streams
    oracle = none_arm(streams, 256, cast, consolidate)
    shared = home_arm(streams, 256, 1, cast, consolidate)
    assert set(shared.snapshots) == set(oracle.snapshots) == set(streams)
    for seat in streams:
        assert len(oracle.snapshots[seat]) >= 2, seat  # day 1's pass and the end
        assert shared.snapshots[seat] == oracle.snapshots[seat], seat
