"""C7: the operator-run spike tool, run end to end against a scripted stub (never a model)."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path
from types import ModuleType

import pytest
from mineworld_sdk.wire.ids import JsonValue
from support import completion

from mineworld_cognition.backend.model import CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.config import BackendConfig, load

TOOLS = Path(__file__).resolve().parent.parent / "tools"
REPOSITORY = TOOLS.parents[2]


def _tool() -> ModuleType:
    spec = importlib.util.spec_from_file_location("model_spike", TOOLS / "model_spike.py")
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    sys.modules["model_spike"] = module
    spec.loader.exec_module(module)
    return module


def _config(directory: Path, base_url: str) -> Path:
    path = directory / "cognition.toml"
    path.write_text(
        '[recording]\nmode = "live"\n[tiers]\nsocial = "local"\n'
        f'[backends.local]\nbase_url = "{base_url}"\nmodel = "placeholder"\nreasoning = "off"\n',
        encoding="utf-8",
    )
    return path


def test_the_thresholds_are_ap5_s_literals_and_the_scenarios_are_fixed() -> None:
    spike = _tool()
    assert (spike.SCENARIO_COUNT, spike.FIRST_ATTEMPT_VALID_MIN, spike.P95_LATENCY_MAX_MS) == (
        40,
        38,
        15_000,
    )
    assert len(spike.scenarios()) == 40


def test_the_spike_runs_end_to_end_against_a_scripted_stub(tmp_path: Path) -> None:
    spike = _tool()
    seen: list[str] = []

    def factory(backend: BackendConfig) -> ScriptedBackend:
        seen.append(backend.model)
        good = '{"act": "say", "to": "Bob", "words": "Morning!", "recalls": []}'
        bad = "Sure! Here is my answer."

        def answer(request: CompletionRequest) -> object:
            # The small model fails the schema on three scenarios; the larger one on none.
            failing = backend.model == "small" and request.messages[1].text.startswith(
                ("Bob:", "Carol:")
            )
            return completion(bad if failing else good)

        return ScriptedBackend(answer)  # pyright: ignore[reportArgumentType]

    config = load(_config(tmp_path, "http://127.0.0.1:11434/v1"))
    report = spike.run_spike(config, ["small", "large"], tmp_path / "out", factory=factory)
    assert seen == ["small", "large"]
    written = json.loads((tmp_path / "out" / "report.json").read_text(encoding="utf-8"))
    assert written == report
    small, large = report["models"]
    for result in (small, large):
        assert set(result) >= {
            "first_attempt_schema_valid",
            "latency_p50_ms",
            "latency_p95_ms",
            "reasoning_setting",
            "reasoning_accepted",
            "meets_criteria",
        }
        assert result["completed"] == 40
        assert result["reasoning_setting"] == "off"
    assert small["first_attempt_schema_valid"] < 38
    assert small["meets_criteria"] is False
    assert large["first_attempt_schema_valid"] == 40
    assert report["default"] == "large"
    assert set(report["machine"]) >= {"os", "cpus", "memory_bytes"}
    assert (tmp_path / "out" / "spike-small.jsonl").exists()


def test_a_remote_server_is_refused_without_allow_remote(tmp_path: Path) -> None:
    spike = _tool()
    config = load(_config(tmp_path, "https://api.example.com/v1"))
    with pytest.raises(SystemExit, match="allow-remote"):
        spike.run_spike(config, ["m"], tmp_path / "out")


FIXTURE: dict[str, JsonValue] = {
    "criteria": {
        "first_attempt_schema_valid_min": 38,
        "latency_p95_max_ms": 15000,
        "scenarios": 40,
        "scenarios_sha256": "0123456789abcdef" * 4,
    },
    "server": {"backend": "local", "structured_output": "none"},
    "models": [
        {
            "model": "qwen3.5:4b",
            "scenarios": 40,
            "first_attempt_schema_valid": 39,
            "completed": 40,
            "failures": {},
            "latency_p50_ms": 2600,
            "latency_p95_ms": 4800,
            "meets_criteria": True,
        },
        {
            "model": "gpt-5-nano",
            "scenarios": 40,
            "first_attempt_schema_valid": 0,
            "completed": 0,
            "failures": {"http_status:400": 40},
            "latency_p50_ms": None,
            "latency_p95_ms": None,
            "meets_criteria": False,
        },
    ],
    "default": "qwen3.5:4b",
    "verdict": "default chosen",
}
"""A two-model report, written by hand: one admitted, one refused by the runtime (C14)."""

GOLDEN = (
    "model                  output       valid done failures             p50 ms p95 ms meets\n"
    "qwen3.5:4b             none         39/40   40 -                      2600   4800 yes\n"
    "gpt-5-nano             none          0/40    0 http_status:400 x 40      -      - no\n"
    "default: qwen3.5:4b   criteria: valid >= 38/40, p95 <= 15000 ms   scenarios sha256 0123456789ab\n"
)
"""The table of `FIXTURE`, by column: each row's cells start at the columns of the heading (C14, §5.15)."""


def test_the_summary_table_is_the_hand_written_golden() -> None:
    spike = _tool()
    assert spike.summary_table(FIXTURE) == GOLDEN


def test_main_prints_the_table_before_the_unchanged_json_line(
    tmp_path: Path, capsys: pytest.CaptureFixture[str], monkeypatch: pytest.MonkeyPatch
) -> None:
    spike = _tool()

    def fake_run(*args: object, **kwargs: object) -> dict[str, JsonValue]:
        return FIXTURE

    monkeypatch.setattr(spike, "run_spike", fake_run)
    config = _config(tmp_path, "http://127.0.0.1:11434/v1")
    assert (
        spike.main(["--config", str(config), "--models", "qwen3.5:4b", "--out", str(tmp_path)]) == 0
    )
    lines = capsys.readouterr().out.splitlines()
    assert "\n".join(lines[:4]) + "\n" == GOLDEN
    assert lines[-1] == json.dumps({"default": "qwen3.5:4b", "verdict": "default chosen"})


def test_no_ci_command_runs_the_spike_or_allows_a_remote_server() -> None:
    for layer in ("fast", "core", "parity", "platforms", "python", "python-smoke"):
        listed = subprocess.run(
            [sys.executable, "scripts/ci_layer.py", "--list", layer],
            cwd=REPOSITORY,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        assert "model_spike" not in listed and "--allow-remote" not in listed, layer
