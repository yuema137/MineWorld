"""Gate 2 on seat views (pr-s28b C6, §6.5, AC-b-9; step-28 §11.1's checkpoint): FG-b's real-binary retention
scenario in a home **shared** by real people.

`social_cafe_100`'s Alice and Bob exports (`mineworld perceived`, read only through the CLI by `History`) are
delivered day by day, each seat's own order kept, into one shared home of the two seats and, in lockstep, into two
one-seat homes, under `consolidate` for all 100 days. After every one of either seat's passes: its view ≤ the
target after the pass and ≤ the bound before it; coverage against the seat's export (AFGb-3's oracle); C-4 (a) to (c)
(`summary_oracle`); its view in the shared home equal to its one-seat home's (AS28-3 on real data). After every
day: I-a-6 (a) and (b) as ruled. The shared home verifies holders at every pass (AS28-6). Figures are literals.
"""

from __future__ import annotations

import re
import time
from itertools import zip_longest

import pytest
from ac10_harness import History
from binary import REPOSITORY
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey
from retention_driver import days, retention
from retention_oracle import check_coverage
from summary_oracle import check
from support import run

from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.home.schema import K_ACCESS_OVERHEAD_BYTES as K
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import absorb

pytestmark = pytest.mark.real_binary

MIB = 1_048_576
BOUND = 20 * MIB  # the default per-person bound (pr-fg-b2 §4.1; operator, 2026-10-14)
TARGET = BOUND * 3 // 4
SEATS = (EntityKey("alice"), EntityKey("bob"))


def _entity(history: History, person: str) -> EntityId:
    yaml = (REPOSITORY / "worlds" / "social-cafe" / "people" / f"{person}.yaml").read_text(
        encoding="utf-8"
    )
    name = re.search(r"^name:\s*(.+?)\s*$", yaml, re.MULTILINE)
    assert name is not None, person
    (entity,) = [entity for entity, known in history.names.items() if known == name.group(1)]
    return entity


def test_consolidate_in_a_shared_home_of_real_people_holds_every_gate(
    social_cafe_100: History,
) -> None:
    history = social_cafe_100
    streams: dict[EntityKey, list[PerceivedEvent]] = {
        EntityKey("alice"): history.alice,
        EntityKey("bob"): history.bob,
    }
    me = {seat: _entity(history, str(seat)) for seat in SEATS}
    ids = {seat: [fact.id for fact in facts] for seat, facts in streams.items()}
    kept = retention("consolidate")
    passes = dict.fromkeys(SEATS, 0)
    started = time.monotonic()

    def identity(seat: EntityKey) -> StoreIdentity:
        return StoreIdentity(world_instance=history.instance, seat=seat, self_entity=me[seat])

    def fed(view: SeatView, frame: list[PerceivedEvent]) -> int:
        report = run(absorb(view, frame, frame[-1].id, kept), timeout_s=120)
        return len(report.passes)

    with MemoryHome.open(":memory:", history.instance, verify="pass") as home:
        shared = {seat: home.seat(identity(seat)) for seat in SEATS}
        alone_homes = {seat: MemoryHome.open(":memory:", history.instance) for seat in SEATS}
        try:
            alone = {seat: alone_homes[seat].seat(identity(seat)) for seat in SEATS}
            for pair in zip_longest(*(list(days(streams[seat])) for seat in SEATS)):
                for seat, frame in zip(SEATS, pair, strict=True):
                    if frame is None:
                        continue
                    before = len(shared[seat].dump())
                    ran = fed(shared[seat], frame)
                    assert fed(alone[seat], frame) == ran, seat
                    dump = shared[seat].dump()
                    assert dump == alone[seat].dump(), seat  # AS28-3 on real data
                    assert len(dump) <= BOUND, seat
                    if ran:
                        passes[seat] += ran
                        assert before <= BOUND and len(dump) <= TARGET, (seat, before, len(dump))
                        check_coverage(dump, ids[seat])
                        check(dump, streams[seat], me[seat])
                separate = 0
                for seat in SEATS:
                    size = len(alone_homes[seat].dump())
                    held = len(alone[seat].row_bytes("l0"))
                    assert size <= len(alone[seat].dump()) + K * held, seat  # I-a-6 (b)
                    separate += size
                assert len(home.dump()) <= separate  # I-a-6 (a)
        finally:
            for one in alone_homes.values():
                one.close()
    wall = time.monotonic() - started
    print(f"\n[Gate 2 shared home] passes {dict(passes)}, {wall:.1f} s")
    assert min(passes.values()) >= 95
