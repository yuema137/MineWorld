"""The command line (pr-s27d §5.8, AC-d-11, M-d-7): the golden tool list, the credential rules, and the bounds.

The credential is read from the environment only, and no argument names it (step-27 §6.3). A refusal exits 2,
names a variable or a field, and never prints a value; stdout stays empty when the command refuses."""

from __future__ import annotations

from pathlib import Path

import pytest

from mineworld_agent.__main__ import GRANT_VARIABLE, INVITE_VARIABLE, main

GOLDEN = Path(__file__).resolve().parent / "golden" / "tools.json"
SERVER = "ws://127.0.0.1:7878/ws"
SECRET = "mwagent-grant-secret-3b9e"


def test_schema_prints_exactly_the_golden_tool_list(capsys: pytest.CaptureFixture[str]) -> None:
    assert main(["schema"]) == 0
    assert capsys.readouterr().out == GOLDEN.read_text(encoding="utf-8")


def test_schema_writes_the_golden_tool_list_to_a_file(tmp_path: Path) -> None:
    target = tmp_path / "tools.json"
    assert main(["schema", "--out", str(target)]) == 0
    assert target.read_bytes() == GOLDEN.read_bytes()


def test_both_credential_variables_are_refused(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.setenv(GRANT_VARIABLE, SECRET)
    monkeypatch.setenv(INVITE_VARIABLE, SECRET)
    assert main(["mcp", "--server", SERVER]) == 2
    captured = capsys.readouterr()
    assert "set one of MINEWORLD_GRANT and MINEWORLD_INVITE, not both" in captured.err
    assert SECRET not in captured.err
    assert captured.out == ""


def test_no_credential_variable_is_refused_by_name(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str]
) -> None:
    monkeypatch.delenv(GRANT_VARIABLE, raising=False)
    monkeypatch.delenv(INVITE_VARIABLE, raising=False)
    assert main(["mcp", "--server", SERVER]) == 2
    captured = capsys.readouterr()
    assert GRANT_VARIABLE in captured.err
    assert INVITE_VARIABLE in captured.err
    assert captured.out == ""


def test_the_credential_is_never_an_argument(capsys: pytest.CaptureFixture[str]) -> None:
    """M-d-7: a `--invite` or `--grant` option would put the secret on the command line."""
    with pytest.raises(SystemExit) as exit_request:
        main(["mcp", "--help"])
    assert exit_request.value.code == 0
    shown = capsys.readouterr().out
    assert "--invite" not in shown
    assert "--grant" not in shown


@pytest.mark.parametrize(
    ("flag", "value", "field"),
    [
        ("--max-result-bytes", "9000", "max_result_bytes"),
        ("--inbox", "513", "inbox"),
        ("--idle-release", "-1", "idle_release_s"),
        ("--server", "http://127.0.0.1:7878/ws", "server"),
    ],
)
def test_an_out_of_range_bound_is_refused_by_field_name(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
    flag: str,
    value: str,
    field: str,
) -> None:
    monkeypatch.setenv(GRANT_VARIABLE, SECRET)
    monkeypatch.delenv(INVITE_VARIABLE, raising=False)
    assert main(["mcp", "--server", SERVER, flag, value]) == 2
    captured = capsys.readouterr()
    assert field in captured.err
    assert value not in captured.err
    assert captured.out == ""
