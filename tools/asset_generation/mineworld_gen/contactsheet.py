"""Build one contact sheet from a candidate directory, so a person can judge the set at a glance.

    python -m mineworld_gen.contactsheet <candidates-dir> -o contact_sheet.png

Transparent assets are composited over a light checkerboard, because judging a cutout against
solid white hides exactly the alpha defects worth catching.
"""

from __future__ import annotations

import argparse
from pathlib import Path

CELL = 300
PAD = 10
LABEL = 22


def _checker(size: int, square: int = 12) -> "object":
    from PIL import Image

    tile = Image.new("RGB", (square * 2, square * 2), (255, 255, 255))
    grey = Image.new("RGB", (square, square), (222, 222, 226))
    tile.paste(grey, (0, 0))
    tile.paste(grey, (square, square))
    out = Image.new("RGB", (size, size))
    for y in range(0, size, square * 2):
        for x in range(0, size, square * 2):
            out.paste(tile, (x, y))
    return out


def build(candidate_dir: Path, out_path: Path, columns: int = 6) -> Path:
    from PIL import Image, ImageDraw, ImageFont

    paths = sorted(p for p in candidate_dir.glob("*.png") if not p.name.startswith("contact"))
    if not paths:
        raise SystemExit(f"no candidate images in {candidate_dir}")

    try:
        font = ImageFont.truetype("/System/Library/Fonts/Supplemental/Arial.ttf", 15)
    except OSError:
        font = ImageFont.load_default()

    rows = (len(paths) + columns - 1) // columns
    cell_h = CELL + LABEL
    sheet = Image.new(
        "RGB",
        (columns * (CELL + PAD) + PAD, rows * (cell_h + PAD) + PAD),
        (248, 248, 250),
    )
    draw = ImageDraw.Draw(sheet)

    for index, path in enumerate(paths):
        col, row = index % columns, index // columns
        x = PAD + col * (CELL + PAD)
        y = PAD + row * (cell_h + PAD)

        image = Image.open(path).convert("RGBA")
        image.thumbnail((CELL, CELL), Image.LANCZOS)
        backdrop = _checker(CELL)
        backdrop.paste(image, ((CELL - image.width) // 2, (CELL - image.height) // 2), image)
        sheet.paste(backdrop, (x, y))
        draw.rectangle([x, y, x + CELL - 1, y + CELL - 1], outline=(205, 205, 212))

        name = path.stem
        while draw.textlength(name, font=font) > CELL - 4 and len(name) > 4:
            name = name[:-2]
        draw.text((x + 2, y + CELL + 3), name, fill=(40, 40, 48), font=font)

    sheet.save(out_path)
    return out_path


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="mineworld_gen.contactsheet")
    p.add_argument("candidates", type=Path)
    p.add_argument("-o", "--out", type=Path, default=None)
    p.add_argument("--columns", type=int, default=6)
    args = p.parse_args(argv)
    out = args.out or args.candidates / "contact_sheet.png"
    print(build(args.candidates, out, args.columns))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
