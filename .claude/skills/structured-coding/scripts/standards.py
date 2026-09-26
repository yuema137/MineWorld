#!/usr/bin/env python3
"""Read a project's declared coding standards. Runs nothing and enforces nothing.

See references/standards.md for the contract, the two layers, and the limits.
Uses only Python 3.9+ and Git; never invokes an LLM, a tool, or a remote API.
"""

import argparse
import hashlib
import json
import os
import re
import signal
import subprocess
import time
import sys
from pathlib import Path

# Installed helpers must not create files in the published skill tree.
sys.dont_write_bytecode = True
# Import the audited worktree resolution explicitly rather than relying on
# sys.path[0], which is how checkpoints.py reaches it and why that script fails
# hard under python3 -P. Fixing checkpoints.py is separate, recorded work.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from continuity import (  # noqa: E402
    HOSTS, Repository, active_record, atomic_json, read_json, safe_path,
)
# recognized() is the careful direct-git-commit test; it is generic rather than
# checkpoints-specific, and both files always ship together.
from checkpoints import bounded, recognized  # noqa: E402

MAX_INPUT = 256 * 1024
ERRORS = (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError)
# 1 is what the shipped defaults declare, because they need nothing newer.
# 2 adds a project-supplied command, for a tool we ship no argv for.
SCHEMA = 1
SCHEMAS = (1, 2)
TRIGGERS = ("off", "pr", "commit")
SCOPES = ("changed", "repository")
NAME = re.compile(r"\A[a-z][a-z0-9_-]{0,31}\Z")
OPENING = re.compile(r"\A\s{0,3}(`{3,}|~{3,})[ \t]*json[ \t]*\Z")
FENCE = re.compile(r"\A\s{0,3}(`{3,}|~{3,})(.*)\Z")


class Invalid(ValueError):
    """Names the file and the offending field, never the file's contents."""

    def __init__(self, path, field, problem):
        super().__init__(f"{path}: {field}: {problem}")


def blocks(text):
    """Every top-level ```json block. A fence inside another fence is content."""
    found, capturing, marker, lines = [], False, None, []
    for line in text.splitlines():
        match = FENCE.match(line)
        if marker is None:
            if match and OPENING.match(line):
                marker, capturing, lines = match.group(1), True, []
            elif match:
                marker, capturing = match.group(1), False
            continue
        closing, tail = match.groups() if match else ("", "x")
        if closing[:1] == marker[0] and len(closing) >= len(marker) and not tail.strip():
            if capturing:
                found.append("\n".join(lines))
            marker, capturing = None, False
        elif capturing:
            lines.append(line)
    return found


def read(path):
    """The declared configuration in one file, or a precise refusal."""
    if path.is_symlink():
        raise Invalid(path, "file", "symlinked configuration files are not read")
    if not path.is_file():
        raise Invalid(path, "file", "not a regular file")
    if path.stat().st_size > MAX_INPUT:
        raise Invalid(path, "file", f"larger than {MAX_INPUT} bytes")
    try:
        text = path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        raise Invalid(path, "file", "not valid UTF-8") from None
    found = blocks(text)
    if len(found) != 1:
        raise Invalid(
            path, "json block", f"expected exactly one ```json block, found {len(found)}"
        )
    try:
        value = json.loads(found[0])
    except json.JSONDecodeError:
        raise Invalid(path, "json block", "is not valid JSON") from None
    return validate(path, value)


def section(path, value, name, extra):
    if not isinstance(value, dict):
        raise Invalid(path, name, "must be an object")
    unknown = set(value) - ({"trigger"} | set(extra))
    if unknown:
        raise Invalid(path, name, f"unknown keys: {', '.join(sorted(unknown))}")
    if "trigger" in value and value["trigger"] not in TRIGGERS:
        raise Invalid(path, f"{name}.trigger", f"must be one of {', '.join(TRIGGERS)}")
    return value


def conventions(path, value):
    if not isinstance(value, list):
        raise Invalid(path, "review.conventions", "must be a list")
    for index, item in enumerate(value):
        if not isinstance(item, str) or not item.strip():
            raise Invalid(
                path, f"review.conventions[{index}]", "must be a non-empty string"
            )
    return list(value)


def command_words(path, field, value, name):
    """argv, not a shell string. Nothing here is ever passed to a shell."""
    if name in ANALYZERS:
        raise Invalid(
            path, field, f"{name} runs with the argv this skill ships; remove command"
        )
    if not isinstance(value, list) or not value:
        raise Invalid(path, field, "must be a non-empty list of argv words")
    for word in value:
        if not isinstance(word, str) or not word:
            raise Invalid(path, field, "every argv word must be a non-empty string")
    if any(character.isspace() for character in value[0]):
        raise Invalid(
            path, field, "the first word is the executable, not a whole shell command"
        )
    return list(value)


def tools(path, value, schema=SCHEMA):
    if not isinstance(value, list):
        raise Invalid(path, "checks.tools", "must be a list")
    seen = set()
    for index, item in enumerate(value):
        field = f"checks.tools[{index}]"
        if not isinstance(item, dict):
            raise Invalid(path, field, "must be an object")
        allowed = {"name", "enabled", "scope"} | ({"command"} if schema >= 2 else set())
        unknown = set(item) - allowed
        if unknown:
            raise Invalid(path, field, f"unknown keys: {', '.join(sorted(unknown))}")
        name = item.get("name")
        if not isinstance(name, str) or not NAME.match(name):
            raise Invalid(path, f"{field}.name", "must be a short lowercase tool name")
        if name in seen:
            raise Invalid(path, f"{field}.name", f"{name} is declared more than once")
        seen.add(name)
        if "enabled" in item and not isinstance(item["enabled"], bool):
            raise Invalid(path, f"{field}.enabled", "must be true or false")
        if "scope" in item and item["scope"] not in SCOPES:
            raise Invalid(path, f"{field}.scope", f"must be one of {', '.join(SCOPES)}")
        if "command" in item:
            command_words(path, f"{field}.command", item["command"], name)
    return [dict(item) for item in value]


def validate(path, value):
    if not isinstance(value, dict):
        raise Invalid(path, "json block", "must be an object")
    schema = value.get("schema")
    if schema not in SCHEMAS:
        raise Invalid(path, "schema", f"must be one of {', '.join(map(str, SCHEMAS))}")
    unknown = set(value) - {"schema", "review", "checks"}
    if unknown:
        raise Invalid(path, "json block", f"unknown keys: {', '.join(sorted(unknown))}")
    declared = {"schema": schema}
    if "review" in value:
        review = section(path, value["review"], "review", ("conventions",))
        declared["review"] = dict(review)
        if "conventions" in review:
            declared["review"]["conventions"] = conventions(path, review["conventions"])
    if "checks" in value:
        checks = section(path, value["checks"], "checks", ("tools",))
        declared["checks"] = dict(checks)
        if "tools" in checks:
            declared["checks"]["tools"] = tools(path, checks["tools"], schema)
    return declared


# Ranked from permissive to strict. An overlay may raise a rank, never lower it.
TRIGGER_RANK = {name: rank for rank, name in enumerate(TRIGGERS)}
SCOPE_RANK = {name: rank for rank, name in enumerate(SCOPES)}

DEFAULTS = {
    "schema": SCHEMA,
    "review": {"trigger": "pr", "conventions": []},
    "checks": {
        "trigger": "pr",
        "tools": [
            {"name": "ruff", "enabled": True, "scope": "changed"},
            {"name": "pyright", "enabled": True, "scope": "changed"},
            {"name": "pytest", "enabled": False, "scope": "repository"},
        ],
    },
}


def defaults():
    """A fresh copy, so a caller can never mutate the shipped values."""
    return json.loads(json.dumps(DEFAULTS))


def trigger(effective, origins, declared, name, layer, path, restrict):
    if "trigger" not in declared:
        return
    value, current = declared["trigger"], effective[name]["trigger"]
    if restrict and TRIGGER_RANK[value] < TRIGGER_RANK[current]:
        raise Invalid(
            path,
            f"{name}.trigger",
            f"a personal file may not relax {current} to {value}",
        )
    effective[name]["trigger"] = value
    origins[f"{name}.trigger"] = layer


def merge_conventions(effective, origins, declared, layer):
    """Overlay entries are additions, so removal is impossible by construction."""
    if "conventions" not in declared:
        return
    existing = effective["review"]["conventions"]
    for item in declared["conventions"]:
        if item not in existing:
            existing.append(item)
            origins[f"review.conventions[{existing.index(item)}]"] = layer


def merge_tools(effective, origins, declared, layer, path, restrict):
    if "tools" not in declared:
        return
    known = {tool["name"]: tool for tool in effective["checks"]["tools"]}
    for item in declared["tools"]:
        name = item["name"]
        current = known.get(name)
        if current is None:
            tool = {"name": name, "enabled": True, "scope": "changed", **item}
            effective["checks"]["tools"].append(tool)
            known[name] = tool
            origins[f"checks.tools.{name}"] = layer
            continue
        field = f"checks.tools.{name}"
        if "enabled" in item:
            if restrict and current["enabled"] and not item["enabled"]:
                raise Invalid(
                    path,
                    f"{field}.enabled",
                    "a personal file may not disable a shared check",
                )
            if current["enabled"] != item["enabled"]:
                current["enabled"] = item["enabled"]
                origins[f"{field}.enabled"] = layer
        if "scope" in item:
            if restrict and SCOPE_RANK[item["scope"]] < SCOPE_RANK[current["scope"]]:
                raise Invalid(
                    path,
                    f"{field}.scope",
                    f"a personal file may not narrow {current['scope']} to {item['scope']}",
                )
            if current["scope"] != item["scope"]:
                current["scope"] = item["scope"]
                origins[f"{field}.scope"] = layer


def apply_layer(effective, origins, layer, path, declared, restrict):
    for name in ("review", "checks"):
        if name not in declared:
            continue
        trigger(effective, origins, declared[name], name, layer, path, restrict)
    if "review" in declared:
        merge_conventions(effective, origins, declared["review"], layer)
    if "checks" in declared:
        merge_tools(effective, origins, declared["checks"], layer, path, restrict)


def resolve(base=None, overlay=None):
    """Defaults, then the shared file, then the personal one which may only tighten."""
    effective = defaults()
    origins = {
        "review.trigger": "default",
        "checks.trigger": "default",
        **{f"checks.tools.{tool['name']}": "default" for tool in effective["checks"]["tools"]},
    }
    for layer, source, restrict in (("base", base, False), ("overlay", overlay, True)):
        if source is not None:
            path, declared = source
            apply_layer(effective, origins, layer, path, declared, restrict)
    return effective, origins


# The two files. Layer comes from the filename; trust never does.
DIRECTORY = ".structured-coding"
LAYERS = (("base", f"{DIRECTORY}/standards.md"), ("overlay", f"{DIRECTORY}/standards.local.md"))

# Analyzers read the project's code. Runners execute it, or code its config names,
# so recognising them is not the same as trusting them.
ANALYZERS = ("ruff", "pyright")
RUNNERS = {
    "pytest": "runs project code through conftest.py by design",
    "mypy": "imports the plugins its configuration names",
}


# What an allowlisted tool is actually invoked as. A project never supplies these.
SHIPPED = {
    "ruff": {"base": ("ruff", "check"), "repository": (".",)},
    "pyright": {"base": ("pyright",), "repository": ()},
}


def argv(tool, paths):
    """The command for one tool, or None when the project must supply it."""
    name = tool["name"]
    if name in SHIPPED:
        shipped = SHIPPED[name]
        if tool.get("scope", "changed") == "repository":
            return [*shipped["base"], *shipped["repository"]]
        return [*shipped["base"], *paths]
    command = tool.get("command")
    return None if command is None else [*command, *paths]


def changed_files(root, base):
    """What this branch changed against base. Deletions are excluded: a file that
    is gone cannot be checked, and reporting it would make a tool fail on it."""
    if not isinstance(base, str) or not base or base.startswith("-"):
        raise Invalid(root, "base", "must be a Git revision")
    result = subprocess.run(
        ["git", "--no-optional-locks", "-C", str(root), "diff", "--name-only",
         "--diff-filter=ACMR", f"{base}...HEAD"],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, timeout=30,
    )
    if result.returncode:
        raise Invalid(
            root, "base", f"{base} is not a revision this repository can compare with"
        )
    names = [name for name in os.fsdecode(result.stdout).splitlines() if name]
    # A path can survive the diff and still be absent from the working tree.
    return [name for name in names if (root / name).is_file()]


def needs_base(effective):
    return any(
        tool.get("enabled", True) and tool.get("scope", "changed") == "changed"
        for tool in effective["checks"]["tools"]
    )


def paths_for(root, tool, base):
    """The paths this tool runs on, or the reason it is not run at all."""
    if tool.get("scope", "changed") == "repository":
        return [], None
    if base is None:
        return [], "a base revision is required for changed-file scope"
    paths = changed_files(root, base)
    if not paths:
        # Never a pass: nothing was examined, so nothing was established.
        return [], "no files changed against the base"
    return paths, None


def classify(name):
    if name in ANALYZERS:
        return "allowlisted", "analyzes without executing project code"
    if name in RUNNERS:
        return "approval required", RUNNERS[name]
    return "approval required", "not a recognized tool"


def tracked(root, path):
    """Git decides trust. A filename cannot, because .gitignore does not apply
    to a file that is already tracked."""
    result = subprocess.run(
        ["git", "--no-optional-locks", "-C", str(root), "ls-files",
         "--error-unmatch", "--", str(path)],
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
        timeout=5,
    )
    return result.returncode == 0


def discover(project):
    """Every configuration file present, with its layer and its trust."""
    root = Repository(project).root
    present = {}
    for layer, relative in LAYERS:
        path = safe_path(root, relative)
        if not path.exists() and not path.is_symlink():
            continue
        present[layer] = {
            "relative": relative,
            "path": path,
            "trust": "shared" if tracked(root, path) else "personal",
            "declared": read(path),
        }
    return root, present


# The approval lives beside the other per-worktree state, outside the working tree,
# so a pull request cannot carry an approval for its own new command.
APPROVAL = "structured-coding-standards/approval.json"


def pending_approval(effective):
    """Enabled tools this skill ships no argv for, paired with what they would run.

    A tool with no command cannot run at all, so it needs a command rather than an
    approval and is not listed here."""
    return sorted(
        [tool["name"], list(tool["command"])]
        for tool in effective["checks"]["tools"]
        if tool.get("enabled", True)
        and tool["name"] not in SHIPPED
        and tool.get("command")
    )


def digest(pending):
    """Bound to the commands only. Binding it to the whole configuration would
    invalidate approval on every unrelated edit, which teaches people to reapprove
    without reading."""
    return hashlib.sha256(
        json.dumps(pending, ensure_ascii=True, sort_keys=True).encode()
    ).hexdigest()


def approval_file(project):
    return safe_path(Repository(project).gitdir, APPROVAL)


def approved(project, pending):
    if not pending:
        return True
    path = approval_file(project)
    if not path.exists():
        return False
    try:
        return read_json(path).get("commands") == digest(pending)
    except (OSError, ValueError):
        return False


def grant(project, effective):
    """Record an approval for exactly the commands currently declared."""
    pending = pending_approval(effective)
    path = approval_file(project)
    if not pending:
        return pending, None
    atomic_json(path, {"schema": 1, "commands": digest(pending)})
    return pending, path


TOOL_SECONDS = 300
TOTAL_SECONDS = 900
MAX_OUTPUT = 16 * 1024

# Exit codes that mean the tool could not run, as opposed to it finding something.
# ruff: 0 none, 1 violations, 2 abnormal termination (docs.astral.sh/ruff/linter).
# pyright: 0 none, 1 errors, 2 fatal, 3 unreadable config, 4 illegal parameters
# (microsoft/pyright docs/command-line.md). Checked 2026-09-09.
ERROR_EXITS = {"ruff": (2,), "pyright": (2, 3, 4)}


def bounded_output(raw):
    text = os.fsdecode(raw)
    if len(text) <= MAX_OUTPUT:
        return text.strip()
    return text[:MAX_OUTPUT].strip() + "\n[output truncated]"


GRACE_SECONDS = 5


def end_group(process):
    """A tool's children must not outlive the event that started it.

    subprocess terminates only the child it started. Under an explicit command an
    operator would notice an orphaned tree; under a hook nothing would."""
    for number in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(process.pid, number)
        except (OSError, ProcessLookupError):
            return
        try:
            process.wait(timeout=GRACE_SECONDS)
            if number is signal.SIGKILL:
                return
        except subprocess.TimeoutExpired:
            continue
        else:
            # The leader is gone; make sure nothing it spawned is still holding on.
            try:
                os.killpg(process.pid, 0)
            except (OSError, ProcessLookupError):
                return


def execute(root, tool, command, seconds):
    """Run one tool and classify from what happened, not from the exit code alone."""
    if seconds <= 0:
        return "INCONCLUSIVE", "the total time budget was already spent", ""
    started = time.monotonic()
    try:
        # Its own session, so a timeout can end the whole tree and not just the leader.
        process = subprocess.Popen(
            command, cwd=str(root), stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT, start_new_session=True,
        )
    except FileNotFoundError:
        # Never a pass: an absent tool established nothing.
        return "INCONCLUSIVE", f"{command[0]} is not installed or not on PATH", ""
    except OSError as error:
        return "INCONCLUSIVE", f"could not start ({type(error).__name__})", ""
    try:
        captured, _ = process.communicate(timeout=seconds)
    except subprocess.TimeoutExpired:
        end_group(process)
        try:
            process.communicate(timeout=GRACE_SECONDS)
        except (subprocess.TimeoutExpired, OSError, ValueError):
            pass
        return "INCONCLUSIVE", f"no result within {seconds:.0f}s", ""
    elapsed = time.monotonic() - started
    output = bounded_output(captured or b"")
    code = process.returncode
    if code == 0:
        return "PASS", f"clean in {elapsed:.1f}s", output
    if code < 0:
        return "INCONCLUSIVE", f"killed by signal {-code}", output
    if code in ERROR_EXITS.get(tool["name"], ()):
        return "INCONCLUSIVE", f"exited {code}, its own error mode", output
    if tool["name"] not in ERROR_EXITS:
        return "FAIL", f"exited {code}; this skill has no exit-code map for it", output
    return "FAIL", f"exited {code} in {elapsed:.1f}s", output


def outcomes(project, effective, base):
    """Every enabled tool, with what happened to it."""
    root = Repository(project).root
    pending = pending_approval(effective)
    permitted = approved(project, pending)
    waiting = {name for name, _ in pending}
    deadline = time.monotonic() + TOTAL_SECONDS
    results = []
    for tool in effective["checks"]["tools"]:
        entry = {"name": tool["name"], "scope": tool.get("scope", "changed")}
        if not tool.get("enabled", True):
            results.append({**entry, "outcome": "NOT RUN", "reason": "disabled"})
            continue
        if tool["name"] in waiting and not permitted:
            results.append({**entry, "outcome": "NOT RUN",
                            "reason": "approval required; run the approve command"})
            continue
        paths, reason = paths_for(root, tool, base)
        if reason:
            results.append({**entry, "outcome": "NOT RUN", "reason": reason})
            continue
        command = argv(tool, paths)
        if command is None:
            results.append({**entry, "outcome": "NOT RUN", "reason":
                            "no command declared, and this skill ships none for it"})
            continue
        outcome, why, output = execute(
            root, tool, command, min(TOOL_SECONDS, deadline - time.monotonic())
        )
        results.append({**entry, "outcome": outcome, "reason": why,
                        "command": command[:1] + (["..."] if len(command) > 1 else []),
                        "files": len(paths), "output": output})
    return results


def bound_base(project, host, session):
    """The base a bound PR recorded, or None when there is none to use.

    Never raises: an unbound, closed, foreign or baseless binding all mean the
    same thing to a caller, which is that it must be told the base instead."""
    if host not in HOSTS or not isinstance(session, str) or not session:
        return None
    try:
        repository = Repository(project)
        record = active_record(
            repository, repository.session_dir(host, session), host, session
        )
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError):
        return None
    base = record.get("base")
    return base if isinstance(base, str) and base.strip() else None


def report(project):
    """What is in effect, where each value came from, and what would need approval."""
    root, present = discover(project)
    base = present.get("base")
    overlay = present.get("overlay")
    effective, origins = resolve(
        None if base is None else (base["relative"], base["declared"]),
        None if overlay is None else (overlay["relative"], overlay["declared"]),
    )
    tools = []
    for tool in effective["checks"]["tools"]:
        verdict, reason = classify(tool["name"])
        tools.append({**tool, "approval": verdict, "reason": reason})
    return {
        "project": str(root),
        "sources": [
            {"layer": layer, "path": present[layer]["relative"], "trust": present[layer]["trust"]}
            for layer, _ in LAYERS
            if layer in present
        ],
        "effective": {
            "review": effective["review"],
            "checks": {"trigger": effective["checks"]["trigger"], "tools": tools},
        },
        "origins": origins,
        "note": (
            "No configuration file found; the shipped defaults apply and nothing changes."
            if not present
            else "Reported only. This helper runs no check and registers no hook."
        ),
    }


def resolved(project):
    _, present = discover(project)
    base, overlay = present.get("base"), present.get("overlay")
    return resolve(
        None if base is None else (base["relative"], base["declared"]),
        None if overlay is None else (overlay["relative"], overlay["declared"]),
    )


def approve(project):
    """Operator command. Nothing here prevents an agent from running it, exactly
    as nothing prevents an agent from running the installer; see references."""
    root, present = discover(project)
    base = present.get("base")
    overlay = present.get("overlay")
    effective, _ = resolve(
        None if base is None else (base["relative"], base["declared"]),
        None if overlay is None else (overlay["relative"], overlay["declared"]),
    )
    pending, path = grant(project, effective)
    if not pending:
        print("Nothing needs approval; no record was written.")
        return 0
    print(f"Approving {len(pending)} command(s) for {root}:")
    for name, command in pending:
        print(f"  {name}: {' '.join(command)}")
    print(f"Recorded in {path}. Changing any of these commands revokes it.")
    return 0


COMMIT_TIMEOUT = 60


def event(project, host, mode, payload):
    """Report after a commit. Never blocks, never raises to the host."""
    if mode != "post-commit" or payload.get("hook_event_name") != "PostToolUse":
        raise Invalid(project, "event", "unexpected lifecycle payload")
    tool_input = payload.get("tool_input")
    if payload.get("tool_name") != "Bash" or not isinstance(tool_input, dict):
        return {}
    if any(key in tool_input for key in ("cwd", "workdir")):
        return {}
    if not recognized(tool_input.get("command")):
        return {}
    cwd = payload.get("cwd")
    if not isinstance(cwd, str) or not cwd or not Path(cwd).is_absolute():
        raise Invalid(project, "cwd", "missing or invalid lifecycle cwd")
    repository = Repository(project)
    # Exact root excludes nested repositories and shell cwd overrides.
    if Path(cwd).resolve(strict=True) != repository.root:
        return {}
    effective, _ = resolved(project)
    parts = []
    if effective["review"]["trigger"] == "commit":
        for convention in effective["review"]["conventions"]:
            parts.append(f"Convention: {convention}")
    if effective["checks"]["trigger"] == "commit":
        base = bound_base(project, host, payload.get("session_id"))
        for result in outcomes(project, effective, base):
            parts.append(f"{result['name']}: {result['outcome']} ({result['reason']})")
    if not parts:
        return {}
    return {
        "hookSpecificOutput": {
            "hookEventName": "PostToolUse",
            "additionalContext": bounded(
                "Structured Coding standards, after this commit. Advisory: nothing "
                "was blocked, a convention is for you to apply, and an INCONCLUSIVE "
                "or NOT RUN check is not a pass. " + "; ".join(parts)
            ),
        }
    }


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("inspect", "run", "approve", "event"))
    parser.add_argument("--event", choices=("post-commit",))
    parser.add_argument("--project", required=True, type=Path)
    parser.add_argument("--base", help="revision changed-file scope compares against")
    parser.add_argument("--host", choices=HOSTS, help="host of a session binding to read the base from")
    parser.add_argument("--session", help="session id of that binding")
    args = parser.parse_args(argv)
    try:
        if args.action == "event":
            raw = sys.stdin.buffer.read(MAX_INPUT + 1)
            if len(raw) > MAX_INPUT:
                raise Invalid(args.project, "event", "oversized lifecycle payload")
            try:
                print(json.dumps(event(args.project, args.host, args.event,
                                       json.loads(raw)), ensure_ascii=True))
            except ERRORS:
                # Fail open: a hook must never block the host it advises.
                print("Standards hook unavailable; reconcile locally.", file=sys.stderr)
                print("{}")
            return 0
        if args.action == "approve":
            return approve(args.project)
        value = report(args.project)
        if args.action == "run":
            effective, _ = resolved(args.project)
            # An explicit revision wins: a person naming one is more specific
            # than a record made when the PR was bound.
            base = args.base or bound_base(args.project, args.host, args.session)
            if needs_base(effective) and base is None:
                raise Invalid(args.project, "base",
                              "--base is required while a changed-scope check is enabled")
            value["results"] = outcomes(args.project, effective, base)
            value["base"] = base
            value["note"] = ("Ran the declared checks. Nothing was registered as a hook; "
                             "this happens only when this command is invoked.")
        print(json.dumps(value, ensure_ascii=True, indent=2))
        return 0
    except Invalid as error:
        # A refusal must never be reported as "the defaults apply".
        print(f"Standards configuration refused: {error}", file=sys.stderr)
        return 1
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(
            f"Standards configuration unavailable ({type(error).__name__}): {error}",
            file=sys.stderr,
        )
        return 1


if __name__ == "__main__":
    sys.exit(main())
