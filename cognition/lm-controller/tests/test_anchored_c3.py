"""C-3 (step-28 §9.2; pr-s28d §6): every `place` / `item` line cites a held fact anchored to its target, its counts
equal a recount over the person's whole perceived export, every place line's fact is in the objective place
history, and no line repeats a fact an earlier section cites. Also AC-d-8 (transparency, a mini set) and AC-d-9
(degradation) at unit speed.

Three arms: `cafe-like` (S28-b's synthetic multi-seat stream, the default world's shape), `lakeside-like` (the
second world: items named by payloads) and `generated` (random ids, kinds, names and payload shapes). The oracle
is `anchor_oracle`, which imports no code under test and reads the facts in their wire form. `lakeside-real`
(the real binary) runs at the milestone.
"""

from __future__ import annotations

import random
import re
from collections.abc import Generator, Iterable
from dataclasses import dataclass

import anchor_oracle
import anchor_streams
import home_streams
import pytest
from memory_fixtures import ALICE, BOB, CAROL, INSTANCE
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey
from retention_driver import days, feed, retention

from mineworld_cognition.memory.anchors import Anchor, AnchorKind
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity, event_key
from mineworld_cognition.memory.retention.registry import JsonObject
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice
from mineworld_cognition.memory.scopes import RecallQuery, SectionLine
from mineworld_cognition.memory.scopes.host import recall
from mineworld_cognition.memory.scopes.registry import ScopeRegistry
from mineworld_cognition.memory.scopes.stack import MemoryScopeChoice, MemoryStack, resolve_stack

ROLES = {"place": ("at",), "item": ("named",)}
KINDS: dict[str, tuple[AnchorKind, ...]] = {"place": ("place",), "item": ("item",)}
QUERIES = 50
KNOWN = re.compile(r": (\d+) moments I remember, Day (\d+) to Day (\d+)$")


def stack(seat: EntityKey, *scopes: str) -> MemoryStack:
    section: JsonObject = {"section_lines": 6, "section_bytes": 600, "anchors_max": 10_000}
    choices = [
        MemoryScopeChoice(
            id="person",
            parameters={"sections": {"L1": {"lines": 8, "bytes": 400}}},
        ),
        *(MemoryScopeChoice(id=scope, parameters=section) for scope in scopes),
    ]
    choice = MemoryPolicyChoice.model_validate({"policy": "consolidate", "scopes": tuple(choices)})
    return resolve_stack(choice, ScopeRegistry.installed(), seat=seat)


@dataclass(frozen=True)
class Arm:
    name: str
    seats: dict[EntityKey, EntityId]
    given: dict[EntityKey, list[PerceivedEvent]]
    names: dict[EntityId, str]


def cafe() -> Arm:
    given = home_streams.seat_streams(home_streams.world(7, 7))
    seats = dict(list(home_streams.CAST.items())[:4])
    return Arm("cafe-like", seats, {seat: given[seat] for seat in seats}, {})


def lakeside() -> Arm:
    given = anchor_streams.streams(anchor_streams.lakeside(11, 7))
    seats = dict(zip(anchor_streams.LAKE_SEATS, anchor_streams.LAKE_PEOPLE, strict=True))
    return Arm("lakeside-like", seats, given, anchor_streams.THING_NAMES)


def generated() -> Arm:
    seats = {EntityKey("alice"): ALICE, EntityKey("bob"): BOB}
    given = {
        seat: anchor_streams.generated(
            31 + n, me, 300, others=(BOB, CAROL) if me == ALICE else (ALICE,)
        )[0]
        for n, (seat, me) in enumerate(seats.items())
    }
    return Arm("generated", seats, given, {})


ARMS = {"cafe-like": cafe, "lakeside-like": lakeside, "generated": generated}


def build(home: MemoryHome, arm: Arm, *scopes: str) -> dict[EntityKey, SeatView]:
    views: dict[EntityKey, SeatView] = {}
    for seat, me in arm.seats.items():
        view = home.seat(StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=me))
        home.bind_scopes(view, stack(seat, *scopes))
        if arm.names:
            with view.transaction():
                book = NameBook.load(view)
                for entity, name in arm.names.items():
                    book.learn(entity, name)
        feed(view, days(arm.given[seat]), retention("consolidate"))
        views[seat] = view
    return views


def queries(arm: Arm, seat: EntityKey, at: int) -> list[RecallQuery]:
    export = [f.model_dump(mode="json") for f in arm.given[seat]]
    me = arm.seats[seat]
    pool = [
        Anchor(EntityId(entity), kind)
        for scope in ("place", "item")
        for kind in KINDS[scope]
        for entity in anchor_oracle.anchors_in(export, ROLES[scope], KINDS[scope], me)
    ]
    rng = random.Random(f"{arm.name}/{seat}")
    return [
        RecallQuery(at=at, anchors=tuple(rng.sample(pool, min(len(pool), rng.randrange(1, 4)))))
        for _ in range(QUERIES)
    ]


def check(arm: Arm, views: dict[EntityKey, SeatView]) -> int:
    exports = {seat: [f.model_dump(mode="json") for f in arm.given[seat]] for seat in arm.seats}
    lines_checked = 0
    for seat, view in views.items():
        me = arm.seats[seat]
        export = exports[seat]
        ids = {str(f["id"]) for f in export}
        at = max(int(str(f["at"])) for f in export) + 60
        for query in queries(arm, seat, at):
            section = recall(view, stack(seat, "place", "item"), query, on_defect="raise").section
            earlier: set[str] = set()
            for line in section.lines:
                ends = {e for r in line.citation.ranges for e in (r.first, r.last)}
                if line.level in ("place", "item"):
                    lines_checked += 1
                    _check_line(line, query, view, export, ids, me, exports.values())
                    assert not ends & earlier, (arm.name, seat, line.text)  # check 4
                earlier |= ends
    return lines_checked


def _check_line(
    line: SectionLine,
    query: RecallQuery,
    view: SeatView,
    export: list[dict[str, object]],
    ids: set[str],
    me: EntityId,
    exports: Iterable[list[dict[str, object]]],
) -> None:
    scope = line.level
    (cited,) = {r.first for r in line.citation.ranges}
    assert view.holds(event_key(cited)) and cited in ids  # check 1: held, perceived
    targets = [a.entity for a in query.anchors if a.kind in KINDS[scope]]
    histories = {
        t: anchor_oracle.history(export, t, ROLES[scope], KINDS[scope], me) for t in targets
    }
    about = [t for t in targets if cited in histories[t]]
    assert about, (scope, line.text)  # check 1: anchored to a target
    known = KNOWN.search(line.text)
    if known is not None:  # check 2: the counts are a recount over the whole export
        visits, first, last = (int(group) for group in known.groups())
        counted = {anchor_oracle.known(export, t, ROLES[scope], KINDS[scope], me) for t in about}
        assert any(
            c is not None
            and (c[0], c[1] // 86_400 + 1, c[2] // 86_400 + 1) == (visits, first, last)
            for c in counted
        ), (scope, line.text, counted)
    if scope == "place":  # check 3: in the objective place history
        everyone = list(exports)
        assert any(cited in anchor_oracle.objective(everyone, t) for t in about)


@pytest.fixture(scope="module", params=sorted(ARMS))
def arm_home(request: pytest.FixtureRequest) -> Generator[tuple[Arm, dict[EntityKey, SeatView]]]:
    arm = ARMS[request.param]()
    with MemoryHome.open(":memory:", INSTANCE) as home:
        yield arm, build(home, arm, "place", "item")


def test_c3_on_every_arm(arm_home: tuple[Arm, dict[EntityKey, SeatView]]) -> None:
    arm, views = arm_home
    assert check(arm, views) > 0, arm.name


def test_transparency_none_and_home_give_the_same_views_and_sections() -> None:
    # AC-d-8 (mini): three lakeside seats alone in their homes, and together in one.
    arm = lakeside()
    with MemoryHome.open(":memory:", INSTANCE) as shared:
        together = build(shared, arm, "place", "item")
        for seat in arm.seats:
            alone_arm = Arm(arm.name, {seat: arm.seats[seat]}, arm.given, arm.names)
            with MemoryHome.open(":memory:", INSTANCE) as own:
                alone = build(own, alone_arm, "place", "item")[seat]
                assert alone.dump() == together[seat].dump()
                at = max(f.at for f in arm.given[seat]) + 60
                for query in queries(arm, seat, at):
                    mine = recall(alone, stack(seat, "place", "item"), query, on_defect="raise")
                    ours = recall(
                        together[seat], stack(seat, "place", "item"), query, on_defect="raise"
                    )
                    assert mine.section.render() == ours.section.render()


def test_removing_and_re_adding_place_rebuilds_a_recount_over_held_facts() -> None:
    # AC-d-9 under `consolidate` (QS28d-5): the rebuilt rows count the facts the seat still holds.
    arm = generated()
    seat = EntityKey("alice")
    one = Arm(arm.name, {seat: ALICE}, arm.given, {})
    with MemoryHome.open(":memory:", INSTANCE) as home:
        view = build(home, one, "place")[seat]
        removed = home.bind_scopes(view, stack(seat))
        assert removed.removed == ("place",)
        assert not home.connection.execute("SELECT 1 FROM place_known").fetchall()
        added = home.bind_scopes(view, stack(seat, "place"))
        assert added.added == ("place",)
        held = [f.model_dump(mode="json") for f in arm.given[seat] if view.holds(event_key(f.id))]
        rows = home.connection.execute(
            "SELECT entity, visits, first_at, last_at FROM place_known WHERE seat = ?",
            (view.seat_id,),
        ).fetchall()
        assert rows
        for entity, visits, first_at, last_at in rows:
            assert anchor_oracle.known(held, entity, ("at",), ("place",), ALICE) == (
                visits,
                first_at,
                last_at,
            )
