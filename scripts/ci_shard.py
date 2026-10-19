#!/usr/bin/env python3
"""Partition the `core` layer's default suite into N shards: deterministic, complete, and balanced.

`docs/DECISIONS.md` ARC-110. `test`, `test-macos` and `test-windows` run `core` as parallel shards because
the whole suite outgrew one job's cap on every OS; each shard runs `scripts/ci_layer.py core --shard K/N`,
and `ci_layer.py` asks this module which cargo commands shard K is.

The unit of partition is a test target *name*, across the workspace: `cargo test --workspace --test NAME`
runs every package's integration test of that name, from that package's directory, exactly as the whole
`cargo test --workspace` does (cargo's own target filter, nothing re-implemented). Two more units stand
for the rest of what `cargo test --workspace` runs: `<lib+bins>` (every package's unit tests) and
`<doc>` (every library's doctests, which cargo cannot mix with other target filters).

The target names come from `cargo metadata` at run time, so a test added after the weights were measured
is still in exactly one shard (weight `DEFAULT_WEIGHT`); a stale weight is ignored. Assignment is the
longest-processing-time rule: units by descending weight (ties by name), each to the least-loaded shard
(ties to the lowest index). Same names and weights, same partition, on every machine.

A targeted run (`docs/DECISIONS.md` ARC-154: a merge train or a `ci`-labelled PR) runs the shards of the
affected packages only: `--packages a,b` scopes the units to those packages' targets and the commands to
`-p a -p b` instead of `--workspace`. `--lib`, `--bins` and `--doc` are passed only when a scoped package has
such a target, because cargo refuses a target filter that no selected package can satisfy.

    python3 scripts/ci_shard.py --plan N [--packages a,b]   print each shard's units and expected seconds
    python3 scripts/ci_shard.py --self-test   check the partition rule on synthetic workspaces
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

LIB_BINS = "<lib+bins>"
DOC = "<doc>"

# Seconds per unit on windows-2025: the wall time between cargo's `Running` lines, summed over the
# packages sharing the name, from the first complete sharded run (PR #224's run 38804018898, main at
# 4457ad47; 2855 s in all). `town_bodies` (12d, #171) is not on main yet: 480 s is its share of
# `market_town` on #171's green run 38664714380 (421 s of 456 s on Windows, 437 of 458 on Linux) scaled to
# this table, rounded up; #171's run 38786685064 was cancelled 4 min into it on macOS. One table serves
# every OS: only the proportions matter for balance, and they agree across the three within the noise of
# one run. Units under 3 s are left to DEFAULT_WEIGHT. Re-measure when a shard passes 35 min (ARC-110).
WEIGHTS: dict[str, int] = {
    "market_town": 491,
    "town_bodies": 480,
    "run": 415,
    "social_interactions": 208,
    "milestone_e": 206,
    "bodies_yard_restart": 163,
    "hosted_town": 120,
    "social_composition": 119,
    "forgetting_runs": 117,
    "walking_pace": 95,
    "interaction_runs": 95,
    "third_party": 91,
    "run_restart": 79,
    "motivated": 72,
    "market_composition": 70,
    "deltas": 60,
    "milestone_b": 59,
    "perceived": 40,
    "entity_packs": 39,
    "bodies_yard": 38,
    "inspect": 29,
    "routines": 28,
    "decision_models": 25,
    "biography": 20,
    "restart": 16,
    "content_kinds": 16,
    "facts": 15,
    "admin": 13,
    "ac3_reconnect": 12,
    "decisions": 12,
    "compile_fail": 11,
    DOC: 8,
    "forgetting": 8,
    "handshake": 8,
    "kill_and_resume": 5,
    "walking": 5,
    "retention": 4,
    "record": 4,
    "ac5_takeover": 4,
    "milestone_c": 3,
    "interactions": 3,
}
DEFAULT_WEIGHT = 1


LIBRARY_KINDS = {"lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"}


class Scope:
    """What the cargo commands select: the whole workspace, or named packages (ARC-154) together with
    whether any of them has a library (doctests, `--lib`) or a binary (`--bins`)."""

    def __init__(self, packages: list[str] | None = None, lib: bool = True, bins: bool = True) -> None:
        self.packages = packages
        self.lib = lib
        self.bins = bins

    def flags(self) -> list[str]:
        if self.packages is None:
            return ["--workspace"]
        return [argument for name in self.packages for argument in ("-p", name)]


WORKSPACE = Scope()


def selected(metadata: dict, packages: list[str] | None) -> list[dict]:
    """The packages a scope names (all of them for the workspace); an unknown name is an error, never a
    silently smaller run."""
    if packages is None:
        return list(metadata["packages"])
    known = {package["name"]: package for package in metadata["packages"]}
    missing = sorted(set(packages) - set(known))
    if missing:
        raise ValueError(f"not workspace packages: {', '.join(missing)}")
    return [known[name] for name in sorted(set(packages))]


def scope_of(metadata: dict, packages: list[str] | None) -> Scope:
    if packages is None:
        return WORKSPACE
    chosen = selected(metadata, packages)
    kinds = {kind for package in chosen for target in package["targets"] for kind in target["kind"]}
    return Scope(sorted(set(packages)), lib=bool(kinds & LIBRARY_KINDS), bins="bin" in kinds)


def units_of(metadata: dict, packages: list[str] | None = None) -> list[str]:
    """Every unit `cargo test --workspace` runs (or `cargo test -p ...` for named packages), from
    `cargo metadata --no-deps`."""
    tests: set[str] = set()
    for package in selected(metadata, packages):
        for target in package["targets"]:
            if target["kind"] == ["test"]:
                tests.add(target["name"])
    if packages is None:
        return [LIB_BINS, DOC, *sorted(tests)]
    scope = scope_of(metadata, packages)
    return [*([LIB_BINS] if scope.lib or scope.bins else []), *([DOC] if scope.lib else []), *sorted(tests)]


def weight_of(units: list[str]) -> int:
    return sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in units)


def partition(units: list[str], count: int, weights: dict[str, int]) -> list[list[str]]:
    """Longest processing time first: deterministic for the same units and weights."""
    if count < 1:
        raise ValueError(f"a partition has at least one shard, not {count}")
    shards: list[list[str]] = [[] for _ in range(count)]
    loads = [0] * count
    for unit in sorted(units, key=lambda unit: (-weights.get(unit, DEFAULT_WEIGHT), unit)):
        lightest = min(range(count), key=lambda index: (loads[index], index))
        shards[lightest].append(unit)
        loads[lightest] += weights.get(unit, DEFAULT_WEIGHT)
    return [sorted(shard) for shard in shards]


def commands(shard: list[str], scope: Scope = WORKSPACE) -> list[list[str]]:
    """The cargo commands that run exactly this shard's units, building first as `core` does."""
    targets: list[str] = []
    if LIB_BINS in shard:
        targets += [*(["--lib"] if scope.lib else []), *(["--bins"] if scope.bins else [])]
    for unit in shard:
        if unit not in (LIB_BINS, DOC):
            targets += ["--test", unit]
    result: list[list[str]] = []
    if targets:
        result += [
            ["cargo", "test", *scope.flags(), "--no-run", *targets],
            ["cargo", "test", *scope.flags(), *targets],
        ]
    if DOC in shard:
        result.append(["cargo", "test", *scope.flags(), "--doc"])
    return result


def parse_shard(text: str) -> tuple[list[int], int]:
    """`K/N` or `K,L,.../N`: 1-based, each 1 <= K <= N, none twice. Several shards of one partition run
    as one (their units together): macOS's job (a) runs `1,2/3` (ARC-110)."""
    indices, _, count = text.partition("/")
    parts = indices.split(",")
    if not (count.isdigit() and parts and all(part.isdigit() for part in parts)):
        raise ValueError(f"a shard is K/N or K,L/N with 1 <= K <= N, not {text!r}")
    numbers = [int(part) for part in parts]
    if len(set(numbers)) != len(numbers) or not all(1 <= number <= int(count) for number in numbers):
        raise ValueError(f"a shard is K/N or K,L/N with 1 <= K <= N, not {text!r}")
    return sorted(numbers), int(count)


def workspace_metadata() -> dict:
    output = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return json.loads(output)


def parse_packages(text: str) -> list[str]:
    """`a,b,c`: the `--packages` argument (ARC-154); never empty."""
    names = [name for name in text.split(",") if name]
    if not names:
        raise ValueError(f"--packages names at least one package, not {text!r}")
    return names


def shard_commands(text: str, packages: list[str] | None = None) -> tuple[list[list[str]], str]:
    """Shard K/N's commands (or K,L/N's, their units together) and a one-line account of the whole
    partition for the log. With `packages`, the partition is of those packages' units only (ARC-154)."""
    indices, count = parse_shard(text)
    metadata = workspace_metadata()
    scope = scope_of(metadata, packages)
    shards = partition(units_of(metadata, packages), count, WEIGHTS)
    loads = [sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
    account = ", ".join(
        f"{number}/{count}: {len(shard)} units ~{load} s" for number, (shard, load) in enumerate(zip(shards, loads), 1)
    )
    units = sorted(unit for index in indices for unit in shards[index - 1])
    scoped = "the workspace" if packages is None else f"packages {', '.join(scope.packages or [])}"
    return commands(units, scope), f"{scoped}; partition by ARC-110 weights: {account}"


def plan(count: int, packages: list[str] | None = None) -> int:
    weights = WEIGHTS
    shards = partition(units_of(workspace_metadata(), packages), count, weights)
    for number, shard in enumerate(shards, 1):
        load = sum(weights.get(unit, DEFAULT_WEIGHT) for unit in shard)
        print(f"shard {number}/{count}: ~{load} s, {len(shard)} units: {' '.join(shard)}")
    return 0


def self_test() -> int:
    failures: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            failures.append(message)

    metadata = {
        "packages": [
            {"targets": [{"kind": ["lib"], "name": "a"}, {"kind": ["test"], "name": "admin"},
                         {"kind": ["test"], "name": "slow"}, {"kind": ["custom-build"], "name": "build-script-build"}]},
            {"targets": [{"kind": ["bin"], "name": "b"}, {"kind": ["test"], "name": "admin"},
                         {"kind": ["test"], "name": "new_one"}]},
        ]
    }
    units = units_of(metadata)
    check(units == [LIB_BINS, DOC, "admin", "new_one", "slow"], f"units_of: a shared name is one unit: {units}")

    many = [LIB_BINS, DOC, *(f"t{number:03}" for number in range(180))]
    weights = {"t000": 480, "t001": 470, "t002": 300, "t003": 260, **{f"t{n:03}": n % 7 for n in range(4, 180)}}
    for count in (1, 2, 3, 4):
        shards = partition(many, count, weights)
        flat = [unit for shard in shards for unit in shard]
        check(sorted(flat) == sorted(many), f"{count} shards: every unit exactly once")
        check(len(flat) == len(set(flat)), f"{count} shards: no unit twice")
        check(shards == partition(list(reversed(many)), count, dict(weights)), f"{count} shards: input order")
        loads = [sum(weights.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
        total = sum(weights.get(unit, DEFAULT_WEIGHT) for unit in many)
        check(max(loads) <= total / count + max(weights.values()), f"{count} shards: LPT bound {loads}")
        ran: list[str] = []
        for shard in shards:
            for command in commands(shard):
                if "--no-run" in command:
                    continue
                ran += [command[i + 1] for i, argument in enumerate(command) if argument == "--test"]
                ran += [LIB_BINS] if "--lib" in command else []
                ran += [DOC] if "--doc" in command else []
        check(sorted(ran) == sorted(many), f"{count} shards: the commands run every unit once")

    check(commands([DOC]) == [["cargo", "test", "--workspace", "--doc"]], "a doc-only shard never mixes --doc")
    check(commands(["x"])[0] == ["cargo", "test", "--workspace", "--no-run", "--test", "x"], "build first")
    for bad in ("0/3", "4/3", "3", "a/b", "1/0", "1,1/3", "1,4/3", ",/3", "1,/3"):
        try:
            parse_shard(bad)
            failures.append(f"parse_shard accepted {bad!r}")
        except ValueError:
            pass
    check(parse_shard("2/3") == ([2], 3), "parse_shard 2/3")
    check(parse_shard("2,1/3") == ([1, 2], 3), "parse_shard 2,1/3")

    # Targeted runs (ARC-154): a package scope selects those packages' units and passes only the target
    # filters cargo can satisfy (cargo errors on `-p bin_only --lib` and on `--doc` without a library).
    scoped = {
        "packages": [
            {"name": "lib-a", "targets": [{"kind": ["lib"], "name": "lib_a"}, {"kind": ["test"], "name": "admin"}]},
            {"name": "bin-b", "targets": [{"kind": ["bin"], "name": "b"}, {"kind": ["test"], "name": "slow"}]},
            {"name": "macro-c", "targets": [{"kind": ["proc-macro"], "name": "c"}]},
        ]
    }
    check(units_of(scoped, ["bin-b"]) == [LIB_BINS, "slow"], f"bin-only scope: no doctests {units_of(scoped, ['bin-b'])}")
    check(units_of(scoped, ["lib-a"]) == [LIB_BINS, DOC, "admin"], "lib scope: doctests and its own tests only")
    check(units_of(scoped, ["macro-c"]) == [LIB_BINS, DOC], "a proc-macro is a library")
    bin_only = scope_of(scoped, ["bin-b"])
    check(commands([LIB_BINS], bin_only)[1] == ["cargo", "test", "-p", "bin-b", "--bins"], "bin-only: --bins only")
    both = scope_of(scoped, ["bin-b", "lib-a"])
    check(commands([LIB_BINS, DOC], both) == [
        ["cargo", "test", "-p", "bin-b", "-p", "lib-a", "--no-run", "--lib", "--bins"],
        ["cargo", "test", "-p", "bin-b", "-p", "lib-a", "--lib", "--bins"],
        ["cargo", "test", "-p", "bin-b", "-p", "lib-a", "--doc"],
    ], f"two packages: -p each, both filters {commands([LIB_BINS, DOC], both)}")
    check(commands(["admin"], WORKSPACE)[1] == ["cargo", "test", "--workspace", "--test", "admin"],
          "the workspace scope is unchanged")
    try:
        units_of(scoped, ["nope"])
        failures.append("an unknown package was accepted")
    except ValueError:
        pass
    for bad in ("", ","):
        try:
            parse_packages(bad)
            failures.append(f"parse_packages accepted {bad!r}")
        except ValueError:
            pass
    check(parse_packages("a,b") == ["a", "b"], "parse_packages a,b")

    for failure in failures:
        print(f"ci_shard self-test: FAIL: {failure}")
    print(f"ci_shard self-test: {'FAILED' if failures else 'passed'}")
    return 1 if failures else 0


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    if len(arguments) in (2, 4) and arguments[0] == "--plan" and arguments[1].isdigit() and int(arguments[1]) >= 1:
        if len(arguments) == 2:
            return plan(int(arguments[1]))
        if arguments[2] == "--packages":
            return plan(int(arguments[1]), parse_packages(arguments[3]))
    print("usage: ci_shard.py --plan N [--packages a,b] | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
