#!/usr/bin/env python3
"""Every decision in `docs/DECISIONS.md` must have an identifier nobody else uses.

This exists because the log silently accepted a duplicate. Two agents branching from one
`main` each allocated the next free `ARC-` number, their entries landed in different regions
of the file, and `git merge` reported no conflict — so `main` gained two decisions with one
identifier and every reference to either became ambiguous. Reading `main` before allocating
is not a defence: nothing serialises the allocation, so two branches can read the same `main`
minutes apart and still collide.

A duplicate is not a formatting complaint. `CLAUDE.md` §2.1 makes `DECISIONS.md` the authority
a specification cites by identifier; an identifier that names two decisions makes the citation
unresolvable, which is the defect §2.1(4) describes.

Exits non-zero and names every collision, so the answer is a red check rather than a reader
noticing.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

HEADING = re.compile(r"^## ((?:DEP|ARC)-\d+)\b\s*(?:—\s*(.*))?$")


def main() -> int:
    log = Path(__file__).resolve().parent.parent / "docs" / "DECISIONS.md"
    if not log.is_file():
        print(f"{log}: not found", file=sys.stderr)
        return 2

    seen: dict[str, list[tuple[int, str]]] = defaultdict(list)
    for number, line in enumerate(log.read_text(encoding="utf-8").splitlines(), start=1):
        if match := HEADING.match(line):
            seen[match.group(1)].append((number, (match.group(2) or "").strip()))

    if not seen:
        print(f"{log}: no decision headings found — has the format changed?", file=sys.stderr)
        return 2

    duplicates = {key: places for key, places in seen.items() if len(places) > 1}
    if not duplicates:
        print(f"{len(seen)} decision ids, all distinct")
        return 0

    for key in sorted(duplicates):
        print(f"{key} names {len(duplicates[key])} different decisions:", file=sys.stderr)
        for line_number, title in duplicates[key]:
            print(f"  docs/DECISIONS.md:{line_number}  {title}", file=sys.stderr)
    print(
        "\nThe earlier entry keeps the id; the later one takes the next free number, and every "
        "citation of it moves with it.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
