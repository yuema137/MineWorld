#!/usr/bin/env python3
"""Release-path checks: the citation's version, and the publishing gates of `publish.yml` (S23 R-d).

Commands:
    python3 scripts/ci_release.py citation            CITATION.cff's `version` equals the workspace version
    python3 scripts/ci_release.py --self-test         the rules above on planted texts, no network

`citation` is also called by R-b's `verify-tag` on a tag run, so a tag cannot ship with a stale citation
(design `pr-s23-rd-publish.md` section 4.1). The other publishing commands (`publish-gate`, `verify-assets`,
`itch-channels`, `check-secrets`, `check-switch`) are added beside this one by the same design (section 3).

Exits non-zero and names every problem.
"""

from __future__ import annotations

import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CITATION_VERSION = re.compile(r"^version:\s*['\"]?(?P<version>[^'\"\s#]+)['\"]?\s*(?:#.*)?$", re.M)


def workspace_version(cargo_toml: str) -> str | None:
    """The version every crate inherits: `[workspace.package]` in the root `Cargo.toml`."""
    package = tomllib.loads(cargo_toml).get("workspace", {}).get("package", {})
    version = package.get("version")
    return version if isinstance(version, str) else None


def citation_version(citation: str) -> str | None:
    """The top-level `version:` of a CFF file; `authors:` and other nested keys are indented, so they never match."""
    found = CITATION_VERSION.search(citation)
    return found.group("version") if found else None


def citation_problems(citation: str, cargo_toml: str) -> list[str]:
    expected = workspace_version(cargo_toml)
    if expected is None:
        return ["Cargo.toml: [workspace.package] has no version"]
    actual = citation_version(citation)
    if actual is None:
        return ["CITATION.cff: no top-level version"]
    if actual != expected:
        return [f"CITATION.cff: version {actual} ≠ {expected}"]
    return []


def check_citation(root: Path) -> list[str]:
    return citation_problems(
        (root / "CITATION.cff").read_text(encoding="utf-8"),
        (root / "Cargo.toml").read_text(encoding="utf-8"),
    )


CARGO = '[workspace]\nmembers = ["a"]\n\n[workspace.package]\nversion = "0.2.0"\nedition = "2024"\n'


def self_test() -> list[str]:
    """Planted texts: an equal version passes; a stale one is named; a missing one is named."""
    failures = []
    cases = [
        ("equal", 'cff-version: 1.2.0\ntitle: MineWorld\nversion: 0.2.0\nauthors:\n  - family-names: Ma\n', []),
        ("stale", "cff-version: 1.2.0\nversion: 0.1.0\n", ["CITATION.cff: version 0.1.0 ≠ 0.2.0"]),
        ("quoted", "version: '0.2.0'\n", []),
        ("nested only", "authors:\n  - family-names: Ma\n    version: 0.2.0\n", ["CITATION.cff: no top-level version"]),
    ]
    for name, citation, expected in cases:
        got = citation_problems(citation, CARGO)
        if got != expected:
            failures.append(f"citation {name}: expected {expected!r}, got {got!r}")
    if citation_problems("version: 0.2.0\n", "[workspace]\n") != ["Cargo.toml: [workspace.package] has no version"]:
        failures.append("citation: a Cargo.toml without a workspace version must be named")
    return failures


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        failures = self_test()
        for failure in failures:
            print(f"ci_release self-test: FAIL: {failure}")
        print(f"ci_release self-test: {'FAILED' if failures else 'passed'}")
        return 1 if failures else 0
    if arguments == ["citation"]:
        problems = check_citation(ROOT)
        for problem in problems:
            print(f"citation: {problem}", file=sys.stderr)
        if not problems:
            print("citation: CITATION.cff version matches the workspace version")
        return 1 if problems else 0
    print("usage: ci_release.py citation | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
