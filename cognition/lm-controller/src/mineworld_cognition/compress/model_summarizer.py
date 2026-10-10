"""`ModelSummarizer` and `embellish`: optional prose for closed summaries, through the gateway (D-P4-10).

Off by default, and never on the ingestion or retrieval path. `embellish` reads each closed episode's
and chapter's structural text back from the store, asks the `summarize` tier for one sentence of prose,
and stores it beside the text with its cassette key. It touches `prose` and `prose_key` only: a model
never writes, adds or drops a citation (P4-1), and the request holds names, never an id.

Every gateway outcome is handled: `Completed` → the prose is validated (one line, at most 160 bytes, no
run of three digits, no citation delimiter) and stored, or discarded and counted; `Refused` and `Failed`
→ nothing is stored and the structural text stands; no gateway (the tier is unbound) → nothing is asked.
A `CassetteMiss` is a stop, never a fallback: it propagates.
"""

from __future__ import annotations

import re
from dataclasses import dataclass
from typing import Literal, assert_never

from mineworld_sdk.wire.ids import EntityKey

from mineworld_cognition.backend.canonical import cassette_key
from mineworld_cognition.backend.model import CompletionRequest, Message, Sampling
from mineworld_cognition.gateway import Completed, Failed, ModelGateway, Refused
from mineworld_cognition.memory.names import one_line
from mineworld_cognition.memory.store import MemoryStore

PROSE_BYTES = 160
INSTRUCTIONS = (
    "Rewrite this memory as one short sentence of plain prose, in the first person. Keep every name "
    "and quote. Add nothing that is not in it. No numbers, no lists, no brackets."
)
SAMPLING = Sampling(temperature_milli=0, max_output_tokens=96, seed=0)
_DIGITS = re.compile(r"\d{3,}")
"""A run that could be an id; days and times have at most two digits per group."""
_DELIMITER = "#"
"""A citation token's mark (`[#12-40]`): prose may never look like provenance."""

Table = Literal["episodes", "chapters"]


@dataclass(frozen=True)
class EmbellishReport:
    asked: int
    stored: int
    discarded: int
    refused: int
    failed: int
    unbound: bool


class ModelSummarizer:
    """Prose through a gateway already bound to the `summarize` tier (`Router.for_tier`), or `None`."""

    def __init__(self, gateway: ModelGateway | None, seat: EntityKey) -> None:
        self.gateway = gateway
        self.seat = seat

    @staticmethod
    def request(text: str) -> CompletionRequest:
        return CompletionRequest(
            purpose="summarize",
            messages=(Message(role="system", text=INSTRUCTIONS), Message(role="user", text=text)),
            sampling=SAMPLING,
        )


def valid_prose(text: str) -> str | None:
    prose = one_line(text)
    if not prose or len(prose.encode("utf-8")) > PROSE_BYTES:
        return None
    if _DIGITS.search(prose) or _DELIMITER in prose:
        return None
    return prose


async def embellish(
    store: MemoryStore, summarizer: ModelSummarizer, *, limit: int
) -> EmbellishReport:
    """Prose for up to `limit` closed summaries that have none: episodes in order, then chapters."""
    gateway = summarizer.gateway
    if gateway is None:
        return EmbellishReport(0, 0, 0, 0, 0, unbound=True)
    asked = stored = discarded = refused = failed = 0
    for table, key, text in _without_prose(store, limit):
        request = ModelSummarizer.request(text)
        asked += 1
        outcome = await gateway.complete(summarizer.seat, request)
        match outcome:
            case Completed(completion=completion):
                prose = valid_prose(completion.text) if completion.finish == "complete" else None
                if prose is None:
                    discarded += 1
                    continue
                with store.transaction() as db:
                    db.execute(
                        f"UPDATE {table} SET prose = ?, prose_key = ? WHERE {_KEY[table]} = ?",
                        (prose, cassette_key(request), key),
                    )
                stored += 1
            case Refused():
                refused += 1
            case Failed():
                failed += 1
            case _:
                assert_never(outcome)
    return EmbellishReport(asked, stored, discarded, refused, failed, unbound=False)


_KEY: dict[Table, str] = {"episodes": "episode", "chapters": "week"}


def _without_prose(store: MemoryStore, limit: int) -> list[tuple[Table, int, str]]:
    db = store.connection
    episodes: list[tuple[int, str]] = db.execute(
        "SELECT episode, text FROM episodes WHERE closed = 1 AND prose IS NULL "
        "ORDER BY episode LIMIT ?",
        (limit,),
    ).fetchall()
    chapters: list[tuple[int, str]] = db.execute(
        "SELECT week, text FROM chapters WHERE prose IS NULL ORDER BY week LIMIT ?",
        (max(0, limit - len(episodes)),),
    ).fetchall()
    targets: list[tuple[Table, int, str]] = [("episodes", n, text) for n, text in episodes]
    targets.extend(("chapters", week, text) for week, text in chapters)
    return targets
