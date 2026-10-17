"""The per-seat circuit breaker (pr-s10-p6 §5.6, C8, AP6-6).

The breaker reads no clock: every test passes the wall second and the time scale. The cooldown is
`cooldown_world_s / time_scale` wall seconds, so 3 600 world seconds at scale 1 is 3 600 wall seconds and at scale
60 it is 60.
"""

from __future__ import annotations

import pytest
from mineworld_sdk.wire.ids import JsonValue

from mineworld_cognition.controller.breaker import (
    BoundBreaker,
    BreakerOutcome,
    BreakerRegistry,
    BreakerState,
    CircuitBreaker,
)


def _breaker(
    name: str = "failure-rate", parameters: dict[str, JsonValue] | None = None
) -> CircuitBreaker:
    bound: BoundBreaker = BreakerRegistry.installed().bind(name, parameters or {}, "test")
    return bound.create()


def _feed(breaker: CircuitBreaker, outcomes: str, now: float = 0.0, scale: int = 1) -> BreakerState:
    """Each `f` is a failure and each other letter a success."""
    state = BreakerState(kind="closed")
    for letter in outcomes:
        outcome: BreakerOutcome = "failure" if letter == "f" else "success"
        state = breaker.observe(outcome, now, scale)
    return state


def test_below_the_minimum_of_consults_nothing_opens() -> None:
    breaker = _breaker()
    assert _feed(breaker, "f" * 9).kind == "closed"


def test_a_window_of_ten_with_three_failures_opens_and_with_two_does_not() -> None:
    assert _feed(_breaker(), "fffsssssss").kind == "open"  # 3 of 10: 30 % > 20 %
    assert _feed(_breaker(), "ffssssssss").kind == "closed"  # 2 of 10: 20 % is not > 20 %


def test_the_cooldown_is_world_seconds_over_the_time_scale() -> None:
    at_scale_one = _feed(_breaker(), "f" * 10, now=100.0, scale=1)
    at_scale_sixty = _feed(_breaker(), "f" * 10, now=100.0, scale=60)
    assert at_scale_one.until_wall_s == pytest.approx(100.0 + 3600.0)
    assert at_scale_sixty.until_wall_s == pytest.approx(100.0 + 60.0)


def test_half_open_with_a_success_closes_and_clears_the_window() -> None:
    breaker = _breaker(parameters={"cooldown_world_s": 60})
    _feed(breaker, "f" * 10, now=0.0, scale=1)
    assert breaker.state(30.0).kind == "open"
    assert breaker.state(61.0).kind == "half_open"
    assert breaker.observe("success", 61.0, 1).kind == "closed"
    assert breaker.state(62.0).kind == "closed"
    assert _feed(breaker, "f" * 9, now=62.0).kind == "closed"  # the window was cleared


def test_half_open_with_a_failure_opens_again_for_a_full_cooldown() -> None:
    breaker = _breaker(parameters={"cooldown_world_s": 60})
    _feed(breaker, "f" * 10, now=0.0, scale=1)
    reopened = breaker.observe("failure", 61.0, 1)
    assert reopened.kind == "open"
    assert reopened.until_wall_s == pytest.approx(61.0 + 60.0)


def test_never_is_always_closed() -> None:
    assert _feed(_breaker("never"), "f" * 50).kind == "closed"
