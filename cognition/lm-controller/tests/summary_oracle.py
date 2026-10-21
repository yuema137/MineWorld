"""The C-4 oracle (step-28 §9.2 (a) … (c), as read by pr-s28b F-S28b-8 … F-S28b-10; §6.3): summaries never contradict
the detail that survives them. Test-only; it imports nothing of the code under test.

Its inputs are a seat view's canonical dump, the facts the seat perceived (in order) and the seat's entity. The
dump is parsed with the column order of `ARC-101`'s note (the seat view's tables, written here, not read from
the schema module): ids and keys are numbers in the dump and are compared as numbers here.

The **oracle's own episode sequence** is derived from the facts by D-P4-7's three rules, transcribed from the
text of `compress/episodes.py`'s docstring, never by calling it: a record joins the open episode when it follows
the previous record by at most 1 800 s, falls on the episode's day (`at // 86 400`), and is at the episode's place
— a record with no place, or an `ambient` one, does not test the place; otherwise it closes the open episode and
opens the next, and an episode opened by a placeless record keeps the place of the episode it closes. Its
counterparts are the union of its records', a record's being the fact's participants other than the seat. A
record is `ambient` when it is another person's coming or going (F-P4-2) or walk (QFGb-6): an `arrived`,
`person-entered-place`, `walk-started` or `walk-ended` fact whose `person` is not the seat.

Checks:

- **(a)** every episode in the dump is an episode of the oracle's sequence (same first key) with the same day,
  place, last key, record count and counterparts; every perceived id it spans falls on its day. Every chapter's
  cited ids (the perceived ids in its ranges) fall in its week; its first and last keys are the smallest and
  largest of them and its first and last times theirs.
- **(b)** every retained record lies in a surviving episode — never only in a chapter's ranges or the forgotten
  set — and in the oracle's episode that holds it.
- **(c)** a retained record's day is its episode's; a retained `notable` record with a place has its episode's
  place.
"""

from __future__ import annotations

import json
from bisect import bisect_right
from collections.abc import Sequence
from dataclasses import dataclass, field

from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId

DAY = 86_400
WEEK_DAYS = 7
GAP = 1_800
AMBIENT_KINDS = frozenset({"arrived", "person-entered-place", "walk-started", "walk-ended"})
COLUMNS: dict[str, tuple[str, ...]] = {
    "meta": ("key", "value"),
    "l0": ("event_id", "at", "kind", "place", "salience"),
    "episodes": (
        "episode",
        "day",
        "place",
        "first_key",
        "last_key",
        "opened_at",
        "closed_at",
        "closed",
        "records",
    ),
    "episode_who": ("episode", "entity"),
    "chapters": (
        "week",
        "first_episode",
        "last_episode",
        "text",
        "prose",
        "prose_key",
        "first_key",
        "last_key",
        "first_at",
        "last_at",
    ),
    "chapter_ranges": ("week", "first_key", "last_key"),
}
"""The leading columns read, per table, in the seat view's dump order (`ARC-101`'s note; pr-s28a §5.4)."""

type Row = dict[str, object]


def _number(value: object) -> int:
    assert isinstance(value, int), value
    return value


def parse(dump: bytes) -> dict[str, list[Row]]:
    tables: dict[str, list[Row]] = {}
    for line in dump.decode("utf-8").splitlines():
        cells: list[object] = json.loads(line)
        table = str(cells[0])
        if table in COLUMNS:
            tables.setdefault(table, []).append(dict(zip(COLUMNS[table], cells[1:], strict=False)))
    return tables


@dataclass
class OracleEpisode:
    first: int
    last: int
    day: int
    place: str | None
    records: int = 0
    who: set[str] = field(default_factory=set[str])


def _ambient(fact: PerceivedEvent, me: EntityId) -> bool:
    if fact.event_type not in AMBIENT_KINDS:
        return False
    body = fact.payload.payload
    person = body.get("person") if isinstance(body, dict) else None
    entity = person.get("entity") if isinstance(person, dict) else None
    return isinstance(entity, str) and entity != me


def episodes(facts: Sequence[PerceivedEvent], me: EntityId) -> list[OracleEpisode]:
    """D-P4-7's sequence over `facts` (module docstring)."""
    out: list[OracleEpisode] = []
    last_at = 0
    for fact in facts:
        number = int(fact.id)
        place = None if fact.place is None else str(fact.place.entity)
        current = out[-1] if out else None
        fits = (
            current is not None
            and fact.at - last_at <= GAP
            and fact.at // DAY == current.day
            and (_ambient(fact, me) or place is None or place == current.place)
        )
        if not fits:
            inherited = place if place is not None or current is None else current.place
            current = OracleEpisode(number, number, fact.at // DAY, inherited)
            out.append(current)
        assert current is not None
        current.last = number
        current.records += 1
        current.who |= {str(p) for p in fact.participants if p != me}
        last_at = fact.at
    return out


def check(dump: bytes, perceived: Sequence[PerceivedEvent], me: EntityId) -> None:
    """C-4 (a) … (c) for one seat view, against the facts it perceived up to its `newest`."""
    tables = parse(dump)
    meta = {str(row["key"]): row["value"] for row in tables.get("meta", [])}
    newest = meta.get("newest")
    upto = [f for f in perceived if newest is not None and int(f.id) <= _number(newest)]
    at = {int(f.id): f.at for f in upto}
    ids = sorted(at)
    sequence = {each.first: each for each in episodes(upto, me)}
    who: dict[int, set[str]] = {}
    for row in tables.get("episode_who", []):
        who.setdefault(_number(row["episode"]), set()).add(str(row["entity"]))
    dumped = tables.get("episodes", [])
    for row in dumped:
        first, last = _number(row["first_key"]), _number(row["last_key"])
        derived = sequence.get(first)
        assert derived is not None, (
            f"(a) episode {row['episode']} at {first} is no episode of the facts"
        )
        found = (_number(row["day"]), row["place"], last, _number(row["records"]))
        assert found == (derived.day, derived.place, derived.last, derived.records), (
            f"(a) episode {row['episode']}: {found} against the facts' {derived}"
        )
        assert who.get(_number(row["episode"]), set()) == derived.who, (
            f"(a) counterparts of {first}"
        )
        spanned = ids[bisect_right(ids, first - 1) : bisect_right(ids, last)]
        assert all(at[i] // DAY == derived.day for i in spanned), f"(a) episode {first} spans days"
    ranges: dict[int, list[tuple[int, int]]] = {}
    for row in tables.get("chapter_ranges", []):
        ranges.setdefault(_number(row["week"]), []).append(
            (_number(row["first_key"]), _number(row["last_key"]))
        )
    for row in tables.get("chapters", []):
        week = _number(row["week"])
        cited = [
            i
            for a, b in ranges.get(week, [])
            for i in ids[bisect_right(ids, a - 1) : bisect_right(ids, b)]
        ]
        assert cited, f"(a) chapter {week} cites no perceived id"
        assert all(at[i] // DAY // WEEK_DAYS == week for i in cited), (
            f"(a) chapter {week} spans weeks"
        )
        assert (_number(row["first_key"]), _number(row["last_key"])) == (min(cited), max(cited)), (
            f"(a) chapter {week}'s keys"
        )
        assert (_number(row["first_at"]), _number(row["last_at"])) == (
            at[min(cited)],
            at[max(cited)],
        ), f"(a) chapter {week}'s times"
    spans = sorted((_number(r["first_key"]), _number(r["last_key"]), r) for r in dumped)
    starts = [first for first, _, _ in spans]
    for record in tables.get("l0", []):
        key = _number(record["event_id"])
        index = bisect_right(starts, key) - 1
        assert index >= 0 and spans[index][1] >= key, (
            f"(b) record {key} lies in no surviving episode"
        )
        _, _, episode = spans[index]
        assert _number(record["at"]) // DAY == _number(episode["day"]), f"(c) record {key}'s day"
        place = record["place"]
        if record["salience"] == "notable" and place is not None:
            assert place == episode["place"], f"(c) record {key}'s place"
