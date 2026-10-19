"""The MCP adapter as a Streamable HTTP server, started for a test (pr-s27e §8, C5).

`HttpAdapter` starts `python -m mineworld_agent mcp --http 127.0.0.1:0 --server <relay> --state-dir <dir>` with no
credential in its environment, its stdout and stderr in files under the test's log directory (AC-e-6 scans them),
and reads the real port from the startup line on stderr. `session(bearer)` opens one MCP session to it, through the
SDK's own `streamable_http_client`, sending `Authorization: Bearer <bearer>` on every request. `stop()` asks the
adapter to shut down as an operator would: SIGTERM on POSIX, `CTRL_BREAK_EVENT` to its own process group on Windows.
"""

from __future__ import annotations

import asyncio
import os
import re
import signal
import subprocess
import sys
import time
from collections.abc import AsyncGenerator
from contextlib import asynccontextmanager
from pathlib import Path

import httpx2
from mcp import ClientSession
from mcp.client.streamable_http import streamable_http_client

STARTUP_PATIENCE = 30.0
_LISTENING = re.compile(
    r"listening on http://(?P<host>[^/\s]+):(?P<port>\d+)/mcp \((?P<mode>[^)]*)\)"
)


def child_environment() -> dict[str, str]:
    """This process's environment without any MINEWORLD_ variable: the HTTP adapter holds no credential."""
    return {key: value for key, value in os.environ.items() if not key.startswith("MINEWORLD_")}


class HttpAdapter:
    """One adapter process serving Streamable HTTP on loopback."""

    def __init__(self, relay_url: str, state: Path, logs: Path, *flags: str) -> None:
        logs.mkdir(parents=True, exist_ok=True)
        self.stdout_path = logs / "http-stdout.log"
        self.stderr_path = logs / "http-stderr.log"
        self._stdout = self.stdout_path.open("w", encoding="utf-8")
        self._stderr = self.stderr_path.open("w", encoding="utf-8")
        group = 0
        if sys.platform == "win32":
            group = subprocess.CREATE_NEW_PROCESS_GROUP  # so CTRL_BREAK_EVENT reaches it alone
        self.process = subprocess.Popen(
            [
                sys.executable,
                "-m",
                "mineworld_agent",
                "mcp",
                "--http",
                "127.0.0.1:0",
                "--server",
                relay_url,
                "--state-dir",
                str(state),
                *flags,
            ],
            env=child_environment(),
            stdin=subprocess.DEVNULL,
            stdout=self._stdout,
            stderr=self._stderr,
            creationflags=group,
        )
        self.port = 0
        self.mode = ""

    async def started(self) -> HttpAdapter:
        """Waits for the startup line and reads the port from it."""
        deadline = time.monotonic() + STARTUP_PATIENCE
        while time.monotonic() < deadline:
            match = _LISTENING.search(self.stderr_path.read_text(encoding="utf-8"))
            if match is not None:
                self.port = int(match.group("port"))
                self.mode = match.group("mode")
                return self
            if self.process.poll() is not None:
                break
            await asyncio.sleep(0.05)
        self.kill()
        raise RuntimeError(f"the HTTP adapter did not start:\n{self.stderr()}")

    @property
    def url(self) -> str:
        return f"http://127.0.0.1:{self.port}/mcp"

    @asynccontextmanager
    async def session(self, bearer: str) -> AsyncGenerator[ClientSession]:
        """One MCP session as an agent with `bearer` (the handshake era, as the stdio play-test's client)."""
        # The SDK's own client timeouts (30 s, 300 s to read): httpx2's default 5 s would cut a `wait` short.
        timeout = httpx2.Timeout(30.0, read=300.0)
        headers = {"Authorization": f"Bearer {bearer}"}
        async with httpx2.AsyncClient(headers=headers, timeout=timeout) as http:
            async with streamable_http_client(self.url, http_client=http) as (read, write):
                async with ClientSession(read, write) as session:
                    await session.initialize()
                    yield session

    def stderr(self) -> str:
        return self.stderr_path.read_text(encoding="utf-8")

    def stdout(self) -> str:
        return self.stdout_path.read_text(encoding="utf-8")

    def stop(self, patience: float = 15.0) -> int | None:
        """Asks the adapter to shut down (SIGTERM, or CTRL_BREAK_EVENT on Windows) and waits; kills it if it does
        not end within `patience`. The exit code, or `None` when it had to be killed."""
        if self.process.poll() is None:
            if sys.platform == "win32":
                self.process.send_signal(signal.CTRL_BREAK_EVENT)
            else:
                self.process.send_signal(signal.SIGTERM)
            try:
                return self.process.wait(timeout=patience)
            except subprocess.TimeoutExpired:
                self.kill()
                return None
        return self.process.returncode

    def kill(self) -> None:
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait()
        self._stdout.close()
        self._stderr.close()


@asynccontextmanager
async def http_adapter(
    relay_url: str, state: Path, logs: Path, *flags: str
) -> AsyncGenerator[HttpAdapter]:
    adapter = HttpAdapter(relay_url, state, logs, *flags)
    try:
        yield await adapter.started()
    finally:
        adapter.stop()
        adapter.kill()
