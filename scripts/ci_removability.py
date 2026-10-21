#!/usr/bin/env python3
"""IC-7: the LM controller is removable by deletion (S10 P7; `step-17-cognition.md` §3.16.1, §10 IC-7); and,
since S14 PR 16e (ARC-347), the 3D client is too (`docs/MVP.md` §9.1, `step-15-demo-3d.md` §9.1).

    python3 scripts/ci_removability.py [--target lm|3d] [--keep] [--python-only]

`--target lm` (the default) is IC-7, below. `--target 3d` checks HEAD out the same way, deletes the 3D client
(`clients/3d-spike/`) and its two launcher scripts (`mineworld-slice`, `mineworld-3d`) there, and in the copy runs:

    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace                    every Rust test
    cargo test -p mineworld-cli --test demo_a -- --ignored --test-threads=1
                                              the 2D client's scripted drive, headless, on two real worlds

Any edit the copy would need to pass is the check failing. The mutation that must fail it: a Rust test, not
`#[ignore]`d, that reads a file under `clients/3d-spike/`.

Checks HEAD out as a detached worktree in a scratch directory, deletes `cognition/lm-controller/` there, removes
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
REMOVED_3D = ["clients/3d-spike", "mineworld-slice", "mineworld-3d"]
"""The 3D client: its Godot project and its two launcher scripts at the repository root (ARC-347)."""
COMMANDS_3D: list[list[str]] = [
    ["cargo", "fmt", "--all", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    ["cargo", "test", "--workspace"],
    ["cargo", "test", "-p", "mineworld-cli", "--test", "demo_a", "--", "--ignored", "--test-threads=1"],
]
"""How the root pyproject.toml names the member, inside `members = [...]`."""
COMMANDS: list[list[str]] = [
    ["uv", "sync"],
    # The SDK's and the agent library's real_server tests start this binary (`MINEWORLD_BIN`).
    ["cargo", "build", "-p", "mineworld-cli"],
    ["uv", "run", "pytest", "sdk/python"],
    ["uv", "run", "pytest", "clients/agent"],
    ["cargo", "test", "--workspace"],
]


IDENTITY = {
    "GIT_AUTHOR_NAME": "removability",
    "GIT_AUTHOR_EMAIL": "removability@invalid",
    "GIT_COMMITTER_NAME": "removability",
    "GIT_COMMITTER_EMAIL": "removability@invalid",
}
"""The scratch commit's identity: a throwaway commit in a throwaway worktree, never pushed."""


def git(*arguments: str, hooks: Path, cwd: Path = ROOT) -> None:
    """`git` with `hooks`, an empty directory, as its hooks: the throwaway worktree runs none of the checkout's
    hooks. A hook the host installed can need a tool the container lacks (Git LFS's `post-checkout`, since the cast
    is stored with LFS, `b805ddaf`), and a hook has nothing to do for a copy that is never pushed."""
    command = ["git", "-c", f"core.hooksPath={hooks}", *arguments]
    subprocess.run(command, cwd=cwd, check=True, env={**os.environ, **IDENTITY})


def checkout_without(scratch: Path, removed: list[str], lm: bool) -> None:
    """A detached worktree of HEAD with the paths deleted (and, for the LM controller, its member line removed),
    committed there, so the copy is a repository with this checkout's history (`ac1_composability` reads both) and
    a clean tree."""
    hooks = scratch.parent / "no-hooks"
    hooks.mkdir(parents=True, exist_ok=True)
    git("worktree", "add", "--quiet", "--detach", str(scratch), "HEAD", hooks=hooks)
    git("rm", "-r", "--quiet", *removed, hooks=hooks, cwd=scratch)
    if lm:
        unregister(scratch)
    git("commit", "--quiet", "--all", "--no-verify", "--message", f"removability: {', '.join(removed)} removed",
        hooks=hooks, cwd=scratch)


def unregister(scratch: Path) -> None:
    pyproject = scratch / "pyproject.toml"
    text = pyproject.read_text(encoding="utf-8")
    members = [line for line in text.splitlines() if line.startswith("members = [")]
    if len(members) != 1 or members[0].count(MEMBER) != 1:
        raise SystemExit(f"[removability] pyproject.toml's members line does not name {REMOVED} once: {members}")
    pyproject.write_text(text.replace(members[0], members[0].replace(MEMBER, "")), encoding="utf-8")
    # `uv sync` re-resolves the lock without the member (the lock in the copy then differs from its commit).


def main(arguments: list[str]) -> int:
    arguments = list(arguments)
    target = "lm"
    if "--target" in arguments:
        at = arguments.index("--target")
        if at + 1 >= len(arguments):
            print(__doc__)
            return 2
        target = arguments[at + 1]
        del arguments[at:at + 2]
    known = {"--keep", "--python-only"}
    if target not in ("lm", "3d") or not set(arguments) <= known or (target == "3d" and "--python-only" in arguments):
        print(__doc__)
        return 2
    keep = "--keep" in arguments
    lm = target == "lm"
    removed = [REMOVED] if lm else REMOVED_3D
    if lm:
        commands = [c for c in COMMANDS if c[0] != "cargo"] if "--python-only" in arguments else COMMANDS
    else:
        commands = COMMANDS_3D
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-removability-")) / "checkout"
    environment = {key: value for key, value in os.environ.items() if key != "VIRTUAL_ENV"}
    # The checkout's own target directory, so the registry dependencies it already compiled are reused.
    target_dir = Path(environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target" / "removability")))
    suffix = ".exe" if os.name == "nt" else ""
    environment["MINEWORLD_BIN"] = str(target_dir / "debug" / f"mineworld{suffix}")
    what = ", ".join(removed)
    try:
        checkout_without(scratch, removed, lm)
        print(f"[removability] worktree of HEAD at {scratch}: {what} deleted"
              + (", its workspace member line removed" if lm else ""))
        assert not any((scratch / path).exists() for path in removed)
        for command in commands:
            started = time.monotonic()
            print(f"[removability] $ {' '.join(command)}", flush=True)
            status = subprocess.run(command, cwd=scratch, env=environment, check=False).returncode
            print(f"[removability]   exit {status} in {time.monotonic() - started:.1f} s", flush=True)
            if status != 0:
                print(f"[removability] FAIL: without {what}, `{' '.join(command)}` failed")
                return status
        print(f"[removability] PASS: everything outside {what} builds and passes without it")
        return 0
    finally:
        if keep:
            print(f"[removability] kept {scratch}")
        else:
            subprocess.run(["git", "worktree", "remove", "--force", str(scratch)], cwd=ROOT, check=False)
            shutil.rmtree(scratch.parent, ignore_errors=True)
            subprocess.run(["git", "worktree", "prune"], cwd=ROOT, check=False)


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
