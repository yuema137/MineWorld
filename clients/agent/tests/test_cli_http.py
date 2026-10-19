"""The `--http` command line (pr-s27e §5.7, F-S27e-7, C4): every refusal exits 2 before anything is bound, names a
flag or a variable, and never prints a value. The serving path itself is `test_http_endpoint.py`'s."""

from __future__ import annotations

import pytest

from mineworld_agent.__main__ import GRANT_VARIABLE, INVITE_VARIABLE, main

SERVER = "ws://127.0.0.1:7878/ws"
SECRET = "mwagent-grant-secret-3b9e"


@pytest.fixture(autouse=True)
def _no_credential(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.delenv(GRANT_VARIABLE, raising=False)
    monkeypatch.delenv(INVITE_VARIABLE, raising=False)


def _refused(capsys: pytest.CaptureFixture[str], *flags: str) -> str:
    assert main(["mcp", "--server", SERVER, *flags]) == 2
    captured = capsys.readouterr()
    assert captured.out == ""
    return captured.err


@pytest.mark.parametrize("variable", [GRANT_VARIABLE, INVITE_VARIABLE])
def test_http_mode_refuses_an_environment_credential(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], variable: str
) -> None:
    monkeypatch.setenv(variable, SECRET)
    err = _refused(capsys, "--http", "127.0.0.1:0")
    assert "unset MINEWORLD_GRANT and MINEWORLD_INVITE" in err
    assert SECRET not in err


@pytest.mark.parametrize("address", ["0.0.0.0:0", "192.0.2.10:8080", "[::]:0", "example.org:80"])
def test_a_non_loopback_address_needs_allow_remote(
    capsys: pytest.CaptureFixture[str], address: str
) -> None:
    err = _refused(capsys, "--http", address)
    assert "is not a loopback address; add --allow-remote" in err
    assert "TLS" in err
    assert address not in err


def test_allow_host_needs_allow_remote(capsys: pytest.CaptureFixture[str]) -> None:
    err = _refused(capsys, "--http", "127.0.0.1:0", "--allow-host", "agents.example")
    assert "--allow-host needs --allow-remote" in err
    assert "agents.example" not in err


@pytest.mark.parametrize(
    "flags",
    [
        ("--allow-remote",),
        ("--allow-origin", "http://127.0.0.1:5173"),
        ("--allow-host", "agents.example"),
        ("--max-credentials", "4"),
    ],
)
def test_an_http_flag_without_http_is_refused_by_name(
    monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str], flags: tuple[str, ...]
) -> None:
    monkeypatch.setenv(GRANT_VARIABLE, SECRET)
    err = _refused(capsys, *flags)
    assert f"{flags[0]} needs --http" in err
    assert SECRET not in err


@pytest.mark.parametrize(
    ("flags", "named"),
    [
        (("--http", "127.0.0.1"), "--http"),
        (("--http", "127.0.0.1:70000"), "--http"),
        (
            ("--http", "127.0.0.1:0", "--allow-origin", "http://127.0.0.1:5173/app"),
            "--allow-origin",
        ),
        (("--http", "127.0.0.1:0", "--allow-origin", "127.0.0.1:5173"), "--allow-origin"),
        (("--http", "127.0.0.1:0", "--max-credentials", "0"), "--max-credentials"),
        (("--http", "127.0.0.1:0", "--max-credentials", "65"), "--max-credentials"),
        (("--http", "0.0.0.0:0", "--allow-remote", "--allow-host", "a b"), "--allow-host"),
    ],
)
def test_a_malformed_http_value_is_refused_by_flag(
    capsys: pytest.CaptureFixture[str], flags: tuple[str, ...], named: str
) -> None:
    err = _refused(capsys, *flags)
    assert f"invalid argument: {named}" in err
    assert flags[-1] not in err


def test_a_bad_bound_is_refused_by_field_before_binding(capsys: pytest.CaptureFixture[str]) -> None:
    err = _refused(capsys, "--http", "127.0.0.1:0", "--inbox", "513")
    assert "inbox" in err
    assert "listening" not in err


def test_help_names_no_credential_option(capsys: pytest.CaptureFixture[str]) -> None:
    with pytest.raises(SystemExit) as exit_request:
        main(["mcp", "--help"])
    assert exit_request.value.code == 0
    shown = capsys.readouterr().out
    assert "--http" in shown
    for option in ("--invite", "--grant", "--bearer", "--token"):
        assert option not in shown
