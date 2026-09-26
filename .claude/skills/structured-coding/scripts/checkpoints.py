#!/usr/bin/env python3
"""Advisory commit preparation and at-most-once, non-continuing review notices."""

import argparse
import fcntl
import json
import shlex
import subprocess
import sys
from contextlib import contextmanager
from pathlib import Path

# Installed helpers must not create files in the published skill tree.
sys.dont_write_bytecode = True
# Import by location rather than relying on sys.path[0], which is absent under
# python3 -P. This import is above main()'s try, so failing here would exit
# non-zero without the empty result a hook must always return.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from continuity import (  # noqa: E402
    HOSTS,
    MAX_INPUT,
    Repository,
    active_record,
    atomic_json,
    digest,
    encoded,
    read_json,
    safe_path,
)

MAX_CONTEXT = 8000
DOCUMENTS = ("design", "contract", "handoff")
ERRORS = (
    OSError,
    ValueError,
    KeyError,
    TypeError,
    RecursionError,
    subprocess.SubprocessError,
)
COMMIT_CHECKLIST = (
    "Before the semantic commit: run checkpoints.py inspect explicitly; inspect the actual "
    "staged diff and staged filenames; synchronize the existing design and handoff; record "
    "relevant evidence and deviations. Refresh continuity.py checkpoint only after that work. "
    "No new operator approval is required for a commit. This context is an advisory backstop "
    "for an already-selected command, not evidence that preparation occurred."
)
REVIEW_CHECKLIST = (
    "Review the primary design's actual terminal conditions. Record final executable HEAD, "
    "PR HEAD, applicable tests/Gates and exact-head CI, limitations and deviations, and the "
    "operator handoff. Keep pending, inconclusive and not run distinct from pass. A blocked "
    "or inconclusive handoff is allowed. Neither intent nor mechanical freshness proves "
    "readiness, test success, semantic review or operator permission to merge."
)


def bounded(message):
    # Bound JSON-escaped string content too, including non-ASCII paths.
    if len(json.dumps(message, ensure_ascii=True)) <= MAX_CONTEXT:
        return message
    suffix = " [details omitted; run inspect and read the bound documents in full]"
    low, high = 0, len(message)
    while low < high:
        middle = (low + high + 1) // 2
        if len(json.dumps(message[:middle] + suffix, ensure_ascii=True)) <= MAX_CONTEXT:
            low = middle
        else:
            high = middle - 1
    return message[:low] + suffix


def recognized(command):
    """Only direct shell words; intentionally exclude even quoted shell metacharacters."""
    if not isinstance(command, str) or any(c in command for c in "\n\r\0;&|<>$`\\()"):
        return False
    try:
        words = shlex.split(command)
    except ValueError:
        return False
    return bool(
        len(words) >= 2
        and (
            words[0] == "git"
            or (Path(words[0]).is_absolute() and Path(words[0]).name == "git")
        )
        and words[1] == "commit"
        and not any(c in command for c in "*?[]{}~")
    )


def inspect_state(repository, host, session):
    directory = repository.session_dir(host, session)
    path = safe_path(directory, "active.json")
    report = {"host": host, "identity": repository.identity(), "freshness": "unbound"}
    if not path.exists():
        return report, None, None
    raw = read_json(path)
    if raw.get("closed") is True:
        # Still validate the association before reporting closed.
        if (
            raw.get("host") != host
            or raw.get("session_id") != session
            or any(raw.get(k) != report["identity"][k] for k in ("worktree", "gitdir"))
        ):
            raise ValueError("Closed binding identity mismatch")
        report["freshness"] = "closed"
        return report, None, None
    active = active_record(repository, directory, host, session)
    report.update(pr=active["pr"], documents={k: active[k] for k in DOCUMENTS})
    if active["branch"] != report["identity"]["branch"]:
        report.update(
            freshness="unavailable", diagnostic="Branch mismatch; rebind explicitly"
        )
        return report, active, None
    try:
        current = repository.snapshot(active)
        checkpoint = safe_path(directory, "checkpoint.json")
        if not checkpoint.exists():
            report.update(
                freshness="missing", diagnostic="Mechanical checkpoint is absent"
            )
        else:
            receipt = read_json(checkpoint)
            report["freshness"] = (
                "fresh"
                if receipt == {"binding": digest(encoded(active)), "snapshot": current}
                else "stale"
            )
        return report, active, current
    except FileNotFoundError:
        report.update(freshness="missing", diagnostic="A bound document is missing")
    except ERRORS as error:
        report.update(
            freshness="unavailable",
            diagnostic=f"State could not be read ({type(error).__name__})",
        )
    return report, active, None


@contextmanager
def locked(directory):
    directory.mkdir(parents=True, exist_ok=True, mode=0o700)
    path = safe_path(directory, "checkpoints-review.lock")
    with path.open("a") as stream:
        try:
            fcntl.flock(stream, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise ValueError(
                "Review intent is busy; retry the explicit helper"
            ) from None
        yield


def intent_path(directory):
    return safe_path(directory, "checkpoints-review.json")


def read_intent(directory):
    path = intent_path(directory)
    if not path.exists():
        return None
    value = read_json(path)
    if (
        value.get("schema") != 1
        or value.get("state") not in ("pending", "consumed", "cancelled")
        or any(
            not isinstance(value.get(k), str)
            for k in ("binding", "snapshot", "request")
        )
        or not isinstance(value.get("documents"), dict)
    ):
        raise ValueError("Invalid review intent; reconcile session state")
    return value


def persist(repository, directory, active, record):
    # Activation is an independent existing helper; refuse an observed concurrent rebind.
    if read_json(safe_path(directory, "active.json")) != active:
        raise ValueError("Binding changed during review preparation")
    atomic_json(intent_path(directory), record)


def prepare_review(repository, host, session):
    directory = repository.session_dir(host, session)
    with locked(directory):
        report, active, snapshot = inspect_state(repository, host, session)
        if active is None or snapshot is None:
            raise ValueError(
                "Review preparation requires a valid binding and readable current state"
            )
        binding = digest(encoded(active))
        fingerprint = digest(encoded(snapshot))
        documents = {key: active[key] for key in DOCUMENTS}
        request = digest(encoded([binding, fingerprint, documents]))
        previous = read_intent(directory)
        if previous is None or previous["request"] != request:
            previous = {
                "schema": 1,
                "binding": binding,
                "snapshot": fingerprint,
                "documents": documents,
                "request": request,
                "state": "pending",
            }
            persist(repository, directory, active, previous)
        return {**report, "notice": previous["state"], "checklist": REVIEW_CHECKLIST}


def cancel_review(repository, host, session):
    directory = repository.session_dir(host, session)
    with locked(directory):
        record = read_intent(directory)
        if record and record["state"] == "pending":
            record["state"] = "cancelled"
            atomic_json(intent_path(directory), record)
    return {"notice": record["state"] if record else "absent"}


def stop(repository, host, session):
    directory = repository.session_dir(host, session)
    if not intent_path(directory).exists():
        return {}
    with locked(directory):
        record = read_intent(directory)
        if record is None or record["state"] != "pending":
            return {}
        report, active, _snapshot = inspect_state(repository, host, session)
        mismatch = (
            active is None
            or record["binding"] != digest(encoded(active))
            or record["documents"] != {key: active[key] for key in DOCUMENTS}
            or active["branch"] != report["identity"]["branch"]
        )
        record["state"] = "cancelled" if mismatch else "consumed"
        atomic_json(intent_path(directory), record)
        if mismatch:
            return {
                "systemMessage": "Structured Coding: old review intent cancelled; reconcile the current binding/branch/documents explicitly."
            }
        return {
            "systemMessage": bounded(
                "Structured Coding review notice (operator-facing, at most once). "
                + REVIEW_CHECKLIST
                + " Current mechanical state: "
                + json.dumps(report, ensure_ascii=True)
            )
        }


def event(project, host, mode, payload):
    expected = {"pre-commit": "PreToolUse", "stop": "Stop"}[mode]
    if not isinstance(payload, dict) or payload.get("hook_event_name") != expected:
        raise ValueError("Unexpected lifecycle payload")
    if mode == "pre-commit":
        tool_input = payload.get("tool_input")
        if payload.get("tool_name") != "Bash" or not isinstance(tool_input, dict):
            return {}
        if any(key in tool_input for key in ("cwd", "workdir")):
            return {}
        if not recognized(tool_input.get("command")):
            return {}
    elif type(payload.get("stop_hook_active")) is not bool:
        raise ValueError("Missing or invalid stop_hook_active")
    elif payload["stop_hook_active"]:
        return {}
    repository = Repository(project)
    session = payload.get("session_id")
    repository.session_dir(host, session)
    cwd = payload.get("cwd")
    if not isinstance(cwd, str) or not cwd or not Path(cwd).is_absolute():
        raise ValueError("Missing or invalid lifecycle cwd")
    # Exact root excludes nested repositories and shell cwd overrides.
    if Path(cwd).resolve(strict=True) != repository.root:
        return {}
    if mode == "stop":
        return stop(repository, host, session)
    report, active, _ = inspect_state(repository, host, session)
    if active is None:
        return {}
    return {
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "additionalContext": bounded(
                "Structured Coding advisory. "
                + COMMIT_CHECKLIST
                + " Mechanical freshness is not test/review/approval evidence. "
                + json.dumps(report, ensure_ascii=True)
            ),
        }
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action", choices=("inspect", "prepare-review", "cancel-review", "event")
    )
    parser.add_argument("--host", choices=HOSTS, required=True)
    parser.add_argument("--project", type=Path, required=True)
    parser.add_argument("--session")
    parser.add_argument("--event", choices=("pre-commit", "stop"))
    args = parser.parse_args(argv)
    try:
        if args.action == "event":
            if not args.event:
                raise ValueError("--event is required")
            raw = sys.stdin.buffer.read(MAX_INPUT + 1)
            if len(raw) > MAX_INPUT:
                raise ValueError("Oversized lifecycle payload")
            result = event(args.project, args.host, args.event, json.loads(raw))
        else:
            repository = Repository(args.project)
            if args.action == "inspect":
                result, _, snapshot = inspect_state(repository, args.host, args.session)
                print(json.dumps(result, ensure_ascii=True))
                return int(
                    result["freshness"] == "unavailable"
                    or (result["freshness"] == "missing" and snapshot is None)
                )
            helper = (
                prepare_review if args.action == "prepare-review" else cancel_review
            )
            result = helper(repository, args.host, args.session)
        print(json.dumps(result, ensure_ascii=True))
        return 0
    except ERRORS as error:
        # No raw input, document contents, or subprocess stderr in diagnostics.
        print(
            f"Checkpoints state unavailable ({type(error).__name__}); reconcile locally.",
            file=sys.stderr,
        )
        if args.action == "event":
            print("{}")
            return 0
        return 1


if __name__ == "__main__":
    sys.exit(main())
