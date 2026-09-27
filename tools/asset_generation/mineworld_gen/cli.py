"""Operator entry point: generate candidates, each with its provenance sidecar.

    python -m mineworld_gen \
        --prompt "a single detached bakery building, ..." \
        --reference presentation/mineworld-default/2D/references/01_town_square.png \
        --out presentation/mineworld-default/2D/candidates \
        --name bakery --transparent

Everything it writes is a **candidate** (ARC-9). It never writes into an approved-asset
directory, and it never sets `approval.status` to anything but `candidate`.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from .normalize import normalize_rgba
from .provenance import write_sidecar
from .provider import GenerationParams


def _parse(argv: list[str] | None) -> argparse.Namespace:
    p = argparse.ArgumentParser(prog="mineworld_gen", description=__doc__)
    p.add_argument("--prompt", required=True)
    p.add_argument(
        "--reference",
        action="append",
        default=[],
        type=Path,
        help="Style reference image; repeatable. Sent as reference conditioning.",
    )
    p.add_argument("--out", required=True, type=Path, help="Candidate directory.")
    p.add_argument("--name", required=True, help="Base filename, e.g. 'bakery'.")
    p.add_argument("--count", type=int, default=1)
    p.add_argument("--size", default="1024x1024")
    p.add_argument("--quality", default="high")
    p.add_argument("--transparent", action="store_true", help="Request an alpha channel.")
    p.add_argument("--model", default=None, help="Override the model id.")
    p.add_argument(
        "--cutout",
        action="store_true",
        help="Fallback background removal via rembg, when native transparency is unavailable.",
    )
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse(argv)
    for ref in args.reference:
        if not ref.is_file():
            print(f"reference not found: {ref}", file=sys.stderr)
            return 2

    from .openai_images import OpenAIImageProvider

    provider = OpenAIImageProvider(model=args.model)
    args.out.mkdir(parents=True, exist_ok=True)

    params = GenerationParams(
        size=args.size,
        quality=args.quality,
        transparent_background=args.transparent,
    )

    total = 0.0
    for i in range(args.count):
        result = provider.generate(args.prompt, args.reference, params)
        stem = f"{args.name}_{i + 1:02d}" if args.count > 1 else args.name
        image_path = args.out / f"{stem}.png"
        image_path.write_bytes(result.image_bytes)

        if args.cutout:
            _cutout(image_path, image_path)
            result.provenance["postprocess"]["background_removed"] = True
            result.provenance["postprocess"]["background_removal_tool"] = "rembg (u2net)"

        # Same normalization the batch path applies, so a single asset and a batch asset are
        # never subtly different files.
        notes = normalize_rgba(image_path) if args.transparent else []
        if notes:
            result.provenance["postprocess"]["cleaned"] = True
            result.provenance["postprocess"]["normalization"] = notes

        write_sidecar(image_path, result.provenance)
        cost = result.stats.get("cost_usd_estimate") or 0.0
        total += cost
        print(f"{image_path}  {result.stats.get('wall_seconds')}s  ~${cost:.4f}")

    if args.count > 1:
        print(f"total ~${total:.4f}")
    return 0


def _cutout(src: Path, dst: Path) -> None:
    """Background removal with a mature tool — never a hand-rolled matting implementation."""
    from PIL import Image
    from rembg import remove

    remove(Image.open(src)).save(dst)


if __name__ == "__main__":
    raise SystemExit(main())
