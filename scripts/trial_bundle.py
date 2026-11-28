#!/usr/bin/env python3
"""Build a local macOS trial bundle of one commit, for an operator's hands-on trial.

This is a thin recipe around `scripts/package.py` (S23 R-a, ARC-80), which stays the one place that knows the
bundle's layout, contents and budget. It adds only what a local trial needs and the release path does not:

1. a clean, detached checkout of exactly one commit (default `origin/main`), with its Git LFS files;
2. `package.py assemble --no-archive` in that checkout (server, launchers, shared runtime, 2D and 3D packs);
3. optionally `package.py smoke` on the result (the launcher's 2D, 3D and 2D + 3D entry points, headless);
4. `START-HERE.txt`: the operator's start commands, every mode on one trial save folder (so a world played in
   2D continues in 3D), kept apart from the player's normal saves;
5. `manifest.json`: the source commit, the tool versions, every command with its duration and exit code, and
   the size and SHA-256 of every bundle file.

    python3 scripts/trial_bundle.py --root DIR [--rev REV] [--godot BIN] [--godot-notices DIR]
                                    [--smoke] [--keep-source] [--manifest-only]

Everything goes to `DIR/<YYYYMMDD>-<sha8>/`:

    source/          the detached checkout (removed at the end unless --keep-source)
    bundle/          MineWorld-<version>-macos-universal/ with its three .app launchers
    trial-data/      the one save folder START-HERE.txt points every mode at (made on first start)
    START-HERE.txt   manifest.json   logs/

`--manifest-only` removes the bundle after the manifest is written (a dry run that records time and size).
Cargo's output goes to `source/target/` unless `CARGO_TARGET_DIR` is set. macOS only: the bundle is
`macos-universal`, and its launchers are `.app` folders. Standard library only. Exits non-zero naming what failed.
"""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TARGET = "macos-universal"
MODES = (("3D", "MineWorld 3D", "3d"), ("2D", "MineWorld 2D", "2d"), ("2D + 3D", "MineWorld 2D + 3D", "both"))


class Failure(Exception):
    """A step that cannot continue; the message names what failed."""


class Recorder:
    """Runs commands, logs each one's output to `logs/`, and records what ran for the manifest."""

    def __init__(self, logs: Path) -> None:
        self.logs = logs
        self.steps: list[dict[str, object]] = []

    def run(self, name: str, command: list[str], cwd: Path, env: dict[str, str] | None = None,
            check: bool = True) -> int:
        self.logs.mkdir(parents=True, exist_ok=True)
        log = self.logs / f"{len(self.steps):02d}-{name}.log"
        print(f"[trial] {name}: {' '.join(command)}", flush=True)
        start = time.monotonic()
        with log.open("w", encoding="utf-8") as out:
            code = subprocess.run(command, cwd=cwd, env=env, stdout=out, stderr=subprocess.STDOUT,
                                  check=False).returncode
        seconds = round(time.monotonic() - start, 1)
        self.steps.append({"name": name, "command": command, "seconds": seconds, "exit": code,
                           "log": str(log)})
        print(f"[trial] {name}: exit {code} after {seconds} s (log {log})", flush=True)
        if check and code != 0:
            raise Failure(f"{name} exited with {code}; see {log}")
        return code


def output(command: list[str], cwd: Path = ROOT) -> str:
    return subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def tool_versions(godot: str) -> dict[str, str]:
    def version(command: list[str]) -> str:
        try:
            return output(command)
        except (OSError, subprocess.CalledProcessError) as error:
            return f"unavailable: {error}"
    return {"godot": version([godot, "--version"]), "rustc": version(["rustc", "--version"]),
            "cargo": version(["cargo", "--version"]), "git-lfs": version(["git", "lfs", "version"]),
            "python": platform.python_version(), "macos": platform.mac_ver()[0], "machine": platform.machine()}


def checkout(sha: str, source: Path, record: Recorder) -> None:
    """A detached worktree of `sha` with its LFS files; refuses a pointer left in place."""
    if source.exists():
        raise Failure(f"{source} already exists; choose another --root or remove it")
    # LFS files come in one batch from `git lfs pull`, not one smudge per file.
    record.run("checkout", ["git", "worktree", "add", "--detach", str(source), sha], cwd=ROOT,
               env={**os.environ, "GIT_LFS_SKIP_SMUDGE": "1"})
    record.run("lfs-pull", ["git", "lfs", "pull"], cwd=source)
    pointers = output(["git", "lfs", "ls-files", "--name-only"], cwd=source).splitlines()
    left = [p for p in pointers if (source / p).is_file()
            and (source / p).read_bytes()[:40].startswith(b"version https://git-lfs")]
    if left:
        raise Failure(f"{len(left)} LFS file(s) are still pointers, such as {left[0]}")


def files_of(bundle: Path) -> list[dict[str, object]]:
    listed = []
    for path in sorted(p for p in bundle.rglob("*") if p.is_file() and not p.is_symlink()):
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            for block in iter(lambda: handle.read(1 << 20), b""):
                digest.update(block)
        listed.append({"path": path.relative_to(bundle).as_posix(), "bytes": path.stat().st_size,
                       "sha256": digest.hexdigest()})
    return listed


def start_here(out: Path, bundle: Path, sha: str, subject: str) -> str:
    """The operator's start page, in Chinese (the operator's language; CLAUDE.md §2.1 governs Markdown, and
    this is a plain-text note beside a local bundle, never a specification)."""
    data = out / "trial-data"
    lines = [f"MineWorld 本地试玩包（macOS）— {sha[:8]} {subject}", "",
             "每一行粘贴到终端就能启动。三种模式共用一个试玩存档（trial-data），和你平时的 MineWorld 存档分开，",
             "所以 2D 里做过的事，换到 3D 还在。", ""]
    for label, app, _key in MODES:
        lines += [f"{label}：", f"open -n '{bundle / (app + '.app')}' --args --user-dir='{data}'", ""]
    lines += ["关掉这个模式的所有窗口，本地服务器会保存并停止。再运行任意一行，就接着上次的存档玩。",
              "换模式前先关掉当前模式。要从头开始，删掉 trial-data 文件夹。", "",
              "3D：WASD 移动，鼠标看方向，Shift 慢跑，Space 跳，V 切换视角，Esc 打开菜单（退出也在这里）。",
              "2D：点地面走过去，点人物打开菜单，点座位走过去坐下；点自己可以站起来。", "",
              "人物由简单规则控制，不需要模型，也不需要网络或付费接口。",
              "这是本地试玩包，不是正式发布。精确版本、每一步命令和文件校验见 manifest.json。", ""]
    return "\n".join(lines)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--root", required=True, help="the folder that holds every trial, one folder each")
    parser.add_argument("--rev", default="origin/main", help="the commit to build (default origin/main)")
    parser.add_argument("--godot", default=os.environ.get("GODOT", "godot"))
    parser.add_argument("--godot-notices", help="a folder with Godot's LICENSE.txt and COPYRIGHT.txt "
                        "(default: package.py fetches them with gh)")
    parser.add_argument("--smoke", action="store_true", help="run package.py smoke on the bundle")
    parser.add_argument("--keep-source", action="store_true", help="keep the checkout after the build")
    parser.add_argument("--manifest-only", action="store_true",
                        help="remove the bundle once the manifest is written (a timed dry run)")
    args = parser.parse_args(argv)
    if platform.system() != "Darwin":
        print("trial_bundle.py: the trial bundle is macos-universal; run it on macOS", file=sys.stderr)
        return 1
    sha = output(["git", "rev-parse", "--verify", f"{args.rev}^{{commit}}"])
    subject = output(["git", "log", "-1", "--format=%s", sha])
    out = Path(args.root).resolve() / f"{datetime.date.today():%Y%m%d}-{sha[:8]}"
    source = out / "source"
    record = Recorder(out / "logs")
    manifest: dict[str, object] = {
        "rev": args.rev, "source_sha": sha, "subject": subject, "target": TARGET,
        "started": datetime.datetime.now().astimezone().isoformat(timespec="seconds"),
        "tools": tool_versions(args.godot), "steps": record.steps}
    started = time.monotonic()
    verdict = "FAIL"
    try:
        out.mkdir(parents=True, exist_ok=True)
        checkout(sha, source, record)
        assemble = [sys.executable, "scripts/package.py", "assemble", "--target", TARGET, "--clients", "2d,3d",
                    "--godot", args.godot, "--no-archive"]
        if args.godot_notices:
            assemble += ["--godot-notices", str(Path(args.godot_notices).resolve())]
        record.run("assemble", assemble, cwd=source)
        built = sorted((source / "target" / "package" / TARGET).glob(f"MineWorld-*-{TARGET}"))
        if len(built) != 1:
            raise Failure(f"expected one assembled bundle, found {[p.name for p in built]}")
        bundle = out / "bundle" / built[0].name
        bundle.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(str(built[0]), bundle)
        if args.smoke:
            record.run("smoke", [sys.executable, "scripts/package.py", "smoke", str(bundle),
                                 "--modes", "2d,3d,both"], cwd=source)
        files = files_of(bundle)
        manifest.update(bundle=str(bundle), bundle_bytes=sum(int(f["bytes"]) for f in files),
                        bundle_files=len(files), files=files)
        (out / "START-HERE.txt").write_text(start_here(out, bundle, sha, subject), encoding="utf-8")
        verdict = "PASS"
    except (Failure, OSError, subprocess.CalledProcessError) as error:
        manifest["failure"] = str(error)
        print(f"trial_bundle.py: {error}", file=sys.stderr)
    finally:
        manifest["verdict"] = verdict
        manifest["seconds"] = round(time.monotonic() - started, 1)
        if source.exists() and not args.keep_source:
            record.run("remove-source", ["git", "worktree", "remove", "--force", str(source)], cwd=ROOT,
                       check=False)
        if args.manifest_only and (out / "bundle").exists():
            shutil.rmtree(out / "bundle")
            manifest["bundle_removed"] = True
        out.mkdir(parents=True, exist_ok=True)
        (out / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n", encoding="utf-8")
        print(f"[trial] {verdict} in {manifest['seconds']} s; manifest {out / 'manifest.json'}")
    return 0 if verdict == "PASS" else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
