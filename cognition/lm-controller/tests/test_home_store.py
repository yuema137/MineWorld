"""A seat view's own behaviour and ingestion into it (pr-s28a C3, C4; `test_memory_store.py`'s P4 AP4-1 … AP4-3,
on a home): identity, cursor, id order, re-delivery, order, atomicity, the canonical dump, and the meter that
keeps `content_size()` equal to the dump without building it (FG-b2 §4.2, now per seat).

The first tests hold facts directly, with a hand-written rendering, so the view is tested without ingestion; the
byte oracle is always `dump()`, read back in the test, never a size the view reports about itself.
"""

from __future__ import annotations

import json
import re
from collections.abc import Generator
from contextlib import contextmanager
from pathlib import Path

import pytest
from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    CAROL,
    INSTANCE,
    alice_identity,
    arrived,
    fact,
    names_of_everyone,
    seat_view,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId

from mineworld_cognition.memory import ConflictingFact, OutOfOrder, ingest, render
from mineworld_cognition.memory.home import HomeIdentityMismatch, MemoryHome, SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity, event_key
from mineworld_cognition.memory.render import Rendered

SOURCE = Path(__file__).resolve().parent.parent / "src" / "mineworld_cognition"
U64_MAX = "18446744073709551615"


def bob_identity() -> StoreIdentity:
    return StoreIdentity(world_instance=INSTANCE, seat=EntityKey("bob"), self_entity=BOB)


@contextmanager
def one_seat(path: Path | None = None) -> Generator[SeatView]:
    with MemoryHome.open(":memory:" if path is None else path, INSTANCE) as home:
        yield home.seat(alice_identity())


def rendering(gist: str, notable: bool = True) -> Rendered:
    return Rendered(gist, (), "notable" if notable else "ambient", False, 300)


def held(view: SeatView, *facts: PerceivedEvent) -> None:
    with view.transaction():
        for each in facts:
            view.hold(each, f"{int(each.id):064x}", rendering(f"gist {each.id}"))


def meta_bytes(dump: bytes) -> int:
    return sum(len(line) + 1 for line in dump.split(b"\n") if line.startswith(b'["meta"'))


def assert_metered(view: SeatView) -> None:
    dump = view.dump()
    assert view.size() == len(dump)
    assert view.content_size() == len(dump) - meta_bytes(dump)


def test_a_seat_keeps_its_identity_and_refuses_another_self_by_name(tmp_path: Path) -> None:
    # AP4-1 on a home: the seat's identity is stored on first use and a different self entity is refused,
    # changing nothing.
    path = tmp_path / "memory.sqlite"
    with MemoryHome.open(path, INSTANCE) as home:
        view = home.seat(alice_identity())
        assert home.seat(alice_identity()) is view
        held(view, spoke(10, 100, BOB, ALICE, "Hello.", CAFE))
        before = home.dump()
        foreign = alice_identity().model_copy(update={"self_entity": BOB})
        with pytest.raises(HomeIdentityMismatch) as raised:
            home.seat(foreign)
        assert (raised.value.field, raised.value.stored, raised.value.given) == (
            "self_entity",
            ALICE,
            BOB,
        )
        assert home.dump() == before


def test_a_seat_keeps_its_cursor_and_identity_across_a_reopen(tmp_path: Path) -> None:
    path = tmp_path / "memory.sqlite"
    with one_seat(path) as view, view.transaction():
        view.set_meta("cursor", "9")
    with one_seat(path) as view:
        meta = {key: view.meta(key) for key in ("world_instance", "seat", "self_entity", "schema")}
        assert meta == {
            "world_instance": INSTANCE,
            "seat": "alice",
            "self_entity": ALICE,
            "schema": "6",
        }
        assert view.cursor() == "9"


def test_ids_order_as_integers_up_to_u64_and_stay_strings() -> None:
    # AP4-2 (a): the view orders by `EventKey`; ids stay decimal strings in every table.
    with one_seat() as view:
        held(
            view,
            spoke(9, 100, BOB, ALICE, "nine", CAFE),
            spoke(10, 101, BOB, ALICE, "ten", CAFE),
            spoke(U64_MAX, 102, BOB, ALICE, "last", CAFE),
        )
        order = [str(cells["event_id"]) for cells, _ in view.row_bytes("l0")]
        lines = [json.loads(line) for line in view.dump().splitlines() if line.startswith(b'["l0"')]
    assert order == ["9", "10", U64_MAX]
    assert [line[1] for line in lines] == [9, 10, int(U64_MAX)]


def test_the_view_shows_its_held_facts_with_their_other_participants_in_envelope_order() -> None:
    with MemoryHome.open(":memory:", INSTANCE) as home:
        alice = home.seat(alice_identity())
        bob = home.seat(bob_identity())
        crowd = fact(
            20,
            200,
            "fished",
            None,
            subjects=[CAFE],
            participants=[CAROL, ALICE, BOB, CAROL],
            place=CAFE,
        )
        held(alice, crowd)
        held(bob, crowd)
        who = {
            name: [cells["entity"] for cells, _ in view.row_bytes("l0_who")]
            for name, view in (("alice", alice), ("bob", bob))
        }
    assert who == {"alice": [CAROL, BOB], "bob": [CAROL, ALICE]}


def test_the_meter_is_the_dump() -> None:
    # FG-b2 §4.2 per seat, two seats in one home: after holds, private writes, releases, and a rollback, each
    # seat's `content_size()` is its dump less its `meta` lines. Its mutation (M-a-4) leaves the `l0_who`
    # lines out of what `hold` adds.
    with MemoryHome.open(":memory:", INSTANCE) as home:
        alice = home.seat(alice_identity())
        bob = home.seat(bob_identity())
        for view in (alice, bob):
            assert_metered(view)
        shared = spoke(30, 300, BOB, ALICE, "Morning, Alice.", CAFE)
        held(alice, shared, arrived(31, 301, CAROL, CAFE))
        held(bob, shared)
        for view in (alice, bob):
            assert_metered(view)
        with alice.transaction():
            alice.connection.execute(
                "INSERT INTO names (seat, entity, name, source_key) VALUES (?, ?, ?, ?)",
                (alice.seat_id, BOB, "Bob Achterberg", event_key(shared.id)),
            )
            alice.set_meta("newest", event_key(shared.id))
        with bob.transaction():
            bob.release([event_key(shared.id)])
        for view in (alice, bob):
            assert_metered(view)
        inside: list[int] = []
        with pytest.raises(RuntimeError), alice.transaction():
            alice.release([event_key(shared.id)])
            inside.append(alice.content_size())
            raise RuntimeError("roll back")
        assert inside[0] < alice.content_size()
        for view in (alice, bob):
            assert_metered(view)


def test_row_bytes_of_l0_and_l0_who_sum_to_their_lines_in_the_dump() -> None:
    with one_seat() as view:
        held(view, spoke(40, 400, BOB, ALICE, "Hello «café» 雨", CAFE), arrived(41, 401, BOB, CAFE))
        for measured in (False, True):
            if measured:
                view.content_size()
            for table in ("l0", "l0_who"):
                lines = [
                    line
                    for line in view.dump().split(b"\n")
                    if line.startswith(f'["{table}",'.encode())
                ]
                assert sum(size for _, size in view.row_bytes(table)) == sum(
                    len(line) + 1 for line in lines
                )


def test_a_home_releases_its_file_when_closed(tmp_path: Path) -> None:
    # AP4-12's Windows half: an open handle would block the delete.
    path = tmp_path / "memory.sqlite"
    with one_seat(path) as view:
        held(view, spoke(50, 500, BOB, ALICE, "Bye.", CAFE))
    path.unlink()
    assert not path.exists()
    with one_seat(path) as reopened:
        assert reopened.cursor() is None


# ── Ingestion into a seat view (P4's AP4-2, AP4-3; moved from test_memory_store.py at C4) ─────────────


def test_a_redelivery_is_a_no_op_and_a_changed_one_is_refused() -> None:
    # AP4-2 (b). Its mutation lets a re-delivery overwrite: the conflict passes silently.
    with seat_view() as store:
        ingest(store, [spoke(5, 100, BOB, ALICE, "Hello.", CAFE)], EventId("5"))
        before = store.dump()
        report = ingest(store, [spoke(5, 100, BOB, ALICE, "Hello.", CAFE)], EventId("5"))
        assert (report.records, report.redeliveries) == (0, 1)
        assert store.dump() == before
        with pytest.raises(ConflictingFact) as raised:
            ingest(store, [spoke(5, 100, BOB, ALICE, "Goodbye.", CAFE)], EventId("5"))
        assert raised.value.event == "5"
        assert store.dump() == before


def test_facts_out_of_order_are_refused() -> None:
    # AP4-2 (b): a batch [5, 3]; and a fact below the acknowledged cursor that was never delivered.
    with seat_view() as store:
        with pytest.raises(OutOfOrder):
            ingest(
                store,
                [spoke(5, 100, BOB, ALICE, "a", CAFE), spoke(3, 99, BOB, ALICE, "b", CAFE)],
                EventId("5"),
            )
        assert store.cursor() is None
        ingest(store, [spoke(5, 100, BOB, ALICE, "a", CAFE)], EventId("20"))
        with pytest.raises(OutOfOrder):
            ingest(store, [spoke(12, 101, BOB, ALICE, "c", CAFE)], EventId("20"))


def test_no_id_is_ever_converted_to_a_number() -> None:
    # AP4-2 (c): no int( or float( in memory/ (retention included) or compress/ but three conversions of a
    # boolean to SQLite's 0/1 (`involved` is derived and no longer stored, F-S28a-7), and the canonical
    # dump's writing of an id as its number (`memory/home/encoding.py`, ARC-101), the one place an id becomes
    # a number, on its way into a dump line and never back.
    found: set[str] = set()
    for package in ("memory", "compress"):
        for path in sorted((SOURCE / package).rglob("*.py")):
            text = path.read_text(encoding="utf-8")
            found.update(m.group(0) for m in re.finditer(r"\b(?:int|float)\([^)]*\)", text))
    assert found == {
        "int(rendered.mine)",
        "int(rendered.fallback)",
        'int(involved and record.salience == "notable")',
        "int(value)",
    }


def test_a_failure_inside_ingestion_leaves_the_seat_and_its_cursor_untouched(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    # AP4-3: a renderer raises on the 3rd of 5 facts. Its mutation moves the cursor before the records,
    # outside the transaction: the cursor moves.
    def explode(_fact: PerceivedEvent, _me: EntityId, _names: NameBook) -> Rendered:
        raise RuntimeError("a renderer failed")

    monkeypatch.setitem(render.RENDERERS, "exploding", explode)
    facts = [
        arrived(20, 100, BOB, CAFE),
        spoke(21, 101, BOB, ALICE, "one", CAFE),
        fact(22, 102, "exploding", None, subjects=[BOB], participants=[BOB], place=CAFE),
        spoke(23, 103, BOB, ALICE, "three", CAFE),
        spoke(24, 104, BOB, ALICE, "four", CAFE),
    ]
    with seat_view() as store:
        ingest(store, names_of_everyone(), EventId("4"))
        before = store.dump()
        with pytest.raises(RuntimeError):
            ingest(store, facts, EventId("24"))
        assert store.dump() == before
        assert store.cursor() == "4"
