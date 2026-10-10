"""AP5-9 (no secret in any artefact) and AP5-13 (key files stay the user's and out of the environment)."""

from __future__ import annotations

import logging
import os
import secrets as stdlib_secrets
import subprocess
import sys
from pathlib import Path

import httpx2
import pytest
from mineworld_sdk.wire.ids import EntityKey
from support import request, run

from mineworld_cognition.backend import registry
from mineworld_cognition.backend.openai_compatible import OpenAICompatibleBackend
from mineworld_cognition.config import BackendConfig, ConfigError, build_router, load
from mineworld_cognition.secrets import Secret, permission_check_applies, resolve_key

ALICE = EntityKey("alice")
REPOSITORY = Path(__file__).resolve().parents[3]
VARIABLE = "MWTEST_PROVIDER_KEY"
ANSWER = (
    b'{"choices":[{"message":{"content":"fine"},"finish_reason":"stop"}],'
    b'"usage":{"prompt_tokens":3,"completion_tokens":1}}'
)


def _key_file(directory: Path, text: str, mode: int = 0o600) -> Path:
    path = directory / "mineworld.env"
    path.write_text(text, encoding="utf-8", newline="\n")
    path.chmod(mode)
    return path


def _configuration(directory: Path, env_file: bool) -> Path:
    path = directory / "cognition.toml"
    secrets_table = '[secrets]\nenv_file = "mineworld.env"\n' if env_file else ""
    path.write_text(
        '[recording]\nmode = "record"\ncassette = "out.jsonl"\n'
        '[budgets]\nledger = "state/budget.sqlite"\n'
        '[tiers]\nsocial = "hosted"\n'
        f"{secrets_table}"
        "[backends.hosted]\n"
        'base_url = "https://api.example.com/v1"\nmodel = "m"\n'
        f'key_env = "{VARIABLE}"\n',
        encoding="utf-8",
        newline="\n",
    )
    return path


@pytest.mark.parametrize("source", ["environment", "env_file"])
def test_a_key_appears_in_no_artefact(
    tmp_path: Path,
    monkeypatch: pytest.MonkeyPatch,
    caplog: pytest.LogCaptureFixture,
    capsys: pytest.CaptureFixture[str],
    source: str,
) -> None:
    key = "MWTEST-" + stdlib_secrets.token_hex(16)
    monkeypatch.delenv(VARIABLE, raising=False)
    if source == "environment":
        monkeypatch.setenv(VARIABLE, key)
    else:
        _key_file(tmp_path, f"{VARIABLE}={key}\n")
    received: list[str] = []
    answers = iter([500, 401, 200])

    def echo(sent: httpx2.Request) -> httpx2.Response:
        # A hostile server: its error bodies repeat every header it was sent, the key among them.
        received.append(sent.headers.get("authorization", ""))
        status = next(answers)
        if status != 200:
            return httpx2.Response(status, content=repr(dict(sent.headers)).encode())
        return httpx2.Response(200, content=ANSWER)

    def factory(config: BackendConfig, secret: Secret | None) -> OpenAICompatibleBackend:
        return OpenAICompatibleBackend(config, secret, transport=httpx2.MockTransport(echo))

    monkeypatch.setitem(registry.FACTORIES, "openai-compatible", factory)
    caplog.set_level(logging.DEBUG)
    config = load(_configuration(tmp_path, env_file=source == "env_file"))
    router = build_router(config)
    gateway = router.for_tier("social")
    assert gateway is not None

    async def session() -> list[str]:
        shown: list[str] = []
        for text in ("one", "two", "three"):
            outcome = await gateway.complete(ALICE, request(text))
            shown += [str(outcome), repr(outcome)]
        shown += [repr(config), str(config), repr(router), repr(gateway)]
        await router.aclose()
        return shown

    shown = run(session())
    print(*shown)  # what a careless caller would print: it must not carry the key either
    captured = capsys.readouterr()
    artefacts = {
        "cassette": (tmp_path / "out.jsonl").read_text(encoding="utf-8"),
        "ledger": (tmp_path / "state" / "budget.sqlite").read_bytes().decode("latin-1"),
        "logs": "\n".join(record.getMessage() for record in caplog.records),
        "stdout": captured.out,
        "stderr": captured.err,
        "shown": "\n".join(shown),
    }
    assert received == [f"Bearer {key}"] * 3  # the key was used
    for name, text in artefacts.items():
        assert key not in text, name
    assert "fine" in artefacts["cassette"]
    assert os.environ.get(VARIABLE) == (key if source == "environment" else None)


def test_an_unset_key_names_the_variable_and_no_value(tmp_path: Path) -> None:
    with pytest.raises(ConfigError) as raised:
        resolve_key("MWTEST_NEVER_SET", _key_file(tmp_path, "OTHER=MWTEST-value\n"), backend="b")
    message = str(raised.value)
    assert "MWTEST_NEVER_SET" in message
    assert str(tmp_path / "mineworld.env") in message
    assert "MWTEST-value" not in message


def test_reading_a_key_file_leaves_the_environment_unchanged(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # AP5-13 (a): dotenv_values, never load_dotenv.
    monkeypatch.delenv(VARIABLE, raising=False)
    before = dict(os.environ)
    secret = resolve_key(
        VARIABLE, _key_file(tmp_path, f"{VARIABLE}=MWTEST-from-file\n"), backend="b"
    )
    assert secret.reveal() == "MWTEST-from-file"
    assert dict(os.environ) == before
    assert repr(secret) == str(secret) == "Secret(<redacted>)"


def test_the_environment_wins_over_the_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    # AP5-13 (b).
    monkeypatch.setenv(VARIABLE, "MWTEST-from-shell")
    path = _key_file(tmp_path, f"{VARIABLE}=MWTEST-from-file\n")
    assert resolve_key(VARIABLE, path, backend="b").reveal() == "MWTEST-from-shell"


def test_configuration_and_keys_inside_a_world_are_refused(tmp_path: Path) -> None:
    # AP5-13 (c): a temporary copy of the layout, a world directory holding world.yaml.
    world = tmp_path / "worlds" / "cafe"
    (world / "config").mkdir(parents=True)
    (world / "world.yaml").write_text("id: cafe\n", encoding="utf-8")
    inside = world / "config" / "cognition.toml"
    inside.write_text('[recording]\nmode = "live"\n', encoding="utf-8")
    with pytest.raises(ConfigError, match="inside the world directory") as raised:
        load(inside)
    assert str(inside) in str(raised.value)
    outside = tmp_path / "cognition.toml"
    outside.write_text(
        '[recording]\nmode = "live"\n[secrets]\nenv_file = "worlds/cafe/config/mineworld.env"\n',
        encoding="utf-8",
    )
    with pytest.raises(ConfigError, match=r"env_file .*mineworld\.env is inside the world"):
        load(outside)


def test_a_key_file_readable_by_others_is_refused_where_modes_apply(tmp_path: Path) -> None:
    # AP5-13 (d).
    loose = _key_file(tmp_path, f"{VARIABLE}=MWTEST-x\n", mode=0o644)
    if sys.platform == "win32":
        # Not applicable on Windows: the check is skipped by platform, visibly, not silently.
        assert permission_check_applies() is False
        assert resolve_key(VARIABLE, loose, backend="b").reveal() == "MWTEST-x"
        return
    assert permission_check_applies() is True
    with pytest.raises(ConfigError, match="chmod 600"):
        resolve_key(VARIABLE, loose, backend="b")
    loose.chmod(0o600)
    assert resolve_key(VARIABLE, loose, backend="b").reveal() == "MWTEST-x"


@pytest.mark.parametrize(
    ("path", "ignored"),
    [
        (".env", True),
        (".env.local", True),
        ("prod.env", True),
        ("secrets.env", True),
        ("cognition/lm-controller/examples/.env.example", False),
        ("cognition/lm-controller/src/mineworld_cognition/secrets.py", False),
    ],
)
def test_key_files_are_ignored_by_git_and_the_examples_are_not(path: str, ignored: bool) -> None:
    # AP5-13 (e).
    result = subprocess.run(
        ["git", "check-ignore", "-q", "--no-index", path], cwd=REPOSITORY, check=False
    )
    assert result.returncode == (0 if ignored else 1), path
