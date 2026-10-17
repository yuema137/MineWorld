"""A scripted stand-in for an agent, for the harness's dry run (PROTOCOL.md §8.3): it reads the prompt on stdin,
copies the committed ON-1 fixture into the sandbox's `work/`, validates it with the sandbox's `mineworld`, writes
DONE, and prints a transcript in the agent CLI's stream-JSON shape. It calls no model.

    python3 tools/onboarding/fake_agent.py SANDBOX
"""

from __future__ import annotations

import json
import shutil
import subprocess
import sys
from pathlib import Path

FIXTURE = Path(__file__).resolve().parent / "tests" / "fixtures" / "on1" / "harbour-village"


def emit(event: dict[str, object]) -> None:
    print(json.dumps(event), flush=True)


def main(arguments: list[str]) -> int:
    sandbox = Path(arguments[0])
    sys.stdin.read()
    work = sandbox / "work"
    shutil.copytree(FIXTURE, work / FIXTURE.name)
    command = f"mineworld validate {FIXTURE.name}"
    emit(
        {
            "type": "assistant",
            "message": {
                "content": [
                    {"type": "text", "text": "I follow chapter 02 of the guide."},
                    {"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": command}},
                ]
            },
        }
    )
    done = subprocess.run(
        ["mineworld", "validate", FIXTURE.name], cwd=work, capture_output=True, text=True
    )
    emit(
        {
            "type": "user",
            "message": {
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "t1",
                        "content": done.stdout + done.stderr,
                        "is_error": done.returncode != 0,
                    },
                ]
            },
        }
    )
    (work / "STATUS").write_text("DONE\n", encoding="utf-8")
    emit({"type": "result", "subtype": "success", "num_turns": 1, "total_cost_usd": 0.0})
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
