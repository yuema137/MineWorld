"""Why a seat can or cannot recall an event (pr-s28e-memory-inspect.md §5.3; step-28 §5.6, C-7; `AC-18`).

Pure over one `SeatView`: every statement names this seat (`seat = ?`), so a home-mate's units, ranges or access
rows never decide an answer (I-S28-1). Three functions:

- `held_counts(view)`: the seat's held records by **how** it knows them (from its own `fact_who` row: role 2 or 3
  is `subject`, 1 `participant`, none `witness`; R-S28e-a-1) and salience, with the cursor, the newest key and the
  forgotten set's watermark;
- `covering(view, event)`: the seat's surviving units whose ranges hold the event: episodes (open or closed) and
  chapters (by their stored ranges);
- `explain(view, event, perceived)`: exactly one class (`Explanation.kind`). A unit's range and a forgotten range
  stand for the **perceived** ids between their ends (`records.py`), so the home alone cannot tell an id merged
  into a unit from one this seat never perceived. `perceived` (the seat's own perceived ids, which the caller
  read from `mineworld perceived --json`; memory never opens it) makes the answer exact: an unheld id not on the
  stream is `covered-not-perceived` when a unit's range holds it and `never-perceived` otherwise, and an id on
  the stream that is neither held, merged nor forgotten is `incoherent`, a defect of memory (FGb-1 broken).

No pass is named for a merge: no schema records which pass merged a record into a unit (QS28e-2). A forgotten
range carries its full attribution (S28-b).
"""

from __future__ import annotations

from collections.abc import Set as AbstractSet
from typing import Literal

from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.backend.model import CognitionModel
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import Salience, event_key, event_of
from mineworld_cognition.memory.retention.forgotten import ForgottenRange, ForgottenSet
from mineworld_cognition.memory.scopes.world import ROLE_BOTH, ROLE_PARTICIPANT, ROLE_SUBJECT

How = Literal["subject", "participant", "witness"]
"""How a seat knows a record it holds: named as its subject, took part in it, or perceived it from beside."""
ExplanationKind = Literal[
    "held",
    "merged",
    "forgotten",
    "forgotten-below-watermark",
    "not-yet-delivered",
    "never-perceived",
    "covered-not-perceived",
    "incoherent",
]
HOWS: tuple[How, ...] = ("subject", "participant", "witness")
SALIENCES: tuple[Salience, ...] = ("notable", "ambient")


class HeldCount(CognitionModel):
    how: How
    salience: Salience
    records: int


class HeldCounts(CognitionModel):
    """The seat's held records by how and salience (every combination, zeros included), and its positions."""

    counts: tuple[HeldCount, ...]
    total: int
    cursor: str | None
    newest: str | None
    """The newest ingested fact's id."""
    watermark: str | None


class WhoRow(CognitionModel):
    entity: str
    role: Literal["participant", "subject", "both"]


class Record(CognitionModel):
    """A fact record's perspective-free envelope, as the home holds it once."""

    event: str
    at: int
    kind: str
    place: str | None
    who: tuple[WhoRow, ...]


class AccessRow(CognitionModel):
    """This seat's rendering of a record it holds."""

    gist: str
    salience: Salience
    mine: bool
    importance: int


class Unit(CognitionModel):
    """A surviving unit of this seat whose range holds an id: an episode (L1) or a chapter range (L2)."""

    unit: Literal["episode", "chapter"]
    number: int
    """The episode's number, or the chapter's week."""
    first: str
    last: str
    day: int | None = None
    place: str | None = None
    records: int | None = None
    closed: bool | None = None


class Forgotten(CognitionModel):
    first: str
    last: str
    first_day: int
    first_policy: str
    last_day: int
    last_policy: str
    passes: int
    forced: bool

    @staticmethod
    def of(found: ForgottenRange) -> Forgotten:
        return Forgotten(
            first=found.first,
            last=found.last,
            first_day=found.first_day,
            first_policy=found.first_policy,
            last_day=found.last_day,
            last_policy=found.last_policy,
            passes=found.passes,
            forced=found.forced(),
        )


class Explanation(CognitionModel):
    """One id's class and what supports it. Only the fields of its class are set."""

    event: str
    kind: ExplanationKind
    note: str = ""
    record: Record | None = None
    access: AccessRow | None = None
    how: How | None = None
    holders: int | None = None
    """How many seats of this home hold the record (diagnostic only, never charged)."""
    units: tuple[Unit, ...] = ()
    forgotten: Forgotten | None = None
    watermark: str | None = None
    cursor: str | None = None


_ROLE_NAMES: dict[int, Literal["participant", "subject", "both"]] = {
    ROLE_PARTICIPANT: "participant",
    ROLE_SUBJECT: "subject",
    ROLE_BOTH: "both",
}


def how_of(role: int | None) -> How:
    """R-S28e-a-1: the seat's own `fact_who` role decides how it knows the record."""
    if role in (ROLE_SUBJECT, ROLE_BOTH):
        return "subject"
    if role == ROLE_PARTICIPANT:
        return "participant"
    return "witness"


def held_counts(view: SeatView) -> HeldCounts:
    rows: list[tuple[str, int | None, int]] = view.connection.execute(
        "SELECT a.salience, w.role, count(*) FROM access a LEFT JOIN fact_who w "
        "ON w.event_key = a.event_key AND w.entity = ? WHERE a.seat = ? GROUP BY a.salience, w.role",
        (view.me, view.seat_id),
    ).fetchall()
    tally: dict[tuple[How, str], int] = {}
    for salience, role, count in rows:
        key = (how_of(role), salience)
        tally[key] = tally.get(key, 0) + count
    counts = tuple(
        HeldCount(how=how, salience=salience, records=tally.get((how, salience), 0))
        for how in HOWS
        for salience in SALIENCES
    )
    cursor = view.cursor()
    newest = view.newest()
    watermark = ForgottenSet.load(view).watermark()
    return HeldCounts(
        counts=counts,
        total=sum(count.records for count in counts),
        cursor=cursor,
        newest=None if newest is None else event_of(newest),
        watermark=watermark,
    )


def covering(view: SeatView, event: EventId) -> tuple[Unit, ...]:
    """The seat's surviving units whose ranges hold `event`: episodes by number, then chapters by week."""
    key = event_key(event)
    db = view.connection
    episodes: list[tuple[int, int, str | None, str, str, int, int]] = db.execute(
        "SELECT episode, day, place, first_key, last_key, records, closed FROM episodes "
        "WHERE seat = ? AND ? BETWEEN first_key AND last_key ORDER BY episode",
        (view.seat_id, key),
    ).fetchall()
    chapters: list[tuple[int, str, str]] = db.execute(
        "SELECT week, first_key, last_key FROM chapter_ranges "
        "WHERE seat = ? AND ? BETWEEN first_key AND last_key ORDER BY week, first_key",
        (view.seat_id, key),
    ).fetchall()
    units = [
        Unit(
            unit="episode",
            number=number,
            first=event_of(first),
            last=event_of(last),
            day=day,
            place=place,
            records=records,
            closed=bool(closed),
        )
        for number, day, place, first, last, records, closed in episodes
    ]
    units += [
        Unit(unit="chapter", number=week, first=event_of(first), last=event_of(last))
        for week, first, last in chapters
    ]
    return tuple(units)


def record_of(view: SeatView, event: EventId) -> tuple[Record, AccessRow, How] | None:
    """The record this seat holds under `event`, its access row and how it knows it; `None` when not held."""
    key = event_key(event)
    db = view.connection
    row: tuple[str, int, str, str | None, str, str, int, int] | None = db.execute(
        "SELECT f.event_id, f.at, f.kind, f.place, a.gist, a.salience, a.mine, a.importance "
        "FROM access a JOIN fact f ON f.event_key = a.event_key WHERE a.seat = ? AND a.event_key = ?",
        (view.seat_id, key),
    ).fetchone()
    if row is None:
        return None
    event_id, at, kind, place, gist, salience, mine, importance = row
    who: list[tuple[str, int]] = db.execute(
        "SELECT entity, role FROM fact_who WHERE event_key = ? ORDER BY ordinal", (key,)
    ).fetchall()
    mine_role = next((role for entity, role in who if entity == view.me), None)
    record = Record(
        event=event_id,
        at=at,
        kind=kind,
        place=place,
        who=tuple(WhoRow(entity=entity, role=_ROLE_NAMES[role]) for entity, role in who),
    )
    access = AccessRow(
        gist=gist,
        salience="notable" if salience == "notable" else "ambient",
        mine=bool(mine),
        importance=importance,
    )
    return record, access, how_of(mine_role)


def explain(
    view: SeatView, event: EventId, perceived: AbstractSet[EventId] | None = None
) -> Explanation:
    """Exactly one class for `event` (module docstring; §5.3's table)."""
    held = record_of(view, event)
    units = covering(view, event)
    if held is not None:
        record, access, how = held
        holders: int = view.connection.execute(
            "SELECT count(*) FROM access WHERE event_key = ?", (event_key(event),)
        ).fetchone()[0]
        return Explanation(
            event=event,
            kind="held",
            record=record,
            access=access,
            how=how,
            holders=holders,
            units=units,
        )
    cursor = view.cursor()
    if cursor is None or event_key(event) > event_key(cursor):
        return Explanation(event=event, kind="not-yet-delivered", cursor=cursor)
    on_stream = None if perceived is None else event in perceived
    if on_stream is False:
        if units:
            return Explanation(
                event=event,
                kind="covered-not-perceived",
                units=units,
                note="a unit's range stands for the ids this seat perceived; this one it did not",
            )
        return Explanation(event=event, kind="never-perceived")
    if units:
        note = "" if on_stream else "if this seat perceived it; pass --perceived to confirm"
        return Explanation(event=event, kind="merged", units=units, note=note)
    forgotten = ForgottenSet.load(view)
    found = forgotten.range_of(event)
    if found is not None:
        return Explanation(event=event, kind="forgotten", forgotten=Forgotten.of(found))
    watermark = forgotten.watermark()
    if watermark is not None and event_key(event) <= event_key(watermark):
        return Explanation(
            event=event,
            kind="forgotten-below-watermark",
            watermark=watermark,
            note="no attribution is kept below the watermark (the forgotten set's 256-range cap)",
        )
    if on_stream:
        return Explanation(
            event=event,
            kind="incoherent",
            cursor=cursor,
            note="perceived, delivered, and neither held, merged nor forgotten: a defect of memory",
        )
    return Explanation(event=event, kind="never-perceived", note="pass --perceived to confirm")
