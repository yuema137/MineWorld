"""One retention pass (pr-fg-b §5.7): items → policy → validate → execute → bound → check → log.

One transaction. The policy sees frozen items and a budget, never the store (FGb-3); a plan it returns is
validated whole, and a refused plan, or a policy that raised, is replaced by `consolidate`'s defaults and
recorded as `fallback:<reason>`. The bound is the host's (FGb-2). A breach of the store-side coverage
invariant (FGb-1) is a defect: it raises, and the pass rolls back. A pass never writes `cursor` or
`newest` (P4-2) and reads no clock: `now` is fixed by the triggering fact (FGb-5).

A pass has a cause (pr-fg-b2 §4.3): `day`, at the first perceived fact of a new day, or `size`, after the
fact that took the store's content over its trigger. Both are the same pass; only a day pass advances
`meta.retention_day`. The log is keyed by the triggering fact, the newest ingested at the pass.
"""

from __future__ import annotations

import sqlite3
from dataclasses import dataclass
from itertools import pairwise
from typing import Literal

from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.retention.bound import enforce, fallback
from mineworld_cognition.memory.retention.forgotten import normalize
from mineworld_cognition.memory.retention.items import extract
from mineworld_cognition.memory.retention.plan import PlanRejected, Resolved, execute, validate
from mineworld_cognition.memory.retention.policy import RetentionBudget
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.store import MemoryStore

PASS_LOG_ROWS = 32

type PassCause = Literal["day", "size"]


@dataclass(frozen=True)
class PassReport:
    day: int
    policy: str
    outcome: str
    """`ok`, or `fallback:<reason>` when the policy's plan was refused or the policy raised."""
    kept: int
    merged: int
    dropped: int
    forced: int
    """Items the host's bound removed after the plan."""
    bytes_before: int
    bytes_after: int
    """The dump before the pass and after its bound, before the pass's own log row."""
    forgotten_ranges: int
    cause: PassCause = "day"
    """Why the pass ran: the day's first fact, or the store nearing its bound (pr-fg-b2 §4.3)."""
    trigger: str = ""
    """The event key of the newest fact ingested at the pass: the pass log's key."""


class RetentionInvariantBroken(RuntimeError):
    """The store-side half of FGb-1 failed after a pass: a defect, never a fallback."""


def run_pass(
    store: MemoryStore, retention: Retention, now: int, cause: PassCause = "day"
) -> PassReport:
    day = now // DAY_S
    bounds = retention.bounds
    with store.transaction() as db:
        extracted = extract(store)
        before = extracted.total
        budget = RetentionBudget.model_construct(
            store_bytes=max(0, bounds.target - extracted.overhead), now=now, calls_left=0
        )
        outcome = "ok"
        resolved: Resolved | None = None
        try:
            resolved = validate(retention.policy.plan(extracted.items, budget), extracted)
        except PlanRejected as rejected:
            outcome = f"fallback:{rejected.reason}"
        except Exception as error:
            outcome = f"fallback:raised: {type(error).__name__}"
        if resolved is None:
            # A refused plan is never partly applied (§5.7.1). The fallback is trusted host policy: a
            # refusal of its plan is a defect and propagates.
            resolved = validate(fallback().plan(extracted.items, budget), extracted)
        counts = execute(db, resolved, extracted, fts=store.fts)
        normalize(db)
        forced, after = enforce(store, now, bounds)
        check_structure(db)
        (ranges,) = db.execute("SELECT count(*) FROM forgotten").fetchone()
        report = PassReport(
            day,
            retention.policy.id,
            outcome,
            counts.kept,
            counts.merged,
            counts.dropped,
            forced,
            before,
            after,
            ranges,
            cause,
            store.newest() or "",
        )
        _log(db, report)
    store.reclaim()
    return report


def _log(db: sqlite3.Connection, report: PassReport) -> None:
    db.execute(
        "INSERT INTO retention_passes (trigger, day, cause, policy, outcome, kept, merged, dropped, "
        "forced, bytes_before, bytes_after, forgotten_ranges) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            report.trigger,
            report.day,
            report.cause,
            report.policy,
            report.outcome,
            report.kept,
            report.merged,
            report.dropped,
            report.forced,
            report.bytes_before,
            report.bytes_after,
            report.forgotten_ranges,
        ),
    )
    db.execute(
        "DELETE FROM retention_passes WHERE trigger NOT IN "
        "(SELECT trigger FROM retention_passes ORDER BY trigger DESC LIMIT ?)",
        (PASS_LOG_ROWS,),
    )
    if report.cause == "day":
        db.execute("UPDATE meta SET value = ? WHERE key = 'retention_day'", (str(report.day),))


def check_structure(db: sqlite3.Connection) -> None:
    """FGb-1's store-side half: every record lies in an episode; episodes are disjoint; every episode of a
    week with a chapter lies in one of its ranges; no episode, chapter range or record meets the
    forgotten set; the forgotten ranges are disjoint."""
    episodes: list[tuple[str, str, int]] = db.execute(
        "SELECT first_key, last_key, day / 7 FROM episodes ORDER BY first_key"
    ).fetchall()
    _disjoint("episode", [(first, last) for first, last, _ in episodes])
    keys: list[tuple[str]] = db.execute("SELECT event_key FROM l0 ORDER BY event_key").fetchall()
    index = 0
    for (key,) in keys:
        while index < len(episodes) and episodes[index][1] < key:
            index += 1
        if index == len(episodes) or episodes[index][0] > key:
            raise RetentionInvariantBroken(f"record {key} lies in no episode")
    ranges: dict[int, list[tuple[str, str]]] = {}
    for week, first, last in db.execute(
        "SELECT week, first_key, last_key FROM chapter_ranges ORDER BY week, first_key"
    ):
        ranges.setdefault(week, []).append((first, last))
    chaptered: set[int] = {week for (week,) in db.execute("SELECT week FROM chapters")}
    if set(ranges) != chaptered:
        raise RetentionInvariantBroken("a chapter and its ranges disagree")
    for first, last, week in episodes:
        if week in ranges and not any(a <= first and last <= b for a, b in ranges[week]):
            raise RetentionInvariantBroken(
                f"episode {first}…{last} lies outside its week's chapter"
            )
    forgotten: list[tuple[str, str]] = db.execute(
        "SELECT first_key, last_key FROM forgotten ORDER BY first_key"
    ).fetchall()
    _disjoint("forgotten range", forgotten)
    units = sorted(
        [(first, last) for first, last, _ in episodes]
        + [pair for pairs in ranges.values() for pair in pairs]
    )
    _apart(units, forgotten)


def _disjoint(what: str, spans: list[tuple[str, str]]) -> None:
    for (first, last), (after, _) in pairwise(spans):
        if first > last or after <= last:
            raise RetentionInvariantBroken(f"{what}s overlap at {after}")


def _apart(units: list[tuple[str, str]], forgotten: list[tuple[str, str]]) -> None:
    """No unit meets a forgotten range: both lists sorted by first key, walked together."""
    index = 0
    for first, last in forgotten:
        while index < len(units) and units[index][1] < first:
            index += 1
        probe = index
        while probe < len(units) and units[probe][0] <= last:
            if units[probe][1] >= first:
                raise RetentionInvariantBroken(f"a surviving unit meets forgotten {first}…{last}")
            probe += 1
