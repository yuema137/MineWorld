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

    python3 scripts/ci_layer.py fast | core | parity
                                                   run a layer (on Linux in the toolchain container;
                                                   parity natively on macOS and Windows runners)
    python3 scripts/ci_layer.py platforms         S16's packages natively on macOS and Windows
                                                   (`.github/actions/native`, not the container)
    python3 scripts/ci_layer.py python            the Python workspace: static checks, the binary, pytest
    python3 scripts/ci_layer.py python-smoke      the same without the binary or the real_server tests
    python3 scripts/ci_layer.py --list <layer>     print a layer's commands without running them
    python3 scripts/ci_layer.py --offline-check    vendor outside the checkout, then check the CLI
                                                   offline with an empty CARGO_HOME (EC-3 (b))
    python3 scripts/ci_layer.py --prune-cache     before CI saves target/: drop the workspace's own
                                                   artifacts and the tests' scratch saves, keep the
                                                   compiled dependencies
"""

from __future__ import annotations

import json
import os
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
        ["python3", "scripts/check_scratch.py", "scan"],
        ["python3", "scripts/ci_parity.py", "--self-test"],
        ["cargo", "check", "--workspace", "--all-targets"],
        ["cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings"],
        # The code graph's licences, sources and bans (DEP-22, deny.toml); never advisories (ARC-48).
        ["cargo", "deny", "check", "licenses", "sources", "bans"],
    ],
    # Building first and running second runs the same tests; it only makes the log say how long the
    # build took and how long the tests did, which is what CI's budget is judged by. After a passing
    # suite, nothing may be left behind (ENGINEERING_STANDARDS.md §22, "Test scratch"; QTH-4).
    "core": [
        ["cargo", "test", "--workspace", "--no-run"],
        ["cargo", "test", "--workspace"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # AC-8's record on a native runner (macOS, Windows): the release binary, then every world recorded
    # (docs/DECISIONS.md ARC-49). The Linux legs record from the runtime image instead. On Windows the
    # script finds target/release/mineworld.exe.
    "parity": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_parity.py", "record", "--binary", "target/release/mineworld"],
    ],
    # S16's packages on every platform (step-16 §16.12 PD-p1, §17.12 PD-q4): run natively on macOS and
    # Windows by the `platforms` job, outside the container. The subset of the suite that S16's crates and
    # commands own and that is portable today; the whole workspace on Windows is S13's (RE-p1). Each PR
    # of S16 that lands a portable CLI target adds it here (E-c: `third_party`, and PD-p3's offline check).
    "platforms": [
        # E-c: everything the lock names, first (EC-3 (a)'s fetch). The lock guard's `cargo metadata
        # --offline` reads every workspace crate's dependencies, dev-dependencies of crates this layer
        # never builds included (`trybuild`'s `glob`), so nothing after this may depend on what a build
        # happened to download.
        ["cargo", "fetch", "--locked"],
        ["cargo", "build", "--locked", "-p", "mineworld-cli"],
        # --no-fail-fast: on a platform, one red test binary must not hide another's result.
        [
            "cargo", "test", "--locked", "--no-fail-fast",
            "-p", "mineworld-packages", "-p", "mineworld-worldpack", "-p", "mineworld-installed-systems",
        ],
        [
            "cargo", "test", "--locked", "--no-fail-fast", "-p", "mineworld-cli",
            "--test", "packs", "--test", "requirements", "--test", "entity_packs",
            "--test", "third_party",
        ],
        # E-c: the lock guard and the graph check over this platform's checkout (EC-13), then the build
        # offline from a vendor directory with an empty CARGO_HOME (PD-p3).
        [
            "cargo", "test", "--locked", "--no-fail-fast",
            "-p", "mineworld-acceptance", "--test", "package_sources",
        ],
        ["python3", "scripts/ci_layer.py", "--offline-check"],
    ],
}

# The Python workspace (sdk/python; DECISIONS.md DEP-26, ARC-48's note of 2026-10-08). Its tests never
# join `core`, whose job is a required check that must not get slower (pr-s10-p3 QP3-3). Every
# command runs through the lock (`uv run --locked`). A repository script is run by `sys.executable`
# rather than the literal `python3`, which Windows does not have; these layers also run outside the
# container, on Windows and macOS.
PYTHON_STATIC: list[list[str]] = [
    ["uv", "sync", "--locked"],
    ["uv", "run", "--locked", "ruff", "check", "sdk/python"],
    ["uv", "run", "--locked", "ruff", "format", "--check", "sdk/python"],
    ["uv", "run", "--locked", "pyright", "sdk/python"],
]
# The static checks are also part of `fast`: they add about 6 s to it, measured on PR #98's first run
# (uv sync 3.0 s, ruff 0.1 s, pyright 2.5 s), well under the 60 s the ruling allows (QP3-3), so a Python
# lint or type error blocks a merge like a Rust one. The `python` layers keep them too, so that they are
# also judged on Windows and macOS, where `fast` does not run (D-P3-11, AP-12).
LAYERS["fast"] += PYTHON_STATIC
LAYERS["python"] = [
    *PYTHON_STATIC,
    # The real_server tests start the real binary; they fail, never skip, without it (D-P3-10).
    ["cargo", "build", "-p", "mineworld-cli"],
    ["uv", "run", "--locked", "pytest", "sdk/python"],
    [sys.executable, "scripts/check_scratch.py", "left", "--target-dir", "target"],
]
# No Rust build: the real_server tests are deselected by name, visibly, here and nowhere else.
LAYERS["python-smoke"] = [
    *PYTHON_STATIC,
    ["uv", "run", "--locked", "pytest", "sdk/python", "-m", "not real_server"],
]

# The pyright wrapper otherwise prefers whatever `node` is on PATH over the locked Node wheel, and asks
# PyPI for its newest version on every run (DEP-26).
COMMAND_ENVIRONMENT = {"PYRIGHT_PYTHON_GLOBAL_NODE": "0", "PYRIGHT_PYTHON_IGNORE_WARNINGS": "1"}

# Layers whose disk use is worth recording (step-14 A13-3: free disk and the size of target/).
MEASURES_DISK = {"core"}

ENVIRONMENT: list[list[str]] = [
    ["rustc", "-V"],
    ["cargo", "-V"],
    ["cargo", "fmt", "--version"],
    ["cargo", "clippy", "--version"],
    ["cargo", "deny", "--version"],
    ["git", "--version"],
    ["uv", "--version"],
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


def size(path: Path) -> int:
    """Bytes under a directory, without following links (`du`, which Windows lacks)."""
    total = 0
    for directory, _, files in os.walk(path):
        for name in files:
            try:
                total += os.lstat(os.path.join(directory, name)).st_size
            except OSError:
                pass  # a file removed while walking: it no longer takes space
    return total


def gigabytes(count: int) -> str:
    return f"{count / 1024**3:.1f} G"


def disk(moment: str) -> None:
    """Prints free disk and target/'s size for the record, with no `df` or `du`, which Windows lacks;
    the record is information, never a verdict."""
    usage = shutil.disk_usage(ROOT)
    print(f"[ci] disk {moment}: {gigabytes(usage.free)} free of {gigabytes(usage.total)}", flush=True)
    for path in ("target", "target/tmp"):
        if (ROOT / path).exists():
            print(f"[ci]   {path}: {gigabytes(size(ROOT / path))}", flush=True)


def resolved(command: list[str]) -> list[str]:
    """`python3` is this interpreter: Windows runners have no `python3` on PATH. The listed command
    (`--list`) stays as written."""
    return [sys.executable, *command[1:]] if command[0] == "python3" else command


def offline_check() -> int:
    """EC-3 (b) on this runner (step-16 §16.12 PD-p3): vendor every dependency to a directory outside
    the checkout, then check the CLI with `--offline --frozen` against it and an empty CARGO_HOME, so
    nothing can come from the network or from a cache.

    The vendor directory and the empty home are siblings of the checkout and are removed afterwards.
    The configuration `cargo vendor` prints is written to a file with its directory as a `/` path, so
    it needs no escaping in TOML on Windows."""
    outside = ROOT.parent
    vendor = outside / "mineworld-vendor"
    home = outside / "mineworld-empty-cargo-home"
    config = outside / "mineworld-vendor.toml"
    for path in (vendor, home):
        shutil.rmtree(path, ignore_errors=True)
    home.mkdir()
    try:
        vendored = subprocess.run(
            ["cargo", "vendor", "--locked", "--versioned-dirs", str(vendor)],
            cwd=ROOT,
            capture_output=True,
            text=True,
        )
        if vendored.returncode != 0:
            print(vendored.stderr, file=sys.stderr, flush=True)
            return vendored.returncode
        lines = [
            f'directory = "{vendor.as_posix()}"' if line.startswith("directory = ") else line
            for line in vendored.stdout.splitlines()
        ]
        config.write_text("\n".join(lines) + "\n", encoding="utf-8")
        crates = sum(1 for entry in vendor.iterdir() if entry.is_dir())
        print(f"[ci] vendored {crates} crates to {vendor}; CARGO_HOME={home} (empty)", flush=True)
        environment = {**os.environ, "CARGO_HOME": str(home)}
        command = [
            "cargo", "check", "--offline", "--frozen", "--config", str(config), "-p", "mineworld-cli",
        ]
        print(f"[ci] $ {shlex.join(command)}", flush=True)
        return subprocess.run(command, cwd=ROOT, env=environment).returncode
    finally:
        for path in (vendor, home):
            shutil.rmtree(path, ignore_errors=True)
        config.unlink(missing_ok=True)


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
            status = subprocess.run(
                resolved(command), cwd=ROOT, env={**os.environ, **COMMAND_ENVIRONMENT}
            ).returncode
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
    if arguments == ["--offline-check"]:
        return offline_check()
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
