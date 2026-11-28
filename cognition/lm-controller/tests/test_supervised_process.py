"""A supervised cognition process against the real server, stopped by the end of its standard input while its one model
request is in flight (design `pr-playable-dialogue-memory.md` §§4.10-4.12, C3; `docs/reference/cognition-supervised.md`).

The process is started exactly as the launcher starts it (`python -I -m mineworld_cognition --supervised …`, the invite
in `MINEWORLD_INVITE`, standard input and output pipes held by the test). Its model is a loopback HTTP stub that lists
one model and holds every chat request until the test ends: the "model thinking" a player waits on. Owned here, from
artefacts: `ready` is sent only once Alice's seat is held and its home opened; while her one request is held, the town's
other residents keep acting (their requests in the save's journal); closing standard input stops the process inside the
bound, with `stopped`/`stopping` as its last status line, Alice's seat left (a fresh join is `hosted`), exactly one
request sent and charged once, nothing submitted for her afterwards; the planted invite is in no status line, log or
store file. The launcher's half (argv, environment, kill after the bound, launcher death) is
`tools/launch/tests/launcher/cognition.rs`.
"""

from __future__ import annotations

import asyncio
import json
import os
import sqlite3
import subprocess
import sys
import threading
import time
from collections.abc import Iterator
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import cast

import pytest
from mineworld_sdk.resuming import ResumingSeat
from mineworld_sdk.wire.frames import Invite
from mineworld_sdk.wire.ids import EntityKey
from realserver import DECISIONS, INVITE, REPOSITORY, Server, binary
from support import run
from test_controller_server import (
    _alice_id,  # pyright: ignore[reportPrivateUsage]
    _connector,  # pyright: ignore[reportPrivateUsage]
    _say,  # pyright: ignore[reportPrivateUsage]
    _talk_available,  # pyright: ignore[reportPrivateUsage]
    _visitor,  # pyright: ignore[reportPrivateUsage]
    _wait_for,  # pyright: ignore[reportPrivateUsage]
    _walk_into_reach,  # pyright: ignore[reportPrivateUsage]
)

pytestmark = pytest.mark.real_binary

MODEL = "stub-model"
STOP_BOUND_S = 15.0
"""`shutdown.FORCE_AFTER_S`: the process is gone by then, however its stop goes."""
CONFIG = """\
nickname = "cognition"

[recording]
mode = "live"

[tiers]
social    = "local"
summarize = "none"

[backends.local]
base_url          = "{base_url}"
model             = "{model}"
structured_output = "none"

[controller]
consult_deadline_s = 20
"""


class _HeldModel:
    """A loopback OpenAI-compatible stub: `GET /models` lists one model; every chat request is counted and held."""

    def __init__(self) -> None:
        self.asked = 0
        self.release = threading.Event()
        owner = self

        class Handler(BaseHTTPRequestHandler):
            def do_GET(self) -> None:
                self._send({"object": "list", "data": [{"id": MODEL, "object": "model"}]})

            def do_POST(self) -> None:
                self.rfile.read(int(self.headers.get("Content-Length", "0")))
                owner.asked += 1
                owner.release.wait(120)
                self._send({"choices": []})

            def _send(self, payload: object) -> None:
                body = json.dumps(payload).encode()
                try:
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(body)))
                    self.end_headers()
                    self.wfile.write(body)
                except OSError:
                    pass

            def log_message(self, format: str, *args: object) -> None:
                pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        threading.Thread(target=self.server.serve_forever, daemon=True).start()

    @property
    def base_url(self) -> str:
        return f"http://127.0.0.1:{self.server.server_address[1]}/v1"

    def close(self) -> None:
        self.release.set()
        self.server.shutdown()
        self.server.server_close()


def _statuses(process: subprocess.Popen[str]) -> Iterator[dict[str, object]]:
    assert process.stdout is not None
    for line in process.stdout:
        yield json.loads(line)


def _requests_by(save: Path) -> list[tuple[int, str]]:
    """(journal sequence, actor) of every request in the save, read the way `mineworld inspect` reads it."""
    with sqlite3.connect(f"file:{save / 'world.sqlite'}?mode=ro", uri=True) as connection:
        rows = connection.execute("SELECT rowid, entry FROM journal ORDER BY rowid").fetchall()
    found: list[tuple[int, str]] = []
    for rowid, entry in cast(list[tuple[int, bytes]], rows):
        loaded = cast(dict[str, dict[str, dict[str, dict[str, object]]]], json.loads(entry))
        actor = loaded.get("input", {}).get("dispatch", {}).get("intent", {}).get("actor")
        if actor is not None:
            found.append((rowid, str(actor)))
    return found


def test_closing_stdin_stops_a_supervised_process_mid_request_and_the_town_keeps_acting(
    tmp_path: Path,
) -> None:
    model = _HeldModel()
    server = Server(tmp_path / "save")
    store = tmp_path / "cognition" / "social-cafe"
    assignment = store / "run" / "assignment.json"
    assignment.parent.mkdir(parents=True)
    with assignment.open("w", encoding="utf-8") as out:
        subprocess.run(
            [
                str(binary()),
                "decisions",
                str(REPOSITORY / "worlds" / "social-cafe"),
                "--decisions",
                str(DECISIONS),
                "--json",
            ],
            stdout=out,
            check=True,
            timeout=60,
        )
    config = tmp_path / "cognition.toml"
    config.write_text(CONFIG.format(base_url=model.base_url, model=MODEL), encoding="utf-8")
    environment = {**os.environ, "MINEWORLD_INVITE": INVITE}
    process = subprocess.Popen(
        [
            sys.executable,
            "-I",
            "-m",
            "mineworld_cognition",
            "--config",
            str(config),
            "--supervised",
            f"--server-url={server.url}",
            "--store-dir",
            str(store),
            "--assignment",
            str(assignment),
        ],
        env=environment,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    lines = _statuses(process)
    try:
        assert next(lines) == {"status": 1, "state": "starting", "seat": None, "reason": None}
        assert next(lines) == {"status": 1, "state": "starting", "seat": "alice", "reason": None}
        assert next(lines) == {"status": 1, "state": "ready", "seat": "alice", "reason": None}
        instance = (store / "instance").read_text(encoding="utf-8").strip()
        assert (store / instance / "memory.sqlite").is_file(), "ready means the home is open"

        async def ask_and_wait() -> tuple[int, int]:
            visitor = await _visitor(server.url)
            async with visitor:
                await _walk_into_reach(visitor)
                alice = _alice_id((await visitor.changed()).observation)
                await _talk_available(visitor, alice)
                await _say(visitor, alice, "Good morning, is the oven on?")
                await _wait_for(lambda: model.asked == 1, "Alice's request reached the model")
                held_at = len(_requests_by(server.save))
                await asyncio.sleep(6)  # the model is still thinking: the town is not
                return held_at, model.asked

        held_at, asked = run(ask_and_wait(), timeout_s=120)
        during = _requests_by(server.save)[held_at:]
        others = {actor for _, actor in during}
        assert others, f"other residents acted while Alice's request was held: {during}"

        stopping = time.monotonic()
        assert process.stdin is not None
        process.stdin.close()
        rest = list(lines)
        code = process.wait(timeout=STOP_BOUND_S + 5)
        took = time.monotonic() - stopping
        assert rest == [{"status": 1, "state": "stopped", "seat": "alice", "reason": "stopping"}], (
            rest
        )
        assert code == 0 and took <= STOP_BOUND_S, (code, took)
        assert (asked, model.asked) == (1, 1), "one request was sent, and nothing after the stop"

        async def join_after() -> str:
            fresh = await ResumingSeat.open(
                _connector(server.url),
                seat=EntityKey("alice"),
                invite=Invite(INVITE),
                nickname="after-stop",
                cursor=None,
                take_over=False,
                sleep=asyncio.sleep,
                clock=time.monotonic,
            )
            took_over = fresh.welcome.took_over
            await fresh.leave()
            return took_over

        assert run(join_after(), timeout_s=60) == "hosted", "the stopped process left Alice's seat"
        with sqlite3.connect(store / "budget.sqlite") as ledger:
            charged = ledger.execute(
                "SELECT COALESCE(SUM(calls), 0) FROM budget_ledger"
            ).fetchone()[0]
        assert charged == 1, (
            "the cancelled in-flight call is charged once (ARC-57 note of 2026-11-27)"
        )

        stderr = process.stderr.read() if process.stderr is not None else ""
        assert INVITE not in stderr
        for path in store.rglob("*"):
            if path.is_file():
                assert INVITE.encode() not in path.read_bytes(), path
    finally:
        if process.poll() is None:
            process.kill()
            process.wait()
        model.close()
        server.stop()
