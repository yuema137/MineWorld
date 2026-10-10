"""AP4-1 (identity), AP4-2 (ids and order), AP4-3 (durable before the cursor): the store and ingestion."""

from __future__ import annotations

import re
import sqlite3
from pathlib import Path

import pytest
from memory_fixtures import (
    ALICE,
    BOB,
    CAFE,
    INSTANCE,
    OTHER_INSTANCE,
    alice_identity,
    arrived,
    fact,
    names_of_everyone,
    spoke,
)
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId

from mineworld_cognition.memory import render
from mineworld_cognition.memory.ingest import ConflictingFact, OutOfOrder, ingest
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.render import Rendered
from mineworld_cognition.memory.store import MemoryStore, NotAMemoryStore, StoreIdentityMismatch

SOURCE = Path(__file__).resolve().parent.parent / "src" / "mineworld_cognition"
U64_MAX = "18446744073709551615"


def _store(path: Path) -> MemoryStore:
    return MemoryStore.open(path, alice_identity())


def _ingested(path: Path, facts: list[PerceivedEvent]) -> bytes:
    with _store(path) as store:
        ingest(store, facts, facts[-1].id)
        return store.dump()


@pytest.mark.parametrize(
    ("field", "changed"),
    [
        ("world_instance", {"world_instance": OTHER_INSTANCE}),
        ("seat", {"seat": EntityKey("bob")}),
        ("self_entity", {"self_entity": BOB}),
        ("schema", {"schema_version": 2}),
    ],
)
def test_a_foreign_identity_is_refused_by_name_and_changes_nothing(
    tmp_path: Path, field: str, changed: dict[str, object]
) -> None:
    # AP4-1 (a). Its mutation skips the instance comparison: the first case opens silently.
    path = tmp_path / "alice.sqlite"
    before = _ingested(path, [*names_of_everyone(), spoke(10, 100, BOB, ALICE, "Hello.", CAFE)])
    foreign = alice_identity().model_copy(update=changed)
    with pytest.raises(StoreIdentityMismatch) as raised:
        MemoryStore.open(path, foreign)
    assert raised.value.field == field
    stored = {"world_instance": INSTANCE, "seat": "alice", "self_entity": ALICE, "schema": "1"}
    assert raised.value.stored == stored[field]
    assert stored[field] in str(raised.value)
    assert raised.value.given in str(raised.value)
    with _store(path) as store:
        assert store.dump() == before


def test_a_new_store_records_its_identity_and_keeps_it_with_the_cursor(tmp_path: Path) -> None:
    # AP4-1 (b), and a store closed and reopened keeps identity and cursor.
    path = tmp_path / "alice.sqlite"
    with _store(path) as store:
        ingest(store, names_of_everyone(), EventId("9"))
    with _store(path) as store:
        meta = {key: store.meta(key) for key in ("world_instance", "seat", "self_entity", "schema")}
        assert meta == {
            "world_instance": INSTANCE,
            "seat": "alice",
            "self_entity": ALICE,
            "schema": "1",
        }
        assert store.cursor() == "9"


def test_a_database_that_is_not_a_store_is_refused(tmp_path: Path) -> None:
    path = tmp_path / "world.sqlite"
    connection = sqlite3.connect(path)
    connection.execute("CREATE TABLE facts (id INTEGER)")
    connection.commit()
    connection.close()
    with pytest.raises(NotAMemoryStore):
        MemoryStore.open(path, alice_identity())


def test_ids_order_as_integers_up_to_u64_and_stay_strings(tmp_path: Path) -> None:
    # AP4-2 (a). Its mutation orders by CAST(event_id AS INTEGER): u64 max overflows SQLite's INTEGER.
    facts = [
        spoke(9, 100, BOB, ALICE, "nine", CAFE),
        spoke(10, 101, BOB, ALICE, "ten", CAFE),
        spoke(U64_MAX, 102, BOB, ALICE, "last", CAFE),
    ]
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, facts, EventId(U64_MAX))
        assert store.cursor() == U64_MAX
        order = [
            event
            for (event,) in store.connection.execute("SELECT event_id FROM l0 ORDER BY event_key")
        ]
    assert order == ["9", "10", U64_MAX]


def test_a_redelivery_is_a_no_op_and_a_changed_one_is_refused() -> None:
    # AP4-2 (b). Its mutation lets a re-delivery overwrite: the conflict passes silently.
    with MemoryStore.open(":memory:", alice_identity()) as store:
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
    with MemoryStore.open(":memory:", alice_identity()) as store:
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
    # AP4-2 (c): no int( or float( in memory/ or compress/ but three conversions of a boolean to
    # SQLite's 0/1.
    found: set[str] = set()
    for package in ("memory", "compress"):
        for path in sorted((SOURCE / package).rglob("*.py")):
            text = path.read_text(encoding="utf-8")
            found.update(m.group(0) for m in re.finditer(r"\b(?:int|float)\([^)]*\)", text))
    assert found == {
        "int(rendered.mine)",
        "int(rendered.fallback)",
        'int(involved and record.salience == "notable")',
    }


def test_a_failure_inside_ingestion_leaves_store_and_cursor_untouched(
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
    with MemoryStore.open(":memory:", alice_identity()) as store:
        ingest(store, names_of_everyone(), EventId("4"))
        before = store.dump()
        with pytest.raises(RuntimeError):
            ingest(store, facts, EventId("24"))
        assert store.dump() == before
        assert store.cursor() == "4"


def test_a_store_releases_its_file_when_closed(tmp_path: Path) -> None:
    # AP4-12's Windows half: an open handle would block the delete (pr-s10-p5 D-P5-13's lesson).
    path = tmp_path / "alice.sqlite"
    with _store(path) as store:
        ingest(store, names_of_everyone(), EventId("4"))
    path.unlink()
    assert not path.exists()
    identity = StoreIdentity(world_instance=INSTANCE, seat=EntityKey("alice"), self_entity=ALICE)
    with MemoryStore.open(path, identity) as reopened:
        assert reopened.cursor() is None
