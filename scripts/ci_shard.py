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

    python3 scripts/ci_shard.py --plan N      print each shard's units and expected seconds
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


def units_of(metadata: dict) -> list[str]:
    """Every unit `cargo test --workspace` runs, from `cargo metadata --no-deps`."""
    tests: set[str] = set()
    for package in metadata["packages"]:
        for target in package["targets"]:
            if target["kind"] == ["test"]:
                tests.add(target["name"])
    return [LIB_BINS, DOC, *sorted(tests)]


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


def commands(shard: list[str]) -> list[list[str]]:
    """The cargo commands that run exactly this shard's units, building first as `core` does."""
    targets: list[str] = ["--lib", "--bins"] if LIB_BINS in shard else []
    for unit in shard:
        if unit not in (LIB_BINS, DOC):
            targets += ["--test", unit]
    result: list[list[str]] = []
    if targets:
        result += [
            ["cargo", "test", "--workspace", "--no-run", *targets],
            ["cargo", "test", "--workspace", *targets],
        ]
    if DOC in shard:
        result.append(["cargo", "test", "--workspace", "--doc"])
    return result


def parse_shard(text: str) -> tuple[int, int]:
    """`K/N`, 1-based, 1 <= K <= N."""
    index, _, count = text.partition("/")
    if not (index.isdigit() and count.isdigit()) or not 1 <= int(index) <= int(count):
        raise ValueError(f"a shard is K/N with 1 <= K <= N, not {text!r}")
    return int(index), int(count)


def workspace_units() -> list[str]:
    output = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return units_of(json.loads(output))


def shard_commands(text: str) -> tuple[list[list[str]], str]:
    """Shard K/N's commands and a one-line account of the whole partition for the log."""
    index, count = parse_shard(text)
    shards = partition(workspace_units(), count, WEIGHTS)
    loads = [sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
    account = ", ".join(
        f"{number}/{count}: {len(shard)} units ~{load} s" for number, (shard, load) in enumerate(zip(shards, loads), 1)
    )
    return commands(shards[index - 1]), f"partition by ARC-110 weights: {account}"


def plan(count: int) -> int:
    weights = WEIGHTS
    shards = partition(workspace_units(), count, weights)
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
    for bad in ("0/3", "4/3", "3", "a/b", "1/0"):
        try:
            parse_shard(bad)
            failures.append(f"parse_shard accepted {bad!r}")
        except ValueError:
            pass
    check(parse_shard("2/3") == (2, 3), "parse_shard 2/3")

    for failure in failures:
        print(f"ci_shard self-test: FAIL: {failure}")
    print(f"ci_shard self-test: {'FAILED' if failures else 'passed'}")
    return 1 if failures else 0


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    if len(arguments) == 2 and arguments[0] == "--plan" and arguments[1].isdigit() and int(arguments[1]) >= 1:
        return plan(int(arguments[1]))
    print("usage: ci_shard.py --plan N | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
