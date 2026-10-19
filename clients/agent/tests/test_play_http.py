"""The play-test over Streamable HTTP against the real server (pr-s27e §8 AC-e-1, CP-e).

The same script as `test_play_mcp.py` (`play`), with one adapter process serving Streamable HTTP for every agent of
the test: each session's credential is its `Authorization: Bearer`, and the adapter's environment holds none. Step
9's take-back is made by a second HTTP agent, in the same adapter process, whose bearer is the `visitor:take-over`
grant. The relay's recorded frames pass S27-d's key-path check (AC-d-2), every answer fits 8 KiB, no secret reaches
an agent or the adapter's stdout or stderr, and the facts shown are one unbroken run of the save's perceived facts.

The server is the real `mineworld` binary (`MINEWORLD_BIN`); a missing binary fails the test, it never skips. The
file runs on the platform's default loop: the adapter is a subprocess (F-S27d-6).
"""

from __future__ import annotations

from collections.abc import AsyncGenerator
from contextlib import AbstractAsyncContextManager, asynccontextmanager
from pathlib import Path

import pytest
from http_adapter import http_adapter
from mcp import ClientSession
from realserver import Server
from support import run_default
from test_play_mcp import Open, play

pytestmark = pytest.mark.real_server


@asynccontextmanager
async def http(relay_url: str, state: Path, logs: Path) -> AsyncGenerator[Open]:
    """The HTTP transport: one adapter process; each session is an agent with its own bearer."""
    async with http_adapter(relay_url, state, logs) as adapter:
        assert adapter.mode == "loopback only", adapter.mode

        def open_session(grant: str, _name: str) -> AbstractAsyncContextManager[ClientSession]:
            return adapter.session(grant)

        yield open_session


def test_the_play_test_over_streamable_http(town: Server, tmp_path: Path) -> None:
    """AC-e-1: S27-d's AC-d-1 … AC-d-5 steps over HTTP, the take-back in the same adapter process."""
    run_default(play(town, tmp_path, http), timeout_s=600)
