#!/usr/bin/env python3
"""No specification may number two sections the same.

Companion to `check_decision_ids.py`, and it exists for the same reason: a duplicate was found by
a reader rather than by a check. `docs/CORE_CONCEPTS.md` carried two sections numbered `## 6.1` —
the spatial frame, and `Location` — and the frame section is the one `space.gd`, `ADOPTION.md` and
the client protocol all cite as "§6.1". An author following that citation found two candidates and
no way to tell which was meant. `docs/VISION.md` had a second `## 2.5` that nobody had noticed at
all.

Specifications in this repository are cited by section number across documents and from code
comments (`CLAUDE.md` §2.1). A number naming two sections makes those citations unresolvable,
which is the contradiction §2.1(4) forbids leaving unrecorded.

Exits non-zero and names every collision with its line numbers.
"""

from __future__ import annotations

import re
import sys
from collections import defaultdict
from pathlib import Path

# A numbered heading at any depth: "## 6.1 Title", "### 2.10.3 Title".
HEADING = re.compile(r"^#{2,6}\s+(\d+(?:\.\d+)+)\s+(.*)$")


def main() -> int:
    root = Path(__file__).resolve().parent.parent
    documents = sorted(root.glob("docs/**/*.md"))
    if not documents:
        print("no documents found under docs/ — has the layout changed?", file=sys.stderr)
        return 2

    failures = 0
    checked = 0
    for document in documents:
        seen: dict[str, list[tuple[int, str]]] = defaultdict(list)
        for number, line in enumerate(document.read_text(encoding="utf-8").splitlines(), start=1):
            if match := HEADING.match(line):
                seen[match.group(1)].append((number, match.group(2).strip()))
        checked += len(seen)

        for section in sorted(duplicate for duplicate, places in seen.items() if len(places) > 1):
            failures += 1
            relative = document.relative_to(root)
            print(f"{relative}: section {section} is used twice:", file=sys.stderr)
            for line_number, title in seen[section]:
                print(f"  {relative}:{line_number}  {title}", file=sys.stderr)

    if failures:
        print(
            "\nRenumber the later section and move every citation of it in the same commit. "
            "A citation is not only in another document — code comments carry them too.",
            file=sys.stderr,
        )
        return 1

    print(f"{checked} numbered sections across {len(documents)} documents, none duplicated")
    return 0


if __name__ == "__main__":
    sys.exit(main())
