#!/usr/bin/env python3
"""Build a downloadable MineWorld bundle on this machine (S23 R-a; `docs/DECISIONS.md` ARC-80).

The layout, what is shipped and what never is, and the size budget are specified in
`.structured-coding/plans/mvp0/step-23-release.md` §§3, 7 and 18. This script is the one place that
knows them; CI's release workflow (R-b) calls the same commands.

    python3 scripts/package.py build    [--target T]          the release server for T
    python3 scripts/package.py export   [--target T] [--clients 2d,3d] [--godot BIN]
                                                               the exported clients for T
    python3 scripts/package.py assemble [--target T] [--clients 2d,3d] [--godot BIN] [--server FILE]
                                         [--godot-notices DIR] [--no-archive]
                                                               the bundle folder and its archive
    python3 scripts/package.py budget   <bundle folder> [<archive>]
                                                               the hard limits; the 20 largest files of each pack
    python3 scripts/package.py probe    <bundle folder> [--clients 2d,3d] [--timeout S]
                                                               the bundle's server with each exported client,
                                                               headless, from wherever the folder now is
    python3 scripts/package.py --self-test

T is macos-universal, windows-x86_64, linux-x86_64 or linux-arm64; absent, the machine's own. Work goes
to target/package/<T>/. Every command exits non-zero and names what failed. Standard library only.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import io
import os
import platform
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
PACK_2D_PARTS = ("art", "assets", "i18n", "renderer", "manifest.yaml", "pack.yaml")
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
    name: str        # the exported file's stem under runtime/clients/
    world: str       # the world `probe` hosts for it
    args: tuple[str, ...]
    verdict: str     # the line a passing headless run prints (F-R6: a missing line is FAIL)


CLIENTS = {c.key: c for c in (
    Client("2d", "clients/2d", "mineworld-2d", "market-town", ("--seat=carol", "--drive"),
           "drive complete: PASS"),
    Client("3d", "clients/3d-spike", "mineworld-3d", "social-cafe", ("--slice-link",),
           "all link checks pass"),
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


def build(target: Target) -> Path:
    """The release server for `target`, path-remapped (§5.3); a universal binary on macOS."""
    env = dict(os.environ)
    env["RUSTFLAGS"] = (env.get("RUSTFLAGS", "") + f" --remap-path-prefix={ROOT}=mineworld").strip()
    built = []
    for triple in target.triples:
        run(["cargo", "build", "--release", "--locked", "-p", "mineworld-cli", "--target", triple],
            cwd=ROOT, env=env)
        built.append(ROOT / "target" / triple / "release" / ("mineworld" + target.exe))
    out = work_dir(target) / ("mineworld" + target.exe)
    out.parent.mkdir(parents=True, exist_ok=True)
    if len(built) > 1:
        run(["lipo", "-create", "-output", str(out), *map(str, built)])
    else:
        shutil.copy2(built[0], out)
    return out


# ----------------------------------------------------------------------------------------------- export

def has_preset(project: Path, preset: str) -> bool:
    presets = project / "export_presets.cfg"
    return presets.is_file() and f'name="{preset}"' in presets.read_text(encoding="utf-8")


def unzip_app(archive: Path, into: Path, name: str) -> Path:
    """Unpack a Godot macOS export, keeping each file's mode, as `<name>.app`."""
    staging = into / (name + ".unzip")
    shutil.rmtree(staging, ignore_errors=True)
    with zipfile.ZipFile(archive) as z:
        for info in z.infolist():
            dest = staging / info.filename
            if info.is_dir():
                dest.mkdir(parents=True, exist_ok=True)
                continue
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_bytes(z.read(info))
            mode = (info.external_attr >> 16) & 0o777
            if mode:
                dest.chmod(mode)
    apps = [p for p in staging.iterdir() if p.suffix == ".app"]
    if len(apps) != 1:
        raise Failure(f"{archive}: expected one .app, found {len(apps)}")
    app = into / (name + ".app")
    shutil.rmtree(app, ignore_errors=True)
    apps[0].rename(app)  # renaming the bundle folder keeps its signature (D-Ra-4)
    shutil.rmtree(staging)
    return app


def client_executable(clients_dir: Path, client: Client, target: Target) -> Path:
    if not target.macos:
        return clients_dir / (client.name + target.exe)
    programs = sorted((clients_dir / (client.name + ".app") / "Contents" / "MacOS").iterdir())
    if len(programs) != 1:
        raise Failure(f"{client.name}.app: expected one program in Contents/MacOS")
    return programs[0]


def export(target: Target, keys: list[str], godot: str) -> dict[str, Path]:
    """Each client exported for `target` into target/package/<T>/clients/ (two exports: D-Ra-1)."""
    out = work_dir(target) / "clients"
    shutil.rmtree(out, ignore_errors=True)  # never ship a client left by an earlier run
    out.mkdir(parents=True)
    executables = {}
    for key in keys:
        client = CLIENTS[key]
        project = ROOT / client.project
        if not has_preset(project, target.preset):
            raise Failure(f"{client.project}: no export preset named {target.preset!r}")
        run([godot, "--headless", "--path", str(project), "--import"])
        dest = out / (client.name + (".zip" if target.macos else target.exe))
        run([godot, "--headless", "--path", str(project), "--export-release", target.preset, str(dest)])
        if target.macos:
            unzip_app(dest, out, client.name)
            dest.unlink()
        executables[key] = client_executable(out, client, target)
        if not executables[key].is_file():
            raise Failure(f"the export of {client.project} produced no {executables[key]}")
    return executables


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
    for key in CLIENTS:
        exe = executables.get(key)
        fields[f"client_{key}"] = exe.relative_to(runtime).as_posix() if exe else ""
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
    for folder in SHARED_FILES:
        copy_tracked(folder, runtime)
    return runtime


def assemble(target: Target, keys: list[str], godot: str, server: str | None,
             notices: str | None, archive: bool) -> Path:
    version = workspace_version()
    bundle = work_dir(target) / f"MineWorld-{version}-{target.name}"
    built = Path(server) if server else build(target)
    executables = export(target, keys, godot)
    runtime = layout(bundle, target, built, version)
    shutil.copytree(work_dir(target) / "clients", runtime / "clients", dirs_exist_ok=True, symlinks=True)
    placed = {k: runtime / "clients" / e.relative_to(work_dir(target) / "clients") for k, e in executables.items()}
    licences(bundle, notices)
    write_bundle_toml(bundle, target, version, placed)
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
                   f"--server={address}", f"--invite={invite}", f"--root={runtime}", "--settings=none",
                   *client.args]
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
    target = TARGETS[args.target] if args.target else host_target()
    if args.command == "build":
        print(build(target))
    elif args.command == "export":
        print(export(target, args.clients.split(","), args.godot))
    else:
        print(assemble(target, args.clients.split(","), args.godot, args.server, args.godot_notices,
                       not args.no_archive))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
