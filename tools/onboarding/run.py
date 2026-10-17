"""Run one onboarding round: assemble a sandbox, start a fresh agent in it, then score and audit what it made.

    python3 tools/onboarding/run.py --instructions ON-1 --agent-command claude-h2 --budget 5 \
        --binary target/debug/mineworld --out EVIDENCE_DIR [--model sonnet] [--max-turns 80]

`tools/onboarding/PROTOCOL.md` §§3, 7 and 8 are the authority. It never runs under CI (INV-S25-6), refuses to start
without a budget and an agent command, and never reads, prints or copies a key: it removes every `*_API_KEY`
variable from the agent's environment, so the agent can only use the operator's own login. Standard library only.
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


class Refused(Exception):
    """The round cannot start as asked."""


def claude_h2(model: str, budget: float, turns: int) -> list[str]:
    """The H2 agent command (PROTOCOL.md §3.2): Claude Code in print mode, with every customisation off, no MCP
    server, no web tool, no saved session, a turn limit and a dollar ceiling. Flags verified 2026-10-16 against
    `claude --help`, Claude Code 2.1.302 (`--max-turns` is accepted though not listed)."""
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
        "--tools",
        TOOLS,
        "--disallowedTools",
        "WebFetch,WebSearch,Agent,Task",
        "--permission-mode",
        "bypassPermissions",
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
    """The agent's environment: the operator's, without any API key (so only the login is used), without the
    variables of a surrounding agent session, with `mineworld` first on a minimal PATH."""
    keep: dict[str, str] = {}
    for name, value in os.environ.items():
        upper = name.upper()
        if upper.endswith("_API_KEY") or upper.startswith(("CLAUDE", "MINEWORLD_")):
            continue
        if upper in {"PATH", "PWD", "OLDPWD", "VIRTUAL_ENV", "UV_PROJECT_ENVIRONMENT"}:
            continue
        keep[name] = value
    system = ["/usr/bin", "/bin", "/usr/sbin", "/sbin"] if os.name != "nt" else []
    keep["PATH"] = os.pathsep.join([str(sandbox / "bin"), *system])
    return keep


def agent_command(spec: str, sandbox: Path, model: str, budget: float, turns: int) -> list[str]:
    if spec == "claude-h2":
        return claude_h2(model, budget, turns)
    words = shlex.split(spec)
    return [
        word.format(sandbox=sandbox, budget=f"{budget:g}", max_turns=turns, python=sys.executable)
        for word in words
    ]


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
    if not 0 < options.max_turns <= MAX_TURNS:
        raise Refused(f"--max-turns must be 1 … {MAX_TURNS} (QS25-3)")
    out: Path = options.out
    if out.exists():
        raise Refused(f"{out} exists; each round writes fresh evidence")
    rubric = score.load_rubric(options.instructions)
    sandbox = Path(tempfile.mkdtemp(prefix="mw-onboard-")) / "sandbox"
    assemble.assemble(sandbox, options.binary.resolve())
    if assemble.leaks(sandbox):
        raise Refused("the sandbox names the repository's path")
    command = agent_command(
        options.agent_command, sandbox, options.model, options.budget, options.max_turns
    )
    out.mkdir(parents=True)
    (out / "prompt.txt").write_text(prompt(options.instructions, sandbox), encoding="utf-8")
    load_before = os.getloadavg() if hasattr(os, "getloadavg") else (0.0, 0.0, 0.0)
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
    load_after = os.getloadavg() if hasattr(os, "getloadavg") else (0.0, 0.0, 0.0)
    shutil.copytree(sandbox / "work", out / "work")
    binary = score.Binary((sandbox / "bin" / options.binary.name).resolve(), [sandbox / "packs"])
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
        "transcript_sha256": hashlib.sha256(transcript.read_bytes()).hexdigest(),
        "worlds": [{"world": s.world, "checks": [c.__dict__ for c in s.checks]} for s in scores],
        "void": [i.__dict__ for i in findings.void],
        "stuck": [i.__dict__ for i in findings.stuck],
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
