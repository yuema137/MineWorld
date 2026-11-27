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
    fast-covered  other scripts and packaging inputs: their self-tests already run in `fast`.
    package file  a path inside a Cargo package:
                    tests/<t>.rs or tests/<t>/...  -> that test target only (a stale expectation is caught);
                    a shared test module           -> every test target of the package;
                    anything else (src, manifest)  -> the package and every package that depends on it,
                                                      by any dependency kind, transitively (all targets).
    data          any other path: the Rust files that name its directory (`git grep`) select their test
                  target, or their whole package; Python members, Godot clients, the removability inputs
                  and platform-specific paths set their own flags. A data path that nothing names and no
                  flag claims is unknown: `full` (fail closed).

Beyond the Rust selection: `python` lists the Python areas to test (a member that changed, or every member
that drives the real binary when `server/` or `tools/cli/src` changed); `godot` runs the Godot client tests
on Linux; `removability` runs IC-7 and ARC-347; `platforms` runs the same Rust selection natively on macOS
and Windows, which otherwise stay on `fast` (Linux only, design §2.4). The repository-scan suite
(`tests/acceptance`) joins every non-empty Rust selection: it is cheap and reads the whole tree.

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
import ci_shard  # noqa: E402  (the partition units and their weights, ARC-110)

# The build's roots and the CI definition: a change here can reach any test, so the run is full.
GLOBAL_FILES = frozenset({
    "Cargo.lock", "Cargo.toml", "rust-toolchain.toml", "Dockerfile", "deny.toml", "clippy.toml",
    ".gitattributes", ".dockerignore",
})
GLOBAL_PREFIXES = (".github/", ".cargo/", ".config/", "scripts/ci_", "scripts/test_ci_")
# Inputs whose own checks run in `fast` (the scripts' self-tests, the packaging self-test).
FAST_COVERED_PREFIXES = ("scripts/", "packaging/")

# The Python workspace's members and tools (ci_layer.py PYTHON_MEMBERS, PYTHON_TOOLS, PIPELINE,
# ASSET_GENERATION): a change under one tests that area.
PYTHON_AREAS = (
    "sdk/python", "cognition/lm-controller", "clients/agent", "tools/onboarding", "tools/worldgen",
    "tools/character_pipeline", "tools/asset_generation",
)
# The areas whose tests start the real `mineworld` binary (real_server / real_binary markers).
BINARY_AREAS = ("sdk/python", "cognition/lm-controller", "clients/agent", "tools/onboarding", "tools/worldgen")
PYTHON_ROOT_FILES = frozenset({"pyproject.toml", "uv.lock", ".python-version"})
# Directories of the packages the Python tests drive through the binary: a direct change runs BINARY_AREAS.
PYTHON_DRIVEN_DIRS = ("server/", "tools/cli/src/")

# The Godot clients, their launchers and what they draw (scripts/ci_godot.py GODOT_TESTS).
GODOT_PREFIXES = (
    "clients/2d/", "clients/3d-spike/", "clients/shared/", "clients/protocol/", "clients/runtime/",
    "presentation/", "tools/presentation/", "tools/far-field/",
)
GODOT_FILES = frozenset({"mineworld-2d", "mineworld-3d", "mineworld-slice"})
# The server the clients talk to: a direct change runs the Godot tests too.
GODOT_DRIVEN_DIRS = ("server/",)

# IC-7 and ARC-347 (ci_removability.py): what deleting the LM controller or the 3D client touches.
REMOVABILITY_PREFIXES = ("cognition/lm-controller/", "clients/3d-spike/", "sdk/python/", "clients/agent/")

# Platform-specific: a path segment, a script kind, or a changed line that branches on the platform.
PLATFORM_SEGMENT = re.compile(r"(^|/)[^/]*(windows|macos|darwin|win32)[^/]*(/|$)", re.IGNORECASE)
PLATFORM_SUFFIXES = (".ps1", ".bat", ".cmd", ".command")
PLATFORM_PREFIXES = ("tools/launch/",)
PLATFORM_LINE = re.compile(
    r"cfg!?\(\s*(?:not\(\s*)?(?:windows|unix|target_os|target_family|target_vendor)"
    r"|sys\.platform|os\.name\s*==|platform\.system\(\)"
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


def git_readers(prefix: str) -> list[str]:
    """The tracked Rust files that name `prefix`."""
    result = subprocess.run(["git", "grep", "-l", "-F", prefix, "--", "*.rs"], cwd=ROOT,
                            capture_output=True, text=True)
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
            selection.reasons.append(f"{path}: test target {package.name}::{target}")
            return
        users = module_users(package, inner, readers)
        for target in users:
            selection.add_test(target, package.name)
        selection.reasons.append(f"{path}: a shared test module of {len(users)} test target(s) of {package.name}")
        return
    reached = workspace.dependents({package.name})
    selection.packages |= reached
    selection.reasons.append(f"{path}: {package.name} and {len(reached) - 1} dependent package(s)")


def select_readers(selection: Selection, workspace: Workspace, path: str, readers: Readers) -> bool:
    """Selects the Rust readers of a data path; whether any was found. A test file that names the path is
    the reader that exercises it; a package's source that names it (a default path, a launcher, a doc
    comment) selects that whole package only when no test file names the path."""
    prefix = data_prefix(path)
    tests: list[tuple[str, str]] = []
    sources: set[str] = set()
    for reader in readers(prefix):
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
    if not tests:
        selection.packages |= sources
    if tests or sources:
        kind = "test files" if tests else "package sources"
        selection.reasons.append(f"{path}: the Rust {kind} that name {prefix}")
    return bool(tests or sources)


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
        if path.startswith(FAST_COVERED_PREFIXES):
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
            if path.startswith(PYTHON_DRIVEN_DIRS):
                selection.python.update(BINARY_AREAS)
                selection.reasons.append(f"{path}: the Python areas that drive the binary")
            if path.startswith(GODOT_DRIVEN_DIRS):
                selection.godot = True
            flag_data(selection, path)
            continue
        claimed = flag_data(selection, path)
        named = select_readers(selection, workspace, path, readers)
        rust = rust or named
        if not (claimed or named or is_platform_path(path)):
            selection.full.append(f"{path}: no rule names it and no Rust file reads it")
    if any(PLATFORM_LINE.search(line) for line in changed_lines):
        selection.platforms = True
        selection.reasons.append("a changed line branches on the platform: macOS and Windows run the selection")
    scan = workspace.scan_package()
    if rust and scan is not None:
        selection.packages.add(scan)
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
    result = subprocess.run(["git", "diff", "-U0", "--no-renames", f"{base}...{head}", "--", "*.rs", "*.py"],
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
                                                          ("mw-weather", "systems/weather")]),
        package("mw-server", "server", ["handshake"], [("mw-kernel", "kernel")]),
        package("mw-cli", "tools/cli", ["market_town", "guide", "client_2d"],
                [("mw-installed", "systems/installed"), ("mw-server", "server"), ("mw-support", "tests/support")],
                lib=False, binary=True, homes={"guide": "tests/guide/main.rs"}),
        package("mw-support", "tests/support", ["scratch"]),
        package("mw-acceptance", "tests/acceptance", ["client_rules"], [("mw-kernel", "kernel")]),
    ]})


SYNTHETIC_READERS = {
    "worlds/social-cafe": ["tools/cli/tests/market_town.rs", "tests/acceptance/tests/client_rules.rs"],
    "clients/2d": ["tools/cli/tests/client_2d.rs", "tools/cli/src/launch.rs", "tools/cli/tests/godot2d/mod.rs"],
    "mod godot2d;": ["tools/cli/tests/client_2d.rs"],
    "docs/guide": ["tools/cli/tests/guide/page_run.rs", "kernel/src/lib.rs"],
    "spike/server": ["server/src/lib.rs"],
    "sdk/python": [],
}


def cases() -> list[tuple[str, list[str], list[str], dict]]:
    """(name, paths, changed lines, expected facts). Facts: full (bool), packages (exact set), tests (subset
    that must be selected), absent (packages that must not be), python, godot, removability, platforms,
    shards (exact count)."""
    cli = {"mw-cli"}
    return [
        ("single crate: a leaf System Pack's source", ["systems/weather/src/lib.rs"], [],
         {"full": False, "packages": {"mw-weather", "mw-installed", "mw-cli", "mw-acceptance"},
          "absent": {"mw-kernel", "mw-economy", "mw-server"}, "python": set(), "godot": False, "platforms": False}),
        ("kernel: every dependent, transitively", ["kernel/src/lib.rs"], [],
         {"full": False, "packages": {"mw-kernel", "mw-economy", "mw-weather", "mw-installed", "mw-server",
                                      "mw-cli", "mw-acceptance"}, "absent": {"mw-contracts", "mw-support"}}),
        ("one test file: that target only", ["tools/cli/tests/market_town.rs"], [],
         {"full": False, "packages": {"mw-acceptance"}, "tests": {"market_town"}, "notests": {"guide", "client_2d"}}),
        ("a test target's module directory", ["tools/cli/tests/guide/page_run.rs"], [],
         {"full": False, "tests": {"guide"}, "notests": {"market_town"}}),
        ("a shared test module: the targets that declare it", ["tools/cli/tests/godot2d/mod.rs"], [],
         {"full": False, "tests": {"client_2d"}, "notests": {"market_town", "guide"}}),
        ("a shared test module nobody declares: every target", ["tools/cli/tests/common/mod.rs"], [],
         {"full": False, "tests": {"client_2d", "market_town", "guide"}}),
        ("a dev-only helper reaches its users", ["tests/support/src/lib.rs"], [],
         {"full": False, "packages": {"mw-support", "mw-cli", "mw-acceptance"}, "absent": {"mw-kernel"}}),
        ("python-only: the SDK", ["sdk/python/mineworld/client.py"], [],
         {"full": False, "packages": set(), "python": {"sdk/python"}, "godot": False, "removability": True,
          "shards": 0}),
        ("python-only: the lock", ["uv.lock"], [],
         {"full": False, "packages": set(), "python": set(PYTHON_AREAS)}),
        ("Godot-only: the 2D client", ["clients/2d/scripts/app.gd"], [],
         {"full": False, "godot": True, "python": set(), "tests": {"client_2d"}, "absent": {"mw-kernel", "mw-cli"},
          "platforms": False}),
        ("Godot-only: a launcher", ["mineworld-slice"], [], {"full": False, "godot": True}),
        ("docs-only", ["docs/MVP.md", "README.md"], [], {"full": False, "packages": set(), "shards": 0,
                                                        "godot": False, "python": set()}),
        ("workflow change: full", [".github/workflows/ci.yml"], [], {"full": True}),
        ("a CI script: full", ["scripts/ci_layer.py"], [], {"full": True}),
        ("the lock file: full", ["Cargo.lock"], [], {"full": True}),
        ("another script: fast covers it", ["scripts/check_cast.py"], [],
         {"full": False, "packages": set(), "shards": 0}),
        ("a world: its readers", ["worlds/social-cafe/world.yaml"], [],
         {"full": False, "tests": {"market_town", "client_rules"}, "notests": {"guide"}}),
        ("unknown path: full", ["mystery/thing.bin"], [], {"full": True}),
        ("a specification a guide page links: the guide test", ["docs/MODULE_SPEC.md"], [],
         {"full": False, "tests": {"guide"}, "notests": {"market_town"}, "absent": {"mw-kernel", "mw-cli"}}),
        ("data only a package's source names: that package", ["spike/server/run.sh"], [],
         {"full": False, "packages": {"mw-server", "mw-acceptance"}}),
        ("the server: Rust, Python drivers and Godot", ["server/src/lib.rs"], [],
         {"full": False, "packages": {"mw-server", "mw-cli", "mw-acceptance"}, "python": set(BINARY_AREAS),
          "godot": True}),
        ("platform path: macOS and Windows run", ["tools/launch/src/windows.rs"], [],
         {"full": False, "platforms": True}),
        ("platform line: a cfg(windows) branch", ["kernel/src/lib.rs"], ["#[cfg(windows)]"],
         {"full": False, "platforms": True}),
        ("a crate manifest: removability", ["systems/economy/Cargo.toml"], [],
         {"full": False, "removability": True, "packages": {"mw-economy", "mw-installed", "mw-cli",
                                                             "mw-acceptance"}}),
        ("cli tests only stay narrow", ["tools/cli/tests/client_2d.rs"], [], {"absent": cli}),
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
