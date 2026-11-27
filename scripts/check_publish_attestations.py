#!/usr/bin/env python3
"""Offline regression for publish.yml's attestation command boundary, not cryptographic verification.

Run the real shell block against a fixed producer contract. No gh executable, token or network
is used. Hosted A-R9 remains the owner of CLI/API and real signed-attestation integration.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import textwrap

ROOT = Path(__file__).resolve().parent.parent
ARCHIVES = (
    "MineWorld-0.1.0-windows-x86_64.zip",
    "MineWorld-0.1.0-macos-universal.zip",
    "MineWorld-0.1.0-linux-x86_64.tar.gz",
    "MineWorld-0.1.0-linux-arm64.tar.gz",
)
SBOM = "MineWorld-0.1.0-sbom.spdx.json"
SPDX = "https://spdx.dev/Document/v2.3"


def attestation_block(workflow: str) -> str:
    """Read the literal block from this workflow's established step indentation."""
    step = workflow.split("      - name: Verify each attestation (", 1)[1]
    body = step.split("        run: |\n", 1)[1].split("      - name:", 1)[0]
    return textwrap.dedent(body).strip()


def run_case(block: str, event: str, missing: tuple[str, str] | None) -> list[str]:
    # Producer contract: raw SBOM has provenance; only archives have SPDX predicates.
    subjects = [(name, "provenance") for name in (*ARCHIVES, SBOM, "SHA256SUMS")]
    subjects += [(name, SPDX) for name in ARCHIVES]
    expected = []
    for name, predicate in subjects:
        argv = ["attestation", "verify", name, "--repo", "yuema137/MineWorld"]
        if predicate != "provenance":
            argv += ["--predicate-type", predicate]
        argv += ["--signer-workflow", "yuema137/MineWorld/.github/workflows/release.yml"]
        if event == "release":
            argv += ["--source-ref", "refs/tags/v0.1.0"]
        expected.append(argv)
    with tempfile.TemporaryDirectory(prefix="publish-attestation-") as scratch:
        folder = Path(scratch)
        assets = folder / "assets"
        assets.mkdir()
        for name in (*ARCHIVES, SBOM, "SHA256SUMS"):
            (assets / name).touch()
        log = folder / "calls.jsonl"
        # The shell function intercepts every gh call, including unexpected commands. It delegates
        # only argument checking to this Python executable, so no installed gh can be reached.
        checker = folder / "check.py"
        checker.write_text(textwrap.dedent('''\
            import json, os, sys
            args = sys.argv[1:]
            with open(os.environ["CALL_LOG"], "a") as out:
                out.write(json.dumps(args) + "\\n")
            allowed = json.loads(os.environ["ALLOWED"])
            sys.exit(0 if args in allowed else 1)
            '''), encoding="utf-8")
        allowed = [argv for argv, subject in zip(expected, subjects) if subject != missing]
        env = {**os.environ, "RUNNER_TEMP": scratch, "EVENT": event, "TAG": "v0.1.0" if event == "release" else "test-42",
               "REPO": "yuema137/MineWorld", "CALL_LOG": str(log),
               "ALLOWED": json.dumps(allowed), "CHECKER": str(checker)}
        env["PYTHON_EXE"] = sys.executable
        shell = 'gh() { "$PYTHON_EXE" "$CHECKER" "$@"; }; export -f gh\n' + block
        result = subprocess.run(["bash", "--noprofile", "--norc", "-c", shell], env=env,
                                capture_output=True, text=True, timeout=10)
        calls = [json.loads(line) for line in log.read_text().splitlines()] if log.exists() else []
    label = f"{event}, missing={missing}"
    if missing is None:
        if result.returncode or sorted(calls) != sorted(expected):
            return [f"{label}: expected all ten restricted verification calls; rc={result.returncode}, calls={calls}"]
    else:
        failed_call = expected[subjects.index(missing)]
        if result.returncode == 0 or not calls or calls[-1] != failed_call:
            return [f"{label}: missing attestation must stop the block at {failed_call}; rc={result.returncode}"]
    return []


def main() -> int:
    workflow = (ROOT / ".github/workflows/publish.yml").read_text(encoding="utf-8")
    block = attestation_block(workflow)
    missing_cases = [(name, "provenance") for name in (*ARCHIVES, SBOM, "SHA256SUMS")]
    missing_cases += [(name, SPDX) for name in ARCHIVES]
    failures = []
    for event in ("release", "workflow_dispatch"):
        for missing in (None, *missing_cases):
            failures.extend(run_case(block, event, missing))
    for failure in failures:
        print(f"publish-attestations: FAIL: {failure}")
    print(f"publish-attestations: {'FAILED' if failures else 'PASS'} (22 offline cases; real A-R9 NOT RUN)")
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
