"""Run a recipe: many candidates, one sidecar each, one contact sheet for the operator.

    python -m mineworld_gen.batch recipes/2d_default.yaml \
        --out ../../presentation/mineworld-default/2D/candidates

Requests run concurrently because each one is ~25 s of server-side work and the wall time
otherwise dominates. Failures are collected and reported rather than aborting the run: a batch
that produced 25 of 27 assets is still useful, and the two that failed are named.
"""

from __future__ import annotations

import argparse
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import yaml

from .normalize import normalize_rgba
from .provenance import write_sidecar
from .provider import GenerationParams

DEFAULT_REFERENCE_DIR = Path("presentation/mineworld-default/2D/references")


def _items(recipe: dict, reference_dir: Path) -> list[dict]:
    defaults = recipe.get("defaults", {})
    base_style = recipe.get("style", "")
    out = []
    for group in recipe.get("groups", []):
        style = group.get("style", base_style)
        refs = group.get("references", defaults.get("references", []))
        for item in group.get("items", []):
            prefix = group.get("prefix") or group["name"]
            out.append(
                {
                    "key": f"{prefix}_{item['name']}",
                    "group": group["name"],
                    "prompt": f"{style.strip()}\n\n{' '.join(item['subject'].split())}",
                    "references": [reference_dir / r for r in item.get("references", refs)],
                    "size": item.get("size", group.get("size", defaults.get("size", "1024x1024"))),
                    "quality": item.get(
                        "quality", group.get("quality", defaults.get("quality", "high"))
                    ),
                    "transparent": item.get(
                        "transparent",
                        group.get("transparent", defaults.get("transparent", True)),
                    ),
                }
            )
    return out


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="mineworld_gen.batch")
    p.add_argument("recipe", type=Path)
    p.add_argument("--out", required=True, type=Path)
    p.add_argument("--reference-dir", type=Path, default=None)
    p.add_argument("--model", default=None)
    p.add_argument("--workers", type=int, default=5)
    p.add_argument("--only", default=None, help="Substring filter on the asset key.")
    p.add_argument("--dry-run", action="store_true")
    args = p.parse_args(argv)

    recipe = yaml.safe_load(args.recipe.read_text(encoding="utf-8"))
    ref_dir = args.reference_dir
    if ref_dir is None:
        root = _repo_root() or Path.cwd()
        ref_dir = root / DEFAULT_REFERENCE_DIR
    items = _items(recipe, ref_dir)
    if args.only:
        items = [i for i in items if args.only in i["key"]]
    if not items:
        print("no items selected", file=sys.stderr)
        return 2

    print(f"{len(items)} candidates -> {args.out}")
    if args.dry_run:
        for i in items:
            print(f"  {i['key']:34s} {i['size']} q={i['quality']} alpha={i['transparent']}")
        return 0

    for item in items:
        for ref in item["references"]:
            if not ref.is_file():
                print(f"reference not found: {ref}", file=sys.stderr)
                return 2

    from .openai_images import OpenAIImageProvider

    provider = OpenAIImageProvider(model=args.model)
    args.out.mkdir(parents=True, exist_ok=True)

    def run(item: dict) -> tuple[dict, object]:
        params = GenerationParams(
            size=item["size"],
            quality=item["quality"],
            transparent_background=item["transparent"],
        )
        return item, provider.generate(item["prompt"], item["references"], params)

    total_cost = 0.0
    failures: list[tuple[str, str]] = []
    produced: list[Path] = []

    with ThreadPoolExecutor(max_workers=args.workers) as pool:
        futures = {pool.submit(run, item): item for item in items}
        for fut in as_completed(futures):
            item = futures[fut]
            try:
                item, result = fut.result()
            except Exception as exc:  # noqa: BLE001 - a batch must survive one bad request
                failures.append((item["key"], f"{type(exc).__name__}: {exc}"))
                print(f"  FAIL {item['key']}: {type(exc).__name__}: {exc}", file=sys.stderr)
                continue

            path = args.out / f"{item['key']}.png"
            path.write_bytes(result.image_bytes)
            notes = normalize_rgba(path) if item["transparent"] else []
            if notes:
                result.provenance["postprocess"]["cleaned"] = True
                result.provenance["postprocess"]["normalization"] = notes
            result.provenance["generation"]["asset_group"] = item["group"]
            write_sidecar(path, result.provenance)

            cost = result.stats.get("cost_usd_estimate") or 0.0
            total_cost += cost
            produced.append(path)
            print(f"  ok   {item['key']:34s} {result.stats.get('wall_seconds'):>6}s  ~${cost:.4f}")

    print(f"\n{len(produced)} produced, {len(failures)} failed, ~${total_cost:.2f} total")
    for key, err in failures:
        print(f"  failed: {key}: {err}")
    return 0 if not failures else 1


def _repo_root() -> Path | None:
    for parent in Path(__file__).resolve().parents:
        if (parent / ".git").exists() or (parent / "Cargo.toml").exists():
            return parent
    return None


if __name__ == "__main__":
    raise SystemExit(main())
