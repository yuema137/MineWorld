"""The real `mineworld` binary, started as a player's machine would start it, for the `real_server` tests.

Nothing here is mocked. The binary is the one cargo built; the World Pack is the repository's own; the
socket is a real WebSocket on loopback. It follows `tools/cli/tests/support/mod.rs`: an ephemeral port
(`--listen 127.0.0.1:0`, whose real port the server prints), the invite given by environment so it is
never printed, and the process killed when the test ends.

Every platform (D-P3-11): the binary is found with the platform's executable suffix, started from an
argument list (never a shell), read through threads (a pipe cannot be polled on Windows), and stopped
with `Popen.kill`, which is `TerminateProcess` on Windows and `SIGKILL` elsewhere.
"""

from __future__ import annotations

import os
import queue
import re
import subprocess
import sys
import threading
import time
from pathlib import Path

from websockets.asyncio.client import ClientConnection
from websockets.asyncio.client import connect as websocket_connect

from mineworld_sdk.session import SeatSession
from mineworld_sdk.wire.contract import PerceivedEvent
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityKey, JsonValue

REPOSITORY = Path(__file__).resolve().parents[3]
BUILD = "cargo build -p mineworld-cli"
INVITE = "mwsdk-marker-invite-7f3a9c2e5b1d"
"""The invite every test server is started with: 32 characters, so a scan can find it anywhere."""
STARTUP_PATIENCE = 30.0
# "[mineworld] listening on http://ADDR (ws://ADDR/ws), protocol 2" — the bound address (app.rs `bind`).
_LISTENING = re.compile(r"listening on \S+ \((?P<url>ws://[^)\s]+)\)")


def binary() -> Path:
    """`MINEWORLD_BIN`, or the debug build under the repository's `target/`."""
    named = os.environ.get("MINEWORLD_BIN")
    if named:
        return Path(named)
    suffix = ".exe" if sys.platform == "win32" else ""
    return REPOSITORY / "target" / "debug" / f"mineworld{suffix}"


class Server:
    """One running `mineworld server <world>`, with `--save DIR` and `--hold SECONDS` when given. A
    server started again on the same save resumes that world (`PROTOCOL.md` §5.7: the same instance)."""

    def __init__(self, world: str, *, save: Path | None = None, hold: int | None = None) -> None:
        executable = binary()
        if not executable.is_file():
            raise FileNotFoundError(
                f"the mineworld binary is not at {executable}. Build it first with `{BUILD}` "
                "(or set MINEWORLD_BIN); these tests start the real server and never skip"
            )
        self.world, self.save, self.hold = world, save, hold
        environment = dict(os.environ)
        environment["MINEWORLD_INVITE"] = INVITE
        options = [] if save is None else ["--save", str(save)]
        options += [] if hold is None else ["--hold", str(hold)]
        self._process = subprocess.Popen(
            [
                str(executable),
                "server",
                str(REPOSITORY / "worlds" / world),
                "--listen",
                "127.0.0.1:0",
                *options,
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
        self._stderr: list[str] = []
        threading.Thread(target=self._drain_stdout, daemon=True).start()
        threading.Thread(target=self._drain_stderr, daemon=True).start()
        self.url = self._wait_for_url()

    def _drain_stdout(self) -> None:
        assert self._process.stdout is not None
        for line in self._process.stdout:
            self._stdout.put(line)
        self._stdout.put(None)

    def _drain_stderr(self) -> None:
        assert self._process.stderr is not None
        self._stderr.extend(self._process.stderr)

    def _wait_for_url(self) -> str:
        deadline = time.monotonic() + STARTUP_PATIENCE
        while (left := deadline - time.monotonic()) > 0:
            try:
                line = self._stdout.get(timeout=left)
            except queue.Empty:
                break
            if line is None:
                break
            if match := _LISTENING.search(line):
                return match.group("url")
        self.stop()
        raise RuntimeError(
            f"the server did not report its address within {STARTUP_PATIENCE:.0f} s; "
            f"stderr:\n{''.join(self._stderr)}"
        )

    def stop(self) -> None:
        """Kills the process (`TerminateProcess` / `SIGKILL`: no graceful shutdown) and waits for it to
        exit, so its save is closed before anything reads or reopens it (R-P3b-4)."""
        self._process.kill()
        self._process.wait()


def perceived_export(world: str, save: Path, person: str) -> list[PerceivedEvent]:
    """`mineworld perceived <world> --save <save> --person <person> --json`: the offline export of what
    `person` perceived, one `PerceivedEvent` per line — the oracle the live stream must equal."""
    done = subprocess.run(
        [
            str(binary()),
            "perceived",
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
        raise RuntimeError(f"mineworld perceived failed ({done.returncode}): {done.stderr}")
    return [
        PerceivedEvent.model_validate_json(line)
        for line in done.stdout.splitlines()
        if line.strip()
    ]


class Recording:
    """A connection that keeps every text frame it received, raw, for AP-2's round trip."""

    def __init__(self, inner: ClientConnection) -> None:
        self._inner = inner
        self.received: list[str] = []

    async def send(self, message: str, /) -> None:
        await self._inner.send(message)

    async def recv(self) -> str | bytes:
        message = await self._inner.recv()
        if isinstance(message, str):
            self.received.append(message)
        return message

    async def close(self) -> None:
        await self._inner.close()

    def count(self, kind: str) -> int:
        marker = f'"t":"{kind}"'
        return sum(1 for frame in self.received if marker in frame.replace(" ", ""))

    def stream(self) -> int:
        """Frames of the observation stream, whole or `delta` (`PROTOCOL.md` §5.3): a session turns
        each into a whole observation."""
        return self.count("observation") + self.count("delta")


async def seated(
    server: Server, seat: str, *, invite: str = INVITE
) -> tuple[SeatSession, Recording]:
    """A session on `seat` whose received frames are recorded."""
    connection = await websocket_connect(server.url, compression=None, max_size=2**24)
    recording = Recording(connection)
    try:
        session = await SeatSession.join(
            recording, seat=EntityKey(seat), invite=Invite(invite), nickname=f"{seat}-sdk"
        )
    except BaseException:
        await connection.close()
        raise
    return session, recording


def first_difference(expected: JsonValue, actual: JsonValue, path: str = "$") -> str | None:
    """The JSON path of the first place two values differ, or `None` when they are equal."""
    if isinstance(expected, dict) and isinstance(actual, dict):
        for key in sorted(expected.keys() | actual.keys()):
            if key not in expected or key not in actual:
                return f"{path}.{key}"
            found = first_difference(expected[key], actual[key], f"{path}.{key}")
            if found:
                return found
        return None
    if isinstance(expected, list) and isinstance(actual, list):
        if len(expected) != len(actual):
            return f"{path} (length {len(expected)} != {len(actual)})"
        for index, (left, right) in enumerate(zip(expected, actual, strict=True)):
            found = first_difference(left, right, f"{path}[{index}]")
            if found:
                return found
        return None
    if type(expected) is not type(actual) or expected != actual:
        return path
    return None
