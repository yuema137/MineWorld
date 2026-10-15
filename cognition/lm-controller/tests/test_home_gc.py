"""Records live exactly while they are held (pr-s28a C3; AC-a-5, AC-a-6; I-S28-4): hold, release and collection
over the synthetic 12-seat world, the last holder, a seat leaving, and a peer's record verified, not trusted.

The oracle is the home's tables read by this test with its own two anti-joins, and the home dump; the home's own
`verify` runs too, but is never what a test asserts.
"""

from __future__ import annotations

import sqlite3
from collections.abc import Generator
from contextlib import contextmanager

import pytest
from home_streams import CAST, SEATS, world
from memory_fixtures import ALICE, BOB, CAFE, CAROL, INSTANCE, spoke
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityKey
from retention_stream import draw

from mineworld_cognition.memory.home import MemoryHome
from mineworld_cognition.memory.ingest import digest
from mineworld_cognition.memory.records import EventKey, StoreIdentity, event_key
from mineworld_cognition.memory.render import Rendered
from mineworld_cognition.memory.scopes.world import ConflictingFact


def identity(seat: EntityKey) -> StoreIdentity:
    return StoreIdentity(world_instance=INSTANCE, seat=seat, self_entity=CAST[seat])


def rendering(one: PerceivedEvent) -> Rendered:
    return Rendered(f"{one.event_type} {one.id}", (), "notable", False, 300)


@contextmanager
def home(verify: bool = True) -> Generator[MemoryHome]:
    with MemoryHome.open(
        ":memory:", INSTANCE, verify="every-commit" if verify else "off"
    ) as opened:
        yield opened


def anti_joins(db: sqlite3.Connection) -> tuple[set[str], set[str]]:
    """(records no access row cites, held keys with no record): both empty when I-S28-4 holds."""
    records = {key for (key,) in db.execute("SELECT event_key FROM fact")}
    held = {key for (key,) in db.execute("SELECT event_key FROM access")}
    who = {key for (key,) in db.execute("SELECT event_key FROM fact_who")}
    return records - held, (held | who) - records


def test_after_every_commit_the_records_are_exactly_the_held_keys() -> None:
    # AC-a-5: the synthetic world held seat by seat, one transaction per fact, and a seeded script of
    # releases (a seat lets go of a seeded third of what it holds, every 25 facts).
    heard = world(7, 3)
    with home() as opened:
        views = {seat: opened.seat(identity(seat)) for seat in SEATS}
        held: dict[EntityKey, list[EventKey]] = {seat: [] for seat in SEATS}
        for index, (one, audience) in enumerate(heard):
            with opened.transaction():
                for seat in sorted(audience):
                    views[seat].hold(one, digest(one), rendering(one))
                    held[seat].append(event_key(one.id))
            assert anti_joins(opened.connection) == (set(), set())
            if index % 25 == 24:
                seat = SEATS[draw(7, index, 70) % len(SEATS)]
                gone = [key for n, key in enumerate(held[seat]) if draw(7, index, 71 + n) % 3 == 0]
                with opened.transaction():
                    views[seat].release(gone)
                held[seat] = [key for key in held[seat] if key not in gone]
                assert anti_joins(opened.connection) == (set(), set())
        records = {key for (key,) in opened.connection.execute("SELECT event_key FROM fact")}
    assert records == {key for keys in held.values() for key in keys}


def _two_holders(first: str, second: str) -> bytes:
    shared = spoke(60, 600, BOB, ALICE, "Shared words.", CAFE)
    key = event_key(shared.id)
    with home() as opened:
        alice = opened.seat(identity(EntityKey("alice")))
        bob = opened.seat(identity(EntityKey("bob")))
        views = {"alice": alice, "bob": bob}
        with opened.transaction():
            for view in (alice, bob):
                view.hold(shared, digest(shared), rendering(shared))
        before = views[second].dump()
        with opened.transaction() as db:
            views[first].release([key])
            assert db.execute(
                "SELECT count(*) FROM fact WHERE event_key = ?", (key,)
            ).fetchone() == (1,)
        assert views[second].dump() == before
        with opened.transaction() as db:
            views[second].release([key])
            # Collected inside the releasing transaction, before its commit.
            assert db.execute(
                "SELECT count(*) FROM fact WHERE event_key = ?", (key,)
            ).fetchone() == (0,)
            assert db.execute(
                "SELECT count(*) FROM fact_who WHERE event_key = ?", (key,)
            ).fetchone() == (0,)
        return opened.dump()


def test_the_last_holder_collects() -> None:
    # AC-a-6. Its mutation (M-a-2) collects without the holder check: the first release deletes the record.
    assert _two_holders("alice", "bob") == _two_holders("bob", "alice")


def test_a_seat_that_leaves_takes_only_what_no_one_else_holds() -> None:
    mine = spoke(70, 700, BOB, ALICE, "Only Alice heard this.", CAFE)
    ours = spoke(71, 701, BOB, ALICE, "Both heard this.", CAFE)
    with home() as opened:
        alice = opened.seat(identity(EntityKey("alice")))
        bob = opened.seat(identity(EntityKey("bob")))
        with opened.transaction():
            for one in (mine, ours):
                alice.hold(one, digest(one), rendering(one))
            bob.hold(ours, digest(ours), rendering(ours))
            alice.set_meta("cursor", "71")
        bob_view = bob.dump()
        opened.remove_seat(EntityKey("alice"))
        db = opened.connection
        assert anti_joins(db) == (set(), set())
        assert {key for (key,) in db.execute("SELECT event_key FROM fact")} == {event_key(ours.id)}
        for table in ("meta", "names", "episodes", "stable", "access"):
            assert db.execute(
                f"SELECT count(*) FROM {table} WHERE seat = ?", (alice.seat_id,)
            ).fetchone() == (0,)
        assert opened.seats() == (EntityKey("bob"),)
        assert bob.dump() == bob_view


def test_a_peers_record_is_verified_not_trusted() -> None:
    # A fact a peer holds, delivered to another seat with different content, is ConflictingFact; with the same
    # content it is held. Its mutation (M-a-3) inserts the access row first and skips the check when the
    # record exists.
    told = spoke(80, 800, BOB, CAROL, "The café opens at nine.", CAFE)
    forged = spoke(80, 800, BOB, CAROL, "The café is closed.", CAFE)
    with home() as opened:
        carol = opened.seat(
            StoreIdentity(world_instance=INSTANCE, seat=EntityKey("carol"), self_entity=CAROL)
        )
        alice = opened.seat(identity(EntityKey("alice")))
        with opened.transaction():
            carol.hold(told, digest(told), rendering(told))
        before = opened.dump()
        with pytest.raises(ConflictingFact) as raised, opened.transaction():
            alice.hold(forged, digest(forged), rendering(forged))
        assert raised.value.event == "80"
        assert opened.dump() == before
        with opened.transaction():
            alice.hold(told, digest(told), rendering(told))
        assert alice.holds(event_key(told.id))
