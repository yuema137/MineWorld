"""Failure classes: one fixed table, one function per source (pr-s10-p6 §5.5.1, C7).

Each observed failure becomes a `FailureRecord` with its class, a reason, whether it is retryable, and the
server's `Retry-After` when there was one. The table is the design's §5.5.1, row for row. A `CassetteMiss` is not
here: it is never an outcome, it propagates and stops the process (R-P6-1).
"""

from __future__ import annotations

from typing import Literal

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import RefusedRequest

from mineworld_cognition.backend.model import BackendFailure, CognitionModel
from mineworld_cognition.budget import BudgetRefusal
from mineworld_cognition.controller.choice import ContentFailure
from mineworld_cognition.controller.intent import SemanticFailure
from mineworld_cognition.gateway import Failed, Refused

FailureClass = Literal["transport", "budget", "content", "semantic", "frame", "seat"]

RETRYABLE_STATUS: frozenset[int] = frozenset({408, 409, 425, 429, 500, 502, 503, 504, 529})
"""The HTTP statuses that are transport failures worth a retry (§5.5.1)."""
TRANSPORT_RETRYABLE: frozenset[str] = frozenset({"timeout", "unreachable", "malformed_response"})


class FailureRecord(CognitionModel):
    cls: FailureClass
    reason: str
    retryable: bool
    retry_after_s: int | None = None


def from_gate(outcome: Failed | Refused) -> FailureRecord:
    """A gateway outcome that is not a completion: a transport failure, or a budget refusal."""
    if isinstance(outcome, Refused):
        return _budget(outcome.refusal)
    return _transport(outcome.failure)


def _transport(failure: BackendFailure) -> FailureRecord:
    if failure.reason in TRANSPORT_RETRYABLE:
        return FailureRecord(
            cls="transport",
            reason=failure.reason,
            retryable=True,
            retry_after_s=failure.retry_after_s,
        )
    if failure.reason == "http_status" and failure.status in RETRYABLE_STATUS:
        return FailureRecord(
            cls="transport",
            reason=f"http_status:{failure.status}",
            retryable=True,
            retry_after_s=failure.retry_after_s,
        )
    if failure.reason == "http_status":
        return FailureRecord(
            cls="transport", reason=f"http_status:{failure.status}", retryable=False
        )
    return FailureRecord(cls="transport", reason=failure.reason, retryable=False)


def _budget(refusal: BudgetRefusal) -> FailureRecord:
    return FailureRecord(cls="budget", reason=str(refusal.limit), retryable=False)


def from_content(failure: ContentFailure) -> FailureRecord:
    return FailureRecord(cls="content", reason=failure.reason, retryable=True)


def from_semantic(failure: SemanticFailure) -> FailureRecord:
    return FailureRecord(cls="semantic", reason=failure.reason, retryable=True)


def from_refused(refused: RefusedRequest) -> FailureRecord:
    """A frame refusal (such as `paused`): not the model's fault, and not retried (§5.5.1)."""
    return FailureRecord(cls="frame", reason=refused.code, retryable=False)


def from_seat(error: MineWorldError) -> FailureRecord:
    """The seat ended or was never connected: no retry, the consult ends `disconnected`."""
    return FailureRecord(cls="seat", reason=type(error).__name__, retryable=False)
