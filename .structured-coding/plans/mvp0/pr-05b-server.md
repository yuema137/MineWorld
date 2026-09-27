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
| 3 | the acceptance tests: two real clients on real sockets | `[x]` | `[x]` | `[x]` |
| 4 | the drop policy made observable, terminal validation and closeout | `[x]` | `[x]` | `[x]` |

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

### Commit 3 — the acceptance, over real sockets

```text
Validation:
  command:      cargo test -p mineworld-server
  environment:  rustc/cargo 1.97.1, macOS (Darwin 25.2.0), Apple silicon; loopback TCP
  result:       15 unit + 4 headless + 8 socket + 1 doc = 28 passed, 0 failed
  runtime:      socket suite 0.21 s, headless suite 0.41 s
  verdict:      PASS
```

The test world is two stub System Packs written the way a System Pack writes one — `placement`
owns a `Room` component and provides `place`; `chatter` provides `speak` and `whisper` and emits
`spoke` (`Visibility::Public`) and `whispered` (`Visibility::Participants`). Four people: alice and
carol in the cafe, bob and dave in the street. Two seats: alice and bob.

#### The transcript, from a run of two real clients against one server

Captured from a throwaway test (deleted before the final head) that printed the frames verbatim.
Trimmed only where a line repeats.

```text
[server] 127.0.0.1:61638

[2d] {"t":"welcome","protocol":1,"seat":"alice","observer":"1","world":{"protocol":1,"at":0,
      "entities":4,"systems":[{"system":"placement","enabled":true},
      {"system":"chatter","enabled":true}],"seats":["alice","bob"],"clients":1}}
[3d] {"t":"welcome","protocol":1,"seat":"bob","observer":"2", ... "clients":2}}

[2d] observation seq 1  observer "1"  entities "1" (cafe), "3" (cafe)
[3d] observation seq 1  observer "2"  entities "2" (street), "4" (street)

[2d] {"t":"submit","token":"2d-1","request":{"actor":"1","action_type":"speak","target":"2", ...}}
[2d] {"t":"result","token":"2d-1","action_id":"1","result":{"accepted":{"events":["1"]}}}

[2d] seq 3 observer 1 events [{"id":"1","event_type":"spoke","subjects":["1","2"],
      "participants":["1","2"],"caused_by":{"action":"1"},
      "payload":{"event_type":"spoke","schema_version":1,
                 "payload":{"to":"2","words":"hello from the cafe"}},
      "visibility":"public","provenance":{"emitted_by":"chatter","controller_decision":"1"}}]
      affordances [{"action_type":"speak","target":"3","available":true, ...}]
[3d] seq 3 observer 2 events [{"id":"1", ... identical envelope ... }]
      affordances [{"action_type":"speak","target":"4","available":true, ...}]

[2d] {"t":"set_state","entity":"1","money":5000}
[2d] {"t":"refused","code":"unknown_frame","detail":"this protocol has no frame of kind
      \"set_state\"; a client may only join or submit"}
```

Read out of it, claim by claim:

```text
A1  two clients, two observers ("1" and "2"), and two different entity lists and affordance lists
    from one world at the same instant. Neither list is a filtered copy of the other: the
    affordance the server computed names entity "3" for one client and "4" for the other.
A2  one fact, EventId "1", reached both clients — the same identity, the same
    caused_by {"action":"1"} and the same provenance controller_decision "1". That is the
    identity-not-appearance standard §1.3 requires, at this layer.
A4  the assertion was refused by name, and nothing about the world changed.
```

Also visible, and each of them a thing the transport did not do:

```text
action_id "1"       allocated by the server; the client sent no identity and no instant
"to":"2"            an EntityId INSIDE an event payload, as a decimal string — the exact position
                    FINDINGS.md F2 measured as unreachable by a protocol-level encoder. This
                    server has no encoder; the contract did it.
seq 1,2,3           the per-connection counter F7 asked for, against an `at` of 0 throughout
```

#### The eight socket tests

```text
two_clients_connect_at_once_and_each_receives_its_own_observers_observation     A1
an_intent_from_one_client_produces_an_event_both_clients_are_entitled_to_see    A2
a_fact_only_one_observer_is_entitled_to_reaches_only_that_client                A2, the other half
an_identity_inside_an_event_payload_reaches_a_client_as_a_decimal_string        F2
killing_one_client_leaves_the_world_running_and_the_other_client_unaffected     A3
a_message_that_asserts_state_is_refused_and_changes_nothing                     A4
a_client_cannot_ask_the_world_to_act_as_somebody_else                           NETWORKING §2
nothing_streams_until_a_seat_is_granted_and_a_seat_is_held_for_the_connection    INV-13
```

`A3` is the one worth stating precisely: the 2D client is **dropped**, not closed — no close frame,
which is what a crashed client does. The 3D client's stream then advances (`seq` before < after),
its own `speak` is still accepted, and `GET /status` answers `clients: 1, entities: 4`.

#### The four headless tests

The host without a transport, which `ENGINEERING_STANDARDS.md` §22 and `NETWORKING.md` §10
require to be possible, and which owns the two claims a socket cannot show honestly:

```text
two_observers_of_one_world_receive_two_different_observations
the_world_keeps_working_while_a_subscriber_never_reads          the no-blocking constraint
the_server_allocates_the_identity_of_every_request              ActionIds 1 then 2, from the server
a_seat_that_does_not_exist_and_an_actor_that_is_not_the_observer_are_both_refused
```

`the_world_keeps_working_while_a_subscriber_never_reads` is deliberately *not* a socket test: an
operating system's own send buffer would absorb a slow reader long before the server's bounded
channel filled, so a socket test of that claim would pass without exercising it. At the host seam
the channel genuinely fills — 400 ms of sweeps at 20 ms against a backlog of 8 — and the world
still answers `status` and still dispatches.

### Commit 4 — the drop policy made observable

Found by the logic review of commit 1's own code: `dropped`, `unscheduled` and `faults` were
counted in the world thread and reachable by nobody, so the three things this server does *quietly*
— losing a frame for a client that is not reading, failing to queue a deferral, surviving a system's
broken contract — were invisible to whoever runs it. A number that only appears in a comment is not
a policy anybody can audit.

`WorldSummary` therefore carries `observations_dropped`, `deferrals_unscheduled` and `faults`, which
puts them in `GET /status` and in the `welcome` frame. The no-blocking claim gains a direct
observable, and this is the measured one:

```text
after 400 ms of 20 ms sweeps with one subscriber never reading, backlog 8:

WorldSummary { protocol: 1, at: WorldTime(0), entities: 4,
               systems: [placement (enabled), chatter (enabled)],
               seats: [alice, bob], clients: 2,
               observations_dropped: 24, deferrals_unscheduled: 0, faults: 0 }
```

24 frames were dropped rather than waited on, across two subscribers that were not reading, while
`status` answered and dispatch continued. That is the constraint "the server never blocks the world
on a client" as a number rather than as a description.

### Mutation evidence

Five mutations, each applied to committed production code, each reverted after the run. Every one
was killed, which is what makes the tests above load-bearing rather than decorative.

| # | Mutation | Expected | Observed |
| --- | --- | --- | --- |
| M1 | `sweep` computes every client's observation for `subscribers[0]` | scoping fails | `two_clients_connect...` FAILED, `two_observers_of_one_world...` FAILED |
| M2 | the actor check in `submit` is disabled | impersonation succeeds | `a_client_cannot_ask_the_world_to_act_as_somebody_else` FAILED, with the server answering `Accepted { events: [EventId(1)] }` to a request from the wrong client |
| M3 | `decode` no longer separates an unknown frame kind | a state assertion is merely "malformed" | unit test FAILED (`left: MalformedFrame, right: UnknownFrame`) and `a_message_that_asserts_state...` FAILED |
| M4 | recorded events are dropped instead of remembered | no client learns of any fact | 3 socket tests FAILED |
| M5 | `sweep` delivers only to the first subscriber | the second client receives nothing | 4 socket tests FAILED, three of them by timing out waiting for a frame |

### Terminal validation

```text
command:  cargo fmt --all --check                                        clean
          cargo check --workspace --all-targets                          clean
          cargo clippy --workspace --all-targets --all-features -D warnings   clean
          cargo test --workspace                                         172 passed, 0 failed
          (144 before this PR + 28 new: 15 unit, 4 headless, 8 socket, 1 doc)
verdict:  PASS
```

## 7. Decisions, discoveries and limitations

### DECISION — the world lives on its own thread, and the transport only sends it messages

Question: how does an `axum` handler reach a `World`?

Evidence: `kernel/src/registry.rs:65` stores `Box<dyn DynSystem>`, unbounded, so `World: !Send`.
The spike answered this with `Arc<Mutex<SpikeWorld>>` locked across `.await` points, which works
for one client and couples two.

Options: (a) `Arc<Mutex<World>>` — impossible, `World` cannot be sent to the thread that would
build it, let alone shared; (b) a `!Send` single-threaded runtime for everything, which gives up
axum's multi-threaded serving; (c) a world thread built from a `Send` closure, reached by channels.

Chosen: (c). It is the only one that compiles without a `Send` bound on systems, and it makes two
frozen constraints structural: the world is never blocked by a client (bounded channels,
`try_send`, drop on full) and dispatch cannot deadlock against streaming (no shared lock exists).

Validation: `the_world_keeps_working_while_a_subscriber_never_reads`, plus every socket test that
has two clients on one world at once.

### DECISION — `Perception` is a seam in the server, not a new kernel hook

Question: who produces the per-observer `Observation`?

Evidence: the `System` trait has four methods and none of them observes; `ARCHITECTURE.md` §6 and
`contracts/src/observation.rs` both assign the decision to perception systems; and
`ENGINEERING_RULES.md` §8 forbids the transport deciding it.

Options: (a) add a fifth method to `System` — a kernel contract change, out of this PR's scope and
a material deviation; (b) compute perception in the server — forbidden; (c) one seam in the server
crate that a perception system's adapter implements.

Chosen: (c), with `PerceivesNothing` as the default so that the safe direction is the default.
PR 05a's `PresenceSystem` is adapted onto it in 05c/05d. If a fifth `System` method turns out to
be the right long-term home, that is a kernel decision with the seam's shape as evidence for it —
not something this PR should have decided.

### DISCOVERY — a World Pack cannot seed component state without inventing `ActionId`s

Found while writing the test world. Component state is writable only by its owning system, only
through a `WorldView`, and only during `resolve` or `react`; `install` may declare tables and
nothing else. So placing four people in two rooms means dispatching four `place` intents, and
`ActionIntent::new` needs an `ActionId` that, at world-assembly time, no allocator has issued —
the world's own event counter is crate-private and the server's request allocator has not started.

The test world works around it by allocating from `9_000_000` upwards, far above the server's
allocator, so that assembly cannot collide with a client's request. That is fine for a test and it
is **not** an answer for PR 05c, which will load a real World Pack and hit the same wall for real.
Recorded here rather than solved here: the options are a kernel-side authored-state path, a
`HostConfig` that seeds the allocator above whatever assembly used, or an assembly-time allocator
the pack loader owns. Whichever 05c chooses is a decision with consequences for the event log, and
it belongs to the PR that has to make it.

### DISCOVERY — `Observation`'s payload type differs in each direction, and it has to

Outbound the wire carries `Observation<serde_json::Value>`; inbound a request arrives as
`ActionRequest<WirePayload>` and is re-parameterized to the kernel's `ActionRequest<Vec<u8>>`. The
asymmetry is not an oversight: a perception implementation *builds* its observation and can build
it with any payload type, while a transport *receives* a request and must hand the kernel bytes,
and `ActionRecord` deliberately cannot be constructed from an `ActionTypeId` plus a payload without
naming the action's Rust type. The conversion therefore goes through the contract's own `serde`
rather than through a mirror of its shapes — one JSON round trip per submitted intent, on the rare
frame rather than the frequent one.

### DEVIATION (bounded) — `tokio-tungstenite` as a dev-dependency

The acceptance requires real WebSocket clients, and `DEP-3` chose the axum/tokio stack without
naming a client. `tokio-tungstenite` is the client half of that same stack — `tungstenite` is what
axum's own WebSocket support is built on, and it is already in the dependency graph. It is a
**dev**-dependency: no production code depends on it.

`ENGINEERING_STANDARDS.md` §16 asks for a `docs/DECISIONS.md` record for a substantial dependency.
This is recorded here rather than there, because `DEP-3` already froze the transport stack and
because `docs/DECISIONS.md` is being touched by concurrent sessions on other branches. **For the
operator:** if a `DEP-3` addendum naming the test client is wanted, it is a one-paragraph edit and
this is the flag for it.

### LIMITATION — the clock is provisional, and it is wall time

`WorldTime` advances one simulated second per real second from a configured epoch, because S4's
clock and scheduler do not exist yet and dispatch has to be told an instant. Consequences, all
recorded rather than hidden: a replay would not reproduce these instants; several observations
share one `at` (which is why the protocol frame carries `seq`); and a system that defers work gets
its deferral counted and logged rather than queued, because there is no queue. `S4` replaces the
clock behind `HostClock` and the deferral count is the seam it will plug into.

### LIMITATION — a `KernelError` out of dispatch does not stop the world

`kernel/src/dispatch.rs` says an `Err` is a bug in a system and that the caller should "stop the
world and report it". This server counts it, logs it, and answers the client `dispatch_failed`,
because tearing down a world other clients are connected to on one system's bug is a worse
behaviour for a server and the right policy needs persistence (S5) to be honest — a world that can
be restored can be stopped. Recorded as a deliberate divergence from the kernel's advice, with the
fault counter as the evidence it is not silent.

### LIMITATION — no Godot client ran

This PR's acceptance is the server's. `AC-13`/`AC-15` and the two real clients are PR 05d's, and
nothing here claims them. What is claimed is that two real WebSocket clients, over real sockets,
receive correctly scoped observations from one world at once.

### LIMITATION — `server/tests/two_clients.rs` is 550 lines

Past the 500-line review threshold, which is a trigger rather than a rule
(`ENGINEERING_STANDARDS.md`). It is one coherent responsibility — the acceptance list of this PR,
plus the WebSocket client and HTTP helper those tests are written against — and splitting it would
put the acceptance in two files and its instruments in a third. Recorded so the threshold is
answered rather than ignored.

### LIMITATION — logging is `println!`

No `tracing` subscriber, no levels, no structured fields. Adequate for a development server and
deliberately not a logging architecture chosen in passing; an observability decision belongs with
the deployment work `NETWORKING.md` §7 anticipates.

## 8. Closeout and handoff

```text
PR                   MVP0 PR 05b — the world server
branch               mvp0/pr-05b-server
base                 main @ 3d834f6
lifecycle            READY FOR OPERATOR REVIEW — NOT MERGED, NO PR OPENED
implementation       CLOSED / AWAITING OPERATOR ACTION

commits              1  feat(server): host a world on its own thread and speak a closed protocol
                     2  feat(server): serve the world over HTTP and WebSocket
                     3  test(server): two real clients, real sockets, one world
                     4  feat(server): report what the world drops, defers and survives
working tree         clean at the final commit
validation           fmt / check / clippy -D warnings clean; cargo test --workspace 172 passed
                     (144 before this PR + 28 new), 0 failed
CI                   none exists in this repository yet; the four commands above are the gate
```

### Files

```text
server/Cargo.toml          the crate
server/README.md           human orientation
server/PROTOCOL.md         the frame-level specification a client is written against
server/src/lib.rs          the crate's map, and where each frozen invariant is enforced
server/src/protocol.rs     the frames, the refusal codes, the payload conversion  (+ tests/)
server/src/perception.rs   the one seam a hosted world provides
server/src/host.rs         WorldHost, HostedWorld, SeatRoster, HostConfig, Seated, Submitted
server/src/runtime.rs      the world thread: clock, allocator, subscribers, sweep
server/src/session.rs      one connection: handshake, then observations out and requests in
server/src/app.rs          /health, /status, /ws
server/src/main.rs         the binary
server/tests/support/      two stub System Packs and a perception that scopes by room
server/tests/two_clients.rs the acceptance, over real sockets
server/tests/headless.rs   the same host with no transport at all
Cargo.toml                 workspace member + the transport dependencies
```

### What the operator may want to decide

```text
1  a DEP-3 addendum naming tokio-tungstenite as the test client, if the DECISIONS record should
   mention it (§7 explains why it was not edited here: concurrent sessions are touching that file)
2  whether the World-Pack seeding gap in §7 is answered in PR 05c or raised to the step level
3  post-merge: CLAUDE.md §1.1's "the contracts and kernel crates exist" inventory sentence,
   docs/MVP_STATUS.md's networking row and the "World server" artefact row, and README.md's status
   paragraph all become stale. They are the planning session's to synchronize, and this PR does not
   touch shared status documents while sibling PRs are in flight.
```

### For PR 05c and 05d

```text
05c  fills the WorldHost::spawn closure: install the World Pack's systems, create its entities,
     name its seats (SeatRoster of entity keys), and pass PresenceSystem's adapter as the
     Perception. Nothing in server/src needs to change for that; main.rs's closure is the seam.
05d  writes both clients against server/PROTOCOL.md. What changes in the spike clients is the frame
     shape (join/submit/welcome/observation/result/refused) and that they no longer invent an
     ActionId; everything they already do with Observation, Affordance and Rejection stands.
```
