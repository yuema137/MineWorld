#!/usr/bin/env python3
"""Optional packs stay optional (X-S30-18, X-S30-19; the S30-h design §5.10, D-h-2).

An optional pack (an Entity Pack under `entities/`, or a world under `worlds/`) declares what it owns in its own
`.optional.yaml`; this script names no pack. The declaration is written in YAML's JSON-compatible form, so the
standard library reads it (§16 D-h1-2):

    {
      "format": 1,
      "optional": "<the pack's directory name>",
      "words": ["<a word only this pack and its allowed paths may name, matched in any case>"],
      "may_name": ["<a tracked file, or a directory ending in />"],
      "dependents": ["<a path that needs the pack, deleted with it>"],
      "checks": [["mineworld", "attributes", "check", "..."], {"command": [...], "expect": "<file>"}]
    }

    python3 scripts/check_optional_packs.py scan              every tracked file outside each pack and its
                                                              may_name paths is free of the pack's words
    python3 scripts/check_optional_packs.py --self-test       a planted tree: a word in a forbidden file fails,
                                                              in an allowed one passes; malformed files fail
    python3 scripts/check_optional_packs.py run --binary B    every declared check, `mineworld` replaced by B;
                                                              a declared `expect` file is compared byte for byte
    python3 scripts/check_optional_packs.py without [--keep]  a detached copy of HEAD with every optional pack
                                                              and its dependents deleted builds and tests

The script fails closed: a malformed declaration, a listed path that does not exist, an unreadable tracked file
or a failing check is a failure. It runs only commands a reviewed `.optional.yaml` lists.
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DECLARATION = ".optional.yaml"
FAMILIES = ("entities", "worlds")
KEYS = {"format", "optional", "words", "may_name", "dependents", "checks"}
WITHOUT_COMMANDS: list[list[str]] = [
    ["cargo", "fmt", "--all", "--check"],
    ["cargo", "clippy", "--workspace", "--all-targets", "--", "-D", "warnings"],
    ["cargo", "test", "--workspace"],
]
"""What the copy without the optional packs must pass: the removability job's Rust commands (ARC-347)."""


class Malformed(Exception):
    """A declaration this script refuses, with the file and the reason."""


@dataclass(frozen=True)
class Check:
    command: list[str]
    expect: str | None


@dataclass(frozen=True)
class Optional:
    path: str
    """The pack's directory, relative to the root, ending in `/`."""
    words: list[str]
    may_name: list[str]
    dependents: list[str]
    checks: list[Check]


def strings(value: object, field: str, where: str, *, empty: bool = True) -> list[str]:
    if not isinstance(value, list) or not all(
        isinstance(item, str) and item for item in value
    ):
        raise Malformed(f"{where}: '{field}' is a list of non-empty strings")
    if not empty and not value:
        raise Malformed(f"{where}: '{field}' names at least one")
    return list(value)


def check_of(entry: object, where: str) -> Check:
    if isinstance(entry, list):
        return Check(strings(entry, "checks", where, empty=False), None)
    if (
        isinstance(entry, dict)
        and set(entry) <= {"command", "expect"}
        and "command" in entry
    ):
        expect = entry.get("expect")
        if expect is not None and not (isinstance(expect, str) and expect):
            raise Malformed(f"{where}: a check's 'expect' is a path")
        return Check(strings(entry["command"], "command", where, empty=False), expect)
    raise Malformed(f"{where}: a check is a command list or {{command, expect}}")


def load(root: Path, relative: str) -> Optional:
    """One declaration, checked; every listed path must exist under `root`."""
    where = relative
    try:
        document = json.loads((root / relative).read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise Malformed(
            f"{where}: not readable as JSON-compatible YAML: {error}"
        ) from error
    if not isinstance(document, dict) or set(document) != KEYS:
        found = (
            sorted(document) if isinstance(document, dict) else type(document).__name__
        )
        raise Malformed(f"{where}: the keys are exactly {sorted(KEYS)}, found {found}")
    if document["format"] != 1:
        raise Malformed(f"{where}: format is 1")
    directory = Path(relative).parent
    if document["optional"] != directory.name:
        raise Malformed(
            f"{where}: 'optional' is the pack's directory name '{directory.name}'"
        )
    words = strings(document["words"], "words", where, empty=False)
    for word in words:
        if word != word.lower() or not word.isascii() or not word.isalnum():
            raise Malformed(
                f"{where}: the word '{word}' is lower-case ASCII letters and digits"
            )
    may_name = strings(document["may_name"], "may_name", where)
    dependents = strings(document["dependents"], "dependents", where)
    for path in [*may_name, *dependents]:
        if not (root / path).exists():
            raise Malformed(f"{where}: '{path}' does not exist")
        if path.endswith("/") != (root / path).is_dir():
            raise Malformed(
                f"{where}: '{path}' names a directory exactly when it ends in '/'"
            )
    checks = (
        [check_of(entry, where) for entry in document["checks"]]
        if isinstance(document["checks"], list)
        else None
    )
    if checks is None:
        raise Malformed(f"{where}: 'checks' is a list")
    for check in checks:
        if check.expect is not None and not (root / check.expect).is_file():
            raise Malformed(
                f"{where}: the expected file '{check.expect}' does not exist"
            )
    return Optional(f"{directory.as_posix()}/", words, may_name, dependents, checks)


def declarations(tracked: list[str]) -> list[str]:
    """Every `<family>/<pack>/.optional.yaml` among the tracked files, sorted."""
    found = []
    for path in tracked:
        parts = path.split("/")
        if len(parts) == 3 and parts[0] in FAMILIES and parts[2] == DECLARATION:
            found.append(path)
    return sorted(found)


def allowed(path: str, pack: Optional) -> bool:
    return path.startswith(pack.path) or any(
        path.startswith(entry) if entry.endswith("/") else path == entry
        for entry in pack.may_name
    )


def scan(root: Path, tracked: list[str]) -> list[str]:
    """Each finding as `file:line: names 'word' (owned by <pack>)`; a malformed declaration is a finding too."""
    findings: list[str] = []
    packs: list[Optional] = []
    for relative in declarations(tracked):
        try:
            packs.append(load(root, relative))
        except Malformed as error:
            findings.append(f"{error}")
    for path in tracked:
        concerned = [pack for pack in packs if not allowed(path, pack)]
        if not concerned:
            continue
        try:
            data = (root / path).read_bytes()
        except OSError as error:
            findings.append(f"{path}: unreadable: {error}")
            continue
        lowered = data.lower()
        for pack in concerned:
            for word in pack.words:
                if word.encode() not in lowered:
                    continue
                for number, line in enumerate(lowered.splitlines(), start=1):
                    if word.encode() in line:
                        findings.append(
                            f"{path}:{number}: names '{word}' (owned by {pack.path})"
                        )
    return findings


def tracked_files(root: Path) -> list[str]:
    listed = subprocess.run(
        ["git", "ls-files", "-z"], cwd=root, check=True, capture_output=True
    ).stdout
    return [
        path
        for path in listed.decode("utf-8").split("\0")
        if path and (root / path).is_file()
    ]


def all_packs(root: Path) -> list[Optional]:
    return [load(root, relative) for relative in declarations(tracked_files(root))]


def command_scan() -> int:
    tracked = tracked_files(ROOT)
    count = len(declarations(tracked))
    findings = scan(ROOT, tracked)
    for finding in findings:
        print(f"[optional] FAIL {finding}")
    if findings:
        return 1
    print(
        f"[optional] {count} optional pack(s); {len(tracked)} tracked files free of their words"
    )
    return 0


def command_run(binary: str) -> int:
    packs = all_packs(ROOT)
    if not packs:
        print("[optional] 0 optional packs: no declared checks")
        return 0
    failed = 0
    for pack in packs:
        for check in pack.checks:
            command = [
                binary if part == "mineworld" else part for part in check.command
            ]
            print(f"[optional] {pack.path}: $ {' '.join(check.command)}", flush=True)
            done = subprocess.run(command, cwd=ROOT, capture_output=True, check=False)
            if done.returncode != 0:
                failed += 1
                print(
                    f"[optional] FAIL exit {done.returncode}\n{done.stdout.decode(errors='replace')}"
                    f"{done.stderr.decode(errors='replace')}"
                )
                continue
            if (
                check.expect is not None
                and done.stdout != (ROOT / check.expect).read_bytes()
            ):
                failed += 1
                print(
                    f"[optional] FAIL stdout differs from {check.expect}:\n"
                    f"{done.stdout.decode(errors='replace')}"
                )
    print(
        f"[optional] {'FAIL' if failed else 'PASS'}: {failed} failing declared check(s)"
    )
    return 1 if failed else 0


def command_without(keep: bool) -> int:
    packs = all_packs(ROOT)
    removed = sorted(
        {path.rstrip("/") for pack in packs for path in [pack.path, *pack.dependents]}
    )
    if not removed:
        print("[optional] 0 optional packs: nothing to remove")
        return 0
    sys.path.insert(0, str(ROOT / "scripts"))
    import ci_removability  # the removability job's throwaway checkout (ARC-347), reused

    scratch = Path(tempfile.mkdtemp(prefix="mineworld-optional-")) / "checkout"
    environment = dict(os.environ)
    environment.setdefault("CARGO_TARGET_DIR", str(ROOT / "target" / "optional"))
    try:
        ci_removability.checkout_without(scratch, removed, lm=False)
        print(
            f"[optional] worktree of HEAD at {scratch}: {', '.join(removed)} deleted",
            flush=True,
        )
        for command in WITHOUT_COMMANDS:
            print(f"[optional] $ {' '.join(command)}", flush=True)
            status = subprocess.run(
                command, cwd=scratch, env=environment, check=False
            ).returncode
            if status != 0:
                print(
                    f"[optional] FAIL: without {', '.join(removed)}, `{' '.join(command)}` failed"
                )
                return status
        print(
            f"[optional] PASS: everything builds and passes without {', '.join(removed)}"
        )
        return 0
    finally:
        if keep:
            print(f"[optional] kept {scratch}")
        else:
            subprocess.run(
                ["git", "worktree", "remove", "--force", str(scratch)],
                cwd=ROOT,
                check=False,
            )
            shutil.rmtree(scratch.parent, ignore_errors=True)
            subprocess.run(["git", "worktree", "prune"], cwd=ROOT, check=False)


def self_test() -> int:
    """The planted cases: each must give the stated verdict."""
    declaration = {
        "format": 1,
        "optional": "demo-pack",
        "words": ["zork"],
        "may_name": ["docs/allowed.md", "notes/"],
        "dependents": ["worlds/zork-demo/"],
        "checks": [],
    }

    def tree(base: Path, files: dict[str, str], declared: object) -> list[str]:
        for relative, text in files.items():
            (base / relative).parent.mkdir(parents=True, exist_ok=True)
            (base / relative).write_text(text, encoding="utf-8")
        (base / "entities/demo-pack").mkdir(parents=True, exist_ok=True)
        (base / "entities/demo-pack" / DECLARATION).write_text(
            declared if isinstance(declared, str) else json.dumps(declared),
            encoding="utf-8",
        )
        return sorted([*files, f"entities/demo-pack/{DECLARATION}"])

    allowed_files = {
        "docs/allowed.md": "Zork is optional.\n",
        "notes/a.md": "ZORK\n",
        "worlds/zork-demo/world.yaml": "x\n",
        "entities/demo-pack/pack.yaml": "id: demo-pack # zork\n",
        "src/lib.rs": "fn main() {}\n",
    }
    cases: list[tuple[str, dict[str, str], object, str | None]] = [
        ("allowed paths pass", allowed_files, declaration, None),
        (
            "a word in a forbidden file fails by file and line",
            {**allowed_files, "src/lib.rs": "fn main() {}\n// zOrK here\n"},
            declaration,
            "src/lib.rs:2: names 'zork'",
        ),
        (
            "a listed path that does not exist fails",
            allowed_files,
            {**declaration, "may_name": ["docs/missing.md"]},
            "'docs/missing.md' does not exist",
        ),
        (
            "a directory without its slash fails",
            allowed_files,
            {**declaration, "may_name": ["notes"]},
            "names a directory exactly when",
        ),
        (
            "an unknown key fails",
            allowed_files,
            {**declaration, "extra": 1},
            "the keys are exactly",
        ),
        (
            "a wrong pack name fails",
            allowed_files,
            {**declaration, "optional": "other"},
            "directory name",
        ),
        (
            "a word with capitals fails",
            allowed_files,
            {**declaration, "words": ["Zork"]},
            "lower-case",
        ),
        (
            "unparsable text fails",
            allowed_files,
            "optional: demo-pack\n",
            "not readable",
        ),
        (
            "a malformed check fails",
            allowed_files,
            {**declaration, "checks": [[]]},
            "names at least one",
        ),
    ]
    failed = 0
    for name, files, declared, expected in cases:
        with tempfile.TemporaryDirectory(
            prefix="mineworld-optional-self-test-"
        ) as base:
            findings = scan(Path(base), tree(Path(base), files, declared))
        ok = (
            not findings
            if expected is None
            else any(expected in finding for finding in findings)
        )
        print(
            f"[optional] self-test {'PASS' if ok else 'FAIL'}: {name}"
            + ("" if ok else f": {findings}")
        )
        failed += not ok
    no_packs = scan(ROOT, [])
    if no_packs:
        failed += 1
        print(f"[optional] self-test FAIL: an empty tree has findings: {no_packs}")
    return 1 if failed else 0


def main(arguments: list[str]) -> int:
    if arguments == ["scan"]:
        return command_scan()
    if arguments == ["--self-test"]:
        return self_test()
    if len(arguments) == 3 and arguments[:2] == ["run", "--binary"]:
        return command_run(arguments[2])
    if arguments in (["without"], ["without", "--keep"]):
        return command_without("--keep" in arguments)
    print(__doc__)
    return 2


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except Malformed as error:
        print(f"[optional] FAIL {error}")
        raise SystemExit(1) from error
