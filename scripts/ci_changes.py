#!/usr/bin/env python3
"""Does this change need the build? CI's `changes` job: `code=true` or `code=false` (documentation only).

The operator's requirement (2026-10-09): a documentation-only update must not run the build. The
design and the audit that fixes the docs set are step-14 §16 (`.structured-coding/plans/mvp0/`);
the decision is `docs/DECISIONS.md` ARC-48's note of 2026-10-09 for 13x. This script decides; the
workflow only reads its output.

It fails closed. A path is documentation only if it is in the audited docs set (`is_docs`): no CI
command reads it, or the only readers are the pure-Python doc checks the `docs` layer still runs.
Everything else is code, and so is every change it cannot see: an empty diff, an unknown event, a
base that is the null id (a new branch), missing from the clone, or not an ancestor (a force-push),
and any `git` failure. A push to `main` and a dispatch are code without diffing, so `main` is always
fully verified.

    python3 scripts/ci_changes.py --event E --ref R --base SHA --head SHA [--github-output FILE]
    python3 scripts/ci_changes.py --self-test

Standard library only: it runs on the bare runner, before any toolchain exists.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NULL = "0" * 40

# The audited docs set (step-14 §16.3). Widening it is a change to that table, after the same audit.
DOCS_PREFIXES = ("docs/", ".structured-coding/plans/")
DOCS_FILES = {"CLAUDE.md"}


def is_docs(path: str) -> bool:
    """Whether a changed path is in the audited docs set."""
    if path.startswith(DOCS_PREFIXES) or path in DOCS_FILES:
        return True
    # README*.md at the repository root only: subdirectory READMEs are code (some are read by tests).
    return "/" not in path and path.startswith("README") and path.endswith(".md")


class Unknown(Exception):
    """The change set cannot be computed: the verdict is code."""


def git(*arguments: str) -> str:
    result = subprocess.run(["git", *arguments], cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        raise Unknown(f"`git {' '.join(arguments)}` failed: {result.stderr.strip()}")
    return result.stdout


def git_diff(base: str, head: str, push: bool) -> list[str]:
    """The paths `base...head` changed, renames split into deletion and addition."""
    if not base or base == NULL:
        raise Unknown(f"no base ({base or 'empty'}): a new branch or an event without one")
    if not head:
        raise Unknown("no head")
    for name, sha in (("base", base), ("head", head)):
        try:
            git("cat-file", "-e", f"{sha}^{{commit}}")
        except Unknown as error:
            raise Unknown(f"{name} {sha} is not a commit in this clone") from error
    if push:
        try:
            git("merge-base", "--is-ancestor", base, head)
        except Unknown as error:
            raise Unknown(f"{base} is not an ancestor of {head}: a force-push") from error
    listed = git("diff", "--name-only", "--no-renames", "-z", f"{base}...{head}")
    return [path for path in listed.split("\0") if path]


Diff = Callable[[str, str, bool], list[str]]


def decide(event: str, ref: str, base: str, head: str, diff: Diff = git_diff) -> tuple[bool, str, list[str]]:
    """`(code, reason, paths)` for one workflow run."""
    if event == "workflow_dispatch":
        return True, "a dispatch runs everything", []
    if event == "push" and ref == "refs/heads/main":
        return True, "a push to main runs everything: main stays fully verified", []
    if event not in ("pull_request", "push"):
        return True, f"unknown event {event!r}", []
    try:
        paths = diff(base, head, event == "push")
    except Unknown as error:
        return True, f"change set unknown: {error}", []
    if not paths:
        return True, "an empty diff", []
    code = [path for path in paths if not is_docs(path)]
    if code:
        return True, f"{len(code)} of {len(paths)} changed path(s) are code", paths
    return False, f"all {len(paths)} changed path(s) are documentation", paths


# ------------------------------------------------------------------------------------------ self-test


def fixed(paths: list[str]) -> Diff:
    def diff(base: str, head: str, push: bool) -> list[str]:
        return paths

    return diff


def self_test() -> int:
    """Each case must get its stated verdict (step-14 §16.5 A-X1)."""
    pr = ("pull_request", "refs/pull/1/merge", "a" * 40, "b" * 40)
    docs = ["docs/DECISIONS.md", "docs/images/readme/hero-en-dark.svg", "docs/references/MICROVERSE_AUDIT.md",
            ".structured-coding/plans/mvp0/step-14-ci.md", "README.md", "README.zh-CN.md", "CLAUDE.md"]
    cases: list[tuple[str, tuple[str, str, str, str], Diff, bool]] = [
        ("docs-only: every entry of the docs set", pr, fixed(docs), False),
        ("docs-only: one README line", pr, fixed(["README.md"]), False),
        ("mixed: docs and a Rust file", pr, fixed(["docs/MVP.md", "kernel/src/lib.rs"]), True),
        ("code-only", pr, fixed(["systems/economy/src/lib.rs", "Cargo.lock"]), True),
        ("the workflow is code", pr, fixed([".github/workflows/ci.yml"]), True),
        ("a rename out of code (deletion + addition)", pr, fixed(["kernel/NOTES.md", "docs/NOTES.md"]), True),
        ("an empty diff", pr, fixed([]), True),
        ("a push to main, without diffing", ("push", "refs/heads/main", "a" * 40, "b" * 40), fixed(["README.md"]), True),
        ("a dispatch", ("workflow_dispatch", "refs/heads/x", "", ""), fixed(["README.md"]), True),
        ("an unknown event", ("schedule", "refs/heads/main", "", ""), fixed(["README.md"]), True),
        ("a docs-only push to a scratch branch", ("push", "refs/heads/scratch/x", "a" * 40, "b" * 40),
         fixed(["docs/MVP.md"]), False),
        ("unknown base: the null id (a new branch)", ("push", "refs/heads/scratch/x", NULL, "b" * 40), git_diff, True),
        ("unknown base: an empty base", ("pull_request", "refs/pull/1/merge", "", "b" * 40), git_diff, True),
        ("unknown base: not an object in this clone", ("pull_request", "refs/pull/1/merge", "f" * 40, "e" * 40),
         git_diff, True),
    ]
    # Files a CI command reads (step-14 §16.3): never documentation, whatever they look like.
    for path in [
        "cognition/lm-controller/README.md",  # test_provider_scan.py; pyproject `readme`
        "sdk/python/README.md",  # pyproject `readme`
        "server/PROTOCOL.md",  # client_text.rs
        "worlds/social-cafe/README.md",  # inside a World Pack
        ".structured-coding/standards.md",  # not a plan
        "systems/economy/README.md",  # a subdirectory README
    ]:
        cases.append((f"a file CI reads is code: {path}", pr, fixed([path]), True))
    failed = 0
    for name, (event, ref, base, head), diff, expected in cases:
        code, reason, _ = decide(event, ref, base, head, diff)
        good = code == expected
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}: code={str(code).lower()} ({reason})")
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}: {len(cases)} case(s)")
    return 1 if failed else 0


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    parser = argparse.ArgumentParser(description="Classify a CI run's change set as code or docs.")
    parser.add_argument("--event", required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--base", default="")
    parser.add_argument("--head", default="")
    parser.add_argument("--github-output", type=Path)
    options = parser.parse_args(arguments)
    code, reason, paths = decide(options.event, options.ref, options.base, options.head)
    for path in paths:
        print(f"[changes] {'docs' if is_docs(path) else 'code'}  {path}")
    verdict = "true" if code else "false"
    print(f"[changes] code={verdict}: {reason}")
    if options.github_output:
        with options.github_output.open("a", encoding="utf-8") as output:
            output.write(f"code={verdict}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
