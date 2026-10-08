#!/usr/bin/env python3
"""Every test removes its own scratch (`docs/ENGINEERING_STANDARDS.md` §22, "Test scratch").

Before this rule a passing `cargo test --workspace` left 135 entries, about 16 GB, under
`target/tmp`: helpers removed a test's directory *before* the test and never after
(`.structured-coding/plans/mvp0/pr-test-hygiene.md` §2). Tests now make scratch through
`mineworld-test-support`'s `scratch!`, which removes it when the test ends. Two checks keep it so:

  scan    static: no test source outside `tests/support/` names `CARGO_TARGET_TMPDIR`,
          `temp_dir()` or a literal "/tmp" path — the ways a test makes scratch nobody removes.
  left    after a run: nothing is left under `<target>/tmp`, and no `mineworld-*` entry is left in
          the temporary directory. Lists every entry with its size.

Each exits non-zero and names what it found.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The helper itself is the one place that may name the scratch root.
HELPER = ROOT / "tests" / "support"
FORBIDDEN = re.compile(r'CARGO_TARGET_TMPDIR|temp_dir\(\)|"/tmp\b')

# Files that keep a scratch guard of their own, each with the reason it cannot use the helper. An entry
# is still bound by the rule's substance (a name of its own, removed on drop), and `left` still checks
# what it leaves. Keep this list short; an entry needs a reason a reviewer can verify.
EXEMPT = {
    "packages/tests/manifest.rs": (
        "mineworld-packages is a leaf, and packages/tests/structure.rs asserts that no table of its "
        "manifest whose name contains 'dependencies' names a mineworld crate, dev-dependencies "
        "included; a dev-dependency on mineworld-test-support would fail that test's assertion "
        "(pr-test-hygiene.md §7 C4)"
    ),
}
SKIPPED_DIRECTORIES = {"target", ".git", "node_modules", ".godot"}


def test_sources() -> list[Path]:
    """Every `.rs` file with a `tests` directory in its path: integration tests and their support."""
    found = []
    for directory, subdirectories, files in os.walk(ROOT):
        subdirectories[:] = [name for name in subdirectories if name not in SKIPPED_DIRECTORIES]
        here = Path(directory)
        if here == HELPER or HELPER in here.parents:
            subdirectories[:] = []
            continue
        if "tests" not in here.relative_to(ROOT).parts:
            continue
        found.extend(here / name for name in files if name.endswith(".rs"))
    return sorted(found)


def scan() -> int:
    sources = test_sources()
    if not sources:
        print("no test sources found — has the layout changed?", file=sys.stderr)
        return 2
    hits = []
    exempted = []
    for source in sources:
        relative = source.relative_to(ROOT).as_posix()
        if relative in EXEMPT:
            exempted.append(relative)
            continue
        for number, line in enumerate(source.read_text(encoding="utf-8").splitlines(), start=1):
            if line.lstrip().startswith("//"):
                continue
            if FORBIDDEN.search(line):
                hits.append(f"{source.relative_to(ROOT)}:{number}: {line.strip()}")
    if hits:
        print("test scratch made outside mineworld-test-support:", file=sys.stderr)
        for hit in hits:
            print(f"  {hit}", file=sys.stderr)
        print(
            "\nUse `mineworld_test_support::scratch!(name)` (a path to be created) or "
            "`scratch!(empty name)` (an empty directory), and hold the guard while the path is used.",
            file=sys.stderr,
        )
        return 1
    stale = sorted(set(EXEMPT) - set(exempted))
    if stale:
        print(f"exemptions naming no test source (remove them): {stale}", file=sys.stderr)
        return 1
    for relative in exempted:
        print(f"exempt: {relative} — {EXEMPT[relative]}")
    print(
        f"{len(sources)} test sources, none makes scratch outside mineworld-test-support "
        f"({len(exempted)} exempt)"
    )
    return 0


def size(path: Path) -> int:
    if path.is_symlink() or not path.is_dir():
        return path.lstat().st_size
    total = 0
    for directory, _, files in os.walk(path):
        for name in files:
            try:
                total += (Path(directory) / name).lstat().st_size
            except OSError:
                pass
    return total


def human(count: int) -> str:
    for unit in ("B", "KiB", "MiB", "GiB"):
        if count < 1024 or unit == "GiB":
            return f"{count:.0f} {unit}" if unit == "B" else f"{count:.1f} {unit}"
        count /= 1024
    return f"{count} B"


def left(target: Path, temporary: Path) -> int:
    entries = []
    scratch_root = target / "tmp"
    if scratch_root.is_dir():
        for entry in sorted(scratch_root.iterdir()):
            # A helper container is listed by the scratches in it, which carry the tests' names.
            inside = sorted(entry.iterdir()) if entry.name.startswith("mineworld-scratch-") else []
            entries += inside or [entry]
    if temporary.is_dir():
        entries += sorted(temporary.glob("mineworld-*"))
    if entries:
        total = 0
        print("scratch left behind:", file=sys.stderr)
        for entry in entries:
            bytes_ = size(entry)
            total += bytes_
            print(f"  {human(bytes_):>10}  {entry}", file=sys.stderr)
        print(f"{len(entries)} entries, {human(total)} in total", file=sys.stderr)
        return 1
    print(f"no scratch left under {scratch_root} or as {temporary}/mineworld-*")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("scan", help="static: no test makes scratch outside the helper")
    after = commands.add_parser("left", help="after a run: no scratch is left")
    after.add_argument(
        "--target-dir",
        type=Path,
        default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")),
        help="the Cargo target directory of the run (default: $CARGO_TARGET_DIR or ./target)",
    )
    after.add_argument(
        "--tmp-dir",
        type=Path,
        default=Path(tempfile.gettempdir()),
        help="the temporary directory of the run (default: the system's)",
    )
    arguments = parser.parse_args()
    if arguments.command == "scan":
        return scan()
    return left(arguments.target_dir, arguments.tmp_dir)


if __name__ == "__main__":
    sys.exit(main())
