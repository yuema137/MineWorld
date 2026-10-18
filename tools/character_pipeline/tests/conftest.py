"""Test setup: the pipeline package on `sys.path`, and a guard that fails any network connection."""

from __future__ import annotations

import socket
import sys
from collections.abc import Iterator
from pathlib import Path

import pytest

PIPELINE_ROOT = Path(__file__).resolve().parents[1]
REPO_ROOT = Path(__file__).resolve().parents[3]

if str(PIPELINE_ROOT) not in sys.path:
    sys.path.insert(0, str(PIPELINE_ROOT))


@pytest.fixture(autouse=True)
def no_network(monkeypatch: pytest.MonkeyPatch) -> Iterator[None]:
    """A unit test that opens a socket fails: core tests never reach a live service (standards §10)."""

    def refuse(*_args: object, **_kwargs: object) -> None:
        raise RuntimeError("network access in a unit test")

    monkeypatch.setattr(socket.socket, "connect", refuse)
    yield
