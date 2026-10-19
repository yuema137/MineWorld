"""`cast.py wardrobe <sub>` — the pilot's command line (design §4.3, last column; PIPELINE.md §16).

Paid subcommands (`image`, `meshy`) read their caps from the recipe and take no cap argument (§4.6 rule 1). A
`--dry-run` prints what would be sent and the admission, reads no key and sends nothing.

Scratch layout (outside the repository, `MINEWORLD_CAST_SCRATCH`):

    credits.json, images.json                 the Meshy ledger and the image-call log
    subjects/<key>/base_layer/                R-1 candidates and sidecars
    subjects/<key>/input/                     the picked base-layer image masked (stage 3) + mask.json
    subjects/<key>/meshy/body-<n>/            R-2 downloads
    garments/<id>/reference/                  R-7 candidates
    garments/<id>/input/                      the picked reference masked + mask.json
    garments/<id>/meshy/<id>-<n>/             R-8 downloads
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
from collections.abc import Mapping
from pathlib import Path
from typing import cast

from mwcast import meshy
from mwcast.records import Mask, RecordError
from mwcast.scratch import ScratchError, resolve_scratch
from mwcast.wardrobe import budget, images
from mwcast.wardrobe.recipe import CAST_OUT_ENV, Pick, Recipe, load_recipe

REPO_ROOT = Path(__file__).resolve().parents[4]
PIPELINE_ROOT = REPO_ROOT / "tools" / "character_pipeline"
DEFAULT_RECIPE = PIPELINE_ROOT / "recipes" / "s32p-pilot.yaml"
# Stage 3 settings for wardrobe images: PIPELINE.md §4.1's defaults with the automatic crop (no fit.yaml
# exists for a base-layer or garment image).
WARDROBE_MASK = Mask(
    session="birefnet-portrait", crop_box=None, fg_ratio=0.85, size_px=1024, alpha_threshold=16
)


def add_parser(subparsers: argparse._SubParsersAction[argparse.ArgumentParser]) -> None:  # pyright: ignore[reportPrivateUsage]
    parser = subparsers.add_parser("wardrobe", help="the S32-p wardrobe pilot (route B; PIPELINE.md §16)")
    parser.add_argument("--recipe", type=Path, default=DEFAULT_RECIPE)
    parser.add_argument("--scratch", type=Path, default=None)
    parser.add_argument("--cast-out", type=Path, default=None)
    sub = parser.add_subparsers(dest="wardrobe_command", required=True)
    sub.add_parser("check", help="validate the recipe and print its library and outfits")
    sub.add_parser("budget", help="the credit ledger against the recipe's budget (§4.6 reports)")
    image = sub.add_parser("image", help="R-1 or R-7 reference images (OpenAI, model pinned)")
    image.add_argument("what", choices=("base-layer", "garment"))
    image.add_argument("--subject", default=None)
    image.add_argument("--garment", default=None)
    image.add_argument("--count", type=int, default=None)
    image.add_argument("--clay", type=Path, default=None, help="R-6's clay render of the wearer (garments)")
    image.add_argument("--dry-run", action="store_true")
    mask_parser = sub.add_parser("mask", help="stage 3 on the picked image (tool venv)")
    mask_parser.add_argument("what", choices=("body", "garment"))
    mask_parser.add_argument("--subject", default=None)
    mask_parser.add_argument("--garment", default=None)
    mesh = sub.add_parser("meshy", help="R-2 or R-8 Meshy image-to-3D under the pilot ledger")
    mesh.add_argument("what", choices=("body", "garment"))
    mesh.add_argument("--subject", default=None)
    mesh.add_argument("--garment", default=None)
    mesh.add_argument("--dry-run", action="store_true")


def _cast_out(args: argparse.Namespace, environ: Mapping[str, str]) -> Path | None:
    if args.cast_out is not None:
        return Path(args.cast_out)
    value = environ.get(CAST_OUT_ENV, "")
    return Path(value) if value else None


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _check(recipe: Recipe) -> int:
    print(f"recipe {recipe.source}: pilot {recipe.pilot}, template v{recipe.template_version}")
    for key, subject in recipe.subjects.items():
        pick = "picked" if subject.base_layer_pick else "no base-layer pick yet"
        print(f"  subject {key}: {subject.body_phase}/{subject.asset_phase}; {pick}")
    for gid, garment in recipe.garments.items():
        slot = garment.slot or "hair"
        print(f"  {garment.kind} {gid}: {slot}, {garment.fit}, worn first by {garment.wearer}")
    for (key, name), outfit in recipe.outfits.items():
        print(f"  outfit {key} {name}: {', '.join(outfit.garments)}; hair {outfit.hair}")
    return 0


def _budget(recipe: Recipe, scratch: Path) -> int:
    ledger = meshy.load_ledger(scratch / "credits.json")
    kept = _kept(scratch)
    for line in budget.summary(ledger, recipe.budget, kept):
        print(line)
    log = images.load_calls(scratch / "images.json")
    print(f"image calls {log.calls} of {recipe.image_model.call_cap}")
    return 0


def _kept(scratch: Path) -> dict[str, bool]:
    """`kept.json` in the scratch root: the raw-gate verdicts the agent recorded per asset (`<kind>:<id>`)."""
    path = scratch / "kept.json"
    if not path.exists():
        return {}
    raw: object = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(raw, dict):
        return {}
    return {str(k): bool(v) for k, v in cast(dict[str, object], raw).items()}


def _target(args: argparse.Namespace, what: str) -> str:
    value = args.subject if what in ("base-layer", "body") else args.garment
    if not value:
        flag = "--subject" if what in ("base-layer", "body") else "--garment"
        raise RecordError(f"wardrobe {what}: {flag} is required")
    return str(value)


def _picked_input(base: Path, pick: Pick) -> Path:
    """The masked input of a pick, checked against `mask.json` (which records the pick it was made from)."""
    record_path = base / "input" / "mask.json"
    masked = base / "input" / "masked.png"
    if not record_path.is_file() or not masked.is_file():
        raise RecordError(f"{masked}: no masked input; run `wardrobe mask` first (stage 3)")
    record = cast(dict[str, object], json.loads(record_path.read_text(encoding="utf-8")))
    if record.get("source_sha256") != pick.sha256:
        raise RecordError(
            f"{record_path}: made from another image than the recorded pick {pick.sha256[:12]}…"
        )
    if record.get("masked_sha256") != _sha256(masked):
        raise RecordError(f"{masked}: changed since stage 3 wrote it")
    return masked


def _base(recipe: Recipe, scratch: Path, what: str, target: str) -> tuple[Path, Pick]:
    if what in ("body", "base-layer"):
        recipe.subject(target)
        return scratch / "subjects" / target, recipe.require_base_layer_pick(target)
    return scratch / "garments" / target, recipe.require_reference_pick(target)


def _verified_pick(base: Path, pick: Pick, folder: str) -> Path:
    path = base / folder / pick.file
    if not path.is_file() or _sha256(path) != pick.sha256:
        raise RecordError(f"{path}: missing, or not the recorded pick (SHA-256 {pick.sha256[:12]}…)")
    return path


def _mask(recipe: Recipe, scratch: Path, args: argparse.Namespace) -> int:
    from mwcast import mask

    what = str(args.what)
    target = _target(args, what)
    base, pick = _base(recipe, scratch, what, target)
    source = _verified_pick(base, pick, "base_layer" if what == "body" else "reference")
    out = base / "input" / "masked.png"
    box = mask.run_mask_with(WARDROBE_MASK, source, out)
    record = {"source": str(source), "source_sha256": pick.sha256, "masked_sha256": _sha256(out), "crop": box}
    (base / "input" / "mask.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    print(f"wardrobe mask: wrote {out} from {source.name} (crop {box})")
    return 0


def _meshy(recipe: Recipe, scratch: Path, args: argparse.Namespace) -> int:
    what = str(args.what)
    target = _target(args, what)
    base, pick = _base(recipe, scratch, what, target)
    masked = _picked_input(base, pick)
    if what == "body":
        kind, phase = "body", recipe.phase_of(target, body=True)
    else:
        kind, phase = recipe.garment(target).kind, recipe.phase_of(target, body=False)
    asset = budget.asset_name(kind, target)
    ledger_path = scratch / "credits.json"
    ledger = meshy.load_ledger(ledger_path)
    admission = budget.admit(ledger, recipe.budget, phase=phase, asset=asset, kind=kind)
    print(
        f"wardrobe meshy: admitted {asset} (phase {phase}, attempt {admission.attempt}, projection "
        f"{admission.projection}); spent {ledger.spent} of {recipe.budget.cap}"
    )
    if args.dry_run:
        plan = meshy.plan(masked, {}, ledger, recipe.budget.cap)
        print(
            f"    dry run: {plan.endpoint}; input {plan.image_path} sha256 {plan.image_sha256}; "
            "no request made"
        )
        return 0
    run_dir = base / "meshy" / f"{target}-{admission.attempt}"
    outcome = meshy.run_live(
        image=masked,
        overrides={},
        run_dir=run_dir,
        ledger_path=ledger_path,
        cap=recipe.budget.cap,
        transport=meshy.UrllibTransport(),
        projection=admission.projection,
        charge=budget.reconcile,
        phase=phase,
        asset=asset,
    )
    after = meshy.load_ledger(ledger_path)
    entry = after.entries[-1]
    print(
        f"wardrobe meshy: task {outcome.task_id} {outcome.status}; measured {entry.measured}, balance delta "
        f"{entry.balance_delta}, charged {entry.charged}; spent {after.spent} of {recipe.budget.cap}"
    )
    if entry.finding:
        print(f"    finding: {entry.finding}")
    for threshold in budget.reports_crossed(ledger.spent, after.spent, recipe.budget):
        print(f"    REPORT DUE: the ledger crossed {threshold} credits (§4.6 rule 5)")
    return 0 if outcome.status == "SUCCEEDED" else 1


def _image(recipe: Recipe, scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    what = str(args.what)
    target = _target(args, what)
    template = images.checked_template(recipe, REPO_ROOT)
    style = recipe.style_anchor.verified(REPO_ROOT, None)
    cast_out = _cast_out(args, environ)
    if what == "base-layer":
        subject = recipe.subject(target)
        identity = subject.identity.verified(REPO_ROOT, cast_out)
        prompt = images.base_layer_prompt(template)
        references = [identity, style]
        out = scratch / "subjects" / target / "base_layer"
        name, count = "base_layer", int(args.count or 3)
    else:
        garment = recipe.garment(target)
        if garment.source != "meshy" and garment.fit != "tight":
            raise RecordError(f"{target}: no reference image is made for it")
        if args.clay is None:
            raise RecordError("wardrobe image garment: --clay <R-6 render of the wearer> is required")
        clay = Path(args.clay)
        identity = recipe.subject(garment.wearer).identity.verified(REPO_ROOT, cast_out)
        prompt = images.garment_prompt(template, garment)
        references = [clay, identity, style]
        out = scratch / "garments" / target / "reference"
        name, count = "reference", int(args.count or 2)
    log = images.load_calls(scratch / "images.json")
    images.admit_calls(log, count, recipe.image_model.call_cap)
    images.check_generator_fix(REPO_ROOT, recipe.image_model.generator_fix)
    print(
        f"wardrobe image {what} {target}: {count} calls ({log.calls} of {recipe.image_model.call_cap} used)"
    )
    print(f"    model {recipe.image_model.model}; references {[str(r) for r in references]}")
    if args.dry_run:
        print("    dry run: prompt follows; nothing sent")
        print(prompt)
        return 0
    written = images.generate(
        recipe=recipe,
        repo_root=REPO_ROOT,
        scratch=scratch,
        environ=environ,
        prompt=prompt,
        references=references,
        out=out,
        name=name,
        count=count,
        label=f"{what}:{target}",
    )
    for path in written:
        print(f"    wrote {path} (sidecar model checked)")
    return 0


def run(args: argparse.Namespace, environ: Mapping[str, str] | None = None) -> int:
    env = os.environ if environ is None else environ
    command = str(args.wardrobe_command)
    try:
        recipe = load_recipe(Path(args.recipe))
        if command == "check":
            return _check(recipe)
        scratch = resolve_scratch(str(args.scratch) if args.scratch else None, env, REPO_ROOT)
        scratch.mkdir(parents=True, exist_ok=True)
        if command == "budget":
            return _budget(recipe, scratch)
        if command == "image":
            return _image(recipe, scratch, args, env)
        if command == "mask":
            return _mask(recipe, scratch, args)
        if command == "meshy":
            return _meshy(recipe, scratch, args)
    except (RecordError, ScratchError, budget.BudgetRefusal, images.ImageError, meshy.MeshyError) as exc:
        print(f"wardrobe {command}: {exc}", file=sys.stderr)
        return 1
    print(f"wardrobe {command}: not yet implemented", file=sys.stderr)
    return 2
