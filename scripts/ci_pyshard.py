#!/usr/bin/env python3
"""Partition the `python` layer's tests into N shards: deterministic, complete, and balanced.

`docs/decisions/ARC-110.md` (note of 2026-10-27). The `python` layer outgrew one job: the LM controller's
tests alone ran 46 min on Linux and more than 63 min on Windows, where the job hit its 75-minute cap before
they finished. So the `python` job runs `scripts/ci_layer.py python --shard K/N`, and `ci_layer.py` asks this
module which units shard K holds. The partition rule is `core`'s (`ci_shard.partition`: longest processing
time first, ties by name), with this module's weights.

There are two kinds of unit:

- a *group*: one of `ci_layer.py`'s named command groups of the `python` layer (the SDK's tests, the agent
  library's, the character pipeline's checks, a Rust test that starts the cognition process, ...). Its
  commands run whole, in the order the unsharded layer runs them;
- an *LM test file*: one `test_*.py` file of `cognition/lm-controller/tests`. A shard's LM files run as one
  `pytest` command, so the member's own `pyproject.toml` (and its `addopts`, the network guard among them) is
  still the configuration pytest reads (D-P5-11).

The LM files are listed at run time, so a test file added after the weights were measured is still in exactly
one shard (weight `DEFAULT_WEIGHT`); a stale weight is ignored.

    python3 scripts/ci_pyshard.py --plan N      print each shard's units and expected seconds
    python3 scripts/ci_pyshard.py --self-test   check the partition rule
"""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import ci_shard  # noqa: E402  (beside this script; the partition rule of ARC-110)

LM_TESTS = "cognition/lm-controller/tests"

# Seconds per unit. See the ARC-110 note of 2026-10-27 for where they were measured; only the proportions
# matter for balance. Units under 3 s are left to DEFAULT_WEIGHT.
WEIGHTS: dict[str, int] = {}
DEFAULT_WEIGHT = 1


def lm_units(root: Path = ROOT) -> list[str]:
    """Every LM test file, as a path from the repository root with `/` separators."""
    return sorted(path.relative_to(root).as_posix() for path in (root / LM_TESTS).glob("test_*.py"))


def shard_units(text: str, groups: list[str], root: Path = ROOT) -> tuple[list[str], str]:
    """Shard K/N's units (or K,L/N's, together), in the order `groups` lists the groups and then the LM files
    by name, and a one-line account of the whole partition for the log."""
    indices, count = ci_shard.parse_shard(text)
    units = [*groups, *lm_units(root)]
    shards = ci_shard.partition(units, count, WEIGHTS)
    loads = [sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard) for shard in shards]
    account = ", ".join(
        f"{number}/{count}: {len(shard)} units ~{load} s" for number, (shard, load) in enumerate(zip(shards, loads), 1)
    )
    chosen = {unit for index in indices for unit in shards[index - 1]}
    ordered = [unit for unit in units if unit in chosen]
    return ordered, f"partition by ARC-110 python weights: {account}"


def plan(count: int, groups: list[str]) -> int:
    shards = ci_shard.partition([*groups, *lm_units()], count, WEIGHTS)
    for number, shard in enumerate(shards, 1):
        load = sum(WEIGHTS.get(unit, DEFAULT_WEIGHT) for unit in shard)
        print(f"shard {number}/{count}: ~{load} s, {len(shard)} units: {' '.join(shard)}")
    return 0


def self_test() -> int:
    failures: list[str] = []

    def check(condition: bool, message: str) -> None:
        if not condition:
            failures.append(message)

    lm = lm_units()
    check(len(lm) > 0 and all(unit.startswith(f"{LM_TESTS}/test_") for unit in lm), f"lm_units: {lm[:3]}")
    groups = ["sdk/python", "clients/agent", "pipeline"]
    for count in (1, 2, 3, 6):
        seen: list[str] = []
        for number in range(1, count + 1):
            units, account = shard_units(f"{number}/{count}", groups)
            seen += units
            check(account.count("/") >= count, f"{count} shards: the account names every shard: {account}")
        check(sorted(seen) == sorted([*groups, *lm]), f"{count} shards: every unit exactly once")
        together, _ = shard_units(",".join(str(n) for n in range(1, count + 1)) + f"/{count}", groups)
        check(together == [*groups, *lm], f"{count} shards: all of them together are the layer, in its order")

    for failure in failures:
        print(f"ci_pyshard self-test: FAIL: {failure}")
    print(f"ci_pyshard self-test: {'FAILED' if failures else 'passed'}")
    return 1 if failures else 0


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    if len(arguments) == 2 and arguments[0] == "--plan" and arguments[1].isdigit() and int(arguments[1]) >= 1:
        import ci_layer  # noqa: PLC0415  (the groups are ci_layer.py's; imported here only, never at load)

        return plan(int(arguments[1]), list(ci_layer.PYTHON_GROUPS))
    print("usage: ci_pyshard.py --plan N | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
