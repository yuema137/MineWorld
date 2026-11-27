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
import math
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parent.parent

LIB_BINS = "<lib+bins>"
DOC = "<doc>"

# The runners a shard's tests can run under (DEP-163): nextest for `core`, cargo for the nightly's
# `core-repeat`. `--profile ci` is .config/nextest.toml's: JUnit, no fail-fast.
RUNNERS = {"nextest", "cargo"}
NEXTEST_FLAGS = ["--profile", "ci", "--no-fail-fast"]
CARGO_TEST_FLAGS = ["--no-fail-fast"]
NEXTEST = ["cargo", "nextest", "run", "--workspace", *NEXTEST_FLAGS]
CARGO_TEST = ["test", "--workspace", *CARGO_TEST_FLAGS]

# The `harness = false` test targets (DEP-163): their own `main` does not answer nextest's `--list`, so they
# run under `cargo test`. The self-test fails when the workspace's manifests name a different set.
CUSTOM_HARNESS = frozenset({"arrival_resolvers_resume", "kill_and_resume", "launcher"})

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


def scope_flags(shard: list[str], scope: dict | None, units: list[str]) -> list[str]:
    """`--workspace`, or the `-p` flags of an impact selection (scripts/ci_impact.py): the packages that
    own this shard's test targets, and, when the shard holds `<lib+bins>` or `<doc>`, the selected
    packages with a library or a binary. Cargo applies every target filter to every listed package and
    refuses only a filter that no listed package has, so the union is exact enough: a lib's unit tests of
    an owner that was not selected whole are a second or two, and selecting more is never unsafe."""
    if scope is None:
        return ["--workspace"]
    names: set[str] = set()
    for unit in units:
        if unit == LIB_BINS:
            names.update(scope["lib_packages"], scope["bin_packages"])
        elif unit == DOC:
            names.update(scope["lib_packages"])
        else:
            names.update(scope["owners"][unit])
    return [flag for name in sorted(names) for flag in ("-p", name)]


def commands(shard: list[str], runner: str = "nextest", archive: Path | None = None,
             scope: dict | None = None) -> list[list[str]]:
    """Run exactly this shard's units, using an archive or building local targets first.

    `runner` is `nextest` (the `core` layer, DEP-163) or `cargo` (the nightly's `core-repeat`, whose flake
    classifier reads `cargo test`'s output). Under nextest, the custom-harness targets and the doctests still
    run with `cargo test`, which is the only runner they support. `scope` is an impact selection
    (scripts/ci_impact.py `Selection.core`): its packages replace `--workspace`, and it never uses an
    archive (the archive is the full run's, built for the whole workspace)."""
    if runner not in RUNNERS:
        raise ValueError(f"a runner is one of {sorted(RUNNERS)}, not {runner!r}")
    if scope is not None and archive is not None:
        raise ValueError("an impact selection builds its own targets; it takes no archive")
    lib_bins: list[str] = []
    if LIB_BINS in shard:
        lib_bins = ["--lib", "--bins"] if scope is None else (
            (["--lib"] if scope["lib_packages"] else []) + (["--bins"] if scope["bin_packages"] else []))
    targets: list[str] = lib_bins
    custom: list[str] = []
    for unit in shard:
        if unit in (LIB_BINS, DOC):
            continue
        if runner == "nextest" and unit in CUSTOM_HARNESS:
            custom += ["--test", unit]
        else:
            targets += ["--test", unit]
    # --no-fail-fast: one red test binary must not hide the results of the binaries after it (F-1).
    result: list[list[str]] = []
    if archive is not None and runner != "nextest":
        raise ValueError("archives are only supported by the nextest runner")
    tested = [unit for unit in shard if unit != DOC]
    where = scope_flags(shard, scope, tested)
    if archive is None and (targets or custom):
        result.append(["cargo", "test", *where, "--no-run", *targets, *custom])
    if targets:
        if archive is not None:
            filters = ["kind(lib) | kind(bin) | kind(proc-macro)"] if LIB_BINS in shard else []
            filters += [f"(kind(test) & binary(={unit}))" for unit in shard
                        if unit not in (LIB_BINS, DOC) and unit not in CUSTOM_HARNESS]
            result.append(["cargo", "nextest", "run", "--profile", "ci", "--no-fail-fast",
                           "--archive-file", str(archive / "tests.tar.zst"),
                           "--extract-to", str(ROOT), "--extract-overwrite",
                           "-E", " | ".join(filters)])
        else:
            regular = [unit for unit in tested if unit not in CUSTOM_HARNESS or runner != "nextest"]
            within = scope_flags(shard, scope, regular)
            result.append(["cargo", "nextest", "run", *within, *NEXTEST_FLAGS, *targets] if runner == "nextest"
                          else ["cargo", "test", *within, *CARGO_TEST_FLAGS, *targets])
    if custom:
        result.append(["cargo", "test", *scope_flags(shard, scope, [u for u in shard if u in CUSTOM_HARNESS]),
                       *CARGO_TEST_FLAGS, *custom])
    if DOC in shard:
        result.append(["cargo", "test", *scope_flags(shard, scope, [DOC]), *CARGO_TEST_FLAGS, "--doc"])
    return result


def is_test_run(command: list[str]) -> bool:
    """A command that runs tests (not a build of them): `cargo test` or `cargo nextest run`, without
    `--no-run`. Its failure is reported and the layer goes on to the next command (F-1)."""
    if len(command) > 2 and command[1] == "scripts/ci_repeat.py" and "--" in command:
        return is_test_run(command[command.index("--") + 1:])
    if "--no-run" in command or command[:1] != ["cargo"]:
        return False
    return command[1:2] == ["test"] or command[1:3] == ["nextest", "run"]


def custom_harness_targets(root: Path = ROOT) -> set[str]:
    """Every `[[test]]` target with `harness = false` in the workspace's manifests: these run under
    `cargo test` only, because their own `main` does not answer nextest's `--list` (DEP-163)."""
    found: set[str] = set()
    for manifest in root.rglob("Cargo.toml"):
        if any(part in ("target", ".git", "node_modules") for part in manifest.relative_to(root).parts):
            continue
        data = tomllib.loads(manifest.read_text(encoding="utf-8"))
        found |= {target["name"] for target in data.get("test", []) if target.get("harness") is False}
    return found


def unit_of(binary_id: str) -> str:
    """The partition unit of a nextest binary id: `crate` and `crate::bin/name` are `<lib+bins>`, and
    `crate::name` is the integration test target `name`."""
    _, separator, rest = binary_id.partition("::")
    if not separator or rest.startswith("bin/"):
        return LIB_BINS
    return rest


def weigh(paths: list[Path]) -> dict[str, int]:
    """Seconds per unit from nextest JUnit files: every test's own time, summed per unit and over the
    files given (one OS's shards). Units under 3 s are left out, as the table leaves them to
    DEFAULT_WEIGHT."""
    totals: dict[str, float] = {}
    seen: set[tuple[str, str]] = set()
    for path in paths:
        root = ElementTree.parse(path).getroot()
        if root.tag != "testsuites":
            raise ValueError(f"{path}: expected nextest testsuites")
        cases = 0
        for suite in root.findall("testsuite"):
            binary = suite.get("name")
            if not binary:
                raise ValueError(f"{path}: missing binary identity")
            unit = unit_of(binary)
            for case in suite.findall("testcase"):
                name = case.get("name")
                identity = (binary, name)
                if not name or identity in seen:
                    raise ValueError(f"{path}: missing or duplicate test identity {identity}")
                seen.add(identity)
                if any(case.find(tag) is not None for tag in ("failure", "error", "skipped")):
                    raise ValueError(f"{path}: incomplete/failed timing for {identity}")
                seconds = float(case.attrib["time"])
                if not math.isfinite(seconds) or seconds < 0:
                    raise ValueError(f"{path}: invalid duration for {identity}")
                totals[unit] = totals.get(unit, 0.0) + seconds
                cases += 1
        if not cases:
            raise ValueError(f"{path}: no test records")
    return {unit: round(seconds) for unit, seconds in totals.items() if seconds >= 3}


def print_weights(weights: dict[str, int]) -> None:
    """The table as WEIGHTS is written, heaviest first, ready to paste."""
    for unit, seconds in sorted(weights.items(), key=lambda item: (-item[1], item[0])):
        name = "DOC" if unit == DOC else "LIB_BINS" if unit == LIB_BINS else f'"{unit}"'
        print(f"    {name}: {seconds},")


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


def shard_commands(
    text: str, runner: str = "nextest", archive: Path | None = None, units: list[str] | None = None,
    scope: dict | None = None,
) -> tuple[list[list[str]], str]:
    """Shard K/N's commands (or K,L/N's, their units together) under `runner`, and a one-line account of
    the whole partition for the log."""
    indices, count = parse_shard(text)
    if units is None:
        units = workspace_units()
    shards = partition(units, count, WEIGHTS)
    loads = [sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
    account = ", ".join(
        f"{number}/{count}: {len(shard)} units ~{load} s" for number, (shard, load) in enumerate(zip(shards, loads), 1)
    )
    units = sorted(unit for index in indices for unit in shards[index - 1])
    return commands(units, runner, archive, scope), f"partition by ARC-110 weights: {account}"


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

    many = [LIB_BINS, DOC, *sorted(CUSTOM_HARNESS), *(f"t{number:03}" for number in range(180))]
    weights = {"t000": 480, "t001": 470, "t002": 300, "t003": 260, **{f"t{n:03}": n % 7 for n in range(4, 180)}}
    for count, runner in ((count, runner) for count in (1, 2, 3, 4) for runner in sorted(RUNNERS)):
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
            for command in commands(shard, runner):
                if not is_test_run(command):
                    continue
                tests = [command[i + 1] for i, argument in enumerate(command) if argument == "--test"]
                check(
                    command[1] == "test" or not CUSTOM_HARNESS & set(tests),
                    f"{count} shards, {runner}: a custom-harness target never runs under nextest: {command}",
                )
                ran += tests
                ran += [LIB_BINS] if "--lib" in command else []
                ran += [DOC] if "--doc" in command else []
        check(sorted(ran) == sorted(many), f"{count} shards, {runner}: the commands run every unit once")

    for runner in sorted(RUNNERS):
        check(
            commands([DOC], runner) == [["cargo", "test", "--workspace", "--no-fail-fast", "--doc"]],
            f"{runner}: a doc-only shard never mixes --doc and runs every doctest binary",
        )
        build = commands(["launcher", "x"], runner)[0]
        check(
            build[:4] == ["cargo", "test", "--workspace", "--no-run"]
            and sorted(build[4:]) == sorted(["--test", "launcher", "--test", "x"]),
            f"{runner}: build every target first, custom-harness ones included: {build}",
        )
        check(
            all("--no-fail-fast" in command for command in commands(["x", "launcher"], runner) if is_test_run(command)),
            f"{runner}: every test command of a shard is --no-fail-fast",
        )
    check(
        commands(["x"])[1] == [*NEXTEST, "--test", "x"] and commands(["x"], "cargo")[1][:2] == ["cargo", "test"],
        "core runs nextest with the ci profile; core-repeat runs cargo test",
    )
    check(not is_test_run(["cargo", "test", "--workspace", "--no-run"]), "a build is not a test run")
    check(is_test_run([*NEXTEST]) and is_test_run(["cargo", *CARGO_TEST]), "both runners are test runs")
    # An impact selection (scripts/ci_impact.py): `-p` per owning package instead of `--workspace`, and the
    # lib/bin flags only where a selected package has that kind of target (cargo refuses a filter nobody has).
    scope = {"lib_packages": ["mw-kernel"], "bin_packages": [], "owners": {"clock": ["mw-kernel"],
             "market_town": ["mw-cli"], "launcher": ["mw-cli"]}}
    scoped = commands([LIB_BINS, DOC, "launcher", "market_town"], "nextest", None, scope)
    check(scoped == [
        ["cargo", "test", "-p", "mw-cli", "-p", "mw-kernel", "--no-run", "--lib", "--test", "market_town",
         "--test", "launcher"],
        ["cargo", "nextest", "run", "-p", "mw-cli", "-p", "mw-kernel", "--profile", "ci", "--no-fail-fast", "--lib",
         "--test", "market_town"],
        ["cargo", "test", "-p", "mw-cli", "--no-fail-fast", "--test", "launcher"],
        ["cargo", "test", "-p", "mw-kernel", "--no-fail-fast", "--doc"],
    ], f"impact scope: -p per owner, no --bins without a binary: {scoped}")
    check(commands(["market_town"], "nextest", None, scope)[1][3:5] == ["-p", "mw-cli"],
          "impact scope: a test-only shard names only its owners")
    try:
        commands([LIB_BINS], "nextest", Path("archive"), scope)
        failures.append("an impact selection accepted an archive")
    except ValueError:
        pass
    try:
        commands(["x"], "pytest")
        failures.append("commands accepted an unknown runner")
    except ValueError:
        pass

    # The custom-harness set is the workspace's: a new `harness = false` target must be named (DEP-163).
    declared = custom_harness_targets()
    check(declared == set(CUSTOM_HARNESS), f"CUSTOM_HARNESS {sorted(CUSTOM_HARNESS)} != manifests' {sorted(declared)}")

    # Weights from JUnit: per-test times summed per unit; lib and bin suites are <lib+bins>.
    check(unit_of("mineworld-kernel") == LIB_BINS, "a library's suite is <lib+bins>")
    check(unit_of("mineworld-cli::bin/mineworld") == LIB_BINS, "a binary's suite is <lib+bins>")
    check(unit_of("mineworld-cli::market_town") == "market_town", "an integration test's suite is its name")
    with tempfile.TemporaryDirectory() as directory:
        junit = Path(directory) / "junit.xml"
        junit.write_text(
            '<testsuites><testsuite name="mineworld-cli::slow">'
            '<testcase name="a" time="40.4"/><testcase name="b" time="2.0"/></testsuite>'
            '<testsuite name="mineworld-kernel"><testcase name="c" time="1.0"/></testsuite>'
            '<testsuite name="mineworld-cli::bin/mineworld"><testcase name="d" time="2.5"/></testsuite>'
            "</testsuites>",
            encoding="utf-8",
        )
        weighed = weigh([junit])
        check(weighed == {"slow": 42, LIB_BINS: 4}, f"weigh sums per unit within a file: {weighed}")
        # Two shards' files hold disjoint tests: their times add up per unit across the files.
        other = Path(directory) / "junit-2.xml"
        other.write_text(
            '<testsuites><testsuite name="mineworld-cli::slow"><testcase name="e" time="10.0"/></testsuite>'
            '<testsuite name="mineworld-persistence"><testcase name="f" time="3.2"/></testsuite>'
            "</testsuites>",
            encoding="utf-8",
        )
        weighed = weigh([junit, other])
        check(weighed == {"slow": 52, LIB_BINS: 7}, f"weigh sums per unit over disjoint files: {weighed}")
        # The same test in two files is a double count (one OS's shards are disjoint): refused.
        try:
            weigh([junit, junit])
            failures.append("weigh accepted the same test identity in two files")
        except ValueError:
            pass
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
    if len(arguments) >= 2 and arguments[0] == "--weigh":
        print_weights(weigh([Path(argument) for argument in arguments[1:]]))
        return 0
    print("usage: ci_shard.py --plan N | --weigh JUNIT.xml... | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
