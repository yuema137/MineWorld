#!/usr/bin/env python3
"""The onboarding cold-start check's commands (`docs/GUIDE_FORMAT.md` §4.7, `docs/decisions/ARC-362.md`,
`.structured-coding/plans/mvp0/pr-25g-cold-start.md`).

`.github/workflows/cold-start.yml` names these subcommands and writes no check command inline (I-S13-9's
spirit). Every value from an input or a branch arrives through the environment or an argument, never through
an expression interpolated into a script.

    python3 scripts/ci_cold_start.py clean
        Removes the tools the hosted image preinstalls that the guide's chapter 01 installs itself: Rust
        (`rustup self uninstall -y`, then any `cargo`, `rustc` or `rustup` still on PATH, and ~/.cargo and
        ~/.rustup) and Git LFS (the package, every `git-lfs` on PATH and, on Windows, Git for Windows' own
        copy; the `filter.lfs.*` configuration). Nothing else is touched. MUTATION=keep-rust or keep-lfs skips
        one removal; MUTATION=godot-on-path puts a dummy `godot` on PATH (pr-25g §9 AD-3 … AD-5).
    python3 scripts/ci_cold_start.py assert-clean --layer a|b
        Proves absence by behaviour: `cargo`, `rustc`, `rustup`, `godot` and `uv` are not found, `git lfs
        version` fails, ~/.cargo and ~/.rustup do not exist, and (layer b) no MineWorld clone is in the home
        folder. Prints what the image keeps by design (the C toolchain, git, Python). Exits 1 naming each
        tool still present.
    python3 scripts/ci_cold_start.py config --layer a|b --phase guide|substitutes --harness DIR --out FILE
            [--bundle DIR] [--godot PATH]
        Writes the TOML the ignored test `cold_start` reads (`MINEWORLD_COLD_START`, §4.7). Environment:
        RUNNER_TEMP, GITHUB_REF (a ref other than refs/heads/main is PRE-MERGE), MUTATION.
    python3 scripts/ci_cold_start.py harness --build-json FILE --dest DIR --commit SHA
        The helper job's output: the `guide` test executable named in cargo's JSON messages, docs/guide,
        scripts/, the committed page mutations and COMMIT, in one folder the clean jobs download.
    python3 scripts/ci_cold_start.py lint
        The workflow's own rules: no cache, checkout or setup action in the clean jobs, no pull_request
        trigger, no input expanded inside a script.
    python3 scripts/ci_cold_start.py install-uv
        Installs uv by the vendor's documented command, at the version CI pins (UV_VERSION), for chapter
        14's substitutes only; run after the guide phase, so it cannot hide a gap in the pages.
    python3 scripts/ci_cold_start.py find-bundle --github-output FILE
        Layer B's release run: RELEASE_RUN_ID if given, else the newest successful run of the release
        workflow on main (`gh run list`). Writes `run-id=` (empty when there is none).
    python3 scripts/ci_cold_start.py unpack --archive-dir DIR --dest DIR --github-output FILE
        Unpacks the one bundle archive of DIR as a player would, into DEST, and writes `bundle=` (the
        unpacked folder holding runtime/).
    python3 scripts/ci_cold_start.py summary --report DIR --layer a|b --started EPOCH --guide-started EPOCH
            --budget-minutes N
        Renders report/guide.json and report/substitutes.json as the job summary's table (pr-25g §7.3),
        appends it to GITHUB_STEP_SUMMARY, and exits 1 when the zero-to-Corner-Shop time (clean start → end
        of page 05) exceeds the budget (layer a only).
    python3 scripts/ci_cold_start.py verdicts --dir DIR
        The `cold-start` job: one row per leg from each leg's verdict file; exits 1 if any leg is not PASS.
    python3 scripts/ci_cold_start.py --self-test

Standard library only.
"""

from __future__ import annotations

import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKFLOW = ROOT / ".github" / "workflows" / "cold-start.yml"

UV_VERSION = "0.12.5"  # the version ci.yml's setup-uv pins
RELEASE_WORKFLOW = "release.yml"
BUILD_PAGE = "01-get-mineworld"
CORNER_SHOP_PAGE = "05-one-shop"
LAYER_B_PAGES = [
    "02-your-first-world",
    "03-two-rooms",
    "04-three-people",
    "05-one-shop",
    "06-play-it",
]
ABSENT_COMMANDS = ["cargo", "rustc", "rustup", "godot", "uv"]
RUNNER_MUTATIONS = {"keep-rust", "keep-lfs", "godot-on-path", "wrong-binary"}
KEPT_BY_DESIGN = (
    "the C toolchain (Visual Studio C++ / Xcode Command Line Tools / gcc and clang), git, Python, "
    "Homebrew, PowerShell"
)
NOT_COVERED = (
    "06.3 on Windows; 06.3 from the bundle; 14.8; 14.9; any live model (INV-S25-6)"
)
CLEAN_JOBS = ("cold-start-a", "cold-start-b")


def windows() -> bool:
    return os.name == "nt"


def home() -> Path:
    return Path(
        os.environ.get("USERPROFILE" if windows() else "HOME", str(Path.home()))
    )


def run(arguments: list[str], check: bool = False) -> subprocess.CompletedProcess[str]:
    print("$ " + " ".join(arguments), flush=True)
    return subprocess.run(arguments, check=check, text=True)


def mutation() -> str:
    name = os.environ.get("MUTATION", "")
    if name and name not in RUNNER_MUTATIONS and not page_mutation_known(name):
        raise SystemExit(f"unknown mutation {name!r}")
    return name


def page_mutation_known(name: str) -> bool:
    text = (ROOT / "tools/cli/tests/guide/cold_start_mutations.toml").read_text(
        encoding="utf-8"
    )
    return re.search(rf"^\[{re.escape(name)}\]$", text, re.MULTILINE) is not None


# ── clean ─────────────────────────────────────────────────────────────────────────────────────────────────


def on_path(name: str) -> list[Path]:
    """Every file named `name` (with Windows' executable suffixes) on PATH."""
    found = []
    suffixes = [""] + (
        os.environ.get("PATHEXT", ".EXE").lower().split(";") if windows() else []
    )
    for folder in os.environ.get("PATH", "").split(os.pathsep):
        for suffix in suffixes:
            candidate = Path(folder) / f"{name}{suffix}"
            if candidate.is_file() and candidate not in found:
                found.append(candidate)
    return found


def remove_file(path: Path) -> None:
    try:
        path.unlink()
    except PermissionError:
        if windows():
            path.rename(path.with_name(path.name + ".removed-by-cold-start"))
        else:
            run(["sudo", "rm", "-f", str(path)], check=True)


def clean_rust() -> None:
    if shutil.which("rustup"):
        run(["rustup", "self", "uninstall", "-y"])
    for name in ("cargo", "rustc", "rustup"):
        for path in on_path(name):
            remove_file(path)
    for folder in (home() / ".cargo", home() / ".rustup"):
        shutil.rmtree(folder, ignore_errors=True)


def clean_lfs() -> None:
    if sys.platform.startswith("linux"):
        run(["sudo", "apt-get", "remove", "-y", "git-lfs"])
    elif sys.platform == "darwin":
        run(["brew", "uninstall", "--ignore-dependencies", "git-lfs"])
    paths = on_path("git-lfs")
    if windows():
        git_root = Path(os.environ.get("ProgramFiles", r"C:\Program Files")) / "Git"
        paths += [path for path in git_root.rglob("git-lfs.exe") if path not in paths]
    for path in paths:
        remove_file(path)
    for scope in ("--system", "--global"):
        for key in (
            "filter.lfs.clean",
            "filter.lfs.smudge",
            "filter.lfs.process",
            "filter.lfs.required",
        ):
            prefix = ["sudo"] if scope == "--system" and not windows() else []
            subprocess.run(
                [*prefix, "git", "config", scope, "--unset-all", key], check=False
            )


def dummy_godot() -> None:
    folder = home() / ".cold-start-mutation"
    folder.mkdir(parents=True, exist_ok=True)
    target = folder / ("godot.cmd" if windows() else "godot")
    target.write_text(
        "@echo 4.7.2\n" if windows() else "#!/bin/sh\necho 4.7.2\n", encoding="utf-8"
    )
    target.chmod(0o755)
    github_path = os.environ.get("GITHUB_PATH")
    if github_path:
        with open(github_path, "a", encoding="utf-8") as handle:
            handle.write(f"{folder}\n")


def clean() -> int:
    planted = mutation()
    if planted != "keep-rust":
        clean_rust()
    if planted != "keep-lfs":
        clean_lfs()
    if planted == "godot-on-path":
        dummy_godot()
    return 0


# ── assert-clean ──────────────────────────────────────────────────────────────────────────────────────────


def still_present(
    layer: str, where: Path, which=shutil.which, lfs_version=None
) -> list[str]:
    """What is not absent, each with where it was found. `which` and `lfs_version` are injectable for the
    self-test."""
    present = []
    for name in ABSENT_COMMANDS:
        found = which(name)
        if found:
            present.append(f"{name} at {found}")
    if lfs_version is None:
        try:
            code = subprocess.run(
                ["git", "lfs", "version"], capture_output=True, check=False
            ).returncode
        except OSError:
            code = 1
        lfs_version = code == 0
    if lfs_version:
        present.append("git-lfs (`git lfs version` succeeds)")
    for folder in (".cargo", ".rustup"):
        if (where / folder).exists():
            present.append(f"the folder {where / folder}")
    if layer == "b" and (where / "MineWorld" / ".git").exists():
        present.append(f"a MineWorld clone at {where / 'MineWorld'}")
    return present


def assert_clean(layer: str) -> int:
    present = still_present(layer, home())
    print(f"kept by design, not claimed clean: {KEPT_BY_DESIGN}")
    for name in (
        "CARGO_HOME",
        "RUSTUP_HOME",
        "GODOT",
        "MINEWORLD_BIN",
        "MINEWORLD_PACKS",
    ):
        value = os.environ.get(name)
        if (
            value
            and Path(value).exists()
            and name not in ("CARGO_HOME", "RUSTUP_HOME")
            or value
            and name in ("CARGO_HOME", "RUSTUP_HOME")
            and Path(value).exists()
        ):
            present.append(f"{name}={value}, which exists")
    if present:
        for item in present:
            print(f"NOT CLEAN: {item}")
        return 1
    print(
        "clean: " + ", ".join(ABSENT_COMMANDS) + ", git-lfs, ~/.cargo, ~/.rustup absent"
    )
    return 0


# ── config ────────────────────────────────────────────────────────────────────────────────────────────────


def toml_text(value: str) -> str:
    return json.dumps(value, ensure_ascii=False)


def bundle_server(bundle: Path) -> str:
    text = (bundle / "runtime" / "BUNDLE.toml").read_text(encoding="utf-8")
    match = re.search(r'^server\s*=\s*"([^"]+)"', text, re.MULTILINE)
    return match.group(1) if match else ""


def config_text(
    layer: str,
    phase: str,
    harness: Path,
    scratch: Path,
    reader_home: Path,
    ref: str,
    planted: str,
    bundle: Path | None,
    godot: str,
) -> str:
    exe = ".exe" if windows() else ""
    commit = (harness / "COMMIT").read_text(encoding="utf-8").strip()
    clone = reader_home / "MineWorld"
    if layer == "a":
        binary = clone / "target" / "release" / f"mineworld{exe}"
        if planted == "wrong-binary":
            binary = clone / "target" / "release" / f"mineworld-missing{exe}"
        repo, pages = clone, []
    else:
        if bundle is None:
            raise SystemExit("layer b needs --bundle")
        binary = bundle / "runtime" / bundle_server(bundle)
        repo, pages = harness, LAYER_B_PAGES
    lines = [
        f"layer = {toml_text(layer)}",
        f"phase = {toml_text(phase)}",
        f"guide = {toml_text(str(harness / 'docs' / 'guide'))}",
        f"repo = {toml_text(str(repo))}",
        f"binary = {toml_text(str(binary))}",
        "pages = [" + ", ".join(toml_text(page) for page in pages) + "]",
        f"expected_commit = {toml_text(commit)}",
        f"pre_merge = {'false' if ref == 'refs/heads/main' else 'true'}",
        f"scratch_root = {toml_text(str(scratch))}",
        f"report = {toml_text(str(scratch / 'report'))}",
        f"mutation = {toml_text(planted if planted not in RUNNER_MUTATIONS else '')}",
        f"godot = {toml_text(godot)}",
    ]
    if layer == "a":
        lines.append(f"clone = {toml_text(str(clone))}")
    else:
        lines.append(f"bundle = {toml_text(str(bundle))}")
    return "\n".join(lines) + "\n"


def write_config(arguments: dict[str, str]) -> int:
    scratch = Path(os.environ.get("RUNNER_TEMP", tempfile.gettempdir())) / "cold-start"
    scratch.mkdir(parents=True, exist_ok=True)
    bundle = Path(arguments["--bundle"]) if arguments.get("--bundle") else None
    text = config_text(
        arguments["--layer"],
        arguments["--phase"],
        Path(arguments["--harness"]),
        scratch,
        home(),
        os.environ.get("GITHUB_REF", ""),
        mutation(),
        bundle,
        arguments.get("--godot", ""),
    )
    Path(arguments["--out"]).write_text(text, encoding="utf-8")
    print(text)
    return 0


# ── harness ───────────────────────────────────────────────────────────────────────────────────────────────


def guide_executable(messages: str) -> Path:
    """The `guide` test executable from `cargo test --no-run --message-format=json`'s messages."""
    found = []
    for line in messages.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        target = message.get("target", {}) if isinstance(message, dict) else {}
        if (
            message.get("reason") == "compiler-artifact"
            and target.get("name") == "guide"
            and "test" in target.get("kind", [])
            and message.get("executable")
        ):
            found.append(Path(message["executable"]))
    if len(found) != 1:
        raise SystemExit(
            f"expected one guide test executable in cargo's messages, found {found}"
        )
    return found[0]


def harness(build_json: Path, dest: Path, commit: str) -> int:
    """The one folder a clean job receives: the guide test executable, the pages, this script and its
    neighbours, the committed mutations, and the commit they were built from (pr-25g §7.2)."""
    executable = guide_executable(build_json.read_text(encoding="utf-8"))
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise SystemExit(f"commit {commit!r} is not a commit id")
    dest.mkdir(parents=True, exist_ok=True)
    shutil.copy2(executable, dest / ("guide.exe" if windows() else "guide"))
    shutil.copytree(ROOT / "docs" / "guide", dest / "docs" / "guide")
    shutil.copytree(
        ROOT / "scripts", dest / "scripts", ignore=shutil.ignore_patterns("__pycache__")
    )
    mutations = Path("tools/cli/tests/guide/cold_start_mutations.toml")
    (dest / mutations).parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / mutations, dest / mutations)
    (dest / "COMMIT").write_text(commit + "\n", encoding="utf-8")
    print(f"harness at {dest}: {executable.name}, commit {commit}")
    return 0


# ── install-uv, find-bundle, unpack ───────────────────────────────────────────────────────────────────────


def install_uv() -> int:
    if windows():
        script = f"irm https://astral.sh/uv/{UV_VERSION}/install.ps1 | iex"
        return run(
            [
                "powershell",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                script,
            ]
        ).returncode
    script = f"curl -LsSf https://astral.sh/uv/{UV_VERSION}/install.sh | sh"
    return run(["sh", "-c", script]).returncode


def find_bundle(github_output: str) -> int:
    run_id = os.environ.get("RELEASE_RUN_ID", "").strip()
    if not run_id:
        listed = subprocess.run(
            [
                "gh",
                "run",
                "list",
                "--workflow",
                RELEASE_WORKFLOW,
                "--branch",
                "main",
                "--status",
                "success",
                "--limit",
                "1",
                "--json",
                "databaseId",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        runs = json.loads(listed.stdout or "[]") if listed.returncode == 0 else []
        run_id = str(runs[0]["databaseId"]) if runs else ""
        print(listed.stderr, end="")
    if run_id and not run_id.isdigit():
        raise SystemExit(f"release-run-id {run_id!r} is not a run id")
    print(f"release run: {run_id or 'none'}")
    with open(github_output, "a", encoding="utf-8") as handle:
        handle.write(f"run-id={run_id}\n")
    return 0


def unpack(archive_dir: Path, dest: Path, github_output: str) -> int:
    archives = sorted(archive_dir.glob("MineWorld-*.zip")) + sorted(
        archive_dir.glob("MineWorld-*.tar.gz")
    )
    if len(archives) != 1:
        raise SystemExit(
            f"expected one bundle archive in {archive_dir}, found {archives}"
        )
    dest.mkdir(parents=True, exist_ok=True)
    archive = archives[0]
    if archive.suffix == ".zip":
        with zipfile.ZipFile(archive) as opened:
            opened.extractall(dest)
        for path in dest.rglob(
            "*"
        ):  # zip keeps no exec bit through Python; restore it on the programs
            if path.is_file() and path.parent.name == "runtime" and windows() is False:
                path.chmod(path.stat().st_mode | 0o111)
    else:
        with tarfile.open(archive) as opened:
            opened.extractall(dest, filter="tar")
    runtimes = [
        path.parent
        for path in dest.rglob("BUNDLE.toml")
        if path.parent.name == "runtime"
    ]
    if len(runtimes) != 1:
        raise SystemExit(
            f"expected one runtime/BUNDLE.toml under {dest}, found {runtimes}"
        )
    with open(github_output, "a", encoding="utf-8") as handle:
        handle.write(f"bundle={runtimes[0].parent}\n")
    return 0


# ── summary, verdicts ─────────────────────────────────────────────────────────────────────────────────────


def clock(seconds: float) -> str:
    minutes, rest = divmod(round(seconds), 60)
    return f"{minutes}:{rest:02d}"


def zero_to_corner_shop(
    guide: dict, started: float, guide_started: float
) -> float | None:
    """Seconds from the clean step's start to the end of page 05: the clean and setup time before the guide
    phase began, plus the pages up to and including 05."""
    pages = guide.get("pages", [])
    names = [page["name"] for page in pages]
    if CORNER_SHOP_PAGE not in names:
        return None
    through = sum(
        page["seconds"] for page in pages[: names.index(CORNER_SHOP_PAGE) + 1]
    )
    return max(0.0, guide_started - started) + through


def render(
    layer: str,
    os_name: str,
    guide: dict,
    substitutes: list,
    budget: float,
    zero: float | None,
) -> str:
    rows = [
        f"### cold-start {layer.upper()} — {os_name}, commit {guide.get('commit', '?')[:8]}",
        "",
    ]
    if guide.get("pre_merge"):
        rows += [
            f"**PRE-MERGE (clone moved to {guide.get('commit', '?')[:8]}); not acceptance evidence**",
            "",
        ]
    rows += ["| phase | time | verdict |", "| --- | --- | --- |"]
    for page in guide.get("pages", []):
        if page["name"] == BUILD_PAGE:
            for block in (
                b for b in guide.get("blocks", []) if b["page"] == BUILD_PAGE
            ):
                rows.append(
                    f"| 01 line {block['line']} ({block['step']}) | {clock(block['seconds'])} | {block['outcome']} |"
                )
        rows.append(
            f"| {page['name']} | {clock(page['seconds'])} | {page['verdict']} |"
        )
    if layer == "a":
        if zero is None:
            rows.append(
                "| zero to Corner Shop (clean → 05) | — | INCONCLUSIVE (page 05 did not run) |"
            )
        else:
            verdict = "PASS" if zero <= budget * 60 else "FAIL"
            rows.append(
                f"| zero to Corner Shop (clean → 05) | {clock(zero)} | {verdict} (budget {clock(budget * 60)}) |"
            )
    for item in substitutes:
        rows.append(
            f"| {item['name']} | {clock(item['seconds'])} | {item['verdict']} |"
        )
    for gate in guide.get("gates", []):
        rows.append(f"| {gate['name']} | | {gate['verdict']} |")
    rows += [
        "",
        f"Guide verdict: **{guide.get('verdict', 'INCONCLUSIVE (no guide report)')}**",
        "",
        f"NOT COVERED: {NOT_COVERED}",
        "",
        f"NOT CLEAN (kept by design): {KEPT_BY_DESIGN}",
        "",
    ]
    return "\n".join(rows)


def read_json(path: Path, default):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError):
        return default


def summary(
    report: Path, layer: str, started: float, guide_started: float, budget: float
) -> int:
    guide = read_json(report / "guide.json", {})
    substitutes = read_json(report / "substitutes.json", [])
    zero = zero_to_corner_shop(guide, started, guide_started) if guide else None
    text = render(
        layer, guide.get("os", sys.platform), guide, substitutes, budget, zero
    )
    print(text)
    step_summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if step_summary:
        with open(step_summary, "a", encoding="utf-8") as handle:
            handle.write(text + "\n")
    if layer == "a" and (zero is None or zero > budget * 60):
        print(
            f"budget: zero to Corner Shop {('not measured' if zero is None else clock(zero))} "
            f"against {clock(budget * 60)}"
        )
        return 1
    return 0


def verdicts(folder: Path) -> int:
    rows = ["| leg | verdict |", "| --- | --- |"]
    red = False
    files = sorted(folder.rglob("verdict-*.txt"))
    for path in files:
        verdict = path.read_text(encoding="utf-8").strip() or "INCONCLUSIVE"
        rows.append(f"| {path.stem.removeprefix('verdict-')} | {verdict} |")
        red = red or verdict != "PASS"
    if not files:
        rows.append("| (no leg reported) | INCONCLUSIVE |")
        red = True
    text = "\n".join(rows)
    print(text)
    step_summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if step_summary:
        with open(step_summary, "a", encoding="utf-8") as handle:
            handle.write("### cold-start\n\n" + text + "\n")
    return 1 if red else 0


# ── workflow lint ─────────────────────────────────────────────────────────────────────────────────────────


def job_blocks(text: str) -> dict[str, str]:
    """The workflow's jobs, by id, each with its text (top-level `jobs:` children at two spaces)."""
    jobs_at = text.find("\njobs:\n")
    if jobs_at < 0:
        return {}
    body = text[jobs_at + len("\njobs:\n") :]
    blocks: dict[str, str] = {}
    current = None
    for line in body.splitlines():
        match = re.match(r"^  ([A-Za-z0-9_-]+):\s*$", line)
        if match:
            current = match.group(1)
            blocks[current] = ""
        elif current is not None:
            blocks[current] += line + "\n"
    return blocks


def lint_workflow(text: str) -> list[str]:
    problems = []
    blocks = job_blocks(text)
    for job in CLEAN_JOBS:
        if job not in blocks:
            problems.append(f"job {job} is missing")
            continue
        for forbidden in (
            "actions/cache",
            "actions/setup-",
            "astral-sh/setup-uv",
            "./.github/actions/native",
            "actions/checkout",
            "dtolnay/rust-toolchain",
        ):
            if forbidden in blocks[job]:
                problems.append(
                    f"job {job} uses {forbidden}: the clean jobs restore nothing and set nothing up"
                )
    if re.search(r"^\s*(pull_request|pull_request_target)\s*:", text, re.MULTILINE):
        problems.append(
            "the workflow must never run on pull_request (it is never a required check)"
        )
    for line in text.splitlines():
        if not re.search(r"\$\{\{[^}]*inputs\.", line):
            continue
        # An input may be a mapping's value (`env:`, `with:`, `if:`), never a line of a script.
        key = re.match(r"^\s*(?:-\s*)?([A-Za-z0-9_.-]+):\s", line)
        if key is None or key.group(1) == "run":
            problems.append(f"an input reaches a script by expression: {line.strip()}")
    return problems


def lint() -> int:
    problems = lint_workflow(WORKFLOW.read_text(encoding="utf-8"))
    for problem in problems:
        print(f"FAIL {problem}")
    return 1 if problems else 0


# ── self-test ─────────────────────────────────────────────────────────────────────────────────────────────


def self_test() -> int:
    failures = []

    def check(condition: bool, what: str) -> None:
        if not condition:
            failures.append(what)

    with tempfile.TemporaryDirectory() as folder:
        where = Path(folder)
        fake = {"cargo": str(where / "bin" / "cargo")}
        present = still_present("a", where, which=fake.get, lfs_version=False)
        check(
            present == [f"cargo at {fake['cargo']}"],
            f"a fake cargo on PATH is named: {present}",
        )
        check(
            still_present("a", where, which=lambda _: None, lfs_version=False) == [],
            "nothing present",
        )
        check(
            any(
                "git-lfs" in item
                for item in still_present("a", where, lambda _: None, True)
            ),
            "lfs named",
        )
        (where / ".rustup").mkdir()
        check(
            any(
                ".rustup" in item
                for item in still_present("a", where, lambda _: None, False)
            ),
            "folder",
        )
        (where / "MineWorld" / ".git").mkdir(parents=True)
        check(
            not any(
                "clone" in item
                for item in still_present("a", where, lambda _: None, False)
            ),
            "a clone",
        )
        check(
            any(
                "clone" in item
                for item in still_present("b", where, lambda _: None, False)
            ),
            "b clone",
        )

        harness = where / "harness"
        harness.mkdir()
        (harness / "COMMIT").write_text("a" * 40 + "\n", encoding="utf-8")
        text = config_text(
            "a",
            "guide",
            harness,
            where / "s",
            where / "home",
            "refs/heads/main",
            "",
            None,
            "",
        )
        check(
            "pre_merge = false" in text and "clone = " in text and "bundle" not in text,
            f"config a: {text}",
        )
        check(
            f'mineworld{".exe" if windows() else ""}"' in text,
            "config a names the reader's binary",
        )
        planted = config_text(
            "a",
            "guide",
            harness,
            where / "s",
            where / "home",
            "refs/heads/x",
            "wrong-binary",
            None,
            "",
        )
        check(
            "mineworld-missing" in planted and "pre_merge = true" in planted,
            "AD-6 and PRE-MERGE",
        )
        check(
            'mutation = ""' in planted,
            "a runner mutation is not passed to the test as a page mutation",
        )
        bundle = where / "bundle"
        (bundle / "runtime").mkdir(parents=True)
        (bundle / "runtime" / "BUNDLE.toml").write_text(
            'commit = "x"\nserver = "mineworld"\n', encoding="utf-8"
        )
        text_b = config_text(
            "b",
            "guide",
            harness,
            where / "s",
            where / "home",
            "refs/heads/main",
            "",
            bundle,
            "",
        )
        check(
            '"06-play-it"]' in text_b
            and "bundle = " in text_b
            and "clone" not in text_b,
            f"config b: {text_b}",
        )

    guide = {
        "commit": "b" * 40,
        "pages": [
            {"name": "01-get-mineworld", "seconds": 600.0, "verdict": "PASS"},
            {"name": "05-one-shop", "seconds": 30.0, "verdict": "PASS"},
            {"name": "06-play-it", "seconds": 5.0, "verdict": "PASS"},
        ],
        "blocks": [
            {
                "page": "01-get-mineworld",
                "step": "Step 1.1",
                "line": 30,
                "seconds": 300.0,
                "outcome": "PASS",
            }
        ],
        "gates": [],
        "verdict": "PASS",
    }
    check(
        zero_to_corner_shop(guide, 1000.0, 1120.0) == 750.0,
        "clean 120 s + 600 + 30 = 750 s",
    )
    check(
        zero_to_corner_shop({"pages": []}, 0.0, 0.0) is None,
        "no page 05 is not measured",
    )
    table = render(
        "a",
        "linux",
        guide,
        [{"name": "substitute 06.2 serve", "seconds": 8, "verdict": "PASS"}],
        60,
        750.0,
    )
    check(
        "| zero to Corner Shop (clean → 05) | 12:30 | PASS (budget 60:00) |" in table,
        table,
    )
    check(
        "| 01 line 30 (Step 1.1) | 5:00 | PASS |" in table and "NOT COVERED" in table,
        table,
    )
    messages = "\n".join(
        [
            json.dumps(
                {
                    "reason": "compiler-artifact",
                    "target": {"name": "guide", "kind": ["test"]},
                    "executable": "/t/debug/deps/guide-1",
                }
            ),
            json.dumps(
                {
                    "reason": "compiler-artifact",
                    "target": {"name": "mineworld", "kind": ["bin"]},
                    "executable": "/t/debug/mineworld",
                }
            ),
            "not json",
        ]
    )
    check(
        guide_executable(messages) == Path("/t/debug/deps/guide-1"),
        "the guide executable is found",
    )
    check(
        "FAIL (budget 1:00)" in render("a", "linux", guide, [], 1, 750.0),
        "AD-7: a one-minute budget fails",
    )

    clean_job = "  cold-start-a:\n    steps:\n      - run: python3 x\n"
    workflow = (
        "on:\n  push:\njobs:\n"
        + clean_job
        + "  cold-start-b:\n    steps:\n      - run: y\n"
    )
    check(
        lint_workflow(workflow) == [],
        f"a clean workflow passes: {lint_workflow(workflow)}",
    )
    cached = workflow.replace(
        "      - run: python3 x\n",
        "      - uses: actions/cache@0000000000000000000000000000000000000000\n",
    )
    check(
        any("actions/cache" in problem for problem in lint_workflow(cached)),
        "AD-8: a cache is refused",
    )
    check(
        any(
            "pull_request" in p
            for p in lint_workflow("on:\n  pull_request:\n" + workflow[4:])
        ),
        "a pull_request trigger is refused",
    )
    check(
        any(
            "by expression" in p
            for p in lint_workflow(
                workflow + "      - run: echo ${{ inputs.layers }}\n"
            )
        ),
        "an input in run: is refused",
    )
    if WORKFLOW.exists():
        check(
            lint_workflow(WORKFLOW.read_text(encoding="utf-8")) == [],
            "the committed workflow passes",
        )

    for failure in failures:
        print(f"FAIL {failure}")
    print(f"ci_cold_start self-test: {'FAIL' if failures else 'PASS'}")
    return 1 if failures else 0


def options(arguments: list[str]) -> dict[str, str]:
    parsed = {}
    iterator = iter(arguments)
    for word in iterator:
        if not word.startswith("--"):
            raise SystemExit(f"unexpected argument {word!r}")
        parsed[word] = next(iterator, "")
    return parsed


def main(argv: list[str]) -> int:
    # Windows' console encoding is a code page; the summary's table holds `→` and `—` (run 46119663095).
    for stream in (sys.stdout, sys.stderr):
        if isinstance(stream, io.TextIOWrapper):
            stream.reconfigure(encoding="utf-8")
    if not argv:
        print(__doc__)
        return 2
    command, rest = argv[0], options(argv[1:]) if argv[0] != "--self-test" else {}
    if command == "--self-test":
        return self_test()
    if command == "clean":
        return clean()
    if command == "assert-clean":
        return assert_clean(rest.get("--layer", "a"))
    if command == "config":
        return write_config(rest)
    if command == "harness":
        return harness(
            Path(rest["--build-json"]), Path(rest["--dest"]), rest["--commit"]
        )
    if command == "install-uv":
        return install_uv()
    if command == "find-bundle":
        return find_bundle(rest["--github-output"])
    if command == "unpack":
        return unpack(
            Path(rest["--archive-dir"]), Path(rest["--dest"]), rest["--github-output"]
        )
    if command == "summary":
        return summary(
            Path(rest["--report"]),
            rest["--layer"],
            float(rest["--started"]),
            float(rest["--guide-started"]),
            float(rest.get("--budget-minutes", "60")),
        )
    if command == "verdicts":
        return verdicts(Path(rest["--dir"]))
    if command == "lint":
        return lint()
    raise SystemExit(f"unknown subcommand {command!r}")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
