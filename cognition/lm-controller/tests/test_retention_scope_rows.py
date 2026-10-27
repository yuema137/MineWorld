"""AC-d-10: the bound's backstop drops scope rows after chapterless episodes and before stable facts, in each
table's evict order, forgetting no range (pr-s28d §5.8).

The arm is `generated`; the fixture scope `Tally` keeps one row per entity named. The expected order is the fold
of the facts written here.
"""

from __future__ import annotations

import json

from anchor_streams import generated
from memory_fixtures import ALICE, BOB, CAROL, INSTANCE
from mineworld_sdk.wire.ids import EntityKey
from scope_fixtures import Tally

from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention import bound
from mineworld_cognition.memory.scopes.person import PersonScope
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryStack

SEAT = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)


def test_the_step_runs_after_chapterless_episodes_and_before_stable_facts() -> None:
    names = [name for _, name in bound.STEPS]
    assert names.index("scope-rows") == names.index("chapterless-episodes") + 1
    assert names.index("stable-facts") == names.index("scope-rows") + 1


def test_scope_rows_go_least_recently_seen_first_and_forget_nothing() -> None:
    facts, _ = generated(9, ALICE, 200, others=(BOB, CAROL))
    registry = ScopeRegistry({"person": PersonScope, "tally": Tally}, base=("person",))
    stack = MemoryStack(
        seat=SEAT.seat, scopes=(registry.bind("person", {}), registry.bind("tally", {}))
    )
    with MemoryHome.open(":memory:", INSTANCE) as home:
        view = home.seat(SEAT)
        home.bind_scopes(view, stack)
        ingest(view, facts, facts[-1].id)

        def rows() -> list[list[object]]:
            return [
                json.loads(line)[1:]
                for line in view.dump().decode().splitlines()
                if line.startswith('["tally_seen",')
            ]

        before = rows()
        forgotten = view.row_bytes("forgotten")
        size = view.size()
        one = len(json.dumps(["tally_seen", *before[0]], separators=(",", ":"))) + 1
        with view.transaction():
            removed, after = bound._scope_rows(view, size, size - one)  # pyright: ignore[reportPrivateUsage]
        assert removed >= 1 and after <= size - one and after == view.size()
        evicted = sorted(before, key=lambda row: (int(str(row[4])), str(row[0])))[:removed]
        assert [row for row in before if row not in rows()] == sorted(
            evicted, key=lambda r: str(r[0])
        )
        assert view.row_bytes("forgotten") == forgotten
