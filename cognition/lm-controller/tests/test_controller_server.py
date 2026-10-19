"""The LM controller against the real `mineworld` binary (S10 P6b, C12; pr-s10-p6 §§5.8-5.10, 6, 8, 9 C12).

Each test starts `mineworld server worlds/social-cafe --town --decisions examples/alice-lm.decisions.yaml` on loopback
with `--hold 3 --pace 1`. A visitor is a protocol client (the SDK's `SeatSession`). The seat runner runs in-process through
`mineworld_cognition.__main__.serve` against a scripted backend, or as a `python -m mineworld_cognition` process against a
replay of an empty cassette when the test must SIGKILL it. No model, no key, no live mode, loopback only.

Assertions read what a player perceives, the save's journal (each request's actor), the decision log, and the resume
file. `ResumingSeat` is used directly only where a welcome's `took_over` is the thing under test (A-P6-2, A-P6-4, AP6-17).
"""

from __future__ import annotations

import asyncio
import hashlib
import json
import math
import os
import shutil
import sqlite3
import subprocess
import sys
import time
from collections.abc import Awaitable, Callable
from itertools import pairwise
from pathlib import Path
from typing import cast

import pytest
from mineworld_sdk import offers
from mineworld_sdk.reconnect import SeatLost
from mineworld_sdk.resuming import ResumingSeat
from mineworld_sdk.session import Answered, Connection, SeatSession, open_socket
from mineworld_sdk.wire.contract import Accepted, Observation
from mineworld_sdk.wire.frames import Invite, ObservationFrame
from mineworld_sdk.wire.ids import ActionTypeId, EntityId, EntityKey, EventId
from realserver import DECISIONS, INVITE, REPOSITORY, Server, binary
from support import completion, run

from mineworld_cognition.__main__ import serve
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import CognitionConfig, build_router
from mineworld_cognition.controller import resume_file
from mineworld_cognition.controller.assignment import LmSeat
from mineworld_cognition.controller.log import DecisionRecord, read_decisions
from mineworld_cognition.memory.records import event_key

pytestmark = pytest.mark.real_binary

ALICE = EntityKey("alice")
VISITOR_LABEL = "Vera Lindgren"
"""The visitor's name (`worlds/social-cafe/people/visitor.yaml`): the label the model sees under "People here"."""
ALICE_AT = (6000, 8000)
"""Alice's start in the cafe (`people/alice.yaml`, `location.position`). She is held by the seat runner, so she stays."""
WORDS = "Hello from the LM seat."
SAY = json.dumps({"act": "say", "to": VISITOR_LABEL, "words": WORDS, "offer": None, "recalls": []})
NOTHING = json.dumps({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
REFUSED = BackendFailure(reason="unauthorized")
"""A non-retryable transport failure: the consult falls back at once (§5.5.1), so the seat is handed back."""
TALK = ActionTypeId("talk")
MOVE = ActionTypeId("move")
STRIDE_MM = 2_000
IN_REACH_MM = 1_800
"""Inside the three-metre talk range the cafe offers (`people/visitor.yaml`)."""
PATIENCE_S = 30.0
HOLD_S, PACE_S = 3, 1
FALLBACK_BOUND_S = HOLD_S + PACE_S + 2
"""AP6-8's kill bound: the fallback's request appears within hold + one pace + 2 s."""
COGNITION_TOML = """\
server         = "{url}"
invite_env     = "MINEWORLD_INVITE"
nickname       = "cognition"
store          = "store"
decisions_json = "seats.json"

[recording]
mode     = "replay"
cassette = "cassette.jsonl"

[tiers]
social    = "local"
summarize = "none"

[backends.local]
base_url          = "http://127.0.0.1:11434/v1"
model             = "fake-model"
structured_output = "none"
"""
CASSETTE_HEADER = '{"cassette":"mineworld-cognition","format":1,"key_scheme":1}\n'
"""An empty cassette: a replay answers no request, so a consult that reached it would fall back."""


class _Cursor:
    """A fixed cursor source, as a caller's durable store presents it at a join."""

    def __init__(self, value: EventId | None) -> None:
        self.value = value

    def cursor(self) -> EventId | None:
        return self.value


SEEN: list[str] = []
"""The "I can talk to" line of each prompt the scripted seat was asked, for a failure message. A prompt also names
people in memory, so only this line says who may be addressed now."""


def _speaks(request: CompletionRequest) -> Completion | BackendFailure:
    talk = next(
        (line for line in _text(request).splitlines() if line.startswith("I can talk to:")), ""
    )
    SEEN.append(talk)
    return completion(SAY if VISITOR_LABEL in talk else NOTHING)


def _refuses(request: CompletionRequest) -> Completion | BackendFailure:
    return REFUSED


def _text(request: CompletionRequest) -> str:
    return "\n".join(message.text for message in request.messages)


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


def _start(
    server: Server,
    store: Path,
    script: Callable[[CompletionRequest], Completion | BackendFailure],
    handback_s: int = 10,
) -> asyncio.Task[None]:
    """The seat runner for Alice, in-process, against `server` and a scripted backend."""
    config = _config(handback_s)
    router = build_router(config, scripted=ScriptedBackend(script))
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


async def _settle_on(predicate: Callable[[], bool], within: float = PATIENCE_S) -> None:
    """Waits as `_wait_for` does, but returns at the bound instead of failing: the caller's own assertion
    then reports the miss with its own message."""
    deadline = time.monotonic() + within
    while not predicate() and time.monotonic() <= deadline:
        await asyncio.sleep(0.1)


def _connector(url: str) -> Callable[[], Awaitable[Connection]]:
    """A `Connector` that opens a socket to `url` each time it is called (a rejoin opens a new one)."""

    async def connect() -> Connection:
        return await open_socket(url)

    return connect


async def _visitor(url: str) -> SeatSession:
    connection = await open_socket(url)
    return await SeatSession.join(
        connection, seat=EntityKey("visitor"), invite=Invite(INVITE), nickname="visitor-probe"
    )


async def _until(
    session: SeatSession, what: str, ready: Callable[[ObservationFrame], bool]
) -> ObservationFrame:
    """The first observation satisfying `ready`, waited for on the visitor's stream."""
    since = None
    deadline = time.monotonic() + PATIENCE_S
    while time.monotonic() < deadline:
        frame = await asyncio.wait_for(session.changed(since), PATIENCE_S)
        if ready(frame):
            return frame
        since = frame.seq
    pytest.fail(f"no observation with {what} arrived within {PATIENCE_S:.0f} s")


def _alice_id(observation: Observation) -> EntityId:
    for entity in observation.entities:
        for record in entity.components:
            if record.component_type == "display-name" and isinstance(record.payload, dict):
                if record.payload.get("name") == "Alice Moreau":
                    return entity.id
    pytest.fail("Alice is not named in the visitor's observation")


def _heard(observation: Observation, listener: EntityId) -> list[tuple[str, str]]:
    """(speaker, utterance) for each line `listener`'s own `conversation-history` discloses."""
    for entity in observation.entities:
        if entity.id != listener:
            continue
        for record in entity.components:
            if record.component_type != "conversation-history":
                continue
            payload = record.payload
            entries = payload.get("heard") if isinstance(payload, dict) else None
            lines: list[tuple[str, str]] = []
            for entry in entries if isinstance(entries, list) else []:
                if isinstance(entry, dict):
                    speaker, said = entry.get("speaker"), entry.get("utterance")
                    if isinstance(speaker, dict) and isinstance(said, str):
                        lines.append((str(speaker.get("entity")), said))
            return lines
    return []


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


async def _walk_into_reach(visitor: SeatSession) -> None:
    """Walks the visitor to within `IN_REACH_MM` of Alice's start, every stride offered and accepted."""
    where = (await visitor.changed()).observation.self_location
    assert where is not None and where.local is not None, "the visitor has a position in the cafe"
    place = where.place.entity
    x, y = where.local.x, where.local.y
    ax, ay = ALICE_AT
    distance = math.hypot(ax - x, ay - y)
    goal = (
        ax - round((ax - x) * IN_REACH_MM / distance),
        ay - round((ay - y) * IN_REACH_MM / distance),
    )
    for step_x, step_y in _strides((x, y), goal):
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
        if not (isinstance(outcome, Answered) and isinstance(outcome.result, Accepted)):
            now = (await visitor.changed()).observation.self_location
            pytest.fail(
                f"a stride to ({step_x}, {step_y}) was not accepted: {outcome}; the walk began at {(x, y)}, "
                f"the visitor is now at {now}"
            )


async def _talk_available(visitor: SeatSession, alice: EntityId) -> None:
    await _until(
        visitor,
        "talk to Alice available",
        lambda frame: any(
            a.action_type == TALK and a.target == alice and a.available
            for a in frame.observation.affordances
        ),
    )


async def _say(visitor: SeatSession, alice: EntityId, text: str) -> None:
    observation = (await visitor.changed()).observation
    request = offers.request(observation, TALK, target=alice, payload={"utterance": text})
    outcome = await visitor.submit(request)
    assert isinstance(outcome, Answered) and isinstance(outcome.result, Accepted), (
        f"the visitor's line was not accepted: {outcome}"
    )


def _diagnose(store: Path, server: Server) -> str:
    """What the runner decided, and what the server printed, for a failure message."""
    lines = [
        f"{record.consult}: {record.record.model_dump_json()[:900]} answers {record.answers}"
        for record in _decisions(store)
    ]
    return "\n".join(
        ["decisions:", *lines, "last prompt:", *SEEN, "server stderr:", server.stderr()]
    )


def _decisions(store: Path) -> list[DecisionRecord]:
    path = store / "alice.decisions.jsonl"
    return read_decisions(path) if path.exists() else []


def _last_counters(store: Path) -> dict[str, object] | None:
    path = store / "alice.decisions.jsonl"
    last: dict[str, object] | None = None
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            loaded: object = json.loads(line) if line else None
            if (
                isinstance(loaded, dict)
                and cast(dict[str, object], loaded).get("kind") == "counters"
            ):
                last = cast(dict[str, object], loaded)
    return last


def _journal(save: Path) -> list[tuple[int, object]]:
    """Every journal entry of the save, read-only: (revision, entry)."""
    connection = sqlite3.connect(f"file:{save / 'world.sqlite'}?mode=ro", uri=True)
    try:
        rows = connection.execute(
            "SELECT revision, entry FROM journal ORDER BY revision"
        ).fetchall()
    finally:
        connection.close()
    return [(int(revision), json.loads(bytes(entry))) for revision, entry in rows]


def _actor_of(entry: object) -> object:
    """The actor of a journal entry's request (`input.dispatch.intent.actor`), or `None` for any other entry."""
    node: object = entry
    for key in ("input", "dispatch", "intent", "actor"):
        if not isinstance(node, dict):
            return None
        node = cast(dict[str, object], node).get(key)
    return node


def _requests_by(save: Path, actor: EntityId, after: int) -> int:
    """How many requests `actor` made in journal entries after revision `after`."""
    return sum(
        1
        for revision, entry in _journal(save)
        if revision > after and _actor_of(entry) == str(actor)
    )


def _latest(save: Path) -> int:
    return max((revision for revision, _ in _journal(save)), default=0)


def _count(counters: dict[str, object] | None, name: str) -> int:
    value = (counters or {}).get(name, 0)
    return value if isinstance(value, int) else -1


def _tree_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


# ── The checks of A-P6-2 and A-P6-4: the server's welcome, read through the SDK ──────────────────


def test_a_takeover_is_hosted_and_a_leave_then_rejoin_is_hosted_with_the_backfill(
    tmp_path: Path,
) -> None:
    """A-P6-2: joining Alice while her in-server controller holds her is `took_over: "hosted"`.
    A-P6-4: `leave`, then a fresh join within milliseconds is `took_over: "hosted"`, and the perceived stream
    backfills every fact after the cursor. A false answer is a DL-a or server defect (a material stop)."""
    server = Server(tmp_path / "save")
    try:

        async def scenario() -> tuple[str, float, str, bool]:
            connector = _connector(server.url)
            seat = await ResumingSeat.open(
                connector,
                seat=ALICE,
                invite=Invite(INVITE),
                nickname="takeover-probe",
                cursor=_Cursor(None),
                sleep=asyncio.sleep,
                clock=time.monotonic,
            )
            first = seat.welcome.took_over
            stream = seat.perceived()
            batch = await asyncio.wait_for(stream.__anext__(), PATIENCE_S)
            await seat.leave()

            began = time.monotonic()
            again = await ResumingSeat.open(
                connector,
                seat=ALICE,
                invite=Invite(INVITE),
                nickname="takeover-probe",
                cursor=_Cursor(batch.through),
                sleep=asyncio.sleep,
                clock=time.monotonic,
            )
            elapsed = time.monotonic() - began
            backfill = again.perceived()
            fresh = await asyncio.wait_for(backfill.__anext__(), PATIENCE_S)
            after = all(event_key(event.id) > event_key(batch.through) for event in fresh.events)
            await again.leave()
            return first, elapsed, again.welcome.took_over, after and bool(fresh.events)

        first, elapsed, rejoined, backfilled = run(scenario(), timeout_s=90)
        print(
            f"A-P6-2: first join took_over {first!r}; A-P6-4: rejoin {rejoined!r} in {elapsed * 1000:.0f} ms, "
            f"backfill after the cursor {backfilled}"
        )
        assert first == "hosted", f"A-P6-2 is false: first join took_over {first!r} (material stop)"
        assert rejoined == "hosted", (
            f"A-P6-4 is false: rejoin took_over {rejoined!r} (material stop)"
        )
        assert backfilled, (
            "A-P6-4 is false: the rejoin did not backfill from the cursor (material stop)"
        )
    finally:
        server.stop()


# ── The seat runner: a line is answered, a second seat is refused, leave on Ctrl-C ───────────────


def test_b_the_lm_seat_answers_a_line_refuses_a_second_seat_and_leaves_on_stop(
    tmp_path: Path,
) -> None:
    """(a) a visitor line gets a scripted reply that the visitor perceives from Alice; (e) a second join for
    Alice while the runner holds her is refused `seat_occupied` (AP6-17); (g) a stopped runner leaves, so a
    fresh join is `hosted` at once, not `held` (§5.16)."""
    server = Server(tmp_path / "save")
    store = tmp_path / "store"
    try:

        async def scenario() -> tuple[str, str, str, list[str], str]:
            task = _start(server, store, _speaks)
            await _wait_for(lambda: (store / "alice.resume").exists(), "the runner joined Alice")
            refusal = ""
            try:
                await ResumingSeat.open(
                    _connector(server.url),
                    seat=ALICE,
                    invite=Invite(INVITE),
                    nickname="second",
                    cursor=None,
                    take_over=False,
                    sleep=asyncio.sleep,
                    clock=time.monotonic,
                )
            except SeatLost as lost:
                refusal = lost.reason
            visitor = await _visitor(server.url)
            async with visitor:
                await _walk_into_reach(visitor)
                alice = _alice_id((await visitor.changed()).observation)
                await _talk_available(visitor, alice)
                await _say(visitor, alice, "Good morning.")
                try:
                    frame = await _until(
                        visitor,
                        f"Alice's line {WORDS!r} perceived by the visitor",
                        lambda f: (str(alice), WORDS) in _heard(f.observation, visitor.observer),
                    )
                except BaseException:
                    print(_diagnose(store, server))
                    raise
                heard = [text for _, text in _heard(frame.observation, visitor.observer)]
            # The visitor can hear the line before the acted consult's record is written (D-P6-42): wait for it
            # within the same bound; without one, (a)'s assertion below fails as before.
            await _settle_on(
                lambda: any(record.record.outcome == "acted" for record in _decisions(store))
            )
            outcomes = [record.record.outcome for record in _decisions(store)]
            await _stop(task)
            try:
                fresh = await ResumingSeat.open(
                    _connector(server.url),
                    seat=ALICE,
                    invite=Invite(INVITE),
                    nickname="after-stop",
                    cursor=None,
                    take_over=False,
                    sleep=asyncio.sleep,
                    clock=time.monotonic,
                )
            except SeatLost:
                print(f"(g) outcomes before the stop: {outcomes}; counters {_last_counters(store)}")
                print(_diagnose(store, server))
                raise
            took_over = fresh.welcome.took_over
            await fresh.leave()
            return refusal, WORDS, took_over, heard, ",".join(outcomes)

        refusal, _, took_over, heard, outcomes = run(scenario(), timeout_s=120)
        print(
            f"(e) second join refused: {refusal!r}; decision outcomes: {outcomes}; (g) join after stop: {took_over!r}"
        )
        assert refusal == "seat_occupied", (
            f"(e) a second seat was not refused seat_occupied: {refusal!r}"
        )
        assert "acted" in outcomes.split(","), f"(a) no consult acted: {outcomes}"
        assert WORDS in heard, "(a) the visitor did not perceive the scripted reply from Alice"
        assert took_over == "hosted", f"(g) a stopped runner did not leave: took_over {took_over!r}"
    finally:
        server.stop()


# ── (b): a script that always fails hands the seat back; the fallback acts; the runner rejoins ──


def test_b_a_failing_script_hands_the_seat_back_and_the_runner_rejoins(tmp_path: Path) -> None:
    """M-P6-6 turns this red: with no `leave` on fallback the runner never hands Alice back, so no request by
    her in-server controller appears in the save, and the decision log shows no second handback."""
    server = Server(tmp_path / "save")
    store = tmp_path / "store"
    try:

        async def scenario() -> tuple[int, int, bool, int]:
            task = _start(server, store, _refuses, handback_s=10)
            await _wait_for(lambda: (store / "alice.resume").exists(), "the runner joined Alice")
            visitor = await _visitor(server.url)
            async with visitor:
                await _walk_into_reach(visitor)
                alice = _alice_id((await visitor.changed()).observation)
                await _talk_available(visitor, alice)
                await _say(visitor, alice, "Good morning.")
                await _wait_for(
                    lambda: any(r.record.outcome == "fell_back" for r in _decisions(store)),
                    "a fallen-back consult",
                )
                marked = _latest(server.save)
                await _wait_for(
                    lambda: _requests_by(server.save, alice, marked) > 0,
                    "a request by Alice's in-server controller during the handback",
                    within=10 + PACE_S + 2,
                )
                during = _requests_by(server.save, alice, marked)
                # The secret is deleted at the handback and written again at the rejoin (§5.8, §5.10).
                await _wait_for(
                    lambda: not (store / "alice.resume").exists(),
                    "the runner handed the seat back",
                    within=30,
                )
                # Read after the secret was seen gone: every consult logged from here on ran on a seat joined
                # after that handback, since a session writes the secret at its join and deletes it only after
                # its consult is logged.
                handed_back_after = len(_decisions(store))
                # A rejoin is seen as the secret written again, or as a consult of the rejoined seat. The secret
                # alone is not enough (D-P6b-40, D-P6b-41): the script refuses every consult, so a rejoin that
                # backfills a trigger hands the seat back again within milliseconds and deletes the secret
                # between two polls.
                await _wait_for(
                    lambda: (
                        (store / "alice.resume").exists()
                        or len(_decisions(store)) > handed_back_after
                    ),
                    "the runner rejoined after the handback period",
                    within=30,
                )
                rejoined = True
            await _stop(task)
            return (
                len(_decisions(store)),
                during,
                rejoined,
                _count(_last_counters(store), "handbacks"),
            )

        decisions, during, rejoined, handbacks = run(scenario(), timeout_s=170)
        print(
            f"(b): {decisions} consults logged; Alice's in-server requests during the handback {during}; "
            f"handbacks counted {handbacks}; rejoined {rejoined}"
        )
        assert during > 0, (
            "(b) no request by Alice's in-server controller appeared after the handback"
        )
        assert handbacks >= 1, "(b) the runner did not count its handback"
        assert rejoined, "(b) the runner did not rejoin"
    finally:
        server.stop()


# ── (c), (d), (f): the process itself is killed; the seat is held, then the fallback acts ───────


def _prepare(directory: Path, url: str) -> None:
    """The decision JSON as `mineworld decisions --json` writes it, an empty cassette, and the configuration."""
    done = subprocess.run(
        [
            str(binary()),
            "decisions",
            str(REPOSITORY / "worlds" / "social-cafe"),
            "--decisions",
            str(DECISIONS),
            "--json",
        ],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=120,
        check=False,
    )
    if done.returncode != 0:
        raise RuntimeError(f"mineworld decisions failed ({done.returncode}): {done.stderr}")
    (directory / "seats.json").write_text(done.stdout, encoding="utf-8")
    (directory / "cassette.jsonl").write_text(CASSETTE_HEADER, encoding="utf-8")
    (directory / "cognition.toml").write_text(COGNITION_TOML.format(url=url), encoding="utf-8")


def _cognition_process(directory: Path) -> subprocess.Popen[str]:
    """`python -m mineworld_cognition --config …`, the entry point under test, in its own process."""
    return subprocess.Popen(
        [
            sys.executable,
            "-m",
            "mineworld_cognition",
            "--config",
            str(directory / "cognition.toml"),
        ],
        cwd=directory,
        env={**os.environ, "MINEWORLD_INVITE": INVITE},
        stdin=subprocess.DEVNULL,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="replace",
    )


def _joined(directory: Path, process: subprocess.Popen[str]) -> None:
    def ready() -> bool:
        if process.poll() is not None:
            pytest.fail(
                f"the cognition process exited early: {process.stderr and process.stderr.read()}"
            )
        return (directory / "store" / "alice.resume").exists()

    deadline = time.monotonic() + PATIENCE_S
    while not ready():
        if time.monotonic() > deadline:
            pytest.fail("the cognition process did not join Alice's seat")
        time.sleep(0.1)


async def _alice_of(url: str) -> EntityId:
    visitor = await _visitor(url)
    async with visitor:
        return _alice_id((await visitor.changed()).observation)


def test_c_a_killed_process_is_replaced_by_the_fallback_within_the_bound_and_the_save_survives_the_store_loss(
    tmp_path: Path,
) -> None:
    """AP6-8 (kill): with the process SIGKILLed, Alice's request by her in-server controller appears within
    hold + one pace + 2 s. AP6-15 (part): deleting the store changes no byte of the save."""
    server = Server(tmp_path / "save", hold=HOLD_S, pace=PACE_S)
    directory = tmp_path / "cognition"
    directory.mkdir()
    try:
        _prepare(directory, server.url)
        process = _cognition_process(directory)
        try:
            _joined(directory, process)
            alice = run(_alice_of(server.url), timeout_s=60)
            process.kill()
            process.wait()
            killed = time.monotonic()
            marked = _latest(server.save)
            _wait_for_sync(
                lambda: _requests_by(server.save, alice, marked) > 0,
                f"a request by Alice's in-server controller within {FALLBACK_BOUND_S} s",
                FALLBACK_BOUND_S,
            )
            elapsed = time.monotonic() - killed
        finally:
            if process.poll() is None:
                process.kill()
                process.wait()
        print(
            f"AP6-8 kill: Alice's fallback request {elapsed:.2f} s after SIGKILL (bound {FALLBACK_BOUND_S} s)"
        )
        assert elapsed <= FALLBACK_BOUND_S, "AP6-8: the fallback came after hold + pace + 2 s"
        server.stop()
        before = _tree_digest(server.save / "world.sqlite")
        shutil.rmtree(directory / "store")
        assert _tree_digest(server.save / "world.sqlite") == before, "AP6-15: the save changed"
    finally:
        server.stop()


def test_d_a_rejoin_with_the_persisted_secret_inside_the_hold_is_held(tmp_path: Path) -> None:
    """AP6-9 (part): after a SIGKILL, a join inside the hold presents the secret the process persisted (§5.10) and
    is welcomed `took_over: "held"`, so the seat is not handed to the fallback. The restart of the whole process
    is not run here: under replay (no model may run) a consult that finds no recorded request ends the process by
    design (`CassetteMiss`), and the in-process restart path is C11's (`test_controller_seat.py`)."""
    server = Server(tmp_path / "save", hold=HOLD_S, pace=PACE_S)
    directory = tmp_path / "cognition"
    directory.mkdir()
    try:
        _prepare(directory, server.url)
        process = _cognition_process(directory)
        try:
            _joined(directory, process)
        finally:
            process.kill()
            process.wait()
        killed = time.monotonic()
        secret = resume_file.read(directory / "store" / "alice.resume")

        async def rejoin() -> str:
            seat = await ResumingSeat.open(
                _connector(server.url),
                seat=ALICE,
                invite=Invite(INVITE),
                nickname="restart-probe",
                cursor=_Cursor(None),
                resume=secret,
                sleep=asyncio.sleep,
                clock=time.monotonic,
            )
            took_over = seat.welcome.took_over
            await seat.detach()
            return took_over

        took_over = run(rejoin(), timeout_s=60)
        elapsed = time.monotonic() - killed
        print(
            f"AP6-9: rejoin with the persisted secret {elapsed:.2f} s after SIGKILL: took_over {took_over!r}"
        )
        assert secret is not None, "AP6-9: no secret was persisted before the kill"
        assert elapsed < HOLD_S, "AP6-9: the rejoin was not inside the hold"
        assert took_over == "held", f"AP6-9: a rejoin inside the hold was {took_over!r}"
    finally:
        server.stop()


def _wait_for_sync(predicate: Callable[[], bool], what: str, within: float) -> None:
    deadline = time.monotonic() + within
    while not predicate():
        if time.monotonic() > deadline:
            pytest.fail(f"{what} did not happen")
        time.sleep(0.1)
