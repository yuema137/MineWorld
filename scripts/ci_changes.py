#!/usr/bin/env python3
"""Does this change need the build? CI's `changes` job: `code=true` or `code=false` (documentation only),
and `clients=true|false`: whether the per-PR far-field job runs (RL-e0 Q-FF-5; `touches_clients`).

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

Since the operator's policy of 2026-10-18 (`docs/DECISIONS.md` ARC-154, speed first) it also decides
whether a code change runs its tests now or at the milestone, as one `mode`:

    docs      documentation only: the `docs` layer, as above.
    deferred  every pull request and every push to `main`: `fast` only. The test layers run at the
              milestone; the required gates pass with a note saying so.
    full      a dispatch with `milestone=true`, a PR labelled `milestone`, or a scratch branch: every
              layer on every operating system.

A deferred run is the default, and it fails closed in the direction that matters: a change set this
script cannot compute is still code, so `fast` (never the `docs` layer) runs on it.

    python3 scripts/ci_changes.py --event E --ref R --base SHA --head SHA [--labels a,b]
                                  [--milestone true|false] [--github-output FILE]
    python3 scripts/ci_changes.py --self-test

Standard library only: it runs on the bare runner, before any toolchain exists.
"""

from __future__ import annotations

import argparse
import posixpath
import re
import subprocess
import sys
from collections.abc import Callable
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
NULL = "0" * 40

# The audited docs set (step-14 §16.3). Widening it is a change to that table, after the same audit.
DOCS_PREFIXES = ("docs/", ".structured-coding/plans/")
DOCS_FILES = {"CLAUDE.md"}

# Files under docs/ that a test reads, so a change to them runs the build (`docs/DECISIONS.md` ARC-90):
# the generated reference, which `mineworld reference --check` compares (a hand edit there is exactly
# what it exists to catch), and the guide, whose every page `tools/cli/tests/guide.rs` executes
# against the binary (`docs/GUIDE_FORMAT.md`): a guide page that stops working must turn CI red.
GENERATED_PREFIXES = ("docs/reference/", "docs/guide/")

# A guide page's links are checked by the guide test too (`docs/GUIDE_FORMAT.md` §§7, 8.3): a heading renamed in a
# specification the guide links into, such as `docs/CORE_CONCEPTS.md`, breaks a guide link. So every file a guide
# page links to is read by a test, and a change to it runs the build (finding F-25d-2, fixed in PR 25e). The set is
# read from the guide pages in this checkout, so a new link is covered the moment it is written.
GUIDE = ROOT / "docs" / "guide"
LINK = re.compile(r"\]\(([^)\s#]+)(?:#[^)\s]*)?\)")
_guide_targets: set[str] | None = None


def guide_link_targets() -> set[str]:
    """Every repository file a guide page or mirror links to, as a path relative to the repository root."""
    global _guide_targets
    if _guide_targets is None:
        targets: set[str] = set()
        for page in sorted(GUIDE.rglob("*.md")):
            folder = page.parent.relative_to(ROOT).as_posix()
            for target in LINK.findall(page.read_text(encoding="utf-8")):
                if "://" in target or target.startswith("mailto:"):
                    continue
                targets.add(posixpath.normpath(posixpath.join(folder, target)))
        _guide_targets = targets
    return _guide_targets


def is_docs(path: str) -> bool:
    """Whether a changed path is in the audited docs set."""
    if path.startswith(GENERATED_PREFIXES) or path in guide_link_targets():
        return False
    if path.startswith(DOCS_PREFIXES) or path in DOCS_FILES:
        return True
    # README*.md at the repository root only: subdirectory READMEs are code (some are read by tests).
    return "/" not in path and path.startswith("README") and path.endswith(".md")


CLIENT_PREFIXES = ("clients/", "presentation/")
CLIENT_FILES = {"mineworld-2d", "mineworld-slice", "tools/cli/tests/client_far_field.rs"}


def touches_clients(path: str) -> bool:
    """Whether a changed path is one the far-field checks render (RL-e0 Q-FF-5): a client, a
    Presentation Pack, a launcher, or the checks' own harness."""
    return path.startswith(CLIENT_PREFIXES) or path in CLIENT_FILES


def clients(code: bool, paths: list[str]) -> bool:
    """Whether the per-PR far-field job runs. Fails closed like `code`: a run that is code without a
    known change set (a dispatch, a push to main, any unknown) runs it."""
    if not code:
        return False
    if not paths:
        return True
    return any(touches_clients(path) for path in paths)


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


# ------------------------------------------------------------------- now or at the milestone (ARC-154)

MILESTONE_LABEL = "milestone"
DEFERRED_NOTE = "deferred to milestone (operator policy 2026-10-18, docs/DECISIONS.md ARC-154)"
DOCS_NOTE = "documentation only: no test layer runs (ARC-48's note of 2026-10-09)"


def mode_of(event: str, ref: str, code: bool, labels: tuple[str, ...] = (), milestone: bool = False) -> str:
    """`docs`, `deferred` or `full` for one run."""
    if not code:
        return "docs"
    if event == "workflow_dispatch" and milestone:
        return "full"
    if event == "pull_request" and MILESTONE_LABEL in labels:
        return "full"
    # A scratch branch exists to show a mutation turning CI red (step-14 §9.0 R-4): it runs everything.
    if event == "push" and ref.startswith("refs/heads/scratch/"):
        return "full"
    return "deferred"


def note_of(mode: str) -> str:
    return {"docs": DOCS_NOTE, "deferred": DEFERRED_NOTE}.get(mode, "")


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
        "docs/reference/sections.md",  # generated: tools/cli/tests/reference.rs compares it (ARC-90)
        "docs/reference/schema/world.schema.json",  # generated, likewise
        "docs/guide/02-your-first-world.md",  # executed: tools/cli/tests/guide.rs runs every page
        "docs/guide/examples/corner-shop/world.yaml",  # compared by the guide's `equals`
        "docs/CORE_CONCEPTS.md",  # a guide page links its headings: the guide test checks them (F-25d-2)
        "docs/MODULE_SPEC.md",  # likewise
    ]:
        cases.append((f"a file CI reads is code: {path}", pr, fixed([path]), True))
    failed = 0
    for name, (event, ref, base, head), diff, expected in cases:
        code, reason, _ = decide(event, ref, base, head, diff)
        good = code == expected
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}: code={str(code).lower()} ({reason})")
    # The per-PR far-field job (RL-e0 Q-FF-5): which change sets run it.
    far: list[tuple[str, tuple[str, str, str, str], Diff, bool]] = [
        ("farfield: a client script", pr, fixed(["clients/2d/scripts/app.gd"]), True),
        ("farfield: a Presentation Pack's far field", pr,
         fixed(["presentation/mineworld-default/3D/far_field/social-cafe.yaml"]), True),
        ("farfield: a launcher", pr, fixed(["kernel/src/lib.rs", "mineworld-slice"]), True),
        ("farfield: the checks' harness", pr, fixed(["tools/cli/tests/client_far_field.rs"]), True),
        ("farfield: Rust only does not run it", pr, fixed(["kernel/src/lib.rs", "Cargo.lock"]), False),
        ("farfield: documentation only does not run it", pr, fixed(["docs/MVP.md"]), False),
        ("farfield: a dispatch runs it", ("workflow_dispatch", "refs/heads/x", "", ""), fixed([]), True),
        ("farfield: an unknown change set runs it", ("pull_request", "refs/pull/1/merge", "", "b" * 40),
         git_diff, True),
    ]
    for name, (event, ref, base, head), diff, expected in far:
        code, _, paths = decide(event, ref, base, head, diff)
        runs = clients(code, paths)
        good = runs == expected
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}: clients={str(runs).lower()}")
    cases += far
    # The mode (ARC-154): tests run at the milestone, never per PR or per merge.
    modes: list[tuple[str, tuple[str, str, str, str], Diff, tuple[str, ...], bool, str]] = [
        ("a code PR is deferred", pr, fixed(["kernel/src/lib.rs"]), (), False, "deferred"),
        ("a docs-only PR is docs", pr, fixed(["docs/MVP.md"]), (), False, "docs"),
        ("a PR labelled milestone runs everything", pr, fixed(["kernel/src/lib.rs"]), ("bug", "milestone"), False,
         "full"),
        ("a docs-only PR labelled milestone is still docs", pr, fixed(["docs/MVP.md"]), ("milestone",), False, "docs"),
        ("a push to main is deferred", ("push", "refs/heads/main", "a" * 40, "b" * 40), fixed([]), (), False,
         "deferred"),
        ("a plain dispatch is deferred", ("workflow_dispatch", "refs/heads/main", "", ""), fixed([]), (), False,
         "deferred"),
        ("a milestone dispatch runs everything", ("workflow_dispatch", "refs/heads/main", "", ""), fixed([]), (),
         True, "full"),
        ("milestone=true outside a dispatch is ignored", pr, fixed(["kernel/src/lib.rs"]), (), True, "deferred"),
        ("a scratch push runs everything", ("push", "refs/heads/scratch/x", "a" * 40, "b" * 40),
         fixed(["kernel/src/lib.rs"]), (), False, "full"),
        ("an unknown change set is code, so deferred (fast runs, not docs)",
         ("pull_request", "refs/pull/1/merge", "", "b" * 40), git_diff, (), False, "deferred"),
    ]
    for name, (event, ref, base, head), diff, labels, milestone, expected in modes:
        code, _, _ = decide(event, ref, base, head, diff)
        mode = mode_of(event, ref, code, labels, milestone)
        good = mode == expected and (note_of(mode) != "") == (mode != "full")
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} mode: {name}: mode={mode}")
    cases += modes  # type: ignore[arg-type]
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
    parser.add_argument("--labels", default="", help="a pull request's labels, comma-separated")
    parser.add_argument("--milestone", default="false", help="a dispatch's `milestone` input")
    parser.add_argument("--github-output", type=Path)
    options = parser.parse_args(arguments)
    code, reason, paths = decide(options.event, options.ref, options.base, options.head)
    labels = tuple(label.strip() for label in options.labels.split(",") if label.strip())
    mode = mode_of(options.event, options.ref, code, labels, options.milestone == "true")
    for path in paths:
        print(f"[changes] {'docs' if is_docs(path) else 'code'}  {path}")
    verdict = "true" if code else "false"
    far = "true" if clients(code, paths) else "false"
    print(f"[changes] code={verdict}: {reason}")
    print(f"[changes] clients={far}: the far-field job {'runs' if far == 'true' and mode == 'full' else 'is skipped'}")
    print(f"[changes] mode={mode}{': ' + note_of(mode) if note_of(mode) else ': every layer on every OS'}")
    if options.github_output:
        with options.github_output.open("a", encoding="utf-8") as output:
            output.write(f"code={verdict}\n")
            output.write(f"clients={far}\n")
            output.write(f"mode={mode}\n")
            output.write(f"note={note_of(mode)}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
