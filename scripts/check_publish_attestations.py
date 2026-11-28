#!/usr/bin/env python3
"""Offline regression for publish.yml's attestation command boundary, not cryptographic verification.

Run the real shell block against the producer contract read from release.yml's `attest` steps (R-b is the
source of truth for which release asset carries which attestation), so the two workflows cannot drift
apart while this test stays green. Only the asset names of a release are fixed here. No gh executable,
token or network is used. Hosted A-R9 remains the owner of CLI/API and real signed-attestation integration.
"""

from __future__ import annotations

from fnmatch import fnmatch
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
ASSETS = (*ARCHIVES, SBOM, "SHA256SUMS")
# release.yml's attest steps by name, and the predicate each one produces.
PRODUCER_STEPS = (("Provenance attestations", "provenance"), ("SBOM attestations", SPDX))


def producer_subjects(release: str) -> list[tuple[str, str]]:
    """(asset, predicate) for every attestation release.yml's attest steps make, from their subject-path globs."""
    subjects = []
    for step, predicate in PRODUCER_STEPS:
        parts = release.split(f"      - name: {step}\n", 1)
        if len(parts) != 2 or "subject-path: |\n" not in parts[1].split("      - name:", 1)[0]:
            raise SystemExit(f"publish-attestations: FAIL: release.yml has no '{step}' step with a subject-path list")
        listing = parts[1].split("      - name:", 1)[0].split("subject-path: |\n", 1)[1]
        patterns = []
        for line in listing.splitlines():
            if not line.startswith(" " * 12) or not line.strip():
                break
            patterns.append(line.strip().rsplit("/", 1)[-1])
        for pattern in patterns:
            if not any(fnmatch(name, pattern) for name in ASSETS):
                raise SystemExit(f"publish-attestations: FAIL: release.yml '{step}' subject {pattern} names no release asset")
        subjects += [(name, predicate) for name in ASSETS if any(fnmatch(name, p) for p in patterns)]
    return subjects


def attestation_block(workflow: str) -> str:
    """Read the literal block from this workflow's established step indentation."""
    step = workflow.split("      - name: Verify each attestation (", 1)[1]
    body = step.split("        run: |\n", 1)[1].split("      - name:", 1)[0]
    return textwrap.dedent(body).strip()


def run_case(block: str, subjects: list[tuple[str, str]], event: str, missing: tuple[str, str] | None) -> list[str]:
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
        for name in ASSETS:
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
            return [f"{label}: expected all {len(expected)} restricted verification calls; rc={result.returncode}, calls={calls}"]
    else:
        failed_call = expected[subjects.index(missing)]
        if result.returncode == 0 or not calls or calls[-1] != failed_call:
            return [f"{label}: missing attestation must stop the block at {failed_call}; rc={result.returncode}"]
    return []


def main() -> int:
    workflow = (ROOT / ".github/workflows/publish.yml").read_text(encoding="utf-8")
    block = attestation_block(workflow)
    subjects = producer_subjects((ROOT / ".github/workflows/release.yml").read_text(encoding="utf-8"))
    failures = []
    cases = 0
    for event in ("release", "workflow_dispatch"):
        for missing in (None, *subjects):
            failures.extend(run_case(block, subjects, event, missing))
            cases += 1
    for failure in failures:
        print(f"publish-attestations: FAIL: {failure}")
    print(f"publish-attestations: {'FAILED' if failures else 'PASS'} ({cases} offline cases, subjects from release.yml; "
          "real A-R9 NOT RUN)")
    return bool(failures)


if __name__ == "__main__":
    raise SystemExit(main())
