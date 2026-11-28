"""Supervised mode's local contracts (`docs/reference/cognition-supervised.md`; design `pr-playable-dialogue-memory.md`
§§3.4, 4.8; commits C2/C3).

What is owned here, deterministically: the supervisor's paths win over the configuration and are refused unless loopback
and absolute; a supervised run's standard output is status lines only, with the closed reason of what stopped it; the
runtime manifest of this checkout passes its own preflight and a changed source file fails it; the store is bound to
the first world instance it is welcomed to, and a later instance is refused before anything in the store changes. The
real-process lifecycle (a launcher, a real server, stdin EOF, readiness, kill) is the launcher's real-process test
(`tools/launch/tests/launcher/cognition.rs`), not repeated here.
"""

from __future__ import annotations

import asyncio
import json
import os
import subprocess
import sys
from pathlib import Path

import pytest
from memory_fixtures import INSTANCE, OTHER_INSTANCE, alice_identity
from mineworld_sdk.reconnect import SeatLost
from support import run
from test_controller_config import write

from mineworld_cognition.config import load
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.memory.home.errors import HomeBusy
from mineworld_cognition.memory.records import StoreIdentity
from mineworld_cognition.runtime_manifest import Manifest, derive, differences, read
from mineworld_cognition.supervised import (
    BoundHomes,
    Degraded,
    InstanceMismatch,
    Ready,
    Status,
    _overridden,  # pyright: ignore[reportPrivateUsage]
    reason_of,
)

LIVE_LOCAL = """
server = "ws://127.0.0.1:1/ws"
store = "elsewhere"
decisions_json = "theirs.json"
invite_env = "SOMETHING_ELSE"

[recording]
mode = "replay"
cassette = "c.jsonl"

[tiers]
social = "local"

[backends.local]
base_url = "http://127.0.0.1:11434/v1"
model = "m"
"""

FIXTURES = Path(__file__).resolve().parent / "fixtures"


def _command(*arguments: str | Path) -> list[str]:
    return [sys.executable, "-I", "-m", "mineworld_cognition", *map(str, arguments)]


def _lines(text: str) -> list[dict[str, object]]:
    return [json.loads(line) for line in text.splitlines()]


def test_the_supervisors_paths_and_invite_variable_win_over_the_configuration(
    tmp_path: Path,
) -> None:
    config = load(write(tmp_path, LIVE_LOCAL))
    store = tmp_path / "cognition" / "market-town"
    assignment = store / "run" / "assignment.json"
    overridden = _overridden(config, "ws://127.0.0.1:5000/ws", store, assignment)
    assert (
        overridden.server,
        overridden.store,
        overridden.decisions_json,
        overridden.invite_env,
    ) == (
        "ws://127.0.0.1:5000/ws",
        str(store),
        str(assignment),
        "MINEWORLD_INVITE",
    )
    assert overridden.budgets.ledger == str(store / "budget.sqlite"), (
        "the store's ledger, none named"
    )
    assert (
        _overridden(config, "ws://[::1]:5000/ws", store, assignment).server == "ws://[::1]:5000/ws"
    )

    for url in (
        "ws://192.168.1.4:5000/ws",
        "ws://localhost:5000/ws",
        "wss://127.0.0.1:5000/ws",
        "ws://127.0.0.1/ws",
    ):
        with pytest.raises(Degraded) as refused:
            _overridden(config, url, store, assignment)
        assert refused.value.reason == "config_invalid", url
    with pytest.raises(Degraded) as relative:
        _overridden(config, "ws://127.0.0.1:5000/ws", Path("cognition"), assignment)
    assert relative.value.reason == "config_invalid"

    remote = load(
        write(
            tmp_path,
            LIVE_LOCAL.replace("http://127.0.0.1:11434/v1", "https://models.example.org/v1"),
        )
    )
    with pytest.raises(Degraded) as not_loopback:
        _overridden(remote, "ws://127.0.0.1:5000/ws", store, assignment)
    assert not_loopback.value.reason == "backend_not_loopback"


def test_a_supervised_run_writes_only_status_lines_on_stdout_and_names_its_reason(
    tmp_path: Path,
) -> None:
    config = write(tmp_path, LIVE_LOCAL)
    store = tmp_path / "store"
    common = [
        "--config",
        config,
        "--supervised",
        "--server-url",
        "ws://127.0.0.1:1/ws",
        "--store-dir",
        store,
    ]
    environment = {**os.environ, "MINEWORLD_INVITE": "planted-invite-4c1e"}

    missing = subprocess.run(
        _command(*common, "--assignment", tmp_path / "absent.json"),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        env=environment,
        timeout=60,
        check=False,
    )
    assert missing.returncode == 3, missing.stderr
    assert _lines(missing.stdout) == [
        {"status": 1, "state": "starting", "seat": None, "reason": None},
        {"status": 1, "state": "degraded", "seat": None, "reason": "assignment_invalid"},
    ]

    two = json.loads((FIXTURES / "decisions_format3.json").read_text(encoding="utf-8"))
    two["seats"][1]["model"] = {"id": "rule", "assigned": "lm", "chain": ["lm", "rule"]}
    (tmp_path / "two.json").write_text(json.dumps(two), encoding="utf-8")
    many = subprocess.run(
        _command(*common, "--assignment", tmp_path / "two.json"),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        env=environment,
        timeout=60,
        check=False,
    )
    assert _lines(many.stdout)[-1]["reason"] == "assignment_invalid", "exactly one lm seat"

    uninvited = subprocess.run(
        _command(*common, "--assignment", FIXTURES / "decisions_format3.json"),
        stdin=subprocess.DEVNULL,
        capture_output=True,
        text=True,
        env={k: v for k, v in environment.items() if k != "MINEWORLD_INVITE"},
        timeout=60,
        check=False,
    )
    assert _lines(uninvited.stdout) == [
        {"status": 1, "state": "starting", "seat": None, "reason": None},
        {"status": 1, "state": "starting", "seat": "alice", "reason": None},
        {"status": 1, "state": "degraded", "seat": "alice", "reason": "config_invalid"},
    ]
    for printed in (missing, many, uninvited):
        assert "planted-invite-4c1e" not in printed.stdout + printed.stderr

    # A supervisor's pipe still open when the process ends on a reason: it exits with its status, never with
    # an interpreter abort from the stdin watcher (a packaged run's SIGABRT, design §10 PR-B finding F-B1).
    held = subprocess.Popen(
        _command(*common, "--assignment", tmp_path / "absent.json"),
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=environment,
    )
    code = held.wait(timeout=60)
    assert code == 3, (code, held.stderr.read() if held.stderr else "")
    if held.stdin is not None:
        held.stdin.close()

    # Without --supervised the supervisor's flags are refused, and the standalone command is unchanged.
    alone = subprocess.run(
        _command("--config", config, "--store-dir", store),
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    assert (alone.returncode, alone.stdout) == (2, "")


def test_this_checkouts_manifest_passes_its_preflight_and_a_changed_source_file_fails_it(
    tmp_path: Path,
) -> None:
    manifest = tmp_path / "runtime-manifest.json"
    written = subprocess.run(
        _command("--write-runtime-manifest", manifest),
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    assert written.returncode == 0, written.stderr
    recorded = read(manifest)
    own = {item.name: item for item in recorded.distributions}
    assert own["mineworld-cognition"].files and own["mineworld-sdk"].files, (
        "the imported source files are hashed"
    )
    assert any(
        file.path == "mineworld_cognition/supervised.py"
        for file in own["mineworld-cognition"].files
    )
    assert differences(recorded, derive()) == [], "the same interpreter derives the same manifest"

    config = write(tmp_path, LIVE_LOCAL)  # replay: no backend is asked
    passed = subprocess.run(
        _command("--config", config, "--preflight", manifest),
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    assert (passed.returncode, _lines(passed.stdout)) == (
        0,
        [{"status": 1, "state": "starting", "seat": None, "reason": None}],
    ), passed.stderr

    # A checkout updated since the manifest was written: one source file's digest differs.
    changed = recorded.model_copy(
        update={
            "distributions": [
                item.model_copy(
                    update={
                        "files": [f.model_copy(update={"sha256": "0" * 64}) for f in item.files]
                    }
                )
                if item.name == "mineworld-sdk"
                else item
                for item in recorded.distributions
            ]
        }
    )
    stale = tmp_path / "stale.json"
    stale.write_text(
        Manifest.model_validate(changed.model_dump()).model_dump_json(), encoding="utf-8"
    )
    failed = subprocess.run(
        _command("--config", config, "--preflight", stale),
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    assert (failed.returncode, _lines(failed.stdout)) == (
        3,
        [{"status": 1, "state": "degraded", "seat": None, "reason": "runtime_mismatch"}],
    )
    assert "distribution mineworld-sdk" in failed.stderr

    garbage = tmp_path / "garbage.json"
    garbage.write_text("{}", encoding="utf-8")
    unreadable = subprocess.run(
        _command("--config", config, "--preflight", garbage),
        capture_output=True,
        text=True,
        timeout=120,
        check=False,
    )
    assert _lines(unreadable.stdout)[-1]["reason"] == "runtime_mismatch"


class _Collected:
    def __init__(self) -> None:
        self.text = ""

    def write(self, text: str) -> int:
        self.text += text
        return len(text)

    def flush(self) -> None:
        pass


def test_the_store_is_bound_to_its_first_world_instance_and_another_is_refused_untouched(
    tmp_path: Path,
) -> None:
    config = load(write(tmp_path, LIVE_LOCAL))
    store = tmp_path / "cognition" / "market-town"
    out = _Collected()
    homes = BoundHomes(store, config, Ready(Status(out), "alice"))  # pyright: ignore[reportArgumentType]

    async def hold(identity: StoreIdentity) -> None:
        await homes.hold(identity)
        homes.release(identity.seat)

    run(hold(alice_identity()))
    assert (store / "instance").read_text(encoding="utf-8").strip() == str(INSTANCE)
    assert _lines(out.text) == [{"status": 1, "state": "ready", "seat": "alice", "reason": None}]
    run(hold(alice_identity()))  # the same instance again: held

    before = {path: path.read_bytes() for path in store.rglob("*") if path.is_file()}
    other = alice_identity().model_copy(update={"world_instance": OTHER_INSTANCE})
    with pytest.raises(InstanceMismatch):
        run(hold(other))
    after = {path: path.read_bytes() for path in store.rglob("*") if path.is_file()}
    assert after == before, (
        "nothing in the store was written, and no home of the other instance was created"
    )
    assert not (store / str(OTHER_INSTANCE)).exists()
    assert len(_lines(out.text)) == 2, "no ready for a refused instance"
    homes.close()


def test_what_ended_serving_is_named_by_its_closed_reason_inside_exception_groups() -> None:
    grouped = ExceptionGroup(
        "serving", [ValueError("unrelated"), ExceptionGroup("seat", [HomeBusy("x")])]
    )
    assert reason_of(grouped) == "home_busy"
    assert reason_of(InstanceMismatch("x")) == "instance_mismatch"
    assert reason_of(SeatLost("unknown_seat")) == "seat_unavailable"
    assert reason_of(SeatLost("gave_up")) == "server_unreachable"
    assert reason_of(ConfigError("x")) == "config_invalid"
    assert reason_of(asyncio.CancelledError()) is None, "anything unnamed is raised, not renamed"
