"""The world stays valid under 30 % garbage, and a late answer is never spoken (S10 P6b, C13; pr-s10-p6 §§5.9, 6, 7, 8).

Scenario A (AP6-1, AP6-2, AP6-3, AP6-11, AP6-12): the seat runner, against the real server, answers visitor lines
through a seeded script that returns garbage on 30 % of attempts and a transport failure on 5 % (`garbage_script`, C9). The
assertions read the save, the decision log and the server's own tools (`mineworld inspect`, `mineworld replay`), never the
runner's memory. The save is then resumed by a second server, and every artefact is scanned for provider concepts.

Scenario B (AP6-16, IC-3): a valid script holds its answer to the visitor's line until the visitor has walked out of
talk range (D-P6b-46: held by the test, not by a wall-clock delay assumed to outlast the walk). No line of Alice's is spoken after the visitor's departure, and the decision log records the local `not_offered`
drop, so the server never sees the request.

Assertions read artefacts only. The real-binary helpers below are copied from `test_controller_server.py` (C12): pytest's
private-name rule forbids importing them, and the copies keep C12's file unchanged (D-P6b-31).
"""

from __future__ import annotations

import asyncio
import json
import math
import sqlite3
import subprocess
import time
from collections.abc import Callable
from itertools import pairwise
from pathlib import Path
from typing import cast

import pytest
from garbage import garbage_script
from mineworld_sdk import offers
from mineworld_sdk.session import Answered, SeatSession, open_socket
from mineworld_sdk.wire.contract import Accepted, ActionRequest, Observation
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey
from realserver import INVITE, REPOSITORY, Server, binary
from support import completion, run
from test_provider_scan import PATTERN

from mineworld_cognition.__main__ import serve
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import CognitionConfig, build_router
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.log import DecisionRecord, read_decisions

pytestmark = pytest.mark.real_binary

ALICE = EntityKey("alice")
VISITOR_LABEL = "Vera Lindgren"
"""The visitor's name (`worlds/social-cafe/people/visitor.yaml`): the label the model sees under "I can talk to"."""
ALICE_AT = (6000, 8000)
"""Alice's start in the cafe (`people/alice.yaml`). She is held by the seat runner, so she stays where she starts."""
WORDS = "Hello from the LM seat."
SAY = json.dumps({"act": "say", "to": VISITOR_LABEL, "words": WORDS, "offer": None, "recalls": []})
NOTHING = json.dumps({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
TALK = ActionTypeId("talk")
MOVE = ActionTypeId("move")
STRIDE_MM = 2_000
IN_REACH_MM = 1_800
"""Inside the three-metre talk range the cafe offers (`people/visitor.yaml`)."""
PATIENCE_S = 30.0
HOLD_S, PACE_S = 3, 1
LINES = 15
"""Scenario A: the visitor's lines (AP6-1, AP6-3)."""
GARBAGE_PERMILLE = 300
FAILURE_PERMILLE = 50
SEED = 7
MAX_CALLS = 5
"""AP6-2: no consult makes more than `max_calls` model calls (the default, `[controller] max_calls`)."""
TERMINAL = frozenset({"acted", "chose", "fell_back", "budget", "disconnected", "dropped"})
"""The terminal outcomes of a consult (`consult.ConsultOutcome`)."""
MARKER = "<<<"
"""The leak kind's marker (`tests/garbage.py`): no utterance in the world may carry it (AP6-1)."""
MOVES = frozenset({"move", "walk-step", "walk-to"})
"""Request types the LM seat must never submit (I-P6-3, AP6-11)."""
DELAY_S = 3
"""Scenario B: the base of the quiet window before the line (`_quiet`), the longest a greeting may take to end."""
SETTLE_S = 1.5
"""Seconds the test lets Alice's view settle after the visitor is in reach, before a line is said. The seat's prompt is
built from its newest observation, which can trail the visitor's own view by a frame (D-P6b-30)."""
START = (1610, 600)
"""The visitor's start (`people/visitor.yaml`): 8.6 m from Alice, outside the three-metre talk range."""


def _text(request: CompletionRequest) -> str:
    return "\n".join(message.text for message in request.messages)


def _talkable(text: str) -> bool:
    return any(
        line.startswith("I can talk to:") and VISITOR_LABEL in line for line in text.splitlines()
    )


def _config(handback_s: int = 10) -> CognitionConfig:
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
            "controller": {"handback_s": handback_s, "consult_deadline_s": 20},
        }
    )


def _start_with(server: Server, store: Path, backend: ScriptedBackend) -> asyncio.Task[None]:
    """The seat runner for Alice, in-process, against `server` and `backend`."""
    config = _config(handback_s=10)
    router = build_router(config, scripted=backend)
    return asyncio.create_task(
        serve(
            config,
            [LmSeat(key=ALICE, memory=None)],
            router=router,
            invite=Invite(INVITE),
            url=server.url,
            store_dir=store,
        )
    )


async def _stop(task: asyncio.Task[None]) -> None:
    """Cancels the runner as Ctrl-C does, so it leaves Alice's seat first (`SeatRunner._leave_on_stop`)."""
    task.cancel()
    await asyncio.gather(task, return_exceptions=True)


async def _wait_for(predicate: Callable[[], bool], what: str, within: float = PATIENCE_S) -> None:
    deadline = time.monotonic() + within
    while not predicate():
        if time.monotonic() > deadline:
            pytest.fail(f"{what} did not happen within {within:.0f} s")
        await asyncio.sleep(0.1)


async def _visitor(url: str) -> SeatSession:
    connection = await open_socket(url)
    return await SeatSession.join(
        connection, seat=EntityKey("visitor"), invite=Invite(INVITE), nickname="visitor-probe"
    )


def _alice_id(observation: Observation) -> EntityId:
    for entity in observation.entities:
        for record in entity.components:
            if record.component_type == "display-name" and isinstance(record.payload, dict):
                if record.payload.get("name") == "Alice Moreau":
                    return entity.id
    pytest.fail("Alice is not named in the visitor's observation")


def _strides(start: tuple[int, int], end: tuple[int, int]) -> list[tuple[int, int]]:
    """The fewest equal strides of at most `STRIDE_MM` from `start` to `end`, integer arithmetic."""
    (x0, y0), (x1, y1) = start, end
    count = max(1, math.ceil(math.hypot(x1 - x0, y1 - y0) / STRIDE_MM))
    while True:
        points = [
            (x0 + (x1 - x0) * k // count, y0 + (y1 - y0) * k // count) for k in range(count + 1)
        ]
        if all(math.dist(a, b) <= STRIDE_MM for a, b in pairwise(points)):
            return points[1:]
        count += 1


async def _walk_to(visitor: SeatSession, goal: tuple[int, int]) -> None:
    """Walks the visitor to `goal` in the cafe: every stride offered and accepted (the walk itself takes longer)."""
    where = (await visitor.changed()).observation.self_location
    assert where is not None and where.local is not None, "the visitor has a position in the cafe"
    place = where.place.entity
    for step_x, step_y in _strides((where.local.x, where.local.y), goal):
        observation = (await visitor.changed()).observation
        request = offers.request(
            observation,
            MOVE,
            target=None,
            payload={
                "to": {
                    "place": {"entity": place, "entity_type": "place"},
                    "local": {"x": step_x, "y": step_y, "z": 0},
                    "facing": None,
                }
            },
        )
        outcome = await visitor.submit(request)
        assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
            f"a stride to ({step_x}, {step_y}) was not accepted: {outcome}"
        )


async def _walk_into_reach(visitor: SeatSession) -> None:
    """Walks the visitor to within `IN_REACH_MM` of Alice's start, every stride offered and accepted."""
    where = (await visitor.changed()).observation.self_location
    assert where is not None and where.local is not None, "the visitor has a position in the cafe"
    x, y = where.local.x, where.local.y
    ax, ay = ALICE_AT
    distance = math.hypot(ax - x, ay - y)
    goal = (
        ax - round((ax - x) * IN_REACH_MM / distance),
        ay - round((ay - y) * IN_REACH_MM / distance),
    )
    await _walk_to(visitor, goal)


async def _talk_offered(visitor: SeatSession, alice: EntityId) -> bool:
    observation = (await visitor.changed()).observation
    return any(
        a.action_type == TALK and a.target == alice and a.available for a in observation.affordances
    )


async def _talk_within(visitor: SeatSession, alice: EntityId, within: float) -> bool:
    """Whether talk to Alice is offered at some point in the next `within` seconds."""
    deadline = time.monotonic() + within
    while time.monotonic() < deadline:
        if await _talk_offered(visitor, alice):
            return True
        await asyncio.sleep(0.2)
    return False


async def _talk_available(visitor: SeatSession, alice: EntityId) -> None:
    deadline = time.monotonic() + PATIENCE_S
    while not await _talk_offered(visitor, alice):
        if time.monotonic() > deadline:
            pytest.fail(
                f"no observation with talk to Alice available arrived within {PATIENCE_S:.0f} s"
            )
        await asyncio.sleep(0.2)


async def _say(visitor: SeatSession, alice: EntityId, text: str) -> None:
    observation = (await visitor.changed()).observation
    request = offers.request(observation, TALK, target=alice, payload={"utterance": text})
    outcome = await visitor.submit(request)
    assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
        f"the visitor's line was not accepted: {outcome}"
    )


async def _await_record(store: Path, before: int, what: str) -> None:
    """Waits until the decision log holds more than `before` records."""
    await _wait_for(lambda: len(_decisions(store)) > before, what)


def _decisions(store: Path) -> list[DecisionRecord]:
    path = store / "alice.decisions.jsonl"
    return read_decisions(path) if path.exists() else []


def _latest(save: Path) -> int:
    """The newest journal revision of the save (read-only)."""
    connection = sqlite3.connect(f"file:{save / 'world.sqlite'}?mode=ro", uri=True)
    try:
        (revision,) = connection.execute("SELECT MAX(revision) FROM journal").fetchone()
    finally:
        connection.close()
    return int(revision or 0)


def _talkable_script(request: CompletionRequest) -> str:
    return SAY if _talkable(_text(request)) else NOTHING


def _garbage_mixed(
    prompts: list[str],
) -> Callable[[CompletionRequest], Completion | BackendFailure]:
    """Seeded garbage over the valid answer; the valid answer is `SAY` only while the visitor is talkable."""
    scripts = {
        SAY: garbage_script(SEED, GARBAGE_PERMILLE, FAILURE_PERMILLE, SAY),
        NOTHING: garbage_script(SEED, GARBAGE_PERMILLE, FAILURE_PERMILLE, NOTHING),
    }

    def script(request: CompletionRequest) -> Completion | BackendFailure:
        text = _text(request)
        prompts.append(text)
        return scripts[SAY if _talkable(text) else NOTHING](request)

    return script


def _valid(request: CompletionRequest) -> Completion | BackendFailure:
    return completion(_talkable_script(request))


class _HeldBackend(ScriptedBackend):
    """Scenario B's model: answers at once until `hold()`, then keeps every answer until `release()`. The line's
    answer is late because the test holds it until the visitor has left talk range, not because a wall-clock delay
    is assumed to outlast the walk (a slow runner walked out after the answer, D-P6b-46)."""

    calls: int

    def __init__(self, script: Callable[[CompletionRequest], Completion | BackendFailure]) -> None:
        super().__init__(script)
        self._answer = script
        self._open = asyncio.Event()
        self._open.set()

    def hold(self) -> None:
        self._open.clear()

    def release(self) -> None:
        self._open.set()

    async def complete(self, request: CompletionRequest) -> Completion | BackendFailure:
        self.calls += 1
        await self._open.wait()
        return self._answer(request)


async def _frames_after(visitor: SeatSession, count: int) -> None:
    """Waits for `count` newer observations on the visitor's stream: the server sweeps every seat in one pass, so
    Alice's seat has been sent the same ticks (world time, not a wall-clock guess)."""
    seen = await visitor.changed()
    for _ in range(count):
        seen = await asyncio.wait_for(visitor.changed(seen.seq), 30)


async def _quiet(backend: ScriptedBackend, store: Path) -> None:
    """Returns once the seat has neither asked the model nor logged a consult for `DELAY_S + SETTLE_S` seconds: no
    consult is in flight and none is pending."""
    seen = (backend.calls, len(_decisions(store)))
    since = time.monotonic()
    while time.monotonic() - since < DELAY_S + SETTLE_S:
        await asyncio.sleep(0.2)
        state = (backend.calls, len(_decisions(store)))
        if state != seen:
            seen, since = state, time.monotonic()


def _find(node: object, key: str) -> str | None:
    """The first string under `key` anywhere in a JSON value."""
    if isinstance(node, dict):
        for name, value in cast(dict[str, object], node).items():
            if name == key and isinstance(value, str):
                return value
            found = _find(value, key)
            if found is not None:
                return found
    if isinstance(node, list):
        for item in cast(list[object], node):
            found = _find(item, key)
            if found is not None:
                return found
    return None


def _utterance(request: ActionRequest) -> str | None:
    return _find(request.model_dump(mode="json"), "utterance")


def _spoken_after(save: Path, after: int) -> list[tuple[int, str, str]]:
    """(revision, speaker, utterance) of each `spoke` fact of the save after revision `after` (read-only)."""
    connection = sqlite3.connect(f"file:{save / 'world.sqlite'}?mode=ro", uri=True)
    try:
        rows = connection.execute("SELECT revision, fact FROM facts").fetchall()
    finally:
        connection.close()
    spoken: list[tuple[int, str, str]] = []
    for revision, blob in rows:
        if int(revision) <= after:
            continue
        fact = cast(dict[str, object], json.loads(bytes(blob)))
        if fact.get("event_type") != "spoke":
            continue
        outer = cast(dict[str, object], fact["payload"])
        inner = cast(dict[str, object], json.loads(bytes(cast(list[int], outer["payload"]))))
        speaker = cast(dict[str, object], inner["speaker"])
        spoken.append((int(revision), str(speaker["entity"]), str(inner["utterance"])))
    return spoken


def _inspect_and_replay(save: Path) -> tuple[int, int]:
    """The exit codes of `mineworld inspect` and `mineworld replay` over the save (an argument list, never a shell)."""
    inspected = subprocess.run(
        [str(binary()), "inspect", str(save)],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=120,
        check=False,
    )
    replayed = subprocess.run(
        [str(binary()), "replay", str(REPOSITORY / "worlds" / "social-cafe"), "--save", str(save)],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=600,
        check=False,
    )
    for result in (inspected, replayed):
        if result.returncode != 0:
            print(result.stdout, result.stderr)
    return inspected.returncode, replayed.returncode


def _assert_provider_free(store: Path, prompts: list[str]) -> None:
    """AP6-12 over the artefacts this scenario writes: the decision log and the store files, and every prompt sent."""
    texts = [
        path.read_bytes().decode("utf-8", "replace") for path in store.iterdir() if path.is_file()
    ]
    texts.extend(prompts)
    for text in texts:
        found = PATTERN.search(text)
        assert found is None, f"AP6-12: a provider concept {found.group(0)!r} is in an artefact"


def test_a_the_world_stays_valid_under_thirty_percent_garbage_and_five_percent_failure(
    tmp_path: Path,
) -> None:
    """AP6-1 (no garbage reaches the world), AP6-2 (one terminal outcome per consult, at most `max_calls` calls),
    AP6-3 (inspect and replay succeed; the save resumes), AP6-11 (no movement request), AP6-12 (no provider concept).
    AP6-2's duration half is not a logged field (D-P6b-29)."""
    save = tmp_path / "save"
    store = tmp_path / "store"
    prompts: list[str] = []
    server = Server(save)
    try:

        async def scenario() -> tuple[EntityId, int]:
            task = _start_with(server, store, ScriptedBackend(_garbage_mixed(prompts)))
            await _wait_for(lambda: (store / "alice.resume").exists(), "the runner joined Alice")
            visitor = await _visitor(server.url)
            said = 0
            async with visitor:
                await _walk_into_reach(visitor)
                alice = _alice_id((await visitor.changed()).observation)
                for line in range(LINES):
                    # A fallback hands Alice back to her own rule, which may walk her out of the visitor's reach:
                    # then no further line can be said, and the run ends with the lines said so far (`said`).
                    if not await _talk_within(visitor, alice, SETTLE_S + 15):
                        break
                    await asyncio.sleep(SETTLE_S)
                    before = len(_decisions(store))
                    await _say(visitor, alice, f"Line {line}.")
                    said += 1
                    await _await_record(store, before, f"a consult for line {line}")
            await _stop(task)
            return alice, said

        alice, said = run(scenario(), timeout_s=300)
        # AP6-3 is about the save the run left: the server stops before it is inspected, replayed and
        # resumed (D-P6b-42). Inspected live, the server's own commits raced the inspection's two passes.
        server.stop()
        records = _decisions(store)
        requests = [
            record.record.request for record in records if record.record.request is not None
        ]
        spoken = _spoken_after(server.save, 0)
        inspect_code, replay_code = _inspect_and_replay(server.save)
        outcomes = sorted({record.record.outcome for record in records})
        garbage = sum(
            1
            for record in records
            for attempt in record.record.attempts
            if attempt.failure is not None and attempt.failure.cls == "content"
        )
        print(
            f"AP6-1/2/3: {said} lines said, {len(records)} consults, outcomes {outcomes}, {len(requests)} acted, "
            f"{garbage} content failures drawn; Alice's spoke facts {len(spoken)}; "
            f"inspect exit {inspect_code}, replay exit {replay_code}"
        )
        assert said >= 1, "the visitor could not say a line: the scenario did not run"
        assert garbage >= 1, "AP6-1: no garbage was drawn, so the scenario proves nothing"
        assert len(records) >= said, "AP6-2: every visitor line must reach the engine"
        assert all(record.record.outcome in TERMINAL for record in records), (
            "AP6-2: a consult ended without a terminal outcome"
        )
        assert all(len(record.record.attempts) <= MAX_CALLS for record in records), (
            "AP6-2: a consult made more than max_calls model calls"
        )
        assert all(_utterance(request) == WORDS for request in requests), (
            "AP6-1: a request the seat submitted is not the scripted valid line"
        )
        assert requests == [] or any(
            speaker == str(alice) and text == WORDS for _, speaker, text in spoken
        ), "AP6-1: the seat acted but Alice was not heard saying the scripted line"
        assert not any(MARKER in text for _, _, text in spoken), (
            "AP6-1: a garbage marker reached the world"
        )
        assert all(str(request.action_type) not in MOVES for request in requests), (
            "AP6-11: the seat submitted a movement request"
        )
        assert inspect_code == 0, "AP6-3: mineworld inspect did not resolve every cause"
        assert replay_code == 0, "AP6-3: mineworld replay did not reproduce the save"
        resumed = Server(save)
        resumed.stop()
        _assert_provider_free(store, prompts)
    finally:
        server.stop()


def test_b_a_late_answer_after_the_visitor_walks_out_is_dropped_locally(tmp_path: Path) -> None:
    """AP6-16 (IC-3): with the answer held until the visitor is out of range (D-P6b-46), no Alice `spoke`
    fact follows the visitor's departure, the decision log records the local `not_offered` drop, and the consult that
    dropped the line submitted nothing. M-P6-8 (a `Say` without `offers.request`) turns this red: the server refuses
    the unchecked talk, which the pre-line check reports."""
    save = tmp_path / "save"
    store = tmp_path / "store"
    server = Server(save)
    try:

        async def scenario() -> tuple[int, EntityId]:
            backend = _HeldBackend(_valid)
            task = _start_with(server, store, backend)
            await _wait_for(lambda: (store / "alice.resume").exists(), "the runner joined Alice")
            visitor = await _visitor(server.url)
            async with visitor:
                await _walk_into_reach(visitor)
                alice = _alice_id((await visitor.changed()).observation)
                await _talk_available(visitor, alice)
                # The consults the visitor's arrival started (greetings, each in flight for the whole delay) end first,
                # so the line below is the only consult the snapshot can see end.
                await _quiet(backend, store)
                refused = [a for r in _decisions(store) for a in r.answers if a != "accepted"]
                assert not refused, (
                    "AP6-16: the seat submitted a request the server refused before the line "
                    f"({refused}); an out-of-reach talk is a local drop, not a server refusal"
                )
                await _talk_available(visitor, alice)
                before = len(_decisions(store))
                asked = backend.calls
                backend.hold()
                await _say(visitor, alice, "Good morning.")
                # The walk starts only once the model holds the request (the consult is in flight, in range).
                await _wait_for(lambda: backend.calls > asked, "the model was asked", within=20)
                await _walk_to(visitor, START)
                departed = _latest(server.save)
                assert not await _talk_offered(visitor, alice), (
                    "the visitor did not leave talk range"
                )
                # Alice's own view trails the visitor's by up to a frame (D-P6b-30): two more sweeps, then the answer.
                await _frames_after(visitor, 2)
                backend.release()
                await _await_record(store, before, "the held consult to end")
            await _stop(task)
            return departed, alice

        departed, alice = run(scenario(), timeout_s=180)
        late = [
            spoken for spoken in _spoken_after(server.save, departed) if spoken[1] == str(alice)
        ]
        records = _decisions(store)
        dropped_locally = [
            record
            for record in records
            if any(
                attempt.failure is not None and attempt.failure.reason == "not_offered"
                for attempt in record.record.attempts
            )
        ]
        print(
            f"AP6-16: visitor out of range after revision {departed}; Alice's spoke facts after it {len(late)}; "
            f"local not_offered records {len(dropped_locally)}; submits answered by the server "
            f"{[record.answers for record in records if record.answers]} (the greeting's, before the departure)"
        )
        assert not late, "AP6-16: Alice spoke after the visitor walked out of range"
        assert dropped_locally, "AP6-16: the decision log has no local not_offered drop"
        assert all(not record.answers for record in dropped_locally), (
            "AP6-16: the consult that dropped the line still submitted a request to the server"
        )
    finally:
        server.stop()
