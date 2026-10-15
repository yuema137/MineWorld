"""The assisted retention interface (pr-fg-c §5.1): a policy that may ask a model, through the host's gateway.

An assisted policy is shown the same frozen items as a plain one, plus two things: the **fallback's plan**,
computed first by the host, so the policy states only where it differs group by group; and a
`SummaryGateway`, the host's bounded view of the seat's `ModelGateway` for one pass. The gateway enforces
the 6 000-byte request bound and the pass's call allowance before any request reaches the backend, so no
policy can exceed them by writing code that does not check (FGc-2). A policy never holds the
`ModelGateway` and never writes: the host validates the merged plan and executes it (FGc-4).

`SummaryUnavailable` is the one stop an assisted policy can meet: the summary boundary could not answer
(a replay that lacks a recording, FGc-6). The host re-raises it and never treats it as a fallback; the
boundary that maps the replay's miss to it is outside this package, so memory never imports `record`.
"""

from __future__ import annotations

from collections.abc import Sequence
from typing import Annotated, ClassVar, Literal, Protocol

from mineworld_sdk.wire.ids import EntityKey
from pydantic import Field

from mineworld_cognition.backend.canonical import CassetteKey
from mineworld_cognition.backend.model import CognitionModel, CompletionRequest
from mineworld_cognition.gateway import Completed, Failed, GateOutcome, ModelGateway, Refused
from mineworld_cognition.memory.records import ItemId
from mineworld_cognition.memory.retention.policy import (
    MemoryItem,
    RetentionBudget,
    RetentionDecision,
)

StopReason = Literal["no-calls", "exhausted", "budget", "failed", "unbound"]
"""Why the remaining groups of a pass were not asked a model: no allowance from the start; the allowance
spent; P5's budget refused the call; the backend failed; no `summarize` tier is bound."""

CALLS_CAP = 24
"""The most calls one pass may make, whatever a policy declares (step-24 §10.1; FGc-2)."""

MESSAGE_BYTES = 6_000
"""The most UTF-8 bytes of message text in one request (step-24 §10.1; D-P4-9)."""


class Exhausted(CognitionModel):
    """The pass's call allowance is spent. An ordinary end of the allowance, returned, never raised."""

    calls: Annotated[int, Field(ge=0)]


SummaryOutcome = Completed | Refused | Failed | Exhausted
"""P5's three gateway outcomes, plus the pass's own allowance."""


class RequestTooLarge(ValueError):
    """A programming error in a policy: a request over `MESSAGE_BYTES`. Raised before the backend."""

    def __init__(self, size: int) -> None:
        super().__init__(
            f"a summary request holds {size} bytes of text; the bound is {MESSAGE_BYTES}"
        )
        self.size = size


class WrongPurpose(ValueError):
    """A programming error in a policy: a request whose purpose is not `summarize`. Raised before the
    backend."""

    def __init__(self, purpose: str) -> None:
        super().__init__(f"an assisted policy may only ask for summaries, not {purpose!r}")
        self.purpose = purpose


class SummaryUnavailable(Exception):
    """The summary boundary could not answer, and the pass must stop: never a fallback (FGc-6). Raised by
    the boundary that maps a replay's miss; the cause is chained (`__cause__`) and is the original error."""


class SummaryGateway:
    """The host's bounded view of one seat's gateway for one pass. Constructed by the host only.

    Each request is checked before `ModelGateway.complete`: its purpose, its size, then the allowance.
    A request that passes all three is one call of the allowance, whatever P5 answers."""

    def __init__(self, gateway: ModelGateway, seat: EntityKey, calls_left: int) -> None:
        self._gateway = gateway
        self._seat = seat
        self._calls_left = calls_left
        self._calls_made = 0

    @property
    def calls_made(self) -> int:
        return self._calls_made

    async def complete(self, request: CompletionRequest) -> SummaryOutcome:
        if request.purpose != "summarize":
            raise WrongPurpose(request.purpose)
        size = sum(len(message.text.encode("utf-8")) for message in request.messages)
        if size > MESSAGE_BYTES:
            raise RequestTooLarge(size)
        if self._calls_made >= self._calls_left:
            return Exhausted(calls=self._calls_made)
        self._calls_made += 1
        outcome: GateOutcome = await self._gateway.complete(self._seat, request)
        return outcome


class GroupAnswer(CognitionModel):
    """What a model's answer decided for one group: the group's container and the decisions that replace
    the fallback's for it. The decisions name the container and its members only (checked by the host)."""

    container: ItemId
    decisions: tuple[RetentionDecision, ...]
    prose: str | None
    """For a container that survives: the summary, validated by the host through `valid_prose`."""
    prose_key: CassetteKey | None
    """The cassette key of the request that produced `prose`."""


class AssistedPlan(CognitionModel):
    answers: tuple[GroupAnswer, ...]
    """The groups decided from a model answer."""
    stopped: StopReason | None
    """Why the remaining groups were not asked, if any."""
    invalid: Annotated[int, Field(ge=0)]
    """Answers discarded (schema, range, prose), each of whose groups fell back."""


class AssistedForgettingPolicy[P: CognitionModel](Protocol):
    """An assisted retention policy: found through the entry-point group `mineworld.forgetting` under its
    `id`, like a plain one, and told apart by `calls_per_pass`. Its `fallback` is a plain policy's id: the
    no-model policy that decides every group the model does not."""

    id: ClassVar[str]
    Parameters: type[P]
    fallback: ClassVar[str]

    def calls_per_pass(self, params: P) -> int:
        """The policy's own allowance of calls per pass, 0 … 24; the host caps it at `CALLS_CAP`."""
        ...

    async def plan(
        self,
        items: Sequence[MemoryItem],
        budget: RetentionBudget,
        params: P,
        gateway: SummaryGateway,
        fallback_plan: Sequence[RetentionDecision],
    ) -> AssistedPlan: ...
