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
    python3 scripts/ci_layer.py docs              a documentation-only change: the doc checks only, on
                                                   the bare runner (step-14 §16)
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
        ["python3", "scripts/ci_changes.py", "--self-test"],
        # The nightly's classifiers (ARC-83), each under a second: a verdict rule that fails open would
        # otherwise be found only by a red night that never comes.
        ["python3", "scripts/ci_nightly.py", "--self-test"],
        ["python3", "scripts/ci_godot.py", "--self-test"],
        ["python3", "scripts/ci_repeat.py", "--self-test"],
        ["python3", "scripts/ci_stability.py", "--self-test"],
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
    # A documentation-only change (step-14 §16, ARC-48's note of 2026-10-09 for 13x): the only CI
    # commands that read the audited docs set, plus the classifier's own self-test. Pure Python and the
    # standard library, run on the bare runner in `fast`'s place; no toolchain, no container.
    "docs": [
        ["python3", "scripts/check_doc_headings.py"],
        ["python3", "scripts/check_decision_ids.py"],
        ["python3", "scripts/ci_changes.py", "--self-test"],
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

# Layer 4, nightly (docs/DECISIONS.md ARC-83; .github/workflows/nightly.yml). None of these runs on a
# pull request or in `core`; each writes its result to artifacts/nightly/ for the night's verdict.
NIGHTLY_OUT = "artifacts/nightly"
LAYERS.update({
    # AC-8 at the long horizon on a native runner (macOS, Windows); the Linux legs record from the
    # runtime image instead (ARC-49's legs, the long profile).
    "parity-long": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_parity.py", "record", "--binary", "target/release/mineworld", "--profile", "long",
         "--timings", f"{NIGHTLY_OUT}/timings-parity-long.json"],
    ],
    # The server killed and restarted under a seated SDK client, every world's 300-day save replayed, and
    # CA-13 on market-town's save (ENGINEERING_STANDARDS.md §16 layer 4). The SDK runs through the lock.
    "stability": [
        ["cargo", "build", "--release", "--locked", "-p", "mineworld-cli"],
        ["uv", "run", "--locked", "python", "scripts/ci_stability.py", "restarts", "--binary", "target/release/mineworld"],
        ["python3", "scripts/ci_stability.py", "replay", "--binary", "target/release/mineworld"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # The unchanged default suite sampled twice more on this commit (flakes); every sample runs.
    "core-repeat": [
        ["cargo", "test", "--workspace", "--no-run"],
        ["python3", "scripts/ci_repeat.py", "--times", "2", "--summary", f"{NIGHTLY_OUT}/repeat.txt", "--",
         "cargo", "test", "--workspace", "--no-fail-fast"],
    ],
    # The 25 #[ignore]d Godot tests, one at a time (each starts Godot and a server), on every OS (DEP-45):
    # `cargo test -p mineworld-cli` over client_2d, client_2d_interact, client_2d_interact_stub and
    # client_settings with `--ignored --test-threads=1`, minus this OS's named skips (QC-5), which
    # ci_godot.py holds with their reasons; it records the failing tests' names for the night's verdict.
    "clients": [
        ["cargo", "build", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_godot.py", "tests"],
        ["python3", "scripts/ci_godot.py", "coverage"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
    # The bash launchers' probes, Linux and macOS (W-9): the 3D slice's verdicts parsed (they exit 0 on
    # failure), the protocol module's live checks, and AC-13's evidence judged by its owning test.
    "clients-probes": [
        ["cargo", "build", "-p", "mineworld-cli"],
        ["python3", "scripts/ci_godot.py", "slice", "--drive"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--link"],
        ["python3", "scripts/ci_godot.py", "slice", "--world", "--target"],
        ["bash", "clients/protocol/run.sh", "evidence"],
        ["bash", "clients/protocol/run.sh", "affordances"],
        ["bash", "clients/protocol/run.sh", "reconnect"],
        ["bash", "clients/protocol/run.sh", "perceived"],
        ["bash", "clients/protocol/run.sh", "deltas"],
        ["bash", "clients/protocol/run.sh", "admin"],
        ["cargo", "test", "-p", "mineworld-cli", "--test", "ac13_semantic_parity"],
        ["python3", "scripts/check_scratch.py", "left", "--target-dir", "target"],
    ],
})
# The nightly's layers record how they ended (artifacts/nightly/layer-<layer>.txt): `started`, then
# `passed` or `failed at: <command>`. The night's verdict reads it, so that a layer that never started is
# INCONCLUSIVE and one that ran and failed is FAIL (I-13c-3). Per-PR layers write nothing.
NIGHTLY_LAYERS = {"parity-long", "stability", "core-repeat", "clients", "clients-probes"}

# The Python workspace (sdk/python; DECISIONS.md DEP-26, ARC-48's note of 2026-10-08). Its tests never
# join `core`, whose job is a required check that must not get slower (pr-s10-p3 QP3-3). Every
# command runs through the lock (`uv run --locked`). A repository script is run by `sys.executable`
# rather than the literal `python3`, which Windows does not have; these layers also run outside the
# container, on Windows and macOS.
# The workspace's members (root pyproject.toml): the SDK (S10 P3) and the cognition package (S10 P5a).
PYTHON_MEMBERS = ["sdk/python", "cognition/lm-controller"]
PYTHON_STATIC: list[list[str]] = [
    ["uv", "sync", "--locked"],
    ["uv", "run", "--locked", "ruff", "check", *PYTHON_MEMBERS],
    ["uv", "run", "--locked", "ruff", "format", "--check", *PYTHON_MEMBERS],
    ["uv", "run", "--locked", "pyright", *PYTHON_MEMBERS],
]
# pytest runs once per member, never over both at once: given two paths, pytest takes its rootdir and
# ini file from their common ancestor, the repository root, whose pyproject.toml has no pytest section,
# and both members' `addopts` (the network guard among them) would be dropped silently
# (pr-s10-p5-backends.md D-P5-11). The cognition member's `addopts` also deselect `live_model`; no CI
# command selects that marker (D-P5-12).
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
    ["uv", "run", "--locked", "pytest", "cognition/lm-controller"],
    [sys.executable, "scripts/check_scratch.py", "left", "--target-dir", "target"],
]
# No Rust build: the tests that need the binary — the SDK's real_server tests and the cognition
# package's real_binary AC-10 scenario (pr-s10-p4 §5.10) — are deselected by name, visibly, here and
# nowhere else. A `-m` on the command line replaces the member's `addopts` `-m "not live_model"` rather
# than adding to it, so the cognition expression restates that deselection.
LAYERS["python-smoke"] = [
    *PYTHON_STATIC,
    ["uv", "run", "--locked", "pytest", "sdk/python", "-m", "not real_server"],
    [
        "uv", "run", "--locked", "pytest", "cognition/lm-controller",
        "-m", "not live_model and not real_binary",
    ],
]

# The pyright wrapper otherwise prefers whatever `node` is on PATH over the locked Node wheel, and asks
# PyPI for its newest version on every run (DEP-26).
COMMAND_ENVIRONMENT = {"PYRIGHT_PYTHON_GLOBAL_NODE": "0", "PYRIGHT_PYTHON_IGNORE_WARNINGS": "1"}

# Layers whose disk use is worth recording (step-14 A13-3: free disk and the size of target/).
MEASURES_DISK = {"core", "parity-long", "stability", "core-repeat"}

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
# The `docs` layer runs on the bare runner: probing `cargo` or `rustc` there would make rustup install
# the toolchain `rust-toolchain.toml` pins, which is the minute the layer exists to save.
ENVIRONMENT_OF: dict[str, list[list[str]]] = {
    "docs": [
        ["python3", "--version"],
        ["git", "--version"],
        ["git", "rev-parse", "HEAD"],
        ["git", "rev-parse", "--is-shallow-repository"],
    ],
}


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


def result(layer: str, line: str) -> None:
    """Appends to a nightly layer's result file; per-PR layers have none."""
    if layer in NIGHTLY_LAYERS:
        path = ROOT / NIGHTLY_OUT / f"layer-{layer}.txt"
        path.parent.mkdir(parents=True, exist_ok=True)
        with open(path, "a", encoding="utf-8", newline="\n") as out:
            out.write(line + "\n")


def run(layer: str) -> int:
    print(f"[ci] layer {layer} in {ROOT}", flush=True)
    for command in ENVIRONMENT_OF.get(layer, ENVIRONMENT):
        report(resolved(command))
    measured = layer in MEASURES_DISK
    if measured:
        disk("before")
    result(layer, "started")
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
            result(layer, f"failed at: {shlex.join(command)} (exit {status})")
            return status
    if measured:
        disk("after")
    print(f"[ci] layer {layer} passed: {len(LAYERS[layer])} command(s) in {time.monotonic() - started:.1f} s")
    result(layer, f"passed in {time.monotonic() - started:.0f} s")
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
