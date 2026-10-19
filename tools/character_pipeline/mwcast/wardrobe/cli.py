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
from mwcast.blender import BlenderError
from mwcast.gates import Verdict
from mwcast.glb import read_glb, skin_joint_names
from mwcast.records import Mask, RecordError
from mwcast.scratch import ScratchError, resolve_scratch
from mwcast.wardrobe import budget, fixtures, godot_runs, images, stages, wgates
from mwcast.wardrobe import garment_format as gf
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
    image.add_argument(
        "--gen-python", default=None, help=f"the tool venv's Python (else {images.GEN_PYTHON_ENV})"
    )
    mask_parser = sub.add_parser("mask", help="stage 3 on the picked image (tool venv)")
    mask_parser.add_argument("what", choices=("body", "garment"))
    mask_parser.add_argument("--subject", default=None)
    mask_parser.add_argument("--garment", default=None)
    mesh = sub.add_parser("meshy", help="R-2 or R-8 Meshy image-to-3D under the pilot ledger")
    mesh.add_argument("what", choices=("body", "garment"))
    mesh.add_argument("--subject", default=None)
    mesh.add_argument("--garment", default=None)
    mesh.add_argument("--dry-run", action="store_true")
    install = sub.add_parser(
        "install-mpfb", help="install MPFB2 2.0.18 into <scratch>/blender-user (zip checked)"
    )
    install.add_argument("--zip", type=Path, required=True)
    sub.add_parser("regions", help="R-5: write wardrobe/template/hm08_regions.json (Blender + MPFB2)")
    register = sub.add_parser(
        "register", help="R-4: register the hm08 template onto a body (Blender + MPFB2)"
    )
    who = register.add_mutually_exclusive_group(required=True)
    who.add_argument("--subject")
    who.add_argument("--fixture", help="a W-5 fixture body of the recipe (hidden test data)")
    register.add_argument("--body", type=Path, required=True, help="the subject's rigged body export (.glb)")
    register.add_argument("--name", default=None, help="output name (default: the subject)")
    register.add_argument("--stature-m", type=float, default=None, help="default: the recipe's stature")
    sub.add_parser("fixtures", help="W-5 fixture bodies and stock CC0 fixture garments (Blender + MPFB2)")
    sub.add_parser("fixture-check", help="fixture bindings (W-1) and the formula oracle against MPFB2's fits")
    clean = sub.add_parser("clean", help="R-8a/R-9 (or R-8b): from a Meshy garment download to garment.glb")
    clean.add_argument("--garment", required=True)
    clean.add_argument("--body", type=Path, required=True, help="the wearer's route-B body export (.glb)")
    clean.add_argument("--template", type=Path, required=True, help="the wearer's <subject>_template.bin")
    clean.add_argument("--corr", type=Path, default=None, help="the wearer's corr.bin (tight-fit only)")
    clean.add_argument("--front", type=Path, default=None, help="the reference image (tight-fit only)")
    bind = sub.add_parser("bind", help="R-10/R-11: MakeClothes binding and the hidden set of one garment")
    bind.add_argument("--id", required=True)
    bind.add_argument("--glb", type=Path, required=True, help="the garment, in the frame of --template")
    bind.add_argument("--template", type=Path, required=True, help="the template the garment sits on (.bin)")
    bind.add_argument(
        "--covers", required=True, help="comma-separated regions (R-11's hidden-set candidates)"
    )
    bind.add_argument("--out", type=Path, required=True)
    bind.add_argument("--hides-only", action="store_true")
    gate = sub.add_parser(
        "gate-outfit", help="G-1/G-2/G-3/G-14 per outfit, W-2, W-3 and V-1 on an outfit GLB"
    )
    gate.add_argument("--glb", type=Path, required=True)
    gate.add_argument("--manifest", type=Path, required=True)
    gate.add_argument("--json", type=Path, default=None, help="also write the results here")
    assemble = sub.add_parser("assemble", help="R-12 (a): the Blender reference fit and the outfit GLB")
    assemble.add_argument("--spec", type=Path, required=True, help="outfit spec JSON (PIPELINE.md §16)")
    assemble.add_argument("--out-dir", type=Path, required=True)
    w9 = sub.add_parser(
        "w9", help="W-9: fit garments in Godot (fit_at_load.gd) and compare with the Python fit"
    )
    w9.add_argument("--template-dir", type=Path, required=True, help="a `register` output folder")
    w9.add_argument("--name", required=True, help="its name (<name>_template.bin)")
    w9.add_argument(
        "--garment", action="append", required=True, metavar="ID=DIR", help="garment.glb + binding.bin"
    )
    w9.add_argument("--out", type=Path, required=True)
    w9.add_argument(
        "--reference-dir", type=Path, default=None, help="an `assemble` folder: compare with Blender"
    )
    preview = sub.add_parser("preview", help="flat-colour front/side/back renders of meshes (review only)")
    preview.add_argument("--item", action="append", required=True, metavar="PATH=R,G,B,A")
    preview.add_argument("--out", type=Path, required=True, help="output prefix (PNG files)")


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
        prompt = images.base_layer_prompt(template, subject.macros)
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
    gen_env = dict(environ)
    if args.gen_python:
        gen_env[images.GEN_PYTHON_ENV] = str(args.gen_python)
    written = images.generate(
        recipe=recipe,
        repo_root=REPO_ROOT,
        scratch=scratch,
        environ=gen_env,
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


def _blender_checked(environ: Mapping[str, str]) -> str:
    from mwcast import blender as blender_cli

    path = stages.default_blender(environ)
    version = blender_cli.read_version_line(Path(path))
    blender_cli.check_pin(version, None)
    return path


def _install(scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    zip_path = Path(args.zip)
    stages.check_zip(zip_path)
    blender = _blender_checked(environ)
    command = stages.install_command(stages.slot(environ), blender, zip_path)
    log = scratch / "logs" / "install-mpfb.log"
    code = stages.run_logged(command, stages.stage_environment(environ, scratch), log)
    version = stages.installed_version(scratch)
    print(f"wardrobe install-mpfb: exit {code}; installed {version}; log {log}")
    return 0 if code == 0 and version == stages.MPFB_VERSION else 1


def _regions(scratch: Path, environ: Mapping[str, str]) -> int:
    blender = _blender_checked(environ)
    out = scratch / "regions" / "hm08_regions.json"
    log = stages.run_stage(
        "regions",
        {"out": str(out)},
        scratch=scratch,
        run_dir=scratch / "regions",
        environ=environ,
        blender=blender,
    )
    stages.REGIONS_JSON.parent.mkdir(parents=True, exist_ok=True)
    stages.REGIONS_JSON.write_bytes(out.read_bytes())
    print(f"wardrobe regions: wrote {stages.REGIONS_JSON} (sha256 {_sha256(stages.REGIONS_JSON)}); log {log}")
    return 0


def _register(recipe: Recipe, scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    if args.fixture:
        fixture = recipe.fixtures.bodies[str(args.fixture)]
        name = str(args.name or f"fixture-{fixture.name}")
        macro_values: dict[str, object] = fixture.macros()
        recorded = fixture.stature_m
        run_dir = scratch / "fixtures" / "templates" / name
    else:
        subject = recipe.subject(str(args.subject))
        name = str(args.name or subject.key)
        macro_values = {
            "sex": subject.macros.sex,
            "apparent_age": subject.macros.apparent_age,
            "build": subject.macros.build,
        }
        recorded = subject.macros.stature_m
        run_dir = scratch / "subjects" / subject.key / "template" / name
    stature = float(args.stature_m) if args.stature_m is not None else recorded
    params: dict[str, object] = {
        "body": str(Path(args.body).resolve()),
        "macros": macro_values,
        "stature_m": stature,
        "out_dir": str(run_dir),
        "name": name,
        "regions": str(stages.REGIONS_JSON),
    }
    log = stages.run_stage(
        "register",
        params,
        scratch=scratch,
        run_dir=run_dir,
        environ=environ,
        blender=_blender_checked(environ),
    )
    # The weights' joint table in the body's glTF skin order: a joint index is a bind index (fit_at_load.gd).
    weights_path = run_dir / f"{name}_template_weights.bin"
    order = skin_joint_names(read_glb(Path(args.body)))
    gf.write_template_weights(weights_path, gf.remap_joints(gf.read_template_weights(weights_path), order))
    result = cast(dict[str, object], json.loads((run_dir / "register.json").read_text(encoding="utf-8")))
    print(f"wardrobe register: {name}: log {log}")
    regions = cast(dict[str, dict[str, object]], result["regions_template_to_body"])
    for region, row in regions.items():
        print(f"    {region:11s} n {row['n']} mean {row['mean_mm']} mm p95 {row['p95_mm']} mm")
    calibration = cast(dict[str, object], result["calibration"])
    print(f"    stature {calibration['stature_m']} m for target {calibration['target_m']} m")
    return 0


MPFB_ASSETS_ENV = "MINEWORLD_MPFB_ASSETS"


def _assets(environ: Mapping[str, str]) -> Path:
    value = environ.get(MPFB_ASSETS_ENV, "")
    if not value:
        raise RecordError(
            f"{MPFB_ASSETS_ENV} is not set: the folder of the stock CC0 MakeHuman assets (fixtures)"
        )
    return Path(value)


def _fixtures(recipe: Recipe, scratch: Path, environ: Mapping[str, str]) -> int:
    assets = _assets(environ)
    out = scratch / "fixtures"
    params: dict[str, object] = {
        "out_dir": str(out),
        "bodies": {
            name: {"macros": body.macros(), "stature_m": body.stature_m}
            for name, body in recipe.fixtures.bodies.items()
        },
        "garments": {gid: str(assets / g.mhclo) for gid, g in recipe.fixtures.garments.items()},
    }
    log = stages.run_stage(
        "fixtures", params, scratch=scratch, run_dir=out, environ=environ, blender=_blender_checked(environ)
    )
    print(f"wardrobe fixtures: {out}; log {log}")
    return 0


def _fixture_check(recipe: Recipe, scratch: Path, environ: Mapping[str, str]) -> int:
    assets = _assets(environ)
    out = scratch / "fixtures"
    failures = 0
    for gid, garment in recipe.fixtures.garments.items():
        mhclo = assets / garment.mhclo
        binding = fixtures.convert(
            mhclo, out / f"{gid}_on_neutral_mpfb.bin", out / gid / "garment.glb", out / gid / "binding.bin"
        )
        template_count = len(gf.read_template(out / "neutral_template.bin"))
        w1 = gf.check_binding(binding, template_count)
        print(
            f"{gid}: binding {len(binding.records)} records (garment.glb order); "
            f"W-1 {'PASS' if not w1 else 'FAIL'}"
        )
        failures += bool(w1)
        source = gf.parse_mhclo(mhclo.read_text(encoding="utf-8"), str(mhclo)).binding
        for body in ("neutral", *recipe.fixtures.bodies):
            template_file = out / (
                "neutral_template.bin" if body == "neutral" else f"{body}_exact_template.bin"
            )
            error = fixtures.oracle(
                source, gf.read_template(template_file), gf.read_template(out / f"{gid}_on_{body}_mpfb.bin")
            )
            verdict = "PASS" if error <= fixtures.ORACLE_M else "FAIL"
            failures += verdict == "FAIL"
            print(
                f"    oracle on {body}: max {1000 * error:.4f} mm "
                f"(bound {1000 * fixtures.ORACLE_M} mm) {verdict}"
            )
    return 0 if failures == 0 else 1


def _clean(recipe: Recipe, scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    garment = recipe.garment(str(args.garment))
    part_class = "hair" if garment.kind == "hair" else str(garment.slot)
    budget_tris = wgates.PART_BUDGET[part_class]
    texture_px = 1024
    base = scratch / "garments" / garment.id
    out = base / "clean"
    params: dict[str, object] = {
        "id": garment.id,
        "body": str(Path(args.body).resolve()),
        "regions": str(stages.REGIONS_JSON),
        "budget": budget_tris,
        "texture_px": texture_px,
        "out_dir": str(out),
    }
    if garment.fit == "tight":
        params.update(
            {
                "op": "tight",
                "corr": str(Path(args.corr).resolve()),
                "covers_regions": list(garment.covers_regions),
                "front_image": str(Path(args.front).resolve()),
                "back_image": None,
            }
        )
    else:
        runs = sorted((base / "meshy").glob(f"{garment.id}-*/model_glb.glb"))
        if not runs:
            raise RecordError(f"{garment.id}: no Meshy download under {base / 'meshy'}")
        params.update(
            {
                "op": "subtract",
                "download": str(runs[-1]),
                "template": str(Path(args.template).resolve()),
                "covers_regions": list(garment.covers_regions),
            }
        )
    log = stages.run_stage(
        "garment_clean",
        params,
        scratch=scratch,
        run_dir=out,
        environ=environ,
        blender=_blender_checked(environ),
    )
    print(
        f"wardrobe clean {garment.id}: {(out / 'clean.json').read_text(encoding='utf-8').strip()}; log {log}"
    )
    return 0


def _bind(recipe: Recipe, scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    covers = [r for r in str(args.covers).split(",") if r]
    unknown = [r for r in covers if r not in recipe.regions]
    if unknown:
        raise RecordError(f"wardrobe bind: unknown regions {unknown}")
    out = Path(args.out).resolve()
    blender = _blender_checked(environ)
    ops = ("hides",) if args.hides_only else ("bind", "hides")
    for op in ops:
        params: dict[str, object] = {
            "op": op,
            "id": str(args.id),
            "template": str(Path(args.template).resolve()),
            "garment": str(Path(args.glb).resolve()),
            "out_dir": str(out),
            "regions": str(stages.REGIONS_JSON),
            "covers_regions": covers,
        }
        log = stages.run_stage(
            "garment_bind", params, scratch=scratch, run_dir=out, environ=environ, blender=blender
        )
        (out / f"garment_bind.{op}.log").write_bytes(log.read_bytes())
    if not args.hides_only:
        binding = gf.read_binding(out / "binding.bin")
        failures = gf.check_binding(binding, len(gf.read_template(Path(args.template))))
        print(
            f"wardrobe bind {args.id}: {len(binding.records)} records; "
            f"W-1 {'PASS' if not failures else failures}"
        )
    print(f"wardrobe bind {args.id}: hidden template vertices {len(gf.read_hides(out / 'hides.bin'))}")
    return 0


def _gate_outfit(recipe: Recipe, args: argparse.Namespace) -> int:
    glb = read_glb(Path(args.glb))
    manifest = cast(dict[str, object], json.loads(Path(args.manifest).read_text(encoding="utf-8")))
    parts = cast(dict[str, dict[str, object]], manifest["parts"])
    classes = {node[len("part_") :]: str(p["class"]) for node, p in parts.items()}
    results = [
        wgates.gate_g1(glb),
        wgates.gate_g2(glb),
        wgates.gate_g3(glb),
        wgates.gate_g14(glb),
        wgates.gate_w2(glb, classes),
        wgates.gate_w3(glb, recipe.texel_density),
        wgates.gate_v1(glb),
    ]
    rows: list[dict[str, object]] = []
    for r in results:
        print(f"{r.gate.value}: {r.verdict.value}")
        for line in r.looked_at:
            print(f"    looked at: {line}")
        for reason in r.reasons:
            print(f"    reason: {reason}")
        rows.append(
            {
                "gate": r.gate.value,
                "verdict": r.verdict.value,
                "where": "; ".join(r.looked_at),
                "reasons": list(r.reasons),
            }
        )
    if args.json is not None:
        Path(args.json).write_text(json.dumps(rows, indent=2) + "\n", encoding="utf-8")
    return 0 if all(r.verdict is Verdict.PASS for r in results) else 1


def part_layer(recipe: Recipe, part_class: str) -> float:
    """Push-out order: the slot's place in §4.5's order; hair sits under headwear (inside the head slot)."""
    if part_class == "hair":
        return recipe.slot_layer("head") - 0.5
    return float(recipe.slot_layer(part_class))


def _assemble(recipe: Recipe, scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    """Spec: {"name", "body", "template_dir", "template_name",
    "parts": [{"id", "dir", "class": slot | "hair", "weights", "covers": [...], "provenance": {...}}]}."""
    spec = cast(dict[str, object], json.loads(Path(args.spec).read_text(encoding="utf-8")))
    folder = Path(str(spec["template_dir"]))
    name = str(spec["template_name"])
    out = Path(args.out_dir).resolve()
    parts = cast(list[dict[str, object]], spec["parts"])
    params: dict[str, object] = {
        "body": str(Path(str(spec["body"])).resolve()),
        "template": str(folder / f"{name}_template.bin"),
        "weights": str(folder / f"{name}_template_weights.bin"),
        "corr": str(folder / f"{name}_corr.bin"),
        "regions": str(stages.REGIONS_JSON),
        "out": str(out / f"outfit_{spec['name']}.glb"),
        "out_dir": str(out),
        "parts": [
            {
                "id": p["id"],
                "dir": str(Path(str(p["dir"])).resolve()),
                "layer": part_layer(recipe, str(p["class"])),
                "weights": p.get("weights", "inherit"),
                "provenance": p.get("provenance", {}),
                # a planted mutation (M-p-1, test only): the part's chest shrunk by this many mm
                "plant_shrink_chest_mm": p.get("plant_shrink_chest_mm", 0),
            }
            for p in parts
        ],
    }
    log = stages.run_stage(
        "assemble", params, scratch=scratch, run_dir=out, environ=environ, blender=_blender_checked(environ)
    )
    manifest = {
        "name": spec["name"],
        "regions": list(recipe.regions),
        "parts": {
            f"part_{p['id']}": {
                "layer": part_layer(recipe, str(p["class"])),
                "covers": p.get("covers", []),
                "class": p["class"],
            }
            for p in parts
        },
    }
    (out / "outfit_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"wardrobe assemble: {params['out']}; manifest {out / 'outfit_manifest.json'}; log {log}")
    return 0


W9_BOUND_M = 0.0005  # W-9: the client's fit equals the pipeline's within 0.5 mm per vertex (step-32 §7.2)


def _w9(args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    folder = Path(args.template_dir).resolve()
    template_path = folder / f"{args.name}_template.bin"
    out = Path(args.out).resolve()
    out.mkdir(parents=True, exist_ok=True)
    garments: list[dict[str, object]] = []
    for spec in args.garment:
        gid, _, directory = str(spec).partition("=")
        base = Path(directory).resolve()
        garments.append({"id": gid, "glb": str(base / "garment.glb"), "binding": str(base / "binding.bin")})
    job = {
        "template": str(template_path),
        "weights": str(folder / f"{args.name}_template_weights.bin"),
        "out_dir": str(out),
        "garments": garments,
        "repeat": 20,
    }
    log = godot_runs.run("fit_check.gd", job, project=out / "project", environ=environ)
    report = cast(dict[str, object], json.loads((out / "fit_check.json").read_text(encoding="utf-8")))
    template = gf.read_template(template_path)
    failures = 0
    for g in garments:
        gid = str(g["id"])
        entry = cast(dict[str, dict[str, object]], report["garments"])[gid]
        if "error" in entry:
            print(f"W-9 {gid}: FAIL: {entry['error']} ({entry})")
            failures += 1
            continue
        if args.reference_dir is not None:
            # the Blender reference fit of R-12 (a), computed from its own copy of the binding (W-9 proper)
            reference = list(gf.read_template(Path(args.reference_dir) / f"{gid}_blender.bin"))
        else:
            reference = gf.fit(gf.read_binding(Path(str(g["binding"]))), template)
        error = gf.max_distance(gf.read_template(out / f"{gid}_godot.bin"), reference)
        verdict = "PASS" if error <= W9_BOUND_M else "FAIL"
        failures += verdict == "FAIL"
        print(
            f"W-9 {gid}: Godot vs Python max {1000 * error:.4f} mm (bound 0.5 mm) {verdict}; "
            f"fit median {entry['fit_ms_median']} ms; vertices {entry['glb_vertices']}"
        )
    print(f"wardrobe w9: log {log}")
    return 0 if failures == 0 else 1


def _preview(scratch: Path, args: argparse.Namespace, environ: Mapping[str, str]) -> int:
    items: list[dict[str, object]] = []
    for spec in args.item:
        path, _, colour = str(spec).rpartition("=")
        items.append({"path": str(Path(path).resolve()), "colour": [float(c) for c in colour.split(",")]})
    out = Path(args.out).resolve()
    log = stages.run_stage(
        "preview",
        {"items": items, "out": str(out)},
        scratch=scratch,
        run_dir=out.parent,
        environ=environ,
        blender=_blender_checked(environ),
    )
    print(f"wardrobe preview: {out}_front.png, _side.png, _back.png; log {log}")
    return 0


def run(args: argparse.Namespace, environ: Mapping[str, str] | None = None) -> int:
    env = os.environ if environ is None else environ
    command = str(args.wardrobe_command)
    try:
        recipe = load_recipe(Path(args.recipe))
        if command == "check":
            return _check(recipe)
        if command == "gate-outfit":
            return _gate_outfit(recipe, args)
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
        if command == "install-mpfb":
            return _install(scratch, args, env)
        if command == "regions":
            return _regions(scratch, env)
        if command == "register":
            return _register(recipe, scratch, args, env)
        if command == "preview":
            return _preview(scratch, args, env)
        if command == "fixtures":
            return _fixtures(recipe, scratch, env)
        if command == "w9":
            return _w9(args, env)
        if command == "bind":
            return _bind(recipe, scratch, args, env)
        if command == "clean":
            return _clean(recipe, scratch, args, env)
        if command == "assemble":
            return _assemble(recipe, scratch, args, env)
        if command == "fixture-check":
            return _fixture_check(recipe, scratch, env)
    except (
        RecordError,
        ScratchError,
        budget.BudgetRefusal,
        images.ImageError,
        meshy.MeshyError,
        stages.StageError,
        BlenderError,
    ) as exc:
        print(f"wardrobe {command}: {exc}", file=sys.stderr)
        return 1
    print(f"wardrobe {command}: not yet implemented", file=sys.stderr)
    return 2
