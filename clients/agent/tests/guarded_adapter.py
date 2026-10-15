"""The adapter under an audit hook (AC-d-7, the guard of AC-S27-2 and R-S27d-c-2).

Run as `python guarded_adapter.py <mineworld-agent arguments>`. Before anything else it opens the audit log named by
`MINEWORLD_AUDIT_LOG` and installs `sys.addaudithook`, which writes one line per `open` and per `socket.connect` to
that log through a file descriptor opened beforehand (so the hook itself makes no `open` events). Then it runs
`mineworld_agent.__main__.main` with the arguments it was given.

The log has two line kinds: `open <path>` and `connect <host> <port>`. The test classifies them (test_mcp_guard.py)."""

import os
import sys
from typing import cast

_LOG = os.open(os.environ["MINEWORLD_AUDIT_LOG"], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)


def _record(event: str, args: tuple[object, ...]) -> None:
    if event == "open" and args and isinstance(args[0], (str, bytes, os.PathLike)):
        path = os.fsdecode(os.fspath(args[0]))  # type: ignore[arg-type]
        os.write(_LOG, f"open {path}\n".encode())
    elif event == "socket.connect" and len(args) >= 2 and isinstance(args[1], tuple):
        address = cast(tuple[object, ...], args[1])
        if len(address) >= 2:
            os.write(_LOG, f"connect {address[0]} {address[1]}\n".encode())


sys.addaudithook(_record)

from mineworld_agent.__main__ import main  # noqa: E402

raise SystemExit(main(sys.argv[1:]))
