#!/usr/bin/env python3
"""Flake detection: run a command several times on one commit and record what each run said.

The nightly's `repeat` group (`docs/DECISIONS.md` ARC-83; `pr-13c-nightly.md` §3.5) runs the unchanged
default suite twice more per operating system:

    python3 scripts/ci_repeat.py --times 2 --summary FILE -- cargo test --workspace --no-fail-fast

Every repetition runs, even after one failed, and nothing is retried to green (`I-13c-4`): a repetition
exists to sample. Each run's output is streamed as it comes; the repetition number is exported as
`MINEWORLD_CI_REPETITION` (no merged test reads it, `I-S13-2`; a scratch mutation may). After each run
`check_scratch.py left` must find nothing left behind. The summary records, per run, its exit status,
wall time, whether any test ran, and every failing test (`<binary>::<test>`) and failing target.

A run that failed before any test ran (a build, tool or network failure) is `inconclusive`: it says
nothing about a test, so it is never a flake candidate (F-13c-7). `classify` turns the samples of one
commit on one OS into flake candidates (failed in one sample, passed in another) and failures (failed in
every sample that ran); the nightly report uses it.

Exits non-zero if any run failed. Standard library only.
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
import time
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# `     Running tests/market_town.rs (target/debug/deps/market_town-0123abcd)`, also with `\` on Windows.
RUNNING = re.compile(r"^\s+Running \S.* \((?:.*[\\/])?(?P<binary>[^\\/]+?)-[0-9a-f]+(?:\.exe)?\)\s*$")
FAILED = re.compile(r"^test (?P<test>\S+) \.\.\. FAILED\s*$")
STARTED = re.compile(r"^running \d+ tests?\s*$")
LISTED = re.compile(r"^    (?P<test>[A-Za-z_][\w:]*)\s*$")
ANSI = re.compile(r"\x1b\[[0-9;]*m")
# `cargo test --no-fail-fast` lists failed targets at the end; a harness = false program reports here only.
TARGET = re.compile(r"^\s+`(?P<target>-p \S+ --(?:test|lib|bin|bench|doc)(?: \S+)?)`\s*$")
TARGET_ONE = re.compile(r"^error: test failed, to rerun pass `(?P<target>[^`]+)`\s*$")


@dataclass
class Sample:
    """One run of the suite: what it said, nothing inferred."""
    name: str
    status: int
    ran: bool
    failed: set[str] = field(default_factory=set)
    targets: set[str] = field(default_factory=set)
    seconds: float = 0.0
    scratch: str = "clean"

    def inconclusive(self) -> bool:
        return self.status != 0 and not self.ran


class Transcript:
    """Reads a `cargo test` transcript line by line.

    A failing test is named by its `test <name> ... FAILED` line or, because a test whose child process
    writes to the shared stderr can split that line, by the `failures:` list libtest prints before
    `test result: FAILED` (four-space-indented names, ended by a blank line)."""

    def __init__(self) -> None:
        self.binary = "?"
        self.ran = False
        self.failed: set[str] = set()
        self.targets: set[str] = set()
        self.listing: list[str] | None = None

    def read(self, line: str) -> None:
        # Cargo colours its status lines when it believes a terminal is attached (the toolchain container).
        line = ANSI.sub("", line.rstrip("\r\n"))
        if self.listing is not None:
            if listed := LISTED.match(line):
                self.listing.append(listed["test"])
                return
            self.failed.update(f"{self.binary}::{test}" for test in self.listing)
            self.listing = None
        if line == "failures:":
            self.listing = []
        elif running := RUNNING.match(line):
            self.binary = running["binary"]
        elif STARTED.match(line):
            self.ran = True
        elif failed := FAILED.match(line):
            self.failed.add(f"{self.binary}::{failed['test']}")
        elif target := TARGET.match(line) or TARGET_ONE.match(line):
            self.targets.add(target["target"])


def repeat(times: int, command: list[str]) -> list[Sample]:
    samples = []
    for repetition in range(1, times + 1):
        print(f"[repeat] run {repetition} of {times}: {' '.join(command)}", flush=True)
        transcript = Transcript()
        began = time.monotonic()
        environment = {**os.environ, "MINEWORLD_CI_REPETITION": str(repetition)}
        try:
            process = subprocess.Popen(command, cwd=ROOT, env=environment, stdout=subprocess.PIPE,
                                       stderr=subprocess.STDOUT)
            assert process.stdout is not None
            for raw in process.stdout:
                # The bytes as they came: a console code page (cp1252 on Windows) cannot encode every
                # character a test prints, and a re-encoding failure must not end the sample.
                sys.stdout.buffer.write(raw)
                sys.stdout.buffer.flush()
                transcript.read(raw.decode("utf-8", errors="replace"))
            status = process.wait()
        except OSError as error:
            print(f"[repeat] could not start {command[0]}: {error}", flush=True)
            status = 127
        sys.stdout.flush()
        sample = Sample(f"nightly-{repetition}", status, transcript.ran, transcript.failed, transcript.targets,
                        round(time.monotonic() - began, 1))
        left = subprocess.run([sys.executable, "scripts/check_scratch.py", "left", "--target-dir", "target"],
                              cwd=ROOT, capture_output=True, text=True)
        if left.returncode != 0:
            sample.scratch = "left: " + " ".join((left.stdout + left.stderr).split())[:300]
            print(f"[repeat] run {repetition}: scratch {sample.scratch}", flush=True)
        print(f"[repeat] run {repetition}: exit {status} after {sample.seconds} s; {len(sample.failed)} failed "
              f"test(s); {'tests ran' if sample.ran else 'NO TEST RAN'}", flush=True)
        samples.append(sample)
    return samples


def render(samples: list[Sample], command: list[str]) -> str:
    out = [f"# scripts/ci_repeat.py: {' '.join(command)}"]
    for sample in samples:
        out.append(f"run {sample.name} exit {sample.status} wall {sample.seconds} ran {'yes' if sample.ran else 'no'} "
                   f"scratch {sample.scratch}")
        out.extend(f"failed {sample.name} {test}" for test in sorted(sample.failed))
        out.extend(f"target {sample.name} {target}" for target in sorted(sample.targets))
    return "\n".join(out) + "\n"


def parse(text: str) -> list[Sample]:
    """The summary `render` wrote, read back (by the report, from every OS's artifact)."""
    samples: dict[str, Sample] = {}
    for line in text.splitlines():
        words = line.split(" ")
        if words[0] == "run" and len(words) >= 10:
            samples[words[1]] = Sample(words[1], int(words[3]), words[7] == "yes", seconds=float(words[5]),
                                       scratch=" ".join(words[9:]))
        elif words[0] == "failed" and words[1] in samples:
            samples[words[1]].failed.add(" ".join(words[2:]))
        elif words[0] == "target" and words[1] in samples:
            samples[words[1]].targets.add(" ".join(words[2:]))
    return list(samples.values())


@dataclass
class Classified:
    flakes: dict[str, list[str]]       # test → "failed in a of b samples (names)"
    failures: dict[str, list[str]]     # test → the samples it failed in (every sample that ran)
    inconclusive: list[str]            # samples that failed before any test ran


def classify(samples: list[Sample]) -> Classified:
    """Samples of ONE commit on ONE OS. A test that failed in some sample that ran and passed in another
    is a flake candidate; one that failed in every sample that ran is a failure. An inconclusive sample
    is never evidence about a test."""
    usable = [sample for sample in samples if not sample.inconclusive()]
    # A harness = false program names no test, only its target, so targets are judged too.
    named = {sample.name: sample.failed | {f"target {target}" for target in sample.targets} for sample in usable}
    tests = sorted(set().union(*named.values()) if usable else set())
    flakes, failures = {}, {}
    for test in tests:
        failing = [sample.name for sample in usable if test in named[sample.name]]
        if len(failing) == len(usable):
            failures[test] = failing
        else:
            flakes[test] = [f"failed in {len(failing)} of {len(usable)} samples ({', '.join(failing)})"]
    return Classified(flakes, failures, [sample.name for sample in samples if sample.inconclusive()])


# ------------------------------------------------------------------------------------------ self-test


PASSING = """\
     Running tests/market_town.rs (target/debug/deps/market_town-0a1b2c3d)

running 2 tests
test a_town_lives ... ok
test b_town_resumes ... ok

test result: ok. 2 passed; 0 failed
"""
FAILING = """\
     Running unittests src/lib.rs (target\\debug\\deps\\mineworld_kernel-99aa00ff.exe)

running 1 test
test clock::tests::ticks ... ok
     Running tests/market_town.rs (target/debug/deps/market_town-0a1b2c3d)

running 2 tests
test a_town_lives ... ok
test b_town_resumes ... FAILED

failures:
error: test failed, to rerun pass `-p mineworld-cli --test market_town`
error: 2 targets failed:
    `-p mineworld-cli --test market_town`
    `-p mineworld-persistence --test kill_and_resume`
"""
# A child process's stderr split the FAILED line; only libtest's `failures:` list names the test.
SPLIT = """\
     Running tests/client_settings.rs (target/debug/deps/client_settings-56a3f7e2)

running 2 tests
test two_d_display_settings_take_effect ... libpulse.so.0: cannot open shared object file
FAILED
test the_settings_folder_is_shared ... ok

failures:

---- two_d_display_settings_take_effect stdout ----
    indented captured output
thread 'two_d_display_settings_take_effect' panicked at tools/cli/tests/client_settings.rs:571:5:

failures:
    two_d_display_settings_take_effect

test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
"""
BUILD_ERROR = """\
   Compiling mineworld-kernel v0.1.0
error[E0425]: cannot find value `x` in this scope
error: could not compile `mineworld-kernel`
"""


def sample_of(name: str, transcript: str, status: int) -> Sample:
    reader = Transcript()
    for line in transcript.splitlines(keepends=True):
        reader.read(line)
    return Sample(name, status, reader.ran, reader.failed, reader.targets)


def self_test() -> int:
    passing = sample_of("p", PASSING, 0)
    failing = sample_of("f", FAILING, 101)
    broken = sample_of("b", BUILD_ERROR, 101)
    flaky = classify([passing, failing, sample_of("p2", PASSING, 0)])
    solid = classify([failing, sample_of("f2", FAILING, 101)])
    with_broken = classify([broken, failing])
    round_trip = parse(render([passing, failing, broken], ["cargo", "test"]))
    cases = [
        ("a passing transcript ran and failed nothing", passing.ran and not passing.failed and not passing.inconclusive()),
        ("a failing test is named with its binary", failing.failed == {"market_town::b_town_resumes"}),
        ("a split FAILED line is named from libtest's failures list",
         sample_of("s", SPLIT, 101).failed == {"client_settings::two_d_display_settings_take_effect"}),
        ("a coloured Running line still names the binary (run 38094541454)",
         sample_of("c", "\x1b[1m\x1b[92m     Running\x1b[0m tests/mn7_flake.rs (target/debug/deps/mn7_flake-59e6f8d7)\n"
                   "test mn7 ... FAILED\n", 101).failed == {"mn7_flake::mn7"}),
        ("failed targets are read, a harness = false program's included",
         failing.targets == {"-p mineworld-cli --test market_town", "-p mineworld-persistence --test kill_and_resume"}),
        ("a Windows path names the binary", "mineworld_kernel" in RUNNING.match(FAILING.splitlines()[0]).group("binary")),  # type: ignore[union-attr]
        ("a build failure is inconclusive, not a test failure", broken.inconclusive() and not broken.failed),
        ("failed in one sample, passed in two: a flake candidate, its target too",
         set(flaky.flakes) == {"market_town::b_town_resumes", "target -p mineworld-cli --test market_town",
                               "target -p mineworld-persistence --test kill_and_resume"} and not flaky.failures
         and "failed in 1 of 3 samples" in flaky.flakes["market_town::b_town_resumes"][0]),
        ("failed in every sample: a failure, not a flake",
         "market_town::b_town_resumes" in solid.failures and not solid.flakes),
        ("an inconclusive sample is no evidence",
         with_broken.inconclusive == ["b"] and "market_town::b_town_resumes" in with_broken.failures
         and not with_broken.flakes),
        ("the summary reads back", [(s.name, s.status, s.ran, s.failed) for s in round_trip]
         == [("p", 0, True, set()), ("f", 101, True, {"market_town::b_town_resumes"}), ("b", 101, False, set())]),
    ]
    failed = 0
    for name, good in cases:
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}")
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


USAGE = "usage: ci_repeat.py --times N --summary FILE -- COMMAND ... | --self-test"


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    if "--" not in arguments:
        print(USAGE, file=sys.stderr)
        return 2
    split = arguments.index("--")
    options, command = dict(zip(arguments[:split:2], arguments[1:split:2])), arguments[split + 1:]
    if split % 2 or set(options) != {"--times", "--summary"} or not command or not options["--times"].isdigit() \
            or int(options["--times"]) < 1:
        print(USAGE, file=sys.stderr)
        return 2
    samples = repeat(int(options["--times"]), command)
    summary = Path(options["--summary"])
    if not summary.is_absolute():
        summary = ROOT / summary
    summary.parent.mkdir(parents=True, exist_ok=True)
    summary.write_text(render(samples, command), encoding="utf-8", newline="\n")
    bad = [sample for sample in samples if sample.status != 0 or sample.scratch != "clean"]
    print(f"[repeat] {len(samples)} run(s): {len(bad)} failed; summary {summary}", flush=True)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
