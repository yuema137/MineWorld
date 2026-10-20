"""`lm-summarise` determinism and replay (pr-fg-c §§5.6, 7.3; AFGc-4, AFGc-5, AFGc-7 (b)).

A scripted run is recorded through `RecordingBackend`, then replayed from a null cursor through
`ReplayBackend`. The committed cassette `tests/cassettes/lm-summarise.jsonl` is a drift guard: it replays the
same stream to a recorded dump digest. Re-recording is a deliberate change: run this module with
`LMS_RECORD=1` and review the diff (QFGc-3). No model is reached (QS10-19).
"""

from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path

import pytest
from memory_fixtures import seat_view
from mineworld_sdk.wire.contract import PerceivedEvent
from retention_driver import feed, frames
from retention_stream import NAMES, PEOPLE, stream
from support import run
from test_lm_summarise import GENEROUS, boundary_gateway, gateway, retention, scripted

from mineworld_cognition.memory.retention.absorb import absorb
from mineworld_cognition.memory.retention.assisted import SummaryUnavailable
from mineworld_cognition.memory.retention.host import run_pass
from mineworld_cognition.memory.retention.prose import KnownNames
from mineworld_cognition.memory.retention.registry import PolicyRegistry
from mineworld_cognition.memory.retention.selection import (
    MemoryPolicyChoice,
    Retention,
    resolve,
)
from mineworld_cognition.record import Cassette, CassetteMiss, RecordingBackend, ReplayBackend

CASSETTE = Path(__file__).resolve().parent / "cassettes" / "lm-summarise.jsonl"
SEED = 11
DAYS = 12
DAY = 86_400
DIGEST = "d08c81673f2631bcb831495c44f8bb9c930853ad4520d93da97178465dde8f19"
"""AFGc-4 (b): the sha256 of the committed cassette's replayed dump (ledger §14.4), set at C4. Reset by
FG-b2 (pr-fg-b2-size-trigger.md §12.4 F-FGb2-4): schema v3 adds `trigger` and `cause` to every pass-log row
and `meta.schema` reads 3. Reset by S28-a (pr-s28a-memory-home.md AC-a-13; was `c3624e03…a106a4`): the dump
is schema 4's positional seat view (`ARC-101`). Re-pinned only after the replay ran with zero `CassetteMiss`
and its sections equalled the golden captured from the schema-3 code (`test_home_golden.py`'s `fgc` arm). The
cassette is unchanged: no request moved. Reset by FG-d (pr-p6c-fgd-memory-coherence.md §9, F-FGd-1):
`consolidate`'s keepsakes keep each person's first exchanges as records, so the dump holds them; the cassette is
unchanged and replays without a miss. Re-pinned on schema 4 (D-P6c-3, was `2163d85b…6df00a`) from scratch CI
run 39125662718, and only with `test_home_golden.py`'s `fgc` arm equal to the golden re-captured from the
schema-3 code with FG-d. Re-pinned for F-FGd-2's host protection (D-P6c-5, was `df105650…296020`): answers
that would drop a direct conversation are overridden, so the store keeps those groups; scratch CI run
39186640715, the cassette unchanged and no `CassetteMiss`. Reset by S28-b (pr-s28b-retention-on-seat-views.md
C2; was `a29ed8d5…53251a` after the merge of main): schema 5 adds the forgotten ranges' attribution and the pass
log's `prose_contradictions`. Re-pinned only after the replay ran with zero `CassetteMiss` and
`test_home_golden.py`'s `fgc` sections still equalled the golden; S28-b's prose check (C3) leaves it unchanged.
Reset by S28-d (pr-s28d-anchored-scope.md C3; was `a4237686…5e9ca4`): schema 6's `meta.schema` line and each
held fact's `access_anchor` lines; replayed with zero `CassetteMiss`, the cassette unchanged, and
`test_home_golden.py` still equal to the golden."""


def _facts() -> list[PerceivedEvent]:
    return list(stream(SEED, DAYS))


def _record(
    path: Path, model: str = "scripted", facts: list[PerceivedEvent] | None = None
) -> bytes:
    backend = RecordingBackend(scripted(), path, binding="local", model=model)
    kept = retention(gateway(backend, **GENEROUS), calls_per_sim_day=24)
    with seat_view() as store:
        feed(store, frames(_facts() if facts is None else facts, 256), kept)
        dump = store.dump()
    run(backend.aclose())
    return dump


def _replayed(path: Path, size: int, facts: list[PerceivedEvent] | None = None) -> bytes:
    replay = ReplayBackend(Cassette.load(path))
    kept = retention(gateway(replay, **GENEROUS), calls_per_sim_day=24)
    with seat_view() as store:
        feed(store, frames(_facts() if facts is None else facts, size), kept)
        return store.dump()


def _consolidated() -> Retention:
    return resolve(MemoryPolicyChoice(policy="consolidate"), PolicyRegistry.installed())


@pytest.mark.skipif(os.environ.get("LMS_RECORD") != "1", reason="records the committed cassette")
def test_record_the_committed_cassette(tmp_path: Path) -> None:
    # C4's one recording: run deliberately with LMS_RECORD=1, then commit the file and the digest.
    path = tmp_path / "lm-summarise.jsonl"
    _record(path)
    CASSETTE.write_bytes(path.read_bytes())


def test_a_recording_replays_byte_identically_in_every_framing(tmp_path: Path) -> None:
    # AFGc-4 (a): a scripted run recorded to a file, then rebuilt from a null cursor through the replay, in
    # frames of 256 and as one frame.
    path = tmp_path / "lm-summarise.jsonl"
    recorded = _record(path)
    assert _replayed(path, 256) == recorded
    assert _replayed(path, len(_facts())) == recorded


def test_a_recording_of_a_prefix_replays_byte_identically_in_frames_of_one(tmp_path: Path) -> None:
    # AFGc-4 (a), frames of 1: the first two simulated days, which hold assisted passes (the full stream
    # in frames of 1 is too slow for a local run, ledger §14.6).
    prefix = [event for event in _facts() if event.at < 2 * DAY]
    path = tmp_path / "lm-summarise.jsonl"
    recorded = _record(path, facts=prefix)
    assert _replayed(path, 1, facts=prefix) == recorded


def test_the_committed_cassette_replays_to_the_recorded_digest() -> None:
    # AFGc-4 (b): the committed cassette holds every request of the 12-day stream and replays to the digest
    # recorded in the ledger. A changed request template or key scheme is a CassetteMiss, not a silent drift.
    assert DIGEST, "the digest is recorded at C4, after the one recording"
    assert hashlib.sha256(_replayed(CASSETTE, 256)).hexdigest() == DIGEST


def test_no_committed_answer_names_a_known_name_of_its_stream() -> None:
    # AC-b-5 (FU-S28-1's "states which cassettes change"): the committed cassette's answers are FG-c's ten fixed
    # sentences, and the stream names only its twelve people (its places are never named), so the host's name
    # check refuses no recorded answer: the replayed digest above is the one C2 pinned, unchanged by C3.
    known = KnownNames(dict(zip(PEOPLE, NAMES, strict=True)))
    summaries: list[str] = []
    for line in CASSETTE.read_text(encoding="utf-8").splitlines():
        entry = json.loads(line)
        if "completion" in entry:
            summaries.append(str(json.loads(entry["completion"]["text"])["summary"]))
    assert len(summaries) == 179  # the file's 180 lines less its header
    assert [s for s in summaries if known.found(s)] == []


def test_a_missing_recording_is_a_stop_and_the_pass_rolls_back() -> None:
    # AFGc-4 (c): a stream the cassette does not hold misses on its first assisted pass. CassetteMiss
    # propagates out of the pass, and the store is exactly as it was before that pass.
    other = list(stream(SEED + 1, 3))
    replay = ReplayBackend(Cassette.load(CASSETTE))
    kept = retention(boundary_gateway(replay, **GENEROUS), calls_per_sim_day=24)
    with seat_view() as store:
        for start in range(0, len(other), 256):
            batch = other[start : start + 256]
            run(absorb(store, batch, batch[-1].id, _consolidated()), timeout_s=120)
        before = store.dump()
        with pytest.raises(SummaryUnavailable) as raised:
            # The next simulated day: its pass has not run yet, so the pass itself is what misses.
            run(run_pass(store, kept, (other[-1].at // DAY + 1) * DAY), timeout_s=120)
        # FGc-6: the stop names the boundary's error, and the replay's miss is its original cause.
        assert isinstance(raised.value.__cause__, CassetteMiss)
        assert store.dump() == before


def test_a_recording_of_another_model_leaves_no_provider_name_in_the_store(tmp_path: Path) -> None:
    # AFGc-7 (b): the model's name is the cassette's, never the store's: a store built through a recording
    # whose model is `qwen-test` holds no `qwen`.
    path = tmp_path / "lm-summarise.jsonl"
    dump = _record(path, model="qwen-test")
    assert b"qwen" not in dump
