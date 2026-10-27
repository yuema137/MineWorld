#!/usr/bin/env python3
"""Build a downloadable MineWorld bundle on this machine (S23 R-a; `docs/DECISIONS.md` ARC-80).

The layout, what is shipped and what never is, and the size budget are specified in
`.structured-coding/plans/mvp0/step-23-release.md` §§3, 7 and 18. This script is the one place that
knows them; CI's release workflow (R-b) calls the same commands.

    python3 scripts/package.py build    [--target T]          the release server and the launchers for T
    python3 scripts/package.py export   [--target T] [--clients 2d,3d] [--godot BIN]
                                                               the shared runtime and the client packs for T
    python3 scripts/package.py assemble [--target T] [--clients 2d,3d] [--godot BIN] [--server FILE]
                                         [--launchers DIR] [--godot-notices DIR] [--no-archive]
                                                               the bundle folder and its archive
    python3 scripts/package.py budget   <bundle folder> [<archive>]
                                                               the hard limits; the 20 largest files of each pack
    python3 scripts/package.py probe    <bundle folder> [--clients 2d,3d] [--timeout S]
                                                               the bundle's server with each exported client,
                                                               headless, from wherever the folder now is
    python3 scripts/package.py smoke    <archive or bundle folder> [--modes 2d,3d,both] [--sums FILE]
                                         [--timeout S]
                                                               step-23 §9.1 steps 1-6 through the launcher's
                                                               entry points (§9.2); step 3 is reported NOT RUN
    python3 scripts/package.py --self-test

T is macos-universal, windows-x86_64, linux-x86_64 or linux-arm64; absent, the machine's own. Work goes
to target/package/<T>/. Every command exits non-zero and names what failed. Standard library only.
"""

from __future__ import annotations

import argparse
import contextlib
import gzip
import hashlib
import io
import json
import os
import platform
import plistlib
import re
import shutil
import signal
import struct
import subprocess
import sys
import tarfile
import tempfile
import time
import zipfile
import zlib
from dataclasses import dataclass
from pathlib import Path

from runtime_settings import RUNTIME, client_settings, judge

ROOT = Path(__file__).resolve().parent.parent
PACKAGING = ROOT / "packaging"
MB = 1_000_000  # the budget's unit (step-23 §7.1): decimal megabytes
GODOT_TAG = "4.7.2-stable"
# Godot's own notices at the tag, fetched once and pinned (DEP-41).
GODOT_NOTICES = {
    "LICENSE.txt": "b0435e3b3e4e55238f05f4b306f30524a1b2e20147810d436eaa554fa6855c80",
    "COPYRIGHT.txt": "cb1980c88089573bcacd7221d777c689bb8bbd778799f24c27fca0fe5f774d6d",
}
WORLDS = ("social-cafe", "market-town", "bodies-yard")
PACK_2D = "presentation/mineworld-default/2D"
PACK_2D_PARTS = ("art", "assets", "i18n", "renderer", "far_field", "daylight.json", "manifest.yaml",
                 "pack.yaml")
# What the exported 3D client reads from its Presentation Pack (D-Ra-5): the far field (ARC-102) and
# the cast binding (cast.yaml and each character's character.yaml).
PACK_3D = "presentation/mineworld-default/3D"
PACK_3D_PARTS = ("far_field", "characters/cast.yaml", "characters/cast", "manifest.yaml", "pack.yaml")
SHARED_FILES = ("clients/shared/settings/locale", "clients/shared/settings/fonts")


@dataclass(frozen=True)
class Target:
    name: str
    triples: tuple[str, ...]
    preset: str
    exe: str
    archive: str

    @property
    def macos(self) -> bool:
        return self.name.startswith("macos")


TARGETS = {t.name: t for t in (
    Target("macos-universal", ("aarch64-apple-darwin", "x86_64-apple-darwin"), "macOS", "", "zip"),
    Target("windows-x86_64", ("x86_64-pc-windows-msvc",), "Windows x86_64", ".exe", "zip"),
    Target("linux-x86_64", ("x86_64-unknown-linux-musl",), "Linux x86_64", "", "tar.gz"),
    Target("linux-arm64", ("aarch64-unknown-linux-musl",), "Linux arm64", "", "tar.gz"),
)}


@dataclass(frozen=True)
class Client:
    key: str
    project: str
    name: str        # the client pack's stem under runtime/clients/ (`<name>.pck`, `<name>.settings.json`)
    scene: str       # the scene the shared runtime starts after mounting the pack (PR S23-RT §4.2)
    world: str       # the world `probe` hosts for it
    args: tuple[str, ...]
    verdict: str     # the line a passing headless run prints (F-R6: a missing line is FAIL)


CLIENTS = {c.key: c for c in (
    Client("2d", "clients/2d", "mineworld-2d", "res://scenes/app.tscn", "market-town",
           ("--seat=carol", "--drive"), "drive complete: PASS"),
    Client("3d", "clients/3d-spike", "mineworld-3d", "res://scenes/slice.tscn", "social-cafe",
           ("--slice-link",), "all link checks pass"),
)}
# The one Godot engine of a bundle (PR S23-RT, ARC-80): `clients/runtime/` exported as `mineworld-client[.exe]`
# beside `mineworld-client.pck`, or `MineWorld.app` on macOS. It loads a client's pack chosen by `--client=`.
RUNTIME_NAME = "mineworld-client"
RUNTIME_APP = "MineWorld"


@dataclass(frozen=True)
class EntryPoint:
    mode: str        # the launcher's mode (`tools/launch/LAUNCHER.md` §2)
    name: str        # the product name: the macOS `.app` and the Windows `.exe` (step-23 §6.3, QR-10)
    file: str        # the Linux executable, without spaces (`PLAY.txt`: ./MineWorld-2D)

    @property
    def binary(self) -> str:
        """The launcher crate's binary for this mode."""
        return f"mineworld-{self.mode}-launch"


# The launcher entry points at the bundle root (step-23 §19.4 C5). The launcher finds the bundle root from
# its own path (`tools/launch/src/bundle.rs` `root_of`): the executable's folder, or the folder holding its
# `.app`.
ENTRY_POINTS = {e.mode: e for e in (
    EntryPoint("2d", "MineWorld 2D", "MineWorld-2D"),
    EntryPoint("3d", "MineWorld 3D", "MineWorld-3D"),
    EntryPoint("both", "MineWorld 2D + 3D", "MineWorld-2D+3D"),
)}


@dataclass(frozen=True)
class Limit:
    hard: int
    goal: int


# step-23 §7.1, compressed sizes; fixed before measuring.
LIMITS = {"bundle": Limit(400 * MB, 250 * MB), "3d": Limit(250 * MB, 150 * MB), "2d": Limit(40 * MB, 20 * MB)}


class Failure(Exception):
    """A step that cannot continue; the message names what failed."""


# ------------------------------------------------------------------------------------- what never ships

EVIDENCE_DIRS = {"shots", "screenshots", "references", "candidates"}


def forbidden(path: str) -> str | None:
    """Why a bundle path (or a path inside a pack) must not ship (A-R3), or None."""
    parts = path.replace("\\", "/").strip("/").split("/")
    name = parts[-1]
    if any(part in EVIDENCE_DIRS for part in parts[:-1]):
        return "an evidence folder"
    if "tools/blender" in "/".join(parts):
        return "the GPL Blender scripts"
    if name.lower().endswith(".md"):
        return "a Markdown file"
    if name == ".env" or name.startswith(".env."):
        return "a secrets file"
    return None


# ------------------------------------------------------------------------------------------ Godot packs

PCK_MAGIC = b"GDPC"


def read_pck(path: Path) -> list[tuple[str, int]]:
    """The (path, size) of every file in a Godot 4 pack (pack format 3 or 4, the directory at the end)."""
    with path.open("rb") as f:
        if f.read(4) != PCK_MAGIC:
            raise Failure(f"{path}: not a Godot pack")
        version, _major, _minor, _patch, _flags = struct.unpack("<5I", f.read(20))
        if version not in (3, 4):
            raise Failure(f"{path}: pack format {version} is not one this script reads")
        _file_base, directory = struct.unpack("<2Q", f.read(16))
        f.seek(directory)
        (count,) = struct.unpack("<I", f.read(4))
        entries = []
        for _ in range(count):
            (length,) = struct.unpack("<I", f.read(4))
            name = f.read(length).rstrip(b"\0").decode("utf-8")
            _offset, size = struct.unpack("<2Q", f.read(16))
            f.read(16 + 4)  # md5, flags
            entries.append((name, size))
        return entries


def compressed_size(paths: list[Path]) -> int:
    """What the files cost inside an archive: zlib level 6, the level of `zip -6` and `gzip -6`."""
    total = 0
    for path in paths:
        packer = zlib.compressobj(6)
        with path.open("rb") as f:
            while chunk := f.read(1 << 20):
                total += len(packer.compress(chunk))
        total += len(packer.flush())
    return total


# ------------------------------------------------------------------------------------------------ build

def host_target() -> Target:
    system, machine = platform.system(), platform.machine().lower()
    arm = machine in ("arm64", "aarch64")
    name = {"Darwin": "macos-universal", "Windows": "windows-x86_64",
            "Linux": "linux-arm64" if arm else "linux-x86_64"}.get(system)
    if name is None:
        raise Failure(f"no bundle target for {system} {machine}")
    return TARGETS[name]


def run(command: list[str], **kwargs) -> None:
    print("$ " + " ".join(command), flush=True)
    code = subprocess.call(command, **kwargs)
    if code != 0:
        raise Failure(f"{command[0]} exited {code}")


def work_dir(target: Target) -> Path:
    return ROOT / "target" / "package" / target.name


def cargo_out() -> Path:
    """Where cargo writes: `CARGO_TARGET_DIR` when set (relative to the repository), else `target/`."""
    return ROOT / os.environ["CARGO_TARGET_DIR"] if os.environ.get("CARGO_TARGET_DIR") else ROOT / "target"


def build(target: Target) -> Path:
    """The release server for `target`, path-remapped (§5.3); a universal binary on macOS."""
    env = dict(os.environ)
    env["RUSTFLAGS"] = (env.get("RUSTFLAGS", "") + f" --remap-path-prefix={ROOT}=mineworld").strip()
    built = []
    for triple in target.triples:
        run(["cargo", "build", "--release", "--locked", "-p", "mineworld-cli", "--target", triple],
            cwd=ROOT, env=env)
        built.append(cargo_out() / triple / "release" / ("mineworld" + target.exe))
    out = work_dir(target) / ("mineworld" + target.exe)
    out.parent.mkdir(parents=True, exist_ok=True)
    if len(built) > 1:
        run(["lipo", "-create", "-output", str(out), *map(str, built)])
    else:
        shutil.copy2(built[0], out)
    return out


# ------------------------------------------------------------------------------------------ the launchers

def build_launchers(target: Target) -> dict[str, Path]:
    """The launcher's three binaries for `target` (ARC-78), built and joined like the server; by mode."""
    env = dict(os.environ)
    env["RUSTFLAGS"] = (env.get("RUSTFLAGS", "") + f" --remap-path-prefix={ROOT}=mineworld").strip()
    for triple in target.triples:
        run(["cargo", "build", "--release", "--locked", "-p", "mineworld-launch", "--target", triple],
            cwd=ROOT, env=env)
    out = work_dir(target) / "launch"
    out.mkdir(parents=True, exist_ok=True)
    binaries = {}
    for entry in ENTRY_POINTS.values():
        built = [cargo_out() / triple / "release" / (entry.binary + target.exe) for triple in target.triples]
        dest = out / (entry.binary + target.exe)
        if len(built) > 1:
            run(["lipo", "-create", "-output", str(dest), *map(str, built)])
        else:
            shutil.copy2(built[0], dest)
        binaries[entry.mode] = dest
    return binaries


def given_launchers(folder: str, target: Target) -> dict[str, Path]:
    """Launcher binaries already built for `target` (`assemble --launchers DIR`), by mode."""
    binaries = {e.mode: Path(folder) / (e.binary + target.exe) for e in ENTRY_POINTS.values()}
    missing = [str(p) for p in binaries.values() if not p.is_file()]
    if missing:
        raise Failure("--launchers: missing " + ", ".join(missing))
    return binaries


def entry_executable(bundle: Path, target: Target, entry: EntryPoint) -> Path:
    """The file a player starts for `entry`; on macOS the binary inside its `.app`."""
    if target.macos:
        return bundle / f"{entry.name}.app" / "Contents" / "MacOS" / entry.binary
    if target.exe:
        return bundle / (entry.name + target.exe)
    return bundle / entry.file


def entry_files(bundle: Path, target: Target, entry: EntryPoint) -> list[Path]:
    """Every file of `entry`: the executable, plus the `.app`'s Info.plist or the Linux `.desktop` file."""
    files = [entry_executable(bundle, target, entry)]
    if target.macos:
        files.append(bundle / f"{entry.name}.app" / "Contents" / "Info.plist")
    elif not target.exe:
        files.append(bundle / f"{entry.file}.desktop")
    return files


def missing_entry_points(bundle: Path, target: Target) -> list[str]:
    """The launcher files a bundle for `target` lacks; empty when every entry point is there."""
    return [f"{path.relative_to(bundle).as_posix()}: missing (the entry point {entry.name!r})"
            for entry in ENTRY_POINTS.values() for path in entry_files(bundle, target, entry)
            if not path.is_file()]


def info_plist(entry: EntryPoint, version: str) -> bytes:
    """A minimal `.app` Info.plist. `LSUIElement`: the launcher has no window, so it takes no Dock place;
    the client it starts has its own."""
    return plistlib.dumps({
        "CFBundleDisplayName": entry.name,
        "CFBundleExecutable": entry.binary,
        "CFBundleIdentifier": f"org.mineworld.launch.{entry.mode}",
        "CFBundleInfoDictionaryVersion": "6.0",
        "CFBundleName": entry.name,
        "CFBundlePackageType": "APPL",
        "CFBundleShortVersionString": version,
        "CFBundleVersion": version,
        "LSUIElement": True,
    }, sort_keys=True)


def desktop_entry(entry: EntryPoint) -> str:
    """A `.desktop` file that starts the executable beside it, wherever the bundle was unpacked: `%k` (this
    file's path) is `sh`'s `$0`. Escaped per the Desktop Entry specification: inside the quoted argument
    `"`, `` ` ``, `$` and `\\` take a backslash, and the file's string escaping doubles every backslash."""
    script = f'exec "$(dirname "$0")/{entry.file}"'
    quoted = '"' + re.sub(r'(["`$\\])', r"\\\1", script) + '"'
    return ("[Desktop Entry]\n"
            "Type=Application\n"
            "Version=1.5\n"
            f"Name={entry.name}\n"
            "Comment=MineWorld downloadable demo\n"
            f"Exec=sh -c {quoted.replace(chr(92), chr(92) * 2)} %k\n"
            "Terminal=false\n"
            "Categories=Game;\n")


def place_launchers(bundle: Path, target: Target, binaries: dict[str, Path], version: str,
                    sign: bool = True) -> None:
    """The entry points at the bundle root: macOS `.app` wrappers (ad-hoc signed), Windows `.exe` files
    (GUI subsystem, set in the crate), Linux executables with their `.desktop` files."""
    for entry in ENTRY_POINTS.values():
        exe = entry_executable(bundle, target, entry)
        exe.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(binaries[entry.mode], exe)
        exe.chmod(0o755)
        if target.macos:
            app = bundle / f"{entry.name}.app"
            (app / "Contents" / "Info.plist").write_bytes(info_plist(entry, version))
            if sign:
                sign_app(app)
        elif not target.exe:
            desktop = bundle / f"{entry.file}.desktop"
            desktop.write_text(desktop_entry(entry), encoding="utf-8")
            desktop.chmod(0o755)  # file managers start a .desktop file only when it is executable


def sign_app(app: Path) -> None:
    """Ad-hoc signature (`codesign -s -`; step-23 §8.2: unsigned for the pre-alpha, no identity)."""
    if platform.system() != "Darwin":
        print(f"[assemble] {app.name} left unsigned: codesign exists only on macOS", flush=True)
        return
    run(["codesign", "--force", "-s", "-", str(app)])


# ----------------------------------------------------------------------------------------------- export

def has_preset(project: Path, preset: str) -> bool:
    presets = project / "export_presets.cfg"
    return presets.is_file() and f'name="{preset}"' in presets.read_text(encoding="utf-8")


def extract_zip(archive: Path, into: Path) -> None:
    """Unpack a zip into `into`, keeping each file's mode (Python's `extractall` drops it)."""
    top = into.resolve()
    with zipfile.ZipFile(archive) as z:
        for info in z.infolist():
            dest = into / info.filename
            if top not in dest.resolve().parents and dest.resolve() != top:
                raise Failure(f"{archive}: {info.filename} would land outside {into}")
            if info.is_dir():
                dest.mkdir(parents=True, exist_ok=True)
                continue
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(z.read(info))
            mode = (info.external_attr >> 16) & 0o777
            if mode:
                dest.chmod(mode)


def unzip_app(archive: Path, into: Path, name: str) -> Path:
    """Unpack a Godot macOS export, keeping each file's mode, as `<name>.app`."""
    staging = into / (name + ".unzip")
    shutil.rmtree(staging, ignore_errors=True)
    extract_zip(archive, staging)
    apps = [p for p in staging.iterdir() if p.suffix == ".app"]
    if len(apps) != 1:
        raise Failure(f"{archive}: expected one .app, found {len(apps)}")
    app = into / (name + ".app")
    shutil.rmtree(app, ignore_errors=True)
    apps[0].rename(app)  # renaming the bundle folder keeps its signature (D-Ra-4)
    shutil.rmtree(staging)
    return app


def runtime_executable(clients_dir: Path, target: Target) -> Path:
    if not target.macos:
        return clients_dir / (RUNTIME_NAME + target.exe)
    programs = sorted((clients_dir / (RUNTIME_APP + ".app") / "Contents" / "MacOS").iterdir())
    if len(programs) != 1:
        raise Failure(f"{RUNTIME_APP}.app: expected one program in Contents/MacOS")
    return programs[0]


ETC2_ASTC = "textures/vram_compression/import_etc2_astc"


@contextlib.contextmanager
def etc2_astc_for(project: Path, target: Target):
    """A macOS universal export requires ETC2/ASTC imports (P-5). The 3D client's textures are
    VRAM-compressed, so setting it in project.godot for good would make every checkout encode an unused
    ASTC copy of each texture and rewrite every texture's `.import` file (measured 2026-10-19: 213
    files; no ASTC file reaches the macOS pack). The setting is therefore given only to the packaging
    run: the line is added to project.godot, and project.godot and every `.import` file the import
    rewrote are restored afterwards (D-Ra-6). Godot ignores `override.cfg` in an editor export
    (measured). A project that already sets it (the 2D client, D-Ra-3) is left alone."""
    settings = project / "project.godot"
    original = settings.read_text(encoding="utf-8")
    needed = target.macos and ETC2_ASTC not in original
    if needed:
        line = f"{ETC2_ASTC}=true\n"
        patched = (original.replace("[rendering]\n", "[rendering]\n" + line, 1) if "[rendering]\n" in original
                   else original + "\n[rendering]\n\n" + line)
        settings.write_text(patched, encoding="utf-8")
    try:
        yield
    finally:
        if needed:
            settings.write_text(original, encoding="utf-8")
            rel = project.relative_to(ROOT).as_posix()
            rewritten = [p for p in git("diff", "--name-only", "--", rel).splitlines() if p.endswith(".import")]
            if rewritten:
                git("checkout", "--", *rewritten)


def export(target: Target, keys: list[str], godot: str) -> dict[str, Path]:
    """The shared runtime and each client's pack for `target`, into target/package/<T>/clients/ (PR S23-RT
    §4.2). Every client's settings are checked first (§4.4), so nothing is exported for a client the runtime
    could not start. Returns each client's executable: the runtime, for all of them."""
    out = work_dir(target) / "clients"
    shutil.rmtree(out, ignore_errors=True)  # never ship a client left by an earlier run
    out.mkdir(parents=True)
    settings = {}
    for key in keys:
        try:
            settings[key] = client_settings(ROOT, CLIENTS[key].project)
        except ValueError as problems:
            raise Failure(f"a client the shared runtime cannot start:\n  {problems}") from None
    runtime = export_runtime(target, godot, out)
    for key in keys:
        client = CLIENTS[key]
        project = ROOT / client.project
        if not has_preset(project, target.preset):
            raise Failure(f"{client.project}: no export preset named {target.preset!r}")
        pack = out / (client.name + ".pck")
        with etc2_astc_for(project, target):
            run([godot, "--headless", "--path", str(project), "--import"])
            run([godot, "--headless", "--path", str(project), "--export-pack", target.preset, str(pack)])
        if not pack.is_file():
            raise Failure(f"the export of {client.project} produced no {pack}")
        (out / (client.name + ".settings.json")).write_text(
            json.dumps(settings[key], indent=1, sort_keys=True) + "\n", encoding="utf-8")
    return {key: runtime for key in keys}


def export_runtime(target: Target, godot: str, out: Path) -> Path:
    """`clients/runtime/` as a full export: the bundle's one engine (PR S23-RT §4.1)."""
    project = ROOT / RUNTIME
    if not has_preset(project, target.preset):
        raise Failure(f"{RUNTIME}: no export preset named {target.preset!r}")
    dest = out / (RUNTIME_NAME + (".zip" if target.macos else target.exe))
    run([godot, "--headless", "--path", str(project), "--import"])
    run([godot, "--headless", "--path", str(project), "--export-release", target.preset, str(dest)])
    if target.macos:
        unzip_app(dest, out, RUNTIME_APP)
        dest.unlink()
    executable = runtime_executable(out, target)
    if not executable.is_file():
        raise Failure(f"the export of {RUNTIME} produced no {executable}")
    return executable


# --------------------------------------------------------------------------------------------- assemble

def git(*args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def tracked(prefix: str) -> list[str]:
    return [p for p in git("ls-files", "-z", prefix).split("\0") if p]


def copy_tracked(prefix: str, runtime: Path) -> None:
    """The tracked files under `prefix`, at the same relative path under `runtime`, minus A-R3's list."""
    for rel in tracked(prefix):
        if forbidden(rel) or Path(rel).name in (".gdignore", "messages.pot"):
            continue
        dest = runtime / rel
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / rel, dest)


def godot_notices(given: str | None, into: Path) -> None:
    into.mkdir(parents=True, exist_ok=True)
    for name, digest in GODOT_NOTICES.items():
        dest = into / name
        if given:
            shutil.copy2(Path(given) / name, dest)
        else:
            api = f"repos/godotengine/godot/contents/{name}?ref={GODOT_TAG}"
            dest.write_bytes(subprocess.check_output(
                ["gh", "api", "-H", "Accept: application/vnd.github.raw", api]))
        if hashlib.sha256(dest.read_bytes()).hexdigest() != digest:
            raise Failure(f"Godot's {name} does not match its pinned SHA-256")


def licences(bundle: Path, godot_dir: str | None) -> None:
    """LICENSES/: Rust notices (DEP-38), Godot's (DEP-41), the font's, the asset records (as .txt)."""
    out = bundle / "LICENSES"
    out.mkdir(parents=True, exist_ok=True)
    run(["cargo", "about", "generate", "--locked", "--manifest-path", "tools/cli/Cargo.toml",
         "-c", "packaging/about.toml", "packaging/about.hbs",
         "-o", str(out / "THIRD_PARTY_RUST.html")], cwd=ROOT)
    godot_notices(godot_dir, out / "godot")
    (out / "fonts").mkdir(parents=True, exist_ok=True)
    shutil.copy2(ROOT / "clients/shared/settings/fonts/OFL.txt", out / "fonts" / "OFL.txt")
    assets = out / "assets"
    assets.mkdir(parents=True, exist_ok=True)
    records = {"clients/3d-spike/ASSETS.md": "3D_ASSETS.txt",
               f"{PACK_2D}/art/PROVENANCE.md": "2D_ART_PROVENANCE.txt"}
    records.update({rel: Path(rel).name for rel in tracked("presentation/mineworld-default/LICENSES")})
    for rel, name in records.items():
        shutil.copy2(ROOT / rel, assets / name)


def file_digest(bundle: Path) -> str:
    lines = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file() and p.name != "BUNDLE.toml"):
        rel = path.relative_to(bundle).as_posix()
        lines.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {rel}\n")
    return hashlib.sha256("".join(lines).encode()).hexdigest()


def workspace_version() -> str:
    """`[workspace.package] version` of Cargo.toml (read by hand: macOS's python3 is 3.9, without tomllib)."""
    text = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    section = text.split("[workspace.package]", 1)[1].split("\n[", 1)[0]
    match = re.search(r'^version\s*=\s*"([^"]+)"', section, re.M)
    if not match:
        raise Failure("Cargo.toml: no [workspace.package] version")
    return match.group(1)


def source_epoch() -> int:
    return int(os.environ.get("SOURCE_DATE_EPOCH") or git("show", "-s", "--format=%ct", "HEAD"))


def write_bundle_toml(bundle: Path, target: Target, version: str, executables: dict[str, Path]) -> None:
    runtime = bundle / "runtime"
    fields = {"version": version, "commit": git("rev-parse", "HEAD"), "target": target.name,
              "built": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(source_epoch())),
              "godot": GODOT_TAG, "server": "mineworld" + target.exe, "files_sha256": file_digest(bundle)}
    for key, client in CLIENTS.items():
        exe = executables.get(key)
        fields[f"client_{key}"] = exe.relative_to(runtime).as_posix() if exe else ""
        fields[f"client_{key}_pack"] = f"clients/{client.name}.pck" if exe else ""
        fields[f"client_{key}_scene"] = client.scene if exe else ""
    text = (PACKAGING / "BUNDLE.toml.in").read_text(encoding="utf-8").format_map(fields)
    (runtime / "BUNDLE.toml").write_text(text, encoding="utf-8")


def layout(bundle: Path, target: Target, server: Path, version: str) -> Path:
    """Everything but the clients and the notices, at the places ARC-80 fixes."""
    shutil.rmtree(bundle, ignore_errors=True)
    runtime = bundle / "runtime"
    runtime.mkdir(parents=True)
    play = (PACKAGING / "PLAY.txt").read_text(encoding="utf-8").replace("{version}", version)
    (bundle / "PLAY.txt").write_text(play, encoding="utf-8")
    for name in ("LICENSE", "NOTICE"):
        shutil.copy2(ROOT / name, bundle / name)
    shutil.copy2(server, runtime / server.name)
    for world in WORLDS:
        copy_tracked(f"worlds/{world}", runtime)
    for part in PACK_2D_PARTS:
        copy_tracked(f"{PACK_2D}/{part}", runtime)
    for part in PACK_3D_PARTS:
        copy_tracked(f"{PACK_3D}/{part}", runtime)
    for folder in SHARED_FILES:
        copy_tracked(folder, runtime)
    return runtime


def assemble(target: Target, keys: list[str], godot: str, server: str | None,
             notices: str | None, archive: bool, launchers: str | None = None) -> Path:
    version = workspace_version()
    bundle = work_dir(target) / f"MineWorld-{version}-{target.name}"
    built = Path(server) if server else build(target)
    binaries = given_launchers(launchers, target) if launchers else build_launchers(target)
    executables = export(target, keys, godot)
    runtime = layout(bundle, target, built, version)
    shutil.copytree(work_dir(target) / "clients", runtime / "clients", dirs_exist_ok=True, symlinks=True)
    placed = {k: runtime / "clients" / e.relative_to(work_dir(target) / "clients") for k, e in executables.items()}
    place_launchers(bundle, target, binaries, version)
    licences(bundle, notices)
    write_bundle_toml(bundle, target, version, placed)
    missing = missing_entry_points(bundle, target)
    if missing:
        raise Failure("the bundle lacks its launcher:\n  " + "\n  ".join(missing))
    problems = check_contents(bundle)
    if problems:
        raise Failure("the bundle holds what must not ship:\n  " + "\n  ".join(problems))
    path = write_archive(bundle, target.archive, source_epoch()) if archive else None
    if budget(bundle, path, LIMITS):
        raise Failure("the bundle is over its budget")
    return bundle


# -------------------------------------------------------------------------------------- check, archive

def pck_files(bundle: Path) -> list[Path]:
    return sorted(p for p in bundle.rglob("*.pck") if p.is_file())


def check_contents(bundle: Path) -> list[str]:
    """Every file of the bundle, and every file inside its packs, against A-R3's list."""
    problems = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file()):
        rel = path.relative_to(bundle).as_posix()
        if why := forbidden(rel):
            problems.append(f"{rel}: {why}")
    for pck in pck_files(bundle):
        for name, _ in read_pck(pck):
            if why := forbidden(name):
                problems.append(f"{pck.name}:{name}: {why}")
    return problems


def file_mode(path: Path) -> int:
    return 0o755 if os.access(path, os.X_OK) or path.suffix == ".exe" else 0o644


def write_archive(bundle: Path, kind: str, epoch: int) -> Path:
    """`.zip` or `.tar.gz` with sorted entries, one mtime and normalized modes (step-23 §5.3)."""
    files = sorted(p for p in bundle.rglob("*") if p.is_file())
    top = bundle.name
    out = bundle.parent / f"{top}.{kind}"
    if kind == "zip":
        stamp = time.gmtime(max(epoch, 315532800))[:6]
        with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
            for path in files:
                info = zipfile.ZipInfo(f"{top}/{path.relative_to(bundle).as_posix()}", stamp)
                info.external_attr = (0o100000 | file_mode(path)) << 16
                info.compress_type = zipfile.ZIP_DEFLATED
                z.writestr(info, path.read_bytes())
        return out
    with out.open("wb") as raw, gzip.GzipFile(fileobj=raw, mode="wb", mtime=epoch, compresslevel=6) as gz:
        with tarfile.open(fileobj=gz, mode="w", format=tarfile.PAX_FORMAT) as tar:
            for path in files:
                info = tar.gettarinfo(str(path), f"{top}/{path.relative_to(bundle).as_posix()}")
                info.mtime, info.mode, info.uid, info.gid, info.uname, info.gname = (
                    epoch, file_mode(path), 0, 0, "", "")
                with path.open("rb") as f:
                    tar.addfile(info, f)
    return out


def budget(bundle: Path, archive: Path | None, limits: dict[str, Limit]) -> list[str]:
    """Prints the measured sizes and the 20 largest files of each pack; returns the hard-limit failures."""
    failures = []

    def judge(what: str, size: int, limit: Limit) -> None:
        verdict = "FAIL" if size > limit.hard else ("over goal" if size > limit.goal else "ok")
        print(f"[budget] {what}: {size / MB:.1f} MB (hard {limit.hard / MB:.0f}, goal {limit.goal / MB:.0f}) {verdict}")
        if size > limit.hard:
            failures.append(f"{what} is {size} bytes, over {limit.hard}")

    if archive is not None:
        judge(archive.name, archive.stat().st_size, limits["bundle"])
    for key in CLIENTS:
        packs = [p for p in pck_files(bundle) if CLIENTS[key].name in p.as_posix()]
        extra = [p for p in (bundle / "runtime" / PACK_2D).rglob("*") if p.is_file()] if key == "2d" else []
        if packs:
            judge(f"{key} pack" + (" and the 2D Presentation Pack" if extra else ""),
                  compressed_size(packs + extra), limits[key])
        for pack in packs:
            print(f"[budget] largest files of {pack.name}:")
            for name, size in sorted(read_pck(pack), key=lambda e: -e[1])[:20]:
                print(f"[budget]   {size:>12}  {name}")
    return failures


# ------------------------------------------------------------------------------------------------ probe

JOIN = re.compile(r"^\[mineworld\] invite (\S+) .* join with: (\S+) ")


def read_bundle_toml(runtime: Path) -> dict[str, str]:
    """BUNDLE.toml's `key = "value"` lines; the file is ours and holds nothing else."""
    text = (runtime / "BUNDLE.toml").read_text(encoding="utf-8")
    return dict(re.findall(r'^(\w+) = "(.*)"$', text, re.M))


def start_server(runtime: Path, info: dict[str, str], world: str, scratch: Path):
    log = (scratch / "server.log").open("w", encoding="utf-8")
    flags = subprocess.CREATE_NEW_PROCESS_GROUP if os.name == "nt" else 0
    server = subprocess.Popen(
        [str(runtime / info["server"]), "server", str(runtime / "worlds" / world), "--listen",
         "127.0.0.1:0", "--save", str(scratch / "save"), "--agent", "alice"],
        stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
    deadline = time.monotonic() + 60
    while time.monotonic() < deadline and server.poll() is None:
        for line in (scratch / "server.log").read_text(encoding="utf-8", errors="replace").splitlines():
            if match := JOIN.match(line):
                return server, log, match.group(2), match.group(1)
        time.sleep(0.2)
    stop(server)
    log.close()
    raise Failure(f"the server printed no join line; its log: {scratch / 'server.log'}")


def stop(process: subprocess.Popen) -> None:
    """The graceful path first (Ctrl-C, or Ctrl-Break on Windows), then only our own child is killed."""
    if process.poll() is None:
        process.send_signal(signal.CTRL_BREAK_EVENT if os.name == "nt" else signal.SIGINT)
        try:
            process.wait(10)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()


def probe_client(runtime: Path, info: dict[str, str], client: Client, timeout: float) -> bool:
    with tempfile.TemporaryDirectory(prefix="mineworld-package-") as tmp:
        scratch = Path(tmp)
        server, log, address, invite = start_server(runtime, info, client.world, scratch)
        command = [str(runtime / info[f"client_{client.key}"]), "--headless", "--",
                   f"--client={client.key}", f"--server={address}", f"--invite={invite}", f"--root={runtime}", "--settings=none",
                   # the world whose far field the client draws (ARC-102); a joining client names it
                   f"--world-key={client.world}", *client.args]
        print(f"[probe] {client.key}: {client.world} on {address}", flush=True)
        try:
            done = subprocess.run(command, capture_output=True, text=True, errors="replace", timeout=timeout)
            output, code = done.stdout + done.stderr, done.returncode
        except subprocess.TimeoutExpired as late:
            output, code = str(late.stdout or "") + str(late.stderr or ""), None
        finally:
            stop(server)
            log.close()
        return judge_client(client, output, code, runtime)


def judge_client(client: Client, output: str, code: int | None, runtime: Path) -> bool:
    lines = output.splitlines()
    checks = [(code == 0, f"exit code {code}"),
              (client.verdict in lines or any(l.strip() == client.verdict for l in lines),
               f"the verdict line {client.verdict!r}")]
    if client.key == "2d":  # A-R1: the pack came from the bundle, not plain drawing
        expected = (runtime / PACK_2D).as_posix()
        checks.append((any("presentation " + expected in l.replace("\\", "/") for l in lines),
                       f"the Presentation Pack loaded from {expected}"))
    for good, what in checks:
        print(f"[probe] {client.key}: {'PASS' if good else 'FAIL'} {what}")
    if not all(good for good, _ in checks):
        print("\n".join(lines[-40:]))
    return all(good for good, _ in checks)


def probe(bundle: Path, keys: list[str], timeout: float) -> bool:
    runtime = bundle / "runtime"
    info = read_bundle_toml(runtime)
    if file_digest(bundle) != info["files_sha256"]:
        print("[probe] FAIL the bundle's files do not match BUNDLE.toml")
        return False
    print(f"[probe] bundle {info['version']} {info['target']} at {bundle}")
    return all([probe_client(runtime, info, CLIENTS[k], timeout) for k in keys])


# ------------------------------------------------------------------------------------------------ smoke

# step-23 §9.1 through the launcher (§9.2, R-c C5). Each mode's clients, and the §9.1 step it covers.
MODE_CLIENTS = {"2d": ("2d",), "3d": ("3d",), "both": ("2d", "3d")}
MODE_STEP = {"2d": "4", "3d": "5", "both": "4+5"}
UNPACK_FOLDER = "MineWorld 世界"  # a space and a non-ASCII character, as players' folders have (§9.1 step 1)
PID = re.compile(r"\(pid (\d+)\)")


@dataclass(frozen=True)
class Step:
    number: str
    what: str
    verdict: str     # PASS, FAIL or NOT RUN; a step not run is never a pass
    why: str = ""


def unpack(source: Path, into: Path) -> Path:
    """§9.1 step 1: the archive unpacked (modes kept) into `into`, or a bundle folder copied there."""
    into.mkdir(parents=True)
    if source.is_dir():
        shutil.copytree(source, into / source.name, symlinks=True)
    elif source.name.endswith(".zip"):
        extract_zip(source, into)
    elif source.name.endswith(".tar.gz"):
        with tarfile.open(source) as tar:
            if hasattr(tarfile, "data_filter"):
                tar.extractall(into, filter="data")
            else:
                tar.extractall(into)
    else:
        raise Failure(f"{source}: neither a .zip, a .tar.gz nor a bundle folder")
    tops = list(into.iterdir())
    if len(tops) != 1 or not tops[0].is_dir():
        raise Failure(f"{source}: expected one bundle folder at its top, found {len(tops)} entries")
    return tops[0]


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        while chunk := f.read(1 << 20):
            digest.update(chunk)
    return digest.hexdigest()


def check_sums(source: Path, sums: Path | None) -> Step:
    """§9.1 step 2, first half: the archive's line of SHA256SUMS. Absent, it is NOT RUN and says so."""
    what = "the archive against SHA256SUMS"
    if source.is_dir():
        return Step("2", what, "NOT RUN", "a bundle folder was given, not an archive")
    sums = sums or source.parent / "SHA256SUMS"
    if not sums.is_file():
        return Step("2", what, "NOT RUN", f"no {sums} (the release workflow writes it)")
    lines = re.findall(r"^([0-9a-fA-F]{64}) [ *](.+?)\r?$", sums.read_text(encoding="utf-8"), re.M)
    listed = {name: digest for digest, name in lines}
    if source.name not in listed:
        return Step("2", what, "FAIL", f"{sums} has no line for {source.name}")
    if listed[source.name].lower() != sha256_file(source):
        return Step("2", what, "FAIL", f"{source.name} does not match its SHA-256 in {sums}")
    return Step("2", what, "PASS")


def check_bundle(bundle: Path) -> tuple[Step, Target | None]:
    """§9.1 step 2, second half: BUNDLE.toml's file digest, its target, and the launcher entry points."""
    problems = []
    try:
        info = read_bundle_toml(bundle / "runtime")
    except OSError as error:
        return Step("2", "BUNDLE.toml", "FAIL", str(error)), None
    if file_digest(bundle) != info.get("files_sha256"):
        problems.append("the files do not match BUNDLE.toml's files_sha256")
    target = TARGETS.get(info.get("target", ""))
    if target is None:
        problems.append(f"BUNDLE.toml names no known target ({info.get('target')!r})")
    else:
        problems += missing_entry_points(bundle, target)
    return Step("2", "BUNDLE.toml's file list and the launcher entry points", "FAIL" if problems else "PASS",
                "; ".join(problems)), target


def server_digest_step() -> Step:
    """§9.1 step 3 is not run (D-RC-5): the AC-8 reference record it compares with does not exist yet."""
    return Step("3", "the release server alone against the AC-8 record of this commit", "NOT RUN",
                "no AC-8 reference record exists yet (open question Q-Rb-3; step-23 D-RC-5)")


def alive(pid: int) -> bool:
    if os.name == "nt":
        listed = subprocess.run(["tasklist", "/FI", f"PID eq {pid}", "/NH", "/FO", "CSV"],
                                capture_output=True, text=True).stdout
        return f'"{pid}"' in listed
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    except PermissionError:
        return True
    stat = Path(f"/proc/{pid}/stat")  # an unreaped child in a container is a zombie, not alive
    with contextlib.suppress(OSError, IndexError):
        return stat.read_text().rsplit(")", 1)[1].split()[0] not in ("Z", "X")
    return True


def kill(pid: int) -> None:
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/PID", str(pid)], capture_output=True)
    else:
        with contextlib.suppress(OSError):
            os.kill(pid, signal.SIGKILL)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError:
        return ""


def run_logs(user: Path) -> tuple[Path, str] | None:
    """The logs folder and the `<run>` prefix of the one run in `user` (`LAUNCHER.md` §5)."""
    logs = user / "logs"
    launched = sorted(logs.glob("*-launch.log"))
    return (logs, launched[-1].name[:-len("-launch.log")]) if launched else None


def clean_exit(user: Path) -> list[str]:
    """§9.1 step 6 for the run in `user`: the server's shutdown lines, a graceful stop, no process the
    launcher started still running, and no smoke scratch left."""
    found = run_logs(user)
    if found is None:
        return [f"the launcher wrote no log under {user / 'logs'}"]
    logs, run_id = found
    problems = []
    server = read_text(logs / f"{run_id}-server.log")
    stopping, ticks = server.find("[mineworld] stopping"), server.find("[world] ticks ")
    if not 0 <= stopping < ticks:
        problems.append(f"{run_id}-server.log lacks the shutdown lines ([mineworld] stopping, then [world] ticks)")
    launch = read_text(logs / f"{run_id}-launch.log")
    if "server stopped" not in launch or "was killed" in launch:
        problems.append(f"{run_id}-launch.log shows no graceful server stop")
    still = [pid for pid in map(int, PID.findall(launch)) if alive(pid)]
    if still:
        problems.append(f"still running after the launcher exited: pid {', '.join(map(str, still))}")
    if (user / "scratch").exists():
        problems.append(f"{user / 'scratch'} was not removed")
    return problems


def run_entry(executable: Path, mode: str, user: Path, timeout: float) -> tuple[int | None, str]:
    """The entry point itself, as a player's double-click would start it, in smoke mode. No `--bundle`:
    the launcher finds the bundle from its own path. Over `timeout` it is killed with what it started."""
    command = [str(executable), f"--smoke={mode}", "--no-dialog", f"--user-dir={user}"]
    print("$ " + " ".join(command), flush=True)
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    try:
        output, _ = process.communicate(timeout=timeout)
        return process.returncode, output.decode("utf-8", "replace")
    except subprocess.TimeoutExpired:
        process.kill()
        output, _ = process.communicate()
        found = run_logs(user)
        for pid in map(int, PID.findall(read_text(found[0] / f"{found[1]}-launch.log") if found else "")):
            kill(pid)
        return None, output.decode("utf-8", "replace") + f"\n(killed after {timeout:.0f} s)"


def smoke_mode(bundle: Path, target: Target, mode: str, user: Path, timeout: float) -> list[Step]:
    """§9.1 steps 4/5 and 6 for one entry point."""
    entry = ENTRY_POINTS[mode]
    code, output = run_entry(entry_executable(bundle, target, entry), mode, user, timeout)
    found = run_logs(user)
    verdicts = []
    for key in MODE_CLIENTS[mode]:
        log = read_text(found[0] / f"{found[1]}-client-{key}.log") if found else ""
        verdicts.append(judge_client(CLIENTS[key], log, code, bundle / "runtime"))
    what = f"{entry.name!r} --smoke={mode}, headless, through the launcher"
    passed = code == 0 and all(verdicts)
    why = "" if passed else f"launcher exit {code}; its output: {output.strip()[-400:]}"
    problems = clean_exit(user)
    return [Step(MODE_STEP[mode], what, "PASS" if passed else "FAIL", why),
            Step("6", f"clean exit after {entry.name!r}", "FAIL" if problems else "PASS", "; ".join(problems))]


def summary(steps: list[Step]) -> int:
    """One line per step; exits 1 on any FAIL. A step NOT RUN is named, never counted as a pass."""
    for step in steps:
        print(f"[smoke] step {step.number} {step.verdict}: {step.what}" + (f" ({step.why})" if step.why else ""))
    failed = [s.number for s in steps if s.verdict == "FAIL"]
    not_run = [s.number for s in steps if s.verdict == "NOT RUN"]
    if failed:
        print(f"[smoke] FAIL: step(s) {', '.join(failed)}")
        return 1
    print("[smoke] no step failed; " + (f"NOT RUN, so not passed: step(s) {', '.join(not_run)}" if not_run
                                        else "every step passed"))
    return 0


def smoke(source: Path, modes: list[str], sums: Path | None, timeout: float) -> int:
    """step-23 §9.1 steps 1-6 on this machine (§9.2): unpack, check, then each mode's entry point."""
    scratch = Path(tempfile.mkdtemp(prefix="mineworld-smoke-")).resolve()
    steps: list[Step] = []
    try:
        try:
            bundle = unpack(source, scratch / UNPACK_FOLDER)
            steps.append(Step("1", f"unpacked {source.name} into {bundle}", "PASS"))
        except (Failure, OSError, zipfile.BadZipFile, tarfile.TarError) as error:
            steps.append(Step("1", f"unpack {source.name}", "FAIL", str(error)))
            return summary(steps)
        steps.append(check_sums(source, sums))
        contents, target = check_bundle(bundle)
        steps += [contents, server_digest_step()]
        if contents.verdict != "PASS" or target is None:
            return summary(steps)
        if target != host_target():
            steps.append(Step("4-6", "the launcher runs", "NOT RUN",
                              f"a {target.name} bundle cannot run on this {host_target().name} machine"))
            return summary(steps)
        for mode in modes:
            steps += smoke_mode(bundle, target, mode, scratch / "user" / mode, timeout)
        return summary(steps)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)


# -------------------------------------------------------------------------------------------- self-test

def write_pck(path: Path, files: dict[str, bytes]) -> None:
    """A minimal format-3 pack, for the self-test's planted cases."""
    body = b"".join(files.values())
    head = PCK_MAGIC + struct.pack("<5I", 3, 4, 7, 2, 2) + struct.pack("<2Q", 0x70, 0x70 + len(body))
    head += b"\0" * (0x70 - len(head))
    directory, offset = struct.pack("<I", len(files)), 0
    for name, data in files.items():
        raw = name.encode() + b"\0" * (4 - len(name) % 4)
        directory += struct.pack("<I", len(raw)) + raw + struct.pack("<2Q", offset, len(data)) + b"\0" * 20
        offset += len(data)
    path.write_bytes(head + body + directory)


def settings_cases(cases: list[tuple[str, bool]]) -> None:
    """PR S23-RT A-RT-5: the shared runtime's settings check passes the real clients and fails planted keys."""
    for client in CLIENTS.values():
        try:
            client_settings(ROOT, client.project)
            cases.append((f"{client.project} passes the shared-runtime settings check", True))
        except ValueError as problems:
            cases.append((f"{client.project} passes the shared-runtime settings check ({problems})", False))
    appliers = json.loads((ROOT / RUNTIME / "runtime" / "appliers.json").read_text(encoding="utf-8"))
    runtime = (ROOT / RUNTIME / "project.godot").read_text(encoding="utf-8")
    good = '[application]\nconfig/name="X"\n[rendering]\nrenderer/rendering_method="forward_plus"\n'
    planted = {
        "an autoload": good + '[autoload]\nGame="*res://game.gd"\n',
        "an unknown rendering key": good + "global_illumination/sdfgi/probe_ray_count=2\n",
        "a start-up value unlike the runtime's": '[rendering]\nrenderer/rendering_method="gl_compatibility"\n',
        "an input action the runtime lacks": good + '[input]\nfly={\n"deadzone": 0.2,\n"events": []\n}\n',
    }
    cases.append(("a client with only shared and run-time keys passes", judge(good, runtime, appliers)[0] == []))
    for what, text in planted.items():
        cases.append((f"a client with {what} fails the settings check", len(judge(text, runtime, appliers)[0]) == 1))


def desktop_exec_argv(value: str) -> list[str]:
    """The argv a Desktop Entry `Exec` value means: the file's string escapes, then the quoting rules."""
    text = re.sub(r"\\([sntr\\])", lambda m: {"s": " ", "n": "\n", "t": "\t", "r": "\r", "\\": "\\"}[m.group(1)],
                  value)
    argv, word, quoted, escaped, started = [], "", False, False, False
    for ch in text:
        if escaped:
            word, escaped = word + ch, False
        elif quoted and ch == "\\":
            escaped = True
        elif ch == '"':
            quoted, started = not quoted, True
        elif ch == " " and not quoted:
            if word or started:
                argv.append(word)
            word, started = "", False
        else:
            word += ch
    if word or started:
        argv.append(word)
    return argv


def launcher_cases(cases: list[tuple[str, bool]]) -> None:
    """R-c C5: a bundle without its launcher fails the entry-point check on every target; the wrappers say
    what they start."""
    with tempfile.TemporaryDirectory(prefix="mineworld-package-") as tmp:
        fake = Path(tmp) / "bin"
        fake.mkdir()
        for target in TARGETS.values():
            bundle = Path(tmp) / target.name
            (bundle / "runtime").mkdir(parents=True)
            per_entry = 1 if target.exe else 2
            cases.append((f"a {target.name} bundle without its launcher fails the entry-point check",
                          len(missing_entry_points(bundle, target)) == per_entry * len(ENTRY_POINTS)))
            binaries = {}
            for entry in ENTRY_POINTS.values():
                binaries[entry.mode] = fake / (entry.binary + target.exe)
                binaries[entry.mode].write_bytes(b"launcher " + entry.mode.encode())
            place_launchers(bundle, target, binaries, "0.0.0", sign=False)
            cases.append((f"a {target.name} bundle with its launcher passes it",
                          missing_entry_points(bundle, target) == []))
            (bundle / "MineWorld 3D.app" / "Contents" / "MacOS" / "mineworld-3d-launch").unlink(missing_ok=True)
            (bundle / "MineWorld 3D.exe").unlink(missing_ok=True)
            (bundle / "MineWorld-3D.desktop").unlink(missing_ok=True)
            cases.append((f"a {target.name} bundle missing one entry point's file fails it",
                          len(missing_entry_points(bundle, target)) == 1))
            if target.macos:
                app = bundle / "MineWorld 2D + 3D.app" / "Contents"
                plist = plistlib.loads((app / "Info.plist").read_bytes())
                cases.append(("the .app's Info.plist starts the binary in Contents/MacOS, with no Dock place",
                              (app / "MacOS" / plist["CFBundleExecutable"]).read_bytes() == b"launcher both"
                              and plist["LSUIElement"] is True))
            if target.name == "linux-x86_64":
                text = (bundle / "MineWorld-2D+3D.desktop").read_text(encoding="utf-8")
                line = next(l for l in text.splitlines() if l.startswith("Exec="))
                cases.append(("the .desktop file starts the executable beside it",
                              desktop_exec_argv(line[len("Exec="):])
                              == ["sh", "-c", 'exec "$(dirname "$0")/MineWorld-2D+3D"', "%k"]
                              and "Terminal=false" in text.splitlines()))
                cases.append(("the Linux entry point is executable", os.name == "nt"
                              or os.access(bundle / "MineWorld-2D+3D", os.X_OK)))


def smoke_cases(cases: list[tuple[str, bool]]) -> None:
    """R-c C5: smoke's unpacking, its clean-exit judgement and its report (step 3 is NOT RUN)."""
    with tempfile.TemporaryDirectory(prefix="mineworld-package-") as tmp:
        root = Path(tmp)
        bundle = root / "MineWorld-0.0.0-test"
        (bundle / "runtime").mkdir(parents=True)
        (bundle / "runtime" / "mineworld").write_bytes(b"#!\n")
        (bundle / "runtime" / "mineworld").chmod(0o755)
        for kind in ("zip", "tar.gz"):
            archive = write_archive(bundle, kind, 1_700_000_000)
            into = unpack(archive, root / kind / UNPACK_FOLDER)
            server = into / "runtime" / "mineworld"
            cases.append((f"a .{kind} unpacks into a folder with a space and 世界, keeping the executable bit",
                          into.name == bundle.name and server.read_bytes() == b"#!\n"
                          and (os.name == "nt" or os.access(server, os.X_OK))))
            (root / "SHA256SUMS").write_text(f"{sha256_file(archive)}  {archive.name}\n", encoding="utf-8")
            cases.append((f"the .{kind} matches its SHA256SUMS line", check_sums(archive, None).verdict == "PASS"))
        (root / "SHA256SUMS").write_text(f"{'0' * 64}  {archive.name}\n", encoding="utf-8")
        cases.append(("an archive unlike its SHA256SUMS line fails", check_sums(archive, None).verdict == "FAIL"))
        (root / "SHA256SUMS").unlink()
        cases.append(("without SHA256SUMS the check is NOT RUN", check_sums(archive, None).verdict == "NOT RUN"))

        ended = subprocess.Popen([sys.executable, "--version"], stdout=subprocess.DEVNULL)
        ended.wait()
        user = root / "user"
        logs = user / "logs"
        logs.mkdir(parents=True)
        good_server = "[mineworld] invite x\n[mineworld] stopping\n[world] ticks 5\n"
        (logs / "1-2-server.log").write_text(good_server, encoding="utf-8")
        launch = f"server started (pid {ended.pid})\nclient 2d started as carol (pid {ended.pid})\nserver stopped: exit status: 0\n"
        (logs / "1-2-launch.log").write_text(launch, encoding="utf-8")
        cases.append(("a run that stopped cleanly passes step 6", clean_exit(user) == []))
        (logs / "1-2-server.log").write_text("[mineworld] invite x\n", encoding="utf-8")
        cases.append(("a server log without the shutdown lines fails step 6", len(clean_exit(user)) == 1))
        (logs / "1-2-server.log").write_text(good_server, encoding="utf-8")
        (logs / "1-2-launch.log").write_text(launch + f"client 3d started as x (pid {os.getpid()})\n", encoding="utf-8")
        cases.append(("a process the launcher started still running fails step 6", len(clean_exit(user)) == 1))
        (logs / "1-2-launch.log").write_text(launch, encoding="utf-8")
        (user / "scratch" / "1-2").mkdir(parents=True)
        cases.append(("a smoke scratch left behind fails step 6", len(clean_exit(user)) == 1))

    report = io.StringIO()
    with contextlib.redirect_stdout(report):
        code = summary([Step("1", "unpack", "PASS"), server_digest_step(), Step("6", "clean", "PASS")])
    last = report.getvalue().splitlines()[-1]
    cases.append(("smoke reports step 3 as NOT RUN, never as a pass",
                  server_digest_step().verdict == "NOT RUN" and code == 0 and "NOT RUN" in last and "3" in last
                  and "every step passed" not in report.getvalue()))
    with contextlib.redirect_stdout(io.StringIO()):
        code = summary([server_digest_step(), Step("4", "2d", "FAIL", "exit 3")])
    cases.append(("a failed step fails smoke", code == 1))


def self_test() -> int:
    cases: list[tuple[str, bool]] = []
    for path, bad in (("runtime/clients/x.pck", False), ("a/shots/x.png", True), ("README.md", True),
                      ("clients/3d-spike/tools/blender/x.py", True), ("tools/physics_engine.gdc", False),
                      ("x/.env", True), ("presentation/x/2D/references/y.png", True)):
        cases.append((f"forbidden({path}) is {bad}", (forbidden(path) is not None) == bad))
    with tempfile.TemporaryDirectory(prefix="mineworld-package-") as tmp:
        bundle = Path(tmp) / "MineWorld-0.0.0-test"
        (bundle / "runtime" / "clients").mkdir(parents=True)
        (bundle / "runtime" / "mineworld").write_bytes(b"#!\n")
        (bundle / "runtime" / "mineworld").chmod(0o755)
        pack = bundle / "runtime" / "clients" / "mineworld-3d.pck"
        write_pck(pack, {"scenes/a.scn": b"a" * 10, "shaders/b.gdshader": os.urandom(3000)})
        cases.append(("read_pck reads a written pack",
                      read_pck(pack) == [("scenes/a.scn", 10), ("shaders/b.gdshader", 3000)]))
        cases.append(("a clean bundle passes A-R3", check_contents(bundle) == []))
        (bundle / "shots").mkdir()
        (bundle / "shots" / "x.png").write_bytes(b"x")
        cases.append(("a planted shots/x.png fails A-R3 (A-R3)", len(check_contents(bundle)) == 1))
        shutil.rmtree(bundle / "shots")
        write_pck(pack, {"tools/blender/x.py": b"x"})
        cases.append(("a GPL file inside a pack fails A-R3", len(check_contents(bundle)) == 1))
        write_pck(pack, {"big.bin": os.urandom(3000)})
        small = {k: Limit(1000, 500) for k in LIMITS}
        cases.append(("a planted oversize pack fails the budget (A-R4)", len(budget(bundle, None, small)) == 1))
        cases.append(("within the limits it passes", budget(bundle, None, LIMITS) == []))
        first = write_archive(bundle, "zip", 1_700_000_000).read_bytes()
        second = write_archive(bundle, "zip", 1_700_000_000).read_bytes()
        cases.append(("the zip is byte-identical twice", first == second))
        with zipfile.ZipFile(io.BytesIO(first)) as z:
            mode = z.getinfo("MineWorld-0.0.0-test/runtime/mineworld").external_attr >> 16
        cases.append(("the zip keeps the executable bit", mode & 0o111 != 0))
        tgz = write_archive(bundle, "tar.gz", 1_700_000_000)
        with tarfile.open(tgz) as tar:
            member = tar.getmember("MineWorld-0.0.0-test/runtime/mineworld")
        cases.append(("the tar keeps the executable bit and the fixed mtime",
                      member.mode & 0o111 != 0 and member.mtime == 1_700_000_000))
    settings_cases(cases)
    launcher_cases(cases)
    smoke_cases(cases)
    line = "[mineworld] invite abc123 — join with: 127.0.0.1:5000 seat=carol invite=abc123"
    match = JOIN.match(line)
    cases.append(("the join line is read", bool(match) and match.groups() == ("abc123", "127.0.0.1:5000")))
    for name, good in cases:
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}")
    failed = sum(1 for _, good in cases if not good)
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}: {len(cases)} case(s)")
    return 1 if failed else 0


# ------------------------------------------------------------------------------------------------- main

def main(argv: list[str]) -> int:
    if argv == ["--self-test"]:
        return self_test()
    parser = argparse.ArgumentParser(prog="package.py", description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)
    for name in ("build", "export", "assemble"):
        p = sub.add_parser(name)
        p.add_argument("--target", choices=sorted(TARGETS))
        if name != "build":
            p.add_argument("--clients", default="2d,3d")
            p.add_argument("--godot", default=os.environ.get("GODOT", "godot"))
    assemble_parser = sub.choices["assemble"]
    assemble_parser.add_argument("--server", help="a release server already built for the target")
    assemble_parser.add_argument("--godot-notices", help="a folder holding Godot's LICENSE.txt and COPYRIGHT.txt")
    assemble_parser.add_argument("--no-archive", action="store_true")
    b = sub.add_parser("budget")
    b.add_argument("bundle")
    b.add_argument("archive", nargs="?")
    p = sub.add_parser("probe")
    p.add_argument("bundle")
    p.add_argument("--clients", default="2d,3d")
    p.add_argument("--timeout", type=float, default=300.0)
    assemble_parser.add_argument("--launchers", help="a folder holding the launcher binaries built for the target")
    s = sub.add_parser("smoke")
    s.add_argument("archive", help="a bundle's .zip or .tar.gz, or an assembled bundle folder")
    s.add_argument("--modes", default="2d,3d,both")
    s.add_argument("--sums", help="the SHA256SUMS file; default: the one beside the archive")
    s.add_argument("--timeout", type=float, default=300.0, help="seconds per entry point")
    args = parser.parse_args(argv)
    try:
        return dispatch(args)
    except (Failure, subprocess.CalledProcessError, OSError) as failure:
        print(f"package.py {args.command}: {failure}", file=sys.stderr)
        return 1


def dispatch(args: argparse.Namespace) -> int:
    if args.command == "budget":
        return 1 if budget(Path(args.bundle), Path(args.archive) if args.archive else None, LIMITS) else 0
    if args.command == "probe":
        return 0 if probe(Path(args.bundle).resolve(), args.clients.split(","), args.timeout) else 1
    if args.command == "smoke":
        modes = args.modes.split(",")
        unknown = [m for m in modes if m not in MODE_CLIENTS]
        if unknown:
            raise Failure(f"--modes: unknown {', '.join(unknown)}; expected 2d, 3d, both")
        return smoke(Path(args.archive).resolve(), modes, Path(args.sums) if args.sums else None, args.timeout)
    target = TARGETS[args.target] if args.target else host_target()
    if args.command == "build":
        print(build(target))
        print("\n".join(map(str, build_launchers(target).values())))
    elif args.command == "export":
        print(export(target, args.clients.split(","), args.godot))
    else:
        print(assemble(target, args.clients.split(","), args.godot, args.server, args.godot_notices,
                       not args.no_archive, args.launchers))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
