"""AP5-7: budgets on wall time, decided before everything, durable across a restart (ARC-57)."""

from __future__ import annotations

import ast
import asyncio
import time
from pathlib import Path
from typing import assert_never

from mineworld_sdk.wire.ids import EntityKey
from support import completion, request, run

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.budget import BudgetPolicy, Ledger, MemoryLedger, SqliteLedger
from mineworld_cognition.gateway import (
    Budget,
    Completed,
    Failed,
    GateOutcome,
    ModelGateway,
    Refused,
)
from mineworld_cognition.record import Cassette, RecordingBackend, ReplayBackend

ALICE = EntityKey("alice")
BOB = EntityKey("bob")
SOURCE = Path(__file__).resolve().parent.parent / "src" / "mineworld_cognition"


class FakeClock:
    def __init__(self, now: int = 1_000_000) -> None:
        self.at = now

    def now(self) -> int:
        return self.at


def _describe(outcome: GateOutcome) -> str:
    # Every consumer matches the outcome exhaustively; pyright checks the assert_never.
    match outcome:
        case Completed(completion=answer):
            return f"completed:{answer.text}"
        case Refused(refusal=refusal):
            return f"refused:{refusal.limit}"
        case Failed(failure=failure):
            return f"failed:{failure.reason}"
        case _:
            assert_never(outcome)


def _gateway(
    backend: ScriptedBackend | RecordingBackend | ReplayBackend,
    ledger: Ledger,
    clock: FakeClock,
    policy: BudgetPolicy | None = None,
) -> ModelGateway:
    return ModelGateway(backend, Budget.of(policy or BudgetPolicy(), ledger, clock))


def _answer(call: CompletionRequest) -> Completion:
    return completion(call.messages[0].text)


def test_the_21st_call_in_a_wall_hour_is_refused_before_the_backend() -> None:
    # (i) and (ii); P5-2: a refused call never reaches the backend.
    clock = FakeClock()
    backend = ScriptedBackend(_answer)
    gateway = _gateway(backend, MemoryLedger(), clock)

    async def session() -> list[str]:
        outcomes = [await gateway.complete(ALICE, request(f"q{i}")) for i in range(21)]
        outcomes.append(await gateway.complete(BOB, request("bob")))
        clock.at += 3601
        outcomes.append(await gateway.complete(ALICE, request("later")))
        return [_describe(outcome) for outcome in outcomes]

    described = run(session())
    assert described[:20] == [f"completed:q{i}" for i in range(20)]
    assert described[20] == "refused:calls_per_wall_hour"
    assert described[21] == "completed:bob"  # another seat is unaffected
    assert described[22] == "completed:later"  # the window rolled after 3 601 wall seconds
    assert backend.calls == 22  # 20 + bob + later: the refused call never reached it


def test_a_call_that_could_exceed_the_daily_tokens_is_refused_before_it_is_made() -> None:
    # (iii): the reservation is the estimated input plus every output token the call may use.
    backend = ScriptedBackend(
        lambda call: completion("x", input_tokens=10_000, output_tokens=9_000)
    )
    gateway = _gateway(backend, MemoryLedger(), FakeClock())

    async def session() -> list[str]:
        first = await gateway.complete(ALICE, request("a", max_output_tokens=1000))
        # 19 000 spent; 4 + 12 000 more would exceed 30 000.
        second = await gateway.complete(ALICE, request("b", max_output_tokens=12_000))
        third = await gateway.complete(ALICE, request("c", max_output_tokens=10_000))
        return [_describe(first), _describe(second), _describe(third)]

    assert run(session()) == [
        "completed:x",
        "refused:tokens_per_wall_day",
        "completed:x",
    ]
    assert backend.calls == 2


def test_the_sqlite_ledger_survives_a_restart(tmp_path: Path) -> None:
    # (iv): 20 calls, the ledger closed and reopened, and the 21st refused.
    path = tmp_path / "state" / "budget.sqlite"
    clock = FakeClock()
    ledger = SqliteLedger(path)
    gateway = _gateway(ScriptedBackend(_answer), ledger, clock)

    async def twenty() -> None:
        for i in range(20):
            assert isinstance(await gateway.complete(ALICE, request(f"q{i}")), Completed)

    run(twenty())
    ledger.close()
    reopened = SqliteLedger(path)
    restarted = _gateway(ScriptedBackend(_answer), reopened, clock)
    outcome = run(restarted.complete(ALICE, request("21st")))
    reopened.close()
    assert _describe(outcome) == "refused:calls_per_wall_hour"


def test_no_budget_code_reads_simulated_time() -> None:
    # (v), P5-3: budget.py and gateway.py import nothing from the world contract and take no WorldTime.
    for name in ("budget.py", "gateway.py"):
        tree = ast.parse((SOURCE / name).read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.ImportFrom):
                assert node.module is not None
                assert "contract" not in node.module, f"{name} imports {node.module}"
            if isinstance(node, ast.Name | ast.Attribute):
                text = node.id if isinstance(node, ast.Name) else node.attr
                assert text not in ("WorldTime", "sim_time", "observation"), f"{name}: {text}"


def test_a_third_concurrent_call_waits_and_calls_are_admitted_first_in_first_out() -> None:
    # (vi): max_in_flight = 2.
    started: list[str] = []
    in_flight = 0
    peak = 0

    class Holding:
        def __init__(self) -> None:
            self.release: dict[str, asyncio.Event] = {}

        async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
            nonlocal in_flight, peak
            name = request.messages[0].text
            started.append(name)
            in_flight += 1
            peak = max(peak, in_flight)
            await self.release.setdefault(name, asyncio.Event()).wait()
            in_flight -= 1
            return completion(name)

        async def aclose(self) -> None:
            return None

    backend = Holding()
    gateway = ModelGateway(backend, Budget.of(BudgetPolicy(), MemoryLedger(), FakeClock()))
    names = ["a", "b", "c", "d", "e"]

    async def session() -> list[str]:
        tasks = [asyncio.create_task(gateway.complete(ALICE, request(name))) for name in names]
        for _ in range(5):
            await asyncio.sleep(0)
        assert started == ["a", "b"]  # c, d and e wait
        for name in names:
            backend.release.setdefault(name, asyncio.Event()).set()
            for _ in range(5):
                await asyncio.sleep(0)
        return [_describe(outcome) for outcome in await asyncio.gather(*tasks)]

    assert run(session()) == [f"completed:{name}" for name in names]
    assert started == names
    assert peak == 2


def test_a_call_past_the_timeout_fails_and_is_charged() -> None:
    # (vii): the test sets call_timeout_s = 1; the scripted backend sleeps 5 s.
    ledger = MemoryLedger()
    clock = FakeClock()
    backend = ScriptedBackend(_answer, delay_s=5)
    gateway = _gateway(backend, ledger, clock, BudgetPolicy(call_timeout_s=1))
    began = time.monotonic()
    outcome = run(gateway.complete(ALICE, request("slow")))
    assert time.monotonic() - began < 2
    assert _describe(outcome) == "failed:timeout"
    window = ledger.window(ALICE, clock.now(), 3600)
    assert (window.calls, window.tokens) == (1, 0)


def test_record_then_replay_across_a_restart_with_the_sqlite_ledger(tmp_path: Path) -> None:
    # Integration: record through the gateway, "restart" (new objects, the same files), replay; the
    # completions are identical and the ledger continues from where it was.
    cassette = tmp_path / "round-trip.jsonl"
    ledger_path = tmp_path / "budget.sqlite"
    clock = FakeClock()
    asked = [request("one"), request("two"), request("one")]

    ledger = SqliteLedger(ledger_path)
    recorder = RecordingBackend(
        ScriptedBackend(_answer), cassette, binding="local", model="test-model"
    )
    gateway = _gateway(recorder, ledger, clock)

    async def ask(target: ModelGateway) -> list[str]:
        outcomes = [_describe(await target.complete(ALICE, call)) for call in asked]
        await target.aclose()
        return outcomes

    recorded = run(ask(gateway))
    ledger.close()

    reopened = SqliteLedger(ledger_path)
    replayed = run(ask(_gateway(ReplayBackend(Cassette.load(cassette)), reopened, clock)))
    window = reopened.window(ALICE, clock.now(), 3600)
    reopened.close()
    assert replayed == recorded == ["completed:one", "completed:two", "completed:one"]
    assert (window.calls, window.tokens) == (6, 6 * 15)
