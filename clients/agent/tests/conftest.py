"""Fixtures shared by this member's tests.

The network guard is not here: it is pytest-socket, configured for every test in `pyproject.toml`'s
`addopts`, so no test can be collected without it (as sdk/python, I-11).
"""

from __future__ import annotations

from collections.abc import Iterator
from typing import Protocol

import pytest
from realserver import BUILD, Server, binary


class Start(Protocol):
    """Starts `mineworld server worlds/<world> <flags…>`."""

    def __call__(self, world: str, *flags: str) -> Server: ...


@pytest.fixture
def mineworld() -> Iterator[Start]:
    """Starts real servers on demand and kills every one afterwards. A missing binary is an error of
    this fixture, never a skip: a test that did not run is not a pass (the SDK's rule, D-P3-10)."""
    if not binary().is_file():
        pytest.fail(
            f"the mineworld binary is not at {binary()}. Build it first with `{BUILD}` "
            "(or set MINEWORLD_BIN); real_server tests never skip",
            pytrace=False,
        )
    started: list[Server] = []

    def start(world: str, *flags: str) -> Server:
        server = Server(world, *flags)
        started.append(server)
        return server

    yield start
    for server in started:
        server.stop()
