"""Run one onboarding round: assemble a sandbox, start a fresh agent in it, then score and audit what it made.

    python3 tools/onboarding/run.py --instructions ON-1 --agent-command claude-allowlist --budget 5 \
        --binary target/debug/mineworld --out EVIDENCE_DIR [--model sonnet] [--max-turns 80]

`tools/onboarding/PROTOCOL.md` §§3, 7 and 8 are the authority. It never runs under CI (INV-S25-6), refuses to start
without a budget and an agent command, runs only the allowlisted agent or the scripted stand-in, and never reads,
prints or copies a key: the agent's environment holds only the variables `AGENT_ENVIRONMENT` names, so no key or token
reaches it and it can only use the operator's own login. Standard library only.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path
from typing import Any

import assemble
import audit
import score

HERE = Path(__file__).resolve().parent
INSTRUCTIONS = HERE / "instructions"
# The per-run ceiling the operator ruled (QS25-3): at most USD 5 and 80 turns.
MAX_BUDGET_USD = 5.0
MAX_TURNS = 80
# Wall-clock ceiling for one agent run.
WALL_SECONDS = 3600
# The tools the agent may use: no web, no subagents (PROTOCOL.md §3).
TOOLS = "Bash,Read,Write,Edit,Glob,Grep"
# The scripted stand-in, the only agent command other than `claude-allowlist` (PROTOCOL.md §8.3).
FAKE_AGENT = HERE / "fake_agent.py"
# Words no agent command may hold: each would turn the allowlist off (operator ruling 2026-10-17, PROTOCOL.md §3.2).
BYPASS_WORDS = ("dangerously", "skip-permissions", "bypasspermissions", "bypass-permissions")
# The only variables the agent inherits (PROTOCOL.md §3.2): what a shell, a login and a temporary folder need, on
# macOS, Linux and Windows. Anything else, a key, a token or a cloud credential, never reaches it.
AGENT_ENVIRONMENT = frozenset(
    {
        "HOME",
        "USER",
        "LOGNAME",
        "SHELL",
        "TERM",
        "LANG",
        "LC_ALL",
        "LC_CTYPE",
        "TZ",
        "TMPDIR",
        "TMP",
        "TEMP",
        "SYSTEMROOT",
        "SYSTEMDRIVE",
        "WINDIR",
        "COMSPEC",
        "PATHEXT",
        "USERPROFILE",
        "HOMEDRIVE",
        "HOMEPATH",
        "APPDATA",
        "LOCALAPPDATA",
    }
)


class Refused(Exception):
    """The round cannot start as asked."""


def rule_path(path: Path) -> str:
    """`path` as a permission rule spells it: POSIX form, so Windows' `C:\\Users\\a` is `/c/Users/a` (the
    agent CLI normalises Windows paths that way before it matches a rule; PROTOCOL.md §3.2)."""
    posix = path.as_posix()
    if path.drive:
        posix = "/" + path.drive[0].lower() + posix[len(path.drive) :]
    return posix


def spellings(sandbox: Path) -> list[str]:
    """The sandbox's path as the agent may write it: as made, and with links resolved (macOS's /var is
    /private/var). A permission rule names each, so neither spelling escapes the allowlist."""
    return sorted({rule_path(sandbox), rule_path(sandbox.resolve())})


def allow_rules(sandbox: Path) -> list[str]:
    """The `allow` rules of the allowlist: `mineworld` in the shell, reads in the sandbox, writes in `work/`."""
    allow = ["Bash(mineworld)", "Bash(mineworld *)"]
    for root in spellings(sandbox):
        allow += [f"Read(/{root}/**)", f"Write(/{root}/work/**)", f"Edit(/{root}/work/**)"]
    return allow


def permissions(sandbox: Path) -> dict[str, object]:
    """The allowlist (PROTOCOL.md §3.2): in `dontAsk` mode every tool call no `allow` rule matches is denied
    before it runs, without a prompt. The shell may run only `mineworld` (the guard of §3.3); files may be read
    only in the sandbox and written only in its `work/`. A `//` prefix is an absolute path in a rule."""
    return {
        "permissions": {
            "defaultMode": "dontAsk",
            "allow": allow_rules(sandbox),
            "deny": ["WebFetch", "WebSearch", "Agent", "Task", "NotebookEdit"],
        }
    }


def claude_allowlist(model: str, budget: float, turns: int, settings: Path) -> list[str]:
    """The agent command (PROTOCOL.md §3.2): Claude Code in print mode, every customisation off, no MCP server,
    no saved session, the allowlist of `settings` in `dontAsk` mode, a turn limit and a dollar ceiling. Verified
    2026-10-17 against Claude Code 2.1.302 by the planted escape test (§3.4)."""
    executable = shutil.which("claude")
    if executable is None:
        raise Refused("claude is not on PATH")
    return [
        executable,
        "-p",
        "--model",
        model,
        "--safe-mode",
        "--strict-mcp-config",
        "--setting-sources",
        "",
        "--settings",
        str(settings),
        "--permission-mode",
        "dontAsk",
        "--tools",
        TOOLS,
        "--disallowedTools",
        "WebFetch,WebSearch,Agent,Task",
        "--no-session-persistence",
        "--output-format",
        "stream-json",
        "--verbose",
        "--max-turns",
        str(turns),
        "--max-budget-usd",
        f"{budget:g}",
    ]


def agent_environment(sandbox: Path) -> dict[str, str]:
    """The agent's environment: only the operator's variables that `AGENT_ENVIRONMENT` names (an allowlist, so no
    API key, auth token or cloud credential, and nothing of a surrounding agent session), with `mineworld` first on
    a minimal PATH."""
    keep = {name: value for name, value in os.environ.items() if name.upper() in AGENT_ENVIRONMENT}
    system: list[str] = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"] if os.name != "nt" else []
    keep["PATH"] = os.pathsep.join([str(sandbox / "bin"), *system])
    return keep


def load_average() -> tuple[float, float, float]:
    """The host's 1, 5 and 15 minute load averages for the ledger; Windows has none, so zeros."""
    if sys.platform == "win32":
        return (0.0, 0.0, 0.0)
    return os.getloadavg()


def agent_command(
    spec: str, sandbox: Path, model: str, budget: float, turns: int, settings: Path
) -> list[str]:
    """The command a round starts: the allowlisted Claude Code, or the scripted stand-in. Anything else is refused,
    so no agent can run with its permissions turned off (operator ruling 2026-10-17, PROTOCOL.md §§3.2, 8.3)."""
    if spec == "claude-allowlist":
        settings.write_text(json.dumps(permissions(sandbox), indent=2) + "\n", encoding="utf-8")
        return claude_allowlist(model, budget, turns, settings)
    words = [
        word.format(sandbox=sandbox, budget=f"{budget:g}", max_turns=turns, python=sys.executable)
        for word in shlex.split(spec)
    ]
    refuse_bypass(" ".join(words))
    stand_in = (
        len(words) == 3
        and Path(words[0]).name.lower().startswith("python")
        and Path(words[1]).resolve() == FAKE_AGENT
        and Path(words[2]) == sandbox
    )
    if not stand_in:
        raise Refused(
            f"the agent command {spec!r} is neither claude-allowlist nor the scripted stand-in "
            "'{python} tools/onboarding/fake_agent.py {sandbox}'"
        )
    return words


def refuse_bypass(spec: str) -> None:
    """Refuses an agent command that names a way to turn permissions off, before anything is assembled."""
    if any(bypass in spec.lower() for bypass in BYPASS_WORDS):
        raise Refused(f"the agent command {spec!r} turns permissions off; a round never runs one")


def prompt(instructions: str, sandbox: Path) -> str:
    preamble = (
        (INSTRUCTIONS / "preamble.txt").read_text(encoding="utf-8").strip().format(sandbox=sandbox)
    )
    instruction = (INSTRUCTIONS / f"{instructions}.txt").read_text(encoding="utf-8").strip()
    return f"{preamble}\n\n{instruction}\n"


def worlds_in(work: Path) -> list[Path]:
    return sorted(path.parent for path in work.glob("*/world.yaml"))


def result_event(transcript: Path) -> dict[str, object]:
    """The agent CLI's last `result` event: turns, cost, and how the run ended."""
    last: dict[str, object] = {}
    for line in transcript.read_text(encoding="utf-8", errors="replace").splitlines():
        try:
            event = audit.fields(json.loads(line))
        except json.JSONDecodeError:
            continue
        if event.get("type") == "result":
            last = event
    return last


def run_round(options: argparse.Namespace) -> dict[str, Any]:
    if os.environ.get("CI"):
        raise Refused("CI is set: the onboarding harness never runs in CI (INV-S25-6)")
    if options.budget is None or options.budget <= 0:
        raise Refused("--budget is required: a positive dollar ceiling for the run")
    if options.budget > MAX_BUDGET_USD:
        raise Refused(
            f"--budget {options.budget:g} is above the ruled ceiling of USD {MAX_BUDGET_USD:g} (QS25-3)"
        )
    if not options.agent_command:
        raise Refused("--agent-command is required")
    refuse_bypass(options.agent_command)
    if not 0 < options.max_turns <= MAX_TURNS:
        raise Refused(f"--max-turns must be 1 … {MAX_TURNS} (QS25-3)")
    out: Path = options.out
    if out.exists():
        raise Refused(f"{out} exists; each round writes fresh evidence")
    rubric = score.load_rubric(options.instructions)
    sandbox = Path(tempfile.mkdtemp(prefix="mw-onboard-")).resolve() / "sandbox"
    assemble.assemble(sandbox, options.binary.resolve())
    if assemble.leaks(sandbox):
        raise Refused("the sandbox names the repository's path")
    out.mkdir(parents=True)
    # The settings file lies in the evidence directory, outside the sandbox, so the agent cannot change it.
    command = agent_command(
        options.agent_command,
        sandbox,
        options.model,
        options.budget,
        options.max_turns,
        out / "settings.json",
    )
    (out / "prompt.txt").write_text(prompt(options.instructions, sandbox), encoding="utf-8")
    load_before = load_average()
    started = time.monotonic()
    transcript = out / "transcript.jsonl"
    with transcript.open("wb") as stdout, (out / "agent.stderr.txt").open("wb") as stderr:
        try:
            done = subprocess.run(
                command,
                input=(out / "prompt.txt").read_bytes(),
                stdout=stdout,
                stderr=stderr,
                cwd=sandbox / "work",
                env=agent_environment(sandbox),
                timeout=WALL_SECONDS,
            )
            code = done.returncode
        except subprocess.TimeoutExpired:
            code = -1
    wall = time.monotonic() - started
    load_after = load_average()
    shutil.copytree(sandbox / "work", out / "work")
    binary = score.Binary(
        (sandbox / "bin" / "real" / options.binary.name).resolve(), [sandbox / "entities"]
    )
    scores = [score.score(world, rubric, binary) for world in worlds_in(sandbox / "work")]
    findings = audit.audit(transcript, sandbox)
    end = result_event(transcript)
    summary: dict[str, Any] = {
        "instructions": options.instructions,
        "agent": options.agent_command,
        "model": options.model,
        "command": [word for word in command if word != str(sandbox)],
        "sandbox": str(sandbox),
        "exit": code,
        "wall_s": round(wall, 1),
        "load_before": [round(x, 2) for x in load_before],
        "load_after": [round(x, 2) for x in load_after],
        "turns": end.get("num_turns"),
        "cost_usd": end.get("total_cost_usd"),
        "terminal": end.get("subtype"),
        # Tool calls the allowlist denied before they ran (PROTOCOL.md §3.2), as the CLI records them.
        "denied": [
            audit.fields(d).get("tool_input") for d in audit.items(end.get("permission_denials"))
        ],
        "transcript_sha256": hashlib.sha256(transcript.read_bytes()).hexdigest(),
        "worlds": [{"world": s.world, "checks": [c.__dict__ for c in s.checks]} for s in scores],
        "void": [i.__dict__ for i in findings.void],
        "stuck": [i.__dict__ for i in findings.stuck],
        "blocked": [i.__dict__ for i in findings.blocked],
    }
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    report = [s.render() for s in scores] or ["score: no world.yaml under work/"]
    report.append(findings.render())
    (out / "report.txt").write_text("\n".join(report) + "\n", encoding="utf-8")
    return summary


def main(arguments: list[str]) -> int:
    parser = argparse.ArgumentParser(description="Run one onboarding round (PROTOCOL.md §8).")
    parser.add_argument("--instructions", required=True, choices=["ON-1", "ON-2", "ON-3"])
    parser.add_argument("--agent-command")
    parser.add_argument("--budget", type=float)
    parser.add_argument("--binary", type=Path, default=score.default_binary())
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--model", default="sonnet")
    parser.add_argument("--max-turns", type=int, default=MAX_TURNS)
    options = parser.parse_args(arguments)
    try:
        summary = run_round(options)
    except (Refused, assemble.Refused) as error:
        print(f"[onboarding] refused: {error}", file=sys.stderr)
        return 2
    print((options.out / "report.txt").read_text(encoding="utf-8"), end="")
    print(
        f"[onboarding] turns {summary['turns']}, cost {summary['cost_usd']}, wall {summary['wall_s']} s, "
        f"load {summary['load_before']} -> {summary['load_after']}, evidence {options.out}"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
