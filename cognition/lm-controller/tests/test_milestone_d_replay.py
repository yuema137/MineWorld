"""Milestone D's recorded half: the local model's run, replayed from its decision log (S10 P7, D-P7-1; AP7-4, AP7-7).

`tools/cli/tests/milestone_d.rs` (`record_the_milestone_d_cassette`) ran step-17 §3.14.2's scenario once against the
real server and the real cognition process in `record` mode, with the admitted local model, and kept two files:

```text
tests/cassettes/milestone_d.jsonl               every model call of the run, both processes, in call order
tests/fixtures/milestone_d.decisions.jsonl      Alice's decision log: each consult's inputs and record
```

A live hosted run cannot be replayed by a strict cassette (P6 F-P6-5), so the claims that depend on the model's own
words are checked here, on the recorded completion, by re-running every logged consult offline (`replay_log`, P6
§5.11). No server, no model, no network. The Event IDs are the recorded run's: the 2D line is the trigger of the
consult that heard it, and Alice's 2D reply is the line of her own memory that quotes what that consult said.

AP7-7 is `AC-4` (IC-6): the same replay under a second backend binding, with another name, another address and another
model, reproduces every key and every request, and nothing under `worlds/` is read or written to make the swap.
"""

from __future__ import annotations

import json
import re
from pathlib import Path

from support import run
from test_provider_scan import PATTERN

from mineworld_cognition.config import CognitionConfig, build_router
from mineworld_cognition.controller.log import DecisionRecord, read_decisions, replay_log
from mineworld_cognition.controller.validate import Say
from mineworld_cognition.record import Cassette

TESTS = Path(__file__).resolve().parent
CASSETTE = TESTS / "cassettes" / "milestone_d.jsonl"
LOG = TESTS / "fixtures" / "milestone_d.decisions.jsonl"
FROM_THE_2D_WINDOW = "Hi, I'm new in town. I left a red umbrella here yesterday."
FROM_THE_3D_WINDOW = "Hello again."
TOKEN = re.compile(r"#?(\d+)(?:-(\d+))?")


def _binding(name: str, base_url: str, model: str) -> CognitionConfig:
    """A `replay` configuration whose `social` tier is bound to `name`; budgets wide enough never to refuse."""
    return CognitionConfig.model_validate(
        {
            "recording": {"mode": "replay", "cassette": str(CASSETTE)},
            "tiers": {"social": name, "summarize": "none"},
            "budgets": {"calls_per_wall_hour": 1_000_000, "tokens_per_wall_day": 1_000_000_000},
            "backends": {
                name: {
                    "kind": "openai-compatible",
                    "base_url": base_url,
                    "model": model,
                    "structured_output": "none",
                }
            },
        }
    )


def _consult_on(records: list[DecisionRecord], words: str) -> DecisionRecord:
    found = [record for record in records if record.input.trigger.words == words]
    assert found, (
        f"a consult on {words!r}: triggers were {[r.input.trigger.words for r in records]}"
    )
    return found[-1]


def _cited(record: DecisionRecord) -> set[int]:
    ids: set[int] = set()
    for line in record.input.memory.lines:
        for span in line.citation.ranges:
            ids.update(range(int(span.first), int(span.last) + 1))
    return ids


def _names(recall: str, event: int) -> bool:
    for part in recall.strip("[]").split(","):
        match = TOKEN.fullmatch(part.strip())
        if match is not None:
            first = int(match[1])
            last = int(match[2] or match[1])
            if first <= event <= last:
                return True
    return False


def _two_d_ids(records: list[DecisionRecord]) -> tuple[int, int]:
    """The 2D line (the trigger of the consult that heard it) and Alice's 2D reply (the line of her 3D memory that
    quotes the words that consult said)."""
    heard = _consult_on(records, FROM_THE_2D_WINDOW)
    assert heard.record.outcome == "acted", heard.record
    said = heard.record.proposal
    assert isinstance(said, Say), said
    recalled = _consult_on(records, FROM_THE_3D_WINDOW)
    opening = said.words[:40]
    mine = [
        line
        for line in recalled.input.memory.lines
        if line.text.startswith("I said to") or ": I said to" in line.text
        if opening in line.text
    ]
    assert mine, f"Alice's 3D memory quotes her 2D reply {said.words!r}"
    return int(heard.input.trigger.event), int(mine[0].citation.ranges[0].first)


def test_the_recorded_run_replays_exactly_and_the_3d_reply_was_decided_from_the_2d_lines() -> None:
    # AP7-4: every logged consult re-runs over the recorded cassette to the same keys, outcome and request; the 3D
    # reply was decided from a context holding both 2D lines, first, under what the visitor and Alice said before
    # (P6c, AP6c-2); the model's answer became the words Alice said; and the recorded model *used* those lines: its
    # recalls cite one and its words mention the umbrella (AP6c-3). Before P6c, `qwen3.5:4b` did not (F-P7-5,
    # Q-P7-2), and this clause was printed, not asserted.
    report = run(replay_log(LOG, CASSETTE))
    records = read_decisions(LOG)
    assert report.checked == len(records) >= 2
    assert report.mismatches == ()

    line, reply = _two_d_ids(records)
    recalled = _consult_on(records, FROM_THE_3D_WINDOW)
    assert recalled.record.outcome == "acted", recalled.record
    assert {line, reply} <= _cited(recalled), (
        f"the context of the 3D reply holds the 2D line #{line} and Alice's reply #{reply}: {_cited(recalled)}"
    )
    said_before = {
        int(span.first)
        for memory in recalled.input.memory.lines
        if memory.level == "with"
        for span in memory.citation.ranges
    }
    assert {line, reply} <= said_before, f"the 2D lines are under `with`: {said_before}"
    proposal = recalled.record.proposal
    assert isinstance(proposal, Say), proposal
    request = recalled.record.request
    assert request is not None
    assert (
        json.loads(request.model_dump_json())["payload"]["payload"]["utterance"] == proposal.words
    )
    cites_2d = any(_names(recall, line) or _names(recall, reply) for recall in proposal.recalls)
    umbrella = "umbrella" in proposal.words.lower()
    print(
        json.dumps(
            {
                "line_2d": line,
                "reply_2d": reply,
                "words_3d": proposal.words,
                "recalls_3d": list(proposal.recalls),
                "recalls_cite_a_2d_line": cites_2d,
                "words_mention_the_umbrella": umbrella,
            }
        )
    )
    assert cites_2d, (
        f"the 3D reply's recalls cite the 2D line #{line} or #{reply}: {proposal.recalls}"
    )
    assert umbrella, f"the 3D reply's words mention the umbrella: {proposal.words!r}"


def test_the_backend_is_swapped_without_touching_the_world() -> None:
    # AP7-7 (AC-4, IC-6): one cassette, two bindings — different name, address and model — the same keys and
    # requests. Its mutation (the model's name in the key) makes the second binding miss.
    worlds = TESTS.parents[2] / "worlds"
    before = sorted(
        (path.as_posix(), path.stat().st_mtime_ns) for path in worlds.rglob("*") if path.is_file()
    )
    for config in (
        _binding("local", "http://127.0.0.1:11434/v1", "qwen3.5:4b"),
        _binding("elsewhere", "https://models.example.invalid/v1", "another-model-entirely"),
    ):
        router = build_router(config)
        gateway = router.for_tier("social")
        assert gateway is not None
        report = run(replay_log(LOG, CASSETTE, gateway=gateway))
        assert report.mismatches == (), report
        assert report.checked == len(read_decisions(LOG))
    after = sorted(
        (path.as_posix(), path.stat().st_mtime_ns) for path in worlds.rglob("*") if path.is_file()
    )
    assert before == after


def test_the_decision_log_names_no_provider() -> None:
    # IC-10 for the recorded decision log; `test_provider_scan` covers every cassette's keys and requests.
    Cassette.load(CASSETTE)  # well-formed: the header and every entry parse
    assert PATTERN.findall(LOG.read_text(encoding="utf-8")) == []
