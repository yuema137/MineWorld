"""SIGINT and SIGTERM stop the cognition process promptly, whatever it is waiting on (D-P6-43; pr-s10-p6 §5.16).

The milestone checklist found a model process that ignored Ctrl-C once the server had stopped. The cause it
reproduces: a process started with SIGINT ignored (a background job of a non-interactive shell) never received
`asyncio.run`'s Ctrl-C handling, and SIGTERM ended it with no `leave` and no counters. Here `main` runs in this
process, against a loopback port nobody listens on (the server has stopped, so every seat retries its join), with
a scripted model that is never reached, and a signal is raised from another thread. No model, no network beyond
loopback; `signal.raise_signal` works on all three platforms.
"""

from __future__ import annotations

import asyncio
import signal
import socket
import threading
import time
from collections.abc import Iterator
from pathlib import Path

import pytest

from mineworld_cognition.__main__ import main
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.shutdown import FORCED_STATUS, Stopper

DECISIONS = Path(__file__).parent / "fixtures" / "decisions_format3.json"
STOPPED_WITHIN_S = 10.0
"""The seats' 5 s `leave` bound plus margin for a loaded CI runner."""
CONFIG = """\
server         = "ws://127.0.0.1:{port}/ws"
invite_env     = "MINEWORLD_TEST_INVITE"
nickname       = "cognition"
store          = "store"
decisions_json = "{decisions}"

[recording]
mode = "scripted"

[tiers]
social    = "local"
summarize = "none"

[backends.local]
base_url          = "http://127.0.0.1:11434/v1"
model             = "fake-model"
structured_output = "none"
"""


def _never(request: CompletionRequest) -> Completion | BackendFailure:
    raise AssertionError("no seat joined, so no model request is made")


def _closed_port() -> int:
    """A loopback port that was just free: a stopped server's."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


@pytest.fixture
def stopped_server(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> Path:
    monkeypatch.setenv("MINEWORLD_TEST_INVITE", "mwsdk-marker-invite-7f3a9c2e5b1d")
    config = tmp_path / "cognition.toml"
    config.write_text(
        CONFIG.format(port=_closed_port(), decisions=DECISIONS.as_posix()), encoding="utf-8"
    )
    return config


@pytest.fixture
def sigint_ignored() -> Iterator[None]:
    """SIGINT as a background job of a non-interactive shell inherits it: ignored."""
    previous = signal.getsignal(signal.SIGINT)
    signal.signal(signal.SIGINT, signal.SIG_IGN)
    try:
        yield
    finally:
        signal.signal(signal.SIGINT, previous)


def _raise_later(number: signal.Signals, after_s: float) -> threading.Timer:
    timer = threading.Timer(after_s, signal.raise_signal, (number,))
    timer.daemon = True
    timer.start()
    return timer


@pytest.mark.parametrize("number", [signal.SIGINT, signal.SIGTERM], ids=["SIGINT", "SIGTERM"])
def test_a_stop_signal_ends_a_process_whose_server_has_stopped(
    stopped_server: Path,
    sigint_ignored: None,
    number: signal.Signals,
    capsys: pytest.CaptureFixture[str],
) -> None:
    previous = signal.getsignal(number)
    timer = _raise_later(number, 1.0)
    started = time.monotonic()
    try:
        status = main(["--config", str(stopped_server)], scripted=ScriptedBackend(_never))
    finally:
        timer.cancel()
        signal.signal(number, previous)
    took = time.monotonic() - started
    assert status == 0
    assert took < STOPPED_WITHIN_S, f"stopping took {took:.1f} s"
    out, err = capsys.readouterr()
    assert f"{number.name}: stopping" in err
    assert "[cognition] alice: no consults recorded" in out  # the status line is still printed


def test_a_stop_that_hangs_is_forced_after_the_bound() -> None:
    """A serving task that will not stop: the bound forces the exit; the force here only records it."""
    forced: list[int] = []

    async def scenario() -> None:
        loop = asyncio.get_running_loop()
        released = asyncio.Event()

        def force(status: int) -> None:
            forced.append(status)
            loop.call_soon_threadsafe(released.set)

        async def stubborn() -> None:
            try:
                await asyncio.sleep(3600)
            except asyncio.CancelledError:
                await released.wait()  # a cleanup that hangs until the forced exit

        stopper = Stopper(force_after_s=0.5, force=force)
        loop.call_later(0.1, signal.raise_signal, signal.SIGTERM)
        await stopper.serve(stubborn())
        stopper.disarm()
        assert stopper.received == signal.SIGTERM

    previous = signal.getsignal(signal.SIGTERM)
    try:
        asyncio.run(asyncio.wait_for(scenario(), timeout=STOPPED_WITHIN_S))
    finally:
        signal.signal(signal.SIGTERM, previous)
    assert forced == [FORCED_STATUS]
