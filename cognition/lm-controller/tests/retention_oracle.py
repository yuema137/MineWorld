"""AFGb-3's coverage oracle (pr-fg-b §9): from a store's canonical dump and the perceived ids alone.

It parses the dump's JSON lines and never calls the code under test. Keys are the store's 20-digit event
keys, made here by padding, as the dump stores them.
"""

from __future__ import annotations

import json
from bisect import bisect_right
from collections.abc import Sequence
from dataclasses import dataclass
from itertools import pairwise

from mineworld_sdk.wire.ids import EventId


def key(event: str) -> str:
    return event.rjust(20, "0")


@dataclass(frozen=True)
class Dump:
    tables: dict[str, list[dict[str, object]]]

    @staticmethod
    def parse(dump: bytes) -> Dump:
        tables: dict[str, list[dict[str, object]]] = {}
        for line in dump.decode("utf-8").splitlines():
            row = json.loads(line)
            tables.setdefault(str(row["table"]), []).append(row)
        return Dump(tables)

    def rows(self, table: str) -> list[dict[str, object]]:
        return self.tables.get(table, [])

    def meta(self, name: str) -> str:
        (value,) = [str(row["value"]) for row in self.rows("meta") if row["key"] == name]
        return value


def _span(row: dict[str, object]) -> tuple[str, str]:
    return str(row["first_key"]), str(row["last_key"])


def check_coverage(dump: bytes, perceived: Sequence[EventId]) -> None:
    """(a) every perceived id up to the newest is in exactly one top-level unit or the forgotten set;
    (b) records lie in surviving episodes, and episodes of a chaptered week in its ranges; (c) no record,
    episode or chapter range meets a forgotten range; (d) every forgotten range's ends were perceived."""
    store = Dump.parse(dump)
    newest = store.meta("newest")
    keys = [key(event) for event in perceived]
    known = set(keys)
    upto = [k for k in keys if k <= newest]
    weeks = {int(str(row["week"])) for row in store.rows("chapters")}
    ranges = [_span(row) for row in store.rows("chapter_ranges")]
    episodes = store.rows("episodes")
    chapterless = [_span(e) for e in episodes if int(str(e["day"])) // 7 not in weeks]
    forgotten = [_span(row) for row in store.rows("forgotten")]
    watermark = store.meta("forgotten_below")
    intervals = sorted(ranges + chapterless + forgotten)
    for (_, last), (after, _) in pairwise(intervals):
        assert after > last, f"(a) two representatives overlap at {after}"
    starts = [first for first, _ in intervals]
    for k in upto:
        at = bisect_right(starts, k) - 1
        inside = at >= 0 and intervals[at][0] <= k <= intervals[at][1]
        assert inside or (watermark != "" and k <= watermark), f"(a) perceived {k} is unrepresented"
    for first, last in intervals:
        assert first in known and last in known, f"(a)/(d) {first}…{last} was not perceived"
    spans = sorted(_span(e) for e in episodes)
    starts = [first for first, _ in spans]
    for row in store.rows("l0"):
        k = str(row["event_key"])
        at = bisect_right(starts, k) - 1
        assert at >= 0 and spans[at][0] <= k <= spans[at][1], f"(b) record {k} lies in no episode"
    by_week: dict[int, list[tuple[str, str]]] = {}
    for row in store.rows("chapter_ranges"):
        by_week.setdefault(int(str(row["week"])), []).append(_span(row))
    for e in episodes:
        week = int(str(e["day"])) // 7
        first, last = _span(e)
        if week in by_week:
            assert any(a <= first and last <= b for a, b in by_week[week]), f"(b) episode {first}"
    lost = sorted(forgotten)
    lows = [low for low, _ in lost]
    records = [(str(r["event_key"]),) * 2 for r in store.rows("l0")]
    for first, last in spans + ranges + records:
        # Forgotten ranges are disjoint and sorted, so the last one starting at or before `last` is the
        # only one that can reach back to `first`.
        at = bisect_right(lows, last) - 1
        assert at < 0 or lost[at][1] < first, f"(c) {first}…{last} meets forgotten {lost[at]}"
