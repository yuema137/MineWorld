"""AP5-6 (a): replay builds no network client; AP5-10 (b), (c): the network guard is active, and pytest
runs once per workspace member."""

from __future__ import annotations

import ast
import json
import shlex
import subprocess
import sys
import time
import tomllib
from pathlib import Path
from typing import cast

import pytest
from pytest_socket import SocketConnectBlockedError
from support import completion, request, run

from mineworld_cognition.backend.openai_compatible import OpenAICompatibleBackend
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import BackendConfig
from mineworld_cognition.record import RecordingBackend

PACKAGE = Path(__file__).resolve().parent.parent
REPOSITORY = PACKAGE.parent.parent

CHILD = """
import asyncio, json, sys
from pathlib import Path
from mineworld_sdk.wire.ids import EntityKey
from mineworld_cognition.backend.model import CompletionRequest, Message, Sampling
from mineworld_cognition.config import build_router, load
from mineworld_cognition.gateway import Completed

ADAPTERS = {"openai_compatible", "anthropic_messages", "cli_bridge"}

async def main() -> None:
    router = build_router(load(Path(sys.argv[1])))
    gateway = router.for_tier("social")
    asked = CompletionRequest(
        purpose="decide",
        messages=(Message(role="user", text="replayed"),),
        sampling=Sampling(temperature_milli=0, max_output_tokens=64, seed=1),
    )
    outcome = await gateway.complete(EntityKey("alice"), asked)
    await router.aclose()
    print(json.dumps({
        "completed": isinstance(outcome, Completed),
        "modules": sorted(name for name in sys.modules
                          if name.startswith("httpx") or name.rsplit(".", 1)[-1] in ADAPTERS),
    }))

asyncio.run(main())
"""


def test_replay_mode_never_imports_the_http_client(tmp_path: Path) -> None:
    # AP5-6 (a), in a child interpreter so that this test process's own imports do not count.
    recorder = RecordingBackend(
        ScriptedBackend(lambda _: completion("from the cassette")),
        tmp_path / "cassette.jsonl",
        binding="local",
        model="m",
    )

    async def record() -> None:
        await recorder.complete(request("replayed"))
        await recorder.aclose()

    run(record())
    config = tmp_path / "cognition.toml"
    config.write_text(
        '[recording]\nmode = "replay"\ncassette = "cassette.jsonl"\n[tiers]\nsocial = "local"\n'
        'major_decision = "claude"\n'
        '[backends.local]\nbase_url = "http://127.0.0.1:11434/v1"\nmodel = "m"\n'
        '[backends.claude]\nkind = "anthropic"\nbase_url = "https://api.example.com"\nmodel = "m"\n'
        'key_env = "MWTEST_UNSET_KEY"\n',
        encoding="utf-8",
    )
    script = tmp_path / "child.py"
    script.write_text(CHILD, encoding="utf-8")
    result = subprocess.run(
        [sys.executable, str(script), str(config)],
        capture_output=True,
        text=True,
        timeout=60,
        check=False,
    )
    assert result.returncode == 0, result.stderr
    assert json.loads(result.stdout) == {"completed": True, "modules": []}


def test_the_member_configures_the_network_guard() -> None:
    # AP5-10 (b), the configuration half.
    with (PACKAGE / "pyproject.toml").open("rb") as file:
        options = tomllib.load(file)["tool"]["pytest"]["ini_options"]
    assert "--allow-hosts=127.0.0.1,::1" in options["addopts"]
    # `slow` cases run each night, not per PR (pr-s28a D-a-15).
    assert options["addopts"][-2:] == ["-m", "not live_model and not slow"]


@pytest.mark.filterwarnings("ignore:A test tried to use socket")
def test_a_connection_beyond_loopback_is_blocked_at_once() -> None:
    # AP5-10 (b): TEST-NET-1 is never routable; without the guard this would hang until a timeout.
    config = BackendConfig.model_validate(
        {"base_url": "https://192.0.2.1:80/v1", "model": "m", "request_timeout_s": 30}
    )
    backend = OpenAICompatibleBackend(config, None)

    async def attempt() -> None:
        try:
            await backend.complete(request("hi"))
        finally:
            await backend.aclose()

    started = time.monotonic()
    with pytest.raises(BaseException) as raised:
        run(attempt())
    elapsed = time.monotonic() - started
    # httpx2's connector runs inside an anyio task group, which wraps the guard's error in a group.
    assert _contains(raised.value, SocketConnectBlockedError), repr(raised.value)
    assert elapsed < 1


def _contains(error: BaseException, kind: type[BaseException]) -> bool:
    if isinstance(error, kind):
        return True
    if isinstance(error, BaseExceptionGroup):
        return any(_contains(inner, kind) for inner in error.exceptions)  # pyright: ignore[reportUnknownVariableType, reportUnknownMemberType, reportUnknownArgumentType]
    return error.__cause__ is not None and _contains(error.__cause__, kind)


def _python_members() -> list[str]:
    """`PYTHON_MEMBERS` as `scripts/ci_layer.py` declares it, read without importing the script."""
    tree = ast.parse((REPOSITORY / "scripts" / "ci_layer.py").read_text(encoding="utf-8"))
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(
            isinstance(t, ast.Name) and t.id == "PYTHON_MEMBERS" for t in node.targets
        ):
            members: object = ast.literal_eval(node.value)
            assert isinstance(members, list)
            return [str(member) for member in cast(list[object], members)]
    raise AssertionError("scripts/ci_layer.py declares no PYTHON_MEMBERS")


def test_ci_runs_pytest_once_per_member() -> None:
    # AP5-10 (c), D-P5-11; pr-s27c D-c-22 (written to hold for every PR that adds a pytest line):
    # every pytest line is unique, every `PYTHON_MEMBERS` entry has exactly one, and any other names a
    # path that exists in the repository.
    members = _python_members()
    assert members[:2] == ["sdk/python", "cognition/lm-controller"], members
    for layer in ("python", "python-smoke"):
        listed = subprocess.run(
            [sys.executable, "scripts/ci_layer.py", "--list", layer],
            cwd=REPOSITORY,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.splitlines()
        pytest_lines = [line for line in listed if " pytest " in line]
        assert len(pytest_lines) == len(set(pytest_lines)), listed
        paths = [shlex.split(line)[4] for line in pytest_lines]
        for member in members:
            assert paths.count(member) == 1, (member, listed)
        for path in paths:
            assert path in members or (REPOSITORY / path).exists(), (path, listed)
        assert (
            pytest_lines[paths.index("cognition/lm-controller")]
            == {
                "python": "uv run --locked pytest cognition/lm-controller",
                # pr-s10-p4 §5.10: a command-line `-m` replaces the addopts one, so it restates it.
                "python-smoke": "uv run --locked pytest cognition/lm-controller "
                "-m 'not live_model and not real_binary and not slow'",
            }[layer]
        )
        assert not any("live_model" in line and "not live_model" not in line for line in listed)
