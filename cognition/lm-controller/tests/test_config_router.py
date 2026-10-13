"""The configuration and the router: AP5-1 (b), AP5-3, AP5-8, AP5-14 (a)-(e), and §5.4's refusals."""

from __future__ import annotations

import re
import socket
import threading
from pathlib import Path
from urllib.parse import urlsplit

import pytest
from mineworld_sdk.wire.ids import EntityKey
from support import GOLDEN_KEY, completion, golden_request, request, run

from mineworld_cognition.backend import registry
from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.backend.providers import PRESETS
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import (
    BackendConfig,
    CognitionConfig,
    ConfigError,
    build_router,
    load,
)
from mineworld_cognition.gateway import Completed, Router
from mineworld_cognition.record import Cassette, CassetteMiss, RecordingBackend

ALICE = EntityKey("alice")
EXAMPLES = Path(__file__).resolve().parent.parent / "examples"
KEY_LIKE = re.compile(r"sk-|xai-|gsk_|AIza|[A-Za-z0-9+/=_-]{20,}")


def _write(path: Path, text: str) -> Path:
    path.write_text(text, encoding="utf-8", newline="\n")
    return path


def _replay_config(directory: Path, cassette: str, backend: str, base_url: str, model: str) -> Path:
    return _write(
        directory / "cognition.toml",
        f"""
[recording]
mode = "replay"
cassette = "{cassette}"

[tiers]
social = "{backend}"
summarize = "none"

[backends.{backend}]
kind = "openai-compatible"
base_url = "{base_url}"
model = "{model}"
""",
    )


def _record_scripted(path: Path, calls: list[CompletionRequest], binding: str) -> None:
    recorder = RecordingBackend(
        ScriptedBackend(lambda call: completion("answer to " + call.messages[-1].text)),
        path,
        binding=binding,
        model="model-a",
    )

    async def session() -> None:
        for call in calls:
            await recorder.complete(call)
        await recorder.aclose()

    run(session())


def _texts(router: Router, calls: list[CompletionRequest]) -> list[str]:
    gateway = router.for_tier("social")
    assert gateway is not None

    async def session() -> list[str]:
        texts: list[str] = []
        for call in calls:
            outcome = await gateway.complete(ALICE, call)
            assert isinstance(outcome, Completed)
            texts.append(outcome.completion.text)
        await router.aclose()
        return texts

    return run(session())


def test_one_cassette_replays_under_a_renamed_and_repointed_binding(tmp_path: Path) -> None:
    # AP5-8, the seam half of AC-4; with AP5-1 (b): two configurations that differ in binding name,
    # base_url and model replay the same keys.
    calls = [golden_request(), request("second"), request("second")]
    _record_scripted(tmp_path / "seam.jsonl", calls, "local")
    keys = [entry.key for entry in Cassette.load(tmp_path / "seam.jsonl").entries]
    first = tmp_path / "first"
    second = tmp_path / "second"
    first.mkdir()
    second.mkdir()
    a = load(
        _replay_config(first, "../seam.jsonl", "local", "http://127.0.0.1:11434/v1", "model-a")
    )
    b = load(
        _replay_config(second, "../seam.jsonl", "elsewhere", "http://[::1]:8080/v1", "model-b")
    )
    assert _texts(build_router(a), calls) == _texts(build_router(b), calls)
    assert keys == [cassette_key(call) for call in calls]
    assert keys[0] == GOLDEN_KEY


class CountingListener:
    """A loopback socket the test owns, counting the connections it accepts (AP5-3 (ii))."""

    def __init__(self) -> None:
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        self.socket.bind(("127.0.0.1", 0))
        self.socket.listen()
        self.socket.settimeout(0.2)
        self.port: int = self.socket.getsockname()[1]
        self.accepted = 0
        self._stop = threading.Event()
        self._thread = threading.Thread(target=self._run, daemon=True)
        self._thread.start()

    def _run(self) -> None:
        while not self._stop.is_set():
            try:
                connection, _ = self.socket.accept()
            except TimeoutError:
                continue
            except OSError:
                return
            self.accepted += 1
            connection.close()

    def close(self) -> None:
        self._stop.set()
        self._thread.join()
        self.socket.close()


def test_a_replay_miss_raises_and_never_reaches_the_configured_server(tmp_path: Path) -> None:
    # AP5-3: the backend `local` points at a listening loopback port; pytest-socket would allow it.
    _record_scripted(tmp_path / "one.jsonl", [request("recorded")], "local")
    listener = CountingListener()
    try:
        config = load(
            _replay_config(
                tmp_path, "one.jsonl", "local", f"http://127.0.0.1:{listener.port}/v1", "m"
            )
        )
        router = build_router(config)
        gateway = router.for_tier("social")
        assert gateway is not None
        asked = request("never recorded")

        async def session() -> CassetteMiss:
            with pytest.raises(CassetteMiss) as raised:
                await gateway.complete(ALICE, asked)
            return raised.value

        miss = run(session())
    finally:
        listener.close()
    assert miss.kind == "absent"
    message = str(miss)
    assert cassette_key(asked) in message
    assert "$.messages[0].text" in message
    assert "recorded" in message
    assert listener.accepted == 0
    window = router._budget.ledger.window(ALICE, 2**62, 2**62)  # pyright: ignore[reportPrivateUsage]
    assert (window.calls, window.tokens) == (0, 0)


def _backend(**fields: object) -> BackendConfig:
    return BackendConfig.model_validate(fields)


def test_every_preset_row_is_https_and_holds_no_key() -> None:
    # AP5-14 (a), (b).
    for name, preset in PRESETS.items():
        assert preset.name == name
        if preset.base_url is not None:
            parts = urlsplit(preset.base_url)
            assert parts.scheme == "https", name
            assert parts.username is None and parts.password is None, name
            assert not parts.query and not parts.fragment, name
        assert re.fullmatch(r"[A-Z_][A-Z0-9_]*", preset.key_env), name
        for value in (preset.base_url or "", preset.key_env):
            stripped = value.removeprefix("https://")
            assert not KEY_LIKE.search(stripped.replace("/", " ").replace(".", " ")), (name, value)


@pytest.mark.parametrize(
    "table",
    [
        'preset = "openai"\nmodel = "m"',
        'preset = "xai"\nmodel = "m"',
        'kind = "openai-compatible"\nbase_url = "https://api.example.com/v1"\nmodel = "m"\n'
        'key_env = "MWTEST_HAND_KEY"',
    ],
)
def test_a_preset_reaches_no_key_and_no_metadata(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, table: str
) -> None:
    # AP5-14 (c): the golden request recorded through config → router → gateway → recorder, with the
    # adapter's factory swapped for a scripted backend (no network); the key is AP5-1's literal under
    # every configuration, and the metadata names the user's binding, never the preset.
    seen: list[BackendConfig] = []

    def factory(config: BackendConfig, key: object) -> ScriptedBackend:
        seen.append(config)
        assert key is not None
        return ScriptedBackend(lambda _: completion("ok"))

    monkeypatch.setitem(registry.FACTORIES, "openai-compatible", factory)
    for name in ("OPENAI_API_KEY", "XAI_API_KEY", "MWTEST_HAND_KEY"):
        monkeypatch.setenv(name, "MWTEST-not-a-real-key")
    path = _write(
        tmp_path / "cognition.toml",
        '[recording]\nmode = "record"\ncassette = "out.jsonl"\n[tiers]\nsocial = "mine"\n'
        f"[backends.mine]\n{table}\n",
    )
    router = build_router(load(path))
    gateway = router.for_tier("social")
    assert gateway is not None

    async def session() -> None:
        assert isinstance(await gateway.complete(ALICE, golden_request()), Completed)
        await router.aclose()

    run(session())
    entry = Cassette.load(tmp_path / "out.jsonl").entries[0]
    assert entry.key == GOLDEN_KEY
    assert entry.meta.binding == "mine"
    assert entry.meta.model == "m"
    assert seen[0].preset in ("openai", "xai", None)


def test_a_field_the_user_sets_wins_over_the_preset() -> None:
    # AP5-14 (d).
    assert _backend(preset="openai", model="m").base_url == "https://api.openai.com/v1"
    overridden = _backend(preset="deepseek", model="m", structured_output="none")
    assert overridden.structured_output == "none"
    assert _backend(preset="deepseek", model="m").structured_output == "json_object"
    assert _backend(preset="moonshot", model="m").temperature == "omit"


def test_dashscope_needs_a_base_url_and_names_the_field() -> None:
    # AP5-14 (e).
    with pytest.raises(ValueError, match="base_url"):
        _backend(preset="dashscope", model="m")
    region = "https://ws1.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1"
    assert _backend(preset="dashscope", model="m", base_url=region).key_env == "DASHSCOPE_API_KEY"


@pytest.mark.parametrize(
    ("body", "names"),
    [
        ('[recording]\nmode = "live"\ncolour = "red"\n', "recording.colour"),
        ('[recording]\nmode = "live"\n[tiers]\nsocial = "ghost"\n', "tiers.social"),
        ('[recording]\nmode = "replay"\n', "recording.cassette"),
        ('[recording]\nmode = "live"\n[tiers]\nroutine = "none"\n', "tiers.routine"),
        (
            '[recording]\nmode = "live"\n[backends.x]\nbase_url = "http://127.0.0.1/v1"\n'
            'model = "m"\nkey_env = "sk-PASTED-not-a-name"\n',
            "backends.x.key_env",
        ),
    ],
)
def test_a_bad_configuration_names_its_table_and_key(tmp_path: Path, body: str, names: str) -> None:
    with pytest.raises(ConfigError, match=re.escape(names)) as raised:
        load(_write(tmp_path / "cognition.toml", body))
    assert "sk-PASTED" not in str(raised.value)  # a pasted key is never echoed back


def test_the_hosted_example_is_a_valid_configuration() -> None:
    config = load(EXAMPLES / "hosted.toml.example")
    assert isinstance(config, CognitionConfig)
    assert config.backends["deepseek"].key_env == "DEEPSEEK_API_KEY"
    assert config.backends["local"].key_env == ""
