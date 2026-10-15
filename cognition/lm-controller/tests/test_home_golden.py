"""C-2 (pr-s28a AC-a-10, AC-a-13's sections): memory renders what the schema-3 code rendered.

The oracle is `tests/golden/s28a_renders.json`, captured once from the schema-3 code by
`capture_golden_renders.py` (`tests/golden/SOURCE.md`). This module recomputes every arm with the code under
test and compares the section digests point by point; it also checks the multi-seat stream the arms are built
from.
"""

from __future__ import annotations

import json
from collections import Counter

from capture_golden_renders import GOLDEN, SEED, absorb_arm, fgc_arm, ingest_arm
from home_streams import SEATS, seat_streams, world


def _golden() -> dict[str, object]:
    loaded: dict[str, object] = json.loads(GOLDEN.read_text(encoding="utf-8"))
    return loaded


def test_the_synthetic_world_is_a_function_of_its_seed_and_shares_its_envelopes() -> None:
    heard = world(SEED, 10)
    again = world(SEED, 10)
    assert [(one.model_dump(), audience) for one, audience in heard] == [
        (one.model_dump(), audience) for one, audience in again
    ]
    streams = seat_streams(heard)
    for one, audience in heard:
        holders = [seat for seat in SEATS if any(each is one for each in streams[seat])]
        assert holders == [seat for seat in SEATS if seat in audience]
    histogram = Counter(len(audience) for _, audience in heard)
    assert sorted(histogram) == list(range(1, 13))
    single = histogram[1] / len(heard)
    assert 0.08 <= single <= 0.18, single


def test_plain_ingest_renders_the_golden_sections() -> None:
    assert ingest_arm(seat_streams(world(SEED, 10))) == _golden()["ingest"]


def test_retention_renders_the_golden_sections() -> None:
    assert absorb_arm(seat_streams(world(SEED, 10))) == _golden()["absorb"]


def test_the_fgc_cassette_replays_to_the_golden_sections() -> None:
    assert fgc_arm() == _golden()["fgc"]
