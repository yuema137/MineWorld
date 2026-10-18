"""No API key, and no path to the operator's secrets file, is committed (step-26 §5.5; AC-S26-9).

Run as `python3 scripts/check_secrets.py scan` (or `--self-test`).

Two rules, both reported by file and pattern, never by the matched text (a finding never prints a key):

  scan         1. no tracked text file contains an AI-service key shape or a bearer token or a signed
                  URL's key-pair id (the patterns below);
               2. no tracked file under a runtime directory (clients/, server/, systems/, kernel/, contracts/,
                  worldpack/, persistence/, cognition/, sdk/) names the secrets file's path.
               Files are listed with `git ls-files -z`; a file Git marks `binary`, or one with a NUL byte in its
               first 8,000 bytes, is not text and is skipped. The count of files scanned is printed.
  --self-test  plants a `msy_`-shaped string and an `sk-`-shaped string in an in-memory sidecar, and requires the
               scan of that text to find both. The planted keys are built at run time from parts, so this file
               contains no key-shaped literal.

The allowlist is a literal dict below. Each entry needs a reason; it starts empty (design §5.11).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The key shapes (step-26 §5.5, design §5.11). `scripts/` and the pipeline's `standard.py` share this list; a test
# in tools/character_pipeline/tests keeps the two equal.
PATTERNS: dict[str, str] = {
    "openai-key": r"sk-[A-Za-z0-9_-]{20,}",
    "openai-project-key": r"sk-proj-[A-Za-z0-9_-]{20,}",
    "meshy-key": r"msy_[A-Za-z0-9]{20,}",
    "bearer-token": r"Bearer [A-Za-z0-9._-]{20,}",
    "signed-url-key-pair": r"Key-Pair-Id=[A-Z0-9]{8,}",
}

SECRETS_PATH = ".config/mineworld/secrets.env"
RUNTIME_DIRECTORIES: tuple[str, ...] = (
    "clients/",
    "server/",
    "systems/",
    "kernel/",
    "contracts/",
    "worldpack/",
    "persistence/",
    "cognition/",
    "sdk/",
)

# (path, pattern name) -> reason. An entry needs a reason.
ALLOWLIST: dict[tuple[str, str], str] = {
    ("tools/cli/tests/milestone_d.rs", "openai-key"): (
        "S10 P7's PLANTED_KEY: a fake sk--shaped string the test plants in the cognition process's"
        " environment to prove no key reaches a save or a log; not a credential (R-B1-6, F-B1-merge-1)"
    ),
}

NUL_WINDOW = 8000


@dataclass(frozen=True)
class Finding:
    path: str
    line: int
    rule: str


def scan_text(path: str, text: str) -> list[Finding]:
    """Every pattern hit in `text`, by line and pattern name, skipping allowlisted (path, pattern) pairs."""
    found: list[Finding] = []
    compiled = {name: re.compile(pattern) for name, pattern in PATTERNS.items()}
    for number, line in enumerate(text.splitlines(), start=1):
        for name, regex in compiled.items():
            if regex.search(line) and (path, name) not in ALLOWLIST:
                found.append(Finding(path, number, name))
        if SECRETS_PATH in line and path.startswith(RUNTIME_DIRECTORIES):
            found.append(Finding(path, number, "secrets-path-in-runtime"))
    return found


def _tracked_files() -> list[str]:
    result = subprocess.run(
        ["git", "ls-files", "-z"],
        cwd=ROOT,
        capture_output=True,
        check=True,
    )
    return [item.decode("utf-8") for item in result.stdout.split(b"\0") if item]


def _binary_attribute(paths: list[str]) -> set[str]:
    """Paths whose `binary` attribute Git sets (.gitattributes), read in one call."""
    if not paths:
        return set()
    payload = b"".join(p.encode("utf-8") + b"\0" for p in paths)
    result = subprocess.run(
        ["git", "check-attr", "-z", "binary", "--stdin"],
        cwd=ROOT,
        input=payload,
        capture_output=True,
        check=True,
    )
    fields = result.stdout.split(b"\0")
    binary: set[str] = set()
    for index in range(0, len(fields) - 2, 3):
        path, value = fields[index], fields[index + 2]
        if value in (b"set", b"true"):
            binary.add(path.decode("utf-8"))
    return binary


def scan() -> int:
    files = _tracked_files()
    binary = _binary_attribute(files)
    findings: list[Finding] = []
    scanned = 0
    for relative in files:
        # A submodule is listed as a directory entry: it is not a file of this tree, so it is not scanned here.
        if relative in binary or not (ROOT / relative).is_file():
            continue
        data = (ROOT / relative).read_bytes()
        if b"\0" in data[:NUL_WINDOW]:
            continue
        scanned += 1
        findings.extend(scan_text(relative, data.decode("utf-8", errors="replace")))
    print(f"check_secrets: scanned {scanned} tracked text files")
    for finding in findings:
        print(f"  {finding.path}:{finding.line}: {finding.rule}")
    print(f"check_secrets: {len(findings)} findings")
    return 1 if findings else 0


def self_test() -> int:
    # Built from parts at run time: this file must not contain a key-shaped literal.
    meshy_key = "msy_" + "A1b2C3d4E5f6G7h8I9j0K1l2"
    openai_key = "sk-" + "x" * 24
    sidecar = f"generator: {meshy_key}\nkey: {openai_key}\n"
    found = {f.rule for f in scan_text("sidecar.json", sidecar)}
    missing = {"meshy-key", "openai-key"} - found
    if missing:
        print(
            f"check_secrets --self-test: FAIL, planted keys not found: {sorted(missing)}"
        )
        return 1
    print(
        "check_secrets --self-test: PASS (planted meshy-key and openai-key both found)"
    )
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("command", nargs="?", default="scan", choices=["scan"])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    if args.command == "scan":
        return scan()
    return 2


if __name__ == "__main__":
    sys.exit(main())
