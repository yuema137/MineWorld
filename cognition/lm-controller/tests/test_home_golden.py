"""C-2 (pr-s28a AC-a-10, AC-a-13's sections): memory renders what the schema-3 code rendered.

The oracle is `tests/golden/s28a_renders.json`, captured once from the schema-3 code by
`capture_golden_renders.py` (`tests/golden/SOURCE.md`). This module recomputes every arm with the code under
test and compares the section digests point by point; it also checks the multi-seat stream the arms are built
from.
"""

from __future__ import annotations

import json
from collections import Counter

import capture_golden_renders
import pytest
from capture_golden_renders import CAST_IDS, GOLDEN, SEED, absorb_arm, fgc_arm, identity, ingest_arm
from home_streams import CAST, SEATS, queries, seat_streams, world
from memory_fixtures import INSTANCE
from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.memory import (
    MemoryHome,
    MemorySection,
    RecallQuery,
    SeatView,
    ingest,
    retrieve,
)
from mineworld_cognition.memory.records import event_key
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.scopes import host
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, resolve_stack


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


@pytest.fixture
def through_an_explicit_person_stack(monkeypatch: pytest.MonkeyPatch) -> None:
    """Every render of the arms made through the recall host with a written `[{id: person}]` stack, every
    check on and `raise` on a defect (pr-s28c AC-c-1 (b), I-c-1)."""
    choice = MemoryPolicyChoice(policy="consolidate", scopes=(MemoryScopeChoice(id="person"),))
    stack = resolve_stack(choice, ScopeRegistry.installed(), seat=EntityKey("golden"))

    def via_the_host(store: SeatView, query: RecallQuery) -> MemorySection:
        return host.recall(store, stack, query, on_defect="raise").section

    monkeypatch.setattr(capture_golden_renders, "retrieve", via_the_host)


@pytest.mark.usefixtures("through_an_explicit_person_stack")
def test_an_explicit_person_stack_renders_the_golden_sections() -> None:
    streams = seat_streams(world(SEED, 10))
    assert ingest_arm(streams) == _golden()["ingest"]
    assert absorb_arm(streams) == _golden()["absorb"]


def test_sections_name_only_held_facts() -> None:
    # I-S28-1 in retrieval: two seats of one home with disjoint streams (the world's facts alternately);
    # every line of 200 queries per seat cites only facts that seat perceived. Its mutation (M-a-5) drops the
    # seat predicate from the L0 query: a home-mate's records reach the section.
    heard = world(SEED, 3)
    seats = SEATS[:2]
    streams = {
        seat: [one for n, (one, _) in enumerate(heard) if n % 2 == k]
        for k, seat in enumerate(seats)
    }
    with MemoryHome.open(":memory:", INSTANCE) as home:
        for seat in seats:
            view = home.seat(identity(seat, CAST[seat]))
            ingest(view, streams[seat], streams[seat][-1].id)
        for index, seat in enumerate(seats):
            view = home.seat(identity(seat, CAST[seat]))
            perceived = {event_key(one.id) for one in streams[seat]}
            now = streams[seat][-1].at
            for at, who, words in queries(SEED, index, 0, now, CAST_IDS):
                section = retrieve(view, RecallQuery(at=at, counterparts=who, words=words))
                for cited in section.citations():
                    assert event_key(cited.first) in perceived, (seat, cited)
                    assert event_key(cited.last) in perceived, (seat, cited)
