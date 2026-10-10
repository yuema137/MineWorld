# PR S10-P3b — The SDK's perceived stream: cursor, resume, `lagged`, reconnect

## DESIGN FROZEN 2026-10-10 (primary session)

```text
Design revision:        §§1–12 as committed on docs/p3b-design (PR #149) with this header
Approved by / evidence: the operator accepted every P3b recommendation on 2026-10-10 (Q-P3b-4
                        included); the primary session's answers of 2026-10-10, relayed by the
                        coordinator: DEP-44, Q-P3b-2/3/5/7/8 accepted, Q-P3b-6 recorded for S11.
                        Recorded in §12.1
Implementation base:    main at the start of implementation (exact commit recorded in C0)
Execution contract:     §11
Lifecycle:              FROZEN
```

Scope, invariants, D-P3b-1 … D-P3b-10, DEP-44, §4.5's table, the acceptance criteria AP3b-1 … AP3b-17
and the commit plan are frozen. Progress, evidence, findings and bounded corrections stay writable
(§13). Superseded: `DRAFT — PR DESIGN, NOT FROZEN` (author: S10 P3b design agent, 2026-10-10, worktree
`design-p3b`, planning base `origin/main @ bb62edf`).

**Amendment at freeze (main moved, 2026-10-10).** #142 (`7eed282`) already runs the SDK suite's
coroutines on `asyncio.SelectorEventLoop` on every platform (a per-module `run` in
`test_session_state.py` and `test_real_server.py`), closing F-P5-4's SDK half. P3b therefore no longer
moves that runner. It keeps: one shared `tests/support.py` `run` with a `loop` choice, which its new
modules use (the default loop is needed by AP3b-9, §4.6); and AP3b-14, the asynchronous-connection
guard test, which #142 did not add. Re-audited on `origin/main @ 737e032`.

**Effort:** `mvp0` · **Step:** S10, [`step-17-cognition.md`](step-17-cognition.md) (§3.4.3, §10 IC-1,
§15.2's P3b row, §15.3, §15.8) · **Parent:** [`overall.md`](overall.md) S10.
**Binding siblings:** [`pr-s10-p3-python-sdk.md`](pr-s10-p3-python-sdk.md) (the SDK this extends; its
D-P3-1 … D-P3-11 stand); [`pr-s10-p4-memory.md`](pr-s10-p4-memory.md) (frozen; its R-P3b-1 binds this
PR); `step-12-server.md` §17 (S11-C, merged as `370bb38`; the stream, the refusals, the golden frames)
and §16 (S11-B: seats, holds, `resume`, `take_over`).

---

## 1. Goal, in one paragraph

A Python controller that must not miss a fact can hold a seat for hours and survive what a real
network does to it. Today `SeatSession` asks for the `perceived` stream and keeps every fact in a list
forever, ends with a `ProtocolViolation` on the server's `lagged` refusal, and ends for good on any
dropped socket (§3, F-P3b-1). This PR adds the half of `step-17-cognition.md` §3.4.3 that P3 left out:
a **resuming seat** that rejoins with `resume` and the caller's cursor when the socket drops, when the
server says `lagged`, or when the server restarts; a **perceived stream** that hands each fact to its
single consumer exactly once per process, in ascending order, and resumes from the cursor the
consumer's own durable store reports (P4's R-P3b-1), never from what merely arrived; and typed, terminal
outcomes for everything that must not be retried (`taken_over`, `cursor_unavailable`, a different world
instance, …). Its integration checkpoint is IC-1's live and resumed halves, through the real binary:
the facts a Python client received, across a dropped socket, a client crash and a server restart,
concatenate to exactly `mineworld perceived` over the same save.

## 2. Scope

### 2.1 In scope

- `sdk/python/src/mineworld_sdk/`:
  - `session.py` (one connection): the `lagged` sequence (`refused { lagged }` without a token, then
    `closing { lagged }`) ends the session as `SessionClosed("lagged")`; `perceived` frames are handed
    to a sink the caller gives, after the connection-local order checks (§4.4), instead of being kept
    in a list; the `through` received before the newest observation is recorded with it.
  - `perceived.py` (new): `PerceivedBatch`, `CursorSource`, `CursorCell`, `PerceivedStream`, the order
    checks and the in-process delivery rule (§4.4).
  - `resuming.py` (new): `ResumingSeat`, `ReconnectPolicy`, `Connector`, `Seen`, `SeatLost`,
    `AnswerLost`, `NotConnected`; the reconnect decision table (§4.5) and its backoff.
  - `wire/ids.py`: one ordering helper for `EventId` that never converts an id to a number (§4.4).
  - `__init__.py`: the new public names.
- `sdk/python/tests/`: deterministic session and seat tests over scripted connections; the real-binary
  tests of §6 (AP3b-9 … AP3b-12); `realserver.py` gains a save, a hold, a restart and the
  `mineworld perceived` export; a shared coroutine runner, selector loop by default and the platform
  default on request (§4.6; the selector-loop runner itself landed in #142).
- `sdk/python/README.md`: one short example of a resuming seat with a perceived stream.
- `docs/DECISIONS.md`: **DEP-44**, the reconnect loop built on our side, with `websockets`'
  reconnecting iterator and the retry libraries declined (§4.3; `REUSE_POLICY.md`, `CLAUDE.md` §4
  rule 16).
- Every platform: Linux, macOS, Windows (operator, 2026-10-08; step-17 §15.8's P3b row).

### 2.2 Not in scope, and where it goes

| Not here | Why | Where |
| --- | --- | --- |
| Any Rust, server, protocol or golden-frame change | The protocol is S11's (ruling 1). P3b is a far-side client of what S11-C landed. A gap found in the protocol is a finding for S11 (F-P3b-2), never an edit here | S11 |
| Memory, ingestion, the durable cursor's storage | P4 (frozen): `ingest(store, facts, through)` and `MemoryStore.cursor()` | P4 |
| Policy over seats: whether a restarted cognition process takes its seat over, how slowly an LM seat retries a seat a human holds (R-S11-4, `AC-5`), what to do on `world_changed` | Policy belongs to the controller; the SDK supplies the mechanism and typed outcomes | P6 |
| Triggers, the decision loop, the pre-submit check that reads `Seen` | step-17 §3.4.5, §3.6 | P6 |
| Persisting the `resume` secret across a process restart | A credential's storage is the operator's and the controller's decision | P6 (Q-P3b-4) |
| `observation.events` as an input to anything | Best effort by design; S10's memory ingests only the reliable stream (step-17 §15.3) | — |
| A head-aware live-only join on an ephemeral world | The protocol carries no head (F-P3b-2) | S11, if wanted |

### 2.3 Invariants this PR must hold

```text
I-1     no change under kernel/, contracts/, persistence/, server/, systems/, worlds/, tools/, clients/;
        no *.rs file; no golden frame edited
I-2     mineworld_sdk imports nothing from mineworld_cognition; no new dependency (uv.lock unchanged)
I-10    no provider concept in mineworld_sdk
I-11    the SDK suite passes with no network except localhost, and the guard sees asynchronous
        connections on Windows too (F-P5-4)
INV-9   the SDK can say exactly join, submit and leave; a resume is a join
INV-13  every observation and every fact a seat delivers is for the observer its welcome named; a
        rejoin that is welcomed as another observer is a protocol violation, never a silent switch
P3b-1   a perceived fact reaches the consumer at most once per process, in strictly ascending EventId,
        and the facts it receives across any number of reconnects have no gap (IC-1)
P3b-2   the cursor a rejoin presents is the caller's CursorSource value at that moment, never a value
        the SDK advanced on receipt (P4 R-P3b-1, step-17 §3.4.3)
P3b-3   nothing is retried that the protocol says is final, and nothing is skipped to make a retry
        succeed: cursor_unavailable is never answered by rejoining with since: null or a newer cursor
P3b-4   a submit is sent at most once; a submit whose connection dropped before its answer fails with
        AnswerLost and is never re-sent by the SDK
P3b-5   the SDK sets no event-loop policy and uses only asyncio primitives both the selector and the
        proactor loop implement (F-P5b-1)
P3b-6   the invite and every resume secret appear in no repr, str, exception message or log line
        (D-P3-9)
```

---

## 3. Audit anchors (`origin/main @ bb62edf`)

Every row was read in this session. The implementing session re-reads each before editing.

| Anchor | Finding | Consequence for P3b |
| --- | --- | --- |
| `sdk/python/src/mineworld_sdk/session.py` (456 lines) `SeatSession`, `route` l. 370–419 | One connection. `Perceived` frames are appended to `self._perceived` and `self._cursor = frame.through` on **receipt** (l. 402–404); properties `perceived`, `perceived_cursor`. Module docstring: "No retry and no reconnect … Reconnecting belongs with `resume` (P3b)". A `Refused` without a token ends the session with `ProtocolViolation` (l. 409–410). | The list grows without bound and the cursor advances on receipt, against R-P3b-1. Both are replaced by a sink (§4.4). The tokenless refusal rule is wrong for `lagged` (F-P3b-1). The file is 44 lines under 500: new logic goes to new modules. |
| `session.py` `changed(since: int)` l. 296 | Newest wins by `seq`, compared as plain integers. | `seq` restarts at 1 on every connection (`PROTOCOL.md` §5.2, §4.2 "the stream restarts with a whole observation"), so across a rejoin `frame.seq > newest.seq` would hide the new connection's frames. The resuming seat orders by `(connection, seq)` (`Seen`, §4.2). |
| `session.py` `connect` l. 186–213; `websocket_connect(url, compression=None, max_size=…, open_timeout=JOIN_PATIENCE)` | `websockets`' default keepalive (ping every 20 s, 20 s timeout) is left on. | A half-open socket is detected within about 40 s, which can outlast the server's 30 s hold; the rejoin then meets `invalid_resume` and joins afresh (§4.5 row 7). |
| `sdk/python/src/mineworld_sdk/wire/frames.py` `Join`, `PerceivedJoin`, `Perceived`, `Refused`, `Closing`, `Welcome`, `RefusalCode`, `ClosingReason`, `TookOver` | Every S11-B/S11-C frame and value is modelled (S11-C's D-SC16): `join.resume`, `join.take_over`, `join.perceived { since }`, `welcome.resume` (repr hidden), `hold_seconds`, `took_over`, refusals `cursor_unavailable`, `lagged`, `invalid_resume`, `seat_occupied`, closing reasons `lagged`, `taken_over`, `superseded`, `kicked`, `server_stopping`. | No wire model changes. P3b builds behaviour on these. |
| `wire/contract.py` `PerceivedEvent` l. 303 | Strict model of `PerceivedEvent<serde_json::Value>`; `id: EventIdField`. | The batch type carries these unchanged; P4's ingestion takes them as is. |
| `wire/ids.py` | Ids are validated decimal strings without leading zeros (`_DECIMAL`), never converted to `int` in the SDK. | Ordering by `(len(id), id)` is exact for such strings; one helper, no `int(` (§4.4). |
| `server/tests/frames/{perceived,refused-lagged,closing-lagged,refused-cursor_unavailable,join}.json` | `refused-lagged.json` is `{ "t": "refused", "code": "lagged", "detail": … }` — **no token**. `closing-lagged.json` is `{ "t": "closing", "reason": "lagged" }`. | These two files, verbatim, are the fixture of AP3b-1. Today's SDK turns the first into `ProtocolViolation` and never reads the second (F-P3b-1). |
| `server/PROTOCOL.md` §4.1 (check 5 `perceived` before check 6 control; `cursor_unavailable` grants nothing, the connection stays), §4.2 (seat states; resume rules; a hold of `--hold` seconds, default 30; after a restart every hold and secret is gone), §5.2 (`acted_through` starts at `null` on a new connection), §5.5, §5.6, §5.8 (head, backfill in frames of ≤ 256 events with the last `through` equal to the head, order before observations, 4 096-fact flow bound, `lagged`, resume) | The client obligations P3b implements. | §4.5's table is derived row by row from these sections. |
| `server/src/protocol/fact.rs` `backfill_frames` l. 58 | Backfill chunks carry `through` = their last id; the last frame's `through` is set to the head; an empty backfill is one frame with no events and `through` = head. A fact that fails to render is logged and skipped. | `through` is non-decreasing on a connection; an empty batch can advance the cursor. Both shape §4.4's checks. |
| `tools/cli/src/main.rs` `Server` l. 98–140 | `--save DIR`, `--hold SECONDS` (default 30), `--town`, `--keyframe-every`, `--listen`; the invite from `MINEWORLD_INVITE`. No flag sets the perceived backlog. | The real-binary tests use `--save` and `--hold`. `lagged` cannot be provoked through the binary deterministically (S11-C E-SC6: OS socket buffers absorb the overflow); it is owned by the scripted tests (§5). |
| `tools/cli/tests/perceived.rs` `a_resumed_stream_equals_the_offline_export` | S11-C's CA-2 in Rust: `--town --save --hold 10`, 20 s + 20 s windows, the export filtered to ids ≤ the last cursor. | P3b's AP3b-9 is the Python mirror, made deterministic: no `--town`, lines spoken by a second seat on a script (§6). |
| `worlds/social-cafe/people/{wanderer,visitor,alice}.yaml` | All three start in `cafe`. `spoke` is `Visibility::Place`. | A line `visitor` says to `alice` is overheard by `wanderer`: a perceived fact the test causes on demand. |
| `sdk/python/tests/realserver.py` | Starts `mineworld server <world> --listen 127.0.0.1:0` only; kills with `Popen.kill`; finds the binary with the platform suffix. | Gains `--save`, `--hold`, restart on the same save, and a `perceived` export helper. |
| `sdk/python/tests/test_session_state.py` `Script` l. 106; `run` l. 147; `test_real_server.py` `run` l. 45 | Scripted connection. At `bb62edf` coroutines ran on the **default** loop; **since #142 (`7eed282`, re-read at `737e032`) each module's `run` uses `loop_factory=asyncio.SelectorEventLoop`**, closing F-P5-4's SDK half. `test_network_guard.py` still checks only a blocking `socket.create_connection`. | P3b's new modules share one `tests/support.py` `run` (selector by default; default loop for AP3b-9); AP3b-14 adds the asynchronous guard test. |
| `cognition/lm-controller/tests/support.py` `run` l. 61 | Runs on `asyncio.SelectorEventLoop` (F-P5-4's fix there). | The same pattern, in the SDK's own test support. |
| `pr-s10-p5b-hosted-subscriptions.md` F-P5b-1 | On Windows a selector loop cannot start subprocesses; a cognition process with a subscription bridge must run the proactor loop. | The SDK library must work on the proactor loop (P3b-5); one real-binary test runs on the platform default loop (§4.6). |
| `pr-s10-p4-memory.md` D-P4-5, P4-2, §11 R-P3b-1, `MemoryStore.cursor() -> EventId \| None`, `ingest(store, facts, through)` | Memory ingests `Sequence[PerceivedEvent]` with the frame's `through`; its cursor is never ahead of what it durably ingested; "the SDK's perceived cursor is advanced by the caller after a successful `ingest`, never by the session on receipt". | `CursorSource` is shaped so that `MemoryStore` satisfies it structurally; a batch is `(events, through)` (§4.2). |
| `step-17-cognition.md` §3.6 layer 2 | A decision records `based_on = (observation.at, observation seq, perceived cursor)`. | `seq` alone is ambiguous across connections; `Seen` carries the connection number and the `through` received before the observation (§4.2). |
| `.venv/.../websockets/asyncio/client.py` `connect.__aiter__` l. 592; `websockets/client.py` `backoff` l. 415, `BACKOFF_*` l. 409–412 (websockets 17.2, `uv.lock`) | The reconnecting iterator retries only a failed *connect*, through `process_exception`; delays come from module globals read from `WEBSOCKETS_BACKOFF_*` environment variables, with an initial delay of `random.random() * 5` from the unseeded global generator, then 3.1 s × 1.618 up to 90 s. | Declined for the protocol-level loop (DEP-44, §4.3). The transport stays `websockets` (DEP-25). |
| `scripts/ci_layer.py --list python`; `.github/workflows/ci.yml` l. 204–241 | The `python` layer builds `mineworld-cli` and runs `pytest sdk/python` on `ubuntu-24.04`, `windows-2025`, `macos-15`, Python 3.12. | Every P3b test, real-binary ones included, runs on all three in the PR's CI. No CI change is needed. |

**Findings of this audit:**

- **F-P3b-1 (defect, S11-C's absorbed SDK).** `SeatSession.route` treats `refused { code: "lagged" }`,
  which carries no token, as "a refusal naming no request" and ends the session with
  `ProtocolViolation` before the following `closing { lagged }` is read. A client that falls behind is
  therefore told it met a protocol violation, the opposite of `PROTOCOL.md` §5.5 ("the client rejoins
  … and loses nothing"). Fixed in C2 with a test that fails before the fix (AP3b-1).
- **F-P3b-2 (protocol gap, for S11; not fixed here).** On a world with no save, `since` older than the
  head is `cursor_unavailable`, and `since` equal to the head is "the way to ask it for the live stream
  alone" (`PROTOCOL.md` §5.8). But no frame tells a client the head (S11-C D-SC10: only `detail`, which
  a client must not branch on). Genesis facts exist from the first instant, so `since: null` is always
  refused on an ephemeral world, and a client cannot ask for the live stream there at all. P3b treats
  this as a terminal `SeatLost("cursor_unavailable")` (P3b-3) and S10 runs on persisted worlds
  (QS11C-2). Proposed to S11 as Q-P3b-6.
- **F-P3b-3 (S10 note).** `seq` restarts per connection, so step-17 §3.6's `based_on` needs the
  connection as well. Absorbed by `Seen` (§4.2); recorded for P6.

---

## 4. Design

### 4.1 Layout

```text
sdk/python/src/mineworld_sdk/
  session.py      one connection (P3), plus: the lagged sequence; perceived frames to a sink after the
                  connection-local checks; the through received before the newest observation
  perceived.py    PerceivedBatch, CursorSource, CursorCell, PerceivedStream; ConnectionOrder (the
                  per-connection checks); the delivery rule (suppression at the delivered high-water mark;
                  the local bound)                                                        (new, ~180 lines)
  resuming.py     ResumingSeat, ReconnectPolicy, Connector, Seen, SeatLost, SeatLossReason, AnswerLost,
                  NotConnected; the supervisor and its decision table (§4.5)              (new, ~350 lines)
  wire/ids.py     event_order(EventId) -> tuple[int, str]: length, then text
sdk/python/tests/
  support.py      run(coroutine, *, timeout_s, loop="selector"|"default")                 (new)
  scripted.py     ScriptedServer: a Connector whose connections the test scripts, with a fake clock
                  and a recorded sleep                                                   (new)
  test_session_state.py   + the lagged sequence (AP3b-1), the order checks (AP3b-2)
  test_perceived.py       delivery, suppression, cursor authority, local bound (AP3b-3, AP3b-4)  (new)
  test_resuming.py        the decision table, backoff, observations and submits across a rejoin
                          (AP3b-5 … AP3b-8)                                              (new)
  test_real_resume.py     IC-1 through the binary (AP3b-9 … AP3b-12)                      (new)
  realserver.py           + save, hold, restart on the same save, the perceived export
  test_network_guard.py   + the asynchronous connection on every platform (AP3b-14)
```

`SeatSession` stays the one-connection primitive that P3 froze, usable alone. `ResumingSeat` composes
one `SeatSession` per connection; it holds what outlives a connection (the instance, the cursor
source, the delivered high-water mark, the connection counter, the backoff). This keeps reconnection out
of the frame-routing code, which a scripted connection can test on its own, and keeps `session.py`
under 500 lines.

### 4.2 Public API (typed; no dict crosses it)

```python
# perceived.py
@dataclass(frozen=True, slots=True)
class PerceivedBatch:
    events: tuple[PerceivedEvent, ...]   # strictly ascending; every id > the previous batch's through
    through: EventId                     # store this as the cursor once `events` are durably ingested
    connection: int                      # which connection of the seat delivered it: 1, 2, …

class CursorSource(Protocol):
    def cursor(self) -> EventId | None: ...     # P4's MemoryStore satisfies this structurally

class CursorCell:                                # for a caller without a durable store
    def __init__(self, start: EventId | None = None) -> None: ...
    def cursor(self) -> EventId | None: ...
    def commit(self, batch: PerceivedBatch) -> None: ...   # refuses a through below the current one

class PerceivedStream:                           # AsyncIterator[PerceivedBatch], one consumer
    def __aiter__(self) -> Self: ...
    async def __anext__(self) -> PerceivedBatch: ...
        # delivers every batch already accepted, then: StopAsyncIteration after leave/detach,
        # or raises the SeatLost that ended the seat

# resuming.py
type Connector = Callable[[], Awaitable[Connection]]    # Connection is session.py's Protocol

@dataclass(frozen=True, slots=True)
class ReconnectPolicy:
    first_delay: float = 0.1        # seconds before the second attempt; the first is immediate
    factor: float = 2.0
    ceiling: float = 5.0
    jitter: float = 0.5             # each delay is scaled by a factor drawn from [1 - jitter, 1]
    give_up_after: float = 120.0    # wall seconds from the loss to SeatLost("gave_up")
    seed: int = 0                   # the jitter's generator: random.Random(seed), per seat
    perceived_buffer: int = 4096    # facts accepted but not yet taken by the consumer (§4.4)

@dataclass(frozen=True, slots=True)
class Seen:
    connection: int                       # 1 for the first connection, +1 per rejoin
    frame: ObservationFrame               # whole; deltas already applied
    perceived_through: EventId | None     # the last `through` received on this connection before
                                          # this frame, or None (step-17 §3.6 `based_on`)

SeatLossReason = Literal[
    "taken_over", "superseded", "kicked", "world_stopped",              # closing reasons
    "unauthorized", "protocol_mismatch", "seat_occupied", "unknown_seat",
    "seat_not_in_world", "invalid_nickname", "cursor_unavailable",      # join refusals
    "world_changed",            # a rejoin was welcomed into another world instance
    "protocol_violation",       # ProtocolViolation or ForeignObserver on any connection
    "gave_up",                  # give_up_after elapsed without a welcome
]

class SeatLost(MineWorldError):     reason: SeatLossReason; cause: MineWorldError | None
class AnswerLost(MineWorldError):   # the connection carrying a submit ended before its answer
class NotConnected(MineWorldError): # a submit while the seat is between connections; nothing sent

class ResumingSeat:
    @classmethod
    async def connect(cls, url: str, *, seat: EntityKey, invite: Invite, nickname: str,
                      cursor: CursorSource | None, take_over: bool = False,
                      resume: ResumeSecret | None = None,
                      policy: ReconnectPolicy = ReconnectPolicy()) -> ResumingSeat: ...
    @classmethod
    async def open(cls, connector: Connector, *, ...same keywords...) -> ResumingSeat: ...
        # returns after the first welcome; raises SeatLost for a terminal first answer
    @property
    def welcome(self) -> Welcome: ...           # the current connection's
    @property
    def instance(self) -> WorldInstanceId: ...  # fixed by the first welcome
    @property
    def connection(self) -> int: ...
    async def changed(self, after: Seen | None = None) -> Seen: ...
    def perceived(self) -> PerceivedStream: ... # once; a second call, or cursor=None, raises
    async def submit(self, request: ActionRequest) -> Outcome: ...
    async def leave(self) -> None: ...          # gives the seat back (closing { left })
    async def detach(self) -> None: ...         # closes without leave: the server holds the seat
    async def __aenter__/__aexit__              # __aexit__ leaves, as SeatSession's does
```

- `cursor=None` means "no perceived stream": the joins carry no `perceived`, as today's default.
- `take_over` and `resume` apply to the **first** join only. A rejoin presents the current
  connection's `resume` while the hold lasts, and never `take_over` (a human who took the seat keeps
  it: R-S11-4, `AC-5`).
- `ResumingSeat.open` with a `Connector` is the seam the tests use to follow a restarted server to its
  new port and to abort a transport. `connect(url)` builds the default connector over
  `websockets.asyncio.client.connect` with P3's arguments.
- Names are proposals (Q-P3b-2).

### 4.3 Decisions (D-P3b-1 … D-P3b-10)

| ID | Decision | Reason |
| --- | --- | --- |
| **D-P3b-1** | **Composition, not growth.** `SeatSession` stays one connection. `ResumingSeat` owns everything that outlives one: instance, cursor source, delivered high-water mark, connection counter, backoff, pending-submit failure. | One responsibility per file (`ENGINEERING_STANDARDS.md` §10); the routing code stays scriptable alone (AP-4's pattern); `session.py` stays under 500 lines. |
| **D-P3b-2** | **The caller's store is the cursor's only authority.** At every join the seat reads `CursorSource.cursor()` and presents it as `since`. The SDK never advances a durable cursor. `CursorCell` is the in-memory source for a caller without a store; it advances only on `commit(batch)`. | P4 R-P3b-1 and P4-2; step-17 §3.4.3 ("the id of the last perceived event that memory has durably ingested"). One authority means no second copy to disagree with the store. |
| **D-P3b-3** | **At most once per process, by a delivered high-water mark.** The seat keeps `delivered`, the `through` of the last batch handed to the consumer. A fact with `event_order(id) ≤ event_order(delivered)` is dropped from every later batch; a batch whose `through ≤ delivered` and which has no remaining fact is not delivered. A batch with no fact but a higher `through` **is** delivered, so the consumer's cursor can advance (P4's `ingest` takes an empty sequence). If the source's cursor is ever above `delivered` while `delivered` is known, the seat ends `SeatLost("protocol_violation")` with a `CursorAhead` cause: the caller claims to have ingested facts it was never given. | The rejoin presents the committed cursor (D-P3b-2), which may be below what was delivered; the server then re-sends (committed, delivered], and the consumer must not see them twice (P3b-1). Across a process crash the new process has no `delivered`, starts from its store, and P4's idempotent ingestion covers the rest. |
| **D-P3b-4** | **The SDK never stops reading its socket; its own buffer is bounded like the server's, and overflows the way the server's does.** Accepted batches wait for the consumer in a queue bounded by `perceived_buffer` facts (default 4 096, the server's own bound, `PROTOCOL.md` §5.8). On overflow the seat discards every accepted-but-undelivered batch, closes the socket **without** `leave` (the seat is held) and rejoins with `resume` and the source's cursor: a local `lagged`. | Pausing the reader instead would stall `result` frames on the same socket: a consumer that awaits a `submit` while not draining facts would deadlock. An unbounded buffer would hold a whole 300-day backfill in memory. Mirroring the server's rule keeps one concept: the price of not reading is a reconnect, never a gap. Each cycle makes progress of at least the facts the consumer committed. Q-P3b-5. |
| **D-P3b-5** | **The decision table of §4.5 is the whole reconnect policy.** Retry: a dropped socket, `closing { lagged }`, `closing { server_stopping }`, a local lag, a connect failure (`OSError`, `TimeoutError`, `websockets.exceptions.InvalidHandshake`), and `refused { invalid_resume }` (retried at once without `resume`). Everything else is terminal and typed. Unknown exceptions propagate. | `PROTOCOL.md` §§4.2, 5.5, 5.6, 5.8 say which outcomes a client recovers from; the rest are decisions a controller makes (P6). P3b-3: `cursor_unavailable` is terminal, never answered with another cursor. |
| **D-P3b-6** | **Resume within the hold, join afresh after it.** The seat notes the wall instant of the loss and the current welcome's `hold_seconds`. Before `loss + hold_seconds` a rejoin carries `resume`; after it, or when `hold_seconds` is 0, it carries none. Either way it carries `perceived { since: cursor() }`. A rejoin welcomed with another `instance` is left at once (`leave`) and ends `SeatLost("world_changed")`; one welcomed with another `observer` ends `SeatLost("protocol_violation")` (INV-13). | §4.2: after the hold "the seat returns to its default"; after a restart "every hold and every secret is gone … a client joins again with its invite". The instance check stops a cursor of one world being replayed against another (`PROTOCOL.md` §5.7: a restarted persisted world keeps its instance). |
| **D-P3b-7** | **Observations across connections are ordered by `(connection, seq)`.** `changed(after)` returns the newest `Seen` strictly after `after` in that order. A new connection's first frame is whole (`PROTOCOL.md` §4.2), so the seat never applies a delta across connections; `SeatSession` already refuses one on a foreign base. | F-P3b-3. `acted_through` restarting at `null` is then visible to the caller as a new `connection`. |
| **D-P3b-8** | **Submits are never re-sent.** A pending submit whose connection ends fails with `AnswerLost`; a submit while no connection is welcomed raises `NotConnected` at once and sends nothing. The caller reads what happened from later observations and facts (`acted_through`, `Causation`). | A request is not idempotent; re-sending one could act twice. step-17 §3.6 already re-decides on the newest state. |
| **D-P3b-9** | **Backoff is ours, seeded and injectable.** The first attempt after a loss is immediate; then `first_delay × factor^k`, capped at `ceiling`, each scaled by a factor drawn from `random.Random(policy.seed)`, until `give_up_after`. The sleep and the clock are injectable for tests (`scripted.py`), and default to `asyncio.sleep` and `loop.time()`. No environment variable is read. | `.structured-coding/standards.md`: randomness is seedable. D-P3-9: the SDK never reads the environment. DEP-44. |
| **D-P3b-10** | **Event loops: neither chosen nor required.** The SDK uses tasks, futures, `asyncio.Event`, `asyncio.Queue`, `asyncio.sleep`, `asyncio.timeout` and `websockets` — all implemented by both the selector and the proactor loop. It never calls `add_reader`, `add_signal_handler`, `set_event_loop_policy`, or names a loop class; a source scan holds this (AP3b-13). | F-P5b-1: a P6 process with a subscription bridge runs the proactor loop on Windows. step-17 §15.8: "Reconnect and backoff use only asyncio primitives. They need no POSIX signals and no Unix sockets." |

**DEP-44 (numbered by the primary session, 2026-10-10; placeholder DEP-P3b-a in the draft).** *The resuming seat's
reconnect loop is MineWorld's own (about 100 lines in `resuming.py`), on top of the adopted
`websockets` transport (DEP-25).* Declined, both directions of `REUSE_POLICY.md`:

- **`websockets`' reconnecting iterator** (`async for connection in connect(url)`, 17.2): it retries a
  failed *connect* only, and knows nothing of `join`, `resume`, the hold, the cursor or a world
  instance, which are the whole problem; its delays are module globals read from `WEBSOCKETS_BACKOFF_*`
  environment variables at import, starting with `random.random() * 5` seconds from the unseeded global
  generator, then 3.1 s growing to 90 s — longer than the server's default 30 s hold, not seedable
  (`standards.md`), and read from the environment (D-P3-9). Kept: the transport and its keepalive.
- **`tenacity` (Apache-2.0) and `backoff` (MIT)**: a dependency for a capped exponential sequence of a
  few lines, while the decision of *what* to retry stays ours either way; each would also put its own
  sleep and clock in the path the scripted tests replace.
- **Revisit when:** a second protocol client in Python needs the same loop (then it moves, unchanged,
  not into a library).

### 4.4 The perceived stream, precisely

```text
per connection (session.py, ConnectionOrder in perceived.py), for each perceived frame F:
  1  events strictly ascending by event_order
  2  first event > the previous frame's through on this connection (no duplicate within a connection)
  3  through ≥ the last event of F, and through ≥ the previous through (non-decreasing)
  any failure: ProtocolViolation → the seat ends SeatLost("protocol_violation")      (fail closed)
  then the frame goes to the sink; the session records through as `last_through`

per seat (perceived.py), on each frame handed over by the current connection:
  keep = [e for e in F.events if delivered is None or e > delivered]
  if keep or delivered is None or F.through > delivered:
      accept PerceivedBatch(tuple(keep), F.through, connection)
      if the accepted-but-undelivered facts now exceed perceived_buffer: local lag (D-P3b-4)
  the consumer's __anext__ takes the oldest accepted batch and sets delivered = its through

per rejoin (resuming.py):
  discard accepted-but-undelivered batches          (they will be re-sent; nothing is lost)
  since = cursor_source.cursor()                    (D-P3b-2; check: ≤ delivered when delivered is known)
  join { resume?, perceived: { since } }
```

`event_order(id) = (len(id), id)`: exact for the SDK's ids, which are decimal strings without leading
zeros (`ids.py` `_DECIMAL`); no id becomes a number (P4 D-P4-4's rule, the SDK's own D-P3-3).

**What the consumer is promised.** Batches arrive in order; their facts are strictly ascending across
all batches and all connections of one `ResumingSeat`; no fact arrives twice; and, if the consumer
commits every batch it ingests, the facts it ingested are exactly the observer's perceived set up to
its last committed `through` — the set `mineworld perceived` exports (I-4 of step-17). A consumer that
stores `through` before ingesting breaks that promise itself; the SDK cannot see it.

### 4.5 The reconnect decision table

Rows are checked in order. "Rejoin" means: wait the next backoff delay (the first is immediate), then
`connector()`, then `join` with D-P3b-6's fields, counting against `give_up_after` from the loss.

| # | What happened | Source | Outcome |
| --- | --- | --- | --- |
| 1 | `leave()` answered by `closing { left }` | §2, §5.6 | the stream ends (`StopAsyncIteration`); `changed` raises `SessionClosed("left")` |
| 2 | `detach()` | — | socket closed without `leave`; the stream ends; no rejoin |
| 3 | `closing { taken_over }`, `{ superseded }`, `{ kicked }`, `{ world_stopped }` | §4.2, §5.6 | `SeatLost(<reason>)`; no rejoin (step-17 §3.4.3: "seat_taken: stop") |
| 4 | `refused { lagged }` then `closing { lagged }`, or the socket closes between them | §5.5, §5.6, §5.8 | rejoin with `resume` |
| 5 | local lag (D-P3b-4) | — | close without `leave`; rejoin with `resume` |
| 6 | socket dropped without `closing`; `closing { server_stopping }`; a connect failure | §4.2 | rejoin, with `resume` while the hold lasts (D-P3b-6) |
| 7 | rejoin refused `invalid_resume` | §4.2 rule 1, §5.5 | rejoin at once without `resume` (the hold ended or the server restarted) |
| 8 | any join refused `cursor_unavailable` | §4.1 check 5, §5.8 | `SeatLost("cursor_unavailable")`; never another cursor (P3b-3); F-P3b-2 |
| 9 | any join refused `seat_occupied`, `unknown_seat`, `seat_not_in_world`, `invalid_nickname`, `unauthorized`, `protocol_mismatch` (or `ProtocolMismatch`) | §4.1, §4.2 | `SeatLost(<code>)`. `unauthorized` at a rejoin is the operator rotating the invite (QS11D-3) |
| 10 | rejoin welcomed with another `instance` | §5.7 | `leave`, then `SeatLost("world_changed")` |
| 11 | `ProtocolViolation` or `ForeignObserver` on any connection, or a welcome naming another observer | INV-13 | `SeatLost("protocol_violation", cause)` |
| 12 | `give_up_after` elapsed since the loss with no welcome | — | `SeatLost("gave_up")` |

Pending submits fail with `AnswerLost` on rows 4–7 and 11; with the `SeatLost` itself on rows 3 and
8–12; with `SessionClosed` on rows 1–2 (D-P3b-8).

### 4.6 Event loops and the network guard on every platform

- **Library (P3b-5, D-P3b-10).** Works on whichever loop the caller runs.
- **Tests.** Since #142 the SDK suite runs on `asyncio.SelectorEventLoop` on every platform, so
  pytest-socket's `socket.connect` patch sees every connection (F-P5-4, closed there). P3b factors that
  into one `tests/support.py` `run(coroutine, *, timeout_s, loop="selector" | "default")`, used by its
  new modules; the two existing per-module `run` helpers may delegate to it in C2 (a helper move, no
  behaviour change).
- **The proactor loop is exercised too.** AP3b-9 (IC-1 through the binary) runs on the platform's
  default loop (`loop="default"`): the proactor on Windows, the loop P6 will run (F-P5b-1). Its only
  connections go to the address the test's own server printed on `127.0.0.1`; that the guard does not
  see proactor connections on Windows is recorded, not hidden.
- **AP3b-14** proves the guard on Windows for asynchronous connections through the runner.

---

## 5. Test ownership (test rules §26)

```text
STATIC     ruff, pyright strict: the typed API (no dict at the boundary), Literal SeatLossReason,
           NewType ids (an EventId is not an EntityId). No unit test re-proves these
UNIT       scripted connections + a fake clock (deterministic, no socket):
           the lagged sequence from the golden files (AP3b-1); the connection-local order checks
           (AP3b-2); delivery, suppression, cursor authority (AP3b-3); the local bound (AP3b-4);
           the decision table (AP3b-5); seeded backoff and give-up (AP3b-6); observations across
           connections (AP3b-7); submits across a loss (AP3b-8); the loop-primitive scan (AP3b-13);
           the guard (AP3b-14); secrets (AP3b-17)
GATE 1     NOT REQUIRED: no language model, no model-facing text (CLAUDE.md §5)
GATE 2     the real mineworld binary, real sockets, real saves, real kills: IC-1 across a dropped
           socket (AP3b-9), a client crash (AP3b-10), a server restart (AP3b-11); the ephemeral
           world's refusal (AP3b-12). In pytest under the real_server marker, so CI's python job runs
           them on all three platforms
NOT OWNED  the server's own lagged trigger through the binary: S11-C CA-5 owns it in-process, and
           E-SC6 recorded that OS socket buffers make it non-deterministic through a socket; P3b owns
           the client's reaction to the golden frames (AP3b-1, AP3b-5 row 4)
CI IMPACT  sdk/python only: the python layer (ruff, pyright, cargo build -p mineworld-cli, pytest)
           on Linux, Windows, macOS; the Rust layers see no change (AP3b-15). The PR's CI on the
           final head is the canonical full run; no local full suite
```

---

## 6. Acceptance criteria (fixed now, before anything is measured)

Bounds are literals from the requirements, never from the implementation (`ARC-23`). Each criterion
names the mutation that must turn it red; mutations are planted on the working tree, seen red, reverted
and recorded in §13.4.

| ID | Criterion | Mutation that must fail it |
| --- | --- | --- |
| **AP3b-1** The lagged sequence (F-P3b-1, regression first) | A scripted connection sends `welcome`, one `perceived` frame, then the bytes of `server/tests/frames/refused-lagged.json` and `closing-lagged.json` verbatim. `SeatSession` ends `SessionClosed("lagged")`, never `ProtocolViolation`. Through `ResumingSeat`, the next connection receives exactly one frame from the client before its welcome: a `join` whose `resume` equals the first welcome's and whose `perceived.since` equals the `CursorCell`'s value. The test is run against `main`'s `route` before the fix, seen red, and that run is recorded (C2). | Today's code (tokenless refusal → `ProtocolViolation`): red. |
| **AP3b-2** Fail closed on a broken stream | Four scripted streams, each ending the seat `SeatLost("protocol_violation")` and delivering nothing past the bad frame: events `[12, 7]`; a frame whose first event equals the previous frame's `through`; `through` below its own last event; `through` below the previous frame's. | Remove check 2 of §4.4: the duplicate is delivered. |
| **AP3b-3** Exactly once, and the store decides the cursor | Script: connection 1 sends frames `A{7,10 → through 10}`, `B{12,15 → 20}`; the consumer takes A and B and commits only A to a `CursorCell`; the socket drops. Connection 2 must receive `join.perceived.since == "10"` (asserted on the sent frame), then is sent `{12,15,18 → 25}`. The consumer receives `{18 → 25}` only; the concatenation of delivered ids is `7, 10, 12, 15, 18`: ascending, no duplicate, no gap. A second case: a source whose cursor is `"30"` while `delivered` is `"25"` ends `SeatLost("protocol_violation")` (`CursorAhead`). | (a) Rejoin with the last *received* `through` (`"20"`): the `since` assertion fails. (b) Remove suppression: 12 and 15 delivered twice. (c) Present `since` one id late (`"11"` for `"10"`, mutation code only): with the script's fact 11 admitted in the variant case, 11 is missing. |
| **AP3b-4** The local bound | `perceived_buffer = 4`; the consumer reads nothing while 6 facts arrive in three frames. The client sends **no** `leave`, closes, and rejoins with `resume` and the cell's cursor; after the consumer drains and commits each batch, the delivered ids equal the script's admitted ids in order. | Drop the oldest batch on overflow instead: a gap. |
| **AP3b-5** The decision table | Parametrized over every row of §4.5 with scripted servers and a fake clock: the outcome (rejoin, with or without `resume`; or `SeatLost` with the row's reason) and the frames the client sent are exactly the row's. Row 6 at `loss + hold − 1 s` carries `resume`; at `loss + hold + 1 s` it does not; with `hold_seconds = 0` it never does. Row 10 sends `leave` before raising. | Answer `cursor_unavailable` with a rejoin at `since: null`: row 8 fails (and P3b-3 is named in the message). |
| **AP3b-6** Seeded, bounded backoff | With a recording sleep and a fake clock and a connector that always fails: the delays for seed 7 are identical across two runs; the first is 0; each is ≤ `ceiling`; the sum before `SeatLost("gave_up")` is ≤ `give_up_after`; seeds 7 and 8 differ. | Draw jitter from the module-level `random`: the two seed-7 runs differ. |
| **AP3b-7** Observations across connections | Connection 1 delivers `seq` 1 … 40 (deltas included); connection 2 starts at `seq` 1, whole. `changed(after=<conn 1, seq 40>)` returns `Seen(connection=2, seq=1)` within the test's 5 s bound; its `acted_through` is `None`; `perceived_through` is the last `through` received on connection 2 before that frame. | Compare by `seq` alone: `changed` waits past the bound. |
| **AP3b-8** Submits across a loss | A submit pending when the socket drops fails `AnswerLost`; a submit while reconnecting raises `NotConnected` and the scripted connections record no `submit` frame; after the rejoin, the first submit's token is fresh on that connection and is answered. | Re-send pending submits on the new connection: a second `submit` frame is recorded. |
| **AP3b-9** IC-1 through the binary: a dropped socket | `mineworld server worlds/social-cafe --save <tmp> --hold 10` (no `--town`: the facts are caused by the test). `wanderer`: a `ResumingSeat` with a `CursorCell` at `None`, on the **default** loop; the consumer commits each batch. `visitor`: a `SeatSession` that says a line to `alice` (`offers.request`, `talk`). Script: 3 lines; wait until the wanderer has delivered their `spoke` facts; abort the wanderer's transport (through the connector); 3 lines while it is away; it rejoins (`Seen.connection == 2`, `welcome.took_over == "held"`); 3 lines; the wanderer leaves; the server is killed (`Popen.kill`). Then `mineworld perceived worlds/social-cafe --save <tmp> --person wanderer --json`, filtered to ids ≤ the last committed cursor: the delivered ids equal the export exactly; strictly ascending; the first delivered id is the first exported (a genesis fact: `since: null` serves from genesis); the 3 lines said while away are delivered on connection 2; the rejoin's `join.perceived.since` equals the cell's value at that moment (recorded connection). Every wait is event-driven with a 30 s ceiling. | (a) The seat drops the first fact of every backfill: the away lines are not all delivered. (b) Rejoin with `since: null`: the recorded `since` assertion fails (suppression alone would hide it, which is why it is asserted). |
| **AP3b-10** IC-1: the client crashes | As AP3b-9 up to a cut chosen by `random.Random(<seed>)` among the first 1 … 4 delivered batches (the seed and cut are printed). Then the wanderer seat is abandoned with `detach()`: the socket closes without `leave`, as a crash would leave it, and the seat is held. A **new** `ResumingSeat` starts cold with the same `CursorCell` and `resume=` the first seat's last `welcome.resume` (Q-P3b-4's recommended path; `welcome.took_over == "held"`), 3 more lines are said, it leaves, the server is killed. Delivered ids of both seats, concatenated, equal the export up to the cell's cursor; no id twice. | The second seat starts with `CursorCell(None)`: genesis facts arrive twice in the concatenation. |
| **AP3b-11** IC-1: the server restarts | As AP3b-9 to the first 3 lines; then the server is killed and started again on the same save (a new port, which the test's connector follows). The seat retries (connect failures), meets `invalid_resume` (holds do not survive a restart) and joins afresh (`took_over == "none"`); `instance` is unchanged; 3 lines (a new `visitor` session) are delivered; the concatenation equals the export of the final save. | Treat `invalid_resume` as terminal (row 9): the test ends `SeatLost`. |
| **AP3b-12** No save, no history | `mineworld server worlds/social-cafe` without `--save`: `ResumingSeat.connect(..., cursor=CursorCell(None))` raises `SeatLost("cursor_unavailable")`, and the same seat is then joinable by a plain `SeatSession` (nothing was granted, `PROTOCOL.md` §4.1). | Retry `cursor_unavailable` with another cursor: the open does not raise. |
| **AP3b-13** Every platform | The whole SDK suite passes in the PR's CI on `ubuntu-24.04`, `windows-2025`, `macos-15`. A source scan of `src/mineworld_sdk/` finds none of `add_reader`, `add_writer`, `add_signal_handler`, `set_event_loop_policy`, `SelectorEventLoop`, `ProactorEventLoop`, `import signal`, `AF_UNIX`. | Plant `loop.add_reader(…)` in `resuming.py`: the scan names the file and line. |
| **AP3b-14** The guard sees asynchronous connections | Under `support.run`, a `websockets` connect to `ws://192.0.2.1:80/ws` fails within 1 s with pytest-socket's error type, on every platform (F-P5-4; #142 fixed the runner, this is the test that pins it). | Make `support.run` use the default loop: the Windows leg fails (a timeout, not the guard's error). |
| **AP3b-15** Scope | `git diff --stat <base>..HEAD` touches only `sdk/python/**`, `docs/DECISIONS.md`, `.structured-coding/**`. No `*.rs`, no golden frame, `uv.lock` unchanged. | — (a diff gate) |
| **AP3b-16** Static | `ruff check`, `ruff format --check`, `pyright` strict over `sdk/python`: zero findings. | `def f(x): return x` in `perceived.py`: pyright reports it. |
| **AP3b-17** Secrets | With the 32-character marker invite and a scripted welcome's resume secret: neither string appears in the `str` or `repr` of any `SeatLost`, `AnswerLost`, `NotConnected`, `Seen`, `ResumingSeat`, `ReconnectPolicy`, nor in any captured log record of a run through rows 3–12. | Put the welcome into `SeatLost`'s message: the scan finds the secret. |

**IC-1, as step-17 §10 states it**, is AP3b-9 … AP3b-11 together: the live stream, killed (socket,
client, server) and resumed with its cursor, concatenates to exactly the offline export. Its literal
mutation, "the cursor is resumed one id late: a fact is missing", is AP3b-3 (c), where the fact after
the cursor is known to be admitted; through the binary the fact after a cursor need not be one the
wanderer perceives, so that literal mutation could stay green there, and AP3b-9 (a) is its
deterministic counterpart.

---

## 7. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-P3b-1 | Real-binary tests flake on timing (CI runners, Windows). | No `--town`: every perceived fact after genesis is caused by the test; every wait is event-driven with a 30 s ceiling; the server's port is the one it prints (`--listen 127.0.0.1:0`), never a guessed free port (F-13w-3's port race). |
| R-P3b-2 | A half-open socket is noticed only by keepalive (~40 s), past the 30 s default hold. | Row 7: `invalid_resume` → a fresh join. Nothing is lost; the cursor is the store's. Recorded for P6, which may set `--hold` higher for LM seats. |
| R-P3b-3 | A slow consumer of a long backfill cycles through local lags. | Each cycle advances by what the consumer committed; the bound is a policy field (Q-P3b-5); the rejoin's backfill is read off the world thread (S11-C SD-C6), so the world never stalls (CA-13). |
| R-P3b-4 | Windows: an open save handle after a kill blocks `tmp_path` removal or the restart. | Servers are stopped with `Popen.kill` and waited; the export runs after the kill (S11-C §17.14's order); the restart in AP3b-11 waits for the old process to exit before starting. |
| R-P3b-5 | The proactor loop behaves differently under `websockets` (close, abort). | AP3b-9 runs on the default loop on all three CI legs; `transport.abort()` is the abort used on both loops. |
| R-P3b-6 | Changing `SeatSession`'s `perceived` and `perceived_cursor` (added by S11-C's D-SC16 as an interim surface) breaks a caller. | Audited: only SDK tests use them; no stable contract exists (`CLAUDE.md` §4 rule 12). Replaced cleanly, without an adapter. |
| R-P3b-7 | P6's needs differ once designed (for example a per-seat hook on rejoin). | `Seen.connection` and `welcome.took_over` carry the facts; anything more is added by P6 against a second consumer, not guessed now. |

---

## 8. Commit plan

Each commit tracks **implementation**, **validation** and **review** separately. `[x]` needs the work
and its evidence. An item that turns out not to apply is `N/A` with the audited reason. Commands run
from the worktree root; `uv` and `cargo` are on `PATH`.

### C0 — Freeze record and contract (Markdown only)

- **Goal.** The freeze, §11 and §12.1 were written by the planning session on 2026-10-10 (PR #149). The
  implementation session's C0 records the exact implementation base and initializes the handoff.
- **Scope.** This document (§13.2 E-P3b-1: base commit, test counts at the base); a new
  `handoff-s10-p3b.md`. No code.
- [x] Implementation: the base recorded (§13.2 E-P3b-1, `f4ed913`); `handoff-s10-p3b.md` initialized
  with the contract's required fields; the endpoint lines of §11 copied with their sources.
- [x] Validation: `python3 scripts/check_doc_headings.py`; `python3 scripts/check_decision_ids.py`:
  both clean (E-P3b-2).
- [x] Review: the handoff's endpoint lines match §11 word for word in substance, each with §11's
  source; none narrowed or widened. Q-P3b-1 … 8 are all ruled in §12.1 (Q-P3b-6 open with S11 only).
- **Commit boundary.** Documentation only.

### C1 — The decision before the code

- **Goal.** DEP-44 exists before the loop it governs (`CLAUDE.md` §2.2).
- **Scope.** `docs/DECISIONS.md` (DEP-44, §4.3's text). **Non-goal:** any code.
- [x] Implementation: `docs/DECISIONS.md` DEP-44, with `websockets` 17.2's facts cited from its
  installed source (corrected in DV-P3b-1: `reconnect_delays` is injectable, the sleep is not).
- [x] Validation: `check_decision_ids.py` → "105 decision ids, all distinct"; `check_doc_headings.py`
  → clean (E-P3b-3).
- [x] Review: both directions answered — reinventing (`websockets`' iterator, `tenacity`, `backoff`
  considered and declined with reasons) and forcing (the iterator knows no join/resume/cursor). Licences
  read from each distribution's own `METADATA`: websockets 17.2 `License-Expression: BSD-3-Clause`;
  tenacity 9.1.4 `License: Apache 2.0`; backoff 2.2.1 `License: MIT` (uv's local cache, no network).

### C2 — One connection: the lagged sequence, the sink, the order checks; the test runner

- **Goal.** F-P3b-1 fixed with its regression test first; `SeatSession` hands perceived frames to a
  sink after the connection-local checks; the shared test runner and the asynchronous guard test
  (F-P5-4's runner fix itself is #142's).
- **Scope.** `session.py` (`route`: tokenless `lagged` expected, then `closing { lagged }` →
  `SessionClosed("lagged")`; `Perceiving(since, deliver)` replaces `perceived: PerceivedJoin` on
  `connect`/`join`; `perceived` and `perceived_cursor` removed; `newest_through` recorded with the
  newest observation); `perceived.py` (`ConnectionOrder` only); `wire/ids.py` (`event_order`);
  `tests/support.py` (new: #142's selector-loop `run`, factored, plus `loop="default"`);
  `test_session_state.py`, `test_real_server.py` (their `run` may delegate to it; the old `perceived`
  assertions rewritten against the sink); `test_network_guard.py` (AP3b-14).
  **Unchanged:** every wire model; newest-wins; token pairing; INV-13.
- [x] Implementation: AP3b-1's test written first and run against `main`'s `route` (red, E-P3b-4);
  then the fix in the same commit. `session.py`: `Perceiving(since, deliver)` replaces
  `perceived: PerceivedJoin` on `connect`/`join`, which also take `resume`; `route` hands each
  perceived frame to the sink after `ConnectionOrder.admit`; a tokenless `refused { lagged }` sets
  `_lagging` and `closing { lagged }` (or a drop after it) ends `SessionClosed("lagged")`;
  `newest_through`, `ended`, `close()` (no `leave`), `wait_closed()`, `open_socket(url)`,
  `TRANSPORT_FAILURES` added; `perceived`/`perceived_cursor` removed (Q-P3b-3). `perceived.py`:
  `ConnectionOrder`. `wire/ids.py`: `event_order`. `errors.py`: the five session errors moved here
  (DV-P3b-2). `tests/support.py`: `run(coroutine, *, timeout_s, loop)`; both existing per-module
  `run`s delegate to it.
- [x] Validation (E-P3b-5): 53 passed (46 at the base + AP3b-1, the lagged-drop case, 4 × AP3b-2,
  AP3b-14; the rewritten perceived test replaces the old one), 4 real_server passed; ruff, ruff
  format, pyright strict clean. Mutations (§13.4): AP3b-1 red on main's code; AP3b-2 check 2 removed →
  `[duplicate]` red; AP3b-14 default loop → inert on macOS (selector is the default there), Windows
  evidence is the PR's CI leg.
- [x] Review: a tokenless refusal other than `lagged` still ends `ProtocolViolation`
  (`test_a_refusal_naming_no_request_ends_the_session` unchanged, green); the sink is called from
  `route`, which only the reader task calls, one frame at a time; `session.py` 495 lines. A perceived
  frame on a join that asked for none now ends `ProtocolViolation` (the server sends none then,
  `PROTOCOL.md` §5 table) — DV-P3b-3.
- **Failure cases.** `lagged` arriving without its `closing` (the socket drops first) → the session
  ends `SessionClosed("lagged")` all the same; a second `lagged` is impossible on one connection.

### C3 — The perceived stream: delivery, suppression, the cursor authority, the local bound

- **Goal.** §4.4's per-seat rules as a pure, scriptable component.
- **Scope.** `perceived.py` (`PerceivedBatch`, `CursorSource`, `CursorCell`, `PerceivedStream`, the
  accept/deliver/discard operations), `tests/test_perceived.py`. **Depends on:** C2.
- [ ] Implementation.
- [ ] Validation: AP3b-3 (a)–(c) and the `CursorAhead` case; AP3b-4. Mutations AP3b-3 (a), (b), (c);
  AP3b-4.
- [ ] Review: no `int(` on an id (scan as in P4 AP4-2 (c)); an empty batch with a higher `through` is
  delivered; a terminal end delivers what was accepted before raising.

### C4 — The resuming seat: the decision table, backoff, observations and submits across connections

- **Goal.** §4.2's `ResumingSeat` and §4.5, deterministic under scripted connections.
- **Scope.** `resuming.py`; `__init__.py`; `tests/scripted.py` (scripted connector, fake clock,
  recorded sleep); `tests/test_resuming.py`. **Depends on:** C3.
- [ ] Implementation.
- [ ] Validation: AP3b-5 (every row), AP3b-6, AP3b-7, AP3b-8, AP3b-13 (the scan), AP3b-17; static.
  Mutations: AP3b-5 (row 8), AP3b-6, AP3b-7, AP3b-8, AP3b-13, AP3b-17.
- [ ] Review: every row of §4.5 traced to a `PROTOCOL.md` sentence; `take_over` never on a rejoin;
  `detach` and `leave` race-free against a rejoin in flight; no task leaks after `leave`, `detach` or
  `SeatLost` (each test asserts `asyncio.all_tasks()` returns to its starting set); `resuming.py`
  under 500 lines.
- **Failure cases.** A connector that raises an unexpected exception type → propagates (D-P3b-5);
  `changed()` after `SeatLost` raises it; `perceived()` called twice → `RuntimeError`.

### C5 — IC-1 through the real binary

- **Goal.** The integration checkpoint: AP3b-9 … AP3b-12.
- **Scope.** `tests/realserver.py` (arguments, restart on a save, `perceived_export(world, save, person)`
  running `mineworld perceived … --json` and parsing each line as `PerceivedEvent`), `tests/test_real_resume.py`.
  **Depends on:** C4; the binary (`cargo build -p mineworld-cli`).
- **Gate spec (before running).** Claim: a Python client's stream across a socket drop, a client crash
  and a server restart equals the offline export. Owner: Gate 2 (real binary, real sockets, real
  saves, real kills). Evidence: the id lists and their equality, the recorded rejoin `since`, the
  connection numbers and `took_over` values. Counterfactual: AP3b-9 (a), (b), AP3b-10, AP3b-11's
  mutations. Cost: four tests, each under 30 s, together normally under a minute.
- [ ] Implementation.
- [ ] Validation: AP3b-9 … AP3b-12 locally (macOS), each classified PASS / FAIL / INCONCLUSIVE from the
  printed evidence; their mutations; the Windows and Linux evidence is the PR's CI (C6).
- [ ] Review: the oracle is the binary's export, never the SDK's own output (test rules §25); no test
  sleeps for a fixed time where an event can be awaited; `tmp_path` only.

### C6 — Close: README, ledger, scope, the PR

- **Scope.** `sdk/python/README.md` (one example, under 40 lines in total); §13 of this document; the
  handoff.
- [ ] Implementation: README; ledger; deviations; findings for P6 and S11.
- [ ] Validation: AP3b-15 (diff gate), AP3b-16; **one** PR CI run on the final head (`fast`, `python`
  on the three platforms), recorded with its run id; AP3b-13's three legs read from it.
- [ ] Review: every AP3b with evidence or an honest INCONCLUSIVE; every mutation planted, red, reverted;
  the PR marked READY FOR OPERATOR REVIEW — DO NOT MERGE.

---

## 9. Requirements this PR places on other PRs

- **R-P6-P3b-1 (on P6).** Drive a seat through `ResumingSeat`; pass the seat's `MemoryStore` as the
  `CursorSource`; `ingest(store, batch.events, batch.through)` for every batch before taking the next;
  record `based_on` with `Seen.connection` as well as `seq` (F-P3b-3); persist the current
  `welcome.resume` beside the memory store and rewrite it after every rejoin, so a restart inside the
  hold rejoins with it (Q-P3b-4, operator ruling 2026-10-10); decide the slow retry after
  `SeatLost("taken_over" | "seat_occupied")` (R-S11-4); run the platform's default loop on Windows if a
  bridge is bound (F-P5b-1).
- **R-S11-P3b-1 (on S11; recorded by the primary session 2026-10-10, not blocking).** F-P3b-2: a client of a world without a save cannot
  ask for the live stream alone, because the head is not on the wire. Options for S11: the head on
  `welcome`, or `since: "live"`. P3b needs neither (Q-P3b-6).

## 10. Edits to parent documents

| Document | Edit | Owner, when | State |
| --- | --- | --- | --- |
| `step-17-cognition.md` §15.2 | P3b's row: designed and frozen in `pr-s10-p3b-perceived.md`; at merge, "merged as …" | this planning session at freeze (primary's instruction); the S10 planning session at merge | **applied at freeze** (PR #149) |
| `step-17-cognition.md` §3.4.3 | Name `ResumingSeat`; the cursor a rejoin presents is the one the memory store reports (R-P3b-1); a rejoin is with `resume` inside the hold | as above | **applied at freeze** (PR #149) |
| `overall.md` | DEP-44 recorded as used by P3b | primary session | primary's |

## 11. Execution contract (filled at freeze, 2026-10-10)

Source of every line marked "primary session": its freeze instruction of 2026-10-10, relayed by the
coordinator ("record rulings in §12, header DESIGN FROZEN 2026-10-10 (primary session), fill §11's
contract (branch `mvp0/pr-s10-p3b`, worktree `/Users/yuema137/mineworld-worktrees/impl-p3b`)"). Lines
marked "S10 convention" follow P3's and P4's contracts (`pr-s10-p3-python-sdk.md` §10,
`pr-s10-p4-memory.md` §12) and were not separately ruled; the endpoints the instruction does not name
take the working rules' shipped defaults (§21), which it does not narrow.

```text
PROJECT / PR:            MineWorld mvp0, S10 PR P3b — the SDK's perceived stream, cursor and resume
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s10-p3b-perceived.md
RELATED / BINDING DOCS:  step-17-cognition.md (§3.4.3, §3.6, §10 IC-1, §15); pr-s10-p3-python-sdk.md;
                         pr-s10-p4-memory.md (R-P3b-1); step-12-server.md §§16–17; server/PROTOCOL.md
                         rev 2 §§4, 5; docs/REUSE_POLICY.md; docs/ENGINEERING_STANDARDS.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-p3b         (primary session)
BRANCH:                  mvp0/pr-s10-p3b, created from main                   (primary session)
IMPLEMENTATION BASE:     origin/main at the start of implementation; C0 records the commit
                         (designed against origin/main @ 737e032, which includes #142)
APPROVED SCOPE:          §2.1, as frozen
FROZEN INVARIANTS:       §2.3; D-P3b-1 … D-P3b-10; DEP-44; §4.5's table; AP3b-1 … AP3b-17;
                         §12.1's rulings
SEQUENCE:                C0 … C6
ALLOWED COMMANDS:        cargo *; git; gh (never merge); uv *; python3 scripts/*; mkdir -p; sed -n;
                         target/*/mineworld *                                 (S10 convention)
NEVER:                   python3 -c; sed -i; awk; xargs; curl; heredoc writes; reading
                         ~/.config/mineworld/secrets.env                      (S10 convention)
MATERIAL STOPS:          any Rust, server, protocol or golden-frame change; any new dependency
                         (uv.lock must not change); a change to §4.5's table or to P3b-1 … P3b-6;
                         any hosted-API use                                   (S10 convention)
PLATFORMS:               Linux, macOS, Windows (operator, 2026-10-08)
VALIDATION BUDGET:       unit, static and local real-binary tests: unrestricted. Gate 1: NOT REQUIRED.
                         No model, no key, no paid API. CI: the PR's runs only; no manual dispatch
                                                                              (S10 convention)
LIVE DOCUMENTATION:      this document (§13 ledger)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s10-p3b.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized        (primary session freeze)
                         semantic commits: authorized                         (primary session freeze)
                         branch push: authorized                (working rules §21 default, not narrowed)
                         PR creation / update: authorized       (working rules §21 default, not narrowed)
                         CI repair to review readiness: authorized (working rules §21 default)
                         merge: explicit operator authorization only
POST-MERGE SYNC OWNER:   the S10 planning session owns step-17 §15 and overall.md; the implementation
                         session owns this document, the merge identity, the evidence and the
                         deviations (the workflow's default)
STOP CONDITION:          READY FOR OPERATOR REVIEW — DO NOT MERGE
```

## 12. Questions

**[operator]** marks a question only the operator can answer; **[primary]** one the primary session may
rule. Each carries a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **Q-P3b-1 [primary]** | Assign the number for DEP-P3b-a (our reconnect loop; `websockets`' iterator, `tenacity` and `backoff` declined). Is a DEP record the right kind, or should the cursor-authority rule (D-P3b-2, D-P3b-3) also get an ARC record? | One DEP number. D-P3b-2/3 are the SDK's realization of step-17 §3.4.3 and P4's R-P3b-1, already recorded there; an ARC record would be a second account of one rule. |
| **Q-P3b-2 [primary]** | Composition (`ResumingSeat` over `SeatSession`) rather than making `SeatSession` itself resumable; and the names `ResumingSeat`, `Seen`, `SeatLost`, `PerceivedBatch`, `CursorSource`, `CursorCell`. | Accept. `SeatSession` stays the scriptable one-connection primitive P3 froze; the names say what each is in protocol terms and collide with no defined term of `CORE_CONCEPTS.md`. |
| **Q-P3b-3 [primary]** | Remove `SeatSession.perceived` and `perceived_cursor` (S11-C's interim surface, D-SC16) in favour of the sink, with no adapter. | Yes: only SDK tests use them; the list grows without bound and its cursor advances on receipt, which R-P3b-1 forbids (`CLAUDE.md` §4 rule 12). |
| **Q-P3b-4 [operator — product policy for P6, recorded here]** | A cognition process that restarts inside its seat's hold cannot rejoin without the old `resume` secret: the seat is `held`, and a join is `seat_occupied`. Options: (a) P6 persists the secret beside its memory store; (b) P6 joins with `take_over: true`; (c) P6 waits out the hold. P3b supports all three and decides none. | (a). The secret is the only proof that a seat is this process's own: `take_over` cannot tell a seat the process held from one a human holds (AC-5), and waiting idles the Person for the whole hold. The secret is worth little if it leaks: it works for one seat, only inside the hold, and dies at the next welcome. P6 stores it beside the memory store, in the operator's cognition directory, and rewrites it after every rejoin (`Seen.connection` changes; `welcome.resume` is the new one). P3b exposes `resume` on the first join and `welcome.resume` on every connection; AP3b-10 exercises (a). Decided finally in P6's design. |
| **Q-P3b-5 [primary]** | The SDK's own buffer bound and its overflow rule (D-P3b-4): 4 096 facts, local `lagged` → rejoin. | Accept. It reuses the protocol's own bound and rule, avoids the submit deadlock of a paused reader, and keeps memory bounded on a 300-day backfill. |
| **Q-P3b-6 [primary, for S11]** | F-P3b-2: on a world without a save no client can ask for the live stream alone, since the head is not on the wire. Raise R-S11-P3b-1 with S11 (head on `welcome`, or `since: "live"`)? | Record it for S11 and do not block P3b: S10's runs are on persisted worlds (QS11C-2), and P3b treats the refusal as terminal. |
| **Q-P3b-7 [primary]** | Defaults of `ReconnectPolicy`: immediate first attempt, then 0.1 s doubling to 5 s, jitter 50 %, give up after 120 s. | Accept. Inside the server's default 30 s hold the seat makes about ten attempts; 120 s covers a server restart on a 300-day save (CA-13 measured its backfill off the world thread). |
| **Q-P3b-8 [primary]** | AP3b-9 drives facts with a second seat speaking lines (no `--town`) rather than S11-C CA-2's paced town. | Accept: deterministic, faster, and it makes "facts recorded while away" certain, so the backfill mutation is always red. CA-2 already covers the paced town in Rust. |

### 12.1 Rulings, 2026-10-10 (operator and primary session, at freeze)

Relayed by the coordinator: "operator accepted all P3b recommendations on 2026-10-10 (incl. Q-P3b-4: P6
persists the resume secret next to its memory store)", with the primary session's answers below.

| ID | Ruling | Status |
| --- | --- | --- |
| Q-P3b-1 | **DEP-44** for the own-reconnect-loop record; no ARC record. DEP-P3b-a is replaced throughout. | closed |
| Q-P3b-2 | Accepted as proposed: composition, and the names. | closed |
| Q-P3b-3 | Accepted: `SeatSession.perceived` and `perceived_cursor` are removed without an adapter. | closed |
| Q-P3b-4 | **Operator:** accepted as recommended — P6 persists the resume secret next to its memory store, rewritten after every rejoin. Binding on P6 (R-P6-P3b-1). | closed |
| Q-P3b-5 | Accepted: 4 096 facts; local `lagged` → rejoin. | closed |
| Q-P3b-6 | R-S11-P3b-1 recorded for S11; not blocking P3b. | closed for P3b; open with S11 |
| Q-P3b-7 | Accepted: the `ReconnectPolicy` defaults as stated. | closed |
| Q-P3b-8 | Accepted: AP3b-9 … AP3b-11 drive facts with a second seat's lines, without `--town`. | closed |

## 13. Ledger (live during implementation)

### 13.1 Planning evidence

```text
E-P3b-0  Planning base origin/main @ bb62edf. Read in this session: CLAUDE.md; the structured-coding
         SKILL.md, agent-workflow.md, adaptation.md, pr-design-requirements.md,
         test-ci-gate-rules.md, implementation-working-rules.md (contract block);
         .structured-coding/standards.md; step-17-cognition.md §§3.2–3.4, 3.6, 10, 11.1, 15;
         step-12-server.md §17 (incl. §§17.13–17.15); server/PROTOCOL.md §§4–5; pr-s10-p3-python-sdk.md
         §§1–8, 10; pr-s10-p4-memory.md (§3 anchors, D-P4-4/5, §11, §12); pr-s10-p5-backends.md F-P5-4;
         pr-s10-p5b-hosted-subscriptions.md F-P5b-1; overall.md (F-P5-4, decision numbers);
         sdk/python/** (every source file and test); server/tests/frames/*.json;
         server/src/protocol/fact.rs backfill_frames; tools/cli/src/main.rs Server;
         tools/cli/tests/perceived.rs (CA-2); scripts/ci_layer.py --list python;
         .github/workflows/ci.yml python job; worlds/social-cafe people and seats;
         websockets 17.2 asyncio/client.py and client.py (installed in the repository's .venv).
         No code was run for evidence; no number in this design comes from a measurement.
```

### 13.2 Evidence (filled during implementation)

```text
E-P3b-1  Implementation base: origin/main @ f4ed913 (merge of #149, this design frozen), which
         contains #142 (7eed282). Worktree /Users/yuema137/mineworld-worktrees/impl-p3b, branch
         mvp0/pr-s10-p3b, sole writer. Re-read in the implementation session (2026-10-10): CLAUDE.md;
         structured-coding SKILL.md, agent-workflow.md, adaptation.md, implementation-working-rules.md
         and test-ci-gate-rules.md in full; .structured-coding/standards.md; this design in full;
         step-17 §3.4.3, §15.2; server/PROTOCOL.md §§4–5; every sdk/python source and test file.
         Test count at the base: `uv run --locked pytest sdk/python -m "not real_server" -q` → 46
         passed, 4 deselected (50 collected; the 4 are test_real_server.py's real_server tests).
         Local Python 3.14 (uv's .venv); CI runs 3.12.
E-P3b-2  C0: check_doc_headings.py → "193 numbered sections across 26 documents, none duplicated",
         rc 0; check_decision_ids.py → "104 decision ids, all distinct", rc 0.
E-P3b-3  C1: DEP-44 appended to docs/DECISIONS.md; check_decision_ids.py → "105 decision ids, all
         distinct"; check_doc_headings.py → "193 numbered sections across 26 documents, none
         duplicated".
E-P3b-4  AP3b-1 red first (C2). The test, written against main's API (perceived=PerceivedJoin), on
         base code: `uv run --locked pytest sdk/python/tests/test_session_state.py -k lagged -q`
         → "AssertionError: ProtocolViolation('a refusal naming no request: lagged')", 1 failed.
E-P3b-5  C2 working tree (base bf44367+C1 e90392c + C2 diff): `pytest sdk/python -m "not
         real_server"` → 53 passed, 4 deselected; `pytest sdk/python -m real_server -v` → 4 passed
         (9.6 s, the binary built from this worktree); ruff check, ruff format --check, pyright
         (0 errors) clean.
         Tooling note: a `uv run pytest … | tail` pipeline can keep the shell waiting after pytest
         has exited; runs are written to files instead (no effect on results).
```

### 13.3 Deviations (filled during implementation)

```text
DV-P3b-1 (bounded; C1) — an audit fact of §3 corrected, the decision unchanged.
  Previous assumption: websockets' reconnecting iterator draws its delays from module globals read
    from WEBSOCKETS_BACKOFF_* (§3 anchor, §4.3 DEP-44 text).
  Audit evidence: .venv websockets 17.2 asyncio/client.py: `connect(..., reconnect_delays=backoff)`
    (l. 283) accepts a replacement delay generator; `__aiter__` (l. 592–631) sleeps with
    asyncio.sleep, logs every retry through self.logger, backs off only when opening fails, and
    reopens a connection that ended after it was yielded at once, resetting the backoff.
  Corrected understanding: the delay *sequence* is injectable; the default still reads the
    environment and the global generator; the sleep is not injectable; the protocol decisions are
    still absent.
  Implementation consequence: none. DEP-44 records the corrected facts; the choice stands.
  Validation consequence: none.

DV-P3b-2 (bounded; C2) — the session errors move to errors.py.
  Reason: session.py reached 542 lines with C2's additions; the design requires < 500 (§4.1, C2 review).
  Change: JoinRefused, ProtocolMismatch, ForeignObserver, ProtocolViolation, SessionClosed now live in
    mineworld_sdk/errors.py and are imported (and still exported) by session.py; every existing import
    path keeps working. resuming.py imports them from errors.py without depending on session's internals.
  Validation: the unchanged tests importing them from mineworld_sdk.session pass.

DV-P3b-3 (bounded; C2) — a perceived frame on a join that asked for none is a ProtocolViolation.
  Reason: with no sink there is nowhere to hand it; PROTOCOL.md §5's table sends perceived "only to a
    connection whose join carried perceived". Before, such frames were silently kept.
  Validation: covered by review; no client of the SDK receives such a frame from the real server.
```

### 13.4 Mutations (filled during implementation)

Each planted on the working tree, run, and reverted; the reverted tree re-run green.

| Criterion | Mutation | Observed | Verdict |
| --- | --- | --- | --- |
| AP3b-1 | base `route` (tokenless refusal → `ProtocolViolation`) | `test_the_lagged_sequence_…` FAILED: `ProtocolViolation('a refusal naming no request: lagged')` | red, as required |
| AP3b-2 | `ConnectionOrder.admit` check 2 disabled | `test_a_broken_perceived_stream_fails_closed[duplicate]` FAILED; the other three passed | red, as required |
| AP3b-14 | `support.run` always on the default loop | macOS: 3 passed (the default loop there is the selector loop) | inert locally, as the design predicts for non-Windows; the Windows leg of the PR's CI is the positive evidence; the red Windows run is not observed (no CI run beyond the PR's) |
