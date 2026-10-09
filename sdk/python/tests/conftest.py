"""Fixtures shared by the SDK's tests.

The network guard is not here: it is pytest-socket, configured for every test in `pyproject.toml`'s
`addopts`, so no test can be collected without it (I-11, D-P3-11 (b)).
"""

from __future__ import annotations

from collections.abc import Callable, Iterator

import pytest
from realserver import BUILD, Server, binary


@pytest.fixture
def mineworld() -> Iterator[Callable[[str], Server]]:
    """Starts `mineworld server <world>` on demand, and kills every server it started afterwards.

    A missing binary is an **error** of this fixture, never a skip (D-P3-10, AP-11): a test that did
    not run is not a pass. A job that deliberately builds no binary deselects these tests by their
    `real_server` marker on its command line instead.
    """
    if not binary().is_file():
        pytest.fail(
            f"the mineworld binary is not at {binary()}. Build it first with `{BUILD}` "
            "(or set MINEWORLD_BIN); real_server tests never skip",
            pytrace=False,
        )
    started: list[Server] = []

    def start(world: str) -> Server:
        server = Server(world)
        started.append(server)
        return server

    yield start
    for server in started:
        server.stop()
