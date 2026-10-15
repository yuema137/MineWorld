"""The retention host over the synthetic stream (pr-fg-b §5.6 to §5.8): AFGb-3 (coverage), AFGb-4 (the bound,
host half), AFGb-5 (determinism), AFGb-6 (a) (the cursor), AFGb-1 (b), (d) (a user's policy run like a
bundled one; `consolidate`'s parameters refused).

Every oracle reads the dump (`retention_oracle`) or the generator's own ids; bounds are §5.7's literals,
written here.
"""

from __future__ import annotations

from itertools import pairwise
from pathlib import Path

import pytest
from memory_fixtures import alice_identity
from mineworld_sdk.wire.ids import EventId
from retention_driver import DAY, Pass, built, by_day, days, feed, frames, retention
from retention_fixtures import DropDayOne, DropEverything, KeepsNothingNamed
from retention_oracle import check_coverage, check_section
from retention_stream import stream
from support import run

from mineworld_cognition.memory.ingest import ingest
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.host import run_pass
from mineworld_cognition.memory.retention.registry import InvalidParameters, PolicyRegistry
from mineworld_cognition.memory.retention.selection import Retention
from mineworld_cognition.memory.retrieve import RecallQuery, retrieve
from mineworld_cognition.memory.store import MemoryStore

MIB = 1_048_576
STORE_BOUND = 20 * MIB  # the default per-person bound (pr-fg-b2 §4.1; operator, 2026-10-14)
TARGET = STORE_BOUND * 3 // 4  # 15 MiB: the dump after every pass
SEED = 7
CHECKPOINTS = (30, 100, 300)


def _drop_everything() -> Retention:
    return retention("drop-everything", PolicyRegistry({"drop-everything": DropEverything}))


def at_checkpoints(passes: list[Pass]) -> dict[int, int]:
    """The dump after the first pass at or after each checkpoint day (a silent day has no pass)."""
    return {day: len(next(each for each in passes if each.day >= day).dump) for day in CHECKPOINTS}


@pytest.mark.parametrize("policy", ["keep-all", "drop-everything", "consolidate"])
def test_every_perceived_fact_is_represented_once_after_every_pass(policy: str) -> None:
    # AFGb-3 (a) to (c) at every pass of 40 synthetic days. Its mutation lets `Drop` forget to add its
    # ranges: (a) finds the dropped ids unrepresented.
    facts = list(stream(SEED, 40))
    perceived = [fact.id for fact in facts]
    kept = _drop_everything() if policy == "drop-everything" else retention(policy)
    passes = 0
    with MemoryStore.open(":memory:", alice_identity()) as store:
        for each in by_day(store, facts, kept):
            check_coverage(each.dump, perceived)
            passes += 1
    assert passes >= 35  # 40 days, a few silent


def _bounded(passes: list[Pass]) -> None:
    for each in passes:
        size = len(each.dump)
        assert size <= TARGET, (each.day, size)
        assert size <= STORE_BOUND, (each.day, size)


def test_the_host_holds_the_bound_whatever_the_policy() -> None:
    # AFGb-4 (a), host half: after every pass of 300 synthetic days, and at days 30, 100 and 300, the dump
    # is at most TARGET under the most aggressive legal plan. keep-all's half moved to the busy stream
    # (test_retention_size.py, pr-fg-b2 §7): at ≈ 120 KB a day this stream reaches the 15 MiB target
    # only after ≈ 130 days, and 170 passes over a 15 MiB store cost minutes per platform, while the busy
    # stream crosses the target within its first day. The backstop's mutation is red there.
    facts = list(stream(SEED, 302))
    with MemoryStore.open(":memory:", alice_identity()) as store:
        passes = list(by_day(store, facts, _drop_everything()))
        # AFGb-7 (a), (b) on the day-300 store, as for the other bundled policies.
        section = retrieve(store, RecallQuery(at=facts[-1].at, words="umbrella zeppelin rain"))
        check_section(section, store.dump())
    _bounded(passes)
    print(f"\n[drop-everything] dump after the passes of days 30/100/300: {at_checkpoints(passes)}")
    check_coverage(passes[-1].dump, [fact.id for fact in facts])


@pytest.mark.parametrize("size", [1, 256])
def test_frames_never_change_the_store(size: int) -> None:
    # AFGb-5 (a) for keep-all (its backstop at 20 MiB is test_retention_size.py's): frames of 1 and of
    # 256 against the whole
    # stream at once. Its mutation splits at frame ends instead of day starts: the dumps differ.
    facts = list(stream(SEED, 24, per_day=120))
    whole = built(facts, retention("keep-all"), len(facts))
    assert built(facts, retention("keep-all"), size) == whole


def test_a_store_rebuilt_from_nothing_is_the_same(tmp_path: Path) -> None:
    # AFGb-5 (b): a file store, deleted, rebuilt from a null cursor.
    facts = list(stream(SEED, 20, per_day=120))
    path = tmp_path / "alice.sqlite"
    dumps: list[bytes] = []
    for _ in range(2):
        with MemoryStore.open(path, alice_identity()) as store:
            feed(store, frames(facts, 256), retention("keep-all"))
            dumps.append(store.dump())
        path.unlink()
    assert dumps[0] == dumps[1]


def test_a_crash_between_an_ingest_and_its_pass_changes_nothing(tmp_path: Path) -> None:
    # AFGb-5 (c): the day-9 trigger ingested, the process gone before the pass; reopened, `absorb` runs
    # the due pass first and the rest of the stream after it.
    facts = list(stream(SEED, 14, per_day=120))
    kept = retention("keep-all")
    uninterrupted = built(facts, kept, 256)
    trigger = next(f for f in facts if f.at // DAY >= 9)
    head, rest = facts[: facts.index(trigger)], facts[facts.index(trigger) + 1 :]
    path = tmp_path / "alice.sqlite"
    with MemoryStore.open(path, alice_identity()) as store:
        feed(store, frames(head, 256), kept)
        ingest(store, [trigger], trigger.id)  # what absorb does before the pass
    with MemoryStore.open(path, alice_identity()) as store:
        report = run(absorb(store, [], trigger.id, kept))
        assert len(report.passes) == 1
        feed(store, frames(rest, 256), kept)
        assert store.dump() == uninterrupted


def test_a_pass_never_moves_the_cursor_or_the_newest_key() -> None:
    # AFGb-6 (a): each day's trigger ingested by plain `ingest`, with a `through` past it where the ids
    # leave room (the server acknowledges ids Alice did not perceive), cursor and newest read, then the
    # pass itself (`run_pass`, which `absorb` calls; `absorb`'s own last ingest would re-set the cursor
    # and hide a pass that moved it). Its mutation lets the pass write the cursor down to the newest
    # fact: they differ.
    facts = list(stream(SEED, 12, per_day=80))
    kept = _drop_everything()
    frames_ = list(days(facts))
    with MemoryStore.open(":memory:", alice_identity()) as store:
        passes = ahead = 0
        # Every frame but the first and the last ends at a day's first fact (`days`).
        for frame, following in pairwise(frames_[1:]):
            last = int(frame[-1].id)
            through = EventId(str(int(following[0].id) - 1))
            ingest(store, frame, through)
            before = (store.cursor(), store.newest())
            ahead += int(through) > last
            run_pass(store, kept, frame[-1].at // DAY * DAY)
            passes += 1
            assert (store.cursor(), store.newest()) == before
    assert passes >= 10
    assert ahead >= 5


def test_a_chapter_never_cites_an_episode_forgotten_before_it() -> None:
    # AFGb-3 (c) for chapters: day 1's episodes are forgotten while week 0 has no chapter yet; the chapter
    # written in week 1 cites the runs of surviving episodes around the hole. Its mutation writes the
    # chapter as one span: it meets the forgotten range.
    facts = list(stream(SEED, 10, per_day=60))
    kept = retention("drop-day-one", PolicyRegistry({"drop-day-one": DropDayOne}))
    with MemoryStore.open(":memory:", alice_identity()) as store:
        for each in by_day(store, facts, kept):
            check_coverage(each.dump, [fact.id for fact in facts])
        (runs,) = store.connection.execute(
            "SELECT count(*) FROM chapter_ranges WHERE week = 0"
        ).fetchone()
    assert runs == 2


def test_a_users_policy_is_bound_and_run_exactly_as_a_bundled_one() -> None:
    # AFGb-1 (b): given through PolicyRegistry({...}), resolved from the JSON choice, run by absorb.
    facts = list(stream(SEED, 6, per_day=60))
    kept = retention("user-policy", PolicyRegistry({"user-policy": KeepsNothingNamed}), hours=3)
    with MemoryStore.open(":memory:", alice_identity()) as store:
        reports = [each.report.passes[-1] for each in by_day(store, facts, kept)]
    assert reports
    assert {(r.policy, r.outcome) for r in reports} == {("user-policy", "ok")}


@pytest.mark.parametrize(
    ("parameters", "field"),
    [({"l0_hours": 721}, "l0_hours"), ({"l0_days": 2}, "l0_days"), ({"l0_hours": "2"}, "l0_hours")],
)
def test_consolidates_parameters_are_refused_by_name(
    parameters: dict[str, int | str], field: str
) -> None:
    # AFGb-1 (d) on the installed `consolidate`.
    with pytest.raises(InvalidParameters) as raised:
        PolicyRegistry.installed().bind("consolidate", dict(parameters))
    assert raised.value.policy == "consolidate"
    assert [e.location for e in raised.value.errors] == [field]
