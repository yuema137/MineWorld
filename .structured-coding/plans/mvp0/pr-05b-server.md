# PR 05b — the world server

## DESIGN FROZEN

Design revision: `step-05-vertical-slice.md` §2 "PR 05b — The server", plus §1.3's frozen
invariants. This file is the PR-level ledger for that frozen scope; it adds no scope of its own.
Approved by / evidence: operator kickoff of this implementation session (2026-09-26), which
states the scope, the acceptance list, the constraints and the publication authority verbatim.
Implementation base: `main` @ `3d834f6`
Branch: `mvp0/pr-05b-server`
Lifecycle: FROZEN

---

## 1. What this PR is

`server/` as a crate producing a binary that hosts **one** world and serves clients:

```text
HTTP        control and status
WebSocket   the live connection: per-observer observations out, intents in
```

The step document's acceptance, restated because it is what this PR is judged on:

```text
A1  two WebSocket clients connect at once, and each receives observations scoped to its own
    observer
A2  an intent from one produces an event that both are entitled to see
A3  killing one client leaves the world running and the other client unaffected
A4  a message that asserts state rather than requesting an action is rejected
    (NETWORKING.md §2)
```

## 2. Implementation contract

```text
PROJECT / PR:        MVP0 PR 05b — the world server
PRIMARY DESIGN DOC:  this file
RELATED / BINDING:   .structured-coding/plans/mvp0/step-05-vertical-slice.md §1.3, §2
                     docs/NETWORKING.md (§1 one binary, §2 authority, §3 transport,
                       §4 rate separation, §5 message classes, §10 networking owns no state)
                     docs/DECISIONS.md DEP-3 (tokio + axum), DEP-9
                     docs/ENGINEERING_RULES.md §§4, 7-9, 12; docs/ENGINEERING_STANDARDS.md
                     contracts/src/action.rs (ActionRequest, ActionResult correlation reasoning)
                     contracts/src/observation.rs (INV-13), kernel/src/{world,dispatch,view}.rs
                     spike/server/src/{main,world}.rs, spike/FINDINGS.md F2, F4, F7, F8, F9
IMPLEMENTATION BASE: main @ 3d834f6 — contracts + kernel merged and green (144 tests)
BRANCH:              mvp0/pr-05b-server

APPROVED SCOPE:
  a new crate under server/, added to the root workspace; its protocol specification and
  README; its integration tests with the crate's own stub systems and stub perception.
  Nothing under contracts/, kernel/, systems/, worlds/, clients/ or spike/ is modified.

NON-GOALS (from step-05 §1.2 and the kickoff):
  systems/conversation and systems/presence  — a sibling session owns those paths
  World Pack loading and the CLI             — PR 05c
  the clients                                — PR 05d
  scheduler, processes, durable persistence  — S4, S5
  authentication beyond a seat name          — NETWORKING.md §9 keeps it to invite + nickname

FROZEN INVARIANTS:
  - INV-13: each client receives what ITS observer may perceive; a client cannot widen its own
    perception by asking, and the widening must be impossible by construction
  - INV-6: a client submits an ActionRequest; the SERVER allocates the ActionId and the instant
  - the transport does not re-implement id encoding (DEP-3, PR 04): every id on the wire is
    encoded by mineworld-contracts' own serde
  - NETWORKING.md §2: a frame that asserts state is a protocol violation and is rejected
  - NETWORKING.md §1: one binary for localhost and network alike; no separate single-player path
  - the server never blocks the world on a client, and observation streaming and dispatch never
    deadlock each other
  - no world rule in the transport: distance, availability, permission and perception are
    decided by systems (ENGINEERING_RULES.md §8)
  - fmt, check, clippy -D warnings and cargo test --workspace clean at the final HEAD

ENDPOINT AUTHORITY:
  implementation + local validation  authorized   source: operator kickoff, this session
  semantic commits                   authorized   source: operator kickoff ("Commits and pushes
                                                   on your branch are authorized")
  branch push                        authorized   source: operator kickoff
                                                   ("git push -u origin mvp0/pr-05b-server")
  PR creation / merge                FORBIDDEN    source: operator kickoff ("No merge, no PR")

TEST OWNERSHIP:
  STATIC       fmt, check, clippy -D warnings; clippy.toml's HashMap/HashSet ban.
  UNIT         protocol frame encoding/decoding: which frames exist, which are refused, and
               that an id reaches the wire as a decimal string produced by the contract.
  INTEGRATION  (this PR's real evidence) a server on an ephemeral port, real WebSocket clients
               over a real TCP socket, two at once: scoping, entitlement, kill-one, state
               assertion refused, and the world unblocked by a client that stops reading.
  GATE 1 (real LLM):      NOT REQUIRED — no LLM-facing semantics exist in this PR.
  GATE 2 (real lifecycle): the integration tests ARE the real-lifecycle evidence: real sockets,
               real axum, real tokio, real kernel dispatch. No Godot client runs here; that is
               PR 05d's acceptance, and this PR must not claim it.

NORMAL STOP CONDITION:
  PR 05b implemented, validated, committed and pushed. NO PR. NO MERGE.
```

## 3. Source audit, and the three things it forced

Read before writing: `kernel/src/{lib,world,dispatch,view,registry,system}.rs`,
`contracts/src/{action,observation,event,time,ids}.rs`, `spike/server/src/{main,world}.rs`,
`spike/FINDINGS.md`, `docs/NETWORKING.md`, `docs/DECISIONS.md` DEP-3/DEP-9.

### 3.1 `World` is not `Send`, so the world lives on a thread of its own

`kernel/src/registry.rs:65` holds `system: Box<dyn DynSystem>` with no `Send` bound, so `World`
is `!Send` and cannot be moved into a `tokio::spawn`ed task or an `std::thread::spawn` closure.

Consequence, and it is the architecture rather than a workaround: the world is **built inside**
its own thread from a `Send` closure, and the transport reaches it only by message passing.
`WorldHost::spawn(build)` takes `FnOnce() -> Result<HostedWorld, E> + Send + 'static`; the
`World`, the systems and the perception implementation never cross a thread boundary. What
crosses is plain data — commands in, `Observation`s and `ActionResult`s out — all of which are
`Send`.

This is also what makes the no-deadlock constraint structural instead of careful: there is no
lock shared between dispatch and observation streaming, because there is no lock at all. The
spike's `Arc<Mutex<Shared>>` held across `.await` points is exactly what a second client turns
into a latency coupling, and it is not reproduced here.

### 3.2 The kernel has no perception hook, so hosting needs one seam

`System` declares four methods — `install`, `validate`, `resolve`, `react` — and none of them
produces an `Observation`. `ARCHITECTURE.md` §6 and `observation.rs` both say *what* is
perceivable is a perception system's decision. The server therefore cannot compute perception
(that would be a world rule in the transport) and cannot get it from the `System` trait either.

So the crate defines one seam, `Perception`, and hosts whatever implements it:

```text
fn observe(&self, ctx: &PerceptionContext<'_>) -> Observation<serde_json::Value>
    ctx: the read-only world, the observer, the instant, and the recent event log
```

`PerceivesNothing` is the default and the safe one: an observation with nothing in it, which
`Observation::new`'s own documentation names as the safe starting point. PR 05a's
`PresenceSystem` is adapted onto this seam in PR 05c/05d without the transport changing.

### 3.3 Payload types: `Value` outbound, JSON bytes inbound

The kernel's action and event payloads are `Vec<u8>` (`ActionIntent` and `EventEnvelope` default
`P = Vec<u8>`), and `FINDINGS.md` F8.2 records that this default is unusable on a JSON wire — a
payload would reach a client as an array of byte integers. The wire therefore carries
`Observation<serde_json::Value>`, which is the instantiation F8.2 asked S11 to state.

Outbound needs no work from the transport: the perception implementation produces
`Observation<Value>` directly, and the system that declared an event type is the only code
entitled to re-encode that event's payload — which is where `EventRecord::new::<E>` requires it
to happen anyway.

Inbound does need a conversion, because the client sends JSON and `World::dispatch` takes
`ActionIntent<Vec<u8>>`. `ActionRecord` cannot be constructed from an `ActionTypeId` plus a
payload without naming the action's Rust type — deliberately, so a record cannot be mislabelled —
so the conversion goes through the contract's own `serde` in both directions rather than through
a hand-written mirror of its shapes: the frame deserializes as `ActionRequest<WirePayload>`,
where `WirePayload` captures any JSON value as the bytes of its canonical JSON text and
serializes exactly as `Vec<u8>` does, and one `serde_json` round trip yields the
`ActionRequest<Vec<u8>>` the kernel wants. Every contract check, including the
envelope/payload agreement check a hand-built client frame can fail, runs on the way in. The
cost is one small JSON round trip per submitted intent, which is a rare frame; observations,
the frequent ones, are serialized once and not converted at all.

The payload bytes are the JSON text of the payload, which is already this repository's
convention: `kernel/tests/dispatch.rs:474` encodes every emission payload with
`serde_json::to_vec`.

## 4. The protocol, as designed

Specified for implementers in `server/PROTOCOL.md`; summarized here because the acceptance
depends on it.

```text
HTTP    GET /health   liveness, answered by the transport
        GET /status   the world's own answer: time, entities, systems, seats, clients
        GET /ws       the WebSocket upgrade

client -> server    join    { seat }            the ONLY way to acquire an observer
                    submit  { token, request }  an ActionRequest, plus the client's own token

server -> client    welcome     { protocol, seat, observer, world }
                    observation { seq, observation }
                    result      { token, action_id, result }
                    refused     { token?, code, detail? }
```

Four properties are load-bearing:

1. **Scope is not addressable.** A client names a *seat*; the server resolves the seat to an
   `EntityId` through a roster the world's host supplied. No frame carries an observer id, so
   there is no request that widens perception — and no frame changes the seat once joined.
2. **A client acts only as itself.** A submitted `ActionRequest` whose `actor` is not the
   session's observer is refused `actor_not_observer`. `ActionRequest::actor` is client-supplied,
   so without this check a client could ask the world to act as somebody else.
3. **Correlation lives in the frame.** `token` is the client's own opaque string, echoed on the
   `result`; `action_id` is the identity the **server** allocated, so a client can recognize
   later facts whose `Causation` names its request. This is the frame `ActionResult`'s
   documentation says the correlation belongs in, and the reason that type still carries no
   `ActionId`.
4. **Anything else is a protocol violation.** `"my money is now 5000"` does not decode as a
   frame this protocol has, and is answered `refused { code: unknown_frame }`. The refusal is
   structural rather than a check: the client frame enum has exactly two variants, and neither
   has a field that could assert state.

`seq` is a per-connection monotonic frame counter, which answers `FINDINGS.md` F7 — a client
could not otherwise order two observations, since `WorldTime` has one-second granularity and a
10 Hz stream stamps ten frames identically.

## 5. Commit plan

| # | Commit | Implementation | Validation | LLM logic review |
| --- | --- | --- | --- | --- |
| 1 | the crate: the frames, the perception seam, the host and the world thread | `[x]` | `[x]` | `[x]` |
| 2 | the transport: HTTP control plane, WebSocket sessions, the binary | `[x]` | `[x]` | `[x]` |
| 3 | the acceptance tests: two real clients on real sockets | `[ ]` | `[ ]` | `[ ]` |
| 4 | terminal validation and closeout | `[ ]` | `[ ]` | `[ ]` |

Commit 1 and commit 2 were planned as three (frames / host / transport) and became two: a commit
containing only the frames would not have a `lib.rs` that compiles, and a commit that cannot be
built is not reviewable. The boundary kept is the one that matters — everything that is *not* the
transport, then the transport — so the second commit's diff is exactly the socket-facing code.

## 6. Live evidence log

### Baseline, before any change

```text
command:  cargo test --workspace
head:     3d834f6 (main, clean tree)
result:   144 passed, 0 failed  (contracts 96 + kernel 48 across their suites)
purpose:  the number this PR must not reduce
verdict:  PASS
```

### Commit 1 — the crate, the frames, the seam and the world thread

```text
Validation:
  command:      cargo fmt --all --check
                cargo clippy --workspace --all-targets --all-features -- -D warnings
                cargo test -p mineworld-server
  purpose:      the protocol's closed vocabulary, the encoding of identity, and the conversion
                onto the kernel's payload type
  environment:  rustc/cargo 1.97.1, macOS (Darwin 25.2.0), Apple silicon
  result:       fmt clean; clippy clean; 15 unit tests passed, 0 failed; 1 doc test passed
  verdict:      PASS
```

The fifteen cover, in the order they matter:

```text
a state assertion is refused as a kind this protocol does not have      NETWORKING.md §2
a frame with no kind, and text that is not JSON, are malformed
the two frames that do exist decode, including a 64-bit target id
a request whose envelope and payload disagree is refused BY THE CONTRACT FINDINGS.md F8.1
a refusal recovers the token of the frame that failed
every identity in a server frame is a decimal string                    DEP-3, FINDINGS.md F2
a welcome names its observer as a decimal string
a rejection arrives inside a result, never as a refusal                  INV-10
a submitted payload reaches the kernel as the JSON the client sent
re-parameterizing a request changes nothing but the payload type
a correlation token is bounded, printable, and round-trips unchanged
```

Every expected JSON string in those tests is written by hand rather than produced by the code under
test (`test-ci-gate-rules.md` §25).

### Commit 2 — the transport and the binary

```text
Validation:
  command:      cargo fmt --all --check
                cargo clippy --workspace --all-targets --all-features -- -D warnings
                cargo test -p mineworld-server
                ./target/debug/mineworld-server --listen 127.0.0.1:7899   (then curl)
  purpose:      the routes answer, and the world answers /status itself rather than the transport
                answering from a cache
  result:       fmt clean; clippy clean; 15 + 1 tests still pass
  verdict:      PASS
```

The binary, run for real:

```text
[server] listening on http://127.0.0.1:7899 (ws://127.0.0.1:7899/ws), protocol 1
[server] world: 0 entities, 0 system(s), 0 seat(s) — an empty world until a World Pack can be
         loaded (PR 05c)

GET /health  ->  {"status":"ok","protocol":1}
GET /status  ->  {"protocol":1,"at":1,"entities":0,"systems":[],"seats":[],"clients":0}
```

`at` is `1` two seconds after start, which is the provisional wall clock in whole simulated
seconds — the evidence that `/status` is answered by the world thread rather than by a value the
transport captured at startup. An empty world with no seats is a valid world (`INV-12`) and the
honest thing for this binary to host until PR 05c can load a World Pack.

## 7. Decisions, discoveries and limitations

(filled during implementation)
