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

Since the operator's CI policy of 2026-10-18 (`docs/DECISIONS.md` ARC-154) it also decides *how much* a
code change runs, as one `mode`:

    docs       documentation only: the `docs` layer, as above.
    component  an ordinary pull request: `fast` only. Its tests are deferred to the merge train that will
               carry it; the required gates pass with a note saying so.
    targeted   a merge-train PR (head branch `train/*`) or a PR labelled `ci`: `fast`, plus the test
               layers the diff affects, on Linux only. Rust packages are the changed ones and every
               workspace package that depends on them (`cargo metadata`, reverse dependencies, dev and
               build dependencies included); Python when a Python tree or the binary its tests drive
               changed; the far-field job when a client changed.
    full       a push to `main`, a dispatch, a scratch branch, or a PR labelled `full-ci`: every layer on
               every operating system.

Targeted runs fail closed as well: a path that belongs to no known class, a `cargo metadata` failure, or
a change set that cannot be computed makes the run `full`. Paths that many tests read (the workflow and
scripts, the lock and toolchain, `worlds/`, `entities/`, `spike/`) are known, and run every Linux layer.

    python3 scripts/ci_changes.py --event E --ref R --base SHA --head SHA [--head-ref BRANCH]
                                  [--labels a,b] [--github-output FILE]
    python3 scripts/ci_changes.py --self-test

Standard library only: it runs on the bare runner, before any toolchain exists (`cargo metadata` is run
only for a targeted run, and the runner's rustup installs the pinned toolchain for it).
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import ci_shard  # noqa: E402  (beside this script; the weights that size a targeted run's shards)

ROOT = Path(__file__).resolve().parent.parent
NULL = "0" * 40

# The audited docs set (step-14 §16.3). Widening it is a change to that table, after the same audit.
DOCS_PREFIXES = ("docs/", ".structured-coding/plans/")
DOCS_FILES = {"CLAUDE.md"}

# Generated output under docs/ that a test regenerates and compares (`docs/DECISIONS.md` ARC-90): a hand
# edit there is exactly what `mineworld reference --check` exists to catch, so it runs the build.
GENERATED_PREFIXES = ("docs/reference/",)


def is_docs(path: str) -> bool:
    """Whether a changed path is in the audited docs set."""
    if path.startswith(GENERATED_PREFIXES):
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


# ------------------------------------------------------------------------- how much a run runs (ARC-154)

# The labels and the branch prefix that select a mode (the operator's CI policy of 2026-10-18).
FULL_LABEL = "full-ci"
TARGETED_LABEL = "ci"
TRAIN_PREFIX = "train/"

FULL_SHARDS = ["1/3", "2/3", "3/3"]
FULL_PYTHON_OS = ["ubuntu-24.04", "windows-2025"]
# A targeted run's shard count by its units' summed ARC-110 weight (seconds on windows-2025, the slowest
# OS): one job up to this many seconds, two up to twice it, else three, as the full run has.
SHARD_SECONDS = 900

# Read by so many tests, scripts and jobs that the affected set is "everything on Linux": the CI machinery
# itself, the build's inputs, and the content trees the Rust tests load by relative path (an audit of
# `git grep` over *.rs on 2026-10-18: worlds/ is read by ten crates, spike/ by eleven).
GLOBAL_PREFIXES = (".github/", ".cargo/", "scripts/", "worlds/", "entities/", "spike/", ".claude/")
GLOBAL_FILES = {
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "Dockerfile", ".dockerignore", "deny.toml",
    "clippy.toml", ".gitattributes", ".gitignore", ".structured-coding/standards.md", "LICENSE", "NOTICE",
}
# The CI machinery also defines the far-field job, so a change to it runs that job too.
GLOBAL_CLIENTS = (".github/", "scripts/ci_")

# The Python workspace and the trees the `python` layer checks (ci_layer.py PYTHON_MEMBERS, PIPELINE,
# ASSET_GENERATION). No Rust test reads them (the same audit), except cognition/lm-controller (below).
PYTHON_PREFIXES = ("sdk/python/", "clients/agent/", "tools/character_pipeline/", "tools/asset_generation/")
# The uv workspace's own files.
PYTHON_WORKSPACE = {"pyproject.toml", "uv.lock", ".python-version"}
# cognition/lm-controller is Python, and tools/cli reads it (milestone_d.rs, reference/pages.rs).
LM_CONTROLLER = "cognition/lm-controller/"
LM_CONTROLLER_READERS = ("tools/cli",)
# The packages whose tests read the client trees and the Presentation Pack by relative path (the same
# audit: client_rules.rs, client_text.rs, the client_* tests of tools/cli, far_field.rs, social_cafe.rs).
CLIENT_READERS = ("tools/cli", "tests/acceptance", "packages", "worldpack")
# Generated reference pages that tools/cli's `reference` test regenerates and compares (ARC-90).
GENERATED_READERS = ("tools/cli",)
# The cast checks of the `python` layer read the Presentation Pack's characters.
PRESENTATION = "presentation/"
# The binary the Python tests drive (the `python` layer builds it; its real_server tests start it).
BINARY_PACKAGE = "mineworld-cli"


@dataclass
class Workspace:
    """The Rust workspace as `cargo metadata --no-deps` describes it: each package's directory and the
    workspace packages it depends on by path (normal, dev and build dependencies alike)."""

    directories: dict[str, str]  # package name -> directory relative to the repository root
    dependencies: dict[str, set[str]]

    @staticmethod
    def of(metadata: dict) -> Workspace:
        root = metadata["workspace_root"]
        directories: dict[str, str] = {}
        for package in metadata["packages"]:
            relative = os.path.relpath(os.path.dirname(package["manifest_path"]), root)
            directories[package["name"]] = relative.replace(os.sep, "/")
        by_directory = {directory: name for name, directory in directories.items()}
        dependencies: dict[str, set[str]] = {}
        for package in metadata["packages"]:
            named: set[str] = set()
            for dependency in package["dependencies"]:
                if "path" not in dependency:
                    continue
                relative = os.path.relpath(dependency["path"], root).replace(os.sep, "/")
                if relative in by_directory:
                    named.add(by_directory[relative])
            dependencies[package["name"]] = named
        return Workspace(directories, dependencies)

    def owner(self, path: str) -> str | None:
        """The package whose directory holds `path` (the deepest one), or None."""
        best: tuple[int, str] | None = None
        for name, directory in self.directories.items():
            if path.startswith(directory + "/") and (best is None or len(directory) > best[0]):
                best = (len(directory), name)
        return best[1] if best else None

    def named(self, directory: str) -> str:
        for name, candidate in self.directories.items():
            if candidate == directory:
                return name
        raise Unknown(f"no workspace package in {directory}/")

    def dependents(self, changed: set[str]) -> set[str]:
        """`changed` and every package that depends on one of them, transitively."""
        reverse: dict[str, set[str]] = {name: set() for name in self.directories}
        for name, needs in self.dependencies.items():
            for needed in needs:
                reverse.setdefault(needed, set()).add(name)
        seen = set(changed)
        frontier = list(changed)
        while frontier:
            for dependent in reverse.get(frontier.pop(), set()):
                if dependent not in seen:
                    seen.add(dependent)
                    frontier.append(dependent)
        return seen


def cargo_metadata() -> dict:
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=ROOT, capture_output=True, text=True,
        )
    except OSError as error:
        raise Unknown(f"cargo metadata: {error}") from error
    if result.returncode != 0:
        raise Unknown(f"cargo metadata failed: {result.stderr.strip()[-400:]}")
    return json.loads(result.stdout)


Metadata = Callable[[], dict]


@dataclass
class Plan:
    """What one run runs: the workflow reads these fields as the `changes` job's outputs."""

    mode: str  # docs | component | targeted | full
    reason: str
    rust: str = "none"  # none | some | all
    packages: list[str] = field(default_factory=list)
    shards: list[str] = field(default_factory=list)
    python: bool = False
    # IC-7's removability check is a required check (operator ruling 2026-10-18): it runs for real on
    # every targeted and full run, and passes with the deferral note otherwise.
    removability: bool = False
    clients: bool = False
    why: list[str] = field(default_factory=list)  # one line per path class, for the log

    def note(self) -> str:
        """The line the required gates print when they pass without their shards."""
        if self.mode == "docs":
            return "documentation only: no test layer runs (ARC-48's note of 2026-10-09)"
        if self.mode == "component":
            return "tests deferred to a merge train (operator CI policy 2026-10-18, docs/DECISIONS.md ARC-154)"
        if self.mode == "targeted" and self.rust == "none":
            return "targeted run: nothing affected in the Rust workspace (operator CI policy 2026-10-18, ARC-154)"
        return ""

    def removability_note(self) -> str:
        if self.mode == "docs":
            return self.note()
        if self.mode == "component":
            return "removability deferred to the merge train (operator CI policy 2026-10-18, docs/DECISIONS.md ARC-154)"
        return ""

    def outputs(self) -> dict[str, str]:
        python_os = FULL_PYTHON_OS if self.mode == "full" else ["ubuntu-24.04"] if self.python else []
        return {
            "mode": self.mode,
            "rust": self.rust,
            "packages": ",".join(self.packages),
            # Never an empty list: a matrix over `[]` is an error even in a job its `if:` skips.
            "shards": json.dumps(self.shards or ["none"]),
            "python": str(self.python).lower(),
            "python_os": json.dumps(python_os or ["none"]),
            "removability": str(self.removability).lower(),
            "farfield": str(self.clients).lower(),
            "note": self.note(),
            "removability_note": self.removability_note(),
        }


def full(reason: str, paths: list[str], scratch: bool = False) -> Plan:
    """Every layer on every OS. The far-field job is one of them, except on a scratch branch, which keeps
    its path rule (RL-e0): a mutation push there need not render the clients."""
    return Plan("full", reason, rust="all", shards=FULL_SHARDS, python=True, removability=True,
                clients=clients(True, paths) if scratch else True)


def affected(paths: list[str], metadata: Metadata) -> Plan:
    """A targeted run's plan for a code change set; `full` for anything it cannot place."""
    try:
        workspace = Workspace.of(metadata())
    except (Unknown, KeyError, ValueError) as error:
        return full(f"targeted, but the workspace is unknown ({error}): everything runs", paths)
    every = set(workspace.directories)
    changed: set[str] = set()  # packages whose own files changed
    readers: set[str] = set()  # packages whose tests read a changed non-Rust file
    rust_all = python = far = False
    why: list[str] = []
    try:
        for path in paths:
            if is_docs(path):
                continue
            if (touches_clients(path) and not path.startswith(PYTHON_PREFIXES)) or path.startswith(GLOBAL_CLIENTS):
                far = True
            if path.startswith(GLOBAL_PREFIXES) or path in GLOBAL_FILES:
                rust_all = python = True
                why.append(f"global   {path}: every Linux layer")
            elif path in PYTHON_WORKSPACE:
                python = True
                why.append(f"python   {path}: the uv workspace")
            elif path.startswith(GENERATED_PREFIXES):
                readers |= {workspace.named(directory) for directory in GENERATED_READERS}
                why.append(f"reader   {path}: generated, compared by tools/cli")
            elif path.startswith(PYTHON_PREFIXES):
                python = True
                why.append(f"python   {path}")
            elif path.startswith(LM_CONTROLLER):
                python = True
                readers |= {workspace.named(directory) for directory in LM_CONTROLLER_READERS}
                why.append(f"python   {path}: and its Rust readers")
            elif touches_clients(path):
                python = python or path.startswith(PRESENTATION)
                owner = workspace.owner(path)
                readers |= {workspace.named(directory) for directory in CLIENT_READERS}
                changed |= {owner} if owner else set()
                why.append(f"client   {path}: the far field and the Rust tests that read clients")
            elif (owner := workspace.owner(path)) is not None:
                changed.add(owner)
                why.append(f"package  {path}: {owner}")
            else:
                return full(f"targeted, but {path} belongs to no known class: everything runs", paths)
    except Unknown as error:
        return full(f"targeted, but {error}: everything runs", paths)
    packages = every if rust_all else workspace.dependents(changed | readers)
    # The binary changed when its own sources or a dependency's did; a reader's tests changing is not it.
    python = python or rust_all or BINARY_PACKAGE in workspace.dependents(changed)
    plan = Plan("targeted", "", python=python, removability=True, clients=far, why=why)
    if not packages:
        plan.reason = "targeted: no Rust package affected"
    elif packages >= every:
        plan.rust, plan.shards = "all", FULL_SHARDS
        plan.reason = f"targeted: every Rust package ({len(every)}) is affected, on Linux"
    else:
        plan.rust, plan.packages = "some", sorted(packages)
        try:
            units = ci_shard.units_of(metadata(), plan.packages)
        except (KeyError, ValueError) as error:
            return full(f"targeted, but the units are unknown ({error}): everything runs", paths)
        count = min(3, 1 + ci_shard.weight_of(units) // SHARD_SECONDS)
        plan.shards = [f"{index}/{count}" for index in range(1, count + 1)]
        plan.reason = (f"targeted: {len(packages)} of {len(every)} Rust packages affected, "
                       f"{len(units)} units, {len(plan.shards)} shard(s), on Linux")
    return plan


def plan_of(event: str, ref: str, base: str, head: str, head_ref: str = "", labels: tuple[str, ...] = (),
            diff: Diff = git_diff, metadata: Metadata = cargo_metadata) -> tuple[bool, Plan, list[str]]:
    """`(code, plan, paths)` for one workflow run."""
    code, reason, paths = decide(event, ref, base, head, diff)
    if not code:
        return False, Plan("docs", reason), paths
    if event != "pull_request":
        return True, full(reason, paths, scratch=event == "push" and ref != "refs/heads/main"), paths
    if FULL_LABEL in labels:
        return True, full(f"labelled {FULL_LABEL}: the full matrix ({reason})", paths), paths
    if not paths:
        return True, full(reason, paths), paths
    if head_ref.startswith(TRAIN_PREFIX) or TARGETED_LABEL in labels:
        why = f"a merge train ({head_ref})" if head_ref.startswith(TRAIN_PREFIX) else f"labelled {TARGETED_LABEL}"
        plan = affected(paths, metadata)
        plan.reason = f"{why}; {plan.reason}"
        return True, plan, paths
    return True, Plan("component", f"a component PR: fast only ({reason})"), paths


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
    planned = plan_self_test()
    failed += planned[0]
    total = len(cases) + planned[1]
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}: {total} case(s)")
    return 1 if failed else 0


def synthetic_metadata() -> dict:
    """A small workspace shaped like the real one: contracts under everything, a kernel above it, a
    leaf crate nothing depends on, and the binary on top."""

    def package(name: str, directory: str, kinds: list[str], tests: list[str], needs: list[str]) -> dict:
        targets = [{"kind": [kind], "name": name} for kind in kinds] + [{"kind": ["test"], "name": t} for t in tests]
        dependencies = [{"name": need, "path": f"/w/{need}"} for need in needs]
        dependencies.append({"name": "serde"})  # a registry dependency: no path, never a workspace edge
        return {"name": name, "manifest_path": f"/w/{directory}/Cargo.toml", "targets": targets,
                "dependencies": dependencies}

    return {"workspace_root": "/w", "packages": [
        package("mineworld-contracts", "contracts", ["lib"], ["identity"], []),
        package("mineworld-kernel", "kernel", ["lib"], ["world"], ["contracts"]),
        package("mineworld-economy", "systems/economy", ["lib"], ["economy"], ["kernel"]),
        package("mineworld-packages", "packages", ["lib"], ["far_field"], ["contracts"]),
        package("mineworld-worldpack", "worldpack", ["lib"], ["social_cafe"], ["kernel", "packages"]),
        package("mineworld-acceptance", "tests/acceptance", ["lib"], ["client_rules"], ["kernel", "worldpack"]),
        package("mineworld-cli", "tools/cli", ["bin"], ["market_town", "run", "town_bodies"],
                ["kernel", "systems/economy", "worldpack", "packages"]),
        package("mineworld-weather-fetch", "tools/weather-fetch", ["lib", "bin"], ["record"], ["contracts"]),
    ]}


def plan_self_test() -> tuple[int, int]:
    """The modes and the targeted classifier (ARC-154): each case must get its stated plan."""
    metadata = synthetic_metadata
    train = ("pull_request", "refs/pull/9/merge", "a" * 40, "b" * 40, "train/lane-x", ())

    def broken() -> dict:
        raise Unknown("cargo metadata failed: no toolchain")

    above_kernel = ["mineworld-acceptance", "mineworld-cli", "mineworld-economy", "mineworld-kernel",
                    "mineworld-worldpack"]
    client_readers = ["mineworld-acceptance", "mineworld-cli", "mineworld-packages", "mineworld-worldpack"]
    # (name, (event, ref, base, head, head_ref, labels), diff, metadata, expected fields)
    cases: list[tuple[str, tuple, Diff, Metadata, dict]] = [
        ("kernel: it and every package above it, not contracts or the leaf; Python (the binary)",
         train, fixed(["kernel/src/lib.rs"]), metadata,
         {"mode": "targeted", "rust": "some", "packages": above_kernel, "python": True, "clients": False,
          "shards": ["1/2", "2/2"], "removability": True}),
        ("contracts: under everything, so every package",
         train, fixed(["contracts/src/ids.rs"]), metadata,
         {"mode": "targeted", "rust": "all", "shards": FULL_SHARDS, "python": True}),
        ("a leaf crate: itself only, one shard, no Python",
         train, fixed(["tools/weather-fetch/src/main.rs"]), metadata,
         {"mode": "targeted", "rust": "some", "packages": ["mineworld-weather-fetch"], "shards": ["1/1"],
          "python": False, "clients": False}),
        ("Python only: no Rust package, the Python layer, test passes as nothing affected",
         train, fixed(["sdk/python/src/mineworld/client.py", "clients/agent/agent.py"]), metadata,
         {"mode": "targeted", "rust": "none", "packages": [], "python": True, "clients": False,
          "note": "nothing affected"}),
        ("cognition/lm-controller: Python, and tools/cli which reads it",
         train, fixed(["cognition/lm-controller/src/x.py"]), metadata,
         {"mode": "targeted", "rust": "some", "packages": ["mineworld-cli"], "python": True}),
        ("a client only: the far field, and the Rust tests that read clients/",
         train, fixed(["clients/2d/scripts/app.gd"]), metadata,
         {"mode": "targeted", "rust": "some", "packages": client_readers, "python": False, "clients": True}),
        ("the Presentation Pack: also the Python cast checks",
         train, fixed(["presentation/mineworld-default/3D/characters/casting.yaml"]), metadata,
         {"mode": "targeted", "python": True, "clients": True}),
        ("the workflow: every Linux layer, the far field included",
         train, fixed([".github/workflows/ci.yml"]), metadata,
         {"mode": "targeted", "rust": "all", "python": True, "clients": True}),
        ("generated reference pages: tools/cli, which compares them",
         train, fixed(["docs/reference/sections.md"]), metadata,
         {"mode": "targeted", "rust": "some", "packages": ["mineworld-cli"], "python": False}),
        ("worlds/: read by many crates, so every Linux layer",
         train, fixed(["worlds/social-cafe/world.yaml"]), metadata, {"mode": "targeted", "rust": "all"}),
        ("documentation only in a train: docs",
         train, fixed(["docs/MVP.md", "README.md"]), metadata, {"mode": "docs", "rust": "none"}),
        ("undecidable: a path in no known class runs everything",
         train, fixed(["kernel/src/lib.rs", "newtree/thing.txt"]), metadata,
         {"mode": "full", "rust": "all", "shards": FULL_SHARDS}),
        ("undecidable: cargo metadata failing runs everything",
         train, fixed(["kernel/src/lib.rs"]), broken, {"mode": "full", "rust": "all"}),
        ("undecidable: an unknown change set runs everything",
         ("pull_request", "refs/pull/9/merge", "", "b" * 40, "train/x", ()), git_diff, metadata,
         {"mode": "full"}),
        ("a component PR (no train, no label): fast only, tests deferred",
         ("pull_request", "refs/pull/9/merge", "a" * 40, "b" * 40, "feat/x", ()), fixed(["kernel/src/lib.rs"]),
         broken, {"mode": "component", "rust": "none", "python": False, "removability": False,
                  "note": "deferred to a merge train"}),
        ("a docs-only component PR: docs, as before",
         ("pull_request", "refs/pull/9/merge", "a" * 40, "b" * 40, "docs/x", ()), fixed(["docs/MVP.md"]),
         broken, {"mode": "docs"}),
        ("labelled ci: targeted on that one PR",
         ("pull_request", "refs/pull/9/merge", "a" * 40, "b" * 40, "feat/x", ("bug", "ci")),
         fixed(["tools/weather-fetch/src/main.rs"]), metadata, {"mode": "targeted", "rust": "some"}),
        ("labelled full-ci: the full matrix, even in a train",
         ("pull_request", "refs/pull/9/merge", "a" * 40, "b" * 40, "train/x", ("full-ci",)),
         fixed(["tools/weather-fetch/src/main.rs"]), broken, {"mode": "full", "rust": "all", "removability": True}),
        ("a push to main: the full matrix",
         ("push", "refs/heads/main", "a" * 40, "b" * 40, "", ()), fixed(["README.md"]), broken,
         {"mode": "full", "clients": True}),
        ("a dispatch: the full matrix",
         ("workflow_dispatch", "refs/heads/x", "", "", "", ()), fixed([]), broken, {"mode": "full"}),
        ("a scratch push: the full matrix, as before",
         ("push", "refs/heads/scratch/x", "a" * 40, "b" * 40, "", ()), fixed(["kernel/src/lib.rs"]), broken,
         {"mode": "full", "clients": False}),
    ]
    failed = 0
    for name, (event, ref, base, head, head_ref, labels), diff, meta, expected in cases:
        _, plan, _ = plan_of(event, ref, base, head, head_ref, labels, diff, meta)
        wrong = []
        for key, value in expected.items():
            actual = plan.note() if key == "note" else getattr(plan, key)
            if (key == "note" and value not in actual) or (key != "note" and actual != value):
                wrong.append(f"{key}={actual!r}, expected {value!r}")
        failed += bool(wrong)
        print(f"[self-test] {'FAIL' if wrong else 'ok  '} plan: {name}: mode={plan.mode} rust={plan.rust}"
              f"{' (' + '; '.join(wrong) + ')' if wrong else ''}")
    # Every output parses, and a matrix is never empty.
    for mode_plan in (Plan("component", ""), full("", []), affected(["sdk/python/x.py"], metadata)):
        outputs = mode_plan.outputs()
        good = json.loads(outputs["shards"]) and json.loads(outputs["python_os"]) and "\n" not in outputs["note"]
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} outputs: {mode_plan.mode} has non-empty matrices")
    return failed, len(cases) + 3


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    parser = argparse.ArgumentParser(description="Classify a CI run's change set as code or docs.")
    parser.add_argument("--event", required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--base", default="")
    parser.add_argument("--head", default="")
    parser.add_argument("--head-ref", default="", help="a pull request's head branch (train/* is a merge train)")
    parser.add_argument("--labels", default="", help="a pull request's labels, comma-separated")
    parser.add_argument("--github-output", type=Path)
    options = parser.parse_args(arguments)
    labels = tuple(label.strip() for label in options.labels.split(",") if label.strip())
    code, plan, paths = plan_of(options.event, options.ref, options.base, options.head, options.head_ref, labels)
    for path in paths:
        print(f"[changes] {'docs' if is_docs(path) else 'code'}  {path}")
    for line in plan.why:
        print(f"[changes] {line}")
    verdict = "true" if code else "false"
    outputs = {"code": verdict, "clients": str(plan.clients).lower(), **plan.outputs()}
    print(f"[changes] mode={plan.mode}: {plan.reason}")
    for key, value in outputs.items():
        print(f"[changes] {key}={value}")
    if options.github_output:
        with options.github_output.open("a", encoding="utf-8") as output:
            for key, value in outputs.items():
                output.write(f"{key}={value}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
