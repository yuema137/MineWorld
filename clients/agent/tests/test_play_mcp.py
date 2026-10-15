"""The play-test over stdio against the real server (step-27 §9.2, CP-d; pr-s27d §8 AC-d-1 … AC-d-5, AC-d-8).

The adapter is a child process, started by the official MCP client over stdio with its credential in the
environment (`MINEWORLD_GRANT`). It reaches the real server through `mcp_relay.Relay`, which records every frame the
adapter sends and every welcome the server sends. Every oracle is the server's: `/status`, the save's own export
(`perceived`, `biography`), a plain SDK client, and the relay's frames. Nothing waits on a sleep where a stream
exists. The file runs on the platform's default loop (F-S27d-6): the child is a subprocess.

The server is the real `mineworld` binary (`MINEWORLD_BIN`, built by `cargo build -p mineworld-cli`); a missing binary
fails the test, it never skips."""

from __future__ import annotations

import asyncio
import itertools
import json
import os
import sys
import time
from collections.abc import AsyncGenerator, Awaitable, Callable, Iterator
from contextlib import asynccontextmanager
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, cast

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.types import CallToolResult, TextContent
from mcp_relay import Relay
from mineworld_sdk.wire.ids import JsonValue
from realserver import (
    ADMIN_CANARY,
    INVITE,
    Server,
    market_town,
    seated,
)
from support import run_default

from mineworld_agent.models import ToolError

PATIENCE = 25.0
"""The longest one wait, or one retry, may take (wall seconds)."""
WALK_PATIENCE = 90.0
"""The ceiling for one walk to its end, in wall seconds (the world's own time runs at scale 1)."""
RESULT_CAP = 8192
REPOSITORY = Path(__file__).resolve().parents[3]
FRAMES = REPOSITORY / "server" / "tests" / "frames"


@dataclass
class Transcript:
    """Every answer the adapter gave, for AC-d-1 step 12 and AC-d-4."""

    sizes: list[int] = field(default_factory=list[int])
    texts: list[str] = field(default_factory=list[str])
    """Every answer's text and structured content, as the agent received them."""

    def record(self, answer: CallToolResult) -> None:
        text = answer.content[0]
        assert isinstance(text, TextContent)
        structured = (
            json.dumps(answer.structured_content, ensure_ascii=False, separators=(",", ":"))
            if answer.structured_content
            else ""
        )
        self.sizes.append(max(len(text.text.encode("utf-8")), len(structured.encode("utf-8"))))
        self.texts += [text.text, structured]

    def largest(self) -> int:
        return max(self.sizes, default=0)


def child_environment(grant: str | None) -> dict[str, str]:
    """This process's environment, minus every MINEWORLD_ variable, plus at most one grant."""
    base = {key: value for key, value in os.environ.items() if not key.startswith("MINEWORLD_")}
    if grant is not None:
        base["MINEWORLD_GRANT"] = grant
    return base


@asynccontextmanager
async def adapter(
    relay_url: str, state: Path, grant: str | None, log: Path
) -> AsyncGenerator[ClientSession]:
    """One adapter process over stdio, with its stderr kept in `log` (AC-d-4 reads it)."""
    parameters = StdioServerParameters(
        command=sys.executable,
        args=["-m", "mineworld_agent", "mcp", "--server", relay_url, "--state-dir", str(state)],
        env=child_environment(grant),
    )
    with log.open("w", encoding="utf-8") as errlog:
        async with stdio_client(parameters, errlog=errlog) as (read, write):
            async with ClientSession(read, write) as session:
                await session.initialize()
                yield session


def text_of(answer: CallToolResult) -> str:
    first = answer.content[0]
    assert isinstance(first, TextContent)
    return first.text


async def call(
    session: ClientSession, transcript: Transcript, tool: str, arguments: dict[str, Any]
) -> dict[str, Any]:
    """A successful call: its structured content, recorded. A refusal here fails the step."""
    answer = await session.call_tool(tool, arguments)
    transcript.record(answer)
    assert answer.is_error is not True, f"{tool}: {text_of(answer)}"
    return cast(dict[str, Any], answer.structured_content)


async def refused(
    session: ClientSession, transcript: Transcript, tool: str, arguments: dict[str, Any]
) -> ToolError:
    """A refused call: the typed `ToolError` of an `isError` result that has no structured content."""
    answer = await session.call_tool(tool, arguments)
    transcript.record(answer)
    assert answer.is_error is True, f"{tool} was answered, not refused"
    assert answer.structured_content is None
    return ToolError.model_validate_json(text_of(answer))


async def until(attempt: Callable[[], Awaitable[bool]], *, patience: float, what: str) -> None:
    """Retries `attempt` until it returns True, for at most `patience` wall seconds."""
    deadline = time.monotonic() + patience
    while not await attempt():
        if time.monotonic() > deadline:
            raise AssertionError(f"{what} did not happen within {patience:.0f} s")
        await asyncio.sleep(0.2)


def offer_of(view: dict[str, Any], action_type: str) -> dict[str, Any] | None:
    for offer in view["offers"]:
        if offer["action_type"] == action_type and offer["available"]:
            return cast(dict[str, Any], offer)
    return None


def person_ref(view: dict[str, Any], name: str) -> str | None:
    for person in view["here"]["people"]:
        if person["name"]["text"].startswith(name):
            return str(person["ref"])
    return None


async def walk(
    session: ClientSession, transcript: Transcript, handle: str, destination: dict[str, str]
) -> None:
    """One walk, waited to its end: each `wait` has its own ceiling, and the walk ends when it has arrived."""
    started = await call(session, transcript, "walk_to", {"handle": handle, "to": destination})
    assert started["outcome"]["kind"] == "accepted", started["outcome"]

    async def arrived() -> bool:
        woke = await call(
            session,
            transcript,
            "wait",
            {"handle": handle, "until": {"kind": "walk_ended"}, "timeout_s": 20},
        )
        if woke["woke"] == "walk_ended":
            return True
        view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
        return view["me"].get("walking") is None

    await until(arrived, patience=WALK_PATIENCE, what="the walk ended")


@pytest.fixture
def town(tmp_path: Path) -> Iterator[Server]:
    """The market-town of §8: the grants, the save the oracles read, a hold of 10 seconds at time-scale 1."""
    server = Server(
        "market-town",
        *market_town(
            "--save",
            str(tmp_path / "save"),
            "--grant",
            "visitor",
            "--grant",
            "visitor:take-over",
            "--grant",
            "alice",
            "--hold",
            "10",
            "--time-scale",
            "1",
        ),
    )
    try:
        yield server
    finally:
        server.stop()


def test_the_play_test_over_stdio(town: Server, tmp_path: Path) -> None:
    """AC-d-1 (§9.2's steps through the adapter), AC-d-2 (frames), AC-d-3 (the grant is the only right),
    AC-d-4 (no secret), AC-d-5 (pacing), each on the one real server."""
    run_default(_play(town, tmp_path), timeout_s=600)


async def _play(town: Server, tmp_path: Path) -> None:
    grants = town.grants()
    relay = Relay(town.url, town.http)
    relay_url = await relay.start()
    transcript = Transcript()
    state = tmp_path / "state"
    logs = tmp_path / "logs"
    logs.mkdir()
    try:
        # 1 to 7: join with the visitor grant, walk, talk, buy.
        async with adapter(relay_url, state, grants["visitor"], logs / "a.log") as session:
            seats = await call(session, transcript, "seats", {})
            assert "visitor" in seats["seats"]
            joined = await call(session, transcript, "join", {"seat": "visitor", "nickname": "Ada"})
            assert joined["took_over"] == "hosted"
            handle = str(joined["handle"])
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            assert offer_of(view, "walk-to") is not None

            café = view["me"]["place"]["ref"]
            street = view["here"]["exits"][0]["leads_to"]["ref"]
            await walk(session, transcript, handle, {"place": street})
            await walk(session, transcript, handle, {"place": café})

            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            alice = person_ref(view, "Alice")
            assert alice is not None, "Alice is not in the café"
            await walk(session, transcript, handle, {"person": alice})
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            said = await call(
                session, transcript, "say", {"handle": handle, "to": alice, "words": "hello"}
            )
            assert said["outcome"]["kind"] == "accepted", said["outcome"]
            await call(
                session,
                transcript,
                "wait",
                {"handle": handle, "until": {"kind": "spoken_to"}, "timeout_s": 20},
            )

            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            buy = offer_of(view, "buy")
            assert buy is not None, "no buy offer is available"
            wallet, holdings = (
                view["me"]["wallet"],
                sum(item["count"] for item in view["me"]["holdings"]),
            )
            bought = await call(
                session,
                transcript,
                "attempt",
                {"handle": handle, "view": view["view"], "offer": buy["n"]},
            )
            assert bought["outcome"]["kind"] == "accepted", bought["outcome"]
            after = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            assert sum(item["count"] for item in after["me"]["holdings"]) == holdings + 1
            assert after["me"]["wallet"] == wallet - buy["price"]

            # AC-d-3, the first half: the visitor grant alone does not seat alice.
            refusal = await refused(
                session, transcript, "join", {"seat": "alice", "nickname": "Ada"}
            )
            assert refusal.model_dump(exclude_none=True) == {
                "kind": "error",
                "code": "join_refused",
                "join": "not_granted",
            }

            # 8: the relay is cut; the next call answers a typed not_connected, and the handle resumes in its hold.
            # The cut: every transport aborted, no close frame. The seat reconnects on its own, with its resume
            # secret, and the server's new welcome is the oracle (`observe` answers from the seat's own state, so
            # a call is no probe). Whatever a call answers during the cut is a typed refusal, never a crash.
            welcomes_before = len(relay.welcomes)
            relay.abort()

            async def reconnected() -> bool:
                return len(relay.welcomes) > welcomes_before

            await until(reconnected, patience=PATIENCE, what="the seat reconnected after the cut")
            answer = await session.call_tool("observe", {"handle": handle})
            transcript.record(answer)
            assert answer.is_error is not True, text_of(answer)
            events = await _all_events(session, transcript, handle)
            refs = [event["ref"] for event in events]
            assert len(refs) == len(set(refs)), "a fact was shown twice"

            # 9: a plain SDK client takes the seat over. The adapter's next call says so (seat_lost, taken_over).
            # A visitor grant cannot take the seat back (AC-d-3); a visitor:take-over grant can (took_over
            # "connection"), and the plain client is displaced.
            plain = await seated(town, "visitor", take_over=True)
            try:
                lost: list[ToolError] = []

                async def displaced() -> bool:
                    answer = await session.call_tool("observe", {"handle": handle})
                    transcript.record(answer)
                    if answer.is_error is not True:
                        return False
                    lost.append(ToolError.model_validate_json(text_of(answer)))
                    return lost[-1].code == "seat_lost"

                await until(
                    displaced, patience=PATIENCE, what="the adapter learned it lost the seat"
                )
                assert lost[-1].seat_lost == "taken_over", lost[-1]
                async with adapter(relay_url, state, grants["visitor"], logs / "b.log") as other:
                    # A plain join onto a held seat is `seat_occupied`: a grant does not turn a join into a takeover.
                    held = await refused(other, transcript, "join", {"seat": "visitor", "nickname": "Ada"})
                    assert held.join == "seat_occupied", held
                    refusal = await refused(
                        other,
                        transcript,
                        "join",
                        {"seat": "visitor", "nickname": "Ada", "take_over": True},
                    )
                    assert refusal.join == "not_granted", refusal
                async with adapter(
                    relay_url, state, grants["visitor take-over"], logs / "c.log"
                ) as taker:
                    taken = await call(
                        taker,
                        transcript,
                        "join",
                        {"seat": "visitor", "nickname": "Ada", "take_over": True},
                    )
                    assert taken["took_over"] == "connection", taken["took_over"]
                    await call(taker, transcript, "leave", {"handle": str(taken["handle"])})
            finally:
                await plain.close()

        # 10: alice, with her own grant: joined, then given back.
        async with adapter(relay_url, state, grants["alice"], logs / "d.log") as session:
            alice_join = await call(
                session, transcript, "join", {"seat": "alice", "nickname": "Ada"}
            )
            await call(session, transcript, "leave", {"handle": str(alice_join["handle"])})

        # 11: every seat is back and the server counts no client (the baseline is zero clients).
        await until(_no_client(town), patience=PATIENCE, what="the server counted no client")

        # 12: every answer fits the cap (AC-d-1 step 12).
        assert transcript.largest() <= RESULT_CAP, transcript.largest()
        _check_frames(relay)
        _check_secrets(transcript, relay, town, logs)
        _check_pacing(relay)
    finally:
        await relay.close()


async def _all_events(
    session: ClientSession, transcript: Transcript, handle: str
) -> list[dict[str, Any]]:
    """Every fact the inbox holds, page by page (`events`, following `next`)."""
    collected: list[dict[str, Any]] = []
    after: str | None = None
    for _ in range(200):
        page = await call(
            session,
            transcript,
            "events",
            {"handle": handle, "after": after} if after else {"handle": handle},
        )
        collected += cast(list[dict[str, Any]], page["events"])
        after = page["next"]
        if after is None:
            return collected
    raise AssertionError("events never ended")


def _no_client(town: Server) -> Callable[[], Awaitable[bool]]:
    async def check() -> bool:
        return await asyncio.to_thread(lambda: town.status().clients == 0)

    return check


def _check_frames(relay: Relay) -> None:
    """AC-d-2: the adapter sent only join, submit and leave; each frame's key paths are a subset of the golden
    frame's (server/tests/frames) for its type; an act's frame carries `actor_location: null`."""
    types = {str(frame.get("t")) for frame in relay.sent}
    assert types <= {"join", "submit", "leave"}, types
    for kind in types:
        golden = json.loads((FRAMES / f"{kind}.json").read_text(encoding="utf-8"))
        allowed = _key_paths(golden)
        for frame in relay.frames_sent(kind):
            extra = _key_paths(frame) - allowed
            assert not extra, (kind, sorted(extra))
    for frame in relay.frames_sent("submit"):
        request = frame.get("request")
        assert isinstance(request, dict)
        assert request.get("actor_location") is None


PACK_OWNED = "request.payload"
"""The payload of a submit is the pack's own JSON (`mineworld_sdk.offers` encodes it, the same code a plain
client runs). Its keys are the pack's, not the envelope's, so the comparison stops at this path."""


def _key_paths(value: JsonValue, prefix: str = "") -> set[str]:
    found: set[str] = set()
    if isinstance(value, dict):
        for key, inner in value.items():
            path = f"{prefix}.{key}" if prefix else key
            found.add(path)
            if path != PACK_OWNED:
                found |= _key_paths(inner, path)
    return found


def _check_secrets(transcript: Transcript, relay: Relay, town: Server, logs: Path) -> None:
    """AC-d-4: no credential, grant, admin token or resume secret in any answer the agent received, or in any
    adapter's stderr. The resume file in the state directory is the one place a secret may be (not read here)."""
    secrets = {INVITE, ADMIN_CANARY, *town.grants().values()}
    for welcome in relay.welcomes:
        resume = welcome.get("resume")
        if isinstance(resume, str):
            secrets.add(resume)
    everything = "\n".join(transcript.texts)
    for log in logs.glob("*.log"):
        everything += log.read_text(encoding="utf-8")
    for secret in secrets:
        assert secret not in everything, "a secret reached the agent or a log"


def _check_pacing(relay: Relay) -> None:
    """AC-d-5: the walk's steps reached the server no faster than 1.2 a wall second."""
    steps = sorted(
        when
        for when, frame in relay.timed_sent
        if frame.get("t") == "submit"
        and isinstance(frame.get("request"), dict)
        and cast(dict[str, Any], frame["request"]).get("action_type") == "walk-step"
    )
    assert len(steps) >= 2, (
        f"the walks sent {len(steps)} walk steps; the pacing check needs several"
    )
    for earlier, later in itertools.pairwise(steps):
        assert later - earlier >= 1 / 1.2 - 0.05, (earlier, later)
