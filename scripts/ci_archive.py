#!/usr/bin/env python3
"""Build and verify the immutable per-OS core archive (DEP-163).

Nextest owns archive contents/extraction. This module owns provenance: compiled env! paths
require the same checkout/target location, OS, architecture, toolchain and revision.
Cargo-only custom harnesses and doctests stay on their owning shard.
"""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import ci_shard

ROOT = Path(__file__).resolve().parent.parent
NEXTEST_VERSION = "0.9.148"
DEFAULT_ARCHIVE = Path("artifacts/core-archive")


def output(*command: str) -> str:
    return subprocess.check_output(command, cwd=ROOT, text=True).strip()


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def identity() -> dict[str, str]:
    version = output("cargo", "nextest", "--version").splitlines()[0]
    if not version.startswith(f"cargo-nextest {NEXTEST_VERSION} "):
        raise ValueError(f"expected nextest {NEXTEST_VERSION}, got {version}")
    metadata = json.loads(
        output("cargo", "metadata", "--no-deps", "--format-version", "1")
    )
    target = Path(metadata["target_directory"]).resolve()
    if target != ROOT.resolve() / "target":
        raise ValueError(
            f"core archives require the default workspace target directory, got {target}"
        )
    return {
        "revision": output("git", "rev-parse", "HEAD"),
        "workspace": str(ROOT.resolve()),
        "system": platform.system(),
        "machine": platform.machine(),
        "rustc": output("rustc", "-vV"),
        "nextest": version,
    }


def verify(directory: Path) -> dict:
    data = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    if data.get("schema") != 1:
        raise ValueError("unsupported core archive manifest")
    expected = identity()
    if data.get("identity") != expected:
        raise ValueError(
            f"core archive identity mismatch: built {data.get('identity')}; current {expected}"
        )
    if data.get("sha256") != digest(directory / "tests.tar.zst"):
        raise ValueError("core archive checksum mismatch")
    units = data.get("units")
    if (
        not isinstance(units, list)
        or not units
        or not all(isinstance(unit, str) for unit in units)
    ):
        raise ValueError("core archive has no valid unit inventory")
    if len(units) != len(set(units)) or not {ci_shard.LIB_BINS, ci_shard.DOC} <= set(
        units
    ):
        raise ValueError("core archive unit inventory is incomplete or duplicated")
    return data


def build(directory: Path = DEFAULT_ARCHIVE) -> int:
    directory.mkdir(parents=True, exist_ok=True)
    # A previous successful manifest must never certify this build attempt.
    manifest = directory / "manifest.json"
    manifest.unlink(missing_ok=True)
    provenance = identity()
    units = ci_shard.workspace_units()
    custom = ci_shard.custom_harness_targets(ROOT)
    if custom != set(ci_shard.CUSTOM_HARNESS):
        raise ValueError(
            f"custom harness inventory changed: {sorted(custom)}; audit before archiving"
        )
    targets = ["--lib", "--bins"]
    for unit in units:
        if unit not in (ci_shard.LIB_BINS, ci_shard.DOC) and unit not in custom:
            targets.extend(["--test", unit])
    command = [
        "cargo",
        "nextest",
        "archive",
        "--workspace",
        "--locked",
        "--profile",
        "ci",
        *targets,
        "--archive-file",
        str(directory / "tests.tar.zst"),
    ]
    status = subprocess.run(command, cwd=ROOT, check=False).returncode
    if status:
        return status
    manifest.write_text(
        json.dumps(
            {
                "schema": 1,
                "identity": provenance,
                "units": units,
                "sha256": digest(directory / "tests.tar.zst"),
            },
            indent=2,
        )
        + "\n",
        encoding="utf-8",
    )
    return 0


def main(arguments: list[str]) -> int:
    if arguments == ["build"]:
        return build()
    raise ValueError("usage: ci_archive.py build")


if __name__ == "__main__":
    try:
        sys.exit(main(sys.argv[1:]))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        print(f"ci_archive: {error}", file=sys.stderr)
        sys.exit(2)
