"""The cognition process of the corrected-memory scenario: `python -m mineworld_cognition`, with a scripted model.

`tools/cli/tests/correction_memory.rs` (`pr-playable-dialogue-memory.md` §5 C4) runs this file as a process with the
workspace's interpreter and `--config FILE`, exactly as `milestone_d_cognition.py` is run: everything but the model is
the shipped controller. The configuration says `recording.mode = "scripted"`, so no adapter is built and no network
client exists.

The model is `corrects`, a pure function of the request that answers **only from the records the request shows**:

- a line I am told that is not a question is acknowledged ("Noted."), citing nothing;
- a question is answered from the lines under WHAT I REMEMBER that the asker said to me: the newest line that states
  where their bag is, and the newest line that states their locker code, each cited by its own token. A fact that
  is not shown is not known: with neither shown it says so and cites nothing.

The words it reads are the scenario's (`correction_memory.rs` `Lines`): "my locker code is <CODE>" and "my bag is
[now] at the <PLACE>". "Newest" is the larger Event ID of the shown token, never the order of the lines, so a
correction is used only if the memory the controller assembled holds it. It has no other source: no save, no log, no
store, no environment.
"""

from __future__ import annotations

import re
import sys

from milestone_d_cognition import REMEMBERED, answer, cited, speaker_of, talk_to

from mineworld_cognition.__main__ import main
from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest
from mineworld_cognition.backend.scripted import ScriptedBackend
from mineworld_cognition.controller.prompt import HEARD_CLOSE, HEARD_OPEN

CODE = re.compile(r"\bmy locker code is (?P<code>[A-Z0-9-]+)")
PLACE = re.compile(r"\bmy bag is (?:now )?at the (?P<place>[a-z]+)")


def corrects(request: CompletionRequest) -> Completion | BackendFailure:
    """Acknowledge a statement; answer a question with the newest shown place and code, citing both."""
    user = request.messages[1].text if len(request.messages) > 1 else ""
    speaker = speaker_of(user)
    if speaker is None or speaker not in talk_to(user):
        return answer({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
    if not _heard(user).rstrip().endswith("?"):
        return _say(speaker, "Noted.", [])
    place = _newest(user, speaker, PLACE, "place")
    code = _newest(user, speaker, CODE, "code")
    if place is None and code is None:
        return _say(speaker, "I don't remember you telling me about that.", [])
    parts: list[str] = []
    recalls: list[str] = []
    if place is not None:
        parts.append(f"your bag is at the {place[0]}")
        recalls.append(place[1])
    if code is not None:
        parts.append(f"your locker code is {code[0]}")
        if code[1] not in recalls:
            recalls.append(code[1])
    sentence = " and ".join(parts)
    return _say(speaker, sentence[0].upper() + sentence[1:] + ".", recalls)


def _say(to: str, words: str, recalls: list[str]) -> Completion:
    return answer(
        {
            "act": "say",
            "to": to,
            "words": words,
            "offer": None,
            "recalls": [cited(t) for t in recalls],
        }
    )


def _heard(user: str) -> str:
    """The words of the line that triggered this consult, as WHAT JUST HAPPENED quotes them."""
    happened = user.split("WHAT JUST HAPPENED", 1)[-1]
    return happened.split(HEARD_OPEN, 1)[-1].split(HEARD_CLOSE, 1)[0]


def _newest(
    user: str, speaker: str, pattern: re.Pattern[str], group: str
) -> tuple[str, str] | None:
    """(value, token) of the shown line said to me by `speaker`, matching `pattern`, with the largest Event ID."""
    best: tuple[int, str, str] | None = None
    section = user.split("WHAT I REMEMBER", 1)[-1].split("WHAT JUST HAPPENED", 1)[0]
    for line in section.splitlines():
        remembered = REMEMBERED.match(line.strip())
        if remembered is None or remembered["who"] != speaker:
            continue
        found = pattern.search(remembered["words"])
        if found is None:
            continue
        last = int(remembered["token"].strip("[#]").split("-")[-1])
        if best is None or last > best[0]:
            best = (last, found[group], remembered["token"])
    return None if best is None else (best[1], best[2])


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:], scripted=ScriptedBackend(corrects)))
