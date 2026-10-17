"""AC-25f-3 and AC-25f-4: the sandbox holds exactly the allowed set, and the runner refuses to start without its
ceilings or under CI; a scripted agent proves the pipeline end to end without a model (PROTOCOL.md §§2, 8)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import pytest

import assemble
import run

REPO = Path(__file__).resolve().parents[3]


def allowed(binary: Path) -> list[str]:
    """The allowed set, computed from the repository independently of assemble.py's walk."""
    files: list[str] = []
    for path in (REPO / "docs" / "guide").rglob("*"):
        if path.is_file() and not path.name.endswith(".zh-CN.md"):
            files.append("guide/" + path.relative_to(REPO / "docs" / "guide").as_posix())
    for path in (REPO / "docs" / "reference").rglob("*"):
        if path.is_file():
            files.append("reference/" + path.relative_to(REPO / "docs" / "reference").as_posix())
    for path in (REPO / "entities" / "modern-goods").rglob("*"):
        if path.is_file():
            files.append(
                "packs/modern-goods/"
                + path.relative_to(REPO / "entities" / "modern-goods").as_posix()
            )
    files.append(f"bin/{binary.name}")
    return sorted(files)


@pytest.mark.real_binary
def test_the_sandbox_holds_exactly_the_allowed_set_and_names_no_repository_path(
    tmp_path: Path, binary: Path
) -> None:
    sandbox = tmp_path / "sandbox"
    files = assemble.assemble(sandbox, binary)
    assert files == allowed(binary)
    assert (sandbox / "work").is_dir() and not any((sandbox / "work").iterdir())
    assert not any(name.endswith(".zh-CN.md") for name in files)
    assert assemble.leaks(sandbox) == []
    with pytest.raises(assemble.Refused, match="exists"):
        assemble.assemble(sandbox, binary)


@pytest.mark.real_binary
def test_a_sandbox_inside_the_repository_is_refused(binary: Path) -> None:
    with pytest.raises(assemble.Refused, match="inside the repository"):
        assemble.assemble(REPO / "target" / "onboarding-sandbox-refused", binary)


def options(tmp_path: Path, **overrides: object) -> argparse.Namespace:
    given: dict[str, object] = {
        "instructions": "ON-1",
        "agent_command": "claude-h2",
        "budget": 5.0,
        "binary": tmp_path / "mineworld",
        "out": tmp_path / "evidence",
        "model": "sonnet",
        "max_turns": 80,
    }
    given.update(overrides)
    return argparse.Namespace(**given)


@pytest.mark.parametrize(
    ("overrides", "reason"),
    [
        ({"budget": None}, "--budget is required"),
        ({"budget": 6.0}, "above the ruled ceiling"),
        ({"agent_command": None}, "--agent-command is required"),
        ({"max_turns": 200}, "--max-turns"),
    ],
)
def test_the_runner_refuses_to_start_without_its_ceilings(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch, overrides: dict[str, object], reason: str
) -> None:
    monkeypatch.delenv("CI", raising=False)
    with pytest.raises(run.Refused, match=reason):
        run.run_round(options(tmp_path, **overrides))
    assert not (tmp_path / "evidence").exists()


def test_the_runner_never_runs_under_ci(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("CI", "true")
    with pytest.raises(run.Refused, match="never runs in CI"):
        run.run_round(options(tmp_path))


def test_the_agent_gets_no_api_key_and_no_surrounding_session(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setenv("ANTHROPIC_API_KEY", "placeholder-not-a-key")
    monkeypatch.setenv("CLAUDECODE", "1")
    environment = run.agent_environment(tmp_path)
    assert "ANTHROPIC_API_KEY" not in environment and "CLAUDECODE" not in environment
    assert environment["PATH"].split(run.os.pathsep)[0] == str(tmp_path / "bin")


@pytest.mark.real_binary
def test_a_scripted_agent_runs_the_whole_pipeline_without_a_model(
    tmp_path: Path, binary: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.delenv("CI", raising=False)
    fake = Path(run.__file__).resolve().parent / "fake_agent.py"
    command = f"{Path(sys.executable).as_posix()} {fake.as_posix()} {{sandbox}}"
    summary = run.run_round(options(tmp_path, agent_command=command, binary=binary))
    assert summary["exit"] == 0, (tmp_path / "evidence" / "agent.stderr.txt").read_text()
    checks = {c["id"]: c["verdict"] for c in summary["worlds"][0]["checks"]}
    assert [checks[c] for c in ("S1", "S2", "S3", "S4")] == ["PASS"] * 4, summary
    assert summary["void"] == [] and summary["stuck"] == [], summary
    written = json.loads((tmp_path / "evidence" / "summary.json").read_text())
    assert written["transcript_sha256"] == summary["transcript_sha256"]
    assert (tmp_path / "evidence" / "work" / "STATUS").read_text().startswith("DONE")
