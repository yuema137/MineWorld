"""The cognition process Milestone D's live test starts: `python -m mineworld_cognition`, with a scripted model.

`tools/cli/tests/milestone_d.rs` (S10 P7, `pr-s10-p7-milestone-d.md` D-P7-1) runs this file as a process, with the
workspace's interpreter and `--config FILE`, exactly as an operator runs `python -m mineworld_cognition`. Everything
but the model is the shipped controller: the assignment, the seat runner, the memory store, the triggers, the consult
engine, the decision log. The configuration says `recording.mode = "scripted"`, so no adapter is built and no network
client exists (`build_router`).

The model is `remembers`, a pure function of the request (step-17 §3.11.3: "it answers with a sentence that quotes the
newest remembered line and cites its Event ID"). It reads only what the request shows a model: the people it may talk
to, and the lines under WHAT I REMEMBER with their citation tokens. It quotes the **earliest** line it is shown as said
to it, and cites that line's token, so a reply in the 3D view can only mention the 2D conversation if the memory the
controller assembled holds it. It has no other source: no save, no log, no store.
"""

from __future__ import annotations

import json
import re
import sys

from mineworld_cognition.__main__ import main
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest, Usage
from mineworld_cognition.backend.scripted import ScriptedBackend

TALK_LINE = "I can talk to: "
REMEMBERED = re.compile(
    r"^(?:Day \d+ \d\d:\d\d: )?(?P<who>.+?) said to me: «(?P<words>[^»]*)» (?P<token>\[#[0-9-]+\])$"
)
"""A remembered line addressed to me (`memory/render/conversation.py`, `retrieve.py`): `Vera said to me: «…» [#12]`,
a `Day 1 08:03: ` prefix allowed. A line cut with an ellipsis has no closing `»` and is not quoted."""


def remembers(request: CompletionRequest) -> Completion | BackendFailure:
    """Answer the person who just spoke to me, if I can talk to them, with the earliest line I remember them
    telling me, citing it; else nothing."""
    user = request.messages[1].text if len(request.messages) > 1 else ""
    speaker = _speaker(user)
    lines = [(words, token) for who, words, token in _remembered(user) if who == speaker]
    if speaker is None or speaker not in _talk_to(user) or not lines:
        return _answer({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
    words, token = lines[0]
    return _answer(
        {
            "act": "say",
            "to": speaker,
            "words": f'I remember you told me: "{words}"',
            "offer": None,
            "recalls": [token],
        }
    )


def _speaker(user: str) -> str | None:
    """The label WHAT JUST HAPPENED names as having spoken to me (`prompt._heard`)."""
    happened = user.split("WHAT JUST HAPPENED", 1)[-1]
    found = re.search(r"^(?P<who>.+?) said to me: ", happened.strip(), re.MULTILINE)
    return None if found is None or found["who"] == "Someone" else found["who"]


def _talk_to(user: str) -> list[str]:
    for line in user.splitlines():
        if line.startswith(TALK_LINE):
            listed = line.removeprefix(TALK_LINE)
            return [] if listed == "nobody" else [label.strip() for label in listed.split(",")]
    return []


def _remembered(user: str) -> list[tuple[str, str, str]]:
    """(who, words, token) of every line said to me, earliest Event ID first."""
    found: list[tuple[int, str, str, str]] = []
    for line in user.split("WHAT I REMEMBER", 1)[-1].split("WHAT JUST HAPPENED", 1)[0].splitlines():
        match = REMEMBERED.match(line.strip())
        if match is not None:
            first = int(match["token"].strip("[#]").split("-")[0])
            found.append((first, match["who"], match["words"], match["token"]))
    return [(who, words, token) for _, who, words, token in sorted(found)]


def _answer(choice: dict[str, object]) -> Completion:
    text = json.dumps(choice)
    return Completion(
        text=text,
        finish="complete",
        usage=Usage(input_tokens=0, output_tokens=len(text) // 4, estimated=True),
    )


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], scripted=ScriptedBackend(remembers)))
