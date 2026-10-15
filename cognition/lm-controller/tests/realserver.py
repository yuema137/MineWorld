"""The real `mineworld` binary, started as a player's machine would start it, for the `real_binary` tests of this member.

Copied from `sdk/python/tests/realserver.py` (S10 P6b, C12, pr-s10-p6 §9 C12: a test module of one member cannot import
another member's tests). The copy keeps the server control and drops the SDK's frame recorder and perceived export. The
flags of the LM controller's server are the caller's (`--town --decisions … --hold … --pace …`), and every test server
listens on loopback at an ephemeral port. The invite is given by environment so it is never printed. The binary is the
one cargo built (`MINEWORLD_BIN` names another); it is never built here and a missing binary fails the test.
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

REPOSITORY = Path(__file__).resolve().parents[3]
BUILD = "cargo build -p mineworld-cli"
INVITE = "mwsdk-marker-invite-7f3a9c2e5b1d"
"""The invite every test server is started with (32 characters, so a scan can find it anywhere)."""
STARTUP_PATIENCE = 30.0
# "[mineworld] listening on http://ADDR (ws://ADDR/ws), protocol 2" (the bound address, app.rs `bind`).
_LISTENING = re.compile(r"listening on \S+ \((?P<url>ws://[^)\s]+)\)")
DECISIONS = REPOSITORY / "cognition" / "lm-controller" / "examples" / "alice-lm.decisions.yaml"


def binary() -> Path:
    """`MINEWORLD_BIN`, or the debug build under the repository's `target/`."""
    named = os.environ.get("MINEWORLD_BIN")
    if named:
        return Path(named)
    suffix = ".exe" if sys.platform == "win32" else ""
    return REPOSITORY / "target" / "debug" / f"mineworld{suffix}"


def require_binary() -> Path:
    executable = binary()
    if not executable.is_file():
        raise FileNotFoundError(
            f"the mineworld binary is not at {executable}. Build it first with `{BUILD}` "
            "(or set MINEWORLD_BIN); these tests start the real server and never skip"
        )
    return executable


class Server:
    """One running `mineworld server social-cafe --town --decisions … --save DIR --hold N --pace N`.

    A server started again on the same save resumes that world (`PROTOCOL.md` §5.7: the same instance)."""

    def __init__(self, save: Path, *, hold: int = 3, pace: int = 1) -> None:
        executable = require_binary()
        self.save = save
        environment = dict(os.environ)
        environment["MINEWORLD_INVITE"] = INVITE
        self._process = subprocess.Popen(
            [
                str(executable),
                "server",
                str(REPOSITORY / "worlds" / "social-cafe"),
                "--town",
                "--decisions",
                str(DECISIONS),
                "--save",
                str(save),
                "--hold",
                str(hold),
                "--pace",
                str(pace),
                "--listen",
                "127.0.0.1:0",
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

    def stderr(self) -> str:
        """What the server has written to its standard error so far (for a failure message)."""
        return "".join(self._stderr)

    def stop(self) -> None:
        """Kills the process (`TerminateProcess` / `SIGKILL`, no graceful shutdown) and waits for it to exit, so
        its save is closed before anything reads it."""
        self._process.kill()
        self._process.wait()
