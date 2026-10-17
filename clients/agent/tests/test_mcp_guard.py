"""AC-d-7 (AC-S27-2, R-S27d-c-2): the adapter opens no file but its own Python modules and its `--state-dir`, and
connects to nothing but the relay.

The adapter runs as `guarded_adapter.py`, which installs an audit hook before the adapter's own code, and the test
plays steps 1 to 4 of AC-d-1 (seats, join, observe, a walk to the street and back) through it. The audit log is
then classified line by line. A file read outside those two places, or a connection to any other address, is a
violation and fails the test."""

from __future__ import annotations

import os
import sys
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager
from pathlib import Path

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp_relay import Relay
from realserver import Server
from support import run_default
from test_play_mcp import Transcript, call, child_environment, offer_of, walk

GUARD = Path(__file__).resolve().parent / "guarded_adapter.py"
MODULE_SUFFIXES = (".py", ".pyc", ".pth", ".so", ".pyd", ".dll", ".dylib")
PACKAGE_MARKERS = (
    "site-packages",
    ".dist-info",
    os.sep + "lib" + os.sep + "python",
    os.sep + "Lib" + os.sep,
)
INTERPRETER_CONFIG = ("pyvenv.cfg",)
"""The interpreter's own startup reads: its module paths and its virtual-environment config."""
PLATFORM_FILES = ("SystemVersion.plist", "os-release")
"""The operating system's identity files, read by the standard `platform` module (macOS, Linux). Not user data."""
NULL_DEVICE = os.devnull
"""Standard streams and subprocess plumbing open the null device; nothing is read from it."""
ZIPPED_STDLIB = frozenset(os.path.realpath(entry) for entry in sys.path if entry.endswith(".zip"))
"""The interpreter's zipped standard library on its module path (`python312.zip` on Windows). The adapter runs on
this interpreter (`sys.executable`), so its module path holds the same entries."""
LOOPBACK = ("127.0.0.1", "::1")


@asynccontextmanager
async def guarded(
    relay_url: str, state: Path, grant: str, audit: Path
) -> AsyncGenerator[ClientSession]:
    """One adapter process, run under the audit hook, with `audit` as its log."""
    environment = child_environment(grant)
    environment["MINEWORLD_AUDIT_LOG"] = str(audit)
    parameters = StdioServerParameters(
        command=sys.executable,
        args=[str(GUARD), "mcp", "--server", relay_url, "--state-dir", str(state)],
        env=environment,
    )
    with (audit.parent / "guard-stderr.log").open("w", encoding="utf-8") as errlog:
        async with stdio_client(parameters, errlog=errlog) as (read, write):
            async with ClientSession(read, write) as session:
                await session.initialize()
                yield session


def violations(audit: Path, state: Path, relay: Relay) -> list[str]:
    """Every audit line the adapter may not have written, classified as the guard's rule states."""
    found: list[str] = []
    host, port = "127.0.0.1", str(relay.port)
    for line in audit.read_text(encoding="utf-8").splitlines():
        kind, _, rest = line.partition(" ")
        if kind == "connect":
            if rest != f"{host} {port}":
                found.append(line)
        elif kind == "connect-pair":
            # `socket.socketpair()` on Windows: a loopback connect to the process's own listener (guarded_adapter)
            if rest.partition(" ")[0] not in LOOPBACK:
                found.append(line)
        elif kind == "open":
            if not _allowed_open(rest, state):
                found.append(line)
    return found


def _allowed_open(path: str, state: Path) -> bool:
    resolved = os.path.realpath(path)
    if resolved.startswith(os.path.realpath(state)):
        return True
    if resolved == os.path.realpath(NULL_DEVICE) or resolved in ZIPPED_STDLIB:
        return True
    if os.path.basename(resolved) in INTERPRETER_CONFIG + PLATFORM_FILES:
        return True
    if f"{os.sep}__pycache__{os.sep}" in resolved:
        return True  # the interpreter's bytecode cache of its own modules, written when a module is compiled
    if resolved.endswith(MODULE_SUFFIXES):
        return True
    return any(marker in resolved for marker in PACKAGE_MARKERS)


def test_the_adapter_reads_only_its_modules_and_state_and_connects_only_to_the_relay(
    town: Server, tmp_path: Path
) -> None:
    run_default(_guard(town, tmp_path), timeout_s=180)


async def _guard(server: Server, tmp_path: Path) -> None:
    relay = Relay(server.url, server.http)
    relay_url = await relay.start()
    transcript = Transcript()
    state = tmp_path / "state"
    audit = tmp_path / "audit.log"
    try:
        async with guarded(relay_url, state, server.grants()["visitor"], audit) as session:
            await call(session, transcript, "seats", {})
            joined = await call(session, transcript, "join", {"seat": "visitor", "nickname": "Ada"})
            handle = str(joined["handle"])
            view = (await call(session, transcript, "observe", {"handle": handle}))["view"]
            assert offer_of(view, "walk-to") is not None
            street = view["here"]["exits"][0]["leads_to"]["ref"]
            café = view["me"]["place"]["ref"]
            await walk(session, transcript, handle, {"place": street})
            await walk(session, transcript, handle, {"place": café})
    finally:
        await relay.close()
    found = violations(audit, state, relay)
    assert audit.read_text(encoding="utf-8"), "the audit hook recorded nothing"
    assert f"connect 127.0.0.1 {relay.port}" in audit.read_text(encoding="utf-8").splitlines()
    assert found == [], found
