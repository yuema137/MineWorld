# PR QS256 — the SDK carries a superseded observation's facts into the next frame read

## DESIGN FROZEN — operator ruling QS256-1, 2026-10-18

Design revision: QS256-r1 (sections 1–5 below are frozen; section 6 is the live ledger)
Approved by / evidence: the operator chose option 1 of QS256-1, "Carry them into the next frame", on
2026-10-18 in the primary session. The operator ruled the approach; the details below (the API shape, the
bound and its value, what happens at the bound, the tests) are the implementer's, within that ruling.
Implementation base: `origin/main` at `96b41db0` (branch `fix/sdk-carry-superseded-facts`).
Parent: [`pr-s27c-player-library.md`](pr-s27c-player-library.md) D-c-19 and D-c-25 (the CI finding this
PR answers); [`step-27-agent-players.md`](step-27-agent-players.md).
Execution contract: section 5.
Lifecycle: FROZEN

## 1. Problem

`SeatSession` keeps only the newest observation (newest wins, `server/PROTOCOL.md` §8). An observation's
`events` are the facts its observer learned **since its previous frame** (§5.2). A consumer that reads
`changed(since)` after two frames arrived gets the second and never sees the first — so the first frame's
facts are gone. With the reliable `perceived` stream (§5.8) that does not matter. Without it — a world
with no save, where `since: null` is refused `cursor_unavailable` (D-c-19) — the agent library's inbox is
fed from each observation's own `events`, and a slow runner loses facts a human client would have seen.
D-c-25: on `python (windows-2025)` the visitor's `walk-ended` rode in exactly one superseded frame, and
`test_two_handles.py::test_the_two_seats_share_no_reference_name_or_event` failed "my walk's end is my
fact".

## 2. Contract change (SDK, Python only)

- **`SeatSession.changed(since)`** returns the newest observation whose `seq` is greater than `since`, and
  its `observation.events` are **every fact of every observation after `since`** this session received,
  oldest first: the facts of the frames it superseded, in frame order, then the newest frame's own. With
  `since=None` that is every frame of the connection still held (see the bound).
- **`SeatSession.folded(since)`** (new, synchronous) is the same answer without waiting: the newest
  observation with the facts after `since` folded in, or `None` before the first. `ResumingSeat.changed`
  uses it, so the frame it hands out and `newest_through` are read together and nothing that arrives
  between the wake-up and the read is lost.
- **`SeatSession.newest`** is unchanged: the wire's newest frame as received (or as a `delta` made it),
  with its own `events` only. A `delta` still applies to that raw frame (§5.3).
- **`SeatSession.carried_dropped`** (new): how many carried facts were dropped at the bound on this
  connection.
- **`ResumingSeat.changed(after)`** hands out the folded frame. Across connections nothing is carried: a
  new connection starts a new session and a whole observation (§4.2), and the facts of a dropped connection
  are the reliable stream's (or lost, as before, with the socket).

The server protocol does not change: no frame, field, or server behaviour is touched.

## 3. Invariants

- **INV-QS256-1 — no fact lost below the bound.** Every fact of every observation frame received on a
  connection reaches exactly one `changed`/`folded` answer of a consumer that passes the `seq` of the frame
  it last read as `since`, unless more than `CARRY_BOUND` facts were superseded unread.
- **INV-QS256-2 — no duplicate.** A carried fact is folded only into answers for `since` below the `seq` of
  the frame that carried it, and is discarded when a call names a `since` at or past that frame. A consumer
  that passes its last frame's `seq` never sees a fact twice.
- **INV-QS256-3 — order kept.** Folded facts are in frame order, and in each frame's own order, followed by
  the newest frame's facts.
- **INV-QS256-4 — bounded, and the loss counted.** At most `CARRY_BOUND` (512, the agent inbox's
  `INBOX_BOUND`: a consumer that falls further behind would drop them from its inbox anyway) carried facts
  are kept. Past it the **oldest** carried facts are dropped and counted in `carried_dropped`; the newest
  frame's own facts are never dropped. The bound is a constructor argument (`carry_bound`, 1 to 512) so a
  test can reach it.
- **INV-QS256-5 — the wire frame is untouched.** `newest`, the delta base, `seq`, `acted_through` and every
  field but `observation.events` of the answer are those of the newest frame received.
- **INV-QS256-6 — no protocol change.** Nothing in `server/` or `server/PROTOCOL.md` changes.

Non-goals: carrying facts across connections; deduplicating by event id (the server never repeats a fact
across one connection's frames, §5.2); surfacing `carried_dropped` in the agent library's inbox status
(a consumer reaching the bound has already lost facts from its inbox; recorded as a possible follow-up).

## 4. Commit plan

### C1 — `SeatSession` carries superseded facts; `ResumingSeat` hands out the folded frame

1. Goal: INV-QS256-1…6. One commit: the carry and its only in-SDK consumer change together.
2. Scope: `sdk/python/src/mineworld_sdk/session.py` (`CARRY_BOUND`, `_carried`, `folded`, `changed`,
   `carried_dropped`, the module docstring's "Newest wins" paragraph), `sdk/python/src/mineworld_sdk/resuming.py`
   (`ResumingSeat.changed`), `sdk/python/tests/test_session_state.py` (new tests). Unchanged:
   `clients/agent` code (it already reads `seen.frame.observation.events`), the cognition seat, the server.
3. Implementation:
   - [x] `route(ObservationFrame)`: before replacing a held frame, append its `(seq, event)` pairs to
     `_carried`; trim the oldest past the bound, counting each in `_carried_dropped`.
   - [x] `folded(since)`: discard carried pairs with `seq <= since`; return the newest frame with the rest
     prepended to its `events` (the frame itself when nothing is carried).
   - [x] `changed(since)` returns `folded(since)`.
   - [x] `ResumingSeat.changed` builds its `Seen` from `session.folded(since)`.
4. Validation:
   - Unit (deterministic, `Script` socket): two frames delivered before the read → both frames' facts
     exactly once, in order; a third read after `since` → no repeat; a `delta` on a superseded frame keeps
     the held frame's facts and adds the delta's; the bound → the oldest dropped, counted, the newest frame's
     own kept; `ResumingSeat` over a scripted server folds too (two frames before the handle reads).
   - Integration: `clients/agent/tests/test_two_handles.py` (real server, no save — D-c-19) passes, and is
     not weakened; the whole SDK and agent suites pass.
   - Mutation: drop the superseded frame's facts (do not record them in `route`) → the new tests go red.
5. Acceptance: the tests above pass; under the mutation the two-frames test and the `ResumingSeat` test
   fail naming the missing facts; `ruff`, `pyright` clean; CI green on the exact head on all three platforms.
6. Failure and edge cases: a stale frame (`seq` not above the held one) is ignored and carries nothing (as
   before); `since=None` folds everything held; the bound reached; a delta that does not apply still ends
   the session `ProtocolViolation`.
7. Commands: `uv run pytest sdk/python/tests -q`, `uv run pytest clients/agent/tests -q`, `ruff`, `pyright`
   (the repository's Python checks), the mutation by a scratch patch reverted after the run.
8. Commit boundary: SDK code and tests only.

### C2 — records

- [x] `docs/DECISIONS.md` ARC-150; the D-c-19 / D-c-25 notes of `pr-s27c-player-library.md` updated
  (D-c-25 closed by this ruling).

## 5. Execution contract

Authorized by the operator's ruling as relayed by the primary session: branch, commits, push, PR, CI
repair. Not authorized: merge. Validation budget: the SDK and agent Python suites locally (light pytest),
CI on the exact head. Stop condition: PR ready for operator review with per-job CI evidence.

## 6. Live ledger

See section 7 (evidence), kept current.

## 7. Evidence

- **C1** `d6ee41f2`. Implementation as in §4. Bounded detail decided here: `changed` itself prunes and
  folds (no separate "read" mark), so a consumer's `since` is the only acknowledgement; `ResumingSeat`
  reads `folded` after the wake-up rather than the frame that woke it, so a frame arriving in between is
  the answer and carries the earlier one's facts.
  - Validation (macOS, local): `pytest sdk/python` 112 passed (11.7 s, real binary built from this head
    via `with-slot.sh cargo build -p mineworld-cli`); `pytest clients/agent` 148 passed (92.8 s);
    `ruff check`, `ruff format --check`, `pyright sdk/python clients/agent` clean.
    `test_two_handles.py::test_the_two_seats_share_no_reference_name_or_event` 10 of 10 passed (≈1.35 s
    each), test unchanged; `pytest cognition/lm-controller` 471 passed, 2 skipped, 52 deselected, 1 xfailed.
  - New tests: `test_session_state.py::test_two_frames_before_the_read_hand_on_both_frames_facts_once_in_order`,
    `::test_a_delta_on_a_superseded_frame_keeps_its_facts_and_adds_its_own`,
    `::test_past_the_carry_bound_the_oldest_facts_are_dropped_and_counted`,
    `test_resuming.py::test_a_seat_without_a_stream_hands_on_the_facts_of_a_frame_it_superseded`.
  - Mutations (scratch edit, file restored and compared byte-identical after each):
    M1 drop the superseded frame's facts (`_carry` not called) → all 4 new tests FAIL
    (`assert ['3'] == ['1', '2', '3']`, `assert ['9'] == ['2', '9']`); M2 `ResumingSeat` hands
    `session.newest` instead of the folded frame → the `test_resuming` test FAILS; M3 no pruning at
    `since` → all 4 FAIL (facts handed on twice). Unmutated: 60 passed.
  - Review: every consumer of `changed` audited — `clients/agent` `Handle._watch`/`_take` (the D-c-19
    path, now complete), the cognition seat (`controller/seat.py`; it ingests the `perceived` stream
    only, never `Observation.events`, `memory/ingest.py`), `ResumingSeat`. `newest` and the delta base
    untouched (INV-QS256-5). No `server/` file changed (INV-QS256-6).
- **C2** `94783c99`. ARC-150; `check_decision_ids.py`: 168 ids, all distinct. D-c-19 and D-c-25 notes
  updated in `pr-s27c-player-library.md`.
- CI on the exact head: see the PR.
