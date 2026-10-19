"""One retention pass (pr-fg-b §5.7; the assisted branch, pr-fg-c §5.2): items → policy → validate → execute →
bound → check → log.

One transaction. The policy sees frozen items and a budget, never the store (FGb-3); a plan it returns is
validated whole, and a refused plan, or a policy that raised, is replaced by `consolidate`'s defaults and
recorded as `fallback:<reason>`. An assisted policy (FG-c) is first given the fallback's plan and a
`SummaryGateway`; it may await, and every await comes before the first write. A `SummaryUnavailable`
propagates and rolls the pass back (FGc-6). The bound is the host's (FGb-2). A breach of the store-side coverage
invariant (FGb-1) is a defect: it raises, and the pass rolls back. A pass never writes `cursor` or
`newest` (P4-2) and reads no clock: `now` is fixed by the triggering fact (FGb-5).

A pass has a cause (pr-fg-b2 §4.3): `day`, at the first perceived fact of a new day, or `size`, after the
fact that took the store's content over its trigger. Both are the same pass; only a day pass advances
`meta.retention_day`. The log is keyed by the triggering fact, the newest ingested at the pass.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from itertools import pairwise
from typing import Literal, cast

from mineworld_sdk.wire.ids import EntityId

from mineworld_cognition.backend.canonical import CassetteKey
from mineworld_cognition.compress.model_summarizer import valid_prose
from mineworld_cognition.compress.summarize import DAY_S
from mineworld_cognition.memory.home import SeatView
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention.assisted import (
    CALLS_CAP,
    AssistedPlan,
    GroupAnswer,
    StopReason,
    SummaryGateway,
    SummaryUnavailable,
)
from mineworld_cognition.memory.retention.bound import enforce, fallback
from mineworld_cognition.memory.retention.forgotten import Forgetter, PassScope, normalize
from mineworld_cognition.memory.retention.items import Extracted, extract
from mineworld_cognition.memory.retention.plan import PlanRejected, Resolved, execute, validate
from mineworld_cognition.memory.retention.policy import Merge, RetentionBudget, RetentionDecision
from mineworld_cognition.memory.retention.prose import KnownNames, allowed_entities, outside_names
from mineworld_cognition.memory.retention.registry import RegisteredPolicy
from mineworld_cognition.memory.retention.selection import Retention

PASS_LOG_ROWS = 32

type PassCause = Literal["day", "size"]


@dataclass(frozen=True)
class PassReport:
    day: int
    policy: str
    outcome: str
    """`ok` (a plain policy's plan stood); `assisted` (every group asked, every answer valid); or
    `fallback:<reason>`: the policy's plan was refused or it raised, or an assisted policy's answers were
    not used whole (its `StopReason`; else `invalid`; else `prose-contradiction`, pr-s28b §5.5)."""
    kept: int
    merged: int
    dropped: int
    forced: int
    """Items the host's bound removed after the plan."""
    bytes_before: int
    bytes_after: int
    """The dump before the pass and after its bound, before the pass's own log row."""
    forgotten_ranges: int
    model_decisions: int = 0
    """Decisions of the final plan that came from accepted model answers (0 for a plain pass)."""
    fallback_decisions: int = 0
    """Decisions of the final plan that came from the fallback (0 for a plain pass)."""
    cause: PassCause = "day"
    """Why the pass ran: the day's first fact, or the store nearing its bound (pr-fg-b2 §4.3)."""
    trigger: str = ""
    """The event key of the newest fact ingested at the pass: the pass log's key."""
    prose_contradictions: int = 0
    """Answers the host refused for prose naming outside their group (FU-S28-1; 0 for a plain pass)."""


class RetentionInvariantBroken(RuntimeError):
    """The store-side half of FGb-1 failed after a pass: a defect, never a fallback."""


@dataclass(frozen=True)
class _Decided:
    """A pass's validated plan, and what the assisted branch recorded about it."""

    outcome: str
    resolved: Resolved
    model_decisions: int
    fallback_decisions: int
    prose: tuple[tuple[ItemId, str, CassetteKey], ...]
    """Prose to store on surviving answered containers: (container, prose, cassette key)."""
    prose_contradictions: int = 0
    """Answers refused because their prose named outside their group (FU-S28-1)."""


async def run_pass(
    store: SeatView, retention: Retention, now: int, cause: PassCause = "day"
) -> PassReport:
    day = now // DAY_S
    bounds = retention.bounds
    with store.transaction(retention_pass=True) as db:
        scope = PassScope.begin(store)
        extracted = extract(store)
        before = extracted.total
        budget = RetentionBudget.model_construct(
            store_bytes=max(0, bounds.target - extracted.overhead), now=now, calls_left=0
        )
        decided = await _decide(store, retention, extracted, budget, cause)
        counts = execute(store, decided.resolved, extracted, Forgetter(day, retention.policy.id))
        _write_prose(store, decided.prose, extracted)
        normalize(store, scope)
        forced, after = enforce(store, now, bounds, scope)
        check_structure(store)
        (ranges,) = db.execute(
            "SELECT count(*) FROM forgotten WHERE seat = ?", (store.seat_id,)
        ).fetchone()
        report = PassReport(
            day,
            retention.policy.id,
            decided.outcome,
            counts.kept,
            counts.merged,
            counts.dropped,
            forced,
            before,
            after,
            ranges,
            decided.model_decisions,
            decided.fallback_decisions,
            cause,
            store.newest() or "",
            decided.prose_contradictions,
        )
        _log(store, report)
    store.reclaim()
    return report


async def _decide(
    store: SeatView,
    retention: Retention,
    extracted: Extracted,
    budget: RetentionBudget,
    cause: PassCause,
) -> _Decided:
    if retention.fallback is None:
        return _plain(retention.policy, extracted, budget)
    if cause == "size":
        # A size pass never calls a model (pr-fg-b2 §4.9, R-FGc-FGb2-1): the fallback decides alone, so
        # an assisted seat's calls per simulated day stay those of its daily passes.
        resolved, decisions, refusal = _fallback_plan(retention.fallback, extracted, budget)
        outcome = f"fallback:{refusal}" if refusal is not None else "fallback:size"
        return _Decided(outcome, resolved, 0, len(decisions), ())
    return await _assisted(store, retention, extracted, budget)


def _plain(policy: RegisteredPolicy, extracted: Extracted, budget: RetentionBudget) -> _Decided:
    outcome = "ok"
    resolved: Resolved | None = None
    try:
        resolved = validate(policy.plan(extracted.items, budget), extracted)
    except PlanRejected as rejected:
        outcome = f"fallback:{rejected.reason}"
    except Exception as error:
        outcome = f"fallback:raised: {type(error).__name__}"
    if resolved is None:
        # A refused plan is never partly applied (§5.7.1). The fallback is trusted host policy: a
        # refusal of its plan is a defect and propagates.
        resolved = validate(fallback().plan(extracted.items, budget), extracted)
    return _Decided(outcome, resolved, 0, 0, ())


async def _assisted(
    store: SeatView, retention: Retention, extracted: Extracted, budget: RetentionBudget
) -> _Decided:
    """The assisted branch (pr-fg-c §5.2): the fallback's plan first, then the policy's answers, merged
    group by group into the fallback's plan and validated whole. Every await precedes every write."""
    policy = retention.policy
    fallback_policy = retention.fallback
    if fallback_policy is None:  # _decide routes only assisted policies here
        raise RetentionInvariantBroken("an assisted pass has no fallback")
    fallback_resolved, fallback_plan, refusal = _fallback_plan(fallback_policy, extracted, budget)
    unused = len(fallback_plan)
    if refusal is not None:
        return _Decided(f"fallback:{refusal}", fallback_resolved, 0, unused, ())
    gateway = retention.gateway
    if gateway is None:
        return _Decided("fallback:unbound", fallback_resolved, 0, unused, ())
    try:
        calls = _allowance(policy)
    except Exception as error:
        return _Decided(
            f"fallback:raised: {type(error).__name__}", fallback_resolved, 0, unused, ()
        )
    if calls == 0:
        return _Decided("fallback:no-calls", fallback_resolved, 0, unused, ())
    asked = RetentionBudget.model_construct(
        store_bytes=budget.store_bytes, now=budget.now, calls_left=calls
    )
    summary = SummaryGateway(gateway, store.identity.seat, calls)
    try:
        answer: object = await policy.plan_assisted(extracted.items, asked, summary, fallback_plan)
    except SummaryUnavailable:
        raise  # FGc-6: the boundary could not answer; a stop, and the pass rolls back
    except Exception as error:
        return _Decided(
            f"fallback:raised: {type(error).__name__}", fallback_resolved, 0, unused, ()
        )
    if not isinstance(answer, AssistedPlan):
        return _Decided(
            f"fallback:raised: {type(answer).__name__}", fallback_resolved, 0, unused, ()
        )
    names = KnownNames(NameBook.load(store).known())
    accepted, discarded, contradictions = _accepted(answer, extracted, store.me, names)
    answered = {each.container for each in accepted}
    allowed = {item for each in accepted for item in _group(each, extracted)}
    kept = [d for d in fallback_plan if not _replaced(d, answered, allowed, extracted)]
    merged: list[RetentionDecision] = [
        *kept,
        *(decision for each in accepted for decision in each.decisions),
    ]
    try:
        resolved = validate(merged, extracted)
    except PlanRejected as rejected:
        return _Decided(f"fallback:{rejected.reason}", fallback_resolved, 0, unused, ())
    from_model = sum(len(each.decisions) for each in accepted)
    stopped: StopReason | None = answer.stopped
    invalid = answer.invalid + discarded
    if stopped is not None:
        outcome = f"fallback:{stopped}"
    elif invalid:
        outcome = "fallback:invalid"
    elif contradictions:
        outcome = "fallback:prose-contradiction"
    else:
        outcome = "assisted"
    prose = tuple(
        (each.container, each.prose, each.prose_key)
        for each in accepted
        if each.prose is not None
        and each.prose_key is not None
        and _survives(each.container, resolved, extracted)
        and extracted.refs[each.container].level in (1, 2)
    )
    return _Decided(outcome, resolved, from_model, len(kept), prose, contradictions)


def _fallback_plan(
    policy: RegisteredPolicy, extracted: Extracted, budget: RetentionBudget
) -> tuple[Resolved, list[RetentionDecision], str | None]:
    """The fallback's validated plan, its decisions, and why it was replaced by `consolidate`'s defaults
    (None when it stood). A pass whose fallback is refused is recorded so; it never asks a model."""
    try:
        resolved, decisions = _checked_pair(policy.plan(extracted.items, budget), extracted)
        return resolved, decisions, None
    except PlanRejected as rejected:
        reason = rejected.reason
    except Exception as error:
        reason = f"raised: {type(error).__name__}"
    resolved, decisions = _checked_pair(fallback().plan(extracted.items, budget), extracted)
    return resolved, decisions, reason


def _allowance(policy: RegisteredPolicy) -> int:
    declared: object = policy.calls_per_pass()
    if not isinstance(declared, int) or isinstance(declared, bool) or declared < 0:
        raise TypeError("calls_per_pass is not a non-negative integer")
    return min(declared, CALLS_CAP)


def _accepted(
    answer: AssistedPlan, extracted: Extracted, me: EntityId, known: KnownNames
) -> tuple[list[GroupAnswer], int, int]:
    """The answers the host accepts, how many it discarded, and how many it refused as contradictions. An
    answer is discarded, and its group falls back, when it names an item outside its group, repeats a
    container, or carries prose that is not valid (AFGc-8), or prose without its cassette key. A valid answer
    whose prose names a known person or place outside its group is a **contradiction** (FU-S28-1, pr-s28b
    §5.4): refused the same way, and counted apart."""
    accepted: list[GroupAnswer] = []
    seen: set[ItemId] = set()
    discarded = contradictions = 0
    for each in answer.answers:
        if each.container in seen or each.container not in extracted.by_id:
            discarded += 1
            continue
        group = _group(each, extracted)
        names = [
            name
            for decision in each.decisions
            for name in (decision.items if isinstance(decision, Merge) else (decision.item,))
        ]
        if any(name not in group for name in names):
            discarded += 1
            continue
        if each.prose is not None and (
            each.prose_key is None or valid_prose(each.prose) != each.prose
        ):
            discarded += 1
            continue
        if each.prose is not None and outside_names(
            each.prose, known, allowed_entities(extracted, each.container, me, known)
        ):
            contradictions += 1
            continue
        seen.add(each.container)
        accepted.append(each)
    return accepted, discarded, contradictions


def _group(answer: GroupAnswer, extracted: Extracted) -> set[ItemId]:
    """A group's items: its container and the items nested directly in it."""
    return {answer.container, *extracted.children.get(answer.container, ())}


def _replaced(
    decision: RetentionDecision,
    answered: set[ItemId],
    allowed: set[ItemId],
    extracted: Extracted,
) -> bool:
    """Whether the fallback's decision belongs to an answered group and is replaced by its answer: a merge
    keyed by an answered container, or a keep or drop naming an answered group's item."""
    if isinstance(decision, Merge):
        return extracted.by_id[decision.items[0]].container in answered
    return decision.item in allowed


def _survives(item: ItemId, resolved: Resolved, extracted: Extracted) -> bool:
    removed = {*resolved.into_episodes, *resolved.into_chapters, *resolved.dropped}
    cursor: ItemId | None = item
    while cursor is not None:
        if cursor in removed:
            return False
        cursor = extracted.by_id[cursor].container
    return True


def _checked_pair(plan: object, extracted: Extracted) -> tuple[Resolved, list[RetentionDecision]]:
    """A plan validated whole (§5.7.1) and its decisions as a list (validation checked each entry)."""
    resolved = validate(plan, extracted)
    return resolved, list(cast("Sequence[RetentionDecision]", plan))


def _write_prose(
    store: SeatView, prose: tuple[tuple[ItemId, str, CassetteKey], ...], extracted: Extracted
) -> None:
    """Prose and its cassette key on surviving episodes and chapters only; citations are never touched
    (P4-1, FGc-7)."""
    db = store.connection
    for item, text, key in prose:
        ref = extracted.refs[item]
        if ref.level == 1:
            db.execute(
                "UPDATE episodes SET prose = ?, prose_key = ? WHERE seat = ? AND episode = ?",
                (text, key, store.seat_id, ref.number),
            )
        elif ref.level == 2:
            db.execute(
                "UPDATE chapters SET prose = ?, prose_key = ? WHERE seat = ? AND week = ?",
                (text, key, store.seat_id, ref.number),
            )


def _log(store: SeatView, report: PassReport) -> None:
    db = store.connection
    seat = store.seat_id
    db.execute(
        "INSERT INTO retention_passes (seat, trigger, day, cause, policy, outcome, kept, merged, "
        "dropped, forced, bytes_before, bytes_after, forgotten_ranges, model_decisions, "
        "fallback_decisions, prose_contradictions) "
        "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        (
            seat,
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
            report.model_decisions,
            report.fallback_decisions,
            report.prose_contradictions,
        ),
    )
    db.execute(
        "DELETE FROM retention_passes WHERE seat = ? AND trigger NOT IN "
        "(SELECT trigger FROM retention_passes WHERE seat = ? ORDER BY trigger DESC LIMIT ?)",
        (seat, seat, PASS_LOG_ROWS),
    )
    if report.cause == "day":
        store.set_meta("retention_day", str(report.day))


def check_structure(store: SeatView) -> None:
    """FGb-1's store-side half, on the seat's view: every record lies in an episode; episodes are disjoint;
    every episode of a week with a chapter lies in one of its ranges; no episode, chapter range or record
    meets the forgotten set; the forgotten ranges are disjoint."""
    db = store.connection
    seat = store.seat_id
    episodes: list[tuple[str, str, int]] = db.execute(
        "SELECT first_key, last_key, day / 7 FROM episodes WHERE seat = ? ORDER BY first_key",
        (seat,),
    ).fetchall()
    _disjoint("episode", [(first, last) for first, last, _ in episodes])
    keys: list[tuple[str]] = db.execute(
        "SELECT event_key FROM access WHERE seat = ? ORDER BY event_key", (seat,)
    ).fetchall()
    index = 0
    for (key,) in keys:
        while index < len(episodes) and episodes[index][1] < key:
            index += 1
        if index == len(episodes) or episodes[index][0] > key:
            raise RetentionInvariantBroken(f"record {key} lies in no episode")
    ranges: dict[int, list[tuple[str, str]]] = {}
    for week, first, last in db.execute(
        "SELECT week, first_key, last_key FROM chapter_ranges WHERE seat = ? ORDER BY week, first_key",
        (seat,),
    ):
        ranges.setdefault(week, []).append((first, last))
    chaptered: set[int] = {
        week for (week,) in db.execute("SELECT week FROM chapters WHERE seat = ?", (seat,))
    }
    if set(ranges) != chaptered:
        raise RetentionInvariantBroken("a chapter and its ranges disagree")
    for first, last, week in episodes:
        if week in ranges and not any(a <= first and last <= b for a, b in ranges[week]):
            raise RetentionInvariantBroken(
                f"episode {first}…{last} lies outside its week's chapter"
            )
    forgotten: list[tuple[str, str]] = db.execute(
        "SELECT first_key, last_key FROM forgotten WHERE seat = ? ORDER BY first_key", (seat,)
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
