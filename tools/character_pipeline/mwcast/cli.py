"""Command-line parsing for `cast.py` (PIPELINE.md §3).

Each subcommand is wired by the commit that implements it. Until then it reports that it is not yet
implemented and returns 2, so a half-built tool cannot pass for a finished one.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
from collections.abc import Sequence
from pathlib import Path
from typing import cast

from mwcast import blender as blender_cli
from mwcast import compare, ground, identifiers, mask, meshy, pipeline, pose_sheet, standard, texture
from mwcast.brief import parse_brief
from mwcast.gates import GateResult, Verdict, run_static_gates
from mwcast.glb import GlbError, read_glb
from mwcast.records import RecordError, load_record, overrides, parse_fit, parse_generation_log
from mwcast.scratch import ScratchError, resolve_scratch
from mwcast.wardrobe import cli as wardrobe_cli

REPO_ROOT = Path(__file__).resolve().parents[3]
MODEL_LIBRARY = REPO_ROOT / "clients/3d-spike/assets/models"

SUBCOMMANDS: dict[str, str] = {
    "gate": "G-1…G-6 and G-14 on one export",
    "check-characters": "validate every committed character",
    "run": "stages 6–10 for one character",
    "compare": "parity comparison of two exports",
    "compare-texture": "texel or mask comparison of two images",
    "mask": "stage 3, the mask",
    "meshy": "stage 4, the Meshy client (a live run needs the key in the environment)",
    "identifiers": "AI-identifier inspection of downloads",
    "pose-sheet": "the step-26 §6.4 pose sheet of one rigged character (Godot 4.7, windowed)",
    "ground": "the ground contact of every body the 3D client draws (Godot 4.7, headless; §15.1)",
    "contact": "C-1 limb in trunk at rest, and C-2 garment layer order with --outfit (S36-P1)",
    "contact-items": "C-3 floor rest and C-4 part overlap over every prop of a model library (S36-P1)",
    "walk-sheets": "the 2D walk atlas of every cast body, from its dumped Walk cycle (Blender)",
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
        if name == "run":
            subparser.add_argument("--fit", type=Path, required=True)
            subparser.add_argument("--download", type=Path, required=True)
            subparser.add_argument("--scratch", type=Path, default=None)
            subparser.add_argument("--from", dest="start", default=None, metavar="STAGE")
            subparser.add_argument("--to", dest="stop", default=None, metavar="STAGE")
            subparser.add_argument("--run-id", default=time.strftime("%Y%m%dT%H%M%S"))
            subparser.add_argument("--allow-blender", default=None, metavar="FIRST_LINE")
        if name == "compare":
            subparser.add_argument("candidate", type=Path)
            subparser.add_argument("accepted", type=Path)
        if name == "identifiers":
            subparser.add_argument("files", nargs="+", type=Path)
        if name == "mask":
            subparser.add_argument("--fit", type=Path, required=True)
            subparser.add_argument("--reference", type=Path, required=True)
            subparser.add_argument("--out", type=Path, required=True)
        if name == "pose-sheet":
            subparser.add_argument("asset", type=Path)
            subparser.add_argument("--out", type=Path, required=True)
            subparser.add_argument("--character-yaml", type=Path, default=None)
            subparser.add_argument(
                "--manifest",
                type=Path,
                default=None,
                help="a cast manifest: build the subject as the client does",
            )
            subparser.add_argument("--label", default=None)
            subparser.add_argument("--project", type=Path, default=None)
            subparser.add_argument("--grip", action="store_true")
            subparser.add_argument("--plant-seat-gap", type=float, default=0.0, metavar="METRES")
            subparser.add_argument("--keep-dumps", action="store_true", help="keep <out>/dumps/*.mwcf")
            subparser.add_argument(
                "--plant-arm-into-torso",
                action="store_true",
                help="G-9w plant: the right arm through the trunk",
            )
            subparser.add_argument(
                "--outfit", type=Path, default=None, help="S32-p outfit manifest (W-4w, S36-P3)"
            )
            subparser.add_argument(
                "--pair", type=Path, default=None, help="S32-p outfit B, beside A and Vera"
            )
        if name == "walk-sheets":
            subparser.add_argument(
                "--poses", type=Path, required=True, help="cast.py ground --dump-pose's poses/"
            )
            subparser.add_argument("--out", type=Path, required=True)
            subparser.add_argument("--only", nargs="*", default=None, metavar="ID")
            subparser.add_argument("--frames", type=int, default=12)
            subparser.add_argument("--write", action="store_true", help="copy into the 2D pack and bind")
        if name == "ground":
            subparser.add_argument("--out", type=Path, required=True)
            subparser.add_argument("--only", nargs="*", default=None, metavar="ID")
            subparser.add_argument("--write", action="store_true")
            subparser.add_argument("--dump-pose", action="store_true")
            subparser.add_argument(
                "--plant-sole-offset",
                type=float,
                default=0.0,
                metavar="METRES",
                help="S36-P2 plant: raise the floor reference by this much (the body reads as sunk)",
            )
            subparser.add_argument(
                "--plant-stance-lift",
                type=float,
                default=0.0,
                metavar="METRES",
                help="S36-P2 plant: raise the left foot by this much in its stance frames",
            )
            subparser.add_argument(
                "--plant-arm-adduct",
                type=float,
                default=0.0,
                metavar="DEGREES",
                help="S36-CC10 plant: turn both upper arms toward the body by this much in every frame",
            )
            subparser.add_argument("--keep-dumps", action="store_true", help="keep <out>/dumps/*.mwcf")
        if name == "contact":
            subparser.add_argument("asset", type=Path)
            subparser.add_argument("--outfit", type=Path, default=None, help="outfit_manifest.json (C-2)")
            subparser.add_argument("--tolerances", type=Path, default=None, help="a pack's tightening file")
        if name == "contact-items":
            subparser.add_argument("--root", type=Path, default=MODEL_LIBRARY, help="the model library")
            subparser.add_argument("--tolerances", type=Path, default=None, help="a pack's tightening file")
        if name == "compare-texture":
            subparser.add_argument("candidate", type=Path)
            subparser.add_argument("accepted", type=Path)
            subparser.add_argument("--alpha-iou", action="store_true")
    wardrobe_cli.add_parser(subparsers)
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


# Files only a generation run writes: their presence without fit.yaml is a broken character directory.
GENERATION_RECORDS = ("generation_log.yaml", "character.yaml", "rederive_report.json")


def _check_characters(args: argparse.Namespace) -> int:
    root = Path(args.root)
    directories = sorted(p for p in root.glob("presentation/*/3D/characters/cast/*") if p.is_dir())
    failures = 0
    for directory in directories:
        name = directory.name
        if not (directory / "fit.yaml").exists():
            # Stage 1 (casting): a brief and its prompt packs exist before any generation. Such a
            # directory holds no generation record, and its brief must be valid (PIPELINE.md §§2-3).
            records = [r for r in GENERATION_RECORDS if (directory / r).exists()]
            if records:
                print(f"{name}: FAIL: {', '.join(records)} without fit.yaml")
                failures += 1
            elif not (directory / "brief.yaml").exists():
                print(f"{name}: FAIL: neither fit.yaml nor brief.yaml")
                failures += 1
            else:
                try:
                    parse_brief(directory / "brief.yaml", name)
                except RecordError as exc:
                    print(f"{name}: FAIL: {exc}")
                    failures += 1
                    continue
                print(f"{name}: brief only (no fit.yaml yet)")
            continue
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
            problem = _manifest_problem(directory / "character.yaml", name, fit.stature_m, root)
            if problem:
                print(f"{name}: FAIL: {problem}")
                failures += 1
                continue
            # The static gates ran on the export at generation (records above); the assets are Git LFS objects
            # that CI does not fetch, and the manifest's client fields are checked by scripts/check_cast.py.
            print(f"{name}: character (manifest consistent with fit.yaml); overrides {overrides(fit)}")
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


def _mask(args: argparse.Namespace) -> int:
    fit_path = Path(args.fit)
    try:
        fit = parse_fit(load_record(fit_path), fit_path)
    except RecordError as exc:
        print(f"mask: {exc}", file=sys.stderr)
        return 1
    box = mask.run_mask(fit, Path(args.reference), Path(args.out))
    print(f"mask: wrote {args.out} from crop {box}")
    return 0


def _compare_texture(args: argparse.Namespace) -> int:
    candidate = Path(args.candidate)
    accepted = Path(args.accepted)
    try:
        if args.alpha_iou:
            report = texture.compare_masks(candidate, accepted)
            print(
                f"mask: alpha IoU {report.alpha_iou:.5f}; RGB mean absolute difference "
                f"{report.rgb_mean_absolute_difference:.4f} / 255"
            )
            return 0
        texels = texture.compare_texels(candidate, accepted)
    except ValueError as exc:
        print(f"compare-texture: {exc}", file=sys.stderr)
        return 1
    verdict = "PASS" if texels.share >= standard.PARITY_TEXEL_SHARE else "FAIL"
    print(
        f"texels: {texels.within_tolerance} of {texels.texels} within 2/255 ({texels.share:.5f}); {verdict}"
    )
    return 0 if verdict == "PASS" else 1


def _run(args: argparse.Namespace) -> int:
    fit_path = Path(args.fit)
    try:
        fit = parse_fit(load_record(fit_path), fit_path)
        scratch = resolve_scratch(str(args.scratch) if args.scratch else None, os.environ, REPO_ROOT)
        blender_path = blender_cli.find_blender(os.environ, sys.platform)
        version = blender_cli.read_version_line(blender_path)
        allowed = blender_cli.check_pin(version, args.allow_blender)
        result = pipeline.run(
            fit=fit,
            fit_path=fit_path,
            download_dir=Path(args.download),
            scratch=scratch,
            run_id=str(args.run_id),
            start=args.start,
            stop=args.stop,
            blender=blender_path,
            blender_version=version,
            allowed_non_pin=allowed,
            executor=blender_cli.run_command,
        )
    except (RecordError, ScratchError, blender_cli.BlenderError, pipeline.PipelineError) as exc:
        print(f"run: {exc}", file=sys.stderr)
        return 1
    print(f"run: stages {', '.join(result.stages_run)}; manifest {result.manifest}")
    for stage, reason in result.skipped:
        print(f"run: skipped {stage}: {reason}")
    return 0


def _compare(args: argparse.Namespace) -> int:
    try:
        report = compare.compare_files(Path(args.candidate), Path(args.accepted))
    except GlbError as exc:
        print(f"compare: {exc}", file=sys.stderr)
        return 1
    for row in report.rows:
        for reason in row.reasons:
            print(f"compare: {row.quantity}: {reason}")
        print(f"{row.quantity}: {row.worst} (bound {row.bound}); {row.verdict.value}")
    print("compare: material name and provenance reported, not compared (F-B1-6)")
    print(f"compare: {'PASS' if report.passed else 'FAIL'}")
    return 0 if report.passed else 1


def _pose_sheet(args: argparse.Namespace) -> int:
    label = args.label or Path(args.asset).stem
    if args.character_yaml is not None:
        try:
            record = load_record(Path(args.character_yaml))
        except RecordError as exc:
            print(f"pose-sheet: {exc}", file=sys.stderr)
            return 1
        if isinstance(record, dict):
            fields = cast(dict[str, object], record)
            label = str(fields.get("character", fields.get("person", label)))
    try:
        sheet = pose_sheet.run(
            repo_root=REPO_ROOT,
            subject=Path(args.asset),
            out=Path(args.out),
            label=label,
            grip=bool(args.grip),
            plant_seat_gap=float(args.plant_seat_gap),
            environ=os.environ,
            project=args.project,
            outfit=args.outfit,
            pair=args.pair,
            manifest=args.manifest,
            plant_arm=bool(args.plant_arm_into_torso),
            keep_dumps=bool(args.keep_dumps),
        )
    except pose_sheet.PoseSheetError as exc:
        print(f"pose-sheet: {exc}", file=sys.stderr)
        return 1
    report = json.loads((Path(args.out) / "report.json").read_text())
    flagged = [f["id"] for f in report["frames"] if not f["pass"]]
    if isinstance(report.get("ground"), dict) and not report["ground"].get("pass", False):
        flagged.append("ground")
    print(f"pose-sheet: {len(report['frames'])} frames; flagged {flagged or 'none'}; sheet {sheet}")
    return 1 if flagged else 0


def _walk_sheets(args: argparse.Namespace) -> int:
    from mwcast import walk_sheets

    try:
        walk_sheets.run(
            REPO_ROOT,
            Path(args.poses),
            Path(args.out),
            args.only,
            int(args.frames),
            bool(args.write),
            os.environ,
        )
    except (walk_sheets.WalkSheetError, blender_cli.BlenderError) as exc:
        print(f"walk-sheets: {exc}", file=sys.stderr)
        return 1
    return 0


def _ground(args: argparse.Namespace) -> int:
    chosen = ground.bodies(REPO_ROOT)
    if args.only:
        chosen = [b for b in chosen if b.id in set(args.only)]
    plants = ground.Plants(
        float(args.plant_sole_offset), float(args.plant_stance_lift), float(args.plant_arm_adduct)
    )
    if plants.any and args.write:
        print("ground: a planted run is never recorded; drop --write or the plant", file=sys.stderr)
        return 2
    try:
        report = ground.run(
            REPO_ROOT,
            Path(args.out),
            chosen,
            os.environ,
            bool(args.dump_pose),
            plants=plants,
            keep_dumps=bool(args.keep_dumps),
        )
    except (ground.GroundError, pose_sheet.PoseSheetError) as exc:
        print(f"ground: {exc}", file=sys.stderr)
        return 1
    (Path(args.out) / "judged.json").write_text(json.dumps(report, indent=2) + "\n")
    print(ground.table(report))
    for line in ground.reasons(report):
        print(f"ground: {line}")
    rows = cast(list[dict[str, object]], report["bodies"])
    failing = [str(r["id"]) for r in rows if not r["pass"]]
    if args.write:
        for path in ground.write(REPO_ROOT, report, chosen):
            print(f"ground: wrote {path}")
    print(f"ground: {len(rows)} bodies; failing {failing or 'none'}")
    return 1 if failing else 0


def _contact(args: argparse.Namespace) -> int:
    # numpy and the mesh libraries are imported here, not at the top: the stdlib-only subcommands
    # (check-characters runs from a bare interpreter in CI) must not need them (S36-P1).
    try:
        from mwcast import contact_gates
        from mwcast.tolerances import ToleranceError, Tolerances
    except ImportError as exc:
        print(f"contact C-1: FAIL\n    reason: cannot look: {exc.name} is not installed", file=sys.stderr)
        return 1
    asset = Path(args.asset)
    try:
        tolerances = Tolerances.load()
        if args.tolerances is not None:
            tolerances = tolerances.tightened_by(Path(args.tolerances))
        manifest = None if args.outfit is None else contact_gates.OutfitManifest.load(Path(args.outfit))
        glb = read_glb(asset)
    except (GlbError, ToleranceError, ValueError, OSError) as exc:
        print(f"contact: {exc}", file=sys.stderr)
        return 1
    results = contact_gates.run_body_gates(glb, tolerances, manifest)
    return 0 if _print_results(asset.name, results) else 1


def _contact_items(args: argparse.Namespace) -> int:
    """Exit 0 when every prop passes or fails only by a finding recorded in contact.json; otherwise name every
    failing prop and exit 1."""
    try:
        from mwcast import contact_gates
        from mwcast.tolerances import ToleranceError, Tolerances
    except ImportError as exc:
        print(
            f"contact-items C-3: FAIL\n    reason: cannot look: {exc.name} is not installed", file=sys.stderr
        )
        return 1
    try:
        tolerances = Tolerances.load()
        if args.tolerances is not None:
            tolerances = tolerances.tightened_by(Path(args.tolerances))
        results = contact_gates.run_item_gates(Path(args.root), tolerances)
    except (ToleranceError, ValueError, OSError) as exc:
        print(f"contact-items: {exc}", file=sys.stderr)
        return 1
    failing: list[str] = []
    known: list[str] = []
    for slug, gate_results in results.items():
        _print_results(slug, gate_results)
        reasons = [r for g in gate_results if g.verdict is Verdict.FAIL for r in g.reasons]
        if any(not r.startswith(contact_gates.KNOWN) for r in reasons):
            failing.append(slug)
        elif reasons:
            known.append(slug)
    print(
        f"contact-items: {len(results)} props; failing {failing or 'none'}; "
        f"recorded findings still open {known or 'none'}"
    )
    return 1 if failing else 0


def _manifest_problem(path: Path, name: str, stature_m: float, root: Path) -> str:
    """S26-c1: a cast character.yaml names this character, its fit's stature and an existing client scene."""
    try:
        raw: object = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as exc:
        return f"{path}: not JSON-compatible YAML ({exc})"
    if not isinstance(raw, dict):
        return f"{path}: not an object"
    manifest = cast(dict[str, object], raw)
    if manifest.get("id") != name:
        return f"{path}: id {manifest.get('id')!r} is not the directory {name!r}"
    stature = manifest.get("stature_m")
    if not isinstance(stature, (int, float)) or abs(float(stature) - stature_m) > 1e-6:
        return f"{path}: stature_m {stature!r} differs from fit.yaml's {stature_m}"
    scene = manifest.get("scene")
    if not isinstance(scene, str) or not scene.startswith("res://"):
        return f"{path}: scene {scene!r} is not a res:// path"
    if not (root / "clients" / "3d-spike" / scene.removeprefix("res://")).is_file():
        return f"{path}: scene {scene} does not exist under clients/3d-spike"
    return ""


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    command = str(args.command)
    if command == "run":
        return _run(args)
    if command == "compare":
        return _compare(args)
    if command == "gate":
        return _gate(args)
    if command == "check-characters":
        return _check_characters(args)
    if command == "meshy":
        return _meshy(args)
    if command == "identifiers":
        return _identifiers(args)
    if command == "mask":
        return _mask(args)
    if command == "compare-texture":
        return _compare_texture(args)
    if command == "pose-sheet":
        return _pose_sheet(args)
    if command == "contact":
        return _contact(args)
    if command == "contact-items":
        return _contact_items(args)
    if command == "wardrobe":
        return wardrobe_cli.run(args)
    if command == "ground":
        return _ground(args)
    if command == "walk-sheets":
        return _walk_sheets(args)
    print(f"cast.py {command}: not yet implemented in this commit", file=sys.stderr)
    return 2
