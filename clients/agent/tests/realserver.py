"""The real `mineworld` binary for this member's `real_server` tests (pr-s27c F-S27c-7).

Nothing here is mocked: the binary cargo built, the repository's own market-town, a real WebSocket on
loopback. It follows the SDK's own helper (`sdk/python/tests/realserver.py`) — an ephemeral port whose real
address the server prints, the invite and a canary admin token given by environment so they are never
printed, the process killed when the test ends — and adds the flags a town needs (`--town`, `--agent`,
`--time-scale`). It is this member's own: tests do not import across members (C-11).

`Wire` is the connector a test gives `Player(connector=…)`: every socket it opens is a `Tap` that records
each frame the Player's connection sent and each frame it received, and that the test can cut as a network
would (no close frame, no `leave`). The oracles are the server's — `/status`, `mineworld perceived`,
`mineworld biography`, a second client's own observation — never the Player's report of itself.
"""

from __future__ import annotations

import asyncio
import json
import os
import queue
import re
import subprocess
import sys
import threading
import time
import urllib.request
from collections.abc import Awaitable, Callable
from dataclasses import dataclass, field
from pathlib import Path

from mineworld_sdk.session import SeatSession
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.frames import Invite, WorldSummary
from mineworld_sdk.wire.ids import EntityKey, JsonValue
from websockets.asyncio.client import ClientConnection
from websockets.asyncio.client import connect as websocket_connect

from mineworld_agent.models import AgentModel, json_bytes, render_text

REPOSITORY = Path(__file__).resolve().parents[3]
BUILD = "cargo build -p mineworld-cli"
INVITE = "mwagent-marker-invite-5d2b9e4f1a7c"
"""The invite every test server is started with; a scan finds it anywhere (AC-c-9)."""
ADMIN_CANARY = "mwagent-admin-canary-8e1f4c2a9b3d"
"""A canary admin token in the server's environment: it must appear nowhere the Player writes (I-c-9)."""
STARTUP_PATIENCE = 30.0
PATIENCE = 30.0
_LISTENING = re.compile(r"listening on (?P<http>http://\S+) \((?P<url>ws://[^)\s]+)\)")
"""The server's grant line (`tools/cli/src/invite.rs`): seats, an optional take-over, the token."""
_GRANT = re.compile(
    r"^\[mineworld\] grant (?P<seats>\S+)(?P<takeover> take-over)? (?P<token>\S+) — "
)


def world_path(world: str) -> Path:
    """A world of the repository by its name (`market-town`), or a scratch copy by its absolute path."""
    return Path(world) if Path(world).is_absolute() else REPOSITORY / "worlds" / world


def binary() -> Path:
    """`MINEWORLD_BIN`, or the debug build under the repository's `target/`."""
    named = os.environ.get("MINEWORLD_BIN")
    if named:
        return Path(named)
    suffix = ".exe" if sys.platform == "win32" else ""
    return REPOSITORY / "target" / "debug" / f"mineworld{suffix}"


class Server:
    """One running `mineworld server worlds/<world> …`."""

    def __init__(self, world: str, *flags: str) -> None:
        executable = binary()
        if not executable.is_file():
            raise FileNotFoundError(
                f"the mineworld binary is not at {executable}. Build it first with `{BUILD}` "
                "(or set MINEWORLD_BIN); these tests start the real server and never skip"
            )
        self.world = world
        environment = dict(os.environ)
        environment["MINEWORLD_INVITE"] = INVITE
        environment["MINEWORLD_ADMIN_TOKEN"] = ADMIN_CANARY
        self._process = subprocess.Popen(
            [
                str(executable),
                "server",
                str(world_path(world)),
                "--listen",
                "127.0.0.1:0",
                *flags,
            ],
            cwd=REPOSITORY,
            env=environment,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        self._stdout: queue.Queue[str | None] = queue.Queue()
        self.lines: list[str] = []
        """Every line the server printed to stdout, in order (the grant lines among them)."""
        self.stderr: list[str] = []
        threading.Thread(target=self._drain_stdout, daemon=True).start()
        threading.Thread(target=self._drain_stderr, daemon=True).start()
        self.url, self.http = self._wait_for_url()

    def _drain_stdout(self) -> None:
        assert self._process.stdout is not None
        for line in self._process.stdout:
            self.lines.append(line)
            self._stdout.put(line)
        self._stdout.put(None)

    def grants(self) -> dict[str, str]:
        """The grant tokens the server printed at start, keyed by seats and suffix: `visitor` for a grant of
        `--grant visitor`, `visitor take-over` for `--grant visitor:take-over` (S27-a's line format). The
        tokens are the test's own secrets; they are never printed by the test."""
        deadline = time.monotonic() + STARTUP_PATIENCE
        while time.monotonic() < deadline and not any(
            line.startswith("[mineworld] hold ") for line in list(self.lines)
        ):
            time.sleep(
                0.05
            )  # the start-up lines follow the listening line; the hold line is the last of them
        found: dict[str, str] = {}
        for line in list(self.lines):
            if match := _GRANT.match(line):
                key = match.group("seats") + (" take-over" if match.group("takeover") else "")
                found[key] = match.group("token")
        return found

    def _drain_stderr(self) -> None:
        assert self._process.stderr is not None
        self.stderr.extend(self._process.stderr)

    def _wait_for_url(self) -> tuple[str, str]:
        deadline = time.monotonic() + STARTUP_PATIENCE
        while (left := deadline - time.monotonic()) > 0:
            try:
                line = self._stdout.get(timeout=left)
            except queue.Empty:
                break
            if line is None:
                break
            if match := _LISTENING.search(line):
                return match.group("url"), match.group("http")
        self.stop()
        raise RuntimeError(
            f"the server did not report its address within {STARTUP_PATIENCE:.0f} s; "
            f"stderr:\n{''.join(self.stderr)}"
        )

    def status(self) -> WorldSummary:
        with urllib.request.urlopen(f"{self.http}/status", timeout=10) as answer:
            body: bytes = answer.read()
        return WorldSummary.model_validate_json(body)

    def stop(self) -> None:
        """Kills the process and waits for it, so its save is closed before anything reads it."""
        self._process.kill()
        self._process.wait()


def market_town(*flags: str) -> tuple[str, ...]:
    """The scenario's server flags (pr-s27c §8): the town lives, Alice answers."""
    return ("--town", "--agent", "alice", *flags)


def lakeside(*flags: str) -> tuple[str, ...]:
    """Lakeside's server flags (S33-i): the pack roots every command on that world names (its world.yaml), so
    its Entity Pack and Presentation Packs resolve."""
    return (
        "--packs",
        str(REPOSITORY / "entities"),
        "--packs",
        str(REPOSITORY / "presentation" / "mineworld-default"),
        *flags,
    )


def _export(command: str, world: str, save: Path, person: str) -> list[str]:
    done = subprocess.run(
        [
            str(binary()),
            command,
            str(REPOSITORY / "worlds" / world),
            "--save",
            str(save),
            "--person",
            person,
            "--json",
        ],
        cwd=REPOSITORY,
        capture_output=True,
        text=True,
        encoding="utf-8",
        timeout=60,
        check=False,
    )
    if done.returncode != 0:
        raise RuntimeError(f"mineworld {command} failed ({done.returncode}): {done.stderr}")
    return [line for line in done.stdout.splitlines() if line.strip()]


def perceived_export(world: str, save: Path, person: str) -> list[PerceivedEvent]:
    """`mineworld perceived … --json`: what `person` perceived, by the server's audience rule."""
    return [
        PerceivedEvent.model_validate_json(line)
        for line in _export("perceived", world, save, person)
    ]


def biography(world: str, save: Path, person: str) -> list[str]:
    """`mineworld biography … --json`: one JSON object per entry, as text."""
    return _export("biography", world, save, person)


# ── The Player's sockets ──────────────────────────────────────────────────────────────────────────


class Tap:
    """One real socket of the Player: every frame sent and received is kept, and the test can abort
    its transport as a network would."""

    def __init__(self, inner: ClientConnection, opened: float) -> None:
        self.inner = inner
        self.opened = opened
        self.sent: list[tuple[float, dict[str, JsonValue]]] = []
        self.received: list[str] = []

    async def send(self, message: str, /) -> None:
        self.sent.append((time.monotonic(), json.loads(message)))
        await self.inner.send(message)

    async def recv(self) -> str | bytes:
        message = await self.inner.recv()
        if isinstance(message, str):
            self.received.append(message)
        return message

    async def close(self) -> None:
        await self.inner.close()

    def abort(self) -> None:
        self.inner.transport.abort()

    def welcomes(self) -> list[dict[str, JsonValue]]:
        found: list[dict[str, JsonValue]] = []
        for text in self.received:
            if '"t":"welcome"' in text.replace(" ", ""):
                frame: JsonValue = json.loads(text)
                if isinstance(frame, dict):
                    found.append(frame)
        return found


@dataclass
class Wire:
    """A connector factory for `Player(connector=…)`: each `join` gets a connector that follows
    `server` (a restart replaces it) and waits while `open` is clear."""

    server: Server
    open: asyncio.Event = field(default_factory=asyncio.Event)
    taps: list[Tap] = field(default_factory=list[Tap])

    def __post_init__(self) -> None:
        self.open.set()

    def __call__(self) -> Callable[[], Awaitable[Tap]]:
        return self.connect

    async def connect(self) -> Tap:
        await self.open.wait()
        inner = await websocket_connect(self.server.url, compression=None, max_size=2**24)
        tap = Tap(inner, time.monotonic())
        self.taps.append(tap)
        return tap

    def frames(self) -> list[dict[str, JsonValue]]:
        return [frame for tap in self.taps for _, frame in tap.sent]


async def seated(server: Server, seat: str, *, take_over: bool = False) -> SeatSession:
    """A plain SDK session on `seat` — a second, ordinary client."""
    connection = await websocket_connect(server.url, compression=None, max_size=2**24)
    try:
        return await SeatSession.join(
            connection,
            seat=EntityKey(seat),
            invite=Invite(INVITE),
            nickname=f"{seat}-sdk",
            take_over=take_over,
        )
    except BaseException:
        await connection.close()
        raise


@dataclass
class Transcript:
    """Every result the test received, for AC-c-5 (≤ 8 KiB of JSON and of text) and AC-c-9 (no
    credential, admin token or resume secret in any of them)."""

    results: list[AgentModel] = field(default_factory=list[AgentModel])

    def __call__[R: AgentModel](self, result: R) -> R:
        self.results.append(result)
        return result

    def largest(self) -> int:
        return max(max(json_bytes(r), len(render_text(r).encode("utf-8"))) for r in self.results)

    def text(self) -> str:
        return "\n".join(r.model_dump_json() + "\n" + render_text(r) for r in self.results)
