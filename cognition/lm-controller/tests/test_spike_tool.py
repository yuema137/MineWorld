"""C7: the operator-run spike tool, run end to end against a scripted stub (never a model)."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
from pathlib import Path
from types import ModuleType

import pytest
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
