#!/usr/bin/env python3
"""Partition the `core` layer's default suite into N shards: deterministic, complete, and balanced.

`docs/DECISIONS.md` ARC-110. `test-windows` runs `core` as three parallel jobs because the whole suite
outgrew one job's cap on windows-2025; each job runs `python scripts/ci_layer.py core --shard K/N`, and
`ci_layer.py` asks this module which cargo commands shard K is.

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
# packages sharing the name. Measured on PR #200's run 38787218062 and PR #206's run 38782285755 (both
# cancelled at 60 min after 78 and 80 of 250 binaries, every one passing); the targets neither run reached
# are Linux's time in run 38787218062 x 1.27, the Windows/Linux ratio over the targets both ran. Units under
# 3 s are left to DEFAULT_WEIGHT. Re-measure when a shard's run passes 35 min (ARC-110).
WEIGHTS: dict[str, int] = {
    "market_town": 487,
    "run": 472,
    "milestone_e": 304,
    "bodies_yard_restart": 264,
    "social_interactions": 192,
    "social_composition": 126,
    "walking_pace": 121,
    "hosted_town": 120,
    "forgetting_runs": 119,
    "run_restart": 115,
    "interaction_runs": 96,
    "third_party": 92,
    "market_composition": 75,
    "motivated": 73,
    "milestone_b": 62,
    "bodies_yard": 61,
    "deltas": 60,
    "perceived": 46,
    "entity_packs": 40,
    "decision_models": 40,
    "biography": 32,
    "inspect": 30,
    "routines": 29,
    "content_kinds": 23,
    "restart": 16,
    "facts": 15,
    "admin": 15,
    "ac3_reconnect": 13,
    "compile_fail": 11,
    "decisions": 10,
    "handshake": 10,
    "forgetting": 7,
    "kill_and_resume": 7,
    "walking": 5,
    "interactions": 5,
    "retention": 5,
    DOC: 4,
    "route": 4,
    "ac5_takeover": 4,
    "record": 3,
    "milestone_c": 3,
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
