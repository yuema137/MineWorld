"""The guide's agent session replays against the real server (pr-s27f §5.2 "Replay", I-f-3's replay half, AC-f-3).

The fixture `docs/guide/examples/agent-session.json` was recorded once by `agent_session.py --record` and is shown
verbatim by chapter 14. This test plays its calls again, through the official MCP client and the adapter over stdio,
against a fresh real server on the guide's Corner Shop, and checks every step: the same success or error code, every
shown path present with the recorded JSON type, and every value that is not `"…"` equal. The oracle is the live
server's answer, never the recorder's output compared with itself."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from agent_session import FIXTURE, replay

pytestmark = pytest.mark.real_server


def test_the_session_replays(mineworld: object, tmp_path: Path) -> None:
    """`mineworld` fails the test when the binary is missing (never a skip); the session starts its own server."""
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    problems = replay(fixture, tmp_path)
    assert problems == [], "\n".join(problems)
