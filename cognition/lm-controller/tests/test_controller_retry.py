"""Failure classes and retry policies (pr-s10-p6 §§5.5.1, 5.5.2, C7, AP6-5).

The classification table is one test with a row per line of §5.5.1. The exponential delays are checked against
values computed by hand: `random.Random(0).random()` gives 0.8444218515250481, 0.7579544029403025 and
0.420571580830845 (the first three draws of the seed-0 stream), so with `first_s` 0.5, `factor` 2, `ceiling_s`
8 and `jitter` 0.5 the first delays are 0.5 * (0.5 + 0.8444218515250481), 1.0 * (0.5 + 0.7579544029403025) and
2.0 * (0.5 + 0.420571580830845).
"""

from __future__ import annotations

import random

import pytest
from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import RefusedRequest
from mineworld_sdk.wire.ids import JsonValue

from mineworld_cognition.backend.model import BackendFailure
from mineworld_cognition.budget import BudgetRefusal
from mineworld_cognition.controller.choice import ContentFailure
from mineworld_cognition.controller.failures import (
    FailureRecord,
    from_content,
    from_gate,
    from_refused,
    from_seat,
    from_semantic,
)
from mineworld_cognition.controller.intent import SemanticFailure
from mineworld_cognition.controller.retry import (
    BoundRetry,
    InvalidRetryParameters,
    RetryRegistry,
    UnknownRetryPolicy,
)
from mineworld_cognition.gateway import Failed, Refused

CLASSIFICATION: list[tuple[str, FailureRecord, str, bool]] = [
    ("timeout", from_gate(Failed(BackendFailure(reason="timeout"))), "transport", True),
    ("unreachable", from_gate(Failed(BackendFailure(reason="unreachable"))), "transport", True),
    (
        "malformed response",
        from_gate(Failed(BackendFailure(reason="malformed_response"))),
        "transport",
        True,
    ),
    *[
        (
            f"http {code}",
            from_gate(Failed(BackendFailure(reason="http_status", status=code))),
            "transport",
            True,
        )
        for code in (408, 409, 425, 429, 500, 502, 503, 504, 529)
    ],
    (
        "http 400 (not retryable)",
        from_gate(Failed(BackendFailure(reason="http_status", status=400))),
        "transport",
        False,
    ),
    (
        "http 404 (not retryable)",
        from_gate(Failed(BackendFailure(reason="http_status", status=404))),
        "transport",
        False,
    ),
    ("unauthorized", from_gate(Failed(BackendFailure(reason="unauthorized"))), "transport", False),
    (
        "budget refusal",
        from_gate(
            Refused(BudgetRefusal(limit="calls_per_wall_hour", spent=20, asked=1, ceiling=20))
        ),
        "budget",
        False,
    ),
    (
        "completed but unusable",
        from_content(ContentFailure(reason="schema", detail="x")),
        "content",
        True,
    ),
    (
        "not offered",
        from_semantic(SemanticFailure(reason="not_offered", detail="x")),
        "semantic",
        True,
    ),
    ("frame refusal", from_refused(RefusedRequest(code="paused", detail=None)), "frame", False),
    (
        "rate limited (D-P6b-35)",
        from_refused(RefusedRequest(code="rate_limited", detail=None, retry_after_ms=250)),
        "transport",
        True,
    ),
    ("seat ended", from_seat(MineWorldError("gone")), "seat", False),
]


@pytest.mark.parametrize(
    ("row", "record", "cls", "retryable"),
    CLASSIFICATION,
    ids=[row[0] for row in CLASSIFICATION],
)
def test_the_classification_table_row_by_row(
    row: str, record: FailureRecord, cls: str, retryable: bool
) -> None:
    assert record.cls == cls
    assert record.retryable is retryable


def _bind(name: str, parameters: dict[str, JsonValue] | None = None) -> BoundRetry:
    return RetryRegistry.installed().bind(name, parameters or {}, "test")


def _failure(retry_after_s: int | None = None) -> FailureRecord:
    return FailureRecord(
        cls="transport", reason="timeout", retryable=True, retry_after_s=retry_after_s
    )


def test_exponential_delays_match_the_hand_computed_values_for_seed_zero() -> None:
    bound = _bind("exponential-backoff")
    jitter = random.Random(0)
    first = bound.decide(1, _failure(), jitter)
    second = bound.decide(2, _failure(), jitter)
    assert first.retry is True
    assert first.delay_s == pytest.approx(0.5 * (0.5 + 0.8444218515250481))
    assert second.delay_s == pytest.approx(1.0 * (0.5 + 0.7579544029403025))


def test_delays_stay_within_the_jitter_band_of_the_base() -> None:
    bound = _bind("exponential-backoff")
    jitter = random.Random(7)
    for _ in range(200):
        decision = bound.decide(2, _failure(), jitter)
        assert 0.5 <= decision.delay_s <= 1.5  # base 1.0, jitter 0.5


def test_retry_after_lengthens_the_delay_to_at_least_its_value() -> None:
    decision = _bind("exponential-backoff").decide(2, _failure(retry_after_s=6), random.Random(0))
    assert decision.delay_s >= 6.0


def test_three_attempts_give_exactly_two_retries() -> None:
    bound = _bind("exponential-backoff", {"attempts": 3})
    retries = [bound.decide(attempt, _failure(), random.Random(0)).retry for attempt in (1, 2, 3)]
    assert retries == [True, True, False]


def test_none_never_retries_and_fixed_waits_its_delay() -> None:
    assert _bind("none").decide(1, _failure(), random.Random(0)).retry is False
    decision = _bind("fixed", {"attempts": 2, "delay_s": 3.0}).decide(
        1, _failure(), random.Random(0)
    )
    assert (decision.retry, decision.delay_s) == (True, 3.0)


def test_an_unknown_id_and_an_out_of_bounds_parameter_are_refused_naming_the_source() -> None:
    registry = RetryRegistry.installed()
    with pytest.raises(UnknownRetryPolicy, match=r"retry\.transport"):
        registry.bind("nope", {}, "retry.transport")
    with pytest.raises(InvalidRetryParameters, match="attempts"):
        registry.bind("fixed", {"attempts": 0}, "retry.content")
