"""AFGb-2: a policy's plan is validated whole; a refused plan is replaced by `consolidate`'s defaults and
recorded (pr-fg-b §5.7.1). Plus the forgotten set's adjacency merge and watermark (§5.8).

Each hostile policy of `retention_fixtures` runs one pass on a store of four synthetic days, and its dump
is compared with the dump of the same pass under `consolidate` itself: equal but for that pass's `policy`
and `outcome` fields.
"""

from __future__ import annotations

import pytest
from memory_fixtures import all_rows, seat_view
from retention_driver import DAY, feed, retention
from retention_fixtures import HOSTILE, DropEverything
from retention_stream import stream
from support import run

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.forgotten import MAX_RANGES, ForgottenSet, normalize
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import Retention

REASONS = {
    "unknown-item": "fallback:unknown-item",
    "duplicate": "fallback:duplicate",
    "merge-mismatch": "fallback:merge-mismatch",
    "drop-nested": "fallback:drop-nested",
    "keep-under-removed": "fallback:keep-under-removed",
    "raises": "fallback:raised: RuntimeError",
    "returns-dict": "fallback:malformed",
    "mutates": "fallback:raised: ValidationError",
}


def _one_pass(kept: Retention) -> tuple[bytes, str]:
    """Days 0 to 3 under `keep-all` (which removes nothing below the bound), then day 4's first fact by
    plain `ingest` and its pass under `kept`: one pass whose policy alone differs."""
    facts = list(stream(11, 5, per_day=60))
    trigger = next(f for f in facts if f.at // DAY == 4)
    head = facts[: facts.index(trigger)]
    with seat_view() as store:
        feed(store, [head], retention("keep-all"))
        ingest(store, [trigger], trigger.id)
        report = run(absorb(store, [], trigger.id, kept))
        (only,) = report.passes
        return store.dump(), only.outcome


def _without_the_policy(dump: bytes) -> list[dict[str, object]]:
    rows = all_rows(dump)
    for row in rows:
        if row["table"] == "retention_passes" and row["day"] == 4:
            row.pop("policy")
            row.pop("outcome")
    return rows


@pytest.mark.parametrize("policy", sorted(HOSTILE))
def test_a_refused_plan_falls_back_to_consolidate_whole(policy: str) -> None:
    # AFGb-2. Its mutation accepts a `Drop` of a nested item: the drop-nested case is applied, and the
    # store differs from consolidate's.
    expected, outcome = _one_pass(retention("consolidate"))
    assert outcome == "ok"
    hostile, outcome = _one_pass(retention(policy, PolicyRegistry({policy: HOSTILE[policy]})))
    assert outcome == REASONS[policy]
    assert _without_the_policy(hostile) == _without_the_policy(expected)


def test_the_most_aggressive_legal_plan_is_applied_not_refused() -> None:
    dump, outcome = _one_pass(
        retention("drop-everything", PolicyRegistry({"drop-everything": DropEverything}))
    )
    assert outcome == "ok"
    rows = all_rows(dump)
    assert [r for r in rows if r["table"] == "stable"] == []
    assert len([r for r in rows if r["table"] == "episodes"]) == 1  # the open one
    assert [r for r in rows if r["table"] == "forgotten"]


def _key(n: int) -> str:
    return str(n).rjust(20, "0")


def test_forgotten_ranges_with_nothing_between_merge_and_the_oldest_become_a_watermark() -> None:
    # §5.8, on a hand-built store: 300 forgotten ranges, each pair separated by a surviving episode,
    # except ranges 10 and 11, which nothing separates. Its mutation merges every adjacent pair: the
    # ranges separated by an episode collapse too.
    with seat_view() as store:
        db = store.connection
        seat = store.seat_id
        for n in range(300):
            db.execute(
                "INSERT INTO forgotten VALUES (?, ?, ?)",
                (seat, _key(1000 * n), _key(1000 * n + 10)),
            )
            if n != 10:
                db.execute(
                    "INSERT INTO episodes (seat, episode, day, place, first_key, last_key, "
                    "opened_at, closed_at, closed, records, text, prose, prose_key, importance, mine) "
                    "VALUES (?, ?, 0, NULL, ?, ?, 0, 0, 1, 1, '', NULL, NULL, 0, 0)",
                    (seat, n + 1, _key(1000 * n + 500), _key(1000 * n + 600)),
                )
        normalize(store)
        forgotten = ForgottenSet.load(store)
    ranges = [(r.first, r.last) for r in forgotten.ranges()]
    # 300 ranges, one merge (10 with 11): 299; capped at 256 by removing the 43 oldest, which are
    # ranges 0 to 9, the merged 10 and 11, and 12 to 43.
    assert len(ranges) == MAX_RANGES
    assert forgotten.watermark() == "43010"
    assert ranges[0] == ("44000", "44010")
    assert ("10000", "11010") not in ranges  # merged, then below the watermark
    assert ranges[-1] == (str(299 * 1000), str(299 * 1000 + 10))
