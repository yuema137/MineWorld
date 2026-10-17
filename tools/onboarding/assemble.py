"""Assemble an onboarding sandbox: the guide, the reference, the binary, the bundled data packs, and nothing else.

    python3 tools/onboarding/assemble.py SANDBOX --binary PATH [--repo ROOT]

`tools/onboarding/PROTOCOL.md` §2 is the authority for the allowed set. SANDBOX must not exist yet. Standard
library only.
"""

from __future__ import annotations

import argparse
import shutil
import stat
import sys
from collections.abc import Callable
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]

# The bundled data packs the sandbox carries, by their folder in the repository (PROTOCOL.md §2).
DATA_PACKS = ("entities/modern-goods",)


class Refused(Exception):
    """The sandbox cannot be assembled as asked."""


def is_mirror(path: Path) -> bool:
    """A Chinese mirror of a guide page (`docs/GUIDE_FORMAT.md` §7): the sandbox carries English pages only."""
    return path.name.endswith(".zh-CN.md")


def copy_tree(source: Path, target: Path, keep: Callable[[Path], bool] | None = None) -> None:
    """Copies `source` to `target`, file by file. A symbolic link is refused, never followed: it could lead
    the agent outside the allowed set."""
    for item in sorted(source.rglob("*")):
        relative = item.relative_to(source)
        if item.is_symlink():
            raise Refused(f"{item} is a symbolic link; the sandbox copies files only")
        if item.is_dir():
            continue
        if keep is not None and not keep(item):
            continue
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(item, destination)


def assemble(sandbox: Path, binary: Path, repo: Path = REPO) -> list[str]:
    """Builds the sandbox and returns every file in it, relative and with `/`, sorted."""
    if sandbox.exists():
        raise Refused(f"{sandbox} exists; a sandbox is always assembled fresh")
    if not binary.is_file():
        raise Refused(f"{binary} is not a file; build the binary first")
    for inside in (repo, *repo.parents):
        if (inside / ".git").exists() and sandbox.resolve().is_relative_to(inside.resolve()):
            raise Refused(
                f"{sandbox} is inside the repository {inside}; a sandbox lies outside every repository"
            )
    sandbox.mkdir(parents=True)
    copy_tree(repo / "docs" / "guide", sandbox / "guide", keep=lambda path: not is_mirror(path))
    copy_tree(repo / "docs" / "reference", sandbox / "reference")
    for pack in DATA_PACKS:
        source = repo / pack
        copy_tree(source, sandbox / "packs" / source.name)
    target = sandbox / "bin" / binary.name
    target.parent.mkdir()
    shutil.copyfile(binary, target)
    target.chmod(target.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    (sandbox / "work").mkdir()
    return listing(sandbox)


def listing(sandbox: Path) -> list[str]:
    """Every file under `sandbox`, relative, with `/`, sorted."""
    return sorted(
        path.relative_to(sandbox).as_posix() for path in sandbox.rglob("*") if path.is_file()
    )


def leaks(sandbox: Path, repo: Path = REPO) -> list[str]:
    """Copied text files that name the repository's absolute path: the agent would learn where the source is."""
    needle = str(repo.resolve()).encode()
    found: list[str] = []
    for path in sandbox.rglob("*"):
        if path.is_file() and path.parent.name != "bin" and needle in path.read_bytes():
            found.append(path.relative_to(sandbox).as_posix())
    return sorted(found)


def main(arguments: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Assemble an onboarding sandbox (PROTOCOL.md §2).")
    parser.add_argument("sandbox", type=Path)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--repo", type=Path, default=REPO)
    options = parser.parse_args(arguments)
    try:
        files = assemble(options.sandbox, options.binary, options.repo)
    except Refused as error:
        print(f"[assemble] refused: {error}", file=sys.stderr)
        return 2
    leaked = leaks(options.sandbox, options.repo)
    if leaked:
        print(
            f"[assemble] refused: these files name the repository's path: {', '.join(leaked)}",
            file=sys.stderr,
        )
        return 2
    print(f"[assemble] {options.sandbox}: {len(files)} file(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
