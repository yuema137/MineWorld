"""FU-S28-1, C-4 (d): model prose may not name outside its group (pr-s28b §§5.4 … 5.6, 6.2; AC-b-4).

Unit cases run `outside_names` on literal name books. Host cases run one assisted pass on a hand-built view of
four closed episodes with named people and places, through `absorb`, with a test-only assisted policy whose
answers are written here (no model, no cassette): every expected outcome, count and row is a literal or the
dump of the same pass without the refused answers (a differential, so the refused groups are shown to keep the
fallback's decisions exactly).
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import ClassVar

import pytest
from memory_fixtures import ALICE, BOB, CAFE, CAROL, DEV, PARK, all_rows, named, seat_view, spoke
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.ids import EntityId
from pydantic import BaseModel, ConfigDict
from support import completion, run

from mineworld_cognition.backend.canonical import CassetteKey
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.forgetting.consolidate import Consolidate
from mineworld_cognition.gateway import Budget, ModelGateway
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention import Keep, MemoryItem, RetentionBudget
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.assisted import (
    AssistedPlan,
    GroupAnswer,
    StopReason,
    SummaryGateway,
)
from mineworld_cognition.memory.retention.policy import RetentionDecision
from mineworld_cognition.memory.retention.prose import KnownNames, allowed, outside_names
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import MemoryPolicyChoice, resolve

ERIN = EntityId("7000000005")
HOUR = 3_600
DAY = 86_400
KEY = CassetteKey("0" * 64)

# ── outside_names ─────────────────────────────────────────────────────────────────────────────────────────

BOOK: dict[EntityId, str] = {
    ALICE: "Alice Moreau",
    BOB: "Bob Achterberg",
    CAROL: "Carol",
    DEV: "Dr. O'Neil (Jr)",
    ERIN: "Zoë",
    CAFE: "Cafe Lune",
    PARK: "Park Green",
    EntityId("7000000006"): "Grace",
    EntityId("7000000007"): "Bob",
}


def _outside(prose: str, *allowed: EntityId) -> tuple[str, ...]:
    return outside_names(prose, KnownNames(BOOK), {ALICE, *allowed})


def test_an_allowed_counterpart_and_the_groups_place_are_no_contradiction() -> None:
    assert _outside("I met Carol at Cafe Lune.", CAROL, CAFE) == ()


def test_another_known_person_or_place_is_a_contradiction() -> None:
    assert _outside("I met Carol and Grace at Cafe Lune.", CAROL, CAFE) == ("Grace",)
    assert _outside("I met Carol at Park Green.", CAROL, CAFE) == ("Park Green",)


def test_an_unknown_capitalised_word_is_no_name() -> None:
    assert _outside("I met Carol by the Old Mill.", CAROL) == ()


def test_the_seat_may_name_itself() -> None:
    # F-S28b-7. Its mutation M-b-6 (no seat entity in the allowed set) reads ("Alice Moreau",).
    known = KnownNames(BOOK)
    assert outside_names("Alice Moreau met Carol.", known, allowed(ALICE, [CAROL], [], known)) == ()


def test_a_shorter_outside_name_inside_an_allowed_one_is_no_match() -> None:
    # `Bob` is another person's name; inside the allowed `Bob Achterberg` it is not a separate match. Its
    # mutation M-b-7 (no longest-first masking) reads ("Bob",).
    assert _outside("Bob Achterberg waved.", BOB) == ()
    assert _outside("Bob waved.", BOB) == ("Bob",)


def test_names_match_whole_words_and_case() -> None:
    assert _outside("grace and Carolina came.", CAROL) == ()


def test_names_with_metacharacters_and_beyond_ascii_match_literally() -> None:
    assert _outside("Dr. O'Neil (Jr) and Zoë came.", CAROL) == ("Dr. O'Neil (Jr)", "Zoë")
    assert _outside("Dr. O'Neil (Jr) and Zoë came.", DEV, ERIN) == ()
    assert _outside("Dr X O'Neil Jr came.", CAROL) == ()


# ── The host's refusal ────────────────────────────────────────────────────────────────────────────────────

GROUPS: dict[str, str] = {
    "l1:2": "I met Bob Achterberg at Cafe Lune.",  # (i) only allowed names
    "l1:3": "Carol and Bob Achterberg came by.",  # (ii) a person outside
    "l1:4": "Bob Achterberg and I walked in Park Green.",  # (iii) a place outside
    "l1:5": "Dev told me Erin is late.",  # (iv) Erin is quoted in a member's gist (QS28b-1)
}


class Parameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


class Speaks:
    """Test-only: answers every `Merge` group of its fallback's plan whose container is in `PROSE`, keeping the
    container and its members, with that prose. `EXTRA` adds answers as given; `STOP` is its stop reason."""

    id: ClassVar[str] = "speaks"
    fallback: ClassVar[str] = "consolidate"
    Parameters = Parameters
    PROSE: ClassVar[dict[str, str]] = GROUPS
    EXTRA: ClassVar[tuple[tuple[str, str], ...]] = ()
    STOP: ClassVar[StopReason | None] = None

    def calls_per_pass(self, params: Parameters) -> int:
        return 4

    async def plan(
        self,
        items: Sequence[MemoryItem],
        budget: RetentionBudget,
        params: Parameters,
        gateway: SummaryGateway,
        fallback_plan: Sequence[RetentionDecision],
    ) -> AssistedPlan:
        # Every container in `PROSE`, whatever the fallback decided for it: since FG-d (F-FGd-1) `consolidate`
        # keeps a person's first exchanges as records, so these episodes need not be among its merges.
        answers: list[GroupAnswer] = []
        for item in items:
            prose = self.PROSE.get(item.item)
            if prose is not None:
                members = tuple(each.item for each in items if each.container == item.item)
                answers.append(_answer(item.item, prose, members))
        for container, prose in self.EXTRA:
            answers.append(_answer(ItemId(container), prose, ()))
        return AssistedPlan(answers=tuple(answers), stopped=self.STOP, invalid=0)


def _answer(container: ItemId, prose: str, members: Sequence[ItemId]) -> GroupAnswer:
    kept = (Keep(item=container), *(Keep(item=each) for each in members))
    return GroupAnswer(container=container, decisions=kept, prose=prose, prose_key=KEY)


def _policy(
    prose: dict[str, str], extra: tuple[tuple[str, str], ...] = (), stop: StopReason | None = None
) -> type[Speaks]:
    fields = {"id": "scripted", "PROSE": prose, "EXTRA": extra, "STOP": stop}
    return type("Scripted", (Speaks,), fields)


class _Clock:
    def now(self) -> int:
        return 1_760_000_000


def _facts() -> list[PerceivedEvent]:
    """Day 0: names, then four episodes (Cafe with Bob, Park with Carol, Cafe with Bob, Park with Dev, who says
    that Erin is late); day 1's first fact runs the pass."""
    t = 8 * HOUR
    return [
        named(1, ALICE, "Alice Moreau"),
        named(2, BOB, "Bob Achterberg"),
        named(3, CAROL, "Carol"),
        named(4, DEV, "Dev"),
        named(5, ERIN, "Erin"),
        spoke(10, t, ALICE, BOB, "Good morning.", CAFE),
        spoke(11, t + 60, BOB, ALICE, "Morning to you.", CAFE),
        spoke(20, t + 2 * HOUR, ALICE, CAROL, "A fine day.", PARK),
        spoke(21, t + 2 * HOUR + 60, CAROL, ALICE, "Indeed it is.", PARK),
        spoke(30, t + 4 * HOUR, ALICE, BOB, "Lunch together?", CAFE),
        spoke(31, t + 4 * HOUR + 60, BOB, ALICE, "Yes, gladly.", CAFE),
        spoke(40, t + 6 * HOUR, DEV, ALICE, "Erin is late again.", PARK),
        spoke(41, t + 6 * HOUR + 60, ALICE, DEV, "Oh no, not again.", PARK),
        # Dev's last line is the one the episode's own text quotes, so Erin is named only in a nested record.
        spoke(42, t + 6 * HOUR + 120, DEV, ALICE, "Never mind that now.", PARK),
        spoke(50, DAY + t, ALICE, BOB, "Hello again.", CAFE),
    ]


def _passed(policy: type[Speaks]) -> tuple[bytes, list[dict[str, object]]]:
    """The pass of day 1 under `policy`; the dump and its pass-log rows."""
    registry = PolicyRegistry({"scripted": policy, "consolidate": Consolidate})
    backend = ScriptedBackend(
        lambda _request: completion("{}")
    )  # never asked: the answers are written here
    policy_budget = BudgetPolicy(calls_per_wall_hour=1_000, tokens_per_wall_day=1_000_000)
    gateway = ModelGateway(backend, Budget.of(policy_budget, MemoryLedger(), _Clock()))
    kept = resolve(MemoryPolicyChoice(policy="scripted"), registry, gateway)
    facts = _facts()
    with seat_view() as store:
        _places(store)
        run(absorb(store, facts, facts[-1].id, kept), timeout_s=60)
        dump = store.dump()
    return dump, [row for row in all_rows(dump) if row["table"] == "retention_passes"]


def _places(store: SeatView) -> None:
    with store.transaction():
        book = NameBook.load(store)
        book.learn(CAFE, "Cafe Lune")
        book.learn(PARK, "Park Green")


def _without_log(dump: bytes) -> list[dict[str, object]]:
    return [row for row in all_rows(dump) if row["table"] != "retention_passes"]


def _prose(dump: bytes) -> dict[object, object]:
    return {row["episode"]: row["prose"] for row in all_rows(dump) if row["table"] == "episodes"}


def test_prose_naming_outside_its_group_is_refused_and_its_group_falls_back() -> None:
    # AC-b-4: (ii) and (iii) keep the fallback's decisions and store no prose; (i) and (iv) stand. The dump equals
    # the same pass with (ii) and (iii) unanswered, but for the pass log's outcome and count. Its mutation M-b-5
    # (no nested items in the allowed set) refuses (iv) too: Erin is named only in a record's gist.
    dump, (log,) = _passed(_policy(GROUPS))
    assert (log["outcome"], log["prose_contradictions"]) == ("fallback:prose-contradiction", 2)
    assert _prose(dump) == {
        1: None,
        2: GROUPS["l1:2"],
        3: None,
        4: None,
        5: GROUPS["l1:5"],
        6: None,
    }
    alone, (quiet,) = _passed(_policy({k: GROUPS[k] for k in ("l1:2", "l1:5")}))
    assert (quiet["outcome"], quiet["prose_contradictions"]) == ("assisted", 0)
    assert _without_log(dump) == _without_log(alone)
    differing = {name for name in log if log[name] != quiet[name]}
    assert differing == {"outcome", "prose_contradictions"}


def test_another_invalid_answer_takes_precedence_and_the_count_stays() -> None:
    # §5.5's precedence: a malformed answer (a `#` in its prose, and an outside place too) is `invalid`, counted
    # once there and never as a contradiction. Its mutations M-b-8 (a contradiction counted as invalid) and
    # M-b-9 (the name check before `valid_prose`'s discard) change the count.
    extra = (("l1:1", "Park Green #1 was busy."),)
    _, (log,) = _passed(_policy(GROUPS, extra))
    assert (log["outcome"], log["prose_contradictions"]) == ("fallback:invalid", 2)


def test_a_stop_takes_precedence_and_the_count_stays() -> None:
    _, (log,) = _passed(_policy(GROUPS, stop="exhausted"))
    assert (log["outcome"], log["prose_contradictions"]) == ("fallback:exhausted", 2)


@pytest.mark.parametrize("policy", ["consolidate"])
def test_a_plain_pass_logs_no_contradiction(policy: str) -> None:
    facts = _facts()
    kept = resolve(MemoryPolicyChoice(policy=policy), PolicyRegistry.installed())
    with seat_view() as store:
        run(absorb(store, facts, facts[-1].id, kept), timeout_s=60)
        (log,) = [row for row in all_rows(store.dump()) if row["table"] == "retention_passes"]
    assert log["prose_contradictions"] == 0
