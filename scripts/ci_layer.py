#!/usr/bin/env python3
"""Run one CI layer: the only place that says which commands a layer is.

`ENGINEERING_STANDARDS.md` §16 names CI's layers and `docs/DECISIONS.md` ARC-48 gives them triggers.
The workflow (`.github/workflows/ci.yml`) names a layer and calls this script inside the toolchain
container; it never names a command (step-14 I-S13-9). Keeping the table here means the commands a
layer runs are reviewed as policy in one list, the same entry point runs on a laptop
(`python3 scripts/ci_layer.py fast`), and moving to another CI service does not move the policy.

Before the first command it prints the toolchain and how the checkout was made (shallow or not,
partial-clone filter), because a green layer is only evidence about the toolchain and history it ran
with. Every command is printed before it runs and timed after; the first failure stops the layer with
that command's exit status. Nothing is retried and nothing is allowed to fail (ARC-48).

    python3 scripts/ci_layer.py fast | core       run a layer
    python3 scripts/ci_layer.py --list <layer>     print a layer's commands without running them
    python3 scripts/ci_layer.py --prune-cache      before CI saves target/: drop the workspace's own
                                                   artifacts and the tests' scratch saves, keep the
                                                   compiled dependencies
"""

from __future__ import annotations

import json
import shlex
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The declared checks of `.structured-coding/standards.md`, split by layer (ARC-48). Order matters
# only for speed of the verdict: the cheapest checks first.
LAYERS: dict[str, list[list[str]]] = {
    "fast": [
        ["cargo", "fmt", "--all", "--check"],
        ["python3", "scripts/check_doc_headings.py"],
        ["python3", "scripts/check_decision_ids.py"],
        ["python3", "scripts/check_ci_pins.py"],
        ["cargo", "check", "--workspace", "--all-targets"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
    ],
    # Building first and running second runs the same tests; it only makes the log say how long the
    # build took and how long the tests did, which is what CI's budget is judged by.
    "core": [
        ["cargo", "test", "--workspace", "--no-run"],
        ["cargo", "test", "--workspace"],
    ],
}

# Layers whose disk use is a finding of its own (step-14 F-3: the suite leaves ~16 GB of saves).
MEASURES_DISK = {"core"}

ENVIRONMENT: list[list[str]] = [
    ["rustc", "-V"],
    ["cargo", "-V"],
    ["cargo", "fmt", "--version"],
    ["cargo", "clippy", "--version"],
    ["git", "--version"],
    ["git", "rev-parse", "HEAD"],
    ["git", "rev-parse", "--is-shallow-repository"],
    ["git", "config", "--get", "remote.origin.partialclonefilter"],
]


def report(command: list[str]) -> None:
    """Prints a command's output for the record; its failure is information, not a verdict."""
    try:
        result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    except FileNotFoundError:
        print(f"[ci] {shlex.join(command)}: (not found on PATH)", flush=True)
        return
    output = (result.stdout or result.stderr).strip() or f"(none; exit {result.returncode})"
    print(f"[ci] {shlex.join(command)}: {output}", flush=True)


def disk(moment: str) -> None:
    print(f"[ci] disk {moment}:", flush=True)
    subprocess.run(["df", "-h", str(ROOT)], cwd=ROOT)
    present = [path for path in ("target", "target/tmp") if (ROOT / path).exists()]
    if present:
        subprocess.run(["du", "-sh", *present], cwd=ROOT)
    sys.stdout.flush()


def run(layer: str) -> int:
    print(f"[ci] layer {layer} in {ROOT}", flush=True)
    for command in ENVIRONMENT:
        report(command)
    measured = layer in MEASURES_DISK
    if measured:
        disk("before")
    started = time.monotonic()
    for command in LAYERS[layer]:
        print(f"[ci] $ {shlex.join(command)}", flush=True)
        began = time.monotonic()
        try:
            status = subprocess.run(command, cwd=ROOT).returncode
        except FileNotFoundError:
            print(f"[ci] {command[0]}: not found on PATH", flush=True)
            status = 127
        print(f"[ci] exit {status} after {time.monotonic() - began:.1f} s: {shlex.join(command)}", flush=True)
        if status != 0:
            if measured:
                disk("after (failed)")
            print(f"[ci] layer {layer} FAILED at: {shlex.join(command)}", flush=True)
            return status
    if measured:
        disk("after")
    print(f"[ci] layer {layer} passed: {len(LAYERS[layer])} command(s) in {time.monotonic() - started:.1f} s")
    return 0


def prune_cache() -> int:
    """Leaves target/ holding compiled dependencies only, so the cache CI saves stays small."""
    metadata = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if metadata.returncode != 0:
        print(metadata.stderr, file=sys.stderr)
        return metadata.returncode
    packages = sorted(package["name"] for package in json.loads(metadata.stdout)["packages"])
    shutil.rmtree(ROOT / "target" / "tmp", ignore_errors=True)
    command = ["cargo", "clean"] + [argument for name in packages for argument in ("-p", name)]
    print(f"[ci] $ cargo clean -p <{len(packages)} workspace packages>", flush=True)
    status = subprocess.run(command, cwd=ROOT).returncode
    disk("after pruning")
    return status


def main(arguments: list[str]) -> int:
    known = ", ".join(LAYERS)
    if arguments == ["--prune-cache"]:
        return prune_cache()
    if len(arguments) == 2 and arguments[0] == "--list" and arguments[1] in LAYERS:
        for command in LAYERS[arguments[1]]:
            print(shlex.join(command))
        return 0
    if len(arguments) == 1 and arguments[0] in LAYERS:
        return run(arguments[0])
    print(
        f"usage: ci_layer.py <layer> | --list <layer> | --prune-cache   (layers: {known})",
        file=sys.stderr,
    )
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
