"""The guide's agent session (pr-s27f §5.2): recorded once into `docs/guide/examples/agent-session.json`, replayed by
`test_agent_session.py` against a fresh real server, and shown verbatim by chapter 14 (`test_guide_chapter.py`).

The fixture is a recorded excerpt (QS27f-3): every call as the agent made it, with the values that change between
runs written as placeholders (`<handle>`, `<ref:NAME>`, `<view>`, `<offer:ACTION>`), and for each answer the values
at a fixed list of paths per tool (`EXCERPT`), with volatile values written `"…"`. The server is the real binary on
the guide's finished Corner Shop, started with chapter 14's flags; the adapter is a child process over stdio,
reached through the official MCP client (S27-d's harness).

Record (a developer action, never part of a test run), from the repository root:

    uv run --locked python clients/agent/tests/agent_session.py --record docs/guide/examples/agent-session.json
"""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
import time
from importlib.metadata import version
from pathlib import Path
from typing import Any, Final, cast

from mcp import ClientSession
from mcp.types import CallToolResult, TextContent
from realserver import REPOSITORY, Server
from support import run_default
from test_play_mcp import adapter

WORLD: Final = REPOSITORY / "docs" / "guide" / "examples" / "corner-shop"
FIXTURE: Final = REPOSITORY / "docs" / "guide" / "examples" / "agent-session.json"
SEAT: Final = "ben"
SERVER_FLAGS: Final = (
    "--town",
    "--agent",
    "ana",
    "--grant",
    SEAT,
    "--hold",
    "10",
    "--time-scale",
    "1",
)
"""Chapter 14's server flags (step 14.2), with a short hold and real time, as S27-d's play-test runs them."""
VOLATILE: Final = "…"
VOLATILE_KEYS: Final = frozenset(
    {"at", "base_view", "by", "handle", "item", "ref", "view", "world_seconds_since_last_call"}
)
"""Keys whose values change between runs: references, view numbers, instants. Masked wherever they occur."""
WALK_PATIENCE: Final = 90.0
"""The ceiling for one walk to its end, in wall seconds."""
REPLY_PATIENCE: Final = 60.0
"""The ceiling for the shopkeeper's reply, in wall seconds."""
SESSION_PATIENCE: Final = 180.0
"""The ceiling for the whole session (the contract's material stop: the replay over 180 s on Linux CI)."""

SCRIPT: Final[tuple[dict[str, Any], ...]] = (
    {"tool": "seats", "arguments": {}},
    {"tool": "join", "arguments": {"seat": SEAT, "nickname": "agent"}},
    {"tool": "observe", "arguments": {"handle": "<handle>"}},
    {
        "tool": "act",
        "arguments": {
            "handle": "<handle>",
            "act": {"kind": "walk_to", "to": {"place": "<ref:shop>"}},
        },
    },
    {
        "tool": "wait",
        "arguments": {"handle": "<handle>", "until": {"kind": "walk_ended"}, "timeout_s": 20},
    },
    {
        "tool": "act",
        "arguments": {
            "handle": "<handle>",
            "act": {"kind": "walk_to", "to": {"person": "<ref:Ana>"}},
        },
    },
    {
        "tool": "wait",
        "arguments": {"handle": "<handle>", "until": {"kind": "walk_ended"}, "timeout_s": 20},
    },
    {
        "tool": "act",
        "arguments": {
            "handle": "<handle>",
            "act": {"kind": "say", "to": "<ref:Ana>", "words": "Hello, Ana."},
        },
    },
    {
        "tool": "wait",
        "arguments": {"handle": "<handle>", "until": {"kind": "spoken_to"}, "timeout_s": 20},
    },
    {"tool": "observe", "arguments": {"handle": "<handle>"}},
    {
        "tool": "act",
        "arguments": {
            "handle": "<handle>",
            "act": {"kind": "attempt", "view": "<view>", "offer": "<offer:buy>"},
        },
    },
    {"tool": "observe", "arguments": {"handle": "<handle>", "since_view": "<view>"}},
    {"tool": "leave", "arguments": {"handle": "<handle>"}},
)
"""The calls of the session, in order (pr-s27f §5.2, as amended for S27-d1's single `act` tool)."""

EXCERPT: Final[dict[str, tuple[str, ...]]] = {
    "seats": ("seats",),
    "join": ("seat", "took_over", "view.me.place"),
    "observe": ("view.kind", "view.me.place", "view.me.wallet", "view.me.holdings"),
    "act": ("outcome.kind",),
    "wait": ("woke",),
    "leave": ("closing",),
}
"""The dotted paths each tool's answer shows on the page. `[0]` indexes a list. A `wait` shows only why it woke: the
order of the facts it carries (an arrival, a new acquaintance, a reply) is the world's, not the agent's."""


class SessionError(AssertionError):
    """The live session did not do what the fixture shows."""


# ── Paths and masks ───────────────────────────────────────────────────────────────────────────────


_MISSING: Final = object()


def at_path(value: object, path: str) -> object:
    """The value at a dotted path (`events[0].words.kind`), or `_MISSING`."""
    current = value
    for part in path.split("."):
        name, _, index = part.partition("[")
        if not isinstance(current, dict) or name not in current:
            return _MISSING
        current = cast(dict[str, object], current)[name]
        if index:
            position = int(index.rstrip("]"))
            if not isinstance(current, list) or position >= len(cast(list[object], current)):
                return _MISSING
            current = cast(list[object], current)[position]
    return current


def masked(value: object, key: str = "") -> object:
    """`value` with every volatile key's value written `"…"` (unless it is null), and a resident's words' text
    too."""
    if key in VOLATILE_KEYS and value is not None:
        return VOLATILE
    if isinstance(value, dict):
        entries = cast(dict[str, object], value)
        kept = {name: masked(inner, name) for name, inner in entries.items()}
        if kept.get("kind") in ("speech", "said") and "text" in kept:
            kept["text"] = VOLATILE
        return kept
    if isinstance(value, list):
        return [masked(inner) for inner in cast(list[object], value)]
    return value


def shows(tool: str, structured: dict[str, Any]) -> dict[str, object]:
    """The excerpt of one answer: each of the tool's paths that the answer has, masked."""
    shown: dict[str, object] = {}
    for path in EXCERPT[tool]:
        found = at_path(structured, path)
        if found is not _MISSING:
            last = path.rsplit(".", 1)[-1].split("[")[0]
            shown[path] = masked(found, last)
    return shown


def json_type(value: object) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if isinstance(value, int | float):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, list):
        return "array"
    return "object"


# ── The live session ──────────────────────────────────────────────────────────────────────────────


class Live:
    """What placeholders resolve from: the handle, the newest whole view and its number."""

    def __init__(self) -> None:
        self.handle: str | None = None
        self.view: dict[str, Any] | None = None
        self.view_number: int | None = None

    def keep(self, tool: str, structured: dict[str, Any]) -> None:
        if tool == "join":
            self.handle = str(structured["handle"])
        seen = structured.get("view")
        if isinstance(seen, dict):
            seen = cast(dict[str, Any], seen)
            self.view_number = int(seen["view"])
            if seen.get("kind") == "view":
                self.view = seen

    def resolve(self, value: object) -> object:
        if isinstance(value, dict):
            return {
                key: self.resolve(inner) for key, inner in cast(dict[str, object], value).items()
            }
        if not isinstance(value, str) or not value.startswith("<"):
            return value
        if value == "<handle>":
            return self._need(self.handle, value)
        if value == "<view>":
            return self._need(self.view_number, value)
        if value.startswith("<ref:"):
            return self._ref(value[5:-1])
        if value.startswith("<offer:"):
            return self._offer(value[7:-1])
        return value

    @staticmethod
    def _need[T](found: T | None, placeholder: str) -> T:
        if found is None:
            raise SessionError(f"{placeholder}: nothing to resolve it from yet")
        return found

    def _ref(self, name: str) -> str:
        view = self._need(self.view, f"<ref:{name}>")
        wanted = name.lower()
        candidates: list[tuple[str, list[str]]] = []
        place = view["me"].get("place")
        if place is not None:
            candidates.append((place["ref"], [label["text"] for label in place["labels"]]))
        for person in view["here"]["people"]:
            names = [person["name"]["text"]] if person.get("name") else []
            candidates.append((person["ref"], names))
        for thing in view["here"]["things"]:
            names = [thing["name"]["text"]] if thing.get("name") else []
            candidates.append((thing["ref"], names + [label["text"] for label in thing["labels"]]))
        for exit_line in view["here"]["exits"]:
            leads = exit_line["leads_to"]
            candidates.append((leads["ref"], [label["text"] for label in leads["labels"]]))
        for ref, names in candidates:
            if any(text.lower().startswith(wanted) for text in names):
                return str(ref)
        raise SessionError(f"<ref:{name}>: no one and nothing by that name in the newest view")

    def _offer(self, action_type: str) -> int:
        view = self._need(self.view, f"<offer:{action_type}>")
        for offer in view["offers"]:
            if offer["action_type"] == action_type and offer["available"]:
                return int(offer["n"])
        raise SessionError(
            f"<offer:{action_type}>: no available offer of that action in the newest view"
        )


def text_of(answer: CallToolResult) -> str:
    first = answer.content[0]
    assert isinstance(first, TextContent)
    return first.text


async def one_call(
    session: ClientSession, live: Live, call: dict[str, Any]
) -> tuple[str | None, dict[str, Any]]:
    """One call with its placeholders resolved: (the error code or None, the structured content)."""
    arguments = cast(dict[str, Any], live.resolve(call["arguments"]))
    answer = await session.call_tool(call["tool"], arguments)
    if answer.is_error:
        return str(json.loads(text_of(answer))["code"]), {}
    structured = cast(dict[str, Any], answer.structured_content)
    live.keep(call["tool"], structured)
    return None, structured


async def until_arrived(
    session: ClientSession, live: Live, call: dict[str, Any], destination: dict[str, Any]
) -> tuple[str | None, dict[str, Any]]:
    """A `wait {walk_ended}`, repeated until the seat stands still at `destination` (a place, or a person).

    The first answer is what the page shows; the repeats and the observes between them are the replayer's own."""
    deadline = time.monotonic() + WALK_PATIENCE
    first: tuple[str | None, dict[str, Any]] | None = None
    while True:
        answered = await one_call(session, live, call)
        first = first or answered
        if answered[0] is not None:
            return first
        _, observed = await one_call(
            session, live, {"tool": "observe", "arguments": {"handle": "<handle>"}}
        )
        me = observed["view"]["me"]
        here = me.get("place")
        place = destination.get("place")
        arrived = me.get("walking") is None and (
            place is None or (here is not None and here["ref"] == place)
        )
        if arrived:
            return first
        if time.monotonic() > deadline:
            raise SessionError(f"the walk did not end within {WALK_PATIENCE:.0f} s")


async def until_answered(
    session: ClientSession, live: Live, call: dict[str, Any]
) -> tuple[str | None, dict[str, Any]]:
    """A `wait {spoken_to}`, repeated until it wakes for speech (a reply takes the resident a moment)."""
    deadline = time.monotonic() + REPLY_PATIENCE
    while True:
        answered = await one_call(session, live, call)
        if answered[0] is not None or answered[1].get("woke") == "spoken_to":
            return answered
        if time.monotonic() > deadline:
            raise SessionError(f"nobody answered within {REPLY_PATIENCE:.0f} s")


async def run_session(
    session: ClientSession, calls: list[dict[str, Any]]
) -> list[tuple[str | None, dict[str, Any]]]:
    """Every call in order, each answered by the live server."""
    live = Live()
    answers: list[tuple[str | None, dict[str, Any]]] = []
    destination: dict[str, Any] = {}
    for call in calls:
        until = call["arguments"].get("until", {})
        if call["tool"] == "wait" and until.get("kind") == "walk_ended":
            answers.append(await until_arrived(session, live, call, destination))
            continue
        if call["tool"] == "wait" and until.get("kind") == "spoken_to":
            answers.append(await until_answered(session, live, call))
            continue
        act = call["arguments"].get("act", {})
        if call["tool"] == "act" and act.get("kind") == "walk_to":
            destination = cast(dict[str, Any], live.resolve(act["to"]))
        answers.append(await one_call(session, live, call))
    return answers


async def against_the_server(
    calls: list[dict[str, Any]], scratch: Path, world: Path = WORLD
) -> list[tuple[str | None, dict[str, Any]]]:
    """Starts the real server on Corner Shop with chapter 14's flags and plays `calls` through the adapter."""
    server = Server(str(world), *SERVER_FLAGS, "--save", str(scratch / "save"))
    try:
        grant = server.grants()[SEAT]
        async with adapter(
            server.url, scratch / "state", grant, scratch / "adapter.log"
        ) as session:
            return await run_session(session, calls)
    finally:
        server.stop()


# ── Record and replay ─────────────────────────────────────────────────────────────────────────────


def fixture_text(steps: list[dict[str, Any]]) -> str:
    body = {
        "recorded_with": f"mineworld-agent {version('mineworld-agent')}",
        "seat": SEAT,
        "steps": steps,
        "world": "corner-shop",
    }
    return json.dumps(body, indent=2, ensure_ascii=False, sort_keys=True) + "\n"


def record(path: Path, world: Path = WORLD) -> None:
    """Plays `SCRIPT` once against a fresh server and writes the fixture."""
    calls = [dict(call) for call in SCRIPT]
    with tempfile.TemporaryDirectory() as scratch:
        answers = run_default(
            against_the_server(calls, Path(scratch), world), timeout_s=SESSION_PATIENCE
        )
    steps = [
        {
            "call": call,
            "error": error,
            "shows": shows(call["tool"], structured) if error is None else {},
        }
        for call, (error, structured) in zip(calls, answers, strict=True)
    ]
    path.write_text(fixture_text(steps), encoding="utf-8", newline="\n")


def compare(
    steps: list[dict[str, Any]], answers: list[tuple[str | None, dict[str, Any]]]
) -> list[str]:
    """Every difference between the fixture's steps and a fresh run's answers, as readable lines."""
    problems: list[str] = []
    for number, (step, (error, structured)) in enumerate(zip(steps, answers, strict=True), start=1):
        tool = step["call"]["tool"]
        if error != step["error"]:
            problems.append(
                f"step {number} ({tool}): error {error!r}, the fixture shows {step['error']!r}"
            )
            continue
        for path, recorded in step["shows"].items():
            found = at_path(structured, path)
            if found is _MISSING:
                problems.append(f"step {number} ({tool}): {path} is missing")
                continue
            last = path.rsplit(".", 1)[-1].split("[")[0]
            live = masked(found, last)
            if json_type(live) != json_type(recorded):
                problems.append(
                    f"step {number} ({tool}): {path} is {json_type(live)}, the fixture shows {json_type(recorded)}"
                )
            elif recorded != VOLATILE and live != recorded:
                problems.append(
                    f"step {number} ({tool}): {path} is {live!r}, the fixture shows {recorded!r}"
                )
    return problems


def replay(fixture: dict[str, Any], scratch: Path, world: Path = WORLD) -> list[str]:
    """Plays the fixture's calls against a fresh server: the differences (none when the page is true)."""
    steps = cast(list[dict[str, Any]], fixture["steps"])
    answers = run_default(
        against_the_server([step["call"] for step in steps], scratch, world),
        timeout_s=SESSION_PATIENCE,
    )
    return compare(steps, answers)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0] if __doc__ else None)
    parser.add_argument("--record", type=Path, required=True, help="the fixture to write")
    parser.add_argument(
        "--world",
        type=Path,
        default=WORLD,
        help="the World Pack to play (default: the guide's Corner Shop)",
    )
    arguments = parser.parse_args(argv)
    record(arguments.record, arguments.world.resolve())
    return 0


if __name__ == "__main__":
    sys.exit(main())
