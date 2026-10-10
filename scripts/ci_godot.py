#!/usr/bin/env python3
"""Godot in CI: the pinned download, the 3D slice probes' parsed verdicts, and the coverage check.

`docs/DECISIONS.md` DEP-45 and ARC-83; `pr-13c-nightly.md` §3.4.

    python3 scripts/ci_godot.py fetch --dest DIR [--github-env FILE --github-path FILE]
        The official Godot build for this OS (VERSION below), downloaded with the standard library and
        refused unless its SHA-512 equals the value pinned here — never the release's own SHA512-SUMS.txt,
        which would only detect corruption, not a replaced asset. Unpacked into DIR (kept when DIR already
        holds this exact pin, the cache's case), `--version` checked, then `GODOT` and a `godot` command on
        PATH exported for the steps after it. Windows uses the archive's `_console.exe`.
    python3 scripts/ci_godot.py slice [--limit SECONDS] MODE-FLAGS ...
        `./mineworld-slice MODE-FLAGS` with its output streamed and its verdict parsed, because the probe
        exits 0 whether its checks pass or fail (F-13c-5): PASS needs exactly one `all <mode> checks
        pass` line and no line beginning `FAIL`; a `<n> <MODE> CHECKS FAILED` line or a `FAIL` line is
        FAIL; neither summary (an early return, a crash, a hang killed at the limit) is INCONCLUSIVE.
        Exits 0, 1 or 3, and writes the decisive lines to artifacts/nightly/probe-<mode>.txt.
    python3 scripts/ci_godot.py tests
        The ignored Godot tests, one at a time (`--test-threads=1`: each starts Godot and a server), with
        this OS's named skips (SKIPS below), through `ci_repeat.py` once so that a failing test is named in
        artifacts/nightly/clients-tests.txt; the skips and their reasons go to artifacts/nightly/notes.txt.
    python3 scripts/ci_godot.py coverage
        Lists the ignored Godot tests (`cargo test … -- --ignored --list`) and fails if none is found, if
        a skip below names a test that does not exist, or if a test is skipped on every OS (QC-5).
    python3 scripts/ci_godot.py --self-test

Standard library only.
"""

from __future__ import annotations

import hashlib
import os
import platform
import re
import shutil
import stat
import subprocess
import sys
import threading
import time
import urllib.request
import zipfile
from pathlib import Path

from ci_repeat import render as render_samples
from ci_repeat import repeat

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "artifacts" / "nightly"

# The one pinned Godot (DEP-45; shared with S23's export job, step-23 §16.4). Pinned 2026-10-10 from the
# release's SHA512-SUMS.txt (godotengine/godot-builds, tag 4.7.2-stable) and cross-checked once by
# downloading the macOS archive and hashing it; a Godot upgrade changes this block and nothing else.
VERSION = "4.7.2-stable"
VERSION_PREFIX = "4.7.2.stable.official"
RELEASE = f"https://github.com/godotengine/godot-builds/releases/download/{VERSION}"
ASSETS = {
    "Linux": ("Godot_v4.7.2-stable_linux.x86_64.zip",
              "9aa00f7a605200940bce3027a567b782f49bd8e940dd06ae9e987bd65aee1b14"
              "67edd56ed84fcdcbdd44354bf613bdbb4e5d2913e925850368e150c59ed54c65",
              "Godot_v4.7.2-stable_linux.x86_64"),
    "Darwin": ("Godot_v4.7.2-stable_macos.universal.zip",
               "38aa16e5bba2083941fc5b3e54be0089bd4cc35e32415f5b9fd9a8a6a7b98182"
               "55d44532ea8ef94b5aef56c4b407c2d634fa4f657e4ebe681ebbf59b7bac69ca",
               "Godot.app/Contents/MacOS/Godot"),
    "Windows": ("Godot_v4.7.2-stable_win64.exe.zip",
                "83decd58fdf67b9d657958a1ae6bf1929c20785315a81effe245874cdc57acb7"
                "09bf868e00778a96984338c1b29dafdb453c6847747694621c6ecf5da2259993",
                "Godot_v4.7.2-stable_win64_console.exe"),
}

# The ignored Godot tests (F-13c-4) and the OSes on which one is skipped by name, each with its reason
# (QC-5: only a test that cannot show a window on that leg, and only while it passes on another leg the
# same night). Empty until a leg needs it.
GODOT_TESTS = ["client_2d", "client_2d_interact", "client_2d_interact_stub", "client_settings"]
SKIPS: dict[str, dict[str, str]] = {}
OSES = ("Linux", "Darwin", "Windows")

SLICE_LIMIT = 900.0
MODES = ("drive", "link", "target", "character", "scale", "settings")


class Unmet(Exception):
    """An expectation did not hold; the message names it."""


# ---------------------------------------------------------------------------------------------- fetch


def sha512(path: Path) -> str:
    digest = hashlib.sha512()
    with open(path, "rb") as file:
        for block in iter(lambda: file.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def unpack(archive: Path, into: Path) -> None:
    """zipfile, keeping each member's Unix mode (the executables) and its symbolic links."""
    with zipfile.ZipFile(archive) as bundle:
        for member in bundle.infolist():
            target = into / member.filename
            mode = (member.external_attr >> 16) & 0o177777
            if stat.S_ISLNK(mode):
                target.parent.mkdir(parents=True, exist_ok=True)
                os.symlink(bundle.read(member).decode("utf-8"), target)
                continue
            bundle.extract(member, into)
            if mode & 0o111:
                target.chmod(target.stat().st_mode | 0o111)


def fetch(dest: Path, system: str, env_file: str | None, path_file: str | None) -> int:
    if system not in ASSETS:
        raise Unmet(f"no Godot build is pinned for {system}")
    asset, pinned, executable = ASSETS[system]
    home = dest / VERSION
    marker = home / "PINNED-SHA512"
    if marker.is_file() and marker.read_text(encoding="utf-8").strip() == pinned and (home / executable).is_file():
        print(f"[godot] {asset}: already unpacked with the pinned SHA-512 (cache)", flush=True)
    else:
        shutil.rmtree(home, ignore_errors=True)
        home.mkdir(parents=True)
        archive = dest / asset
        began = time.monotonic()
        print(f"[godot] downloading {RELEASE}/{asset}", flush=True)
        try:
            with urllib.request.urlopen(f"{RELEASE}/{asset}", timeout=120) as answer, open(archive, "wb") as out:
                shutil.copyfileobj(answer, out, 1 << 20)
        except OSError as error:
            shutil.rmtree(home, ignore_errors=True)
            raise Unmet(f"download of {asset} failed: {error}") from error
        found = sha512(archive)
        if found != pinned:
            archive.unlink(missing_ok=True)
            shutil.rmtree(home, ignore_errors=True)
            raise Unmet(f"SHA-512 mismatch for {asset}: pinned {pinned}, downloaded {found}; nothing unpacked")
        unpack(archive, home)
        archive.unlink()
        marker.write_text(pinned + "\n", encoding="utf-8")
        print(f"[godot] {asset}: SHA-512 matches the pin; unpacked in {time.monotonic() - began:.1f} s", flush=True)
    binary = home / executable
    version = subprocess.run([str(binary), "--version"], capture_output=True, text=True, timeout=120)
    reported = (version.stdout or version.stderr).strip().splitlines()
    if version.returncode != 0 or not reported or not reported[-1].startswith(VERSION_PREFIX):
        raise Unmet(f"{binary} --version: exit {version.returncode}, {reported!r}; expected {VERSION_PREFIX}…")
    print(f"[godot] {binary}: {reported[-1]}", flush=True)
    # A `godot` command for the bash launchers (mineworld-slice, clients/protocol/run.sh).
    commands = home / "bin"
    commands.mkdir(exist_ok=True)
    command = commands / ("godot.exe" if system == "Windows" else "godot")
    if not command.exists():
        if system == "Windows":
            shutil.copy2(binary, command)
        else:
            command.symlink_to(binary)
    if env_file:
        with open(env_file, "a", encoding="utf-8") as out:
            out.write(f"GODOT={binary}\n")
    if path_file:
        with open(path_file, "a", encoding="utf-8") as out:
            out.write(f"{commands}\n")
    return 0


# ---------------------------------------------------------------------------------------------- slice


def judge_probe(lines: list[str]) -> tuple[str, list[str]]:
    """The probe's verdict from what it printed. The exit status never decides PASS."""
    passes = [line.strip() for line in lines if re.fullmatch(r"all [a-z]+ checks pass", line.strip())]
    failed = [line.strip() for line in lines if re.fullmatch(r"\d+ [A-Z]+ CHECKS FAILED", line.strip())]
    bare = [line.strip() for line in lines if line.strip().startswith("FAIL")]
    if failed or bare:
        return "FAIL", failed + bare[:20]
    if len(passes) == 1:
        return "PASS", passes
    if len(passes) > 1:
        return "INCONCLUSIVE", [f"{len(passes)} summary lines, expected one: {passes}"]
    return "INCONCLUSIVE", ["no summary line (an early return, a crash, or killed at the limit)"]


def slice_probe(flags: list[str], limit: float) -> int:
    modes = [flag[2:] for flag in flags if flag[2:] in MODES]
    name = "-".join(modes) or "probe"
    command = ["bash", str(ROOT / "mineworld-slice"), *flags]
    print(f"[godot] $ {' '.join(command)} (limit {limit:.0f} s)", flush=True)
    lines: list[str] = []
    began = time.monotonic()
    process = subprocess.Popen(command, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               start_new_session=os.name != "nt")

    def read() -> None:
        assert process.stdout is not None
        for raw in process.stdout:
            text = raw.decode("utf-8", errors="replace").rstrip("\r\n")
            lines.append(text)
            print(text, flush=True)

    reader = threading.Thread(target=read, daemon=True)
    reader.start()
    try:
        status: int | None = process.wait(timeout=limit)
    except subprocess.TimeoutExpired:
        status = None
        kill_group(process)
    reader.join(timeout=10)
    verdict, decisive = judge_probe(lines)
    if status is None:
        decisive.append(f"killed at the {limit:.0f} s limit")
        verdict = "INCONCLUSIVE" if verdict == "PASS" else verdict
    detail = [f"probe {name}: {verdict} (exit {status}, {time.monotonic() - began:.0f} s)", *decisive]
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / f"probe-{name}.txt").write_text("\n".join([f"verdict {verdict}", *detail]) + "\n", encoding="utf-8")
    for line in detail:
        print(f"[godot] {line}", flush=True)
    return {"PASS": 0, "FAIL": 1}.get(verdict, 3)


def kill_group(process: subprocess.Popen[bytes]) -> None:
    """The launcher and everything it started (Godot, a server): one process group, killed together."""
    try:
        if os.name != "nt":
            os.killpg(process.pid, 9)
        else:
            process.kill()
    except OSError:
        pass
    process.wait(timeout=60)


# ------------------------------------------------------------------------------------------- coverage


def godot_tests_command(*libtest: str) -> list[str]:
    command = ["cargo", "test", "-p", "mineworld-cli"]
    for test in GODOT_TESTS:
        command += ["--test", test]
    return [*command, "--", "--ignored", *libtest]


def run_tests(system: str) -> int:
    skips = SKIPS.get(system, {})
    command = godot_tests_command("--test-threads=1", *[part for name in sorted(skips) for part in ("--skip", name)])
    samples = repeat(1, command)
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "clients-tests.txt").write_text(render_samples(samples, command), encoding="utf-8", newline="\n")
    if skips:
        with open(OUT / "notes.txt", "a", encoding="utf-8", newline="\n") as out:
            out.writelines(f"skipped on {system} (QC-5): {name} — {reason}\n" for name, reason in sorted(skips.items()))
    failed = [sample for sample in samples if sample.status != 0 or sample.scratch != "clean"]
    print(f"[godot] {len(skips)} test(s) skipped on {system} by name (QC-5); "
          f"{'FAILED' if failed else 'passed'}", flush=True)
    return 1 if failed else 0


def listed_tests() -> list[str]:
    command = godot_tests_command("--list")
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True)
    if result.returncode != 0:
        raise Unmet(f"{' '.join(command)} exited {result.returncode}:\n{result.stderr[-2000:]}")
    return sorted(line[: -len(": test")] for line in result.stdout.splitlines() if line.endswith(": test"))


def judge_coverage(tests: list[str], skips: dict[str, dict[str, str]]) -> list[str]:
    failures = [] if tests else ["no ignored Godot test was listed"]
    for os_name, skipped in skips.items():
        failures += [f"{os_name} skips {name}, which is not an ignored Godot test" for name in skipped if name not in tests]
    failures += [f"{name} is skipped on every OS, so it runs on no leg"
                 for name in tests if all(name in skips.get(os_name, {}) for os_name in OSES)]
    return failures


def coverage() -> int:
    tests = listed_tests()
    failures = judge_coverage(tests, SKIPS)
    skipped = {os_name: sorted(names) for os_name, names in SKIPS.items() if names}
    print(f"[godot] coverage: {len(tests)} ignored Godot tests ({', '.join(GODOT_TESTS)}); skips {skipped or 'none'}")
    for failure in failures:
        print(f"[godot] FAIL {failure}", file=sys.stderr)
    return 0 if not failures else 1


# ------------------------------------------------------------------------------------------ self-test


def self_test() -> int:
    passing = ["[slice] drive: walking in", "  ok  the door opens", "", "all drive checks pass"]
    failing = ["  FAIL the door opens (expected Cafe.Door)", "", "1 DRIVE CHECKS FAILED"]
    aborted = ["FAIL: the world never connected"]
    truncated = ["[slice] drive: walking in", "  ok  the door opens"]
    tests = ["client_2d::a", "client_2d::b"]
    cases = [
        ("a summary pass and no FAIL line is PASS", judge_probe(passing)[0] == "PASS"),
        ("a CHECKS FAILED summary is FAIL, with the line", judge_probe(failing) == ("FAIL", ["1 DRIVE CHECKS FAILED",
                                                                                         "FAIL the door opens (expected Cafe.Door)"])),
        ("a bare FAIL with no summary (an early return) is FAIL", judge_probe(aborted)[0] == "FAIL"),
        ("no summary line is INCONCLUSIVE, whatever the exit status", judge_probe(truncated)[0] == "INCONCLUSIVE"),
        ("an empty transcript is INCONCLUSIVE", judge_probe([])[0] == "INCONCLUSIVE"),
        ("a pass line beside a FAIL line is FAIL", judge_probe(passing + ["FAIL late"])[0] == "FAIL"),
        ("two summaries are INCONCLUSIVE", judge_probe(passing + passing)[0] == "INCONCLUSIVE"),
        ("coverage with no skips passes", judge_coverage(tests, {}) == []),
        ("a test skipped on every OS fails coverage",
         judge_coverage(tests, {o: {"client_2d::a": "no window"} for o in OSES}) == ["client_2d::a is skipped on every OS, so it runs on no leg"]),
        ("a skip of an unknown test fails coverage", bool(judge_coverage(tests, {"Linux": {"client_2d::z": "r"}}))),
        ("no listed test fails coverage", judge_coverage([], {}) == ["no ignored Godot test was listed"]),
        ("every OS has a pinned build", set(ASSETS) == set(OSES) and all(len(pin) == 128 for _, pin, _ in ASSETS.values())),
    ]
    failed = 0
    for name, good in cases:
        failed += not good
        print(f"[self-test] {'ok  ' if good else 'FAIL'} {name}")
    print(f"[self-test] {'passed' if not failed else f'FAILED: {failed} case(s)'}")
    return 0 if not failed else 1


USAGE = ("usage: ci_godot.py fetch --dest DIR [--github-env FILE] [--github-path FILE]\n"
         "       ci_godot.py slice [--limit SECONDS] FLAGS ...\n"
         "       ci_godot.py tests | coverage | --self-test")


def main(arguments: list[str]) -> int:
    try:
        if arguments == ["--self-test"]:
            return self_test()
        if arguments == ["coverage"]:
            return coverage()
        if arguments == ["tests"]:
            return run_tests(platform.system())
        if arguments[:1] == ["slice"] and len(arguments) > 1:
            flags, limit = arguments[1:], SLICE_LIMIT
            if flags[0] == "--limit" and len(flags) > 2:
                limit, flags = float(flags[1]), flags[2:]
            return slice_probe(flags, limit)
        if arguments[:1] == ["fetch"]:
            options = dict(zip(arguments[1::2], arguments[2::2]))
            if len(arguments) % 2 == 1 and "--dest" in options and set(options) <= {"--dest", "--github-env", "--github-path"}:
                return fetch(Path(options["--dest"]), platform.system(), options.get("--github-env"),
                             options.get("--github-path"))
    except Unmet as unmet:
        print(f"[godot] FAIL {unmet}", file=sys.stderr)
        return 1
    print(USAGE, file=sys.stderr)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
