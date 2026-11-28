"""The LM seat runner over a fake server (pr-s10-p6 §§5.4, 5.8, 5.9, 5.10, 6 I-P6-6, C11; M-P6-7, M-P6-10).

Each test runs the real `SeatRunner` against a scripted connector (`scripted_seat.py`) and a real gateway over a
scripted backend. The clock and the sleep are fakes: a sleep is recorded and moves the clock, so the handback
period is asserted, not waited for. Assertions read the wire (what the runner sent), the decision log and the resume
file, never the runner's memory.
"""

from __future__ import annotations

import asyncio
import json
import time
from collections.abc import Callable, Sequence
from pathlib import Path

import pytest
from mineworld_sdk.wire.contract import Observation, PerceivedEvent, PlaceId
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityId, EntityKey, EventId, WorldInstanceId
from scripted_seat import (
    INVITE,
    OBSERVER,
    FakeClock,
    Link,
    ScriptedServer,
    observation,
    perceived,
    secret,
    spoke,
    until,
    welcome,
)
from support import completion, request, run

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    ModelBackend,
)
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.config import CognitionConfig
from mineworld_cognition.controller import seat as seat_module
from mineworld_cognition.controller.assignment import LmSeat, read_assignment
from mineworld_cognition.controller.deadline import EXPIRED, Expiry, send_locked
from mineworld_cognition.controller.homes import SeatHomes
from mineworld_cognition.controller.log import DecisionRecord, read_decisions
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.seat import SeatRunner
from mineworld_cognition.controller.triggers import detect
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.gateway import Budget, ModelGateway, Router, Tier
from mineworld_cognition.memory.config import MemoryConfig
from mineworld_cognition.memory.home import MemoryHome, SeatView
from mineworld_cognition.memory.home.errors import HolderInvariantBroken
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.memory.retention.absorb import AbsorbReport, absorb
from mineworld_cognition.memory.retention.assisted import SummaryUnavailable
from mineworld_cognition.memory.retention.bound import StoreBoundOutOfRange
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import (
    MemoryPolicyChoice,
    Retention,
    resolve,
)
from mineworld_cognition.memory.scopes.host import accounted
from mineworld_cognition.memory.scopes.stack import MemoryStack
from mineworld_cognition.record import CassetteMiss

ALICE = EntityKey("alice")
WALL_NOW = 1_700_000_000
NOTHING = json.dumps({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
REFUSING = BackendFailure(reason="unauthorized")
"""A non-retryable transport failure: the consult falls back at once (§5.5.1)."""
INSTANCE = WorldInstanceId("1a2b3c4d5e6f70819293a4b5c6d7e8f9")


class _WallClock:
    def now(self) -> int:
        return WALL_NOW


class ScriptedAnswers:
    """A backend that answers each request with `answer(request)`; with `gate`, its first call waits for it."""

    def __init__(
        self,
        answer: Callable[[CompletionRequest], Completion | BackendFailure],
        gate: asyncio.Event | None = None,
    ) -> None:
        self._answer = answer
        self._gate = gate
        self.requests: list[CompletionRequest] = []

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.requests.append(request)
        if self._gate is not None:
            gate, self._gate = self._gate, None
            await gate.wait()
        return self._answer(request)

    async def aclose(self) -> None:
        return None


def _nothing(request: CompletionRequest) -> Completion:
    return completion(NOTHING)


def _config() -> CognitionConfig:
    return CognitionConfig.model_validate(
        {
            "recording": {"mode": "scripted"},
            "tiers": {"social": "local"},
            "backends": {
                "local": {
                    "kind": "openai-compatible",
                    "base_url": "http://127.0.0.1:11434/v1",
                    "model": "fake-model",
                    "structured_output": "none",
                }
            },
            "controller": {"handback_s": 10, "consult_deadline_s": 20},
        }
    )


def _runner(
    tmp_path: Path,
    server: ScriptedServer,
    clock: FakeClock,
    backend: ModelBackend,
    key: EntityKey = ALICE,
) -> SeatRunner:
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), _WallClock())
    router = Router({"social": ModelGateway(backend, budget)}, budget)
    return SeatRunner(
        _config(),
        LmSeat(key=key, memory=None),
        router=router,
        connector=server,
        invite=Invite(INVITE),
        nickname="cognition",
        store_dir=tmp_path / "store",
        homes=SeatHomes(tmp_path / "store", MemoryConfig()),
        clock=clock.time,
        sleep=clock.sleep,
    )


def _decisions(tmp_path: Path) -> list[DecisionRecord]:
    path = tmp_path / "store" / "alice.decisions.jsonl"
    return read_decisions(path) if path.exists() else []


async def _logged(tmp_path: Path, count: int) -> None:
    await until(lambda: len(_decisions(tmp_path)) >= count, f"{count} logged consults")


async def _cancel(task: asyncio.Task[None]) -> None:
    task.cancel()
    await asyncio.gather(task, return_exceptions=True)


async def _settle(rounds: int = 200) -> None:
    for _ in range(rounds):
        await asyncio.sleep(0)


def test_a_spoken_line_is_a_trigger_and_the_consult_chooses_nothing(tmp_path: Path) -> None:
    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(link), FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await _logged(tmp_path, 1)
        await _cancel(task)

    run(scenario())
    (record,) = _decisions(tmp_path)
    assert record.input.trigger.kind == "spoken_to"
    assert record.input.trigger.speaker == "5"
    assert record.record.outcome == "chose"
    assert len(record.record.attempts) == 1


def test_recall_includes_the_lines_said_after_the_observation_the_consult_reads(
    tmp_path: Path,
) -> None:
    # fix/recall-at-trigger: the seat's newest observation trails the facts it absorbed (D-P6b-30); at 12x one
    # 100 ms tick is 1.2 world seconds. Here the observation is at world second 1, the visitor's earlier line at
    # 10 and the line that triggers the consult at 12. Recall must be made at the trigger's instant, so the earlier
    # line is in the consult's memory. Its mutation (recall at `observation.at`) leaves the memory empty.
    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1, at=1))
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(link), FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(
            perceived(
                "3",
                spoke("2", speaker="5", utterance="my umbrella is red", at=10),
                spoke("3", speaker="5", utterance="what colour was it?", at=12),
            )
        )
        await until(
            lambda: any(record.input.trigger.event == "3" for record in _decisions(tmp_path)),
            "the consult on the second line",
        )
        await _cancel(task)

    run(scenario())
    record = next(record for record in _decisions(tmp_path) if record.input.trigger.event == "3")
    assert record.input.observation.at == 1
    # P6c (pr-p6c-fgd §4.1, §9 D-P6c-4): a line the speaker said to me is shown under `with`, and L0 leaves out
    # what `with` shows, so the earlier line is read from either section; recall at `observation.at` drops it from
    # both.
    recalled = [line for line in record.input.memory.lines if line.level in ("with", "L0")]
    assert [line.citation.ranges[0].first for line in recalled if "umbrella" in line.text] == ["2"]


def test_a_restarted_process_does_not_answer_again_a_line_its_store_already_held(
    tmp_path: Path,
) -> None:
    # S10 P7 F-P7-3: a process's first join presents no cursor (D-P6b-15), so the server sends the history again.
    # The line the first process heard is redelivered; only the line said since is a trigger. Its mutation (detect
    # over every redelivered fact) consults on "2" a second time.
    async def scenario() -> None:
        first = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(first), FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in first.kinds(), "the first process's join")
        first.say(perceived("2", spoke("2", speaker="5")))
        await _logged(tmp_path, 1)
        await _cancel(task)

        again = Link(welcome(resume=2), observation(1))
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(again), FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in again.kinds(), "the restarted process's join")
        again.say(perceived("2", spoke("2", speaker="5")))  # the backfill: the old line alone
        await _settle()
        again.say(perceived("3", spoke("3", speaker="5", utterance="again")))
        await _logged(tmp_path, 2)
        await _settle()
        await _cancel(task)

    run(scenario())
    assert [record.input.trigger.event for record in _decisions(tmp_path)] == ["2", "3"]


def test_absorb_runs_to_its_end_before_triggers_are_detected(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    calls: list[str] = []
    real_absorb = absorb
    real_detect = detect

    async def recording_absorb(
        store: SeatView, facts: Sequence[PerceivedEvent], through: EventId, retention: Retention
    ) -> AbsorbReport:
        calls.append("absorb:start")
        report = await real_absorb(store, facts, through, retention)
        calls.append("absorb:end")
        return report

    def recording_detect(
        facts: Sequence[PerceivedEvent],
        me: EntityId,
        store: SeatView,
        here: PlaceId | None,
        now_at: int,
        greeted: dict[EntityId, int],
        cooldown: int,
    ) -> list[Trigger]:
        calls.append("detect")
        return real_detect(facts, me, store, here, now_at, greeted, cooldown)

    monkeypatch.setattr(seat_module, "absorb", recording_absorb)
    monkeypatch.setattr(seat_module, "detect", recording_detect)

    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(link), FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await _logged(tmp_path, 1)
        await _cancel(task)

    run(scenario())
    assert calls[:3] == ["absorb:start", "absorb:end", "detect"]


def test_a_trigger_that_arrives_during_a_consult_runs_once_afterwards(tmp_path: Path) -> None:
    async def scenario() -> None:
        gate = asyncio.Event()
        link = Link(welcome(resume=1), observation(1))
        backend = ScriptedAnswers(_nothing, gate)
        task = asyncio.create_task(
            _runner(tmp_path, ScriptedServer(link), FakeClock(), backend).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await until(lambda: len(backend.requests) == 1, "the first consult is asked")
        link.say(perceived("3", spoke("3", speaker="5", utterance="again")))
        await _settle()
        gate.set()
        await _logged(tmp_path, 2)
        await _cancel(task)

    run(scenario())
    records = _decisions(tmp_path)
    assert [record.consult for record in records] == [1, 2]
    assert records[-1].input.trigger.event == "3"


def test_a_fallback_at_a_hosted_seat_hands_it_back_and_rejoins_after_the_handback_period(
    tmp_path: Path,
) -> None:
    async def scenario() -> tuple[list[str], list[float], int]:
        first = Link(welcome(resume=1, took_over="hosted"), observation(1))
        second = Link(welcome(resume=2, took_over="hosted"), observation(2))
        server = ScriptedServer(first, second)
        clock = FakeClock()
        task = asyncio.create_task(
            _runner(tmp_path, server, clock, ScriptedAnswers(lambda request: REFUSING)).run()
        )
        await until(lambda: "join" in first.kinds(), "the first join")
        first.say(perceived("2", spoke("2", speaker="5")))
        await until(lambda: len(server.links) == 2, "the rejoin")
        await until(lambda: len(second.joins()) == 1, "the second join")
        await _cancel(task)
        return first.kinds(), clock.sleeps, len(second.joins())

    kinds, sleeps, joins = run(scenario())
    assert kinds[-1] == "leave"
    assert 10.0 in sleeps
    assert joins == 1
    assert [record.record.outcome for record in _decisions(tmp_path)] == ["fell_back"]


def test_a_fallback_at_a_seat_that_is_not_hosted_stands_and_sends_nothing(tmp_path: Path) -> None:
    async def scenario() -> list[str]:
        link = Link(welcome(resume=1, took_over="none"), observation(1))
        task = asyncio.create_task(
            _runner(
                tmp_path,
                ScriptedServer(link),
                FakeClock(),
                ScriptedAnswers(lambda request: REFUSING),
            ).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await _logged(tmp_path, 1)
        kinds = link.kinds()  # read while the seat stands: a shutdown's leave (§5.16) comes after
        await _cancel(task)
        return kinds

    kinds = run(scenario())
    assert "leave" not in kinds
    (record,) = _decisions(tmp_path)
    assert record.record.outcome == "fell_back"


def test_the_resume_file_is_rewritten_when_the_connection_number_grows(tmp_path: Path) -> None:
    resume_path = tmp_path / "store" / "alice.resume"

    async def scenario() -> str:
        first = Link(welcome(resume=1, took_over="none"), observation(1))
        second = Link(welcome(resume=2, took_over="none"), observation(2))
        server = ScriptedServer(first, second)
        task = asyncio.create_task(
            _runner(tmp_path, server, FakeClock(), ScriptedAnswers(_nothing)).run()
        )
        await until(
            lambda: resume_path.exists() and resume_path.read_text() == secret(1), "secret 1"
        )
        first.drop()
        await until(lambda: len(server.links) == 2, "the rejoin")
        await until(
            lambda: resume_path.exists() and resume_path.read_text() == secret(2), "secret 2"
        )
        written = resume_path.read_text()
        await _cancel(task)
        return written

    assert run(scenario()) == secret(2)


def test_a_leave_deletes_the_resume_file_and_nothing_writes_it_back_while_the_seat_is_away(
    tmp_path: Path,
) -> None:
    resume_path = tmp_path / "store" / "alice.resume"

    async def scenario() -> bool:
        link = Link(welcome(resume=1, took_over="hosted"), observation(1))
        server = ScriptedServer(link)
        task = asyncio.create_task(
            _runner(tmp_path, server, FakeClock(), ScriptedAnswers(lambda request: REFUSING)).run()
        )
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await until(lambda: "leave" in link.kinds(), "the handback")
        await until(lambda: len(server.links) == 1 and not resume_path.exists(), "the deletion")
        await _settle()
        present = resume_path.exists()
        await _cancel(task)
        return present

    assert run(scenario()) is False


def test_a_seat_that_is_occupied_is_retried_at_seat_retry_s(tmp_path: Path) -> None:
    async def scenario() -> FakeClock:
        occupied = Link('{"t":"refused","code":"seat_occupied"}')
        busy = Link(welcome(resume=1))
        server = ScriptedServer(occupied, busy)
        clock = FakeClock()
        task = asyncio.create_task(
            _runner(tmp_path, server, clock, ScriptedAnswers(_nothing)).run()
        )
        await until(lambda: "join" in busy.kinds(), "the retry")
        await _cancel(task)
        return clock

    clock = run(scenario())
    assert 60.0 in clock.sleeps


def test_no_secret_reaches_the_decision_log_the_store_or_a_repr(tmp_path: Path) -> None:
    planted = secret(0xC0FFEE)

    async def scenario() -> tuple[SeatRunner, str]:
        link = Link(welcome(resume=0xC0FFEE, took_over="none"), observation(1))
        runner = _runner(tmp_path, ScriptedServer(link), FakeClock(), ScriptedAnswers(_nothing))
        task = asyncio.create_task(runner.run())
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await _logged(tmp_path, 1)
        written = (tmp_path / "store" / "alice.resume").read_text()
        await _cancel(task)
        return runner, written

    runner, written = run(scenario())
    assert planted in written
    assert not (tmp_path / "store" / "alice.resume").exists(), (
        "a shutdown deletes the secret (§5.16)"
    )
    files = [path for path in (tmp_path / "store").rglob("*") if path.is_file()]
    assert any(path.name == "memory.sqlite" for path in files), "the memory home is scanned too"
    for path in files:
        assert planted.encode() not in path.read_bytes(), path.name
    assert planted not in repr(runner)


def test_an_lm_seat_the_welcome_does_not_list_is_refused_by_name(tmp_path: Path) -> None:
    """D-P6b-21: at the welcome, the runner's own seat must be one of `welcome.world.seats`; a key that is not
    refuses to run, with the seat named, and the seat is detached rather than held."""
    link = Link(welcome(resume=1, took_over="none"), observation(1))
    runner = _runner(
        tmp_path,
        ScriptedServer(link),
        FakeClock(),
        ScriptedAnswers(_nothing),
        key=EntityKey("bob"),
    )
    with pytest.raises(ConfigError, match="seat bob: the decision file names an lm seat"):
        run(runner.run())
    assert "leave" not in link.kinds()


def _arrival(event: str) -> PerceivedEvent:
    return PerceivedEvent.model_validate(
        {
            "id": event,
            "at": 4,
            "event_type": "person-entered-place",
            "subjects": ["5"],
            "participants": ["5"],
            "place": {"entity": "3", "entity_type": "place"},
            "caused_by": {"action": "41"},
            "payload": {
                "event_type": "person-entered-place",
                "schema_version": 1,
                "payload": {
                    "person": {"entity": "5", "entity_type": "person"},
                    "place": {"entity": "3", "entity_type": "place"},
                    "from": {"entity": "4", "entity_type": "place"},
                },
            },
            "visibility": {"place": {"entity": "3", "entity_type": "place"}},
            "provenance": {"emitted_by": "presence", "controller_decision": None},
        }
    )


def test_a_remembered_person_entering_my_place_triggers_once_per_cooldown(tmp_path: Path) -> None:
    identity = StoreIdentity(world_instance=INSTANCE, seat=ALICE, self_entity=EntityId(OBSERVER))
    home = MemoryHome.open(tmp_path / "memory.sqlite", INSTANCE)
    store = home.seat(identity)
    store.connection.execute(
        "INSERT INTO stable (seat, entity, first_key, last_key, times_met, exchanges, level, "
        "level_key, first_at, last_at) VALUES (?, '5', '', '', 3, 3, 'L3', '', 0, 0)",
        (store.seat_id,),
    )
    here = PlaceId.model_validate({"entity": "3", "entity_type": "place"})
    greeted: dict[EntityId, int] = {}
    counts: list[int] = []
    for event, at in (("10", 100), ("11", 200), ("12", 2_000)):
        found = detect([_arrival(event)], EntityId(OBSERVER), store, here, at, greeted, 1_800)
        counts.append(len(found))
        for trigger in found:
            assert trigger.speaker is not None
            greeted[trigger.speaker] = at
    home.close()
    assert counts == [1, 0, 1]


class _MissingRecording:
    """A `summarize` backend whose replay holds nothing: every call is a cassette miss."""

    def __init__(self) -> None:
        self.requests: list[CompletionRequest] = []

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.requests.append(request)
        raise CassetteMiss("absent", cassette_key(request))

    async def aclose(self) -> None:
        return None


def _gateway(backend: ScriptedAnswers | _MissingRecording) -> ModelGateway:
    return ModelGateway(backend, Budget.of(BudgetPolicy(), MemoryLedger(), _WallClock()))


def _social() -> ModelGateway:
    return _gateway(ScriptedAnswers(_nothing))


def _assisted_gateway(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, tiers: dict[Tier, ModelGateway | None]
) -> ModelGateway | None:
    """Builds a runner whose seat chose `lm-summarise`, and returns the gateway it gave FG-b's `resolve`."""
    given: list[ModelGateway | None] = []
    real_resolve = resolve

    def recording_resolve(
        choice: MemoryPolicyChoice,
        registry: PolicyRegistry,
        gateway: ModelGateway | None = None,
        *,
        seat: EntityKey | None = None,
    ) -> Retention:
        given.append(gateway)
        return real_resolve(choice, registry, gateway, seat=seat)

    monkeypatch.setattr(seat_module, "resolve", recording_resolve)
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), _WallClock())
    SeatRunner(
        _config(),
        LmSeat(key=ALICE, memory=MemoryPolicyChoice(policy="lm-summarise")),
        router=Router(tiers, budget),
        connector=ScriptedServer(Link(welcome(resume=1), observation(1))),
        invite=Invite(INVITE),
        nickname="cognition",
        store_dir=tmp_path / "store",
        homes=SeatHomes(tmp_path / "store", MemoryConfig()),
        clock=FakeClock().time,
        sleep=FakeClock().sleep,
    )
    (gateway,) = given
    return gateway


def test_an_assisted_seat_asks_the_summarize_tier_through_the_summary_boundary(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """R-P6-FGc-1, D-P6b-36: the seat's assisted policy gets the router's `summarize` model, seen through
    FG-c's boundary: a replay's missing recording is `SummaryUnavailable`, chained to the miss (a plain
    gateway would let `CassetteMiss` through)."""
    missing = _MissingRecording()
    gateway = _assisted_gateway(
        tmp_path, monkeypatch, {"social": _social(), "summarize": _gateway(missing)}
    )
    assert gateway is not None
    with pytest.raises(SummaryUnavailable) as raised:
        run(gateway.complete(ALICE, request("a group to summarise")))
    assert isinstance(raised.value.__cause__, CassetteMiss)
    assert len(missing.requests) == 1  # the router's `summarize` backend, not the social one


def test_an_assisted_seat_with_no_summarize_model_resolves_to_its_no_model_fallback(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    assert (
        _assisted_gateway(tmp_path, monkeypatch, {"social": _social(), "summarize": None}) is None
    )


FORMAT_3 = Path(__file__).parent / "fixtures" / "decisions_format3.json"


def _retention_for(tmp_path: Path, monkeypatch: pytest.MonkeyPatch, seat: LmSeat) -> Retention:
    """Builds a runner for `seat` and returns the `Retention` FG-b's `resolve` gave it (what `absorb` runs)."""
    given: list[Retention] = []
    real_resolve = resolve

    def recording_resolve(
        choice: MemoryPolicyChoice,
        registry: PolicyRegistry,
        gateway: ModelGateway | None = None,
        *,
        seat: EntityKey | None = None,
    ) -> Retention:
        resolved = real_resolve(choice, registry, gateway, seat=seat)
        given.append(resolved)
        return resolved

    monkeypatch.setattr(seat_module, "resolve", recording_resolve)
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), _WallClock())
    SeatRunner(
        _config(),
        seat,
        router=Router({"social": _social()}, budget),
        connector=ScriptedServer(Link(welcome(resume=1), observation(1))),
        invite=Invite(INVITE),
        nickname="cognition",
        store_dir=tmp_path / "store",
        homes=SeatHomes(tmp_path / "store", MemoryConfig()),
        clock=FakeClock().time,
        sleep=FakeClock().sleep,
    )
    (retention,) = given
    return retention


def test_a_seats_configured_store_bound_reaches_its_retention_host(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """D-P6b-39: format 2's `store_bound_mib: 40` read from the decision JSON is the bound, target and size
    trigger `absorb` and the retention host run with (FG-b2 §4.1), not the 20 MiB default."""
    (alice,) = read_assignment(FORMAT_3, None).seats
    bounds = _retention_for(tmp_path, monkeypatch, alice).bounds
    assert (bounds.bound, bounds.target, bounds.trigger) == (41_943_040, 31_457_280, 39_321_600)


def test_a_seat_without_a_memory_choice_keeps_the_default_store_bound(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    bounds = _retention_for(tmp_path, monkeypatch, LmSeat(key=ALICE, memory=None)).bounds
    assert bounds.bound == 20 * 1024 * 1024


def test_a_seat_whose_store_bound_is_below_the_floor_is_refused_before_it_joins(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    choice = MemoryPolicyChoice(policy="consolidate", store_bound_mib=19)
    with pytest.raises(StoreBoundOutOfRange, match="19"):
        _retention_for(tmp_path, monkeypatch, LmSeat(key=ALICE, memory=choice))


def _verifying_runner(tmp_path: Path, server: ScriptedServer) -> SeatRunner:
    """A runner under `[memory] verify = "pass"` (pr-s28f §5.5)."""
    config = _config().model_copy(update={"memory": MemoryConfig(verify="pass")})
    budget = Budget.of(BudgetPolicy(), MemoryLedger(), _WallClock())
    clock = FakeClock()
    return SeatRunner(
        config,
        LmSeat(key=ALICE, memory=None),
        router=Router({"social": ModelGateway(ScriptedAnswers(_nothing), budget)}, budget),
        connector=server,
        invite=Invite(INVITE),
        nickname="cognition",
        store_dir=tmp_path / "store",
        homes=SeatHomes(tmp_path / "store", config.memory),
        clock=clock.time,
        sleep=clock.sleep,
    )


def _across_a_day(link: Link) -> None:
    """Two lines, the second on the next simulated day: its `absorb` runs one retention pass."""
    link.say(perceived("2", spoke("2", speaker="5", at=40)))
    link.say(perceived("3", spoke("3", speaker="5", utterance="next day", at=86_400 + 40)))


def _pass_lines(tmp_path: Path) -> list[dict[str, object]]:
    path = tmp_path / "store" / "alice.decisions.jsonl"
    if not path.exists():
        return []
    loaded = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines() if line]
    return [line for line in loaded if line["kind"] == "pass"]


def test_under_verify_a_seat_whose_scopes_account_for_its_content_runs_its_pass(
    tmp_path: Path,
) -> None:
    # R-S28f-c-1: the bundled stack (person, world) accounts for every content byte, so the check holds.
    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(_verifying_runner(tmp_path, ScriptedServer(link)).run())
        await until(lambda: "join" in link.kinds(), "the join")
        _across_a_day(link)
        await until(lambda: bool(_pass_lines(tmp_path)), "the pass line")
        await _settle()
        assert not task.done(), "the check held and the seat serves on"
        await _cancel(task)

    run(scenario())
    (line,) = _pass_lines(tmp_path)
    assert (line["seat"], line["day"], line["policy"]) == ("alice", 1, "consolidate")


def test_under_verify_a_scope_account_that_under_reports_stops_the_seat_by_name(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # R-S28f-c-1, F-S28c-16: a stack whose accounts miss one byte of the seat's content is a defect.
    def under_reported(view: SeatView, stack: MemoryStack) -> int:
        return accounted(view, stack) - 1

    monkeypatch.setattr(seat_module, "accounted", under_reported)

    async def scenario() -> None:
        link = Link(welcome(resume=1), observation(1))
        task = asyncio.create_task(_verifying_runner(tmp_path, ScriptedServer(link)).run())
        await until(lambda: "join" in link.kinds(), "the join")
        _across_a_day(link)
        with pytest.raises(HolderInvariantBroken) as raised:
            await task
        assert raised.value.seat == "alice"

    run(scenario())


# ── The absolute consult deadline (DM-20261127-3 C1, §§4.4, 4.5) ─────────────────────────────────────────


def _stale_triggers(tmp_path: Path) -> int:
    """The runner's `stale_triggers` counter from the last counters line of the decision log."""
    lines = (tmp_path / "store" / "alice.decisions.jsonl").read_text(encoding="utf-8").splitlines()
    counters = [json.loads(line) for line in lines if '"kind":"counters"' in line]
    return int(counters[-1]["stale_triggers"])


def test_a_trigger_whose_deadline_passes_before_the_first_observation_is_discarded_unasked(
    tmp_path: Path,
) -> None:
    # `consult_deadline_s` runs from the trigger's receipt (cognition-toml.md). A line heard before the seat's
    # first observation, which arrives 25 s later, is past its 20 s deadline: it is counted stale and never sent
    # to the model. Before the fix the consult began after the observation with a fresh 20 s.
    async def scenario() -> ScriptedAnswers:
        clock = FakeClock()
        link = Link(welcome(resume=1))
        backend = ScriptedAnswers(_nothing)
        task = asyncio.create_task(_runner(tmp_path, ScriptedServer(link), clock, backend).run())
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await _settle()
        clock.now += 25.0
        link.say(observation(1))
        await _settle()
        await _cancel(task)
        return backend

    backend = run(scenario())
    assert backend.requests == []
    assert _decisions(tmp_path) == []
    assert _stale_triggers(tmp_path) == 1


class _EachHeld:
    """A backend whose every call waits for its own release; `started` counts the calls that reached it."""

    def __init__(self) -> None:
        self.started = 0
        self.release: list[asyncio.Event] = []

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.started += 1
        self.release.append(asyncio.Event())
        await self.release[-1].wait()
        return completion(NOTHING)

    async def aclose(self) -> None:
        return None


def test_the_pending_wait_spends_the_deadline_and_a_late_answer_is_not_acted_on(
    tmp_path: Path,
) -> None:
    # Line 3 arrives while line 2's consult runs; that consult takes 15 s, so line 3 has 5 s left when its own
    # consult starts. Its answer comes 6 s later: past the expiry, the answer is not acted on and the consult
    # ends `fell_back` at the deadline. Before the fix line 3's consult started a fresh 20 s and chose.
    async def scenario() -> None:
        clock = FakeClock()
        link = Link(welcome(resume=1), observation(1))
        backend = _EachHeld()
        task = asyncio.create_task(_runner(tmp_path, ScriptedServer(link), clock, backend).run())
        await until(lambda: "join" in link.kinds(), "the join")
        link.say(perceived("2", spoke("2", speaker="5")))
        await until(lambda: backend.started == 1, "line 2 is asked")
        link.say(perceived("3", spoke("3", speaker="5", utterance="again")))
        await _settle()
        clock.now += 15.0
        backend.release[0].set()
        await until(lambda: backend.started == 2, "line 3 is asked")
        clock.now += 6.0
        backend.release[1].set()
        await _logged(tmp_path, 2)
        await _cancel(task)

    run(scenario())
    first, second = _decisions(tmp_path)
    assert first.record.outcome == "chose"
    assert second.input.trigger.event == "3"
    assert (second.record.outcome, second.record.cause) == ("fell_back", "deadline")


def test_a_submit_waiting_for_the_lock_past_its_expiry_is_never_sent() -> None:
    # §4.4: the submit lock's wait spends the deadline, and the expiry is checked again once the lock is held.
    # (a) the pacer holds the lock while the fake clock passes the expiry; (b) the lock is held longer than what is
    # left, on the real clock: the wait ends at the expiry, and nothing is sent in either case.
    sent: list[str] = []

    async def send(request: str) -> str:
        sent.append(request)
        return request

    async def scenario() -> tuple[object, object, object]:
        lock = asyncio.Lock()
        clock = FakeClock()
        await lock.acquire()
        waiting = asyncio.create_task(
            send_locked(lock, Expiry.after(clock.now, 20), clock.time, send, "late")
        )
        await _settle(5)
        clock.now += 20.0
        lock.release()
        held_past = await waiting
        await lock.acquire()
        bounded = await send_locked(
            lock, Expiry.after(time.monotonic(), 0.05), time.monotonic, send, "x"
        )
        lock.release()
        in_time = await send_locked(lock, Expiry.after(clock.now, 20), clock.time, send, "sent")
        return held_past, bounded, in_time

    assert run(scenario()) == (EXPIRED, EXPIRED, "sent")
    assert sent == ["sent"]


def test_a_waiting_persons_words_are_never_displaced_by_another_kind_of_trigger() -> None:
    """PR-B red-team #565: during the live gate a consult fell back while the player's line waited, and the next
    `arrived` overwrote it, so the line was never answered and no counter said so. A waiting `spoken_to` now stays;
    the other trigger is dropped; a newer `spoken_to` still supersedes an older one; every loss is counted."""
    from mineworld_cognition.controller.triggers import Pending

    said = Trigger(
        event=EventId("338"), kind="spoken_to", speaker=EntityId("20"), words="My code is NWP-2641."
    )
    arrived = Trigger(event=EventId("463"), kind="arrived", speaker=EntityId("7"), words="")
    said_again = Trigger(
        event=EventId("470"), kind="spoken_to", speaker=EntityId("20"), words="Hello?"
    )
    bob_says = Trigger(
        event=EventId("480"), kind="spoken_to", speaker=EntityId("7"), words="Morning!"
    )

    async def scenario() -> list[Trigger]:
        pending = Pending()
        taken: list[Trigger] = []
        pending.offer(said, 1, 0.0)
        pending.offer(arrived, 2, 1.0)
        taken.append((await pending.take()).trigger)
        assert pending.superseded == 1, "the dropped arrival is counted"
        pending.offer(arrived, 3, 2.0)
        pending.offer(said, 4, 3.0)
        pending.offer(said_again, 5, 4.0)
        pending.offer(bob_says, 6, 5.0)
        taken.append((await pending.take()).trigger)
        assert pending.superseded == 4, (
            "a displaced arrival, a superseded line and Bob's dropped line"
        )
        return taken

    assert run(scenario()) == [said, said_again]


def test_a_greeting_nobody_can_hear_waits_and_is_offered_when_the_person_comes_within_range() -> (
    None
):
    """DM-20261128-4 A4 (operator, 2026-11-27: "跳过没人听见的打招呼"): an arrival of someone I may not talk to yet
    costs no consult and is not marked greeted; once the world offers `talk` to that person it is offered and
    marked (red-team #565). The range is the server's: only an available `talk` affordance counts."""
    from mineworld_cognition.controller.triggers import Greetings, Pending

    vera, bob = EntityId("20"), EntityId("7")
    vera_arrived = Trigger(event=EventId("338"), kind="arrived", speaker=vera, words="")
    greetings, pending = Greetings(), Pending()
    greetings.arrived(vera_arrived)

    def talk_to(*people: EntityId, available: bool = True) -> Observation:
        return Observation.model_validate(
            {
                "observer": "1",
                "at": 100,
                "self_location": None,
                "entities": [],
                "relations": [],
                "events": [],
                "affordances": [
                    {
                        "action_type": "talk",
                        "target": person,
                        "available": available,
                        "unavailable_reason": None if available else "too_far_away",
                        "requirement": {
                            "place": "any",
                            "within_range": None,
                            "requires_line_of_access": False,
                            "requires_target_available": True,
                        },
                    }
                    for person in people
                ],
            }
        )

    greetings.offer(talk_to(bob), pending, 1.0)
    greetings.offer(talk_to(vera, available=False), pending, 2.0)
    assert vera not in greetings.greeted, "a greeting nobody could hear is not marked done"

    async def scenario() -> None:
        async def take() -> Trigger:
            return (await asyncio.wait_for(pending.take(), 0.5)).trigger

        with pytest.raises(TimeoutError):
            await take()
        greetings.offer(talk_to(vera), pending, 3.0)
        assert await take() == vera_arrived
        assert greetings.greeted[vera] == 100
        greetings.offer(talk_to(vera), pending, 4.0)
        with pytest.raises(TimeoutError):
            await take()

    run(scenario())
