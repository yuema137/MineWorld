"""The R1 runtime manifest (`docs/reference/cognition-supervised.md` §3; design `pr-playable-dialogue-memory.md` §3.4.2).

`python -m mineworld_cognition --write-runtime-manifest PATH`, run with a source checkout's interpreter, records what that
interpreter imports: its Python version, the checkout's revision and lock digest, the source files of
`mineworld-cognition` and `mineworld-sdk` as imported (an editable install resolves to the checkout's real files), every
other installed distribution by name and version (each checked against `uv.lock` when written), and the entry-point
groups the controller loads. `--preflight MANIFEST` derives the same document again and compares it whole: any
difference, a checkout updated by `git pull` or `uv sync` among them, is a runtime mismatch, and the launcher plays the
world rule-only until the manifest is written again. Nothing here imports a backend or opens a connection.
"""

from __future__ import annotations

import hashlib
import platform
import tomllib
from collections.abc import Iterator
from importlib import import_module, metadata
from pathlib import Path
from typing import cast

from pydantic import BaseModel, ConfigDict

FORMAT = 1
OWN = ("mineworld-cognition", "mineworld-sdk")
"""The distributions whose source files are hashed, with their import package names below."""
PACKAGES = {"mineworld-cognition": "mineworld_cognition", "mineworld-sdk": "mineworld_sdk"}
ENTRY_POINT_GROUPS = (
    "mineworld.forgetting",
    "mineworld.retry",
    "mineworld.breaker",
    "mineworld.memory_scopes",
)
LOCK = "uv.lock"


class ManifestError(Exception):
    """The manifest cannot be written or derived: no checkout, or an environment that disagrees with its lock."""


class _Strict(BaseModel):
    model_config = ConfigDict(strict=True, frozen=True, extra="forbid")


class SourceFile(_Strict):
    path: str
    sha256: str


class Distribution(_Strict):
    name: str
    version: str
    files: list[SourceFile]


class Manifest(_Strict):
    manifest: int
    python: str
    source_revision: str | None
    lock_sha256: str
    distributions: list[Distribution]
    entry_point_groups: list[str]


def derive() -> Manifest:
    """The manifest of the running interpreter and the checkout its `mineworld-cognition` was imported from."""
    root = checkout_root()
    lock = root / LOCK
    locked = _locked_versions(lock)
    distributions: list[Distribution] = []
    for name in sorted(_installed()):
        version = metadata.version(name)
        if name in OWN:
            files = list(_source_files(PACKAGES[name]))
        else:
            files = []
            wanted = locked.get(_normal(name))
            if wanted is not None and wanted != version:
                raise ManifestError(
                    f"{name} {version} is installed, and {LOCK} locks {wanted}: run `uv sync --locked`"
                )
        distributions.append(Distribution(name=name, version=version, files=files))
    return Manifest(
        manifest=FORMAT,
        python=platform.python_version(),
        source_revision=_revision(root),
        lock_sha256=_sha256(lock),
        distributions=distributions,
        entry_point_groups=list(ENTRY_POINT_GROUPS),
    )


def write(path: Path) -> Manifest:
    """Writes the running interpreter's manifest to `path`, whole or not at all."""
    manifest = derive()
    partial = path.with_name(path.name + ".partial")
    partial.write_text(manifest.model_dump_json(indent=1) + "\n", encoding="utf-8")
    partial.replace(path)
    return manifest


def read(path: Path) -> Manifest:
    try:
        return Manifest.model_validate_json(path.read_bytes())
    except (OSError, ValueError) as error:
        raise ManifestError(f"{path}: not a runtime manifest ({type(error).__name__})") from None


def differences(recorded: Manifest, current: Manifest) -> list[str]:
    """What differs between a recorded manifest and the running interpreter's, by field and distribution."""
    found = [
        field
        for field in ("manifest", "python", "source_revision", "lock_sha256", "entry_point_groups")
        if getattr(recorded, field) != getattr(current, field)
    ]
    before = {item.name: item for item in recorded.distributions}
    after = {item.name: item for item in current.distributions}
    for name in sorted(before.keys() | after.keys()):
        if before.get(name) != after.get(name):
            found.append(f"distribution {name}")
    return found


def unresolved_entry_points() -> list[str]:
    """The groups the controller loads that publish nothing, or publish an entry that cannot be loaded."""
    missing: list[str] = []
    for group in ENTRY_POINT_GROUPS:
        entries = metadata.entry_points(group=group)
        try:
            for entry in entries:
                entry.load()
        except Exception:  # any import failure of a published entry is the finding
            missing.append(group)
            continue
        if not entries:
            missing.append(group)
    return missing


def checkout_root() -> Path:
    """The source checkout `mineworld_cognition` was imported from: the nearest folder above it holding `uv.lock`."""
    here = Path(cast(str, import_module(PACKAGES["mineworld-cognition"]).__file__)).resolve()
    for folder in here.parents:
        if (folder / LOCK).is_file():
            return folder
    raise ManifestError(
        "mineworld_cognition is not imported from a source checkout (no uv.lock above it): "
        "the R1 runtime is a checkout's `uv sync --locked` environment"
    )


def _installed() -> set[str]:
    names: set[str] = set()
    for distribution in metadata.distributions():
        name = distribution.metadata["Name"]
        if name:
            names.add(name)
    return names


def _source_files(package: str) -> Iterator[SourceFile]:
    folder = Path(cast(str, import_module(package).__file__)).resolve().parent
    for path in sorted(folder.rglob("*.py")):
        relative = path.relative_to(folder.parent).as_posix()
        yield SourceFile(path=relative, sha256=_sha256(path))


def _locked_versions(lock: Path) -> dict[str, str]:
    try:
        with lock.open("rb") as file:
            document = tomllib.load(file)
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise ManifestError(f"{lock}: cannot be read ({type(error).__name__})") from None
    packages = cast(list[dict[str, object]], document.get("package", []))
    return {
        _normal(str(package["name"])): str(package["version"])
        for package in packages
        if "name" in package and "version" in package
    }


def _normal(name: str) -> str:
    """A distribution name as the lock writes it (PEP 503)."""
    return "-".join(
        part for part in name.lower().replace("_", "-").replace(".", "-").split("-") if part
    )


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _revision(root: Path) -> str | None:
    """The checkout's commit, read from its Git metadata without running Git (a launcher's environment need not
    have it on `PATH`), or `None` when the folder is not a Git checkout."""
    try:
        git = root / ".git"
        directory = git
        if git.is_file():
            pointer = git.read_text(encoding="utf-8").strip().removeprefix("gitdir:").strip()
            directory = (root / pointer).resolve()
        head = (directory / "HEAD").read_text(encoding="utf-8").strip()
        if not head.startswith("ref: "):
            return head
        reference = head.removeprefix("ref: ")
        common = directory
        if (directory / "commondir").is_file():
            common = (
                directory / (directory / "commondir").read_text(encoding="utf-8").strip()
            ).resolve()
        for folder in (directory, common):
            if (folder / reference).is_file():
                return (folder / reference).read_text(encoding="utf-8").strip()
        for line in (common / "packed-refs").read_text(encoding="utf-8").splitlines():
            if line.endswith(" " + reference):
                return line.split(" ", 1)[0]
    except OSError:
        return None
    return None
