"""The bundled policies (pr-fg-b §5.9): AFGb-8 (their semantics, on hand-built items against literals),
AFGb-4 (a) and AFGb-5 (a), (d) for each over the synthetic stream, AFGb-7 (a), (b) on each policy's
300-day store.

The tie-break literals of AFGb-8 were computed with `shasum -a 256` and recorded in the design's §15.6
before this file existed.
"""

from __future__ import annotations

import time

import pytest
from memory_fixtures import alice_identity
from retention_driver import DAY, built, by_day, retention
from retention_oracle import check_coverage, check_section
from retention_stream import stream

from mineworld_cognition.forgetting.consolidate import Consolidate, ConsolidateParameters
from mineworld_cognition.forgetting.decay import Decay, DecayParameters
from mineworld_cognition.forgetting.importance_recency_cap import (
    ImportanceRecencyCap,
    ImportanceRecencyCapParameters,
)
from mineworld_cognition.memory.retention import (
    Drop,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
    half_life_permille,
    hyperbolic_permille,
)
from mineworld_cognition.memory.retention.items import extract
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve
from mineworld_cognition.memory.store import MemoryStore

HOUR = 3_600
WEEK = 604_800
YEAR = 31_536_000
NOW = 400 * DAY
MIB = 1_048_576
STORE_BOUND = 20 * MIB  # the default per-person bound (pr-fg-b2 §4.1; operator, 2026-10-14)
TARGET = STORE_BOUND * 3 // 4  # 15 MiB: the dump after every pass


def item(
    name: str,
    level: int,
    *,
    container: str | None = None,
    last_at: int = NOW - DAY,
    importance: int = 500,
    first: str = "1000",
    last: str | None = None,
    nbytes: int = 100,
) -> MemoryItem:
    return MemoryItem.model_validate(
        {
            "item": name,
            "level": level,
            "container": container,
            "first_at": last_at,
            "last_at": last_at,
            "day": max(0, last_at) // DAY,
            "kind": ("fished", "episode", "chapter", "stable")[level],
            "counterparts": (),
            "place": None,
            "mine": False,
            "importance": importance,
            "citation": {"ranges": ({"first": first, "last": last or first},)},
            "text": "",
            "bytes": nbytes,
        }
    )


def budget(store_bytes: int = 10**9) -> RetentionBudget:
    return RetentionBudget(store_bytes=store_bytes, now=NOW, calls_left=0)


def _named(plan: object) -> dict[str, str]:
    """Item → what the plan does to it: `keep`, `drop`, `merge1`, `merge2`."""
    assert isinstance(plan, list | tuple)
    fates: dict[str, str] = {}
    for decision in plan:  # pyright: ignore[reportUnknownVariableType]
        if isinstance(decision, Merge):
            fates.update(dict.fromkeys(decision.items, f"merge{decision.into}"))
        elif isinstance(decision, Drop):
            fates[decision.item] = "drop"
        elif isinstance(decision, Keep):
            fates[decision.item] = "keep"
    return fates


def test_consolidate_merges_and_drops_exactly_past_its_ages() -> None:
    # AFGb-8. Kept at the boundary, merged or dropped one second past it. Its mutation uses `<=`: the
    # boundary item is merged.
    items = [
        item("l1:1", 1, container="l2:0", last_at=NOW - 8 * WEEK, first="10", last="19"),
        item("l1:2", 1, container="l2:0", last_at=NOW - 8 * WEEK - 1, first="20", last="29"),
        item("l0:a", 0, container="l1:3", last_at=NOW - 2 * HOUR, first="31"),
        item("l0:b", 0, container="l1:3", last_at=NOW - 2 * HOUR - 1, first="32"),
        item("l1:3", 1, last_at=NOW - 2 * HOUR, first="31", last="32"),
        item("l2:0", 2, last_at=NOW - 5 * YEAR, first="10", last="29"),
        item("l2:1", 2, last_at=NOW - 5 * YEAR - 1, first="40", last="49"),
        item("l2:2", 2, last_at=NOW - 5 * YEAR - 1, first="50", last="59"),
        item("l3:7", 3, first="55"),  # established inside l2:2: that chapter stays
    ]
    params = ConsolidateParameters(l0_hours=2, l1_weeks=8, l2_years=5)
    fates = _named(Consolidate().plan(items, budget(), params))
    assert fates == {"l1:2": "merge2", "l0:b": "merge1", "l2:1": "drop"}


def test_the_forgetting_curves_are_integer_literals() -> None:
    hour = HOUR
    assert [half_life_permille(age, hour) for age in (0, hour // 2, hour, 2 * hour, 10 * hour)] == [
        1000,
        750,
        500,
        250,
        0,
    ]
    assert [hyperbolic_permille(age, hour) for age in (0, hour, 3 * hour)] == [1000, 500, 250]


@pytest.mark.parametrize(("curve", "threshold", "fate"), [
    ("half-life", 300, None), ("half-life", 301, "merge1"),
    ("hyperbolic", 300, None), ("hyperbolic", 301, "merge1"),
])  # fmt: skip
def test_decay_keeps_an_item_while_its_weighted_importance_reaches_the_threshold(
    curve: str, threshold: int, fate: str | None
) -> None:
    # AFGb-8: one half-life (or one scale) of age leaves 500 per mille; importance 600 x 500 // 1000 =
    # 300: kept at 300, merged at 301. Its mutation uses `>` for `>=`: the first case merges.
    age = 168 * HOUR if curve == "half-life" else 72 * HOUR
    items = [
        item("l1:1", 1, last_at=NOW - DAY, importance=1000, first="1", last="2"),
        item("l0:x", 0, container="l1:1", last_at=NOW - age, importance=600, first="1"),
        item("l0:y", 0, container="l1:1", last_at=NOW - DAY, importance=1000, first="2"),
    ]
    params = DecayParameters.model_validate({"curve": curve, "threshold": threshold})
    assert _named(Decay().plan(items, budget(), params)).get("l0:x") == fate


def test_importance_recency_cap_breaks_ties_by_the_recorded_digests() -> None:
    # AFGb-8: five equal top-level episodes, k_l1 = 3: chosen in the order of §15.6's digests (seed 0:
    # 1001, 1005, 1004, 1002, 1003); seed 42 over three: 1001, 1003, 1002. Unchosen top-level units drop.
    five = [item(f"l1:{n}", 1, first=f"100{n}", last=f"100{n}") for n in range(1, 6)]
    params = ImportanceRecencyCapParameters(k_l1=3)
    assert _named(ImportanceRecencyCap().plan(five, budget(), params)) == {
        "l1:2": "drop",
        "l1:3": "drop",
    }
    three = five[:3]
    seeded = ImportanceRecencyCapParameters(k_l1=2, seed=42)
    assert _named(ImportanceRecencyCap().plan(three, budget(), seeded)) == {"l1:2": "drop"}


def test_importance_recency_cap_with_no_room_for_episodes_merges_them_into_their_chapter() -> None:
    # AFGb-8: k_l1 = 0.
    items = [
        item("l2:0", 2, first="1", last="9"),
        *(item(f"l1:{n}", 1, container="l2:0", first=str(n)) for n in (1, 2, 3)),
    ]
    plan = ImportanceRecencyCap().plan(items, budget(), ImportanceRecencyCapParameters(k_l1=0))
    assert list(plan) == [Merge(items=(ItemId("l1:1"), ItemId("l1:2"), ItemId("l1:3")), into=2)]


def test_a_memory_item_cannot_be_changed_by_a_policy() -> None:
    # AFGb-9 (c): the host passes a tuple of frozen items.
    shown = item("l1:1", 1)
    with pytest.raises(Exception, match="frozen"):
        shown.importance = 0  # pyright: ignore[reportAttributeAccessIssue]


POLICIES = ["consolidate", "decay", "importance-recency-cap"]


@pytest.mark.parametrize("policy", [*POLICIES, "keep-all"])
@pytest.mark.parametrize("size", [1, 256])
def test_frames_never_change_any_policys_store(policy: str, size: int) -> None:
    # AFGb-5 (a) for every bundled policy (synthetic): frames of 1 and of 256 against the whole stream.
    facts = list(stream(3, 24, per_day=120))
    assert built(facts, retention(policy), size) == built(facts, retention(policy), len(facts))


@pytest.mark.parametrize("policy", POLICIES)
def test_a_policy_decides_the_same_twice(policy: str) -> None:
    # AFGb-5 (d): `plan` twice on the items of one store. Its mutation reads a wall clock in decay.
    facts = list(stream(5, 20, per_day=120))
    with MemoryStore.open(":memory:", alice_identity()) as store:
        for _ in by_day(store, facts, retention("keep-all")):
            pass
        items = extract(store).items
    kept = retention(policy)
    shown = RetentionBudget(store_bytes=200_000, now=20 * DAY, calls_left=0)
    first = kept.policy.plan(items, shown)
    assert first == kept.policy.plan(items, shown)
    assert _named(first)  # it decided something: the comparison is not of two empty plans


@pytest.mark.parametrize("policy", POLICIES)
def test_each_policy_holds_the_bound_and_retrieves_within_its_ceiling(policy: str) -> None:
    # AFGb-4 (a) for each bundled policy at every pass of 302 synthetic days (days 30, 100, 300
    # reported), AFGb-3 at the checkpoints, AFGb-7 (a), (b) on the day-300 store.
    facts = list(stream(7, 302))
    perceived = [fact.id for fact in facts]
    started = time.monotonic()
    sizes: dict[int, int] = {}
    with MemoryStore.open(":memory:", alice_identity()) as store:
        for each in by_day(store, facts, retention(policy)):
            size = len(each.dump)
            assert size <= TARGET and size <= STORE_BOUND, (each.day, size)
            for checkpoint in (30, 100, 300):
                if each.day >= checkpoint and checkpoint not in sizes:
                    sizes[checkpoint] = size
                    check_coverage(each.dump, perceived)
        wall = time.monotonic() - started
        section = retrieve(store, RecallQuery(at=facts[-1].at, words="umbrella zeppelin rain"))
        dump = store.dump()
    print(f"\n[{policy}] dump at days 30/100/300: {sizes}; {wall:.1f} s")
    check_section(section, dump)
