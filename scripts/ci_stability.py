#!/usr/bin/env python3
"""The nightly's stability programs (`docs/DECISIONS.md` ARC-83; `pr-13c-nightly.md` §0, N-C3).

`ENGINEERING_STANDARDS.md` §16, layer 4: "restart server repeatedly, replay event log". Two programs, each
judged by MineWorld's own answers, never by this script's bookkeeping:

    uv run --locked python scripts/ci_stability.py restarts --binary target/release/mineworld
        `mineworld server worlds/market-town --town --save D` is started, a client of the Python SDK joins
        it with the invite the server printed and asks for the `perceived` stream from the cursor it had
        reached, the server is killed (Popen.kill: SIGKILL, TerminateProcess) while the client is seated,
        and started again on the same save — ten times. A restart drops every hold and resume secret
        (`server/PROTOCOL.md` §4.2), so the client joins again with its invite and its cursor (§5.8).
        Each cycle's oracle is the server's `GET /status`: the same `instance` (AC-6), a `revision` not
        behind the last one seen before the kill, `faults` 0. Then `mineworld replay` of the killed save.
    python3 scripts/ci_stability.py replay --binary target/release/mineworld
        every world under worlds/ saved 300 days (seed 7), then `mineworld replay` of the save; for
        market-town, CA-13's ignored test (`perceived.rs`) hosts that save before it is removed. Saves are
        made and removed one at a time.
    python3 scripts/ci_stability.py replay-check --binary B --world W --save DIR
        `mineworld replay` of one existing save, judged (MN-3's tampered copy).
    python3 scripts/ci_stability.py --self-test
        the oracles' verdicts against recorded answers.

Wall times and save sizes go to `artifacts/nightly/timings-stability-<program>.json` for the report's
benchmark. Exits non-zero naming the first expectation that did not hold. The SDK is imported only by
`restarts`, so the other subcommands and the self-test need nothing but the standard library.
"""

from __future__ import annotations

import asyncio
import json
import os
import queue
import re
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import urllib.request
from dataclasses import dataclass
from pathlib import Path

from ci_parity import PACK_ROOTS, enumerate_worlds

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "artifacts" / "nightly"
SEED = 7
REPLAY_DAYS = 300
CYCLES = 10
RESTART_WORLD = "market-town"
# The server is told to run faster than wall time and to drive every seat, so the world has moved between
# one kill and the next.
SERVER_FLAGS = ["--listen", "127.0.0.1:0", "--town", "--time-scale", "60"]
START_LIMIT = 120.0
SEATED_LIMIT = 60.0
SEEN_DEATH_LIMIT = 20.0
RUNNING_BEFORE_KILL = 3.0
# The server's join line for a generated invite (tools/cli/src/invite.rs `join_line`).
JOIN = re.compile(r"^\[mineworld\] invite (?P<invite>\S+) — join with: (?P<address>\S+) seat=(?P<seat>\S+) invite=")
REPLAYED = re.compile(r"revision\(s\) re-executed from genesis, (?P<facts>\d+) fact\(s\) and (?P<snapshots>\d+) "
                      r"snapshot\(s\) reproduced byte for byte; head revision (?P<head>\d+)")
CA13 = "a_resume_of_a_long_save_does_not_stall_the_world"


class Unmet(Exception):
    """An expectation did not hold; the message names it."""


# --------------------------------------------------------------------------------------------- oracles


@dataclass(frozen=True)
class Status:
    instance: str
    revision: int | None
    faults: int


def parse_status(body: bytes) -> Status:
    """`GET /status` (`server/PROTOCOL.md` §5.7), only the three fields the oracle reads."""
    try:
        answer = json.loads(body)
        revision = answer["revision"]
        return Status(str(answer["instance"]), None if revision is None else int(revision), int(answer["faults"]))
    except (ValueError, KeyError, TypeError) as error:
        raise Unmet(f"/status answered something that is not a WorldSummary ({error}): {body[:200]!r}") from error


def judge_restart(cycle: int, first: Status, seen: int, after: Status) -> list[str]:
    """One restart, judged by the server's own answer: the same world, not behind, no fault."""
    failures = []
    if after.instance != first.instance:
        failures.append(f"cycle {cycle}: instance {first.instance} became {after.instance} after a restart (AC-6)")
    if after.revision is None:
        failures.append(f"cycle {cycle}: revision is null, so the world is not persisted")
    elif after.revision < seen:
        failures.append(f"cycle {cycle}: revision {after.revision} after the restart is behind {seen}, "
                        "the last revision seen before the kill")
    if after.faults != 0:
        failures.append(f"cycle {cycle}: faults {after.faults}")
    return failures


def parse_join(line: str) -> tuple[str, str, str] | None:
    """(address, invite, seat) from the server's join line, or None for any other line."""
    match = JOIN.match(line)
    return (match["address"], match["invite"], match["seat"]) if match else None


def judge_replay(world: str, status: int, output: str) -> str | None:
    """None if `mineworld replay` reproduced the save, else the reason, naming the world."""
    if status != 0:
        return f"replay {world}: exited {status}: {output.strip()[-600:]}"
    if not REPLAYED.search(output):
        return f"replay {world}: exited 0 but printed no 'reproduced byte for byte' line: {output.strip()[-600:]}"
    return None


# ---------------------------------------------------------------------------------------------- server


def binary_path(given: str) -> str:
    path = Path(given)
    if not path.is_absolute():
        path = ROOT / path
    if not path.is_file() and os.name == "nt" and path.with_suffix(".exe").is_file():
        path = path.with_suffix(".exe")
    if not path.is_file():
        raise Unmet(f"--binary {given}: no such file")
    return str(path)


class Server:
    """`mineworld server` on a save, read line by line so that its join line is seen as it is printed."""

    def __init__(self, binary: str, world: str, save: Path) -> None:
        arguments = [binary, "server", f"worlds/{world}", *PACK_ROOTS, "--save", str(save), *SERVER_FLAGS]
        self.lines: queue.Queue[str] = queue.Queue()
        self.output: list[str] = []
        self.process = subprocess.Popen(arguments, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        threading.Thread(target=self._read, daemon=True).start()
        self.address, self.invite, self.seat = self._joined()

    def _read(self) -> None:
        assert self.process.stdout is not None
        for raw in self.process.stdout:
            self.lines.put(raw.decode("utf-8", errors="replace").rstrip("\r\n"))

    def _joined(self) -> tuple[str, str, str]:
        deadline = time.monotonic() + START_LIMIT
        while time.monotonic() < deadline:
            try:
                line = self.lines.get(timeout=0.5)
            except queue.Empty:
                if self.process.poll() is not None:
                    break
                continue
            self.output.append(line if not line.startswith("[mineworld] invite ") else "[mineworld] invite <kept out>")
            if found := parse_join(line):
                return found
        self.kill()
        raise Unmet("the server printed no join line within "
                    f"{START_LIMIT:.0f} s (exit {self.process.poll()}):\n" + "\n".join(self.output[-40:]))

    def status(self) -> Status:
        try:
            with urllib.request.urlopen(f"http://{self.address}/status", timeout=10) as answer:
                return parse_status(answer.read())
        except OSError as error:
            raise Unmet(f"GET http://{self.address}/status: {error}") from error

    def kill(self) -> None:
        """A kill, never a graceful stop: nothing of the server's shutdown path runs."""
        if self.process.poll() is None:
            self.process.kill()
        self.process.wait(timeout=60)


async def seated(server: Server, cursor: str | None) -> tuple[str, Status]:
    """Joins the server's suggested seat with its invite and the cursor reached so far, waits for an
    observation and the backfill, lets the world run, reads `/status`, and kills the server while the
    client is seated; the client must see the server go. Returns the cursor reached and that status."""
    from mineworld_sdk.session import JoinRefused, SeatSession, SessionClosed
    from mineworld_sdk.wire.frames import Invite, PerceivedJoin
    from mineworld_sdk.wire.ids import EntityKey, EventId

    since = None if cursor is None else EventId(cursor)
    try:
        session = await SeatSession.connect(
            f"ws://{server.address}/ws", seat=EntityKey(server.seat), invite=Invite(server.invite),
            nickname="nightly", perceived=PerceivedJoin(since=since),
        )
    except JoinRefused as refused:
        raise Unmet(f"the client's join (seat {server.seat}, since {cursor}) was refused: {refused.code}") from refused
    try:
        await asyncio.wait_for(session.changed(), SEATED_LIMIT)
        deadline = time.monotonic() + SEATED_LIMIT
        while session.perceived_cursor is None or (cursor is not None and int(session.perceived_cursor) < int(cursor)):
            if time.monotonic() > deadline:
                raise Unmet(f"no perceived backfill from {cursor} within {SEATED_LIMIT:.0f} s "
                            f"(cursor {session.perceived_cursor})")
            await asyncio.sleep(0.1)
        reached = str(session.perceived_cursor)
        await asyncio.sleep(RUNNING_BEFORE_KILL)
        seen = server.status()
        server.kill()
        try:
            await asyncio.wait_for(session.changed(since=10**12), SEEN_DEATH_LIMIT)
            raise Unmet("the client was not told the server had gone")
        except SessionClosed:
            pass
        except TimeoutError as error:
            raise Unmet(f"the client did not see the killed server's socket close within {SEEN_DEATH_LIMIT:.0f} s") from error
        return reached, seen
    finally:
        server.kill()
        await session.__aexit__(None, None, None)


def restarts(binary: str) -> int:
    binary = binary_path(binary)
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-nightly-restarts-"))
    save = scratch / RESTART_WORLD
    timings: list[dict[str, object]] = []
    failures: list[str] = []
    try:
        first: Status | None = None
        cursor: str | None = None
        seen = 0
        for cycle in range(CYCLES + 1):
            began = time.monotonic()
            server = Server(binary, RESTART_WORLD, save)
            try:
                started = server.status()
                first = first or started
                failures += judge_restart(cycle, first, seen, started)
                if cycle == CYCLES or failures:
                    break
                previous = cursor
                cursor, before_kill = asyncio.run(seated(server, cursor))
                seen = before_kill.revision if before_kill.revision is not None else seen
            finally:
                server.kill()
            timings.append({"cycle": cycle, "seconds": round(time.monotonic() - began, 2),
                            "revision_before_kill": seen, "cursor": cursor})
            print(f"[stability] cycle {cycle}: instance {started.instance}, revision {started.revision} at start, "
                  f"{seen} before the kill; perceived from {previous} to {cursor}; faults {started.faults}", flush=True)
        if not failures:
            replayed = replay_one(binary, RESTART_WORLD, save)
            if replayed:
                failures.append(f"the save killed {CYCLES} times does not replay: {replayed}")
            else:
                print(f"[stability] the save killed {CYCLES} times replays", flush=True)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    write_timings("restarts", {"world": RESTART_WORLD, "cycles": timings})
    return verdict("restarts", failures)


# ---------------------------------------------------------------------------------------------- replay


def replay_one(binary: str, world: str, save: Path) -> str | None:
    result = subprocess.run([binary, "replay", f"worlds/{world}", "--save", str(save), *PACK_ROOTS],
                            cwd=ROOT, capture_output=True)
    output = (result.stdout + result.stderr).decode("utf-8", errors="replace")
    reason = judge_replay(world, result.returncode, output)
    if reason is None:
        print(f"[stability] {output.strip()}", flush=True)
    return reason


def ca13(save: Path) -> str | None:
    """CA-13 (`perceived.rs`, ignored: it needs a 300-day market-town save) on the save just made."""
    command = ["cargo", "test", "--locked", "-p", "mineworld-cli", "--test", "perceived", "--", "--ignored",
               "--exact", CA13]
    print(f"[stability] $ MINEWORLD_CA13_SAVE={save} {' '.join(command)}", flush=True)
    status = subprocess.run(command, cwd=ROOT, env={**os.environ, "MINEWORLD_CA13_SAVE": str(save)}).returncode
    return None if status == 0 else f"CA-13 ({CA13}) exited {status}"


def replay(binary: str) -> int:
    binary = binary_path(binary)
    failures: list[str] = []
    timings: dict[str, dict[str, object]] = {}
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-nightly-replay-"))
    try:
        for world in enumerate_worlds(ROOT):
            save = scratch / world
            began = time.monotonic()
            arguments = [binary, "run", f"worlds/{world}", "--headless", "--seed", str(SEED), "--days",
                         str(REPLAY_DAYS), *PACK_ROOTS, "--save", str(save)]
            result = subprocess.run(arguments, cwd=ROOT, capture_output=True)
            ran = time.monotonic() - began
            if result.returncode != 0:
                failures.append(f"run {world} {REPLAY_DAYS} days saved: exited {result.returncode}: "
                                f"{result.stderr.decode(errors='replace')[-600:]}")
                shutil.rmtree(save, ignore_errors=True)
                continue
            size = sum(path.stat().st_size for path in save.rglob("*") if path.is_file())
            began = time.monotonic()
            reason = replay_one(binary, world, save)
            replayed = time.monotonic() - began
            if reason:
                failures.append(reason)
            if world == RESTART_WORLD:
                failures += [reason] if (reason := ca13(save)) else []
            timings[world] = {"days": REPLAY_DAYS, "run_seconds": round(ran, 2), "save_bytes": size,
                              "replay_seconds": round(replayed, 2)}
            print(f"[stability] {world}: {REPLAY_DAYS} days saved in {ran:.1f} s ({size / 1024**3:.2f} GiB), "
                  f"replayed in {replayed:.1f} s", flush=True)
            shutil.rmtree(save, ignore_errors=True)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    write_timings("replay", {"worlds": timings})
    return verdict("replay", failures)


def replay_check(binary: str, world: str, save: str) -> int:
    reason = replay_one(binary_path(binary), world, Path(save))
    return verdict(f"replay {world}", [reason] if reason else [])


# ---------------------------------------------------------------------------------------------- output


def write_timings(program: str, data: dict[str, object]) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"timings-stability-{program}.json").write_text(json.dumps(data, indent=1, sort_keys=True) + "\n",
                                                           encoding="utf-8")


def verdict(program: str, failures: list[str]) -> int:
    for failure in failures:
        print(f"[stability] FAIL {failure}", file=sys.stderr, flush=True)
    print(f"[stability] {program} {'PASS' if not failures else 'FAIL'}", flush=True)
    return 0 if not failures else 1


# ------------------------------------------------------------------------------------------- self-test


def self_test() -> int:
    """The oracles against recorded answers: each case must get its stated verdict."""
    recorded = b'{"protocol":2,"instance":"1a2b","at":4112,"time_scale":60,"paused":false,"entities":41,' \
               b'"systems":[],"seats":["alice"],"clients":1,"observations_dropped":0,"events_dropped":0,' \
               b'"faults":0,"revision":7}'
    first = parse_status(recorded)
    cases: list[tuple[str, bool]] = [
        ("a recorded /status parses", first == Status("1a2b", 7, 0)),
        ("a restart at the same instance, ahead, with no fault passes", judge_restart(1, first, 7, Status("1a2b", 9, 0)) == []),
        ("equal revisions pass", judge_restart(1, first, 9, Status("1a2b", 9, 0)) == []),
        ("another instance fails (AC-6)", "instance 1a2b became ffff" in " ".join(judge_restart(2, first, 7, Status("ffff", 9, 0)))),
        ("a revision behind the kill fails", "behind 9" in " ".join(judge_restart(3, first, 9, Status("1a2b", 8, 0)))),
        ("a null revision fails", "not persisted" in " ".join(judge_restart(4, first, 9, Status("1a2b", None, 0)))),
        ("a fault fails", "faults 2" in " ".join(judge_restart(5, first, 7, Status("1a2b", 7, 2)))),
        ("the join line is read",
         parse_join("[mineworld] invite abc123 — join with: 127.0.0.1:5000 seat=alice invite=abc123")
         == ("127.0.0.1:5000", "abc123", "alice")),
        ("a given invite's join line is not taken for a generated one",
         parse_join("[mineworld] join with: 127.0.0.1:5000 seat=alice invite=<the invite you gave>") is None),
        ("a replay that reproduced passes", judge_replay("w", 0, "x: 9 revision(s) re-executed from genesis, 5 fact(s) "
                                                         "and 2 snapshot(s) reproduced byte for byte; head revision 9") is None),
        ("a refused replay fails, naming the world", "replay w: exited 1" in (judge_replay("w", 1, "does not reproduce") or "")),
        ("exit 0 without the verdict line fails", "printed no" in (judge_replay("w", 0, "") or "")),
    ]
    try:
        parse_status(b'{"instance": "x"}')
        cases.append(("a /status without faults is refused", False))
    except Unmet:
        cases.append(("a /status without faults is refused", True))
    failed = 0
    for name, good in cases:
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}")
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


USAGE = ("usage: ci_stability.py restarts --binary B | replay --binary B | "
         "replay-check --binary B --world W --save DIR | --self-test")


def main(arguments: list[str]) -> int:
    options = dict(zip(arguments[1::2], arguments[2::2]))
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if arguments[:1] == ["restarts"] and set(options) == {"--binary"} and len(arguments) == 3:
            return restarts(options["--binary"])
        if arguments[:1] == ["replay"] and set(options) == {"--binary"} and len(arguments) == 3:
            return replay(options["--binary"])
        if arguments[:1] == ["replay-check"] and set(options) == {"--binary", "--world", "--save"} and len(arguments) == 7:
            return replay_check(options["--binary"], options["--world"], options["--save"])
    except Unmet as unmet:
        return verdict(arguments[0], [str(unmet)])
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
