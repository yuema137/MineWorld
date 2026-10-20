"""The decision log, the counters, and logged-input replay (pr-s10-p6 §§5.11, 5.12, C10).

The log is JSON Lines: one `decision` line per consult (its inputs, its terminal record, and the answers the world
gave to each submit), and one `counters` line when asked. It holds no secret and no provider concept (I-P6-8): its
records are the controller's own types. Rotation keeps one older file, `<path>.1`, so 8 MiB per file bounds it.

`replay_log` re-runs every logged consult offline: a `ModelGateway` over a `ReplayBackend` on the cassette, a submit
that returns the logged answers, and a clock that never expires. It reports every key and every request that differs
from the log. A cassette miss raises (`CassetteMiss`): it is never an outcome (R-P6-1).
"""

from __future__ import annotations

import json
import os
from collections.abc import Iterable
from pathlib import Path
from typing import Annotated, Literal, cast

from mineworld_sdk.errors import MineWorldError
from mineworld_sdk.session import Answered, RefusedRequest
from mineworld_sdk.wire.contract import Accepted, ActionRequest, ActionResult, Observation, Rejected
from mineworld_sdk.wire.frames import RefusalCode
from mineworld_sdk.wire.ids import ActionId, EntityKey
from pydantic import Field

from mineworld_cognition.backend.model import CognitionModel, CompletionRequest
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.config import BUNDLED_RETRY, Layer
from mineworld_cognition.controller.consult import (
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    ConsultRecord,
    LayerPolicies,
    NewestState,
    consult,
)
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.gateway import Budget, GateOutcome, ModelGateway
from mineworld_cognition.memory.retrieve import MemorySection
from mineworld_cognition.record import Cassette, ReplayBackend

ROTATE_BYTES = 8 * 1024 * 1024
"""One file's bound before rotation (§5.12): two files, so 16 MiB per seat."""
REPLAY_CLOCK_S = 0.0
REPLAY_DEADLINE_S = 1.0e9
"""A replayed consult never expires: the log records outcomes, not wall time (§5.11)."""
SERVER_REFUSALS = frozenset({"rejected", "unavailable"})
"""The submit answers that the server gave a refusal (`server_refusals`, §5.12)."""


class DecisionInput(CognitionModel):
    """Everything a consult read: enough to run it again (§5.11, the logged-input replay)."""

    persona: Persona
    situation: Situation
    memory: MemorySection
    trigger: Trigger
    observation: Observation


class DecisionRecord(CognitionModel):
    kind: Literal["decision"] = "decision"
    seat: str
    consult: int = Field(ge=1)
    input: DecisionInput
    record: ConsultRecord
    answers: tuple[str, ...]
    """The world's answer to each submit, in order: `accepted`, `rejected`, `unavailable`, or `refused:<code>`."""


class Counters(CognitionModel):
    """The per-seat counters of §5.12, every field a non-negative integer. Never a world fact.

    `breaker_trips`, `stale_triggers` and `handbacks` are events of the seat runner, not of a consult, so
    `counters_from` leaves them at zero; the runner counts them itself (C11).
    """

    kind: Literal["counters"] = "counters"
    consults: int = Field(ge=0, default=0)
    acted: int = Field(ge=0, default=0)
    chose_nothing: int = Field(ge=0, default=0)
    invalid_answers: int = Field(ge=0, default=0)
    retries_transport: int = Field(ge=0, default=0)
    retries_content: int = Field(ge=0, default=0)
    retries_semantic: int = Field(ge=0, default=0)
    fallbacks: int = Field(ge=0, default=0)
    breaker_trips: int = Field(ge=0, default=0)
    timeouts: int = Field(ge=0, default=0)
    server_refusals: int = Field(ge=0, default=0)
    frame_refusals: int = Field(ge=0, default=0)
    stale_triggers: int = Field(ge=0, default=0)
    dropped: int = Field(ge=0, default=0)
    handbacks: int = Field(ge=0, default=0)
    model_calls: int = Field(ge=0, default=0)
    scope_defects: int = Field(ge=0, default=0)
    """Memory scopes withheld from a consult (pr-s28c §5.8): counted by the runner, as `handbacks` is."""


class ScopeDefectRecord(CognitionModel):
    """A memory scope withheld from one consult (pr-s28c §5.8, QS28c-3): written before the consult's own
    line. It names the scope and the error's class and message, never a gist or a parameter value."""

    kind: Literal["scope-defect"] = "scope-defect"
    seat: str
    consult: int = Field(ge=1)
    scope: str
    error: str


LogLine = Annotated[DecisionRecord | Counters | ScopeDefectRecord, Field(discriminator="kind")]


def counters_from(records: Iterable[DecisionRecord]) -> Counters:
    """Counts recomputed from the consult records alone (AP6-13 checks the status line against this)."""
    counts = {name: 0 for name in Counters.model_fields if name != "kind"}
    for item in records:
        result = item.record
        counts["consults"] += 1
        # One attempt per model call, except a budget refusal (no call made) and a disconnect (a call made, and
        # its submit failed before any attempt was written, so it is counted here).
        calls = [a for a in result.attempts if a.failure is None or a.failure.cls != "budget"]
        counts["model_calls"] += len(calls) + (1 if result.outcome == "disconnected" else 0)
        if result.outcome == "acted":
            counts["acted"] += 1
        if result.outcome == "chose":
            counts["chose_nothing"] += 1
        if result.outcome == "dropped":
            counts["dropped"] += 1
        if result.outcome in ("fell_back", "budget", "disconnected"):
            counts["fallbacks"] += 1
        if result.cause == "frame":
            counts["frame_refusals"] += 1
        if result.cause == "deadline":
            counts["timeouts"] += 1
        for layer, field in (
            ("transport", "retries_transport"),
            ("content", "retries_content"),
            ("semantic", "retries_semantic"),
        ):
            used = sum(1 for attempt in result.attempts if attempt.layer == layer)
            counts[field] += max(0, used - 1)
        for attempt in result.attempts:
            if attempt.failure is None:
                continue
            if attempt.failure.reason == "timeout":
                counts["timeouts"] += 1
            if attempt.failure.cls == "content":
                counts["invalid_answers"] += 1
        counts["server_refusals"] += sum(1 for answer in item.answers if answer in SERVER_REFUSALS)
    return Counters.model_validate({"kind": "counters", **counts})


class DecisionLog:
    """Appends JSON Lines to `path`, rotating to `<path>.1` when a line would pass `rotate_bytes`."""

    def __init__(self, path: Path, rotate_bytes: int = ROTATE_BYTES) -> None:
        self._path = path
        self._rotate_bytes = rotate_bytes

    def append(self, record: DecisionRecord) -> None:
        self._write(record.model_dump_json())

    def counters(self, counters: Counters) -> None:
        self._write(counters.model_dump_json())

    def scope_defect(self, record: ScopeDefectRecord) -> None:
        self._write(record.model_dump_json())

    def _write(self, line: str) -> None:
        encoded = (line + "\n").encode("utf-8")
        if self._path.exists() and self._path.stat().st_size + len(encoded) > self._rotate_bytes:
            os.replace(self._path, Path(f"{self._path}.1"))
        with self._path.open("ab") as file:
            file.write(encoded)


def read_decisions(path: Path) -> list[DecisionRecord]:
    """The decision lines of one log file, in order (counters lines are skipped)."""
    records: list[DecisionRecord] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        loaded: object = json.loads(line)
        if isinstance(loaded, dict) and cast(dict[str, object], loaded).get("kind") == "decision":
            # Lax only for JSON's lists-for-tuples; every bound and every extra-key rule still applies.
            records.append(DecisionRecord.model_validate(loaded, strict=False))
    return records


class ReplayReport(CognitionModel):
    checked: int = Field(ge=0)
    mismatches: tuple[str, ...]


async def replay_log(
    log_path: Path, cassette_path: Path, *, gateway: ModelGateway | None = None
) -> ReplayReport:
    """Re-runs every logged consult over the cassette and reports each key or request that differs.

    `gateway` replaces the default replay gateway with one an operator's configuration built
    (`build_router` in `replay` mode), so the `AC-4` swap is the same replay under another backend binding
    (S10 P7, IC-6). It is closed here, as the default one is."""
    if gateway is None:
        gateway = ModelGateway(
            ReplayBackend(Cassette.load(cassette_path)),
            Budget.of(
                BudgetPolicy(calls_per_wall_hour=1_000_000, tokens_per_wall_day=1_000_000_000),
                MemoryLedger(),
                _Frozen(),
            ),
        )
    mismatches: list[str] = []
    checked = 0
    try:
        for logged in read_decisions(log_path):
            checked += 1
            replayed = await _replay_one(gateway, logged)
            problem = _differs(logged.record, replayed)
            if problem is not None:
                mismatches.append(f"consult {logged.consult} of {logged.seat}: {problem}")
    finally:
        await gateway.aclose()
    return ReplayReport(checked=checked, mismatches=tuple(mismatches))


class _Frozen:
    """A clock that never moves: a replayed call is never rate-limited by wall time."""

    def now(self) -> int:
        return int(REPLAY_CLOCK_S)


async def _replay_one(gateway: ModelGateway, logged: DecisionRecord) -> ConsultRecord:
    answers = iter(logged.answers)
    seat = EntityKey(logged.seat)
    inputs = logged.input

    async def ask(request: CompletionRequest, timeout: float) -> GateOutcome:
        return await gateway.complete(seat, request, timeout_s=timeout)

    async def submit(request: ActionRequest) -> Answered | RefusedRequest:
        """Answers with the logged kind of answer. The consult branches on the kind alone, so the replayed
        rejection code is a placeholder: the log records kinds, and the requests and outcomes are what is compared."""
        answer = next(answers, "accepted")
        if answer == "accepted":
            result: ActionResult = Accepted.model_validate({"accepted": {"events": []}})
        elif answer == "unavailable":
            result = "unavailable"
        elif answer == "rejected":
            result = Rejected.model_validate({"rejected": "busy"})
        elif answer.startswith("refused:"):
            return RefusedRequest(
                code=cast(RefusalCode, answer.removeprefix("refused:")), detail=None
            )
        else:
            raise MineWorldError(f"logged answer {answer} is not one the log writes")
        return Answered(action_id=ActionId("1"), result=result)

    deps = ConsultDeps(
        ask=ask,
        newest=lambda: NewestState(situation=inputs.situation, observation=inputs.observation),
        # The pre-submit check read live state the log does not hold; its verdict is the logged outcome: a
        # consult logged `dropped` was superseded at submit time (S10 P7 F-P7-6).
        current=lambda: logged.record.outcome != "dropped",
        submit=submit,
        clock=lambda: REPLAY_CLOCK_S,
        sleep=_no_sleep,
        policies=_bundled(),
        limits=ConsultLimits(deadline_s=REPLAY_DEADLINE_S, max_calls=5),
    )
    return await consult(
        ConsultInput(
            seat=seat,
            persona=inputs.persona,
            situation=inputs.situation,
            memory=inputs.memory,
            trigger=inputs.trigger,
        ),
        deps,
    )


async def _no_sleep(seconds: float) -> None:
    return None


def _bundled() -> LayerPolicies:
    def bind(layer: Layer) -> BoundRetry:
        default = BUNDLED_RETRY[layer]
        return RetryRegistry.installed().bind(default.policy, dict(default.parameters), layer)

    return LayerPolicies(
        transport=bind("transport"), content=bind("content"), semantic=bind("semantic")
    )


def _differs(logged: ConsultRecord, replayed: ConsultRecord) -> str | None:
    logged_keys = [attempt.key for attempt in logged.attempts]
    replayed_keys = [attempt.key for attempt in replayed.attempts]
    if logged_keys != replayed_keys:
        return "cassette keys differ"
    if logged.outcome != replayed.outcome:
        return f"outcome {replayed.outcome} != logged {logged.outcome}"
    if logged.request != replayed.request:
        return "the request differs"
    return None
