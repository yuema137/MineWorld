"""An outside agent on the `visitor` seat, through the MCP adapter, for the corrected-memory scenario.

`tools/cli/tests/correction_memory.rs` (`pr-playable-dialogue-memory.md` §4.13, §5 C4) starts this file as a process
with the workspace's interpreter. It is the agent half of the scenario's rejoin: the player's protocol client has
left the seat, and an outside agent takes it through `python -m mineworld_agent mcp` (stdio, the official MCP client,
S27-d's `adapter`), talks to the LM-driven barista with the same `talk` intent a 2D or 3D client sends, and leaves.
No model is asked here: what to say is the test's, line by line.

The Rust test drives it over this process's standard input and output, one JSON object per line:

```text
start-up               join `visitor`, find the person labelled `barista`, walk to her
  → {"ready": {"took_over": …, "barista": "E3"}}
{"say": "<words>"}     act `say` to her
  → {"said": "<outcome kind>"}
{"heard": true}        every line she has said to me so far, in order
  → {"heard": ["…", …]}
{"leave": true}        leave the seat, then exit
  → {"left": true}
```

The credential is the server's invite, passed in `MINEWORLD_INVITE`; it is handed to the adapter as its grant
variable (the adapter accepts either, `mineworld_agent.__main__`) and never printed.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import sys
from pathlib import Path
from typing import Any, cast

from mcp import ClientSession
from support import run_default
from test_play_mcp import Transcript, adapter, call

BARISTA = "barista"
WALK_PATIENCE_S = 20
HEARD_WAIT_S = 1
EVENTS_PAGE = 20
SESSION_S = 600.0
"""The longest the agent's whole session may last: the scenario's turns, each bounded by its consult."""


def _emit(answer: dict[str, Any]) -> None:
    sys.stdout.write(json.dumps(answer) + "\n")
    sys.stdout.flush()


def _barista(view: dict[str, Any]) -> str:
    for person in view["here"]["people"]:
        if any(label.get("text") == BARISTA for label in person.get("labels", [])):
            return str(person["ref"])
    raise AssertionError(f"no person labelled {BARISTA!r} in the visitor's view: {view['here']}")


class _Inbox:
    """The lines the barista said to me, read from the adapter's inbox page by page, each fact once."""

    def __init__(self, barista: str) -> None:
        self.barista = barista
        self.after: str | None = None
        self.lines: list[str] = []

    async def read(self, session: ClientSession, transcript: Transcript, handle: str) -> list[str]:
        while True:
            arguments: dict[str, Any] = {"handle": handle, "limit": EVENTS_PAGE}
            if self.after is not None:
                arguments["after"] = self.after
            page = await call(session, transcript, "events", arguments)
            for event in cast(list[dict[str, Any]], page["events"]):
                words = cast(dict[str, Any] | None, event.get("words"))
                if event["event_type"] == "spoke" and words is not None:
                    if words.get("by") == self.barista:
                        self.lines.append(str(words["text"]))
                self.after = str(event["ref"])
            if page["next"] is None:
                return list(self.lines)
            self.after = str(page["next"])


async def play(server: str, state: Path, log: Path) -> None:
    credential = os.environ["MINEWORLD_INVITE"]
    transcript = Transcript()
    async with adapter(server, state, credential, log) as session:
        joined = await call(
            session, transcript, "join", {"seat": "visitor", "nickname": "outside-agent"}
        )
        handle = str(joined["handle"])
        barista = _barista(joined["view"])
        await call(
            session,
            transcript,
            "act",
            {"handle": handle, "act": {"kind": "walk_to", "to": {"person": barista}}},
        )
        await call(
            session,
            transcript,
            "wait",
            {"handle": handle, "until": {"kind": "walk_ended"}, "timeout_s": WALK_PATIENCE_S},
        )
        _emit({"ready": {"took_over": joined["took_over"], "barista": barista}})
        inbox = _Inbox(barista)
        while line := await asyncio.to_thread(sys.stdin.readline):
            command = json.loads(line)
            if "say" in command:
                act = {"kind": "say", "to": barista, "words": command["say"]}
                answer = await call(session, transcript, "act", {"handle": handle, "act": act})
                _emit({"said": answer["outcome"]["kind"]})
            elif "heard" in command:
                await call(
                    session,
                    transcript,
                    "wait",
                    {"handle": handle, "until": {"kind": "any_event"}, "timeout_s": HEARD_WAIT_S},
                )
                _emit({"heard": await inbox.read(session, transcript, handle)})
            elif "leave" in command:
                await call(session, transcript, "leave", {"handle": handle})
                _emit({"left": True})
                return


def main() -> int:
    parser = argparse.ArgumentParser(description="An outside agent on the visitor seat, over MCP.")
    parser.add_argument("--server", required=True, help="the server's ws:// URL")
    parser.add_argument("--state", required=True, type=Path, help="the adapter's state directory")
    parser.add_argument("--log", required=True, type=Path, help="where the adapter's stderr goes")
    arguments = parser.parse_args()
    run_default(play(arguments.server, arguments.state, arguments.log), timeout_s=SESSION_S)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
