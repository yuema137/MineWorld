"""The `world` scope's records (step-28 §§4.2, 6.4; pr-s28a-memory-home.md §5.6): each perceived fact's
perspective-free envelope, stored once per home and keyed by its `EventKey`.

- **Insert-or-verify.** The first holder inserts the record; every later holder compares the full sha256 of the
  canonical fact with the stored one: equal is a no-op, different is `ConflictingFact`. A record a peer holds is
  verified, never trusted.
- **Immutable.** A record is never updated, only inserted and collected.
- **Collected exactly.** The holders of a record are the `access` rows that cite it; there is no stored count. A
  release deletes the records of its keys that no access row cites any more, in the releasing transaction.

Only `memory.home` calls these functions, on the home's connection, inside its transaction.

`WorldScope` is the scope's entry in the registry (pr-s28c §5.4): a base scope with no parameters and no
recall section, never written in a stack (`ScopeNotSelectable`); its placement is `cognition.toml`'s.
"""

from __future__ import annotations

import sqlite3
from collections.abc import Iterable, Mapping
from typing import ClassVar

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.memory.records import EventKey, event_key
from mineworld_cognition.memory.scopes.interface import (
    FactRecord,
    Reading,
    RecallQuery,
    ScopeBytes,
    ScopeParameters,
    ScopeRow,
    ScopeView,
    SectionBudget,
    SectionLine,
    SectionSpec,
    TableSpec,
)

PREFIX = "memory.scopes.world:"
ROLE_PARTICIPANT = 1
ROLE_SUBJECT = 2
ROLE_BOTH = 3


class ConflictingFact(ValueError):
    """A fact delivered again, by this seat or by a peer, differs from the record held under its id."""

    def __init__(self, event: EventId) -> None:
        super().__init__(f"{PREFIX} fact {event} was delivered again with different content")
        self.event = event


def who(fact: PerceivedEvent) -> list[tuple[EntityId, int]]:
    """The record's `fact_who` rows in ordinal order: the participants, deduplicated in envelope order, then the
    subjects that are not participants, in envelope order; each with its role (QS28a-3)."""
    subjects = set(fact.subjects)
    rows = [
        (entity, ROLE_BOTH if entity in subjects else ROLE_PARTICIPANT)
        for entity in dict.fromkeys(fact.participants)
    ]
    named = {entity for entity, _ in rows}
    rows += [
        (entity, ROLE_SUBJECT) for entity in dict.fromkeys(fact.subjects) if entity not in named
    ]
    return rows


def insert_or_verify(db: sqlite3.Connection, fact: PerceivedEvent, digest: str) -> bool:
    """The fact's record, inserted if absent, verified if present. Returns whether it was inserted."""
    key = event_key(fact.id)
    stored = db.execute("SELECT digest FROM fact WHERE event_key = ?", (key,)).fetchone()
    if stored is not None:
        if stored[0] != digest:
            raise ConflictingFact(fact.id)
        return False
    place = None if fact.place is None else fact.place.entity
    db.execute(
        "INSERT INTO fact (event_key, event_id, at, kind, place, digest) VALUES (?, ?, ?, ?, ?, ?)",
        (key, fact.id, fact.at, fact.event_type, place, digest),
    )
    db.executemany(
        "INSERT INTO fact_who (event_key, ordinal, entity, role) VALUES (?, ?, ?, ?)",
        [(key, ordinal, entity, role) for ordinal, (entity, role) in enumerate(who(fact))],
    )
    return True


def collect(db: sqlite3.Connection, keys: Iterable[EventKey]) -> None:
    """Deletes the records of `keys` that no access row cites (I-S28-4), in the caller's transaction."""
    rows = [(key,) for key in keys]
    for table in ("fact_who", "fact"):
        db.executemany(
            f"DELETE FROM {table} WHERE event_key = ? AND NOT EXISTS "
            f"(SELECT 1 FROM access WHERE access.event_key = {table}.event_key)",
            rows,
        )


def check_holders(db: sqlite3.Connection) -> list[str]:
    """I-S28-4's two anti-joins: records no access row cites, held keys with no record, and `fact_who` rows
    with no record. Empty when the home is sound."""
    found: list[tuple[str]] = db.execute(
        "SELECT event_key FROM fact f WHERE NOT EXISTS "
        "(SELECT 1 FROM access a WHERE a.event_key = f.event_key) "
        "UNION SELECT DISTINCT event_key FROM access a WHERE NOT EXISTS "
        "(SELECT 1 FROM fact f WHERE f.event_key = a.event_key) "
        "UNION SELECT DISTINCT event_key FROM fact_who w WHERE NOT EXISTS "
        "(SELECT 1 FROM fact f WHERE f.event_key = w.event_key) ORDER BY 1"
    ).fetchall()
    return [key for (key,) in found]


class WorldParameters(ScopeParameters):
    """None: the world scope's one setting, `sharing`, is the operator's (`cognition.toml`, S28-f)."""


class WorldScope:
    """The shared fact records as a scope: no section, no rows of its own, no recall. Its bytes are the home's
    `fact` and `fact_who` lines, reported as home totals, not per seat (S28-e)."""

    id: ClassVar[str] = "world"
    Parameters = WorldParameters
    tables: ClassVar[tuple[TableSpec, ...]] = ()

    def sections(self, params: WorldParameters) -> tuple[SectionSpec, ...]:
        return ()

    def index(
        self, fact: FactRecord, reading: Reading, params: WorldParameters
    ) -> tuple[ScopeRow, ...]:
        return ()

    def recall(
        self,
        view: ScopeView,
        query: RecallQuery,
        budgets: Mapping[str, SectionBudget],
        params: WorldParameters,
    ) -> tuple[SectionLine, ...]:
        return ()

    def account(self, view: ScopeView) -> ScopeBytes:
        return ScopeBytes(tables=())
