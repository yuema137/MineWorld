"""Stopping the cognition process on SIGINT and SIGTERM, promptly, on every platform (pr-s10-p6 §5.16, D-P6-43).

`asyncio.run` turns Ctrl-C into a cancellation of its main task only while SIGINT has Python's default handler.
A process started with SIGINT ignored (a background job of a non-interactive shell, or a parent that ignores
it) then never stops on Ctrl-C, and SIGTERM ends it with no `leave`, no counters and no closed cassette. So a
`Stopper` installs its own handlers while serving:

- The first SIGINT or SIGTERM (and SIGBREAK, Ctrl-Break, on Windows) cancels the serving task. The cancellation
  reaches every seat runner, its reconnect loop and its in-flight model requests; each seat leaves within
  `STOP_LEAVE_S`, and the router is closed.
- If the process has not stopped `force_after_s` seconds later, or a second signal arrives, it exits at once
  with status 130 (`os._exit`). This is the bound a stuck cleanup cannot outlast. `disarm` ends the bound once
  the caller has finished stopping.

POSIX uses `loop.add_signal_handler`, which also wakes the selector loop. Windows has no `add_signal_handler`, so
`signal.signal` is used there; the proactor loop is woken by the wakeup descriptor it installs itself. Handlers
are installed only on the main thread, and removed when serving ends.
"""

from __future__ import annotations

import asyncio
import os
import signal
import sys
import threading
from collections.abc import Callable, Coroutine
from types import FrameType
from typing import Any

FORCE_AFTER_S = 15.0
"""How long a stop may take before the process exits regardless: the seats' 5 s `leave` bound, plus margin."""
FORCED_STATUS = 130
"""The exit status of a forced stop: 128 + SIGINT, as shells report an interrupted command."""

type Handler = Callable[[int, FrameType | None], Any] | int | signal.Handlers | None


def stop_signals() -> tuple[signal.Signals, ...]:
    """The signals that stop the process on this platform."""
    if sys.platform == "win32":
        return (signal.SIGINT, signal.SIGTERM, signal.SIGBREAK)
    return (signal.SIGINT, signal.SIGTERM)


def _exit_now(status: int) -> None:
    os._exit(status)  # pyright: ignore[reportPrivateUsage]


class Stopper:
    """Serves a coroutine until it ends or a stop signal cancels it; forces an exit if stopping hangs."""

    def __init__(
        self,
        *,
        force_after_s: float = FORCE_AFTER_S,
        force: Callable[[int], None] = _exit_now,
    ) -> None:
        self._force_after_s = force_after_s
        self._force = force
        self._timer: threading.Timer | None = None
        self.received: signal.Signals | None = None
        """The first stop signal received, or `None`."""

    async def serve(self, serving: Coroutine[Any, Any, None]) -> None:
        """Awaits `serving` in its own task. A stop signal cancels it and returns normally; anything else it
        raised is raised here."""
        loop = asyncio.get_running_loop()
        task = loop.create_task(serving)
        installed = self._install(loop, task)
        try:
            await task
        except asyncio.CancelledError:
            if self.received is None:
                raise
        finally:
            self._uninstall(loop, installed)

    def disarm(self) -> None:
        """Ends the forced-exit bound: the caller has finished stopping."""
        if self._timer is not None:
            self._timer.cancel()
            self._timer = None

    def _install(
        self, loop: asyncio.AbstractEventLoop, task: asyncio.Task[None]
    ) -> dict[signal.Signals, Handler | None]:
        """The handlers replaced: `None` for one installed on the loop, else the previous handler."""
        installed: dict[signal.Signals, Handler | None] = {}
        if threading.current_thread() is not threading.main_thread():
            return installed
        for number in stop_signals():
            if sys.platform != "win32":
                try:
                    loop.add_signal_handler(number, self._received, number, loop, task)
                    installed[number] = None
                    continue
                except (NotImplementedError, RuntimeError):
                    pass
            installed[number] = signal.getsignal(number)

            def handler(signum: int, frame: FrameType | None) -> None:
                self._received(signal.Signals(signum), loop, task)

            signal.signal(number, handler)
        return installed

    @staticmethod
    def _uninstall(
        loop: asyncio.AbstractEventLoop, installed: dict[signal.Signals, Handler | None]
    ) -> None:
        for number, previous in installed.items():
            if previous is None:
                loop.remove_signal_handler(number)
            else:
                signal.signal(number, previous)

    def _received(
        self, number: signal.Signals, loop: asyncio.AbstractEventLoop, task: asyncio.Task[None]
    ) -> None:
        if self.received is not None:
            self._forced(f"{number.name} again")
            return
        self.received = number
        print(f"[cognition] {number.name}: stopping", file=sys.stderr, flush=True)
        timer = threading.Timer(
            self._force_after_s,
            self._forced,
            (f"still stopping after {self._force_after_s:g} s",),
        )
        timer.daemon = True
        timer.start()
        self._timer = timer
        loop.call_soon_threadsafe(task.cancel)

    def _forced(self, why: str) -> None:
        print(f"[cognition] {why}: exiting now", file=sys.stderr, flush=True)
        self._force(FORCED_STATUS)
