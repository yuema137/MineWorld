"""The scripted price clerk: a pure function of the request, the model of the LM clerk's tests (S31-d §9 C6, C7).

It reads only what the request shows a model: the `TO FILL` line's F-number and the lines of `MY PRICE BOOK`. For
each kind with at least one sale shown it posts the median of the sales (an even count takes the floor of the two
middle values' mean), resting on every note shown for that kind. A request with nothing to fill, or a book with no
sale, is answered `nothing`. No clock, no randomness, no network: recording it costs nothing.
"""

from __future__ import annotations

import json
import re

from mineworld_cognition.backend.model import BackendFailure, Completion, CompletionRequest, Usage

TO_FILL = re.compile(r"^(?P<number>F\d+)\. publish-notice: ", re.MULTILINE)
KIND = re.compile(r"^(?P<label>.+) \((?P<notes>\d+) notes\): sales (?P<sales>[^;]+); asks .*$")


def reporter(request: CompletionRequest) -> Completion | BackendFailure:
    """The bulletin the request's book supports, or nothing."""
    user = request.messages[1].text if len(request.messages) > 1 else ""
    found = TO_FILL.search(user)
    lines: list[dict[str, object]] = []
    if found is not None and "MY PRICE BOOK" in user:
        book = user.split("MY PRICE BOOK", 1)[1].split("WHAT I REMEMBER", 1)[0]
        for line in book.splitlines()[1:]:
            kind = KIND.match(line.strip())
            if kind is None or kind["sales"] == "none":
                continue
            sales = [int(price) for price in kind["sales"].split(", ")]
            lines.append(
                {"item": kind["label"], "price": median(sales), "basis": int(kind["notes"])}
            )
    if found is None or not lines:
        return _answer({"act": "nothing", "to": None, "words": "", "offer": None, "recalls": []})
    return _answer(
        {
            "act": "fill",
            "to": None,
            "words": "",
            "offer": None,
            "recalls": [],
            "fill": {"offer": found["number"], "notice": {"lines": lines}},
        }
    )


def median(values: list[int]) -> int:
    ordered = sorted(values)
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) // 2


def _answer(choice: dict[str, object]) -> Completion:
    text = json.dumps(choice)
    return Completion(
        text=text,
        finish="complete",
        usage=Usage(input_tokens=0, output_tokens=len(text) // 4, estimated=True),
    )
