"""A server the test scripts, connection by connection, with a fake clock (design §4.1, C4).

`ScriptedServer` is a `Connector`: each call hands out the next planned connection, raises the next
planned exception (a connect failure), or waits at a `Gate` the test opens. A `Link` is one scripted
socket: the frames the server will send are queued in it, and every frame the client sends is recorded,
decoded. `FakeClock` is the seat's wall clock and its sleep: a sleep is recorded and moves the clock, so
backoff and holds are tested without waiting (D-P3b-9). Nothing here opens a socket.
"""

from __future__ import annotations

import asyncio
import json
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

from websockets.exceptions import ConnectionClosedError

from mineworld_sdk.wire.ids import JsonValue

FRAMES = Path(__file__).resolve().parents[3] / "server" / "tests" / "frames"
OBSERVER = "101"
INSTANCE = "1a2b3c4d5e6f70819293a4b5c6d7e8f9"
INVITE = "mwsdk-marker-invite-7f3a9c2e5b1d"
"""32 characters, so a scan finds it anywhere (AP3b-17)."""
_DROP = object()


def golden_text(kind: str) -> str:
    """A reviewed server frame, byte for byte (`server/tests/frames/`)."""
    return (FRAMES / f"{kind}.json").read_text(encoding="utf-8")


def secret(n: int) -> str:
    """The n-th resume secret a scripted server hands out: 32 lowercase hexadecimal characters."""
    return f"{n:x}".rjust(32, "e")


def welcome(
    *,
    resume: int = 1,
    hold: int = 30,
    took_over: str = "none",
    instance: str = INSTANCE,
    observer: str = OBSERVER,
    protocol: int = 2,
) -> str:
    world: JsonValue = {
        "protocol": 2,
        "instance": instance,
        "at": 0,
        "time_scale": 1,
        "paused": False,
        "entities": 3,
        "systems": [],
        "seats": ["wanderer"],
        "clients": 1,
        "observations_dropped": 0,
        "events_dropped": 0,
        "faults": 0,
        "revision": 1,
    }
    return json.dumps(
        {
            "t": "welcome",
            "protocol": protocol,
            "seat": "wanderer",
            "observer": observer,
            "nickname": "tester",
            "session": str(resume),
            "resume": secret(resume),
            "hold_seconds": hold,
            "took_over": took_over,
            "world": world,
        }
    )


def observation(seq: int, acted_through: str | None = None) -> str:
    return json.dumps(
        {
            "t": "observation",
            "seq": seq,
            "revision": 1,
            "acted_through": acted_through,
            "observation": {
                "observer": OBSERVER,
                "at": seq,
                "self_location": None,
                "entities": [],
                "relations": [],
                "events": [],
                "affordances": [],
            },
        }
    )


def delta(seq: int) -> str:
    return json.dumps(
        {
            "t": "delta",
            "seq": seq,
            "base": seq - 1,
            "revision": 1,
            "acted_through": None,
            "delta": {"at": seq, "events": []},
        }
    )


def fact(event_id: str) -> JsonValue:
    return {
        "id": event_id,
        "at": 4,
        "event_type": "spoke",
        "subjects": ["5"],
        "participants": ["5", OBSERVER],
        "place": {"entity": "3", "entity_type": "place"},
        "caused_by": {"action": "41"},
        "payload": {"event_type": "spoke", "schema_version": 1, "payload": {"utterance": "hi"}},
        "visibility": {"place": {"entity": "3", "entity_type": "place"}},
        "provenance": {"emitted_by": "conversation", "controller_decision": "41"},
    }


def perceived(through: str, *ids: str) -> str:
    return json.dumps({"t": "perceived", "through": through, "events": [fact(i) for i in ids]})


def refused(code: str, token: str | None = None) -> str:
    frame: dict[str, JsonValue] = {"t": "refused", "code": code}
    if token is not None:
        frame["token"] = token
    return json.dumps(frame)


def closing(reason: str) -> str:
    return json.dumps({"t": "closing", "reason": reason})


def result(token: str, action_id: str) -> str:
    return json.dumps(
        {"t": "result", "token": token, "action_id": action_id, "result": "unavailable"}
    )


class Link:
    """One scripted socket. A `leave` is answered `closing { left }`, as the server answers it (§2)."""

    def __init__(self, *frames: str) -> None:
        self.inbound: asyncio.Queue[object] = asyncio.Queue()
        self.sent: list[dict[str, JsonValue]] = []
        self.closed = False
        for frame in frames:
            self.inbound.put_nowait(frame)

    async def send(self, message: str, /) -> None:
        if self.closed:
            raise ConnectionClosedError(None, None)
        frame = json.loads(message)
        self.sent.append(frame)
        if frame["t"] == "leave":
            self.say(closing("left"))

    async def recv(self) -> str:
        item = await self.inbound.get()
        if item is _DROP:
            self.closed = True
            raise ConnectionClosedError(None, None)
        assert isinstance(item, str)
        return item

    async def close(self) -> None:
        self.closed = True

    def say(self, *frames: str) -> None:
        for frame in frames:
            self.inbound.put_nowait(frame)

    def drop(self) -> None:
        """The socket ends with no `closing`, as a dropped connection does."""
        self.inbound.put_nowait(_DROP)

    def kinds(self) -> list[JsonValue]:
        return [frame["t"] for frame in self.sent]

    def join(self) -> dict[str, JsonValue]:
        (frame,) = [frame for frame in self.sent if frame["t"] == "join"]
        return frame


@dataclass
class Gate:
    """A connect that waits until the test opens it, then hands out `link`."""

    link: Link
    opened: asyncio.Event = field(default_factory=asyncio.Event)


type Plan = Link | BaseException | Gate


class ScriptedServer:
    """A `Connector` handing out planned connections in order; when the plans run out, every further
    attempt fails to connect, as an unreachable server does."""

    def __init__(self, *plans: Plan) -> None:
        self.plans: deque[Plan] = deque(plans)
        self.attempts = 0
        self.links: list[Link] = []

    def plan(self, *plans: Plan) -> None:
        self.plans.extend(plans)

    async def __call__(self) -> Link:
        self.attempts += 1
        plan = self.plans.popleft() if self.plans else ConnectionRefusedError("no server")
        if isinstance(plan, BaseException):
            raise plan
        if isinstance(plan, Gate):
            await plan.opened.wait()
            plan = plan.link
        self.links.append(plan)
        return plan


class FakeClock:
    """The seat's wall clock and sleep: a sleep is recorded and moves the clock at once."""

    def __init__(self) -> None:
        self.now = 1_000.0
        self.sleeps: list[float] = []

    def time(self) -> float:
        return self.now

    async def sleep(self, seconds: float) -> None:
        self.sleeps.append(seconds)
        self.now += seconds
        await asyncio.sleep(0)


async def settle(rounds: int = 20) -> None:
    for _ in range(rounds):
        await asyncio.sleep(0)


async def until(condition: Callable[[], bool], what: str, rounds: int = 2_000) -> None:
    """Yields to the loop until `condition()` holds; a scripted run settles in a few rounds."""
    for _ in range(rounds):
        if condition():
            return
        await asyncio.sleep(0)
    raise AssertionError(f"never: {what}")
