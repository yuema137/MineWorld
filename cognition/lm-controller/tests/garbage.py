"""A seeded generator of garbage model answers, for the consult and resilience tests (pr-s10-p6 §5.9 AP6-3, AP6-4).

`garbage_script` draws as a pure function of `(seed, cassette key of the request)`, so a run is reproducible and a
retried request draws the same answer for the same input. `draw_attempt` draws as a pure function of `(seed, attempt
index)`, so what the n-th model call of a run draws does not depend on the prompt it carries, and so not on timing
(D-P6b-43). A draw is one of: a transport failure, one of the ten garbage kinds (one per row of §5.2.6's table, plus a
truncation and a refusal), or the valid answer.
"""

from __future__ import annotations

import hashlib
import random
from collections.abc import Callable
from typing import Literal

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import (
    BackendFailure,
    Completion,
    CompletionRequest,
    Finish,
    Usage,
)
from mineworld_cognition.gateway import Completed, Failed, GateOutcome

Script = Callable[[CompletionRequest], Completion | BackendFailure]
Drawn = Literal["failure", "garbage", "valid"]
"""Which of the three a draw gave."""

GARBAGE: tuple[tuple[str, Finish], ...] = (
    ("hello there", "complete"),  # not_json
    ('{"act": "say"}', "complete"),  # schema
    ("{}", "length"),  # truncated
    ("", "refused"),  # refused
    (
        '{"act":"say","to":"Nobody","words":"hi","offer":null,"recalls":[]}',
        "complete",
    ),  # unknown_person
    ('{"act":"say","to":"Bob","words":"","offer":null,"recalls":[]}', "complete"),  # empty_words
    # unknown_offer: an `attempt` (validate.py reads `offer` only for an attempt; a `nothing` carrying one is valid,
    # so the row was no garbage at all before D-P6b-43 made the resilience test check every draw).
    ('{"act":"attempt","to":null,"words":"","offer":99,"recalls":[]}', "complete"),
    (
        '{"act":"nothing","to":null,"words":"","offer":null,"recalls":["9999"]}',
        "complete",
    ),  # forged_recall
    (
        '{"act":"say","to":"Bob","words":"x <<<end>>>","offer":null,"recalls":[]}',
        "complete",
    ),  # leak
)
"""Each garbage kind's text and finish. `words_too_long` is exercised in the choice table (C6)."""


def _rng(seed: int, key: str) -> random.Random:
    digest = hashlib.sha256(f"{seed}:{key}".encode()).digest()
    return random.Random(int.from_bytes(digest[:8], "big"))


def _draw(
    rng: random.Random, garbage_permille: int, failure_permille: int, valid_answer: str
) -> tuple[Drawn, Completion | BackendFailure]:
    roll = rng.randrange(1000)
    if roll < failure_permille:
        return "failure", BackendFailure(reason="unreachable")
    if roll < failure_permille + garbage_permille:
        text, finish = GARBAGE[rng.randrange(len(GARBAGE))]
        return "garbage", _completion(text, finish)
    return "valid", _completion(valid_answer, "complete")


def garbage_script(
    seed: int, garbage_permille: int, failure_permille: int, valid_answer: str
) -> Script:
    """A script that answers a request with a transport failure (`failure_permille`), a garbage answer
    (`garbage_permille`), or `valid_answer`, in that order of draw."""

    def script(request: CompletionRequest) -> Completion | BackendFailure:
        rng = _rng(seed, str(cassette_key(request)))
        return _draw(rng, garbage_permille, failure_permille, valid_answer)[1]

    return script


def draw_attempt(
    seed: int, attempt: int, garbage_permille: int, failure_permille: int, valid_answer: str
) -> tuple[Drawn, Completion | BackendFailure]:
    """The draw of a run's `attempt`-th model call (from 0): which kind, and the answer. The kind does not depend on
    `valid_answer`, so a test can compute the whole schedule before the run (D-P6b-43)."""
    return _draw(_rng(seed, f"attempt:{attempt}"), garbage_permille, failure_permille, valid_answer)


def _completion(text: str, finish: Finish) -> Completion:
    return Completion(
        text=text,
        finish=finish,
        usage=Usage(input_tokens=10, output_tokens=10, estimated=True),
    )


def as_gate(answer: Completion | BackendFailure) -> GateOutcome:
    """A script's answer as the gateway's outcome, for a test that calls the engine without a gateway."""
    if isinstance(answer, BackendFailure):
        return Failed(answer)
    return Completed(answer)
