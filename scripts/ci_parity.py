#!/usr/bin/env python3
"""AC-8's instrument: record what a platform makes of every world, and compare platforms' records.

`AC-8` (`docs/MVP.md` §9): the same World Pack runs on a laptop and inside a cloud Docker container
with no semantic differences; the operator requires macOS, Linux and Windows alike. How that is measured
is `docs/DECISIONS.md` ARC-49 (design: step-14 §13.4). This script is the whole instrument, so that the
laptop, the macOS and Windows runners and the Linux container all hash with one implementation:

    python3 scripts/ci_parity.py record --binary target/release/mineworld [--out F]
    python3 scripts/ci_parity.py record --image mineworld:ci [--platform linux/amd64] [--out F]
    python3 scripts/ci_parity.py record ... --profile long [--timings T]
    python3 scripts/ci_parity.py compare F F [F ...]
    python3 scripts/ci_parity.py baseline check RECORD [BASELINES]
    python3 scripts/ci_parity.py baseline write RECORD [BASELINES]
    python3 scripts/ci_parity.py diff OLD NEW
    python3 scripts/ci_parity.py --self-test

`record` runs the binary (natively, or in the runtime image with the save on a bind-mounted host
directory, because the image has no Python) for every world under `worlds/`: `validate`; 300 days in
memory, seed 7, keeping every summary line but `wall`; 30 days with `--save`, seed 7, keeping every line
but the header (it names the host's save path) and `wall`, then every stored byte of the save. It writes
one plain-text record, `key value` per line, `\\n` endings on every platform. Nothing in a record is a
wall-clock value or a host path, so two records of one commit on one platform are byte-identical.

`compare` exits 0 only if G-1 … G-5 hold (well-formed; one commit and the same worlds, equal to this
checkout's; Darwin arm64 native, Linux x86_64 in a container and Windows x86_64 native all present; one
toolchain, equal to rust-toolchain.toml's; every world's every key equal across every record). Otherwise it
exits 1, naming each failure, and for an unequal world which platforms agree with which, the first
differing summary line or table chunk. Equality of fewer platforms never passes.

The nightly's long profile (`docs/DECISIONS.md` ARC-83; `pr-13c-nightly.md` §3.2) keeps every key of the
default profile and adds `summary-1000` (memory, seed 7, 1 000 days) and `summary-300s` with every stored
byte of a 300-day save (`manifest-300s`, `table-300s.*`). Its `[source]` names `profile long`; a default
record names no profile and is byte-identical to 13b's. `compare` refuses records of different profiles.
`--timings T` writes wall times and save sizes to a separate JSON file, never into the record.

`baseline check` holds a long record's history keys equal to the committed `scripts/baselines.txt`
(`summary-300`, `summary-1000`, `summary-300s`, `facts-300s`, `journal-300s` for every world, the same
world set both ways); `baseline write` rewrites that file from a long record, keeping each world's
`reason` lines. `diff` locates where two records of different commits first differ, whatever their
platforms.

Standard library only. Every failure of a step is a named verdict, never a traceback.
"""

from __future__ import annotations

import hashlib
import json
import os
import platform
import re
import shutil
import sqlite3
import struct
import subprocess
import sys
import tempfile
import time
import tomllib
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECORD_FORMAT = "1"
SEED = 7
LONG_DAYS = 300
SAVE_DAYS = 30
CHUNK_ROWS = 1000
# Where a container writes its save: a host directory bind-mounted here.
CONTAINER_SAVES = "/var/lib/mineworld/ac8"
# The repository's data-pack roots, named on every command that reads a world (docs/DECISIONS.md
# ARC-77): a world's required Entity and Presentation Packs are found only where a root points. Relative,
# so the same words reach the binary natively (run from the checkout's root) and in the runtime image
# (WORKDIR /opt/mineworld, which holds both directories).
PACK_ROOTS = ["--packs", "entities", "--packs", "presentation/mineworld-default"]
PLATFORM_KEYS = ("arch", "container", "emulated", "os", "rustc", "target", "translated")
SOURCE_KEYS = ("commit", "record-format", "worlds")
WORLD_KEYS = ("manifest", "summary-300", "summary-30s", "validate")
REQUIRED_TABLES = ("facts", "journal", "snapshots")

# The nightly's long profile (ARC-83). `default` is 13b's record, unchanged; `long` adds the keys below.
PROFILES = ("default", "long")
LONG_PROFILE_DAYS = 1000
SAVE_LONG_DAYS = 300
LONG_WORLD_KEYS = ("summary-1000", "summary-300s", "manifest-300s")
# A table key is `<prefix>.<table>.rows` or `<prefix>.<table>.chunk.<n>`: `table` for the 30-day save,
# `table-300s` for the long profile's 300-day save.
TABLE_PREFIXES = ("table", "table-300s")
# Each summary digest and the prefix of its verbatim lines.
SUMMARIES = (("summary-300", "line-300."), ("summary-30s", "line-30s."), ("summary-1000", "line-1000."),
             ("summary-300s", "line-300s."))

# What a world did, as `scripts/baselines.txt` holds it: one constant, so `write` and `check` cannot
# disagree about the key set. Snapshots and manifests are derived storage and are left to cross-platform
# parity: a storage-format change must not read as a behaviour change (pr-13c-nightly.md §3.3).
BASELINE_KEYS = (
    ("summary-300", "summary-300"),
    ("summary-1000", "summary-1000"),
    ("summary-300s", "summary-300s"),
    ("facts-300s", "table-300s.facts.rows"),
    ("journal-300s", "table-300s.journal.rows"),
)
BASELINES = ROOT / "scripts" / "baselines.txt"
BASELINE_HEADER = (
    "# MineWorld long-run baselines: ci_parity.py's long profile, seed 7. Read by the nightly (ARC-83).\n"
    "# A PR that changes a world's behaviour updates its section here, with a line\n"
    "#   reason <PR or step id>: <why the world's history changed>\n"
    "# Regenerate from any platform's long record (they are equal, ARC-49); reason lines are kept:\n"
    "#   python3 scripts/ci_parity.py baseline write <long record> scripts/baselines.txt\n"
)


class Unmet(Exception):
    """A step of recording or comparing could not be done, or did not hold; the message says which."""


# --------------------------------------------------------------------------------------------- record


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def enumerate_worlds(root: Path) -> list[str]:
    """Every directory under worlds/ holding a World Pack manifest (worldpack/src/read.rs MANIFEST)."""
    worlds = sorted(path.parent.name for path in (root / "worlds").glob("*/world.yaml"))
    if not worlds:
        raise Unmet(f"no World Pack under {root / 'worlds'} (no worlds/*/world.yaml)")
    return worlds


def toolchain_channel(root: Path) -> str:
    try:
        with open(root / "rust-toolchain.toml", "rb") as file:
            return str(tomllib.load(file)["toolchain"]["channel"])
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        raise Unmet(f"rust-toolchain.toml: no toolchain.channel ({error})") from error


def command(arguments: list[str], what: str) -> bytes:
    try:
        result = subprocess.run(arguments, cwd=ROOT, capture_output=True)
    except OSError as error:
        raise Unmet(f"{what}: could not start {arguments[0]}: {error}") from error
    if result.returncode != 0:
        raise Unmet(
            f"{what}: exited {result.returncode}\n--- stdout\n{result.stdout.decode(errors='replace')}"
            f"\n--- stderr\n{result.stderr.decode(errors='replace')}"
        )
    return result.stdout


def text_lines(output: bytes, what: str) -> list[str]:
    """Output as lines split on \\n only: a stray \\r stays in its line, where a comparison sees it."""
    try:
        text = output.decode("utf-8")
    except UnicodeDecodeError as error:
        raise Unmet(f"{what}: output is not UTF-8 ({error})") from error
    lines = text.split("\n")
    if lines and lines[-1] == "":
        lines.pop()
    return lines


def normalized_arch(machine: str) -> str:
    machine = machine.strip().lower()
    return {"aarch64": "arm64", "arm64": "arm64", "x86_64": "x86_64", "amd64": "x86_64", "x64": "x86_64"}.get(
        machine, machine
    )


class Native:
    """The binary run directly, on the laptop or a macOS or Windows runner."""

    def __init__(self, binary: str) -> None:
        path = Path(binary)
        if not path.is_absolute():
            path = Path.cwd() / path
        # One layer command names the binary on every OS; Windows adds the suffix.
        if not path.is_file() and os.name == "nt" and path.with_suffix(".exe").is_file():
            path = path.with_suffix(".exe")
        if not path.is_file():
            raise Unmet(f"--binary {binary}: no such file")
        self.binary = str(path)

    def run(self, arguments: list[str], what: str, save: Path | None = None) -> bytes:
        extra = ["--save", str(save)] if save is not None else []
        return command([self.binary, *arguments, *extra], what)

    def platform(self) -> dict[str, str]:
        system = platform.system()
        facts = {
            "os": system,
            "arch": normalized_arch(platform.machine()),
            "container": "none",
            "translated": "absent",
            "emulated": "absent",
        }
        if system == "Darwin":
            # 1 under Rosetta; the key is absent on a Mac that has never run translated code.
            result = subprocess.run(["sysctl", "-n", "sysctl.proc_translated"], capture_output=True, text=True)
            facts["translated"] = result.stdout.strip() if result.returncode == 0 and result.stdout.strip() else "absent"
        if system == "Windows":
            # WOW64 sets PROCESSOR_ARCHITEW6432; x86_64 emulated on an Arm machine names ARM in the
            # processor identifier. Either is refused by G-3, as Rosetta is.
            wow64 = bool(os.environ.get("PROCESSOR_ARCHITEW6432"))
            arm_host = "ARM" in os.environ.get("PROCESSOR_IDENTIFIER", "").upper()
            facts["emulated"] = "1" if wow64 or (arm_host and facts["arch"] == "x86_64") else "0"
        verbose = text_lines(command(["rustc", "-vV"], "rustc -vV"), "rustc -vV")
        found = {line.split(": ", 1)[0]: line.split(": ", 1)[1] for line in verbose if ": " in line}
        if "release" not in found or "host" not in found:
            raise Unmet(f"rustc -vV printed no release or host line: {verbose}")
        facts["rustc"] = found["release"]
        facts["target"] = found["host"]
        return facts


class Image:
    """The runtime image, run by `docker` on the host, with the save on a bind-mounted host directory."""

    def __init__(self, tag: str, docker_platform: str | None) -> None:
        self.tag = tag
        self.docker_platform = docker_platform

    def docker_run(self, entrypoint: list[str], arguments: list[str], what: str, mount: Path | None) -> bytes:
        options = ["--rm"]
        if self.docker_platform:
            options += ["--platform", self.docker_platform]
        if hasattr(os, "getuid"):
            options += ["--user", f"{os.getuid()}:{os.getgid()}"]
        if mount is not None:
            options += ["--volume", f"{mount}:{CONTAINER_SAVES}"]
        return command(["docker", "run", *options, *entrypoint, self.tag, *arguments], what)

    def run(self, arguments: list[str], what: str, save: Path | None = None) -> bytes:
        if save is None:
            return self.docker_run([], arguments, what, None)
        inside = f"{CONTAINER_SAVES}/{save.name}"
        return self.docker_run([], [*arguments, "--save", inside], what, save.parent)

    def platform(self) -> dict[str, str]:
        uname = text_lines(self.docker_run(["--entrypoint", "uname"], ["-sm"], "uname -sm in the image", None), "uname")
        parts = uname[0].split() if uname else []
        if len(parts) != 2:
            raise Unmet(f"uname -sm in the image printed {uname!r}")
        inspected = command(
            ["docker", "image", "inspect", "--format", "{{.Id}} {{.Os}}/{{.Architecture}}", self.tag],
            "docker image inspect",
        ).decode().strip()
        dockerfile = (ROOT / "Dockerfile").read_text(encoding="utf-8")
        tag = re.search(r"^FROM rust:(\d+\.\d+\.\d+)-", dockerfile, re.MULTILINE)
        if not tag:
            raise Unmet("Dockerfile: no `FROM rust:<version>-` line, so the image's toolchain is unknown")
        return {
            "os": parts[0],
            "arch": normalized_arch(parts[1]),
            "container": inspected,
            "translated": "absent",
            "emulated": "absent",
            # The build stage is FROM the toolchain stage, whose rust tag check_ci_pins.py holds equal to
            # rust-toolchain.toml; the binary is the glibc Linux build of that toolchain.
            "rustc": tag.group(1),
            "target": f"{parts[1]}-unknown-linux-gnu",
        }


def summary(lines: list[str], drop_header: bool) -> list[str]:
    kept = [line for line in lines if not line.startswith("wall ")]
    if drop_header:
        kept = [line for line in kept if not line.startswith("[mineworld] run ")]
    return kept


def digest_lines(lines: list[str]) -> str:
    return f"{len(lines)} {sha256(''.join(line + chr(10) for line in lines).encode('utf-8'))}"


def encode_value(value: object) -> bytes:
    """One column: NULL is 0x00; a present value is 0x01 then an i64 little-endian, or a length-prefixed
    blob or UTF-8 text, or an f64 little-endian (none of the schema's columns is real today)."""
    if value is None:
        return b"\x00"
    if isinstance(value, int):
        return b"\x01" + struct.pack("<q", value)
    if isinstance(value, float):
        return b"\x01" + struct.pack("<d", value)
    if isinstance(value, str):
        value = value.encode("utf-8")
    if isinstance(value, bytes):
        return b"\x01" + struct.pack("<Q", len(value)) + value
    raise Unmet(f"a stored value of unexpected type {type(value).__name__}")


def table_digests(connection: sqlite3.Connection, table: str, prefix: str = "table") -> dict[str, str]:
    columns = connection.execute(f'PRAGMA table_info("{table}")').fetchall()
    keys = [column[1] for column in sorted((c for c in columns if c[5] > 0), key=lambda c: c[5])]
    order = keys or [column[1] for column in columns]
    quoted = ", ".join(f'"{name}"' for name in order)
    rows = connection.execute(f'SELECT * FROM "{table}" ORDER BY {quoted}')
    entries: dict[str, str] = {}
    whole = hashlib.sha256()
    count = 0
    chunk = hashlib.sha256()
    first = last = None
    position = [column[1] for column in columns].index(order[0])
    for row in rows:
        encoded = b"".join(encode_value(value) for value in row)
        whole.update(encoded)
        chunk.update(encoded)
        if first is None:
            first = row[position]
        last = row[position]
        count += 1
        if count % CHUNK_ROWS == 0:
            entries[f"{prefix}.{table}.chunk.{count // CHUNK_ROWS - 1:05d}"] = f"{first} {last} {chunk.hexdigest()}"
            chunk, first = hashlib.sha256(), None
    if first is not None:
        entries[f"{prefix}.{table}.chunk.{count // CHUNK_ROWS:05d}"] = f"{first} {last} {chunk.hexdigest()}"
    entries[f"{prefix}.{table}.rows"] = f"{count} {whole.hexdigest()}"
    return entries


def save_digests(file: Path, prefix: str = "table", manifest_key: str = "manifest") -> dict[str, str]:
    if not file.is_file():
        raise Unmet(f"the save {file} was not written")
    connection = sqlite3.connect(file)
    try:
        entries: dict[str, str] = {}
        tables = [row[0] for row in connection.execute("SELECT name FROM sqlite_master WHERE type = 'table'")]
        for table in REQUIRED_TABLES:
            if table not in tables:
                raise Unmet(f"{file}: no table {table}")
        manifest = connection.execute("SELECT format, body FROM manifest").fetchall()
        if len(manifest) != 1:
            raise Unmet(f"{file}: manifest has {len(manifest)} rows, not 1")
        form, body = manifest[0]
        try:
            decoded = json.loads(body)
            decoded.pop("instance")
        except (ValueError, KeyError, AttributeError) as error:
            raise Unmet(f"{file}: the manifest body is not a JSON object with an instance ({error})") from error
        canonical = json.dumps(decoded, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
        entries[manifest_key] = f"{form} {sha256(encode_value(form) + encode_value(canonical))}"
        for table in sorted(tables):
            if table != "manifest":
                entries.update(table_digests(connection, table, prefix))
        return entries
    except sqlite3.Error as error:
        raise Unmet(f"{file}: {error}") from error
    finally:
        connection.close()


def disk_bytes(directory: Path) -> int:
    return sum(path.stat().st_size for path in directory.rglob("*") if path.is_file())


def run_days(runner: Native | Image, world: str, days: int, key: str, prefix: str,
             timings: list[dict[str, object]], save: Path | None = None) -> dict[str, str]:
    """One headless run, seed 7, as `key` (its digest) and `prefix` (its lines). A saved run drops the
    header, which names the host's save path. Its wall time goes to `timings`, never into the record."""
    arguments = ["run", f"worlds/{world}", "--headless", "--seed", str(SEED), "--days", str(days), *PACK_ROOTS]
    mode = "saved" if save is not None else "in memory"
    began = time.monotonic()
    lines = summary(text_lines(runner.run(arguments, f"{world}: run {days} days {mode}", save), world), save is not None)
    timings.append({"run": key, "days": days, "saved": save is not None,
                    "seconds": round(time.monotonic() - began, 2), "save_bytes": None})
    entries = {key: digest_lines(lines)}
    entries.update({f"{prefix}{index:04d}": line for index, line in enumerate(lines)})
    return entries


def saved_run(runner: Native | Image, world: str, days: int, key: str, prefix: str, scratch: Path,
              table_prefix: str, manifest_key: str, timings: list[dict[str, object]]) -> dict[str, str]:
    """A saved run and every stored byte of its save; the save is removed before the next run, so
    peak disk is one save (pr-13c-nightly.md §3.2)."""
    save = scratch / world
    entries = run_days(runner, world, days, key, prefix, timings, save)
    timings[-1]["save_bytes"] = disk_bytes(save)
    entries.update(save_digests(save / "world.sqlite", table_prefix, manifest_key))
    remove(save)
    return entries


def record_world(runner: Native | Image, world: str, scratch: Path, profile: str = "default",
                 timings: list[dict[str, object]] | None = None) -> dict[str, str]:
    began = time.monotonic()
    timings = [] if timings is None else timings
    entries = {"validate": sha256(runner.run(["validate", f"worlds/{world}", *PACK_ROOTS], f"validate worlds/{world}"))}
    entries.update(run_days(runner, world, LONG_DAYS, "summary-300", "line-300.", timings))
    entries.update(saved_run(runner, world, SAVE_DAYS, "summary-30s", "line-30s.", scratch, "table", "manifest", timings))
    if profile == "long":
        entries.update(run_days(runner, world, LONG_PROFILE_DAYS, "summary-1000", "line-1000.", timings))
        entries.update(saved_run(runner, world, SAVE_LONG_DAYS, "summary-300s", "line-300s.", scratch,
                                 "table-300s", "manifest-300s", timings))
    print(f"[parity] {world}: recorded in {time.monotonic() - began:.1f} s", flush=True)
    return entries


def remove(directory: Path) -> None:
    try:
        shutil.rmtree(directory)
    except OSError as error:
        raise Unmet(f"could not remove the scratch save {directory}: {error}") from error


def render(sections: dict[str, dict[str, str]]) -> bytes:
    out = ["# MineWorld AC-8 parity record: scripts/ci_parity.py, docs/DECISIONS.md ARC-49"]
    for name, entries in sections.items():
        out.append(f"[{name}]")
        out.extend(f"{key} {entries[key]}" for key in sorted(entries))
    return ("\n".join(out) + "\n").encode("utf-8")


def record(runner: Native | Image, out: Path | None, profile: str = "default", timings: Path | None = None) -> int:
    started = time.monotonic()
    facts = runner.platform()
    worlds = enumerate_worlds(ROOT)
    commit = command(["git", "rev-parse", "HEAD"], "git rev-parse HEAD").decode().strip()
    source = {"commit": commit, "record-format": RECORD_FORMAT, "worlds": " ".join(worlds)}
    if profile != "default":
        # Written only when not default, so a default record stays 13b's byte for byte (I-13c-2).
        source["profile"] = profile
    sections: dict[str, dict[str, str]] = {"platform": facts, "source": source}
    print(f"[parity] recording {commit} on {facts['os']}/{facts['arch']} (container {facts['container']}), "
          f"profile {profile}", flush=True)
    measured: dict[str, list[dict[str, object]]] = {}
    scratch = Path(tempfile.mkdtemp(prefix="ac8-parity-"))
    try:
        for world in worlds:
            measured[world] = []
            sections[f"world {world}"] = record_world(runner, world, scratch, profile, measured[world])
    finally:
        if scratch.exists():
            shutil.rmtree(scratch, ignore_errors=True)
    if scratch.exists():
        raise Unmet(f"could not remove the scratch directory {scratch}")
    # The default record lands where 13b's jobs upload it from; a long record beside the nightly's
    # other results (artifacts/nightly/, ARC-83), whichever leg made it.
    name = f"-{facts['os'].lower()}-{facts['arch']}.txt"
    target = out or (Path(f"ac8{name}") if profile == "default" else ROOT / "artifacts" / "nightly" / f"parity-{profile}{name}")
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(render(sections))
    print(f"[parity] wrote {target} ({len(worlds)} worlds) in {time.monotonic() - started:.1f} s")
    if timings is not None:
        timings.parent.mkdir(parents=True, exist_ok=True)
        timings.write_text(json.dumps({
            "commit": commit, "profile": profile, "platform": f"{facts['os']}/{facts['arch']}",
            "container": facts["container"] != "none", "seconds": round(time.monotonic() - started, 1),
            "worlds": measured,
        }, indent=1, sort_keys=True) + "\n", encoding="utf-8")
        print(f"[parity] wrote timings {timings}")
    return 0


# -------------------------------------------------------------------------------------------- compare


@dataclass
class Record:
    name: str
    sections: dict[str, dict[str, str]] = field(default_factory=dict)

    def label(self) -> str:
        facts = self.sections.get("platform", {})
        return f"{facts.get('os', '?')}/{facts.get('arch', '?')}"

    def profile(self) -> str:
        """An absent `profile` is 13b's record: `default`."""
        return self.sections.get("source", {}).get("profile", "default")


def parse(name: str, data: bytes) -> Record:
    """G-1: a record parses, and holds every section and key the format requires."""
    parsed = Record(name)
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise Unmet(f"G-1 {name}: not UTF-8 ({error})") from error
    current: dict[str, str] | None = None
    for number, line in enumerate(text.split("\n"), start=1):
        if line == "" or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            current = parsed.sections.setdefault(line[1:-1], {})
            continue
        key, separator, value = line.partition(" ")
        if current is None or not separator:
            raise Unmet(f"G-1 {name}:{number}: not a `key value` line inside a section: {line!r}")
        if key in current:
            raise Unmet(f"G-1 {name}:{number}: {key} appears twice")
        current[key] = value
    for section, keys in (("platform", PLATFORM_KEYS), ("source", SOURCE_KEYS)):
        missing = [key for key in keys if key not in parsed.sections.get(section, {})]
        if missing:
            raise Unmet(f"G-1 {name}: [{section}] lacks {', '.join(missing)}")
    if parsed.sections["source"]["record-format"] != RECORD_FORMAT:
        raise Unmet(f"G-1 {name}: record-format {parsed.sections['source']['record-format']}, not {RECORD_FORMAT}")
    profile = parsed.profile()
    if profile not in PROFILES:
        raise Unmet(f"G-1 {name}: profile {profile}, not one of {', '.join(PROFILES)}")
    required = WORLD_KEYS + (LONG_WORLD_KEYS if profile == "long" else ())
    prefixes = TABLE_PREFIXES if profile == "long" else TABLE_PREFIXES[:1]
    for world in parsed.sections["source"]["worlds"].split():
        entries = parsed.sections.get(f"world {world}")
        if entries is None:
            raise Unmet(f"G-1 {name}: no [world {world}] section")
        missing = [key for key in required if key not in entries]
        missing += [f"{prefix}.{table}.rows" for prefix in prefixes for table in REQUIRED_TABLES
                    if f"{prefix}.{table}.rows" not in entries]
        if missing:
            raise Unmet(f"G-1 {name}: [world {world}] lacks {', '.join(missing)}")
    return parsed


def platform_groups(records: list[Record]) -> list[str]:
    """G-3: the operator's three platforms must each be present, natively where it is native."""
    def has(predicate) -> bool:
        return any(predicate(record.sections["platform"]) for record in records)

    failures = []
    if not has(lambda p: p["os"] == "Darwin" and p["arch"] == "arm64" and p["translated"] in ("0", "absent")
               and p["container"] == "none"):
        failures.append("G-3 no Darwin arm64 record (native, not translated by Rosetta, no container)")
    if not has(lambda p: p["os"] == "Linux" and p["arch"] == "x86_64" and p["container"] != "none"):
        failures.append("G-3 no Linux x86_64 record from the container")
    if not has(lambda p: p["os"] == "Windows" and p["arch"] == "x86_64" and p["emulated"] == "0"
               and p["container"] == "none"):
        failures.append("G-3 no Windows x86_64 record (native, not emulated)")
    return failures


def grouping(labels: list[str], values: list[tuple]) -> str:
    groups: dict[tuple, list[str]] = {}
    for label, value in zip(labels, values):
        groups.setdefault(value, []).append(label)
    parts = sorted(groups.values(), key=lambda members: (len(members), members))
    return " ≠ ".join(members[0] if len(members) == 1 else "{" + ", ".join(sorted(members)) + "}" for members in parts)


def first_difference(world: str, keys: list[str], records: list[Record], labels: list[str]) -> list[str]:
    """Where an unequal world first differs: a summary line shown as text, or a table chunk located."""
    notes = []
    entries = [record.sections[f"world {world}"] for record in records]
    pairs = list(zip(labels, entries))
    for summary_key, prefix in SUMMARIES:
        if not any(summary_key in entry for entry in entries):
            continue
        for key in keys:
            if key.startswith(prefix) and len({entry.get(key) for entry in entries}) > 1:
                shown = "; ".join(f"{label}: {entry.get(key, '(none)')!r}" for label, entry in pairs)
                notes.append(f"  {summary_key} first differs at line {int(key[len(prefix):])}: {shown}")
                break
        else:
            if len({entry.get(summary_key) for entry in entries}) > 1:
                notes.append(f"  {summary_key} differs in its line count: "
                             + "; ".join(f"{label}: {entry.get(summary_key)}" for label, entry in pairs))
    tables = sorted({tuple(key.split(".")[:2]) for key in keys if key.split(".")[0] in TABLE_PREFIXES})
    for prefix, table in tables:
        rows = f"{prefix}.{table}.rows"
        chunk = next((key for key in keys if key.startswith(f"{prefix}.{table}.chunk.")
                      and len({entry.get(key) for entry in entries}) > 1), None)
        if chunk is None and len({entry.get(rows) for entry in entries}) == 1:
            continue
        counts = ", ".join(f"{label} {(entry.get(rows) or '?').split()[0]}" for label, entry in pairs)
        where = "no chunk differs (only the table digest)"
        if chunk is not None:
            ranges = "; ".join(f"{label}: keys {' … '.join((entry.get(chunk) or '? ?').split()[:2])}"
                               for label, entry in pairs)
            where = f"first differing chunk {int(chunk.rsplit('.', 1)[1])} ({ranges})"
        # 13b's wording for the 30-day save; the long save's tables are named by their prefix.
        name = table if prefix == "table" else f"{table} ({prefix})"
        notes.append(f"  table {name}: rows {counts}; {where}")
    for key in ("validate", "manifest", "manifest-300s"):
        if len({entry.get(key) for entry in entries}) > 1:
            notes.append(f"  {key} differs")
    return notes


def compare_records(records: list[Record], expected_worlds: list[str], channel: str) -> tuple[list[str], list[str]]:
    """Returns (failures, report). No failures means AC-8 holds for these records."""
    failures: list[str] = []
    report: list[str] = []

    commits = {record.sections["source"]["commit"] for record in records}
    if len(commits) != 1:
        failures.append("G-2 the records name different commits: "
                        + ", ".join(f"{r.name} {r.sections['source']['commit']}" for r in records))
    profiles = {record.profile() for record in records}
    if len(profiles) != 1:
        failures.append("G-2 the records name different profiles: "
                        + ", ".join(f"{r.name} {r.profile()}" for r in records))
    world_lists = {record.sections["source"]["worlds"] for record in records}
    if len(world_lists) != 1:
        failures.append("G-2 the records name different worlds: "
                        + "; ".join(f"{r.name}: {r.sections['source']['worlds']}" for r in records))
    elif sorted(world_lists.pop().split()) != sorted(expected_worlds):
        failures.append(f"G-2 the records' worlds are not this checkout's worlds/ ({' '.join(expected_worlds)})")

    failures += platform_groups(records)

    wrong = [f"{r.name} {r.sections['platform']['rustc']}" for r in records if r.sections["platform"]["rustc"] != channel]
    if wrong:
        failures.append(f"G-4 toolchain is not rust-toolchain.toml's {channel}: {', '.join(wrong)}")

    plain = [record.label() for record in records]
    seen: dict[str, int] = {}
    labels = []
    for label in plain:
        seen[label] = seen.get(label, 0) + 1
        labels.append(f"{label}#{seen[label]}" if plain.count(label) > 1 else label)
    compared = 0
    shared = set.intersection(*(set(r.sections["source"]["worlds"].split()) for r in records)) if records else set()
    for world in sorted(shared):
        entries = [record.sections.get(f"world {world}", {}) for record in records]
        keys = sorted(set().union(*entries))
        compared += len(keys)
        signatures = [tuple(entry.get(key) for key in keys) for entry in entries]
        if len(set(signatures)) == 1:
            report.append(f"world {world}: equal on {len(records)} records, {len(keys)} keys "
                          f"(summary-300 {entries[0].get('summary-300')})")
            continue
        failures.append(f"G-5 world {world} differs: {grouping(labels, signatures)}")
        failures.extend(first_difference(world, keys, records, labels))
    report.insert(0, f"{len(records)} records: {', '.join(labels)}; {len(shared)} worlds; {compared} keys compared")
    return failures, report


def compare(paths: list[str]) -> int:
    try:
        records = [parse(path, Path(path).read_bytes()) for path in paths]
    except OSError as error:
        print(f"[parity] FAIL G-1 {error}", file=sys.stderr)
        return 1
    except Unmet as unmet:
        print(f"[parity] FAIL {unmet}", file=sys.stderr)
        return 1
    failures, report = compare_records(records, enumerate_worlds(ROOT), toolchain_channel(ROOT))
    for line in report:
        print(f"[parity] {line}")
    verdict = "AC-8 PASS" if not failures else "AC-8 FAIL"
    for line in failures:
        print(f"[parity] {line}", file=sys.stderr)
    print(f"[parity] {verdict}", flush=True)
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8", newline="\n") as out:
            out.write(f"## {verdict}\n\n```text\n" + "\n".join(report + failures) + "\n```\n")
    return 0 if not failures else 1


# ------------------------------------------------------------------------------------------ baselines


@dataclass
class Baselines:
    """`scripts/baselines.txt`: per world, BASELINE_KEYS' values and the reason lines that explain them."""
    values: dict[str, dict[str, str]] = field(default_factory=dict)
    reasons: dict[str, list[str]] = field(default_factory=dict)


def parse_baselines(name: str, text: str) -> Baselines:
    """A file that does not parse is refused with its line number; a reason is kept, never compared."""
    parsed = Baselines()
    names = [key for key, _ in BASELINE_KEYS]
    world: str | None = None
    for number, line in enumerate(text.split("\n"), start=1):
        if line.strip() == "" or line.startswith("#"):
            continue
        if line.startswith("[world ") and line.endswith("]"):
            world = line[len("[world "):-1]
            if world in parsed.values:
                raise Unmet(f"{name}:{number}: [world {world}] appears twice")
            parsed.values[world], parsed.reasons[world] = {}, []
            continue
        key, separator, value = line.partition(" ")
        if world is None or not separator or not value.strip():
            raise Unmet(f"{name}:{number}: not a `key value` line inside a [world …] section: {line!r}")
        if key == "reason":
            parsed.reasons[world].append(value)
        elif key not in names:
            raise Unmet(f"{name}:{number}: unknown key {key!r} (keys: {', '.join(names)}, reason)")
        elif key in parsed.values[world]:
            raise Unmet(f"{name}:{number}: {key} appears twice in [world {world}]")
        else:
            parsed.values[world][key] = value
    for world, values in parsed.values.items():
        missing = [key for key in names if key not in values]
        if missing:
            raise Unmet(f"{name}: [world {world}] lacks {', '.join(missing)}")
    return parsed


def baselines_of(record: Record) -> dict[str, dict[str, str]]:
    if record.profile() != "long":
        raise Unmet(f"{record.name}: profile {record.profile()}; baselines are read from a long record")
    return {world: {key: record.sections[f"world {world}"][source] for key, source in BASELINE_KEYS}
            for world in record.sections["source"]["worlds"].split()}


def check_baselines(record: Record, baselines: Baselines) -> list[str]:
    """Every world of the record has a baseline and every baseline a world, and every key is equal."""
    found = baselines_of(record)
    failures = [f"no baseline for world {world}" for world in sorted(set(found) - set(baselines.values))]
    failures += [f"a baseline for world {world}, which the record does not have"
                 for world in sorted(set(baselines.values) - set(found))]
    for world in sorted(set(found) & set(baselines.values)):
        for key, _ in BASELINE_KEYS:
            if found[world][key] != baselines.values[world][key]:
                failures.append(f"world {world} key {key}: baseline {baselines.values[world][key]}, "
                                f"record {found[world][key]}")
    return failures


def render_baselines(record: Record, kept: Baselines | None) -> str:
    found = baselines_of(record)
    out = [BASELINE_HEADER.rstrip("\n")]
    for world in sorted(found):
        out.append(f"[world {world}]")
        out.extend(f"reason {reason}" for reason in (kept.reasons.get(world, []) if kept else []))
        out.extend(f"{key} {found[world][key]}" for key, _ in BASELINE_KEYS)
    return "\n".join(out) + "\n"


def read_record(path: str) -> Record:
    try:
        return parse(path, Path(path).read_bytes())
    except OSError as error:
        raise Unmet(f"G-1 {error}") from error


def baseline(action: str, record_path: str, baselines_path: Path) -> int:
    record = read_record(record_path)
    if action == "write":
        kept = parse_baselines(str(baselines_path), baselines_path.read_text(encoding="utf-8")) \
            if baselines_path.is_file() else None
        baselines_path.write_text(render_baselines(record, kept), encoding="utf-8", newline="\n")
        print(f"[baseline] wrote {baselines_path} from {record_path} ({record.label()}, "
              f"commit {record.sections['source']['commit']})")
        return 0
    try:
        text = baselines_path.read_text(encoding="utf-8")
    except OSError as error:
        raise Unmet(f"{baselines_path}: {error}") from error
    failures = check_baselines(record, parse_baselines(str(baselines_path), text))
    worlds = record.sections["source"]["worlds"].split()
    verdict = "baselines PASS" if not failures else "baselines FAIL"
    lines = [f"{len(worlds)} worlds ({' '.join(worlds)}), {len(BASELINE_KEYS)} keys each, against {baselines_path.name}"]
    lines += failures
    for line in lines:
        print(f"[baseline] {line}", file=sys.stderr if line in failures else sys.stdout)
    print(f"[baseline] {verdict}", flush=True)
    summary_file = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary_file:
        with open(summary_file, "a", encoding="utf-8", newline="\n") as out:
            out.write(f"## {verdict}\n\n```text\n" + "\n".join(lines) + "\n```\n")
    return 0 if not failures else 1


def diff_records(old: Record, new: Record) -> list[str]:
    """Where two records first differ, world by world, whatever their platforms or commits."""
    notes = []
    old_worlds = set(old.sections["source"]["worlds"].split())
    new_worlds = set(new.sections["source"]["worlds"].split())
    notes += [f"world {world}: only in {old.name}" for world in sorted(old_worlds - new_worlds)]
    notes += [f"world {world}: only in {new.name}" for world in sorted(new_worlds - old_worlds)]
    for world in sorted(old_worlds & new_worlds):
        entries = [old.sections[f"world {world}"], new.sections[f"world {world}"]]
        keys = sorted(set(entries[0]) | set(entries[1]))
        if all(entries[0].get(key) == entries[1].get(key) for key in keys):
            continue
        notes.append(f"world {world} differs")
        notes.extend(first_difference(world, keys, [old, new], ["old", "new"]))
    return notes


def diff(old_path: str, new_path: str) -> int:
    old, new = read_record(old_path), read_record(new_path)
    notes = diff_records(old, new)
    print(f"[diff] old {old_path}: {old.sections['source']['commit']} ({old.label()}, {old.profile()})")
    print(f"[diff] new {new_path}: {new.sections['source']['commit']} ({new.label()}, {new.profile()})")
    for line in notes or ["no world differs"]:
        print(f"[diff] {line}")
    return 1 if notes else 0


# ------------------------------------------------------------------------------------------ self-test


def synthetic(os_name: str, arch: str, container: str = "none") -> dict[str, dict[str, str]]:
    facts = {
        "os": os_name, "arch": arch, "container": container,
        "translated": "0" if os_name == "Darwin" else "absent",
        "emulated": "0" if os_name == "Windows" else "absent", "rustc": "1.97.1", "target": "t",
    }
    sections = {"platform": facts, "source": {"commit": "c0ffee", "record-format": RECORD_FORMAT, "worlds": "a b"}}
    for world in ("a", "b"):
        sections[f"world {world}"] = {
            "validate": "v", "manifest": "2 m", "summary-300": "2 s", "line-300.0000": "day 1",
            "line-300.0001": "faults 0", "summary-30s": "1 t", "line-30s.0000": "faults 0",
            "table.facts.rows": "2000 f", "table.facts.chunk.00000": "1 1000 x", "table.facts.chunk.00001": "1001 2000 y",
            "table.journal.rows": "1 j", "table.snapshots.rows": "1 s",
        }
    return sections


def four() -> list[dict[str, dict[str, str]]]:
    return [synthetic("Darwin", "arm64"), synthetic("Linux", "x86_64", "sha256:img linux/amd64"),
            synthetic("Windows", "x86_64"), synthetic("Linux", "arm64", "sha256:img linux/arm64")]


def self_test() -> int:
    """The comparator's verdicts against records built here: each case must get its stated verdict."""
    def modified(leg: dict, section: str, key: str, value: str) -> dict:
        copy = {name: dict(entries) for name, entries in leg.items()}
        copy[section][key] = value
        return copy

    base = four()
    linux = base[1]
    cases = [
        ("equal records of the four platforms", base, None),
        ("a differing summary line", [base[0], modified(linux, "world a", "line-300.0001", "faults 1"), *base[2:]],
         "first differs at line 1"),
        ("a differing facts chunk",
         [base[0], modified(modified(linux, "world b", "table.facts.chunk.00001", "1001 2000 z"),
                            "world b", "table.facts.rows", "2000 g"), *base[2:]],
         "first differing chunk 1 (Darwin/arm64: keys 1001 … 2000"),
        ("the grouping places a Windows-only difference",
         [*base[:2], modified(base[2], "world a", "validate", "w"), base[3]],
         "Windows/x86_64 ≠ {Darwin/arm64, Linux/arm64, Linux/x86_64}"),
        ("a missing world", [base[0], modified(linux, "source", "worlds", "a"), *base[2:]], "G-2"),
        ("a same-platform pair", [base[1], base[1]], "G-3 no Darwin arm64"),
        ("three platforms, Windows missing", [base[0], base[1], base[3]], "G-3 no Windows x86_64"),
        ("a Rosetta-translated Mac", [modified(base[0], "platform", "translated", "1"), *base[1:]], "G-3 no Darwin arm64"),
        ("an emulated Windows", [*base[:2], modified(base[2], "platform", "emulated", "1"), base[3]], "G-3 no Windows"),
        ("a Linux record outside the container", [base[0], modified(linux, "platform", "container", "none"), *base[2:]],
         "G-3 no Linux x86_64"),
        ("a commit mismatch", [base[0], modified(linux, "source", "commit", "beef"), *base[2:]], "G-2 the records name different commits"),
        ("another toolchain", [base[0], modified(linux, "platform", "rustc", "1.97.0"), *base[2:]], "G-4"),
    ]
    failed = 0
    for name, legs, expected in cases:
        records = [parse(f"leg{index}", render(leg)) for index, leg in enumerate(legs)]
        failures, _ = compare_records(records, ["a", "b"], "1.97.1")
        text = "\n".join(failures)
        good = not failures if expected is None else bool(failures) and expected in text
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}" + ("" if good else f": got {failures or 'PASS'}"))
    malformed = [b"[platform]\nos\n", render({**base[0], "world b": {"validate": "v"}}),
                 render(base[0]).replace(b"record-format 1", b"record-format 9")]
    for index, data in enumerate(malformed):
        try:
            parse(f"malformed{index}", data)
            good = False
        except Unmet as unmet:
            good = str(unmet).startswith("G-1")
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} a malformed record ({index}) is refused by G-1")
    failed += self_test_long(modified)
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


def lengthened(leg: dict[str, dict[str, str]]) -> dict[str, dict[str, str]]:
    """A synthetic default record turned into a long one: the profile and the long profile's keys."""
    copy = {name: dict(entries) for name, entries in leg.items()}
    copy["source"]["profile"] = "long"
    for world in ("a", "b"):
        copy[f"world {world}"].update({
            "summary-1000": "2 k", "line-1000.0000": "day 1", "line-1000.0001": "faults 0",
            "summary-300s": "1 h", "line-300s.0000": "faults 0", "manifest-300s": "2 m",
            "table-300s.facts.rows": "2000 F", "table-300s.facts.chunk.00000": "1 1000 X",
            "table-300s.facts.chunk.00001": "1001 2000 Y", "table-300s.journal.rows": "3 J",
            "table-300s.snapshots.rows": "1 S",
        })
    return copy


def self_test_long(modified) -> int:
    """The long profile, the baselines and `diff` (pr-13c-nightly.md N-C2): each case its stated verdict."""
    failed = 0

    def judged(name: str, good: bool, got: object) -> None:
        nonlocal failed
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}" + ("" if good else f": got {got}"))

    base = [lengthened(leg) for leg in four()]
    cases = [
        ("equal long records of the four platforms", base, None),
        ("a default record among long ones", [four()[0], *base[1:]], "G-2 the records name different profiles"),
        ("a Windows-only difference after day 300",
         [*base[:2], modified(modified(base[2], "world a", "line-1000.0001", "faults 1"), "world a", "summary-1000", "2 w"),
          base[3]], "summary-1000 first differs at line 1"),
        ("a differing chunk of the 300-day save",
         [base[0], modified(base[1], "world b", "table-300s.facts.chunk.00001", "1001 2000 Z"), *base[2:]],
         "table facts (table-300s): rows"),
    ]
    for name, legs, expected in cases:
        failures, _ = compare_records([parse(f"leg{i}", render(leg)) for i, leg in enumerate(legs)], ["a", "b"], "1.97.1")
        judged(name, not failures if expected is None else bool(failures) and expected in "\n".join(failures),
               failures or "PASS")
    try:
        parse("short", render(modified(base[0], "world a", "summary-1000", "2 k")).replace(b"summary-300s 1 h\n", b""))
        judged("a long record without summary-300s is refused by G-1", False, "accepted")
    except Unmet as unmet:
        judged("a long record without summary-300s is refused by G-1", "summary-300s" in str(unmet), unmet)

    record = parse("long", render(base[1]))
    written = render_baselines(record, None)
    equal = parse_baselines("written", written)
    judged("baselines written from a record check equal", check_baselines(record, equal) == [], check_baselines(record, equal))
    changed = parse_baselines("one key", written.replace("summary-1000 2 k", "summary-1000 2 q", 1))
    judged("a differing baseline key is named", any("world a key summary-1000" in f for f in check_baselines(record, changed)),
           check_baselines(record, changed))
    without_b = parse_baselines("no b", written.split("[world b]")[0])
    judged("a world missing from the baselines", "no baseline for world b" in check_baselines(record, without_b),
           check_baselines(record, without_b))
    extra = parse_baselines("extra", written + written.split("[world b]")[1].join(["[world c]", ""]))
    judged("a baseline for a world that does not exist", any("world c" in f for f in check_baselines(record, extra)),
           check_baselines(record, extra))
    try:
        parse_baselines("bad", written + "summary-300\n")
        judged("an unparseable baseline line is refused with its number", False, "accepted")
    except Unmet as unmet:
        judged("an unparseable baseline line is refused with its number", f"bad:{len(written.split(chr(10)))}:" in str(unmet), unmet)
    reasoned = parse_baselines("reasons", written.replace("[world a]\n", "[world a]\nreason #1: a rule changed\n"))
    judged("a reason line is kept by write and ignored by check",
           "reason #1: a rule changed" in render_baselines(record, reasoned) and check_baselines(record, reasoned) == [],
           render_baselines(record, reasoned))
    try:
        baselines_of(parse("default", render(four()[1])))
        judged("baselines refuse a default record", False, "accepted")
    except Unmet as unmet:
        judged("baselines refuse a default record", "long record" in str(unmet), unmet)

    new = parse("new", render(modified(modified(base[1], "world a", "line-300.0001", "faults 2"),
                                       "world a", "table-300s.facts.chunk.00000", "1 1000 Q")))
    notes = "\n".join(diff_records(record, new))
    judged("diff locates the first differing line and chunk",
           "world a differs" in notes and "summary-300 first differs at line 1" in notes
           and "first differing chunk 0" in notes and "world b" not in notes, notes)
    judged("diff of equal records is empty", diff_records(record, record) == [], diff_records(record, record))
    return failed


# ----------------------------------------------------------------------------------------------- main

USAGE = ("usage: ci_parity.py record (--binary PATH | --image TAG [--platform P]) [--out FILE]\n"
         "                           [--profile default|long] [--timings FILE]\n"
         "       ci_parity.py compare RECORD RECORD [RECORD ...]\n"
         "       ci_parity.py baseline check|write RECORD [BASELINES]   (default scripts/baselines.txt)\n"
         "       ci_parity.py diff OLD NEW\n"
         "       ci_parity.py --self-test")
RECORD_OPTIONS = {"--binary", "--image", "--platform", "--out", "--profile", "--timings"}


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if len(arguments) >= 3 and arguments[0] == "compare":
            return compare(arguments[1:])
        if len(arguments) in (3, 4) and arguments[0] == "baseline" and arguments[1] in ("check", "write"):
            return baseline(arguments[1], arguments[2], Path(arguments[3]) if len(arguments) == 4 else BASELINES)
        if len(arguments) == 3 and arguments[0] == "diff":
            return diff(arguments[1], arguments[2])
        if arguments and arguments[0] == "record":
            options = dict(zip(arguments[1::2], arguments[2::2]))
            if len(arguments[1:]) % 2 or set(options) - RECORD_OPTIONS \
                    or ("--binary" in options) == ("--image" in options) \
                    or ("--platform" in options and "--image" not in options) \
                    or options.get("--profile", "default") not in PROFILES:
                print(USAGE, file=sys.stderr)
                return 2
            runner = Native(options["--binary"]) if "--binary" in options else Image(options["--image"], options.get("--platform"))
            out = Path(options["--out"]) if "--out" in options else None
            timings = Path(options["--timings"]) if "--timings" in options else None
            return record(runner, out, options.get("--profile", "default"), timings)
    except Unmet as unmet:
        print(f"[parity] FAIL {unmet}", file=sys.stderr)
        return 1
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
