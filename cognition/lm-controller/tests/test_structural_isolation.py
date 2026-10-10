"""AP5-6 (a): replay builds no network client; AP5-10 (b), (c): the network guard is active, and pytest
runs once per workspace member."""

from __future__ import annotations

import json
import subprocess
import sys
import time
import tomllib
from pathlib import Path

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
    assert options["addopts"][-2:] == ["-m", "not live_model"]


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


def test_ci_runs_pytest_once_per_member() -> None:
    # AP5-10 (c), D-P5-11.
    for layer in ("python", "python-smoke"):
        listed = subprocess.run(
            [sys.executable, "scripts/ci_layer.py", "--list", layer],
            cwd=REPOSITORY,
            capture_output=True,
            text=True,
            check=True,
        ).stdout.splitlines()
        pytest_lines = [line for line in listed if " pytest " in line]
        assert len(pytest_lines) == 2, listed
        assert pytest_lines[0].startswith("uv run --locked pytest sdk/python")
        assert (
            pytest_lines[1]
            == {
                "python": "uv run --locked pytest cognition/lm-controller",
                # pr-s10-p4 §5.10: a command-line `-m` replaces the addopts one, so it restates it.
                "python-smoke": "uv run --locked pytest cognition/lm-controller "
                "-m 'not live_model and not real_binary'",
            }[layer]
        )
        assert not any("live_model" in line and "not live_model" not in line for line in listed)
