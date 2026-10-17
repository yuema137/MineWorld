"""The adapter under an audit hook (AC-d-7, the guard of AC-S27-2 and R-S27d-c-2).

Run as `python guarded_adapter.py <mineworld-agent arguments>`. Before anything else it opens the audit log named by
`MINEWORLD_AUDIT_LOG` and installs `sys.addaudithook`, which writes one line per `open` and per `socket.connect` to
that log through a file descriptor opened beforehand (so the hook itself makes no `open` events). Then it runs
`mineworld_agent.__main__.main` with the arguments it was given.

The log has three line kinds: `open <path>`, `connect <host> <port>`, and `connect-pair <host> <port>`. The last is a
connect made inside `socket.socketpair()`: on Windows the standard library builds a socket pair by listening on
loopback and connecting to itself, and asyncio's event loop makes one for its self-pipe; it reaches nothing outside
the process. `socket.socketpair` is wrapped before any other import so those connects are told apart from every other
one. The test classifies the lines (test_mcp_guard.py)."""

import os
import socket
import sys
from typing import Any, cast

_LOG = os.open(os.environ["MINEWORLD_AUDIT_LOG"], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)
_pairing = [False]
"""True while `socket.socketpair` runs (its connects are the process's own, to itself)."""
_socketpair = socket.socketpair


def _paired(*args: Any, **kwargs: Any) -> tuple[socket.socket, socket.socket]:  # noqa: ANN401
    _pairing[0] = True
    try:
        return _socketpair(*args, **kwargs)
    finally:
        _pairing[0] = False


socket.socketpair = _paired


def _record(event: str, args: tuple[object, ...]) -> None:
    if event == "open" and args and isinstance(args[0], (str, bytes, os.PathLike)):
        path = os.fsdecode(os.fspath(args[0]))  # type: ignore[arg-type]
        os.write(_LOG, f"open {path}\n".encode())
    elif event == "socket.connect" and len(args) >= 2 and isinstance(args[1], tuple):
        address = cast(tuple[object, ...], args[1])
        if len(address) >= 2:
            kind = "connect-pair" if _pairing[0] else "connect"
            os.write(_LOG, f"{kind} {address[0]} {address[1]}\n".encode())


sys.addaudithook(_record)

from mineworld_agent.__main__ import main  # noqa: E402

raise SystemExit(main(sys.argv[1:]))
