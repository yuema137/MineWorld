#!/usr/bin/env python3
"""AC-8's instrument: record what a platform makes of every world, and compare platforms' records.

`AC-8` (`docs/MVP.md` §9): the same World Pack runs on a laptop and inside a cloud Docker container
with no semantic differences; the operator requires macOS, Linux and Windows alike. How that is measured
is `docs/DECISIONS.md` ARC-49 (design: step-14 §13.4). This script is the whole instrument, so that the
laptop, the macOS and Windows runners and the Linux container all hash with one implementation:

    python3 scripts/ci_parity.py record --binary target/release/mineworld [--out F]
    python3 scripts/ci_parity.py record --image mineworld:ci [--platform linux/amd64] [--out F]
    python3 scripts/ci_parity.py compare F F [F ...]
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
PLATFORM_KEYS = ("arch", "container", "emulated", "os", "rustc", "target", "translated")
SOURCE_KEYS = ("commit", "record-format", "worlds")
WORLD_KEYS = ("manifest", "summary-300", "summary-30s", "validate")
REQUIRED_TABLES = ("facts", "journal", "snapshots")


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


def table_digests(connection: sqlite3.Connection, table: str) -> dict[str, str]:
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
            entries[f"table.{table}.chunk.{count // CHUNK_ROWS - 1:05d}"] = f"{first} {last} {chunk.hexdigest()}"
            chunk, first = hashlib.sha256(), None
    if first is not None:
        entries[f"table.{table}.chunk.{count // CHUNK_ROWS:05d}"] = f"{first} {last} {chunk.hexdigest()}"
    entries[f"table.{table}.rows"] = f"{count} {whole.hexdigest()}"
    return entries


def save_digests(file: Path) -> dict[str, str]:
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
        entries["manifest"] = f"{form} {sha256(encode_value(form) + encode_value(canonical))}"
        for table in sorted(tables):
            if table != "manifest":
                entries.update(table_digests(connection, table))
        return entries
    except sqlite3.Error as error:
        raise Unmet(f"{file}: {error}") from error
    finally:
        connection.close()


def record_world(runner: Native | Image, world: str, scratch: Path) -> dict[str, str]:
    path = f"worlds/{world}"
    began = time.monotonic()
    entries = {"validate": sha256(runner.run(["validate", path], f"validate {path}"))}

    long_args = ["run", path, "--headless", "--seed", str(SEED), "--days", str(LONG_DAYS)]
    long = summary(text_lines(runner.run(long_args, f"{world}: run {LONG_DAYS} days in memory"), world), False)
    entries["summary-300"] = digest_lines(long)
    entries.update({f"line-300.{index:04d}": line for index, line in enumerate(long)})

    save = scratch / world
    save_args = ["run", path, "--headless", "--seed", str(SEED), "--days", str(SAVE_DAYS)]
    saved = summary(text_lines(runner.run(save_args, f"{world}: run {SAVE_DAYS} days saved", save), world), True)
    entries["summary-30s"] = digest_lines(saved)
    entries.update({f"line-30s.{index:04d}": line for index, line in enumerate(saved)})
    entries.update(save_digests(save / "world.sqlite"))
    remove(save)
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


def record(runner: Native | Image, out: Path | None) -> int:
    started = time.monotonic()
    facts = runner.platform()
    worlds = enumerate_worlds(ROOT)
    commit = command(["git", "rev-parse", "HEAD"], "git rev-parse HEAD").decode().strip()
    sections: dict[str, dict[str, str]] = {
        "platform": facts,
        "source": {"commit": commit, "record-format": RECORD_FORMAT, "worlds": " ".join(worlds)},
    }
    print(f"[parity] recording {commit} on {facts['os']}/{facts['arch']} (container {facts['container']})", flush=True)
    scratch = Path(tempfile.mkdtemp(prefix="ac8-parity-"))
    try:
        for world in worlds:
            sections[f"world {world}"] = record_world(runner, world, scratch)
    finally:
        if scratch.exists():
            shutil.rmtree(scratch, ignore_errors=True)
    if scratch.exists():
        raise Unmet(f"could not remove the scratch directory {scratch}")
    target = out or Path(f"ac8-{facts['os'].lower()}-{facts['arch']}.txt")
    target.write_bytes(render(sections))
    print(f"[parity] wrote {target} ({len(worlds)} worlds) in {time.monotonic() - started:.1f} s")
    return 0


# -------------------------------------------------------------------------------------------- compare


@dataclass
class Record:
    name: str
    sections: dict[str, dict[str, str]] = field(default_factory=dict)

    def label(self) -> str:
        facts = self.sections.get("platform", {})
        return f"{facts.get('os', '?')}/{facts.get('arch', '?')}"


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
    for world in parsed.sections["source"]["worlds"].split():
        entries = parsed.sections.get(f"world {world}")
        if entries is None:
            raise Unmet(f"G-1 {name}: no [world {world}] section")
        missing = [key for key in WORLD_KEYS if key not in entries]
        missing += [f"table.{table}.rows" for table in REQUIRED_TABLES if f"table.{table}.rows" not in entries]
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
    for prefix, summary_key in (("line-300.", "summary-300"), ("line-30s.", "summary-30s")):
        for key in keys:
            if key.startswith(prefix) and len({entry.get(key) for entry in entries}) > 1:
                shown = "; ".join(f"{label}: {entry.get(key, '(none)')!r}" for label, entry in pairs)
                notes.append(f"  {summary_key} first differs at line {int(key[len(prefix):])}: {shown}")
                break
        else:
            if len({entry.get(summary_key) for entry in entries}) > 1:
                notes.append(f"  {summary_key} differs in its line count: "
                             + "; ".join(f"{label}: {entry.get(summary_key)}" for label, entry in pairs))
    tables = sorted({key.split(".")[1] for key in keys if key.startswith("table.")})
    for table in tables:
        rows = f"table.{table}.rows"
        chunk = next((key for key in keys if key.startswith(f"table.{table}.chunk.")
                      and len({entry.get(key) for entry in entries}) > 1), None)
        if chunk is None and len({entry.get(rows) for entry in entries}) == 1:
            continue
        counts = ", ".join(f"{label} {(entry.get(rows) or '?').split()[0]}" for label, entry in pairs)
        where = "no chunk differs (only the table digest)"
        if chunk is not None:
            ranges = "; ".join(f"{label}: keys {' … '.join((entry.get(chunk) or '? ?').split()[:2])}"
                               for label, entry in pairs)
            where = f"first differing chunk {int(chunk.rsplit('.', 1)[1])} ({ranges})"
        notes.append(f"  table {table}: rows {counts}; {where}")
    for key in ("validate", "manifest"):
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
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


# ----------------------------------------------------------------------------------------------- main

USAGE = ("usage: ci_parity.py record (--binary PATH | --image TAG [--platform P]) [--out FILE]\n"
         "       ci_parity.py compare RECORD RECORD [RECORD ...]\n"
         "       ci_parity.py --self-test")


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if len(arguments) >= 3 and arguments[0] == "compare":
            return compare(arguments[1:])
        if arguments and arguments[0] == "record":
            options = dict(zip(arguments[1::2], arguments[2::2]))
            if len(arguments[1:]) % 2 or set(options) - {"--binary", "--image", "--platform", "--out"} \
                    or ("--binary" in options) == ("--image" in options) \
                    or ("--platform" in options and "--image" not in options):
                print(USAGE, file=sys.stderr)
                return 2
            runner = Native(options["--binary"]) if "--binary" in options else Image(options["--image"], options.get("--platform"))
            out = Path(options["--out"]) if "--out" in options else None
            return record(runner, out)
    except Unmet as unmet:
        print(f"[parity] FAIL {unmet}", file=sys.stderr)
        return 1
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
