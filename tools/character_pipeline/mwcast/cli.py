"""Command-line parsing for `cast.py` (PIPELINE.md §3).

Each subcommand is wired by the commit that implements it. Until then it reports that it is not yet
implemented and returns 2, so a half-built tool cannot pass for a finished one.
"""

from __future__ import annotations

import argparse
import hashlib
import os
import sys
import time
from collections.abc import Sequence
from pathlib import Path

from mwcast import identifiers, meshy
from mwcast.gates import GateResult, Verdict, run_static_gates
from mwcast.glb import GlbError, read_glb
from mwcast.records import RecordError, load_record, overrides, parse_fit, parse_generation_log
from mwcast.scratch import ScratchError, resolve_scratch

REPO_ROOT = Path(__file__).resolve().parents[3]

SUBCOMMANDS: dict[str, str] = {
    "gate": "G-1…G-6 and G-14 on one export",
    "check-characters": "validate every committed character",
    "run": "stages 6–10 for one character",
    "compare": "parity comparison of two exports",
    "compare-texture": "texel or mask comparison of two images",
    "mask": "stage 3, the mask",
    "meshy": "stage 4, the Meshy client (a live run needs the key in the environment)",
    "identifiers": "AI-identifier inspection of downloads",
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="cast.py", description="MineWorld character pipeline (mwcast-1).")
    subparsers = parser.add_subparsers(dest="command", required=True)
    for name, help_text in SUBCOMMANDS.items():
        subparser = subparsers.add_parser(name, help=help_text)
        if name == "gate":
            subparser.add_argument("asset", type=Path)
            subparser.add_argument("--fit", type=Path, required=True)
            subparser.add_argument("--run-dir", type=Path, default=None)
            subparser.add_argument("--repo-root", type=Path, default=REPO_ROOT)
        if name == "check-characters":
            subparser.add_argument("--root", type=Path, default=REPO_ROOT)
        if name == "meshy":
            subparser.add_argument("--fit", type=Path, required=True)
            subparser.add_argument("--input", type=Path, required=True)
            subparser.add_argument("--scratch", type=Path, default=None)
            subparser.add_argument("--cap", type=int, required=True)
            subparser.add_argument("--run-id", default=time.strftime("%Y%m%dT%H%M%S"))
            subparser.add_argument("--dry-run", action="store_true")
        if name == "identifiers":
            subparser.add_argument("files", nargs="+", type=Path)
    return parser


def _print_results(name: str, results: Sequence[GateResult]) -> bool:
    all_pass = True
    for result in results:
        print(f"{name} {result.gate.value}: {result.verdict.value}")
        for line in result.looked_at:
            print(f"    looked at: {line}")
        for reason in result.reasons:
            print(f"    reason: {reason}")
        if result.verdict is Verdict.FAIL:
            all_pass = False
    return all_pass


def _gate(args: argparse.Namespace) -> int:
    asset = Path(args.asset)
    fit_path = Path(args.fit)
    repo_root = Path(args.repo_root)
    try:
        glb = read_glb(asset)
        fit = parse_fit(load_record(fit_path), fit_path)
    except (GlbError, RecordError) as exc:
        print(f"gate: {exc}", file=sys.stderr)
        return 1
    results = run_static_gates(glb, fit, repo_root, args.run_dir)
    return 0 if _print_results(asset.name, results) else 1


def _check_characters(args: argparse.Namespace) -> int:
    root = Path(args.root)
    directories = sorted(p for p in root.glob("presentation/*/3D/characters/cast/*") if p.is_dir())
    failures = 0
    for directory in directories:
        name = directory.name
        try:
            fit = parse_fit(load_record(directory / "fit.yaml"), directory / "fit.yaml")
            parse_generation_log(
                load_record(directory / "generation_log.yaml"), directory / "generation_log.yaml"
            )
        except RecordError as exc:
            print(f"{name}: FAIL: {exc}")
            failures += 1
            continue
        if (directory / "character.yaml").exists():
            print(f"{name}: FAIL: character.yaml present; its checks arrive with S26-c1")
            failures += 1
            continue
        extra = overrides(fit)
        print(f"{name}: records only (no character.yaml); overrides {extra}")
    print(f"{len(directories)} character directories checked; {failures} failing")
    return 0 if failures == 0 else 1


def _meshy(args: argparse.Namespace) -> int:
    fit_path = Path(args.fit)
    image = Path(args.input)
    try:
        fit = parse_fit(load_record(fit_path), fit_path)
        scratch = resolve_scratch(str(args.scratch) if args.scratch else None, os.environ, REPO_ROOT)
    except (RecordError, ScratchError) as exc:
        print(f"meshy: {exc}", file=sys.stderr)
        return 1
    digest = hashlib.sha256(image.read_bytes()).hexdigest()
    if digest != fit.source.input_sha256:
        print(
            f"meshy: refused: the input's SHA-256 is {digest}, but fit.source.input_sha256 is "
            f"{fit.source.input_sha256}",
            file=sys.stderr,
        )
        return 1
    ledger_path = scratch / "credits.json"
    try:
        if args.dry_run:
            projection = meshy.plan(image, {}, meshy.load_ledger(ledger_path), args.cap)
            print(f"meshy dry run: endpoint {projection.endpoint}")
            print(f"    input {projection.image_path} sha256 {projection.image_sha256} (matches fit)")
            print(f"    params {dict(projection.params)}")
            print(
                f"    projected credits {projection.projected_credits} within cap {args.cap}; no request made"
            )
            return 0
        outcome = meshy.run_live(
            image=image,
            overrides={},
            run_dir=scratch / "meshy" / args.run_id,
            ledger_path=ledger_path,
            cap=args.cap,
            transport=meshy.UrllibTransport(),
        )
    except meshy.MeshyError as exc:
        print(f"meshy: {exc}", file=sys.stderr)
        return 1
    print(
        f"meshy: task {outcome.task_id} ended {outcome.status}; consumed credits {outcome.consumed_credits}"
    )
    for name in outcome.files:
        print(f"    wrote {outcome.run_dir / name}")
    return 0 if outcome.status == "SUCCEEDED" else 1


def _identifiers(args: argparse.Namespace) -> int:
    for path in args.files:
        print(f"== {path}")
        try:
            findings = identifiers.inspect_file(Path(path))
        except (GlbError, OSError) as exc:
            print(f"    could not read: {exc}")
            continue
        if not findings:
            print("    nothing found")
        for finding in findings:
            print(f"    {finding.where}: {finding.kind}")
    return 0


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    command = str(args.command)
    if command == "gate":
        return _gate(args)
    if command == "check-characters":
        return _check_characters(args)
    if command == "meshy":
        return _meshy(args)
    if command == "identifiers":
        return _identifiers(args)
    print(f"cast.py {command}: not yet implemented in this commit", file=sys.stderr)
    return 2
