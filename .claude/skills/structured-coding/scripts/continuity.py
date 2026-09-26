#!/usr/bin/env python3
"""Optional continuity hooks and explicit per-session checkpoint commands.

See references/continuity.md for scope, lifecycle, limits, and host setup.
Uses only Python 3.9+ and Git; never invokes an LLM, a job, or a remote API.
"""

import argparse
import hashlib
import json
import os
import shlex
import stat
import subprocess
import sys
import tempfile
import time
import uuid
from pathlib import Path

HOSTS = ("codex", "claude-code")
# 2 adds an optional base revision. A schema-1 record is never rewritten, so
# anything that hashed it stays valid.
BINDING_SCHEMA = 2
BINDING_SCHEMAS = (1, 2)
MAX_INPUT = 1024 * 1024
MAX_FILES = 10000
MAX_BYTES = 256 * 1024 * 1024
SECONDS = 8


def digest(value):
    return hashlib.sha256(value).hexdigest()


def encoded(value):
    return (
        json.dumps(value, ensure_ascii=True, sort_keys=True, indent=2) + "\n"
    ).encode()


def safe_path(root, relative):
    """Reject symlink components, even ones that resolve back inside the root."""
    relative = Path(relative)
    if relative.is_absolute() or ".." in relative.parts:
        raise ValueError("Expected a project-relative path without '..'")
    path = root
    for part in relative.parts:
        path = path / part
        if path.is_symlink():
            raise ValueError("Symlink metadata/document paths are not supported")
    return path


def atomic_json(path, value):
    if path.is_symlink():
        raise ValueError("Refusing a redirected state file")
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    with tempfile.NamedTemporaryFile(dir=path.parent, delete=False) as stream:
        temporary = Path(stream.name)
        try:
            stream.write(encoded(value))
            stream.flush()
            os.fsync(stream.fileno())
            stream.close()
            os.replace(temporary, path)
        finally:
            temporary.unlink(missing_ok=True)


def read_json(path):
    if path.is_symlink() or not path.is_file() or path.stat().st_size > MAX_INPUT:
        raise ValueError("Missing, redirected, or oversized state file")
    value = json.loads(path.read_bytes())
    if not isinstance(value, dict):
        raise ValueError("Expected a JSON object")
    return value


class Repository:
    def __init__(self, project):
        self.root = Path(project).expanduser().resolve(strict=True)
        self.deadline = time.monotonic() + SECONDS
        self.used = 0
        actual = Path(
            os.fsdecode(self.git("rev-parse", "--show-toplevel")).strip()
        ).resolve()
        if self.root != actual:
            raise ValueError("Use the exact Git worktree root as --project")
        self.gitdir = Path(
            os.fsdecode(self.git("rev-parse", "--absolute-git-dir")).strip()
        ).resolve()
        self.state = safe_path(self.gitdir, "structured-coding-continuity")

    def budget(self):
        remaining = self.deadline - time.monotonic()
        if remaining <= 0 or self.used > MAX_BYTES:
            raise ValueError(
                "Snapshot budget exceeded; split the worktree or compact with recovery"
            )
        return remaining

    def git(self, *args):
        result = subprocess.run(
            ["git", "--no-optional-locks", "-C", str(self.root), *args],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            timeout=min(2, self.budget()),
        )
        if result.returncode:
            raise ValueError(
                "Git state is unavailable (a committed, non-bare worktree is required)"
            )
        return result.stdout

    def identity(self):
        return {
            "worktree": str(self.root),
            "gitdir": str(self.gitdir),
            "branch": os.fsdecode(
                self.git("symbolic-ref", "--quiet", "--short", "HEAD")
            ).strip(),
            "head": self.git("rev-parse", "HEAD").decode().strip(),
        }

    def session_dir(self, host, session):
        if not isinstance(session, str) or not session or len(session) > 256:
            raise ValueError(
                "Missing or invalid session_id; do not guess another session"
            )
        return safe_path(self.state, f"{host}/{digest(session.encode())}")

    def file_hash(self, path):
        before = path.lstat()
        if stat.S_ISLNK(before.st_mode):
            return digest(os.fsencode(os.readlink(path)))
        if not stat.S_ISREG(before.st_mode):
            raise ValueError(
                "Submodules and special files are unsupported by this preset"
            )
        self.used += before.st_size
        self.budget()
        result = hashlib.sha256()
        with path.open("rb") as stream:
            while True:
                self.budget()
                chunk = stream.read(1024 * 1024)
                if not chunk:
                    break
                result.update(chunk)
        after = path.lstat()
        if (before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
            after.st_ino,
            after.st_size,
            after.st_mtime_ns,
            after.st_ctime_ns,
        ):
            raise ValueError(
                "A file changed during snapshot; retry after synchronizing"
            )
        return result.hexdigest()

    def snapshot(self, active):
        identity = self.identity()
        if any(identity[k] != active[k] for k in ("worktree", "gitdir", "branch")):
            raise ValueError(
                "Active PR belongs to a different worktree or branch; rebind explicitly"
            )
        index = self.git("ls-files", "--stage", "-z")
        names = self.git("ls-files", "--cached", "--others", "--exclude-standard", "-z")
        files = sorted(set(names.split(b"\0")) - {b""})
        if len(files) > MAX_FILES:
            raise ValueError("Worktree exceeds the 10000-file snapshot limit")
        tree = hashlib.sha256(index)
        for name in files:
            self.budget()
            relative = Path(os.fsdecode(name))
            if relative.is_absolute() or ".." in relative.parts:
                raise ValueError("Invalid Git file path")
            path = self.root / relative
            # A symlink leaf is hashed as a link. Never traverse a symlink parent.
            safe_path(self.root, relative.parent)
            tree.update(name + b"\0")
            if not os.path.lexists(path):
                tree.update(b"deleted\0")
            else:
                tree.update(str(stat.S_IMODE(path.lstat().st_mode)).encode() + b"\0")
                tree.update(self.file_hash(path).encode() + b"\0")
        documents = {}
        for key in ("design", "contract", "handoff"):
            path = safe_path(self.root, active[key])
            documents[key] = {"path": active[key], "sha256": self.file_hash(path)}
        if (
            identity != self.identity()
            or index != self.git("ls-files", "--stage", "-z")
            or names
            != self.git("ls-files", "--cached", "--others", "--exclude-standard", "-z")
        ):
            raise ValueError(
                "Git state changed during snapshot; retry after synchronizing"
            )
        return {**identity, "fingerprint": tree.hexdigest(), "documents": documents}


def active_record(repository, directory, host, session):
    record = read_json(safe_path(directory, "active.json"))
    required = (
        "worktree",
        "gitdir",
        "branch",
        "pr",
        "design",
        "contract",
        "handoff",
        "binding_id",
    )
    if (
        record.get("schema") not in BINDING_SCHEMAS
        or record.get("host") != host
        or record.get("session_id") != session
    ):
        raise ValueError(
            "Active PR identity is invalid; rebind this session explicitly"
        )
    if any(not isinstance(record.get(key), str) or not record[key] for key in required):
        raise ValueError("Active PR metadata is incomplete")
    if record.get("closed"):
        raise ValueError(
            "This PR binding is closed; do not continue its old next actions"
        )
    if record["worktree"] != str(repository.root) or record["gitdir"] != str(
        repository.gitdir
    ):
        raise ValueError("Active PR belongs to another worktree")
    return record


def entrypoint():
    """The installed entrypoint, absolute, or None when it is absent or unreadable.

    Named rather than replayed: a session-start message cannot deliver the
    routing table, only say where it is.
    """
    path = Path(__file__).resolve().parents[1] / "SKILL.md"
    try:
        return path if path.is_file() else None
    except OSError:
        return None


def context(message):
    return {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": message,
        }
    }


def manual_denial(host, message):
    if host == "codex":
        return {"continue": False, "stopReason": message}
    return {"decision": "block", "reason": message}


def event(project, host, mode, payload):
    """Mode comes from our installed command, not from untrusted stdin."""
    repository = Repository(project)
    expected = "SessionStart" if mode == "session-start" else "PreCompact"
    if not isinstance(payload, dict) or payload.get("hook_event_name") != expected:
        raise ValueError("Unexpected hook event payload")
    session = payload.get("session_id")
    directory = repository.session_dir(host, session)
    active_path = safe_path(directory, "active.json")
    reported = payload.get("cwd")
    if not isinstance(reported, str) or not Path(reported).is_absolute():
        # Never fall back to this process's own directory as the reported cwd.
        if not active_path.exists():
            return {}
        raise ValueError("Hook payload has a missing or relative cwd")
    cwd = Path(reported).resolve(strict=True)
    if not cwd.is_relative_to(repository.root):
        if not active_path.exists():
            return {}
        raise ValueError("Hook cwd is outside the configured project")
    actual = Path(
        os.fsdecode(
            subprocess.check_output(
                ["git", "-C", str(cwd), "rev-parse", "--show-toplevel"],
                stderr=subprocess.DEVNULL,
                timeout=min(2, repository.budget()),
            )
        ).strip()
    ).resolve()
    if actual != repository.root:
        if not active_path.exists():
            return {}
        raise ValueError("Hook cwd belongs to another Git worktree")
    if mode != "session-start" and payload.get("trigger") != mode.removeprefix("pre-"):
        raise ValueError("Compaction trigger does not match the configured handler")
    if mode == "session-start" and payload.get("source") not in (
        "startup",
        "resume",
        "compact",
        "clear",
    ):
        raise ValueError("Unknown session source")
    helper = shlex.join([sys.executable, str(Path(__file__).resolve())])
    invocation = f"{helper} activate --host {host} --project {shlex.quote(str(repository.root))} --session {shlex.quote(session)}"
    if not active_path.exists():
        if mode == "session-start":
            routing = entrypoint()
            return context(
                "Structured Coding continuity is installed, but no PR is bound to this session. "
                "Ordinary work is unaffected. "
                + (
                    f"If this session does Structured Coding work of any kind, read {routing} first and then "
                    "the complete resources its table lists for the current phase; that table is the required "
                    "list, and this message does not replay it. "
                    if routing
                    else ""
                )
                + "If executing a Structured Coding PR, read the installed "
                "references/continuity.md and explicitly bind its current design, contract, and handoff. "
                f"Start with: {invocation} --pr PR_ID --design PATH --contract PATH --handoff PATH. "
                "Do not infer authorization or reuse another session's PR."
            )
        return {}
    if read_json(active_path).get("closed") is True:
        return (
            context(
                "Structured Coding: this session's PR binding is CLOSED. Do not execute its old next actions. Bind a new PR only in its own implementation session."
            )
            if mode == "session-start"
            else {}
        )
    active = active_record(repository, directory, host, session)
    receipt_path = safe_path(directory, "checkpoint.json")
    if mode == "pre-manual":
        current = repository.snapshot(active)
        receipt = read_json(receipt_path) if receipt_path.exists() else {}
        if receipt != {"binding": digest(encoded(active)), "snapshot": current}:
            raise ValueError(
                "Handoff checkpoint is stale or absent. Synchronize the PR design and handoff with "
                "actual code/jobs, then run continuity.py checkpoint with this host/project/session and retry. "
                "A checkpoint is not test evidence or approval."
            )
        return {}
    if mode == "pre-auto":
        rescue = {
            "schema": 1,
            "recovery_required": True,
            "pr": active["pr"],
            "binding": digest(encoded(active)),
        }
        try:
            rescue["snapshot"] = repository.snapshot(active)
        except (OSError, ValueError, subprocess.SubprocessError) as error:
            rescue["warning"] = (
                f"Snapshot unavailable ({type(error).__name__}); recover repository truth."
            )
            print(rescue["warning"], file=sys.stderr)
        atomic_json(safe_path(directory, "rescue.json"), rescue)
        return {}
    # Recovery is procedural in v1: this injection is not a mutation guard.
    identity = repository.identity()
    mismatch = identity["branch"] != active["branch"]
    warning = (
        "Branch mismatch: rebind explicitly before implementation. " if mismatch else ""
    )
    rescue_path = safe_path(directory, "rescue.json")
    if rescue_path.exists():
        rescue = read_json(rescue_path)
        warning += (
            str(rescue.get("warning", "An automatic compact snapshot is available."))
            + " "
        )
    documents = [
        str(safe_path(repository.root, active[key]))
        for key in ("design", "contract", "handoff")
    ]
    skill = Path(__file__).resolve().parents[1]
    rules = [
        str(skill / "prompts" / name)
        for name in ("implementation-working-rules.md", "test-ci-gate-rules.md")
    ]
    routing = entrypoint()
    # First in the whole list, because a recovering session is exactly the case
    # that reads the documents and prompts and skips the routing rules neither
    # of them contains.
    lead = [str(routing)] if routing else []
    return context(
        f"Structured Coding: RECOVERY REQUIRED for PR {active['pr']}. {warning}"
        f"Worktree: {repository.root}; bound branch: {active['branch']}; current HEAD: {identity['head']}. "
        "Read these files IN FULL before further implementation (this message does not replay their contents): "
        + json.dumps(lead + documents + rules, ensure_ascii=True)
        + ". The entrypoint, when present, is the first of them: re-read it and its current phase row, then "
        "whatever that row requires and this session has not read. "
        "Reconcile actual Git state, handoff checkpoint/next actions, and known jobs/logs; "
        "reuse existing jobs. Confirm the PR is still active; remote closure/merge is not checked by this hook. "
        "Do not invent decisions, test results, or approval. Follow the filled contract and stopping conditions. "
        "This preset does not enforce full reads or prevent mutations/merges. After reconciliation and "
        "semantic handoff updates, refresh the mechanical checkpoint."
    )


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action", choices=("activate", "checkpoint", "deactivate", "event")
    )
    parser.add_argument("--host", required=True, choices=HOSTS)
    parser.add_argument("--project", required=True, type=Path)
    parser.add_argument("--session")
    parser.add_argument("--pr")
    parser.add_argument("--base", help="revision a changed-file check compares against")
    for name in ("design", "contract", "handoff"):
        parser.add_argument(f"--{name}")
    parser.add_argument("--event", choices=("pre-manual", "pre-auto", "session-start"))
    args = parser.parse_args(argv)
    try:
        if args.action == "event":
            if args.event is None:
                raise ValueError("--event is required")
            raw = sys.stdin.buffer.read(MAX_INPUT + 1)
            if len(raw) > MAX_INPUT:
                raise ValueError("Hook input exceeds 1 MiB")
            result = event(args.project, args.host, args.event, json.loads(raw))
            print(json.dumps(result, ensure_ascii=True))
            return 0
        repository = Repository(args.project)
        directory = repository.session_dir(args.host, args.session)
        active_path = safe_path(directory, "active.json")
        if args.action == "activate":
            if (
                not args.pr
                or len(args.pr) > 256
                or any(
                    not getattr(args, key) for key in ("design", "contract", "handoff")
                )
            ):
                raise ValueError(
                    "activate requires --pr, --design, --contract, and --handoff"
                )
            if args.base is not None and (
                not args.base.strip() or args.base.startswith("-")
            ):
                raise ValueError("--base must be a revision, not an option")
            active = {
                "schema": BINDING_SCHEMA,
                "host": args.host,
                "session_id": args.session,
                "pr": args.pr,
                "binding_id": uuid.uuid4().hex,
                **repository.identity(),
            }
            for key in ("design", "contract", "handoff"):
                path = safe_path(repository.root, getattr(args, key))
                if not path.is_file():
                    raise ValueError(f"Create the {key} document before binding the PR")
                active[key] = path.relative_to(repository.root).as_posix()
            if args.base is not None:
                # Recorded, not resolved: a base that resolves now can stop
                # resolving after a rebase, so resolution belongs where it is used.
                active["base"] = args.base
            if active_path.exists() and not read_json(active_path).get("closed"):
                raise ValueError(
                    "Session already bound; deactivate explicitly before rebinding"
                )
            atomic_json(active_path, active)
            print(
                "Bound this session only. No approval, checkpoint, or test evidence was created."
            )
        elif args.action == "deactivate":
            active = read_json(active_path)
            active["closed"] = True
            atomic_json(active_path, active)
            print(
                "Closed this session binding; retained local checkpoints and rescue data."
            )
        else:
            active = active_record(repository, directory, args.host, args.session)
            atomic_json(
                safe_path(directory, "checkpoint.json"),
                {
                    "binding": digest(encoded(active)),
                    "snapshot": repository.snapshot(active),
                },
            )
            print(
                "Recorded mechanical freshness only; semantic handoff/recovery remains the agent's responsibility."
            )
        return 0
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        subprocess.SubprocessError,
    ) as error:
        if args.action == "event":
            # Never echo raw input, file contents, or subprocess stderr into context.
            message = (
                str(error)
                if isinstance(error, ValueError)
                and not isinstance(error, json.JSONDecodeError)
                else f"Continuity state unavailable ({type(error).__name__}); reconcile actual state."
            )
            if args.event == "pre-manual":
                print(json.dumps(manual_denial(args.host, message)))
            elif args.event == "session-start":
                print(
                    json.dumps(
                        context("Structured Coding: RECOVERY REQUIRED. " + message)
                    )
                )
            else:
                print(
                    "Structured Coding: automatic compact allowed; snapshot failed. Recovery required.",
                    file=sys.stderr,
                )
                print("{}")
            return 0
        print(f"Continuity command failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
