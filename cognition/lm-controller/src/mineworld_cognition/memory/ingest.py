"""Ingestion: perceived facts → L0 records, the roll-up and the cursor, in one transaction (§5.4).

Input is the seat's perceived facts only (D-P4-5), never `Observation.events`, in ascending id order,
with the frame's `through`. A fact the store already holds is a re-delivery: identical content is a
no-op, different content is `ConflictingFact`. A fact the store does not hold, at or below what it has
already ingested or acknowledged, is `OutOfOrder`. Everything one call writes — records, episodes,
chapters, stable facts, names and the cursor — commits together or not at all, so the cursor is never
ahead of what is durable (P4-2).
"""

from __future__ import annotations

import hashlib
import json
from collections.abc import Sequence
from dataclasses import dataclass

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.compress.chapters import close_weeks
from mineworld_cognition.compress.episodes import Episodes
from mineworld_cognition.compress.stable import Stable
from mineworld_cognition.compress.summarize import StructuralSummarizer
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import L0Record, event_key, event_of
from mineworld_cognition.memory.render import Rendered, render
from mineworld_cognition.memory.store import MemoryStore


class ConflictingFact(ValueError):
    """A re-delivered fact differs from the one ingested under its id."""

    def __init__(self, event: EventId) -> None:
        super().__init__(f"fact {event} was delivered again with different content")
        self.event = event


class OutOfOrder(ValueError):
    """A fact arrived below what the store has already ingested or acknowledged."""

    def __init__(self, event: EventId, after: EventId) -> None:
        super().__init__(f"fact {event} arrived after fact {after}: perceived facts ascend")
        self.event = event


@dataclass(frozen=True)
class IngestReport:
    records: int
    redeliveries: int
    decode_fallbacks: int
    closed_episodes: int
    chapters: int


def digest(fact: PerceivedEvent) -> str:
    """sha256 of the fact's canonical JSON: what a re-delivery must match."""
    canonical = json.dumps(
        fact.model_dump(mode="json"), sort_keys=True, ensure_ascii=False, separators=(",", ":")
    )
    return hashlib.sha256(canonical.encode("utf-8")).hexdigest()


def ingest(store: MemoryStore, facts: Sequence[PerceivedEvent], through: EventId) -> IngestReport:
    me = store.me
    records = redeliveries = fallbacks = 0
    with store.transaction() as db:
        names = NameBook.load(store)
        stable = Stable(store)
        summarizer = StructuralSummarizer()
        episodes = Episodes(store, names, summarizer, stable)
        cursor = store.cursor()
        bound = max(store.newest() or "", "" if cursor is None else event_key(cursor))
        for fact in facts:
            key = event_key(fact.id)
            fingerprint = digest(fact)
            held = db.execute("SELECT digest FROM l0 WHERE event_key = ?", (key,)).fetchone()
            if held is not None:
                if held[0] != fingerprint:
                    raise ConflictingFact(fact.id)
                redeliveries += 1
                continue
            if key <= bound:
                raise OutOfOrder(fact.id, event_of(bound))
            rendered = render(fact, me, names)
            for entity, name in rendered.names:
                names.learn(entity, name, fact.id)
            record = _write(store, fact, rendered, fingerprint)
            episodes.feed(record)
            stable.feed(record, involved=me in fact.participants, level=rendered.level)
            records += 1
            fallbacks += int(rendered.fallback)
            bound = key
        through_key = event_key(through)
        if through_key < bound and records:
            raise OutOfOrder(event_of(bound), through)
        written = close_weeks(store, names, summarizer)
        if cursor is None or through_key > event_key(cursor):
            store.set_meta("cursor", through)
        if records:
            store.set_meta("newest", bound)
    return IngestReport(records, redeliveries, fallbacks, episodes.closed, written)


def _write(store: MemoryStore, fact: PerceivedEvent, rendered: Rendered, digest: str) -> L0Record:
    """The L0 row, its counterparts, and (for a notable record) its lexical index entry."""
    db = store.connection
    key = event_key(fact.id)
    place = None if fact.place is None else fact.place.entity
    mine = int(rendered.mine)
    row = db.execute(
        "INSERT INTO l0 (event_key, event_id, at, kind, place, salience, mine, gist, digest) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            key,
            fact.id,
            fact.at,
            fact.event_type,
            place,
            rendered.salience,
            mine,
            rendered.gist,
            digest,
        ),
    )
    db.executemany(
        "INSERT INTO l0_who (event_key, entity) VALUES (?, ?)",
        [(key, entity) for entity in rendered.counterparts],
    )
    if store.fts and rendered.salience == "notable":
        db.execute(
            "INSERT INTO l0_text (rowid, gist) VALUES (?, ?)", (row.lastrowid, rendered.gist)
        )
    return L0Record.model_construct(
        event=fact.id,
        at=fact.at,
        kind=fact.event_type,
        place=place,
        salience=rendered.salience,
        mine=rendered.mine,
        gist=rendered.gist,
        counterparts=rendered.counterparts,
    )
