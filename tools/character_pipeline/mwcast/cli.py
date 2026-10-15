"""Command-line parsing for `cast.py` (PIPELINE.md §3).

Each subcommand is wired by the commit that implements it. Until then it reports that it is not yet
implemented and returns 2, so a half-built tool cannot pass for a finished one.
"""

from __future__ import annotations

import argparse
import sys
from collections.abc import Sequence

SUBCOMMANDS: dict[str, str] = {
    "gate": "G-1…G-6 and G-14 on one export",
    "check-characters": "validate every committed character",
    "run": "stages 6–10 for one character",
    "compare": "parity comparison of two exports",
    "compare-texture": "texel or mask comparison of two images",
    "mask": "stage 3, the mask",
    "meshy": "stage 4, the Meshy client (dry run without --dry-run refused)",
    "identifiers": "AI-identifier inspection of downloads",
}


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="cast.py", description="MineWorld character pipeline (mwcast-1).")
    subparsers = parser.add_subparsers(dest="command", required=True)
    for name, help_text in SUBCOMMANDS.items():
        subparsers.add_parser(name, help=help_text)
    return parser


def main(argv: Sequence[str] | None = None) -> int:
    args = build_parser().parse_args(list(argv) if argv is not None else None)
    command = str(args.command)
    print(f"cast.py {command}: not yet implemented in this commit", file=sys.stderr)
    return 2
