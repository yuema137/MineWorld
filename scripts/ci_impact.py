#!/usr/bin/env python3
"""Which tests a pull request's change can reach: CI's per-PR impact selection.

Operator ruling of 2026-11-27 (`docs/decisions/ARC-154.md`, dated note): a pull request runs `fast` plus the
tests relevant to what it changed; the full matrix runs only at a milestone (the `milestone` label or a
milestone dispatch) and in the nightly. `scripts/ci_changes.py` owns the mode; this module owns the
selection, from the change set and the workspace's own description of itself (`cargo metadata`), so no
table of crates, worlds or tests is kept by hand. Design: `pr-ci-test-economy.md` §2.4, with the departures
recorded in `pr-ci-test-economy-contract.md` (E4).

Rules, applied to every changed path (the first that names a path decides its kind):

    docs          the audited docs set (ci_changes.is_docs): nothing.
    global        the build's roots, the CI definition and its scripts (GLOBAL_*): the whole run is `full`.
    fast-covered  a file a `fast` command names (a script and its self-test), and packaging inputs.
    package file  a path inside a Cargo package:
                    tests/<t>.rs or tests/<t>/...  -> that test target only (a stale expectation is caught);
                    a shared test module           -> every test target of the package;
                    anything else (src, manifest)  -> the package and every package that depends on it,
                                                      by any dependency kind, transitively (all targets).
    data          any other path: the Rust files that name its directory (`git grep`) select their test
                  target, and their whole package when a crate's source names it (an `include_str!`
                  fixture). Under a content root (CONTENT_ROOTS: worlds/, presentation/, ...) the search is
                  for the root's own name, and the Godot client tests and the binary-driving Python areas
                  run too. Python members, Godot clients, the removability inputs and platform-specific paths
                  set their own flags. A data path that nothing names and no flag claims, a platform file among them, is `full`.

Then every selected test must run somewhere (frozen I-1; `cover_selected`): a target whose tests need
`--ignored` adds the layer that runs them (the Godot client tests, a Python area); a changed test file whose
tests only a nightly layer runs makes the run `full`. A selection that reaches the binary's package
(`tools/cli`, which links every crate) also runs the Godot client tests and the binary-driving Python areas.
`removability` also runs when a changed line names the LM controller or the 3D client; `platforms` runs the
Rust selection natively on macOS and Windows for a platform path or a changed line that branches on the
platform. The repository-scan suite (`tests/acceptance`) joins every non-empty Rust selection.

    python3 scripts/ci_impact.py --base SHA --head SHA     print the selection of a real diff
    python3 scripts/ci_impact.py --self-test               synthetic cases (also run by ci_changes.py)

Standard library only, like ci_changes.py: it runs on the bare runner.
"""

from __future__ import annotations

import json
import math
import posixpath
import re
import subprocess
import sys
from collections.abc import Callable
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
import ci_godot  # noqa: E402  (the Godot client test targets)
import ci_layer  # noqa: E402  (the layers: fast's commands, the Python areas, the ignored-test commands)
import ci_shard  # noqa: E402  (the partition units and their weights, ARC-110)

# The build's roots and the CI definition: a change here can reach any test, so the run is full.
GLOBAL_FILES = frozenset({
    "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "Dockerfile", "deny.toml", "clippy.toml",
    ".gitattributes", ".dockerignore",
})
GLOBAL_PREFIXES = (".github/", ".cargo/", ".config/", "scripts/ci_", "scripts/test_ci_")
# Packaging inputs, whose self-test (`package.py --self-test`) runs in `fast`. A script is covered by `fast`
# only when a `fast` command names it (`fast_covered`); any other script falls through to the readers rule.
FAST_COVERED_PREFIXES = ("packaging/",)

# The Python areas (ci_layer.py PYTHON_AREA_COMMANDS, not restated here): a change under one tests that area.
PYTHON_AREAS = tuple(dict.fromkeys(area for area, _ in ci_layer.PYTHON_AREA_COMMANDS))
# The areas whose tests start the real `mineworld` binary (real_server / real_binary markers).
BINARY_AREAS = tuple(area for area in PYTHON_AREAS if area not in ci_layer.PYTHON_NO_BINARY)
PYTHON_ROOT_FILES = frozenset({"pyproject.toml", "uv.lock", ".python-version"})
# The package that builds the `mineworld` binary. Every Godot client test and every Python real_server test
# drives it, so a selection that reaches this package (any crate it links, by reverse dependency) also runs
# the Godot client tests and the binary-driving Python areas (red-team M-1).
BINARY_PACKAGE_DIR = "tools/cli"

# Content roots: data that the binary, the clients and the Python tools read by listing directories,
# `include_str!` and subprocess as well as by name. A change under one is not mapped by name (red-team B-2):
# it selects every Rust package that names the root, the Godot client tests and the binary-driving Python
# areas (and the character pipeline for presentation/).
CONTENT_ROOTS = ("worlds", "entities", "presentation", "spike", "tests/fixtures")

# The Godot clients, their launchers and what they draw (scripts/ci_godot.py GODOT_TESTS).
GODOT_PREFIXES = (
    "clients/2d/", "clients/3d-spike/", "clients/shared/", "clients/protocol/", "clients/runtime/",
    "presentation/", "tools/presentation/", "tools/far-field/",
)
GODOT_FILES = frozenset({"mineworld-2d", "mineworld-3d", "mineworld-slice"})

# IC-7 and ARC-347 (ci_removability.py): what deleting the LM controller or the 3D client touches, by path,
# and by a changed line that names them (a new reader breaks removability too; red-team M-3).
REMOVABILITY_PREFIXES = ("cognition/lm-controller/", "clients/3d-spike/", "sdk/python/", "clients/agent/")
REMOVABILITY_NAMES = ("clients/3d-spike", "lm-controller", "mineworld-slice", "mineworld-3d")

# Platform-specific: a path segment, a script kind, or a changed line that branches on the platform.
PLATFORM_SEGMENT = re.compile(r"(^|/)[^/]*(windows|macos|darwin|win32)[^/]*(/|$)", re.IGNORECASE)
PLATFORM_SUFFIXES = (".ps1", ".bat", ".cmd", ".command")
PLATFORM_PREFIXES = ("tools/launch/",)
PLATFORM_LINE = re.compile(
    r"cfg!?\([^\n]*\b(?:windows|unix|target_os|target_family|target_env|target_vendor)\b"
    r"|std::os::(?:unix|windows)|consts::OS\b|MAIN_SEPARATOR"
    r"|sys\.platform|os\.name\s*==|platform\.system\(\)|OS\.get_name\(|OS\.has_feature\("
)

# The repository-scan suite: joins every non-empty Rust selection.
SCAN_PACKAGE_DIR = "tests/acceptance"

# Seconds of selected tests per Linux shard (ARC-110 weights are Windows seconds, ~1.3x Linux): a hub change
# lands on 6 shards, like the full run, and a single crate on one.
SECONDS_PER_SHARD = 1300
MAX_SHARDS = 6


class Unknown(Exception):
    """The selection cannot be computed: the run is full."""


@dataclass
class Package:
    name: str
    directory: str  # relative to the repository root, `/`-separated, no trailing slash
    tests: dict[str, str]  # test target name -> its home prefix (`tests/x.rs` -> `tests/x`)
    has_lib: bool
    has_bin: bool
    depends_on: set[str]  # workspace packages, every dependency kind


@dataclass
class Workspace:
    packages: dict[str, Package]

    @classmethod
    def from_metadata(cls, metadata: dict) -> Workspace:
        root = Path(metadata["workspace_root"]).resolve()

        def relative(path: str) -> str:
            return Path(path).resolve().relative_to(root).as_posix()

        directories = {
            package["name"]: relative(str(Path(package["manifest_path"]).parent)) for package in metadata["packages"]
        }
        by_directory = {directory: name for name, directory in directories.items()}
        packages: dict[str, Package] = {}
        for package in metadata["packages"]:
            name, directory = package["name"], directories[package["name"]]
            tests: dict[str, str] = {}
            kinds: set[str] = set()
            for target in package["targets"]:
                kinds.update(target["kind"])
                if target["kind"] == ["test"]:
                    home = posixpath.relpath(relative(target["src_path"]), directory or ".")
                    home = home[: -len("/main.rs")] if home.endswith("/main.rs") else home.removesuffix(".rs")
                    tests[target["name"]] = home
            depends = {
                by_directory[relative(dependency["path"])]
                for dependency in package["dependencies"]
                if dependency.get("path") and relative(dependency["path"]) in by_directory
            }
            packages[name] = Package(name, directory, tests, "lib" in kinds or "proc-macro" in kinds,
                                     "bin" in kinds, depends - {name})
        return cls(packages)

    def owner(self, path: str) -> Package | None:
        """The package whose directory is the longest prefix of `path`."""
        best: Package | None = None
        for package in self.packages.values():
            if package.directory and (path.startswith(package.directory + "/")):
                if best is None or len(package.directory) > len(best.directory):
                    best = package
        return best

    def dependents(self, names: set[str]) -> set[str]:
        """`names` and every package that depends on one of them, transitively."""
        found = set(names)
        changed = True
        while changed:
            changed = False
            for package in self.packages.values():
                if package.name not in found and package.depends_on & found:
                    found.add(package.name)
                    changed = True
        return found

    def scan_package(self) -> str | None:
        return next((p.name for p in self.packages.values() if p.directory == SCAN_PACKAGE_DIR), None)


@dataclass
class Selection:
    full: list[str] = field(default_factory=list)  # why the run must be full; empty when it need not be
    packages: set[str] = field(default_factory=set)  # every target of these
    tests: dict[str, set[str]] = field(default_factory=dict)  # test target -> owning packages
    changed_tests: set[str] = field(default_factory=set)  # targets a changed test file selected (rule 1, I-1)
    python: set[str] = field(default_factory=set)
    godot: bool = False
    removability: bool = False
    platforms: bool = False
    reasons: list[str] = field(default_factory=list)

    def add_test(self, target: str, package: str) -> None:
        self.tests.setdefault(target, set()).add(package)

    def core(self, workspace: Workspace) -> dict:
        """The Rust selection as ci_shard.py reads it: partition units, and who owns each."""
        owners: dict[str, set[str]] = {}
        for name in sorted(self.packages):
            for target in workspace.packages[name].tests:
                owners.setdefault(target, set()).add(name)
        for target, names in self.tests.items():
            owners.setdefault(target, set()).update(names)
        lib = sorted(n for n in self.packages if workspace.packages[n].has_lib)
        bins = sorted(n for n in self.packages if workspace.packages[n].has_bin)
        units = ([ci_shard.LIB_BINS] if lib or bins else []) + ([ci_shard.DOC] if lib else []) + sorted(owners)
        return {
            "schema": 1,
            "units": units,
            "lib_packages": lib,
            "bin_packages": bins,
            "owners": {target: sorted(names) for target, names in sorted(owners.items())},
        }

    def shards(self, workspace: Workspace) -> list[str]:
        units = self.core(workspace)["units"]
        if not units:
            return []
        load = sum(ci_shard.WEIGHTS.get(unit, ci_shard.DEFAULT_WEIGHT) for unit in units)
        count = max(1, min(MAX_SHARDS, math.ceil(load / SECONDS_PER_SHARD)))
        return [f"{index}/{count}" for index in range(1, count + 1)]


Readers = Callable[[str], list[str]]


# A reader query that starting with WORD matches the rest as a whole word (`git grep -w`): a content root's
# readers name it in any form, `"worlds"`, `join("../../worlds")`, `concat!(…, "/../../worlds/…")`.
WORD = "word:"


def git_readers(prefix: str) -> list[str]:
    """The tracked Rust files that name `prefix` (as a whole word when it starts with WORD)."""
    word = prefix.startswith(WORD)
    needle = prefix[len(WORD):] if word else prefix
    result = subprocess.run(["git", "grep", "-l", "-F", *(["-w"] if word else []), needle, "--", "*.rs"],
                            cwd=ROOT, capture_output=True, text=True)
    if result.returncode not in (0, 1):
        raise Unknown(f"`git grep {prefix}` failed: {result.stderr.strip()}")
    return [line for line in result.stdout.splitlines() if line]


def data_prefix(path: str) -> str:
    """The directory a reader names: the first two segments of a deeper path, else the path itself. A
    specification under docs/ that is code (ci_changes.is_docs) is so because a guide page links into it:
    its reader is the guide's own test, which names docs/guide."""
    if path.startswith("docs/") and not path.startswith(("docs/guide/", "docs/reference/")):
        return "docs/guide"
    parts = path.split("/")
    return "/".join(parts[:2]) if len(parts) > 2 else path


def target_of(package: Package, inner: str) -> str | None:
    """The test target whose own file or module directory holds `inner` (a path inside the package)."""
    return next((t for t, home in package.tests.items() if inner == home + ".rs" or inner.startswith(home + "/")),
                None)


def module_users(package: Package, inner: str, readers: Readers) -> list[str]:
    """The test targets of `package` that declare the shared test module `inner` (`tests/m.rs` or
    `tests/m/...`) with `mod m;`; every test target when none is found (fail closed)."""
    module = inner.split("/")[1].removesuffix(".rs")
    users = set()
    for reader in readers(f"mod {module};"):
        if reader.startswith(package.directory + "/tests/"):
            target = target_of(package, reader[len(package.directory) + 1:])
            if target is not None:
                users.add(target)
    return sorted(users) or sorted(package.tests)


def select_in_package(selection: Selection, workspace: Workspace, package: Package, path: str,
                      readers: Readers) -> None:
    inner = path[len(package.directory) + 1:]
    if inner.startswith("tests/"):
        target = target_of(package, inner)
        if target is not None:
            selection.add_test(target, package.name)
            selection.changed_tests.add(target)
            selection.reasons.append(f"{path}: test target {package.name}::{target}")
            return
        users = module_users(package, inner, readers)
        for target in users:
            selection.add_test(target, package.name)
            selection.changed_tests.add(target)
        selection.reasons.append(f"{path}: a shared test module of {len(users)} test target(s) of {package.name}")
        return
    reached = workspace.dependents({package.name})
    selection.packages |= reached
    selection.reasons.append(f"{path}: {package.name} and {len(reached) - 1} dependent package(s)")


def select_readers(selection: Selection, workspace: Workspace, path: str, readers: Readers) -> bool:
    """Selects the Rust readers of a data path; whether any was found. A test file that names the path
    selects its target; a package's source that names it (an `include_str!` fixture, a default path, a
    launcher) selects that whole package. Under a content root, the readers are every Rust file that names
    the root itself, since its readers also list directories (red-team B-2)."""
    root = content_root(path)
    prefixes = [f"{WORD}{root}"] if root else [data_prefix(path)]
    prefix = " or ".join(prefixes)
    tests: list[tuple[str, str]] = []
    sources: set[str] = set()
    for reader in sorted({reader for query in prefixes for reader in readers(query)}):
        package = workspace.owner(reader)
        if package is None:
            continue
        inner = reader[len(package.directory) + 1:]
        target = target_of(package, inner)
        if target is not None:
            tests.append((target, package.name))
        elif inner.startswith("tests/"):
            tests += [(name, package.name) for name in module_users(package, inner, readers)]
        else:
            sources.add(package.name)
    for target, package_name in tests:
        selection.add_test(target, package_name)
    selection.packages |= sources
    if tests or sources:
        selection.reasons.append(f"{path}: {len(tests)} test target(s) and {len(sources)} package(s) whose Rust "
                                 f"files name {prefix}")
    return bool(tests or sources)


def content_root(path: str) -> str | None:
    return next((root for root in CONTENT_ROOTS if path.startswith(root + "/")), None)


def fast_covered(path: str) -> bool:
    """Whether a `fast` command names this file (a script and its self-test), or it is a packaging input."""
    if path.startswith(FAST_COVERED_PREFIXES):
        return True
    return any(path in command for command in ci_layer.LAYERS["fast"])


def ignored_coverage() -> dict[str, str]:
    """Test targets whose tests run only with `--ignored`, and where a pull request runs them: `godot` (the
    Godot client tests, ci_godot.GODOT_TESTS), a Python area (the `python` layer's ignored commands), or
    `nightly` (any other layer's; no per-PR job runs them). Read from the layers, not restated."""
    def target(command: list[str]) -> str | None:
        return command[command.index("--test") + 1] if "--ignored" in command and "--test" in command else None

    coverage = {name: "godot" for name in ci_godot.GODOT_TESTS}
    for area, command in ci_layer.PYTHON_AREA_COMMANDS:
        if target(command):
            coverage.setdefault(target(command), area)
    for commands in ci_layer.LAYERS.values():
        for command in commands:
            if target(command):
                coverage.setdefault(target(command), "nightly")
    return coverage


def cover_selected(selection: Selection, workspace: Workspace) -> None:
    """Every selected test runs somewhere (frozen I-1): a target whose tests need `--ignored` adds the layer
    that runs them; a changed test file whose layer no PR runs makes the run full. A selection that reaches
    the binary's package also runs the Godot client tests and the binary-driving Python areas (M-1)."""
    coverage = ignored_coverage()
    for target in sorted(selection.tests):
        where = coverage.get(target)
        if where == "godot":
            selection.godot = True
            selection.reasons.append(f"{target}: its ignored tests run in impact-godot")
        elif where == "nightly" and target in selection.changed_tests:
            selection.full.append(f"{target}: a changed test file whose tests only a nightly layer runs")
        elif where == "nightly":
            selection.reasons.append(f"{target}: its ignored tests run nightly and at the milestone only")
        elif where is not None:
            selection.python.add(where)
            selection.reasons.append(f"{target}: its ignored tests run in impact-python ({where})")
    binary = next((p.name for p in workspace.packages.values() if p.directory == BINARY_PACKAGE_DIR), None)
    if binary is not None and binary in selection.packages:
        selection.godot = True
        selection.python.update(BINARY_AREAS)
        selection.reasons.append(f"{binary} is selected: the Godot client tests and the Python areas that "
                                 "drive the binary run too")


def flag_data(selection: Selection, path: str) -> bool:
    """Python, Godot and removability flags of a non-package path; whether any claimed it."""
    claimed = False
    if path in PYTHON_ROOT_FILES:
        selection.python.update(PYTHON_AREAS)
        selection.reasons.append(f"{path}: every Python area")
        claimed = True
    for area in PYTHON_AREAS:
        if path.startswith(area + "/"):
            selection.python.add(area)
            selection.reasons.append(f"{path}: Python area {area}")
            claimed = True
    root = content_root(path)
    if root is not None:
        selection.godot = True
        selection.python.update(BINARY_AREAS)
        if root == "presentation":
            selection.python.add(ci_layer.PIPELINE)
        selection.reasons.append(f"{path}: content root {root}/: the Godot client tests and the Python areas "
                                 "that drive the binary")
        claimed = True
    if path.startswith(GODOT_PREFIXES) or path in GODOT_FILES:
        selection.godot = True
        selection.reasons.append(f"{path}: the Godot client tests")
        claimed = True
    if path.startswith(REMOVABILITY_PREFIXES) or path in PYTHON_ROOT_FILES:
        selection.removability = True
    return claimed


def is_platform_path(path: str) -> bool:
    return bool(PLATFORM_SEGMENT.search(path)) or path.endswith(PLATFORM_SUFFIXES) or path.startswith(
        PLATFORM_PREFIXES)


def select(paths: list[str], workspace: Workspace, readers: Readers, changed_lines: list[str],
           is_docs: Callable[[str], bool]) -> Selection:
    selection = Selection()
    rust = False
    for path in paths:
        if is_docs(path):
            continue
        if path in GLOBAL_FILES or path.startswith(GLOBAL_PREFIXES):
            selection.full.append(f"{path}: a build root or the CI definition")
            continue
        if fast_covered(path):
            selection.reasons.append(f"{path}: covered by fast")
            continue
        if is_platform_path(path):
            selection.platforms = True
            selection.reasons.append(f"{path}: platform-specific: macOS and Windows run the selection")
        if path.endswith("/Cargo.toml"):
            selection.removability = True
        package = workspace.owner(path)
        if package is not None:
            rust = True
            select_in_package(selection, workspace, package, path, readers)
            flag_data(selection, path)
            continue
        claimed = flag_data(selection, path)
        named = select_readers(selection, workspace, path, readers)
        rust = rust or named
        if not (claimed or named):
            # A platform file nothing reads (a new .ps1 or launcher) is unknown too: full, on every OS.
            selection.full.append(f"{path}: no rule names it and no Rust file reads it")
    if any(PLATFORM_LINE.search(line) for line in changed_lines):
        selection.platforms = True
        selection.reasons.append("a changed line branches on the platform: macOS and Windows run the selection")
    if any(name in line for line in changed_lines for name in REMOVABILITY_NAMES):
        selection.removability = True
        selection.reasons.append("a changed line names the LM controller or the 3D client: removability runs")
    scan = workspace.scan_package()
    if rust and scan is not None:
        selection.packages.add(scan)
    cover_selected(selection, workspace)
    return selection


# ------------------------------------------------------------------------------------- real inputs


def workspace_of(root: Path = ROOT) -> Workspace:
    """The workspace from `cargo metadata --no-deps` (no build, no network). Any failure is Unknown."""
    try:
        result = subprocess.run(["cargo", "metadata", "--no-deps", "--format-version", "1", "--offline"],
                                cwd=root, capture_output=True, text=True)
    except OSError as error:
        raise Unknown(f"cargo metadata: {error}") from error
    if result.returncode != 0:
        raise Unknown(f"cargo metadata failed: {result.stderr.strip()[:400]}")
    try:
        return Workspace.from_metadata(json.loads(result.stdout))
    except (KeyError, ValueError) as error:
        raise Unknown(f"cargo metadata unreadable: {error}") from error


def changed_lines(base: str, head: str) -> list[str]:
    """The added and removed lines of the diff's Rust and Python files."""
    result = subprocess.run(["git", "diff", "-U0", "--no-renames", f"{base}...{head}", "--", "*.rs", "*.py", "*.gd"],
                            cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        raise Unknown(f"git diff failed: {result.stderr.strip()}")
    return [line[1:] for line in result.stdout.splitlines()
            if line[:1] in "+-" and not line.startswith(("+++", "---"))]


# ------------------------------------------------------------------------------------------ self-test


def synthetic() -> Workspace:
    """A workspace of MineWorld's shape: a hub (kernel), a leaf System Pack, the installed set, the CLI
    that links everything, the server, the scan suite and a dev-only test helper."""

    def package(name, directory, tests=(), deps=(), lib=True, binary=False, homes=None):
        targets = [{"kind": ["lib"], "name": name, "src_path": f"/w/{directory}/src/lib.rs"}] if lib else []
        if binary:
            targets.append({"kind": ["bin"], "name": name, "src_path": f"/w/{directory}/src/main.rs"})
        for test in tests:
            home = (homes or {}).get(test, f"tests/{test}.rs")
            targets.append({"kind": ["test"], "name": test, "src_path": f"/w/{directory}/{home}"})
        return {"name": name, "manifest_path": f"/w/{directory}/Cargo.toml", "targets": targets,
                "dependencies": [{"name": d, "path": f"/w/{p}"} for d, p in deps] + [{"name": "serde"}]}

    return Workspace.from_metadata({"workspace_root": "/w", "packages": [
        package("mw-contracts", "contracts", ["schema"]),
        package("mw-kernel", "kernel", ["clock"], [("mw-contracts", "contracts")]),
        package("mw-economy", "systems/economy", ["ledger"], [("mw-kernel", "kernel")]),
        package("mw-weather", "systems/weather", [], [("mw-kernel", "kernel")]),
        package("mw-installed", "systems/installed", [], [("mw-economy", "systems/economy"),
                                                          ("mw-weather", "systems/weather"),
                                                          ("mw-posture", "systems/posture")]),
        package("mw-server", "server", ["handshake"], [("mw-kernel", "kernel")]),
        package("mw-cli", "tools/cli", ["market_town", "guide", "client_2d", "milestone_d", "speech_both_ways"],
                [("mw-installed", "systems/installed"), ("mw-server", "server"), ("mw-support", "tests/support")],
                lib=False, binary=True, homes={"guide": "tests/guide/main.rs"}),
        package("mw-support", "tests/support", ["scratch"]),
        package("mw-acceptance", "tests/acceptance", ["client_rules"], [("mw-kernel", "kernel")]),
        package("mw-posture", "systems/posture", ["worlds"], [("mw-kernel", "kernel")]),
    ]})


SYNTHETIC_READERS = {
    # Content roots are searched by the root's own name (B-2): an include_str! fixture in a crate's source,
    # a directory-listing test, a CLI test.
    # As git grep -w finds them: "worlds", join("../../worlds"), concat!(…, "/../../worlds/…").
    "word:worlds": ["systems/weather/src/fixture.rs", "tools/cli/tests/market_town.rs",
                    "systems/posture/tests/worlds.rs", "tools/cli/tests/guide.rs"],
    "clients/2d": ["tools/cli/tests/client_2d.rs", "tools/cli/src/launch.rs", "tools/cli/tests/godot2d/mod.rs"],
    "clients/shared": ["tests/acceptance/tests/client_rules.rs"],
    "mod godot2d;": ["tools/cli/tests/client_2d.rs"],
    "docs/guide": ["tools/cli/tests/guide/page_run.rs", "kernel/src/lib.rs"],
    "scripts/depiction_variant.py": ["tools/cli/tests/guide.rs"],
    "sdk/python": [],
}

def cases() -> list[tuple[str, list[str], list[str], dict]]:
    """(name, paths, changed lines, expected facts). Facts: full (bool), packages (exact set), tests (subset
    that must be selected), notests, absent (packages that must not be), python (exact set), python_has
    (subset), godot, removability, platforms, shards (exact count)."""
    binary = set(BINARY_AREAS)
    return [
        ("single crate: a leaf System Pack reaches the binary, so Godot and Python run (M-1)",
         ["systems/weather/src/lib.rs"], [],
         {"full": False, "packages": {"mw-weather", "mw-installed", "mw-cli", "mw-acceptance"},
          "absent": {"mw-kernel", "mw-economy", "mw-server"}, "python": binary, "godot": True, "platforms": False}),
        ("kernel: every dependent, transitively", ["kernel/src/lib.rs"], [],
         {"full": False, "packages": {"mw-kernel", "mw-economy", "mw-weather", "mw-installed", "mw-server",
                                      "mw-cli", "mw-acceptance", "mw-posture"}, "absent": {"mw-contracts", "mw-support"},
          "godot": True, "python": binary}),
        ("contracts: the clients and the SDK run (M-1)", ["contracts/src/lib.rs"], [],
         {"full": False, "godot": True, "python": binary}),
        ("one test file: that target only", ["tools/cli/tests/market_town.rs"], [],
         {"full": False, "packages": {"mw-acceptance"}, "tests": {"market_town"}, "notests": {"guide", "client_2d"},
          "godot": False, "python": set()}),
        ("B-1: a Godot test file runs the Godot client tests", ["tools/cli/tests/client_2d.rs"], [],
         {"full": False, "tests": {"client_2d"}, "godot": True, "absent": {"mw-cli"}}),
        ("B-1: an LM-only test file runs its Python area", ["tools/cli/tests/milestone_d.rs"], [],
         {"full": False, "tests": {"milestone_d"}, "python": {"cognition/lm-controller"}, "godot": False}),
        ("B-1: a test file only a nightly layer runs: full", ["tools/cli/tests/speech_both_ways.rs"], [],
         {"full": True}),
        ("a test target's module directory", ["tools/cli/tests/guide/page_run.rs"], [],
         {"full": False, "tests": {"guide"}, "notests": {"market_town"}}),
        ("a shared Godot test module: its targets and the Godot tests", ["tools/cli/tests/godot2d/mod.rs"], [],
         {"full": False, "tests": {"client_2d"}, "notests": {"market_town", "guide"}, "godot": True}),
        ("a shared test module nobody declares: every target, a nightly one among them: full",
         ["tools/cli/tests/common/mod.rs"], [], {"full": True}),
        ("a dev-only helper reaches its users", ["tests/support/src/lib.rs"], [],
         {"full": False, "packages": {"mw-support", "mw-cli", "mw-acceptance"}, "absent": {"mw-kernel"}}),
        ("python-only: the SDK", ["sdk/python/mineworld/client.py"], [],
         {"full": False, "packages": set(), "python": {"sdk/python"}, "godot": False, "removability": True,
          "shards": 0}),
        ("python-only: the lock", ["uv.lock"], [],
         {"full": False, "packages": set(), "python": set(PYTHON_AREAS)}),
        ("Godot-only: the 2D client; the CLI source that launches it reaches the binary",
         ["clients/2d/scripts/app.gd"], [],
         {"full": False, "godot": True, "tests": {"client_2d"}, "absent": {"mw-kernel"}, "platforms": False}),
        ("Godot-only: a shared client file", ["clients/shared/hud/hud.gd"], [],
         {"full": False, "godot": True, "python": set(), "absent": {"mw-cli", "mw-kernel"}}),
        ("Godot-only: a launcher", ["mineworld-slice"], [], {"full": False, "godot": True}),
        ("docs-only", ["docs/MVP.md", "README.md"], [], {"full": False, "packages": set(), "shards": 0,
                                                        "godot": False, "python": set()}),
        ("workflow change: full", [".github/workflows/ci.yml"], [], {"full": True}),
        ("a CI script: full", ["scripts/ci_layer.py"], [], {"full": True}),
        ("the lock file: full", ["Cargo.lock"], [], {"full": True}),
        ("a script fast self-tests: fast covers it", ["scripts/check_cast.py"], [],
         {"full": False, "packages": set(), "shards": 0}),
        ("M-5: a script fast does not run: its Rust readers", ["scripts/depiction_variant.py"], [],
         {"full": False, "tests": {"guide"}}),
        ("M-5: a script nothing in fast or Rust names: full", ["scripts/far_field_variant.py"], [], {"full": True}),
        ("B-2, D-1: a world file: include_str! fixtures, relative-path and directory readers, Godot, Python",
         ["worlds/market-town/configure/weather.yaml"], [],
         {"full": False, "tests": {"market_town", "worlds", "guide"}, "godot": True, "python_has": binary,
          "packages_has": {"mw-weather"}}),
        ("B-2: presentation adds the character pipeline", ["presentation/mineworld-default/3D/x.yaml"], [],
         {"full": False, "godot": True, "python_has": {"tools/character_pipeline"}}),
        ("unknown path: full", ["mystery/thing.bin"], [], {"full": True}),
        ("a specification a guide page links: the guide test", ["docs/MODULE_SPEC.md"], [],
         {"full": False, "tests": {"guide"}, "notests": {"market_town"}, "absent": {"mw-cli"}}),
        ("the server: Rust, Python drivers and Godot", ["server/src/lib.rs"], [],
         {"full": False, "packages": {"mw-server", "mw-cli", "mw-acceptance"}, "python": binary, "godot": True}),
        ("platform path in a package: macOS and Windows run", ["kernel/src/windows.rs"], [],
         {"full": False, "platforms": True}),
        ("D-2: a platform file nothing reads: full", ["tools/install.ps1"], [], {"full": True}),
        ("platform line: a cfg(windows) branch", ["kernel/src/lib.rs"], ["#[cfg(windows)]"],
         {"full": False, "platforms": True}),
        ("M-2: a nested cfg", ["kernel/src/lib.rs"], ['#[cfg(all(unix, not(target_os = "macos")))]'],
         {"platforms": True}),
        ("M-2: a platform module import", ["kernel/src/lib.rs"], ["use std::os::unix::fs::PermissionsExt;"],
         {"platforms": True}),
        ("M-2: a GDScript platform branch", ["clients/shared/hud/hud.gd"], ['if OS.get_name() == "Windows":'],
         {"platforms": True}),
        ("no platform marker: Linux only", ["kernel/src/lib.rs"], ["let x = 1;"], {"platforms": False}),
        ("M-3: a new reader of the 3D client runs removability", ["tools/cli/tests/market_town.rs"],
         ['let scene = root.join("clients/3d-spike/scenes/slice.tscn");'], {"removability": True}),
        ("a crate manifest: removability", ["systems/economy/Cargo.toml"], [],
         {"full": False, "removability": True, "packages": {"mw-economy", "mw-installed", "mw-cli",
                                                             "mw-acceptance"}}),
    ]

def self_test(is_docs: Callable[[str], bool] | None = None) -> int:
    if is_docs is None:
        import ci_changes  # beside this script; imported late: it imports this module
        is_docs = ci_changes.is_docs
    workspace = synthetic()
    failed = 0
    for name, paths, lines, expected in cases():
        got = select(paths, workspace, lambda prefix: SYNTHETIC_READERS.get(prefix, []), lines, is_docs)
        core = got.core(workspace)
        problems = []
        if "full" in expected and bool(got.full) != expected["full"]:
            problems.append(f"full={got.full}")
        if "packages" in expected and got.packages != expected["packages"]:
            problems.append(f"packages={sorted(got.packages)}")
        if not expected.get("packages_has", set()) <= got.packages:
            problems.append(f"packages={sorted(got.packages)}")
        if not expected.get("python_has", set()) <= got.python:
            problems.append(f"python={sorted(got.python)}")
        if not expected.get("absent", set()).isdisjoint(got.packages):
            problems.append(f"selected {sorted(expected['absent'] & got.packages)}")
        if not expected.get("tests", set()) <= set(core["owners"]):
            problems.append(f"tests={sorted(core['owners'])}")
        if not expected.get("notests", set()).isdisjoint(core["owners"]):
            problems.append(f"extra tests={sorted(expected['notests'] & set(core['owners']))}")
        for flag in ("python", "godot", "removability", "platforms"):
            if flag in expected and getattr(got, flag) != expected[flag]:
                problems.append(f"{flag}={getattr(got, flag)}")
        if "shards" in expected and len(got.shards(workspace)) != expected["shards"]:
            problems.append(f"shards={got.shards(workspace)}")
        failed += bool(problems)
        print(f"[self-test] {'FAIL' if problems else 'ok  '} impact: {name}{': ' + '; '.join(problems) if problems else ''}")
    # The shard count follows ARC-110's weights: a hub-sized selection takes the full run's six shards.
    heavy = Selection(tests={unit: {"mw-cli"} for unit in ci_shard.WEIGHTS if unit not in (ci_shard.DOC, ci_shard.LIB_BINS)})
    count = len(heavy.shards(workspace))
    good = count == MAX_SHARDS
    failed += not good
    print(f"[self-test] {'ok  ' if good else 'FAIL'} impact: every weighed unit takes {MAX_SHARDS} shards ({count})")
    # The partition stays complete: each shard's units together are the selection's.
    sel = select(["kernel/src/lib.rs"], workspace, lambda prefix: [], [], is_docs)
    core = sel.core(workspace)
    parts = ci_shard.partition(core["units"], 3, ci_shard.WEIGHTS)
    good = sorted(u for part in parts for u in part) == sorted(core["units"])
    failed += not good
    print(f"[self-test] {'ok  ' if good else 'FAIL'} impact: a selection partitions completely")
    total = len(cases()) + 2
    print(f"[self-test] impact {'passed' if not failed else f'FAILED: {failed} case(s)'}: {total} case(s)")
    return 1 if failed else 0


def main(arguments: list[str]) -> int:
    import ci_changes

    if arguments == ["--self-test"]:
        return self_test(ci_changes.is_docs)
    if len(arguments) == 4 and arguments[0] == "--base" and arguments[2] == "--head":
        base, head = arguments[1], arguments[3]
        paths = ci_changes.git_diff(base, head, False)
        workspace = workspace_of()
        selection = select(paths, workspace, git_readers, changed_lines(base, head), ci_changes.is_docs)
        print("\n".join(f"[impact] {reason}" for reason in selection.reasons + selection.full))
        print(json.dumps({"full": selection.full, "shards": selection.shards(workspace),
                          "python": sorted(selection.python), "godot": selection.godot,
                          "removability": selection.removability, "platforms": selection.platforms,
                          "core": selection.core(workspace)}, indent=1))
        return 0
    print("usage: ci_impact.py --base SHA --head SHA | --self-test", file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
