#!/usr/bin/env python3
"""Parse every GDScript file of the Godot clients headlessly, and fail on any script that does not compile.

docs/decisions/ARC-154.md, note of 2026-10-26. `fast` runs it on every code change, so that a client that no longer compiles
(PRs #454 and #456 read the removed `SliceMain.PACK_DIR`) is red before it merges.

    python3 scripts/ci_gdparse.py [PROJECT ...]
        For each project (default: PROJECTS below): one `godot --headless --import`, which writes the
        global class cache that `class_name` references resolve through, then one headless run of
        scripts/gdparse_check.gd, which loads every *.gd file of the project (its symlinked shared
        modules included). Godot exits 0 even when a script fails to compile, so the verdict is read
        from the log: PASS needs the checker's summary line and no line matching ERROR_PATTERNS.
        A project that has never been imported (no `.godot/`, a fresh CI checkout) is imported
        scripts-only: every top-level folder that holds no *.gd file (3D models, textures, shots) gets a
        temporary `.gdignore` for the import and the check, removed afterwards. No client script
        preloads such a resource, and importing the 3D client's 360 MB of assets is what makes a full
        import take ten minutes. A project imported before keeps its import untouched.
        The Godot binary is $GODOT, else `godot` on PATH.
    python3 scripts/ci_gdparse.py --self-test
        The log classifier's cases.

Exits 0 when every project passes, 1 otherwise. Standard library only.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CHECKER = ROOT / "scripts" / "gdparse_check.gd"
PROJECTS = ("clients/shared", "clients/2d", "clients/3d-spike")
LIMIT_SECONDS = 600

# What Godot prints for a script that does not parse, compile or load. `--check-only` and `--script`
# both exit 0 regardless, which is why this list, not the exit code, is the verdict.
ERROR_PATTERNS = (
    re.compile(r"SCRIPT ERROR"),
    re.compile(r"Parse Error"),
    re.compile(r"Compile Error"),
    re.compile(r"Failed to load"),
    re.compile(r"GDPARSE FAIL"),
)
SUMMARY = re.compile(r"^GDPARSE checked (\d+) scripts, (\d+) failed to load$")


def classify(log: str) -> tuple[str, list[str]]:
    """PASS, FAIL or INCONCLUSIVE for one checker run's log, with the decisive lines."""
    lines = log.splitlines()
    errors = [line for line in lines if any(p.search(line) for p in ERROR_PATTERNS)]
    summaries = [m for m in (SUMMARY.match(line.strip()) for line in lines) if m]
    if errors:
        return "FAIL", errors
    if len(summaries) != 1:
        return "INCONCLUSIVE", ["no single `GDPARSE checked` summary line: the checker did not finish"]
    if int(summaries[0].group(1)) == 0:
        return "INCONCLUSIVE", ["the checker found no script"]
    if int(summaries[0].group(2)) != 0:
        return "FAIL", [summaries[0].group(0)]
    return "PASS", [summaries[0].group(0)]


def holds_scripts(folder: Path) -> bool:
    return next(folder.rglob("*.gd"), None) is not None


def asset_folders(project: Path) -> list[Path]:
    """Top-level folders a scripts-only import skips: real folders (never a symlinked shared module),
    not hidden, not ignored already, holding no *.gd file."""
    return sorted(
        folder for folder in project.iterdir()
        if folder.is_dir() and not folder.is_symlink() and not folder.name.startswith(".")
        and not (folder / ".gdignore").exists() and not holds_scripts(folder)
    )


def run(godot: str, project: Path, extra: list[str]) -> tuple[int, str]:
    try:
        done = subprocess.run(
            [godot, "--headless", "--path", str(project), *extra],
            capture_output=True, text=True, errors="replace", timeout=LIMIT_SECONDS, check=False,
        )
    except subprocess.TimeoutExpired as expired:
        out = expired.stdout or ""
        if isinstance(out, bytes):
            out = out.decode(errors="replace")
        return 124, out + f"\n(killed at the {LIMIT_SECONDS} s limit)"
    return done.returncode, done.stdout + done.stderr


def check(godot: str, name: str) -> str:
    project = ROOT / name
    ignored: list[Path] = []
    if not (project / ".godot").exists():
        ignored = [folder / ".gdignore" for folder in asset_folders(project)]
    try:
        for marker in ignored:
            marker.write_text("", encoding="utf-8")
        began = time.monotonic()
        code, log = run(godot, project, ["--import"])
        imported = time.monotonic() - began
        if code != 0:
            print(f"[gdparse] {name}: FAIL: the import exited {code}")
            print("\n".join(log.splitlines()[-40:]))
            return "FAIL"
        began = time.monotonic()
        code, log = run(godot, project, ["--script", str(CHECKER)])
        checked = time.monotonic() - began
    finally:
        for marker in ignored:
            marker.unlink(missing_ok=True)
    verdict, lines = classify(log)
    skipped = f", {len(ignored)} asset folders skipped" if ignored else ""
    print(f"[gdparse] {name}: {verdict} (import {imported:.0f} s, check {checked:.0f} s{skipped})")
    for line in lines:
        print(f"    {line}")
    if verdict == "PASS" and code != 0:
        print(f"[gdparse] {name}: FAIL: the checker exited {code}")
        return "FAIL"
    return verdict


def self_test() -> int:
    cases = [
        ("clean", "Godot Engine v4.7.2\nGDPARSE checked 42 scripts, 0 failed to load\n", "PASS"),
        ("parse error, exit 0",
         "SCRIPT ERROR: Parse Error: Cannot find member \"PACK_DIR\" in base \"SliceMain\".\n"
         "   at: GDScript::reload (res://scripts/depictions/social-cafe/depiction.gd:45)\n"
         "GDPARSE checked 119 scripts, 0 failed to load\n", "FAIL"),
        ("failed dependency",
         "ERROR: Failed to load script \"res://scripts/a.gd\" with error \"Parse error\".\n"
         "GDPARSE checked 3 scripts, 0 failed to load\n", "FAIL"),
        ("a null load", "GDPARSE FAIL res://a.gd: did not load\nGDPARSE checked 1 scripts, 1 failed to load\n",
         "FAIL"),
        ("crash before the summary", "Godot Engine v4.7.2\nhandle_crash: Program crashed\n", "INCONCLUSIVE"),
        ("nothing found", "GDPARSE checked 0 scripts, 0 failed to load\n", "INCONCLUSIVE"),
        ("warnings only",
         "WARNING: something deprecated\nGDPARSE checked 5 scripts, 0 failed to load\n", "PASS"),
    ]
    bad = 0
    for name, log, want in cases:
        got, _ = classify(log)
        good = got == want
        bad += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}: {got} (want {want})")
    return 1 if bad else 0


def main(arguments: list[str]) -> int:
    if arguments == ["--self-test"]:
        return self_test()
    godot = os.environ.get("GODOT") or shutil.which("godot")
    if not godot:
        print("[gdparse] no Godot: set GODOT or put `godot` on PATH")
        return 1
    verdicts = [check(godot, name) for name in (arguments or PROJECTS)]
    return 0 if all(v == "PASS" for v in verdicts) else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
