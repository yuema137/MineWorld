"""SCRATCH plugin: dumps every asyncio task's stack at fixed wall times into the test."""
import asyncio
import sys
import threading
import time
import traceback

DUMPS = (25.0, 45.0, 100.0, 180.0, 240.0)


def _dump(label: str) -> None:
    out = [f"===== WATCHDOG {label} t={time.monotonic():.1f}"]
    try:
        tasks = list(asyncio.tasks._scheduled_tasks)  # type: ignore[attr-defined]
    except Exception as error:  # noqa: BLE001
        tasks = []
        out.append(f"no tasks: {error!r}")
    for task in tasks:
        try:
            out.append(f"--- {task!r}"[:400])
            for frame in task.get_stack(limit=12):
                out.append(f"    {frame.f_code.co_filename}:{frame.f_lineno} {frame.f_code.co_name}")
        except Exception as error:  # noqa: BLE001
            out.append(f"    ? {error!r}")
    for ident, frame in sys._current_frames().items():
        out.append(f"--- thread {ident}")
        out.extend("    " + line.rstrip() for line in traceback.format_stack(frame, limit=8))
    print("\n".join(out), file=sys.stderr, flush=True)


def pytest_runtest_call(item):  # noqa: ANN001, ANN201
    stop = threading.Event()
    began = time.monotonic()

    def watch() -> None:
        for at in DUMPS:
            if stop.wait(max(0.0, began + at - time.monotonic())):
                return
            _dump(f"{at:.0f}s")

    thread = threading.Thread(target=watch, daemon=True)
    thread.start()
    item.config._scratch_stop = stop  # noqa: SLF001


def pytest_runtest_teardown(item):  # noqa: ANN001, ANN201
    stop = getattr(item.config, "_scratch_stop", None)
    if stop is not None:
        stop.set()
    print(f"===== TEARDOWN t={time.monotonic():.1f}", file=sys.stderr, flush=True)
