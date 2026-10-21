"""The API and the command share one implementation (pr-s28e §5.4; step-28 §5.6; I-e-6).

`inspect.report` over an open home (a running process's own, here the writable home that built it) returns the
report the command prints for the stopped file, byte for byte; a printed report parses back as `InspectReport`
to the same value.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

import pytest
from home_streams import SEATS
from inspect_fixtures import build
from memory_fixtures import INSTANCE
from mineworld_sdk.wire.ids import EventId
from pydantic import TypeAdapter

from mineworld_cognition.inspect import (
    AccessRequest,
    BytesRequest,
    ForgottenRequest,
    InspectReport,
    InspectRequest,
    report,
)
from mineworld_cognition.memory.home import MemoryHome


@pytest.fixture(scope="module")
def synthetic(tmp_path_factory: pytest.TempPathFactory) -> Path:
    path = tmp_path_factory.mktemp("synthetic") / "memory.sqlite"
    build(path)
    return path


CASES: list[tuple[InspectRequest, list[str]]] = [
    (AccessRequest(seat=SEATS[3]), ["--seat", SEATS[3], "access"]),
    (
        AccessRequest(seat=SEATS[3], event=EventId("1700")),
        ["--seat", SEATS[3], "access", "--event", "1700"],
    ),
    (ForgottenRequest(seat=SEATS[4]), ["--seat", SEATS[4], "forgotten"]),
    (BytesRequest(seat=SEATS[5]), ["--seat", SEATS[5], "bytes"]),
    (BytesRequest(), ["bytes"]),
]


def test_the_api_on_an_open_home_is_the_command_on_the_stopped_file(synthetic: Path) -> None:
    adapter: TypeAdapter[InspectReport] = TypeAdapter(InspectReport)
    printed: list[bytes] = []
    for _, arguments in CASES:
        done = subprocess.run(
            [
                sys.executable,
                "-m",
                "mineworld_cognition.inspect.cli",
                "inspect",
                str(synthetic),
                *arguments,
                "--json",
            ],
            capture_output=True,
            timeout=300,
            check=False,
        )
        assert done.returncode == 0, done.stderr
        printed.append(done.stdout)
    with MemoryHome.open(synthetic, INSTANCE) as home:
        for (request, _), out in zip(CASES, printed, strict=True):
            found = report(home, request)
            # The file's size (page count) is the only figure a writable open may move; none here does.
            assert (found.model_dump_json() + "\n").encode("utf-8") == out, request
            assert adapter.validate_json(out) == found
