"""The forgotten set (pr-fg-b §5.8; `CORE_CONCEPTS.md` §5.3): what a seat no longer holds, as sorted,
disjoint ranges over its perceived sequence, plus a watermark; each range says who forgot it (pr-s28b §5.2,
step-28 §5.6).

Two ranges with no surviving unit between them merge: by coverage (FGb-1), every perceived id between them
is forgotten already. Beyond `MAX_RANGES` ranges the oldest are removed and the watermark rises to their
end: "every perceived id at or below the watermark that no surviving unit represents is forgotten". The
set is therefore bounded by construction. The facts themselves stay in the save and resolve through
`mineworld perceived` (FGb-6); this is how a tool says "Alice has forgotten 12 Oct 08:00 to 14:00".

**Attribution.** A range added by a pass carries the pass's day and the policy that dropped the unit: the
pass's policy id, or `host:<step>` for the host bound's backstop (`Forgetter`). When ranges merge, the merged
range keeps the earliest part's first attribution and the latest part's last, and `passes` adds the parts'
counts; one pass adds exactly 1 to a range it changes however often `normalize` runs in it (`PassScope`). So
`passes` is an upper bound on the distinct passes that touched the range: exact unless one pass added two
ranges a later pass joined (pr-s28b F-S28b-2). A range the cap removes takes its attribution with it; the
watermark carries none. Attribution never changes which ids are forgotten.
"""

from __future__ import annotations

from dataclasses import dataclass, field

from mineworld_sdk.wire.ids import EventId

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.records import EventRange, event_key, event_of

MAX_RANGES = 256
HOST = "host:"
"""The prefix of a policy value the host bound's backstop signs (`host:consolidate`, `host:chapters`,
`host:chapterless-episodes`)."""

_COLUMNS = "first_key, last_key, first_day, first_policy, last_day, last_policy, passes"

type _Row = tuple[str, str, int, str, int, str, int]


@dataclass(frozen=True)
class Forgetter:
    """Who adds a range: the pass's simulated day (`now // DAY_S`) and its policy id, or `host:<step>`."""

    day: int
    policy: str


@dataclass
class PassScope:
    """One pass's view of the set, built at its start and owned by the pass (never stored): the first key of
    every range the seat held when the pass began, and of every range the pass's `normalize` calls rewrote.
    A range is **touched** by the pass when it was not held at the start or was rewritten: its `passes`
    already counts this pass."""

    old: frozenset[str]
    rewritten: set[str] = field(default_factory=set[str])

    @staticmethod
    def begin(store: SeatView) -> PassScope:
        rows: list[tuple[str]] = store.connection.execute(
            "SELECT first_key FROM forgotten WHERE seat = ?", (store.seat_id,)
        ).fetchall()
        return PassScope(frozenset(first for (first,) in rows))

    def touched(self, first_key: str) -> bool:
        return first_key not in self.old or first_key in self.rewritten


@dataclass(frozen=True)
class ForgottenRange:
    """One explicit range and who forgot it (step-28 §5.6)."""

    first: EventId
    last: EventId
    first_day: int
    first_policy: str
    last_day: int
    last_policy: str
    passes: int
    """An upper bound on the distinct passes that added to this range (module docstring)."""

    def forced(self) -> bool:
        """Whether the host bound, not a policy, forgot part of it."""
        return self.first_policy.startswith(HOST) or self.last_policy.startswith(HOST)

    def holds(self, event: EventId) -> bool:
        return event_key(self.first) <= event_key(event) <= event_key(self.last)


@dataclass(frozen=True)
class ForgottenSet:
    _ranges: tuple[ForgottenRange, ...]
    _watermark: EventId | None

    @staticmethod
    def load(store: SeatView) -> ForgottenSet:
        rows: list[_Row] = store.connection.execute(
            f"SELECT {_COLUMNS} FROM forgotten WHERE seat = ? ORDER BY first_key",
            (store.seat_id,),
        ).fetchall()
        watermark = store.meta("forgotten_below")
        return ForgottenSet(
            tuple(ForgottenRange(event_of(row[0]), event_of(row[1]), *row[2:]) for row in rows),
            event_of(watermark) if watermark else None,
        )

    def ranges(self) -> tuple[EventRange, ...]:
        return tuple(EventRange(first=r.first, last=r.last) for r in self._ranges)

    def attributed(self) -> tuple[ForgottenRange, ...]:
        """The ranges with their attribution, in the order of `ranges()`."""
        return self._ranges

    def range_of(self, event: EventId) -> ForgottenRange | None:
        """The explicit range holding `event`, or `None` (at or below the watermark, ask `watermark`)."""
        return next((r for r in self._ranges if r.holds(event)), None)

    def watermark(self) -> EventId | None:
        return self._watermark

    def contains(self, event: EventId) -> bool:
        """Whether `event` lies in a forgotten range. At or below the watermark, an id is forgotten
        unless a surviving unit represents it, which only the store can say; `contains` answers for the
        explicit ranges and reports the watermark separately."""
        return self.range_of(event) is not None


def add(store: SeatView, ranges: list[tuple[str, str]], forgetter: Forgetter) -> None:
    """New ranges, each first added by `forgetter` (one pass)."""
    day, policy = forgetter.day, forgetter.policy
    store.connection.executemany(
        f"INSERT INTO forgotten (seat, {_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, 1)",
        [(store.seat_id, first, last, day, policy, day, policy) for first, last in ranges],
    )


def normalize(store: SeatView, scope: PassScope) -> None:
    """Merges ranges no surviving unit separates, then caps the set at `MAX_RANGES` with the watermark."""
    db = store.connection
    seat = store.seat_id
    rows: list[_Row] = db.execute(
        f"SELECT {_COLUMNS} FROM forgotten WHERE seat = ? ORDER BY first_key", (seat,)
    ).fetchall()
    runs: list[list[_Row]] = []
    for row in rows:
        if runs and not _unit_between(store, runs[-1][-1][1], row[0]):
            runs[-1].append(row)
        else:
            runs.append([row])
    merged = [_merged(run, scope) for run in runs]
    watermark = store.meta("forgotten_below")
    while len(merged) > MAX_RANGES:
        oldest = merged.pop(0)
        watermark = max(watermark, oldest[1])
    if merged != rows:
        db.execute("DELETE FROM forgotten WHERE seat = ?", (seat,))
        db.executemany(
            f"INSERT INTO forgotten (seat, {_COLUMNS}) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            [(seat, *row) for row in merged],
        )
    store.set_meta("forgotten_below", watermark)


def _merged(run: list[_Row], scope: PassScope) -> _Row:
    """One range from a run of parts that merge (§5.2). A single part is left as it is."""
    if len(run) == 1:
        return run[0]
    first = min(run, key=lambda part: (part[2], part[0]))
    last = max(run, key=lambda part: (part[4], part[0]))
    passes = sum(part[6] - 1 if scope.touched(part[0]) else part[6] for part in run) + 1
    scope.rewritten.add(run[0][0])
    return (run[0][0], run[-1][1], first[2], first[3], last[4], last[5], passes)


def _unit_between(store: SeatView, after: str, before: str) -> bool:
    """Whether a surviving unit of the seat (an episode, open or closed, or a chapter range) starts strictly
    between two forgotten ranges. Units never overlap the forgotten set, so one between them starts there."""
    seat = store.seat_id
    found = store.connection.execute(
        "SELECT 1 FROM episodes WHERE seat = ? AND first_key > ? AND first_key < ? "
        "UNION ALL SELECT 1 FROM chapter_ranges WHERE seat = ? AND first_key > ? AND first_key < ? "
        "LIMIT 1",
        (seat, after, before, seat, after, before),
    ).fetchone()
    return found is not None
