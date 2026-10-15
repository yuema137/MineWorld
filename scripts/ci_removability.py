#!/usr/bin/env python3
"""IC-7: the LM controller is removable by deletion (S10 P7; `step-17-cognition.md` §3.16.1, §10 IC-7).

    python3 scripts/ci_removability.py [--keep] [--python-only]

Copies every tracked file of this checkout into a scratch directory, except `cognition/lm-controller/`, removes
that directory's one line from the root `pyproject.toml`'s uv workspace members (its registration, the Python twin
of a pack's `installed!` line; `pr-s10-p7-milestone-d.md` D-P7-3), and in the copy runs:

    uv sync                                   the workspace without the member, re-resolved
    uv run pytest sdk/python                  the SDK's tests
    uv run pytest clients/agent               the agent player library's tests
    cargo test --workspace                    every Rust test, Milestone D's unplugged run (step 8) among them

`--python-only` stops before the Rust commands (a laptop's check under R-LOAD-1; CI runs everything). Any other
edit the copy would need to pass is IC-7 failing. The scratch directory is removed afterwards unless
`--keep` is given. The mutation that must fail it: an SDK module importing `mineworld_cognition`.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REMOVED = "cognition/lm-controller"
MEMBER = f'"{REMOVED}", '
"""How the root pyproject.toml names the member, inside `members = [...]`."""
COMMANDS: list[list[str]] = [
    ["uv", "sync"],
    # The SDK's and the agent library's real_server tests start this binary (`MINEWORLD_BIN`).
    ["cargo", "build", "-p", "mineworld-cli"],
    ["uv", "run", "pytest", "sdk/python"],
    ["uv", "run", "pytest", "clients/agent"],
    ["cargo", "test", "--workspace"],
]


def tracked() -> list[str]:
    listing = subprocess.run(
        ["git", "ls-files", "-z"], cwd=ROOT, check=True, capture_output=True
    ).stdout.decode("utf-8")
    return [path for path in listing.split("\0") if path]


def copy_without(scratch: Path) -> int:
    copied = 0
    for path in tracked():
        if path == REMOVED or path.startswith(REMOVED + "/"):
            continue
        source = ROOT / path
        if not source.is_file():  # deleted in the working tree
            continue
        target = scratch / path
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)
        copied += 1
    return copied


def unregister(scratch: Path) -> None:
    pyproject = scratch / "pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    members = [line for line in text.splitlines() if line.startswith("members = [")]
    if len(members) != 1 or members[0].count(MEMBER) != 1:
        raise SystemExit(f"[removability] pyproject.toml's members line does not name {REMOVED} once: {members}")
    pyproject.write_text(text.replace(members[0], members[0].replace(MEMBER, "")), encoding="utf-8")
    (scratch / "uv.lock").unlink()  # re-resolved by `uv sync` without the member


def main(arguments: list[str]) -> int:
    known = {"--keep", "--python-only"}
    if not set(arguments) <= known:
        print(__doc__)
        return 2
    keep = "--keep" in arguments
    commands = [c for c in COMMANDS if c[0] != "cargo"] if "--python-only" in arguments else COMMANDS
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-removability-"))
    environment = {key: value for key, value in os.environ.items() if key != "VIRTUAL_ENV"}
    # The checkout's own target directory, so the registry dependencies it already compiled are reused.
    target = Path(environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target" / "removability")))
    suffix = ".exe" if os.name == "nt" else ""
    environment["MINEWORLD_BIN"] = str(target / "debug" / f"mineworld{suffix}")
    try:
        print(f"[removability] {copy_without(scratch)} files copied to {scratch}, {REMOVED}/ left out")
        unregister(scratch)
        print(f"[removability] {REMOVED} removed from the uv workspace members")
        assert not (scratch / REMOVED).exists()
        for command in commands:
            started = time.monotonic()
            print(f"[removability] $ {' '.join(command)}", flush=True)
            status = subprocess.run(command, cwd=scratch, env=environment, check=False).returncode
            print(f"[removability]   exit {status} in {time.monotonic() - started:.1f} s", flush=True)
            if status != 0:
                print(f"[removability] FAIL: without {REMOVED}, `{' '.join(command)}` failed")
                return status
        print(f"[removability] PASS: everything outside {REMOVED} builds and passes without it")
        return 0
    finally:
        if keep:
            print(f"[removability] kept {scratch}")
        else:
            shutil.rmtree(scratch, ignore_errors=True)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
