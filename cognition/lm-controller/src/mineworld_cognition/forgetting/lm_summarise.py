"""`lm-summarise`: LLM-assisted forgetting through the host's summary gateway (pr-fg-c §§5.4-5.5).

The groups are the ones `consolidate` would form. Oldest first, the policy asks the `summarize` tier for one
sentence of prose and an importance from 1 to 10 for each group, and turns each answer into decisions by the
table of §5.4. A group it cannot ask, or whose answer is invalid, keeps the fallback's decisions; the host
records why the fallback was used. A request holds names, never an id (FGc-3); the model decides which
memories survive and the host validates the result (FGc-4).
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Annotated, ClassVar, assert_never

from mineworld_sdk.wire.ids import JsonValue
from pydantic import BaseModel, ConfigDict, Field, ValidationError

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import Completion, CompletionRequest, Message
from mineworld_cognition.compress.model_summarizer import INSTRUCTIONS, SAMPLING, valid_prose
from mineworld_cognition.forgetting.consolidate import Consolidate, ConsolidateParameters
from mineworld_cognition.gateway import Completed, Failed, Refused
from mineworld_cognition.memory.retention import (
    MESSAGE_BYTES,
    AssistedPlan,
    Drop,
    Exhausted,
    GroupAnswer,
    ItemId,
    Keep,
    MemoryItem,
    Merge,
    RetentionBudget,
    RetentionDecision,
    StopReason,
    SummaryGateway,
)

RATING = (
    'Then rate from 1 to 10 how much this matters to me later. Answer as JSON: {"summary": "...", '
    '"importance": n}.'
)
"""P4's instructions, plus the importance question (pr-fg-c §5.4)."""

SCHEMA: JsonValue = {
    "type": "object",
    "properties": {
        "summary": {"type": "string"},
        "importance": {"type": "integer", "minimum": 1, "maximum": 10},
    },
    "required": ["summary", "importance"],
    "additionalProperties": False,
}
"""The answer's shape: integer-only, as the request key requires (no float)."""


class LmSummariseParameters(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    calls_per_sim_day: Annotated[int, Field(ge=0, le=24)] = 4
    """Model calls per pass: the pass's allowance (FGc-2)."""
    keep_at: Annotated[int, Field(ge=1, le=10)] = 8
    """An importance at or above this keeps the group's members and its container."""
    drop_at: Annotated[int, Field(ge=0, le=10)] = 2
    """An importance at or below this drops a top-level container; 0 disables it."""
    consolidate: ConsolidateParameters = Field(default_factory=ConsolidateParameters)
    """How the groups are formed: `consolidate`'s own parameters."""


class _Answer(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")

    summary: str
    importance: Annotated[int, Field(ge=1, le=10)]


@dataclass(frozen=True)
class _Group:
    container: ItemId
    decision: Merge | Drop
    """`consolidate`'s decision for this group: a merge of members into the container, or a top-level drop
    of the unit and what nests in it."""
    top: bool
    """Whether the container is top-level: a low importance may forget it (§5.4)."""


class LmSummarise:
    id: ClassVar[str] = "lm-summarise"
    fallback: ClassVar[str] = "consolidate"
    Parameters = LmSummariseParameters

    def calls_per_pass(self, params: LmSummariseParameters) -> int:
        return params.calls_per_sim_day

    async def plan(
        self,
        items: Sequence[MemoryItem],
        budget: RetentionBudget,
        params: LmSummariseParameters,
        gateway: SummaryGateway,
        fallback_plan: Sequence[RetentionDecision],
    ) -> AssistedPlan:
        by_id = {item.item: item for item in items}
        answers: list[GroupAnswer] = []
        invalid = 0
        stopped: StopReason | None = None
        for group in _groups(items, budget, params, by_id):
            request = _request(group, items, by_id)
            outcome = await gateway.complete(request)
            match outcome:
                case Completed(completion=completion):
                    answer = _answer(group, completion, request, params)
                    if answer is None:
                        invalid += 1
                    else:
                        answers.append(answer)
                case Refused():
                    stopped = "budget"
                case Failed():
                    stopped = "failed"
                case Exhausted(calls=calls):
                    stopped = "no-calls" if calls == 0 else "exhausted"
                case _:
                    assert_never(outcome)
            if stopped is not None:
                break
        return AssistedPlan(answers=tuple(answers), stopped=stopped, invalid=invalid)


def _groups(
    items: Sequence[MemoryItem],
    budget: RetentionBudget,
    params: LmSummariseParameters,
    by_id: dict[ItemId, MemoryItem],
) -> list[_Group]:
    """`consolidate`'s groups with the policy's own parameters, oldest container first (ties by ItemId)."""
    found: list[_Group] = []
    for decision in Consolidate().plan(items, budget, params.consolidate):
        if isinstance(decision, Merge):
            container = by_id[decision.items[0]].container
            if container is not None:
                found.append(_Group(container, decision, by_id[container].container is None))
        elif isinstance(decision, Drop):
            found.append(_Group(decision.item, decision, by_id[decision.item].container is None))
    found.sort(key=lambda group: (by_id[group.container].first_at, group.container))
    return found


def _request(
    group: _Group, items: Sequence[MemoryItem], by_id: dict[ItemId, MemoryItem]
) -> CompletionRequest:
    """The summarisation request (§5.4): the container's text, then its moments in perceived order, names
    only. Moments that would take the user message past the bound are left out with every later one, and
    `(n more)` says how many."""
    system = f"{INSTRUCTIONS} {RATING}"
    room = MESSAGE_BYTES - len(system.encode("utf-8"))
    if isinstance(group.decision, Merge):
        members = group.decision.items
    else:
        members = tuple(item.item for item in items if item.container == group.container)
    ordered = sorted(members, key=lambda member: (by_id[member].first_at, member))
    head = f"So far: {_cut(by_id[group.container].text, room // 2)}\nMoments:"
    moments = [by_id[member].text for member in ordered]
    user = _user(head, moments, room)
    return CompletionRequest(
        purpose="summarize",
        messages=(Message(role="system", text=system), Message(role="user", text=user)),
        output_schema=SCHEMA,
        sampling=SAMPLING,
    )


def _user(head: str, moments: Sequence[str], room: int) -> str:
    """The longest prefix of `moments` whose user text fits `room` bytes, with `(n more)` for the rest."""
    for shown in range(len(moments), -1, -1):
        rest = len(moments) - shown
        text = head + "".join(f"\n- {moment}" for moment in moments[:shown])
        if rest:
            text += f"\n({rest} more)"
        if len(text.encode("utf-8")) <= room:
            return text
    return head


def _cut(text: str, limit: int) -> str:
    """The container's text, cut to `limit` bytes at a character boundary, so that a long container can
    never by itself take a request over the bound."""
    raw = text.encode("utf-8")
    if len(raw) <= limit:
        return text
    return raw[:limit].decode("utf-8", errors="ignore")


def _answer(
    group: _Group, completion: Completion, request: CompletionRequest, params: LmSummariseParameters
) -> GroupAnswer | None:
    """The group's decisions from a valid answer, or None: an answer that is not complete, not the strict
    shape, out of range, or not valid prose is invalid, and its group keeps the fallback's decisions."""
    if completion.finish != "complete":
        return None
    try:
        answer = _Answer.model_validate_json(completion.text)
    except ValidationError:
        return None
    prose = valid_prose(answer.summary)
    if prose is None:
        return None
    decisions, keeps_prose = _decisions(group, answer.importance, params)
    return GroupAnswer(
        container=group.container,
        decisions=decisions,
        prose=prose if keeps_prose else None,
        prose_key=cassette_key(request) if keeps_prose else None,
    )


def _decisions(
    group: _Group, importance: int, params: LmSummariseParameters
) -> tuple[tuple[RetentionDecision, ...], bool]:
    """The decision table of §5.4, cell by cell: the decisions, and whether the container keeps prose."""
    container = group.container
    if isinstance(group.decision, Drop):
        if importance >= params.keep_at:
            return (Keep(item=container),), True
        return (group.decision,), False
    merge = group.decision
    if importance >= params.keep_at:
        return (Keep(item=container), *(Keep(item=member) for member in merge.items)), True
    if importance <= params.drop_at and group.top:
        return (Drop(item=container),), False
    return (merge,), importance > params.drop_at
