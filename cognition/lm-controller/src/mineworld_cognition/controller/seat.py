"""The LM seat runner: one seat's lifecycle and its state machine (pr-s10-p6 §§5.8, 5.9, 5.10; C11).

A `SeatRunner` joins one `lm` seat of the decision file as a protocol client (`ARC-56`) and keeps it. Three tasks
serve a joined seat: `_ingest` absorbs every perceived batch into the store and only then detects triggers (I-P6-6);
`_watch` keeps the newest observation and rewrites the resume secret when the connection number grows; `_consults`
runs one consult at a time on the pending trigger. The runner never writes world state and never builds a backend:
every model call goes through the router's social gateway, every submit through the seat.

Handback (§5.8): a consult that fell back, a budget refusal, a disconnect, or an open breaker hands the seat back by
`leave`, and only when the welcome said `took_over: "hosted"`; otherwise the seat stands and sends nothing for that
consult (F-P6-3). A handback deletes the resume secret, waits `handback_s` (or the breaker's cooldown, if longer), and
rejoins with the store's cursor. A killed process sends no `leave` (F-P6-4): it rejoins inside the hold with its
persisted secret (§5.10).
"""

from __future__ import annotations

import asyncio
from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import Literal

from mineworld_sdk.reconnect import ReconnectPolicy, SeatLost
from mineworld_sdk.resuming import Connector, ResumingSeat, Seen
from mineworld_sdk.session import TRANSPORT_FAILURES, Answered, RefusedRequest
from mineworld_sdk.wire.contract import (
    Accepted,
    ActionRequest,
    Observation,
    PerceivedEvent,
    PlaceId,
)
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityId, EventId

from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.config import (
    CognitionConfig,
    ControllerConfig,
    Layer,
    LayerConfig,
    controller_for,
    layer_policy,
)
from mineworld_cognition.controller import resume_file
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.breaker import BreakerRegistry, CircuitBreaker
from mineworld_cognition.controller.consult import (
    ConsultDeps,
    ConsultInput,
    ConsultLimits,
    ConsultRecord,
    LayerPolicies,
    NewestState,
    breaker_outcome,
    consult,
)
from mineworld_cognition.controller.log import (
    Counters,
    DecisionInput,
    DecisionLog,
    DecisionRecord,
    counters_from,
)
from mineworld_cognition.controller.memory_view import memory_view
from mineworld_cognition.controller.persona import Persona
from mineworld_cognition.controller.prompt import Trigger
from mineworld_cognition.controller.retry import BoundRetry, RetryRegistry
from mineworld_cognition.controller.situation import Situation
from mineworld_cognition.controller.triggers import Pending, body_of, detect, person_of
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.gateway import GateOutcome, Router
from mineworld_cognition.memory.names import NameBook
from mineworld_cognition.memory.records import StoreIdentity, event_key
from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import DEFAULT_CHOICE, Retention, resolve
from mineworld_cognition.memory.retrieve import RecallQuery
from mineworld_cognition.memory.store import MemoryStore
from mineworld_cognition.summary_boundary import SummaryBoundaryGateway

type Clock = Callable[[], float]
"""Monotonic wall seconds: the consult deadline, the breaker's cooldown and the handback wait."""
type Sleep = Callable[[float], Awaitable[None]]

FALLBACK_OUTCOMES = frozenset({"fell_back", "budget", "disconnected"})
"""The consult outcomes that hand the seat back (§5.8)."""
SEAT_CHANGED = frozenset({"taken_over", "seat_occupied"})
"""The losses after which the runner waits `seat_retry_s` and joins again (§5.8)."""
SPOKE = "spoke"
STOP_LEAVE_S = 5
"""The bound of a shutdown's `leave` (§5.16: every seat leaves within 5 s on Ctrl-C)."""


class _StoreCursor:
    """The cursor a join presents: the store's, once the store is open (the first join of a process has none,
    because the store's identity is known only from its first welcome; D-P6b-15)."""

    def __init__(self) -> None:
        self.store: MemoryStore | None = None

    def cursor(self) -> EventId | None:
        return None if self.store is None else self.store.cursor()


class SeatRunner:
    """Joins one `lm` seat and drives it until it is lost. `run()` returns when the seat ends for us, and raises the
    `SeatLost` that ends it for good."""

    def __init__(
        self,
        config: CognitionConfig,
        seat: LmSeat,
        *,
        router: Router,
        connector: Connector,
        invite: Invite,
        nickname: str,
        store_dir: Path,
        clock: Clock,
        sleep: Sleep,
    ) -> None:
        key = str(seat.key)
        social = config.tiers.get("social")
        gateway = router.for_tier("social")
        if social is None or gateway is None:
            raise ConfigError("tiers.social: no model is bound to the social tier")
        self._key = seat.key
        self._controller: ControllerConfig = controller_for(config, key)
        self._gateway = gateway
        retry = RetryRegistry.installed()
        self._policies = LayerPolicies(
            transport=_bind(retry, config, key, social, "transport"),
            content=_bind(retry, config, key, social, "content"),
            semantic=_bind(retry, config, key, social, "semantic"),
        )
        breaker_config = config.breaker or LayerConfig(policy="failure-rate", parameters={})
        self._breaker: CircuitBreaker = (
            BreakerRegistry.installed()
            .bind(breaker_config.policy, dict(breaker_config.parameters), "breaker")
            .create()
        )
        # R-P6-FGc-1 (D-P6b-36): an assisted policy asks the `summarize` tier through FG-c's boundary, so a
        # replay's missing recording stops the pass as `SummaryUnavailable`. No `summarize` model: the
        # policy's no-model fallback decides every group. A plain policy ignores the gateway.
        summarize = router.for_tier("summarize")
        self._retention: Retention = resolve(
            seat.memory or DEFAULT_CHOICE,
            PolicyRegistry.installed(),
            None if summarize is None else summarize.sharing(SummaryBoundaryGateway),
        )
        self._connector = connector
        self._invite = invite
        self._nickname = nickname
        self._clock = clock
        self._sleep = sleep
        self._store_dir = store_dir
        self._store_path = store_dir / f"{key}.sqlite"
        self._resume_path = store_dir / f"{key}.resume"
        self._log = DecisionLog(store_dir / f"{key}.decisions.jsonl")
        self._cursor = _StoreCursor()
        self._store: MemoryStore | None = None
        self._pending = Pending()
        self._newest: Seen | None = None
        self._observed = asyncio.Event()
        self._written: int | None = None
        self._greeted: dict[EntityId, int] = {}
        self._last_line: dict[EntityId, EventId] = {}
        self._consult_count = 0
        self._records: list[DecisionRecord] = []
        self._runner_counts = {"breaker_trips": 0, "stale_triggers": 0, "handbacks": 0}

    async def run(self) -> None:
        try:
            while await self._join_and_serve() == "handback":
                await self._sleep(self._handback_wait())
        finally:
            self._close()

    # ── Joining ───────────────────────────────────────────────────────────────────────────────────

    async def _join_and_serve(self) -> Literal["handback", "ended"]:
        while True:
            try:
                seat = await self._open()
            except SeatLost as lost:
                if lost.reason not in SEAT_CHANGED:
                    raise
                resume_file.delete(self._resume_path)
                await self._sleep(float(self._controller.seat_retry_s))
                continue
            break
        if self._key not in seat.welcome.world.seats:
            await seat.detach()
            raise ConfigError(
                f"seat {self._key}: the decision file names an lm seat the world does not have "
                "(the welcome's seats; D-P6b-21)"
            )
        self._open_store(seat)
        self._persist(seat)
        return await self._serve(seat)

    async def _open(self) -> ResumingSeat:
        """One join, retried while the socket cannot be opened, with delays capped at `seat_retry_s`."""
        delay = 1.0
        while True:
            try:
                return await ResumingSeat.open(
                    self._connector,
                    seat=self._key,
                    invite=self._invite,
                    nickname=self._nickname,
                    cursor=self._cursor,
                    take_over=False,
                    resume=resume_file.read(self._resume_path),
                    policy=ReconnectPolicy(),
                    sleep=self._sleep,
                    clock=self._clock,
                )
            except TRANSPORT_FAILURES:
                await self._sleep(min(delay, float(self._controller.seat_retry_s)))
                delay *= 2

    def _open_store(self, seat: ResumingSeat) -> MemoryStore:
        if self._store is None:
            identity = StoreIdentity(
                world_instance=seat.instance, seat=self._key, self_entity=seat.observer
            )
            self._store_dir.mkdir(parents=True, exist_ok=True)
            self._store = MemoryStore.open(self._store_path, identity)
            self._cursor.store = self._store
        return self._store

    def _persist(self, seat: ResumingSeat) -> None:
        """Writes the current connection's secret (§5.10), and records which connection it belongs to."""
        secret = seat.welcome.resume
        if secret is None:
            resume_file.delete(self._resume_path)
        else:
            resume_file.write(self._resume_path, secret)
        self._written = seat.connection

    def _handback_wait(self) -> float:
        wait = float(self._controller.handback_s)
        state = self._breaker.state(self._clock())
        if state.kind == "open" and state.until_wall_s is not None:
            wait = max(wait, state.until_wall_s - self._clock())
        return wait

    # ── Serving a joined seat ─────────────────────────────────────────────────────────────────────

    async def _serve(self, seat: ResumingSeat) -> Literal["handback", "ended"]:
        store = self._open_store(seat)
        ingest = asyncio.create_task(self._ingest(seat, store))
        watch = asyncio.create_task(self._watch(seat))
        consults = asyncio.create_task(self._consults(seat))
        try:
            await asyncio.wait({ingest, watch, consults}, return_when=asyncio.FIRST_COMPLETED)
        except asyncio.CancelledError:
            await self._leave_on_stop(seat)
            raise
        finally:
            for task in (ingest, watch, consults):
                task.cancel()
            await asyncio.gather(ingest, watch, consults, return_exceptions=True)
        for task in (ingest, watch, consults):
            error = None if task.cancelled() else task.exception()
            if error is not None:
                raise error
        if consults.cancelled() or consults.result() is not True:
            return "ended"
        await seat.leave()
        resume_file.delete(self._resume_path)
        self._runner_counts["handbacks"] += 1
        return "handback"

    async def _ingest(self, seat: ResumingSeat, store: MemoryStore) -> None:
        """Absorbs each batch, then detects triggers in it (R-P6-FG-1, I-P6-6), then offers them to the pending slot."""
        async for batch in seat.perceived():
            # A process's first join presents no cursor (D-P6b-15), so a restarted process is sent the world's
            # history again; the store takes it without change, and a fact it already held is no trigger: it
            # was answered, or let pass, before the restart (S10 P7 F-P7-3, the F-13 rule for an lm seat).
            held = store.cursor()
            fresh = tuple(
                fact
                for fact in batch.events
                if held is None or event_key(fact.id) > event_key(held)
            )
            await absorb(store, batch.events, batch.through, self._retention)
            self._note_lines(batch.events)
            here, now_at = self._here()
            for trigger in detect(
                fresh,
                seat.observer,
                store,
                here,
                now_at,
                self._greeted,
                self._controller.greet_cooldown_s,
            ):
                if trigger.kind == "arrived" and trigger.speaker is not None:
                    self._greeted[trigger.speaker] = now_at
                self._pending.offer(trigger, self._clock())

    def _note_lines(self, events: tuple[PerceivedEvent, ...]) -> None:
        for fact in events:
            if fact.event_type == SPOKE:
                speaker = person_of(body_of(fact), "speaker")
                if speaker is not None:
                    self._last_line[speaker] = fact.id

    async def _watch(self, seat: ResumingSeat) -> None:
        while True:
            seen = await seat.changed(self._newest)
            if seen.connection != self._written:
                self._persist(seat)
            self._newest = seen
            self._observed.set()

    async def _consults(self, seat: ResumingSeat) -> bool:
        """Runs one consult at a time. Returns `True` when the seat must be handed back (§5.8)."""
        deadline = float(self._controller.consult_deadline_s)
        while True:
            trigger, received = await self._pending.take()
            if self._clock() - received > deadline:
                self._runner_counts["stale_triggers"] += 1
                continue
            await self._observed.wait()
            record = await self._consult_once(seat, trigger)
            if self._handback_due(record) and seat.welcome.took_over == "hosted":
                return True

    async def _leave_on_stop(self, seat: ResumingSeat) -> None:
        """A shutdown (Ctrl-C, §5.16) gives the seat back at once, within `STOP_LEAVE_S`, and deletes the
        secret. A killed process sends nothing: the hold runs out instead (F-P6-4)."""
        try:
            async with asyncio.timeout(STOP_LEAVE_S):
                await seat.leave()
        except Exception:
            pass
        resume_file.delete(self._resume_path)

    # ── One consult ───────────────────────────────────────────────────────────────────────────────

    async def _consult_once(self, seat: ResumingSeat, trigger: Trigger) -> ConsultRecord:
        store = self._open_store(seat)
        observation = self._observation()
        names = NameBook.load(store)
        situation = Situation.of(observation, names)
        persona, _absent = Persona.of(observation)
        counterparts = tuple(person.entity for person in situation.present)
        if trigger.speaker is not None and trigger.speaker not in counterparts:
            counterparts = (*counterparts, trigger.speaker)
        memory = memory_view(
            store,
            RecallQuery(
                at=observation.at,
                counterparts=counterparts,
                words=trigger.words,
                speaker=trigger.speaker,
                heard=trigger.event if trigger.kind == "spoken_to" else None,
            ),
            self._controller.recall_episodes,
        )
        answers: list[str] = []

        async def submit(request: ActionRequest) -> Answered | RefusedRequest:
            answer = await seat.submit(request)
            answers.append(_answer_kind(answer))
            return answer

        deps = ConsultDeps(
            ask=self._ask,
            newest=self._newest_state,
            current=lambda: not self._superseded(trigger),
            submit=submit,
            clock=self._clock,
            sleep=self._sleep,
            policies=self._policies,
            limits=ConsultLimits(
                deadline_s=float(self._controller.consult_deadline_s),
                max_calls=self._controller.max_calls,
            ),
        )
        record = await consult(
            ConsultInput(
                seat=self._key,
                persona=persona,
                situation=situation,
                memory=memory,
                trigger=trigger,
            ),
            deps,
        )
        self._account(
            seat,
            DecisionInput(
                persona=persona,
                situation=situation,
                memory=memory,
                trigger=trigger,
                observation=observation,
            ),
            record,
            tuple(answers),
        )
        return record

    async def _ask(self, request: CompletionRequest, timeout: float) -> GateOutcome:
        return await self._gateway.complete(self._key, request, timeout_s=timeout)

    def _observation(self) -> Observation:
        if self._newest is None:
            raise RuntimeError("a consult runs only after an observation has arrived")
        return self._newest.frame.observation

    def _here(self) -> tuple[PlaceId | None, int]:
        if self._newest is None:
            return None, 0
        observation = self._newest.frame.observation
        location = observation.self_location
        return (None if location is None else location.place), observation.at

    def _newest_state(self) -> NewestState:
        observation = self._observation()
        return NewestState(
            situation=Situation.of(observation, NameBook.load(self._store_or_raise())),
            observation=observation,
        )

    def _store_or_raise(self) -> MemoryStore:
        if self._store is None:
            raise RuntimeError("the store is opened before the seat serves")
        return self._store

    def _superseded(self, trigger: Trigger) -> bool:
        """A newer line from the trigger's speaker arrived after the trigger (§5.3, pre-submit check)."""
        if trigger.speaker is None:
            return False
        last = self._last_line.get(trigger.speaker)
        return last is not None and event_key(last) > event_key(trigger.event)

    def _account(
        self,
        seat: ResumingSeat,
        inputs: DecisionInput,
        record: ConsultRecord,
        answers: tuple[str, ...],
    ) -> None:
        self._consult_count += 1
        decision = DecisionRecord(
            seat=str(self._key),
            consult=self._consult_count,
            input=inputs,
            record=record,
            answers=answers,
        )
        self._log.append(decision)
        self._records.append(decision)
        outcome = breaker_outcome(record)
        if outcome is not None:
            now = self._clock()
            before = self._breaker.state(now).kind
            state = self._breaker.observe(outcome, now, seat.welcome.world.time_scale)
            if state.kind == "open" and before != "open":
                self._runner_counts["breaker_trips"] += 1
        if self._consult_count % 50 == 0:
            self._log.counters(self._counters())

    def _handback_due(self, record: ConsultRecord) -> bool:
        if record.outcome in FALLBACK_OUTCOMES:
            return True
        return self._breaker.state(self._clock()).kind == "open"

    # ── Ending ────────────────────────────────────────────────────────────────────────────────────

    def _counters(self) -> Counters:
        return counters_from(self._records).model_copy(update=self._runner_counts)

    def _close(self) -> None:
        # A seat refused before its store was opened (D-P6b-21) has no directory to write its counters into.
        if self._store_dir.is_dir():
            self._log.counters(self._counters())
        if self._store is not None:
            self._store.close()
            self._store = None
            self._cursor.store = None


def _bind(
    registry: RetryRegistry, config: CognitionConfig, seat: str, backend: str, layer: Layer
) -> BoundRetry:
    chosen = layer_policy(config, seat, backend, layer)
    return registry.bind(chosen.policy, dict(chosen.parameters), f"retry.{layer}")


def _answer_kind(answer: Answered | RefusedRequest) -> str:
    """What the world said to one submit, as the decision log writes it (`DecisionRecord.answers`)."""
    if isinstance(answer, RefusedRequest):
        return f"refused:{answer.code}"
    if isinstance(answer.result, Accepted):
        return "accepted"
    return "unavailable" if answer.result == "unavailable" else "rejected"
