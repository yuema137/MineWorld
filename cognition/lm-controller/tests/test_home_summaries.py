"""Summaries never contradict retained detail (step-28 §9.2 C-4 (a) … (c); pr-s28b §6.3, AC-b-6; AS28-18).

The synthetic 12-seat world (seed 7, 30 days) in one shared home, under each bundled policy; after every pass of
every seat, `summary_oracle.check` reads that seat's view against the facts it perceived. `lm-summarise` answers
through FG-c's scripted backend, whose importance is a pure function of the request text, so all three columns of
its decision table occur (a keep with prose, the fallback's merge with prose, a drop or merge without). The oracle
imports nothing of the code under test.
"""

from __future__ import annotations

from collections.abc import Callable

import pytest
from capture_s28b_parity import shared, streams
from home_streams import CAST
from memory_fixtures import all_rows
from mineworld_sdk.wire.ids import EntityKey
from summary_oracle import check
from test_lm_summarise import GENEROUS, gateway, scripted
from test_lm_summarise import retention as assisted

from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.retention.host import PassReport
from mineworld_cognition.memory.retention.selection import Retention

PLAIN: dict[str, dict[str, int]] = {
    "consolidate": {},
    "decay": {"threshold": 1000},
    "importance-recency-cap": {"k_l0": 0, "k_l1": 0, "k_l2": 0, "k_l3": 0},
    "keep-all": {},
}
SEATS = 12
LEAST_PASSES = 20
"""Passes each seat runs at least: the world has 30 days, some of them quiet (`home_streams.world`)."""


def _checked(
    policy: str,
    parameters: dict[str, int],
    policies: Callable[[EntityKey], Retention] | None = None,
) -> tuple[dict[EntityKey, int], list[PassReport], dict[EntityKey, bytes]]:
    perceived = streams()
    checked: dict[EntityKey, int] = {}
    reports: list[PassReport] = []

    def after(seat: EntityKey, view: SeatView, report: PassReport) -> None:
        check(view.dump(), perceived[seat], CAST[seat])
        checked[seat] = checked.get(seat, 0) + 1
        reports.append(report)

    dumps = shared(policy, parameters, on_pass=after, policies=policies)
    for seat, dump in dumps.items():
        check(dump, perceived[seat], CAST[seat])
    return checked, reports, dumps


@pytest.mark.parametrize("policy", sorted(PLAIN))
def test_every_summary_agrees_with_the_facts_under_a_plain_policy(policy: str) -> None:
    # AC-b-6, one bundled no-model policy per case. Its mutations: M-b-10 (an episode's `records` one more than
    # its records) and M-b-12 (an ambient record tests the place) fail (a); M-b-11 (a record left behind when
    # its episode merges into a chapter) fails (b).
    checked, _, _ = _checked(policy, PLAIN[policy])
    assert len(checked) == SEATS
    assert min(checked.values()) >= LEAST_PASSES


def test_every_summary_agrees_with_the_facts_under_lm_summarise() -> None:
    # AC-b-6 under `lm-summarise`, scripted: the three columns of FG-c's table all occur (some prose stored,
    # some range forgotten by the model's drop, some group merged).
    summarise = gateway(scripted(), **GENEROUS)

    def choose(_seat: EntityKey) -> Retention:
        return assisted(summarise, calls_per_sim_day=24)

    checked, reports, dumps = _checked("lm-summarise", {}, choose)
    assert len(checked) == SEATS
    assert min(checked.values()) >= LEAST_PASSES
    assert "assisted" in {report.outcome for report in reports}
    assert sum(report.model_decisions for report in reports) > 0
    rows = [row for dump in dumps.values() for row in all_rows(dump)]
    assert any(row["table"] in ("episodes", "chapters") and row["prose"] for row in rows)
    assert any(
        row["table"] == "forgotten" and row["first_policy"] == "lm-summarise" for row in rows
    )
    assert sum(report.merged for report in reports) > 0
