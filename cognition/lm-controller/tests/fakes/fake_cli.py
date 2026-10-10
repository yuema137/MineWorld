"""A fake model CLI for the bridge's tests. No real CLI ever runs in a test (pr-s10-p5b §9 NEVER).

Run as `[python, fake_cli.py, --record FILE, --mode MODE, ...]`. Every start appends one JSON line to
FILE: its argv, its environment, its working directory and that directory's entries, its stdin (hex)
and its pid, so a test can count spawns and see exactly what the bridge passed. Modes:

- `answer`: prints `{"text", "usage"}` on stdout;
- `answer-no-usage`: prints `{"text"}` only;
- `fail`: prints `error: not logged in` on stderr and exits 3;
- `hang`: starts a grandchild (this script in `sleep` mode), records both pids, then sleeps 60 s;
- `sleep`: sleeps 60 s.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time


def main() -> int:
    args = sys.argv[1:]
    record = args[args.index("--record") + 1]
    mode = args[args.index("--mode") + 1]
    if mode == "sleep":
        time.sleep(60)
        return 0
    stdin = b"" if mode == "hang" else sys.stdin.buffer.read()
    cwd = os.getcwd()
    entry: dict[str, object] = {
        "argv": sys.argv,
        "env": dict(os.environ),
        "cwd": cwd,
        "cwd_entries": sorted(os.listdir(cwd)),
        "stdin_hex": stdin.hex(),
        "pid": os.getpid(),
    }
    if mode == "hang":
        grandchild = subprocess.Popen(
            [sys.executable, os.path.abspath(__file__), "--record", record, "--mode", "sleep"]
        )
        entry["grandchild"] = grandchild.pid
    with open(record, "a", encoding="utf-8", newline="\n") as file:
        file.write(json.dumps(entry) + "\n")
    if mode == "hang":
        time.sleep(60)
        return 0
    if mode == "fail":
        sys.stderr.write("error: not logged in\nmore detail\n")
        return 3
    answer: dict[str, object] = {"text": f"answered {len(stdin)} bytes"}
    if mode == "answer":
        answer["usage"] = {"input_tokens": 11, "output_tokens": 4}
    sys.stdout.write(json.dumps(answer))
    return 0


if __name__ == "__main__":
    sys.exit(main())
