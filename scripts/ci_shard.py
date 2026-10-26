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
# packages sharing the name. Re-measured 2026-10-21 (ARC-110 re-weigh note) from the `core` shard logs of
# runs 39635558077 and 39671562559 (PR #455, two heads of that day): 8705 s in all,
# three times the 2855 s of the first table, and the old table missed new heavy units such as
# `market_settlement` (763 s) and `guide` (402 s), which fell to DEFAULT_WEIGHT and unbalanced the shards.
# `run` was cancelled on Windows past 600 s in both runs; its weight is Linux's 656 s times 1.3, the
# suite's Windows/Linux ratio. Units seen only on Linux are scaled the same way. One table serves every OS:
# only the proportions matter for balance. Units under 3 s are left to DEFAULT_WEIGHT. `market_town` (one
# 300-day test, 1168 s) is the floor of any shard. Re-measure when a shard passes 35 min (ARC-110).
WEIGHTS: dict[str, int] = {
    "market_town": 1168,
    "town_bodies": 1123,
    "run": 853,
    "market_settlement": 763,
    "market_composition": 433,
    "social_interactions": 409,
    "guide": 402,
    "milestone_e": 327,
    "run_restart": 292,
    "social_composition": 263,
    "forgetting_runs": 250,
    "third_party": 222,
    "interaction_runs": 199,
    "bodies_yard_restart": 198,
    "motivated": 190,
    "entity_packs": 129,
    "milestone_b": 127,
    "activity_length": 122,
    "hosted_town": 120,
    "decision_models": 113,
    "walking_pace": 95,
    "perceived": 73,
    "notices": 67,
    "inspect": 65,
    "biography": 62,
    "deltas": 60,
    "routines": 58,
    "lateness": 51,
    "bodies_yard": 48,
    "content_kinds": 41,
    "decisions": 31,
    "host_clock": 31,
    "random": 30,
    "needs_town": 26,
    "punctuality": 24,
    "compile_fail": 23,
    "item_names": 21,
    "milestone_d": 19,
    "restart": 17,
    "facts": 15,
    "ac3_reconnect": 13,
    "ac7_many_players": 13,
    "admin": 13,
    "milestone_c": 13,
    DOC: 11,
    "needs_controllers": 10,
    "forgetting": 8,
    "handshake": 8,
    "kill_and_resume": 7,
    "interactions": 6,
    "inv9_closed_vocabulary": 6,
    "settlement": 6,
    "route": 5,
    "walking": 5,
    "ac15_one_alice": 4,
    "ac5_takeover": 4,
    "retention": 4,
    "launcher": 3,
    "limits": 3,
    "record": 3,
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


def parse_shard(text: str) -> tuple[list[int], int]:
    """`K/N` or `K,L,.../N`: 1-based, each 1 <= K <= N, none twice. Several shards of one partition run
    as one (their units together), for a job that carries more than one shard (ARC-110)."""
    indices, _, count = text.partition("/")
    parts = indices.split(",")
    if not (count.isdigit() and parts and all(part.isdigit() for part in parts)):
        raise ValueError(f"a shard is K/N or K,L/N with 1 <= K <= N, not {text!r}")
    numbers = [int(part) for part in parts]
    if len(set(numbers)) != len(numbers) or not all(1 <= number <= int(count) for number in numbers):
        raise ValueError(f"a shard is K/N or K,L/N with 1 <= K <= N, not {text!r}")
    return sorted(numbers), int(count)


def workspace_units() -> list[str]:
    output = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    return units_of(json.loads(output))


def shard_commands(text: str) -> tuple[list[list[str]], str]:
    """Shard K/N's commands (or K,L/N's, their units together) and a one-line account of the whole
    partition for the log."""
    indices, count = parse_shard(text)
    shards = partition(workspace_units(), count, WEIGHTS)
    loads = [sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
    account = ", ".join(
        f"{number}/{count}: {len(shard)} units ~{load} s" for number, (shard, load) in enumerate(zip(shards, loads), 1)
    )
    units = sorted(unit for index in indices for unit in shards[index - 1])
    return commands(units), f"partition by ARC-110 weights: {account}"


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
    for bad in ("0/3", "4/3", "3", "a/b", "1/0", "1,1/3", "1,4/3", ",/3", "1,/3"):
        try:
            parse_shard(bad)
            failures.append(f"parse_shard accepted {bad!r}")
        except ValueError:
            pass
    check(parse_shard("2/3") == ([2], 3), "parse_shard 2/3")
    check(parse_shard("2,1/3") == ([1, 2], 3), "parse_shard 2,1/3")

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
