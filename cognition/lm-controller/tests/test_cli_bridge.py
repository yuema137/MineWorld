"""AB-3, AB-4, AB-5, AB-6 and AB-8: the subscription bridge's core, over a test-only preset and a fake CLI.

No real CLI runs: every child is `tests/fakes/fake_cli.py` under this interpreter, started directly, or
through a launcher found on `PATH` (a `.cmd` shim on Windows, a shell script elsewhere). These tests run
on the platform's default event loop, not `support.run`'s selector loop: Windows' selector loop cannot
start a subprocess (F-P5b-1), and no test here opens a socket.
"""

from __future__ import annotations

import asyncio
import json
import os
import re
import secrets as stdlib_secrets
import shutil
import stat
import sys
import time
from collections.abc import Coroutine
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

import pytest
from mineworld_sdk.wire.ids import EntityKey, JsonValue
from support import request

from mineworld_cognition.backend.cli_bridge import CliBridgeBackend, child_environment
from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    Message,
    Sampling,
    Usage,
    estimate_tokens,
)
from mineworld_cognition.budget import BudgetPolicy, MemoryLedger
from mineworld_cognition.errors import ConfigError
from mineworld_cognition.gateway import Budget, Completed, Failed, ModelGateway, Refused
from mineworld_cognition.secrets import resolve_key

FAKE_CLI = Path(__file__).resolve().parent / "fakes" / "fake_cli.py"
SOURCE = Path(__file__).resolve().parent.parent / "src" / "mineworld_cognition"
ALICE = EntityKey("alice")


def run_loop[T](coroutine: Coroutine[Any, Any, T], *, timeout_s: float = 30) -> T:
    async def bounded() -> T:
        async with asyncio.timeout(timeout_s):
            return await coroutine

    return asyncio.run(bounded())


class FakeClock:
    def __init__(self, now: int = 1_000_000) -> None:
        self.at = now

    def now(self) -> int:
        return self.at


@dataclass(frozen=True)
class FakePreset:
    """A test-only preset: the fake CLI's argv literals carry its record file and its mode."""

    record: Path
    mode: str = "answer"
    executable: str = "fake_model_cli"
    home_variable: str | None = "FAKE_CLI_HOME"
    seen_schema: list[JsonValue] = field(default_factory=list[JsonValue])

    def arguments(self, *, model: str | None, schema_path: Path | None) -> list[str]:
        argv = [str(FAKE_CLI)] if self.executable == "<direct>" else []
        argv += ["--record", str(self.record), "--mode", self.mode]
        if schema_path is not None:
            argv += ["--schema", str(schema_path)]
        if model is not None:
            argv += ["-m", model]
        return argv

    def schema_document(self, schema: JsonValue) -> JsonValue:
        self.seen_schema.append(schema)
        return schema

    def render_prompt(self, request: CompletionRequest) -> str:
        return "\n\n".join(f"[{message.role}]\n{message.text}" for message in request.messages)

    def parse(self, stdout: bytes, request: CompletionRequest) -> Completion | BackendFailure:
        payload = json.loads(stdout)
        text = payload["text"]
        usage = payload.get("usage")
        if usage is None:
            asked = "".join(message.text for message in request.messages)
            counted = Usage(
                input_tokens=estimate_tokens(asked),
                output_tokens=estimate_tokens(text),
                estimated=True,
            )
        else:
            counted = Usage(
                input_tokens=usage["input_tokens"],
                output_tokens=usage["output_tokens"],
                estimated=False,
            )
        return Completion(text=text, finish="complete", usage=counted)

    def classify(self, stderr_first_line: str, exit_code: int) -> BackendFailure:
        if exit_code == 3 and "not logged in" in stderr_first_line:
            return BackendFailure(reason="unauthorized")
        return BackendFailure(reason="unreachable")


def _direct(
    record: Path,
    mode: str = "answer",
    *,
    model: str | None = None,
    request_timeout_s: int | None = None,
) -> CliBridgeBackend:
    preset = FakePreset(record, mode, executable="<direct>")
    return CliBridgeBackend(
        preset, command=[sys.executable], model=model, request_timeout_s=request_timeout_s
    )


def _records(record: Path) -> list[dict[str, Any]]:
    if not record.exists():
        return []
    return [json.loads(line) for line in record.read_text(encoding="utf-8").splitlines()]


def _alive(pid: int) -> bool:
    if sys.platform == "win32":
        import ctypes

        kernel32 = ctypes.windll.kernel32
        handle = kernel32.OpenProcess(
            0x00100000 | 0x1000, False, pid
        )  # SYNCHRONIZE | QUERY_LIMITED
        if not handle:
            return False
        try:
            return kernel32.WaitForSingleObject(handle, 0) == 0x102  # WAIT_TIMEOUT: still running
        finally:
            kernel32.CloseHandle(handle)
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    stat_file = Path(f"/proc/{pid}/stat")
    if stat_file.exists():  # Linux: a zombie nobody has reaped yet is dead
        return stat_file.read_text().rsplit(")", 1)[1].split()[0] != "Z"
    return True


def test_no_secret_reaches_the_cli_and_its_directory_is_empty_and_removed(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # AB-3.
    planted = {
        name: f"MWTEST-{name}-{stdlib_secrets.token_hex(8)}"
        for name in ("OPENAI_API_KEY", "ANTHROPIC_API_KEY", "CODEX_API_KEY", "FOO_TOKEN")
    }
    for name, value in planted.items():
        monkeypatch.setenv(name, value)
    file_value = "MWTEST-file-" + stdlib_secrets.token_hex(8)
    env_file = tmp_path / "mineworld.env"
    env_file.write_text(f"MWTEST_FILE_KEY={file_value}\n", encoding="utf-8")
    env_file.chmod(0o600)
    monkeypatch.delenv("MWTEST_FILE_KEY", raising=False)
    # The same process has resolved a key from the file, as an API-key backend beside the bridge would.
    assert resolve_key("MWTEST_FILE_KEY", env_file, backend="claude").reveal() == file_value
    monkeypatch.setenv("FAKE_CLI_HOME", str(tmp_path / "cli-home"))
    record = tmp_path / "record.jsonl"
    backend = _direct(record)
    answer = run_loop(backend.complete(request("hello")))
    assert isinstance(answer, Completion)
    (seen,) = _records(record)
    shown = json.dumps({key: seen[key] for key in ("argv", "env")}) + bytes.fromhex(
        seen["stdin_hex"]
    ).decode("utf-8")
    for value in (*planted.values(), file_value):
        assert value not in shown
    assert not any(re.search(r"_API_KEY$|_TOKEN$", name) for name in seen["env"])
    assert seen["env"]["FAKE_CLI_HOME"] == str(tmp_path / "cli-home")  # the preset's home variable
    assert seen["cwd_entries"] == []
    cwd = Path(seen["cwd"])
    assert cwd.name.startswith("mineworld-bridge-")
    assert not cwd.exists()


def test_the_environment_is_an_allowlist() -> None:
    source = {
        "PATH": "/bin",
        "HOME": "/home/u",
        "FAKE_CLI_HOME": "/home/u/.fake",
        "OPENAI_BASE_URL": "x",
        "SOMETHING_ELSE": "y",
        "AWS_SESSION_TOKEN": "z",
    }
    assert child_environment(source, "FAKE_CLI_HOME") == {
        "PATH": "/bin",
        "HOME": "/home/u",
        "FAKE_CLI_HOME": "/home/u/.fake",
    }
    assert child_environment(source, "OPENAI_BASE_URL") == {"PATH": "/bin", "HOME": "/home/u"}


HOSTILE = '& calc.exe %PATH% $(id) "; rm -rf /'


def _hostile_request() -> CompletionRequest:
    filler = "".join(f"line {number}: ünïcødé & | < > ^ %X% !Y!\n" for number in range(300))
    return CompletionRequest(
        purpose="decide",
        messages=(
            Message(role="system", text="You are Alice."),
            Message(role="user", text=HOSTILE + "\n" + filler),
        ),
        output_schema={"type": "object", "properties": {"say": {"type": "string"}}},
        sampling=Sampling(temperature_milli=0, max_output_tokens=64),
    )


def _assert_prompt_only_on_stdin(seen: dict[str, Any], asked: CompletionRequest) -> None:
    argv = " ".join(seen["argv"])
    for fragment in ("calc.exe", "%PATH%", "$(id)", "rm -rf", "line 1:", "Alice"):
        assert fragment not in argv
    preset_prompt = "\n\n".join(f"[{m.role}]\n{m.text}" for m in asked.messages)
    assert bytes.fromhex(seen["stdin_hex"]) == preset_prompt.encode("utf-8")
    assert len(seen["stdin_hex"]) // 2 > 10_000


def test_no_model_facing_text_reaches_argv(tmp_path: Path) -> None:
    # AB-4, the direct start; the schema goes by a generated path, and the model is validated.
    record = tmp_path / "record.jsonl"
    asked = _hostile_request()
    answer = run_loop(_direct(record, model="gpt-test.1:mini").complete(asked))
    assert isinstance(answer, Completion)
    (seen,) = _records(record)
    _assert_prompt_only_on_stdin(seen, asked)
    schema_at = seen["argv"][seen["argv"].index("--schema") + 1]
    assert Path(schema_at).name == "schema.json"
    assert seen["argv"][-2:] == ["-m", "gpt-test.1:mini"]
    with pytest.raises(ConfigError, match="model"):
        _direct(record, model="m; rm -rf /")


@pytest.mark.skipif(sys.platform != "win32", reason="a .cmd shim exists on Windows only")
def test_no_model_facing_text_reaches_argv_through_a_cmd_shim_on_windows(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # AB-4 through discovery (D-B1): the `.cmd` shim, found through PATHEXT and run by cmd.exe, whose
    # argument parsing is the injection surface. AB-9 requires this case by name on the Windows leg.
    _through_a_launcher(tmp_path, monkeypatch)
    found = shutil.which("fake_model_cli")
    assert found is not None and found.lower().endswith(".cmd")


@pytest.mark.skipif(sys.platform == "win32", reason="the shell launcher is POSIX's")
def test_no_model_facing_text_reaches_argv_through_a_shell_launcher(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    _through_a_launcher(tmp_path, monkeypatch)


def _through_a_launcher(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    launchers = tmp_path / "bin"
    launchers.mkdir()
    if sys.platform == "win32":
        shim = launchers / "fake_model_cli.cmd"
        shim.write_text(
            f'@echo off\r\n"{sys.executable}" "{FAKE_CLI}" %*\r\n', encoding="utf-8", newline=""
        )
    else:
        shim = launchers / "fake_model_cli"
        shim.write_text(f'#!/bin/sh\nexec "{sys.executable}" "{FAKE_CLI}" "$@"\n', encoding="utf-8")
        shim.chmod(shim.stat().st_mode | stat.S_IXUSR)
    monkeypatch.setenv("PATH", f"{launchers}{os.pathsep}{os.environ.get('PATH', '')}")
    record = tmp_path / "record.jsonl"
    backend = CliBridgeBackend(FakePreset(record))
    assert repr(backend) == "CliBridgeBackend('fake_model_cli', model=None)"
    asked = _hostile_request()
    answer = run_loop(backend.complete(asked))
    assert isinstance(answer, Completion), answer
    (seen,) = _records(record)
    _assert_prompt_only_on_stdin(seen, asked)


def test_a_missing_executable_names_what_was_looked_for(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # D-B1: no fallback search; the message names the command and suggests `command = [...]`.
    monkeypatch.setenv("PATH", str(tmp_path))
    with pytest.raises(ConfigError) as refused:
        CliBridgeBackend(FakePreset(tmp_path / "r.jsonl", executable="mwtest_absent_cli"))
    assert "'mwtest_absent_cli' was not found on PATH" in str(refused.value)
    assert "command = [" in str(refused.value)
    # An executable that disappears after construction is unreachable at call time.
    gone = CliBridgeBackend(FakePreset(tmp_path / "r.jsonl"), command=[str(tmp_path / "gone")])
    assert run_loop(gone.complete(request("x"))) == BackendFailure(reason="unreachable")


def test_a_failing_cli_is_classified_from_stderr_and_carries_none_of_it(tmp_path: Path) -> None:
    record = tmp_path / "record.jsonl"
    answer = run_loop(_direct(record, mode="fail").complete(request("x")))
    assert answer == BackendFailure(reason="unauthorized")
    assert "logged" not in repr(answer)


def test_the_timeout_kills_the_whole_tree(tmp_path: Path) -> None:
    # AB-5: through the gateway, whose timeout cancels the call.
    record = tmp_path / "record.jsonl"
    policy = BudgetPolicy(call_timeout_s=1)
    gateway = ModelGateway(
        _direct(record, mode="hang"), Budget.of(policy, MemoryLedger(), FakeClock())
    )
    started = time.monotonic()
    outcome = run_loop(gateway.complete(ALICE, request("hang")))
    elapsed = time.monotonic() - started
    assert outcome == Failed(BackendFailure(reason="timeout"))
    assert elapsed < 2, elapsed
    (seen,) = _records(record)
    pids = {"child": seen["pid"], "grandchild": seen["grandchild"]}
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline and any(_alive(pid) for pid in pids.values()):
        time.sleep(0.05)
    assert {name: pid for name, pid in pids.items() if _alive(pid)} == {}


def test_the_bridges_own_bound_also_kills_the_tree(tmp_path: Path) -> None:
    record = tmp_path / "record.jsonl"
    backend = _direct(record, mode="hang", request_timeout_s=1)
    assert run_loop(backend.complete(request("hang"))) == BackendFailure(reason="timeout")
    (seen,) = _records(record)
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline and _alive(seen["grandchild"]):
        time.sleep(0.05)
    assert not _alive(seen["grandchild"])


def test_budgets_bound_the_bridge_before_the_spawn(tmp_path: Path) -> None:
    # AB-8: the 21st call is refused and never spawned; usage is charged as reported, else estimated.
    record = tmp_path / "record.jsonl"
    ledger = MemoryLedger()
    clock = FakeClock()
    budget = Budget.of(BudgetPolicy(calls_per_wall_hour=20), ledger, clock)
    reported = ModelGateway(_direct(record), budget)
    estimated = ModelGateway(_direct(record, mode="answer-no-usage"), budget)

    async def session() -> list[object]:
        outcomes: list[object] = []
        for number in range(21):
            gateway = reported if number < 19 else estimated
            outcomes.append(await gateway.complete(ALICE, request(f"call {number}")))
        return outcomes

    outcomes = run_loop(session(), timeout_s=120)
    assert all(isinstance(outcome, Completed) for outcome in outcomes[:20])
    last = outcomes[20]
    assert isinstance(last, Refused)
    assert last.refusal.limit == "calls_per_wall_hour"
    assert len(_records(record)) == 20  # the spawn counter
    first, twentieth = outcomes[0], outcomes[19]
    assert isinstance(first, Completed) and isinstance(twentieth, Completed)
    assert first.completion.usage == Usage(input_tokens=11, output_tokens=4, estimated=False)
    assert twentieth.completion.usage.estimated is True
    window = ledger.window(ALICE, clock.now(), 3600)
    assert window.calls == 20
    assert window.tokens == 19 * 15 + (
        twentieth.completion.usage.input_tokens + twentieth.completion.usage.output_tokens
    )


CREDENTIAL_MARKERS = ["auth.json", ".codex", ".claude", ".credentials", "keychain", "keyring",
                      "CLAUDE_CODE_OAUTH"]  # fmt: skip


def credential_findings(root: Path) -> list[str]:
    return [
        f"{path.relative_to(root).as_posix()}:{number}: {marker}"
        for path in sorted(root.rglob("*.py"))
        for number, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1)
        for marker in CREDENTIAL_MARKERS
        if marker.lower() in line.lower()
    ]


def test_nothing_in_the_package_touches_a_cli_credential(tmp_path: Path) -> None:
    # AB-6, the source half; the environment half is AB-3's allowlist assertion.
    assert credential_findings(SOURCE) == []
    planted = tmp_path / "backend" / "presets" / "codex.py"
    planted.parent.mkdir(parents=True)
    planted.write_text('AUTH = Path.home() / ".codex" / "auth.json"\n', encoding="utf-8")
    assert credential_findings(tmp_path) == [
        "backend/presets/codex.py:1: auth.json",
        "backend/presets/codex.py:1: .codex",
    ]
