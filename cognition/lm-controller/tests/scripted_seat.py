"""A fake server for the seat runner's tests: scripted connections, a fake clock, and wire frames (C11).

Copied from `sdk/python/tests/scripted.py` (P3b's fake, D-P3b-9), with the helpers this member needs: a `spoke` fact
with its speaker and listener, and an observation with a place. A member's tests may not import another member's test
modules (P6b C11), so the copy lives here. Nothing here opens a socket. Only the wire form changed: the frames are the
ones `server/tests/frames/` pins, and the SDK validates every one the runner reads.
"""

from __future__ import annotations

import asyncio
import json
from collections import deque
from collections.abc import Callable
from dataclasses import dataclass, field
from typing import Any

from websockets.exceptions import ConnectionClosedError

OBSERVER = "101"
INSTANCE = "1a2b3c4d5e6f70819293a4b5c6d7e8f9"
INVITE = "mwsdk-marker-invite-7f3a9c2e5b1d"
"""32 characters, so a scan finds it anywhere (AP3b-17)."""
PLACE = "3"
_DROP = object()


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
) -> str:
    world: dict[str, Any] = {
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
            "protocol": 2,
            "seat": "alice",
            "observer": observer,
            "nickname": "tester",
            "session": str(resume),
            "resume": secret(resume),
            "hold_seconds": hold,
            "took_over": took_over,
            "world": world,
        }
    )


def observation(seq: int, *, at: int | None = None, place: str | None = PLACE) -> str:
    where = (
        None
        if place is None
        else {"place": {"entity": place, "entity_type": "place"}, "local": None, "facing": None}
    )
    return json.dumps(
        {
            "t": "observation",
            "seq": seq,
            "revision": 1,
            "acted_through": None,
            "observation": {
                "observer": OBSERVER,
                "at": seq if at is None else at,
                "self_location": where,
                "entities": [],
                "relations": [],
                "events": [],
                "affordances": [],
            },
        }
    )


def _ref(entity: str, entity_type: str = "person") -> dict[str, str]:
    return {"entity": entity, "entity_type": entity_type}


def spoke(
    event_id: str, *, speaker: str, listener: str = OBSERVER, utterance: str = "hi"
) -> dict[str, Any]:
    """A `spoke` fact in the server's perceived wire form, with the payload the conversation pack writes."""
    return {
        "id": event_id,
        "at": 4,
        "event_type": "spoke",
        "subjects": [listener],
        "participants": [speaker, listener],
        "place": _ref(PLACE, "place"),
        "caused_by": {"action": "41"},
        "payload": {
            "event_type": "spoke",
            "schema_version": 1,
            "payload": {
                "speaker": _ref(speaker),
                "listener": _ref(listener),
                "utterance": utterance,
            },
        },
        "visibility": {"place": _ref(PLACE, "place")},
        "provenance": {"emitted_by": "conversation", "controller_decision": "41"},
    }


def perceived(through: str, *facts: dict[str, Any]) -> str:
    return json.dumps({"t": "perceived", "through": through, "events": list(facts)})


def refused(code: str, token: str | None = None) -> str:
    frame: dict[str, Any] = {"t": "refused", "code": code}
    if token is not None:
        frame["token"] = token
    return json.dumps(frame)


def closing(reason: str) -> str:
    return json.dumps({"t": "closing", "reason": reason})


class Link:
    """One scripted socket. A `leave` is answered `closing { left }`, as the server answers it (§2)."""

    def __init__(self, *frames: str) -> None:
        self.inbound: asyncio.Queue[object] = asyncio.Queue()
        self.sent: list[dict[str, Any]] = []
        self.closed = False
        for frame in frames:
            self.inbound.put_nowait(frame)

    async def send(self, message: str, /) -> None:
        if self.closed:
            raise ConnectionClosedError(None, None)
        frame: dict[str, Any] = json.loads(message)
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

    def kinds(self) -> list[str]:
        return [str(frame["t"]) for frame in self.sent]

    def joins(self) -> list[dict[str, Any]]:
        return [frame for frame in self.sent if frame["t"] == "join"]


@dataclass
class Gate:
    """A connect that waits until the test opens it, then hands out `link`."""

    link: Link
    opened: asyncio.Event = field(default_factory=asyncio.Event)


type Plan = Link | BaseException | Gate


class ScriptedServer:
    """A connector handing out planned connections in order; when the plans run out, every further attempt
    fails to connect, as an unreachable server does."""

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


async def until(condition: Callable[[], bool], what: str, rounds: int = 2_000) -> None:
    """Yields to the loop until `condition()` holds; a scripted run settles in a few rounds."""
    for _ in range(rounds):
        if condition():
            return
        await asyncio.sleep(0)
    raise AssertionError(f"never: {what}")
