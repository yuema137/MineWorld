# Step 12 — S11: Server and networking

**Effort:** `mvp0` · **Parent:** [`overall.md`](overall.md) §3 S11, §4, §7
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions and coordination rulings in `overall.md` "Parallel build-out, 2026-10-08", which bind and override this document where they differ (decision numbers, protocol ownership, event perception, the shared module, digests). Superseded wording below: `DRAFT — awaiting the primary session's review`
**Author:** the S11 planning agent, 2026-10-08, in worktree `plan-s11-server`, branch `plan/s11-server`
**Base audited:** `main @ 0fd0be3` (S15's 12b merged as `9c617ed`, 12c design frozen)
**Role of this document:** step plan. PR scopes, dependencies, integration checkpoints and adversarial
criteria are decided here; each PR is detailed to the commit in its own design (or in this document,
expanded in place) and frozen in turn. Nothing here authorizes implementation.

Binding specifications: [`docs/NETWORKING.md`](../../../docs/NETWORKING.md) ·
[`docs/MVP.md`](../../../docs/MVP.md) §§6, 8, 9 · [`docs/CORE_CONCEPTS.md`](../../../docs/CORE_CONCEPTS.md)
§§2, 14, 15 · [`docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md) §§6, 9, 11 ·
[`docs/ENGINEERING_RULES.md`](../../../docs/ENGINEERING_RULES.md) ·
[`docs/ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) ·
[`docs/REUSE_POLICY.md`](../../../docs/REUSE_POLICY.md) · [`server/PROTOCOL.md`](../../../server/PROTOCOL.md)
(revision 1, the contract this step revises) · [`docs/DECISIONS.md`](../../../docs/DECISIONS.md) `DEP-3`,
`ARC-6`, `ARC-23`, `ARC-25`, `ARC-27`, `ARC-34`.

**Operator directive carried into this design (2026-10-08, relayed by the primary session).**
> 保持代码干净整洁，模块化，可插拔。遇到问题要积极寻找已有的开源方案，不要重复发明轮子，而且要对比多种实现方案，
> 不要找到一个就觉得万事大吉。当然如果没有现成的能满足我们的大目标的，那我们自己改进或者发明轮子也是可以的。

Keep the code clean, modular and pluggable; look actively for existing open-source solutions and compare
several rather than stopping at the first; build our own only where nothing existing serves the goal. §7
(reuse, a comparison table per infrastructure piece) and §8 (modularity and pluggability) answer it.

**Parallel-planning note.** Four other planning agents are writing S12, S13, S14 and Milestone E at the
same time. This document edits no shared document. Every edit it needs in `overall.md`, `MVP_STATUS.md`,
`DECISIONS.md` or another specification is *proposed* in §13 for the primary session to apply. New
decision records carry placeholders (`ARC-S11-a`, `DEP-S11-a`, …), and PRs carry placeholders
(`S11-A` … `S11-E`); the primary session assigns real numbers at freeze. **S11 owns the wire protocol**:
§5 is the versioned contract S12 and S14 design against.

---

# 1. The requirement

## 1.1 Quoted from `overall.md` §3

> ### S11 — Server and networking
>
> - **Output:** the server process: HTTP control plane, WebSocket observation streams and intent
>   submission, invite-token plus nickname authentication, multi-client sessions, admin client
>   surface.
> - **Depends on:** S7; meaningful with S8.
> - **Acceptance checkpoint:** simulation continues while clients disconnect and reconnect
>   (`AC-3`); a human takes over an existing NPC with biography, relationships, inventory and
>   employment intact (`AC-5`); several clients inhabit one world and interact with the same NPCs
>   (`AC-7`); a message asserting state rather than requesting an action is rejected (`INV-9`).

`overall.md` §4 places `AC-3` in "S11, end-to-end in S12", `AC-5` in "S11, with S10", `AC-7` in S11, and
`AC-15` in "S14, against S11's server with S12's client also connected". §7 lists S11's remaining work as
"authentication, admin frames, deltas", and carries `F-13` (a restarted `--agent` re-answers its last line)
to "S10 (cognition)"; the operator's brief for this step carries `F-13` here. Risk `R-9` assigns S11 the
rule that every cross-language boundary is verified from the far side (a real Godot client), not only by a
Rust test.

## 1.2 Quoted from `docs/MVP.md` §9

| ID | Criterion | Observable test |
| --- | --- | --- |
| **AC-3** | Renderer independence | The simulation continues correctly while the Godot client is disconnected, and a client may reconnect to the running world. |
| **AC-5** | Controller independence | A human takes control of an existing NPC without destroying that Person's biography, relationships, inventory, or employment. |
| **AC-7** | Networking | Several human clients inhabit the same world simultaneously and interact with the same NPCs. |
| **AC-15** | There is only one Alice | A 2D client, a 3D client and an agent-driven Person are connected to one running server at the same time. … |

`AC-15` is not S11's to demonstrate (it is S14's), but S11 must not break it: the existing
`tools/cli/tests/ac15_one_alice.rs` stays green through every S11 PR.

## 1.3 Quoted from the specifications this step implements

`docs/NETWORKING.md` §5 (message classes):

```text
Client → Server   authenticate · subscribe / unsubscribe observation scope · submit ActionIntent ·
                  chat / interaction request · admin command (authorized clients only)
Server → Client   world snapshot / delta · observation update ·
                  action result (accepted | rejected | ActionUnavailable) ·
                  event notification (subject to Visibility) · error
```

`NETWORKING.md` §3: HTTP is the control plane, world lifecycle and admin; WebSocket carries live
observation streams, state deltas and action submission. §4: network replication is state-delta based and
the server never blocks on a controller. §9: MVP authentication is *server invite token + player
nickname*; matchmaking, global accounts and server browsers are out of scope. §10: networking never owns
world state and must be removable — the world runs headless with nobody connected.

`docs/MVP.md` §6: up to four human players and 8–12 NPCs. `CORE_CONCEPTS.md` §14 and `MODULE_SPEC.md` §5
rule 4: rebinding a Person's controller preserves biography, relationships, inventory and employment
(`INV-1`). `ARCHITECTURE.md` §9: the server never blocks on a controller; an intent that returns late is
revalidated against current state.

## 1.4 The operator's brief for this step (2026-10-08)

Design auth, sessions, seat takeover, reconnect, an admin surface, observation deltas, **server-side
controllers so the hosted town lives** (today the walking controller does not run in `mineworld server`,
so a connected world has no walkers), and the protocol revision as an explicit versioned contract; close
`F-13`; expose facts to clients (protocol revision 1 sends none).

---

# 2. Source audit (`main @ 0fd0be3`)

Every claim below names the file and symbol it was read from. Nothing here is inferred from a document.

## 2.1 The server crate, `server/`

| File | Lines | What it holds |
| --- | --- | --- |
| `server/src/lib.rs` | 83 | the crate map and re-exports; four non-negotiables, each pointed at its enforcing symbol |
| `server/src/app.rs` | 94 | `router`: `GET /health`, `GET /status`, `GET /ws`; `serve`, `serve_with_shutdown`, `bind` |
| `server/src/session.rs` | 209 | `run`: `handshake` (loop until `join` succeeds), then `stream` (one `tokio::select!` over the observation receiver and the socket), `submit`, `receive`, `send` |
| `server/src/host.rs` | 498 | `HostedWorld` (`Ephemeral` / `Persisted`), `SeatRoster`, `HostConfig`, `Seated`, `Submitted`, `Command` (`Status`, `Join`, `Leave`, `Submit`, `Sweep`, `Shutdown`), `WorldHost` (`spawn`, `status`, `join`, `submit`, `leave`, `shutdown`), `HostError` |
| `server/src/runtime.rs` | 435 | `WorldRuntime`: the world thread. `HostClock` (one simulated second per wall second), `ActionIds`, `Subscriber`, `run` loop, `join`, `submit` (actor check, allocate, advance, dispatch, sweep), `tick`, `remember` (bounded recent window), `sweep` (one `Perception::observe` per subscriber, `try_send`), `summary` |
| `server/src/perception.rs` | 118 | the `Perception` seam (`observe(&PerceptionContext) -> WireObservation`), `PerceptionContext` (world, observer, at, `recent_events`), `PerceivesNothing` |
| `server/src/protocol.rs` | 513 | `PROTOCOL_VERSION = 1`, `ClientFrame` (`Join { seat }`, `Submit { token, request }`), `CLIENT_FRAME_TAGS`, `ClientFrame::decode`, `ServerFrame` (`Welcome`, `Observation`, `Result`, `Refused`), `RefusalCode` (9 codes), `WorldInstanceId`, `WorldSummary`, `SystemSummary`, `WirePayload`, `into_kernel_request`, `CorrelationToken` |
| `server/src/parity.rs` | 251 | `AC-13`'s `semantic_core` and `differing_fields` |
| `server/tests/two_clients.rs` | 640 | nine socket tests, including `a_message_that_asserts_state_is_refused_and_changes_nothing` (l. 433) and `killing_one_client_leaves_the_world_running_and_the_other_client_unaffected` (l. 384) |
| `server/tests/headless.rs` | 203 | the host with no transport: two observers, a non-reading subscriber, allocation, refusals |

Findings that bind the design:

- **A-1. A seat is not exclusive.** `WorldRuntime::join` (`runtime.rs` l. 255) checks only that the seat is
  in the roster and resolves it; it pushes a new `Subscriber` every time. Two connections may hold one seat
  at once, and both may submit as that observer. A human joining `alice` while `--agent alice` drives her
  is therefore not a takeover but **two controllers on one Person**. Nothing in the server knows which
  controller, if any, drives a seat.
- **A-2. No authentication.** `ClientFrame::Join` carries only `seat`. There is no token, no nickname and
  no notion of a player. `PROTOCOL.md` §9 records this as "not in revision 1".
- **A-3. No reconnect semantics.** A connection's subscription is released by `session::run` →
  `host.leave` on close, or reaped by `sweep` when its channel closes. A client that reconnects simply
  joins again and is a new subscriber. There is no seat hold, no resume, and no way to supersede a
  half-open connection — though with A-1 none is needed, which is itself the defect.
- **A-4. Whole observations only.** `sweep` computes a full `Observation<Value>` per subscriber and the
  session serializes it whole at 10 Hz (`HostConfig::observation_interval`, 100 ms). No delta exists.
- **A-5. No facts reach a client.** `Observation.events` is always empty: `systems/presence/src/observe.rs`
  (module doc, "Events") leaves it empty and names the transport (S11) and perception as the owners of the
  per-observer log position. `PerceptionContext::recent_events` exists and no implementation reads it.
  `clients/protocol/ADOPTION.md` §6 tells clients that "revision 1 leaves `events` empty". This is the
  brief's "protocol rev 1 does not expose fact types".
- **A-6. No admin surface.** `app::router` has three routes; `Command` has no admin variant.
  `WorldSummary.clients` counts subscribers and is the only operator-facing view of connections.
- **A-7. `deferrals_unscheduled` is dead.** Always `0` since S4 (`runtime.rs` `summary`); kept "until the
  next protocol revision removes it" (step-04 §11 L-2).
- **A-8. The composition is public; the vocabulary is not.** `WorldSummary.systems` names systems and their
  enabled flag. The kernel already exposes `SystemDeclaration::provides()` and `emits()`
  (`kernel/src/system.rs` l. 206, 211) through `Registry::declarations()` (`kernel/src/registry.rs` l. 110),
  so the vocabulary can be published with no kernel change.
- **A-9. The world thread is the only owner of the world.** `host.rs` module doc: a `World` is not `Send`,
  it is built inside its thread and every access is a message. Any new world-touching feature (seat
  bindings, hosted controllers, event learning) must live on that thread, not in a session task.
- **A-10. Size.** `runtime.rs` is 435 lines and `host.rs` 498, both near the 500-line review trigger
  (`ENGINEERING_STANDARDS.md` §10). Seat bindings, hosted controllers and admin cannot be added *into*
  these files without crossing it; the design gives each its own module (§8).

## 2.2 The composition root, `tools/cli/`

- `tools/cli/src/main.rs` `serve` (l. 286): reads the pack, checks every `--agent` seat against
  `pack.seats()`, spawns the world with `HostedWorld::new` or `HostedWorld::persisted` plus
  `PackPerception`, binds, prints the instance, then `tokio::spawn(agent::drive(host.clone(), seat))` per
  agent, and serves until Ctrl-C. No `--invite`, no nickname, no `--town`.
- `tools/cli/src/agent.rs` `drive` (l. 41): occupies a seat through `host.join` — the same call a session
  makes — and runs the **reactive** `RuleController` on every observation it is sent, submitting through
  `host.submit`. Runs as a tokio task, so it decides on whichever observation the 10 Hz `try_send` stream
  happened to deliver.
- `tools/cli/src/perceive.rs` `PackPerception`: the adapter from presence's `observe` onto the server's
  `Perception` seam. It lives in the composition root because the server must not depend on a pack.
- `tools/cli/src/run.rs`: the headless driver (`ARC-27`). It consults each seat at
  `genesis + k + m·PACE` (`PACE` = 900 s) with the **stateless** `PacedRuleController`, on one thread, and
  calls only what the server calls. This is the controller that makes a town walk; the server never runs it.

## 2.3 Controllers, `cognition/rule-controller/`

- `RuleController` (`lib.rs`): reactive. Holds `answered: BTreeMap<EntityId, Heard>` — the line it last
  answered per speaker. **`F-13` is here:** a controller rebuilt after a restart has an empty map and
  answers each speaker's newest line again. `ConversationHistory` (`systems/conversation/src/component.rs`
  l. 73) holds only lines *heard*, each `Heard { speaker, at, utterance }` with its instant; the
  controller's own replies are not in it, so "answered" cannot be read back from the history.
- `PacedRuleController` (`paced.rs`): stateless, `decide(&self, &Observation<Value>)`. Answers a line only
  if it was heard in `(t − pace, t]`, greets, approaches, wanders, uses doorways, follows the agenda,
  attempts complete affordances. Its rates are tuned to a 900-second pace (`ANSWERS` 75 %, greeting band 20 %,
  `DOOR_WINDOW` six hours). It answers **probabilistically**, so it cannot drive Alice in `AC-15`'s test,
  which needs every line answered.

## 2.4 Persistence, `persistence/`

- `WorldInput` (`persistence/src/input.rs`) has exactly three kinds: `Genesis`, `Dispatch`, `Advance`.
  Enabling or disabling a system at runtime is **not** a journalable input. An admin command that toggled a
  system would need a fourth kind — a persistence contract change outside this step (§12, QS11-10).
- Controller bindings, sessions, nicknames and tokens appear nowhere in the save, and must not: they are
  host state, not world state (`NETWORKING.md` §10). This is what makes `AC-5` testable as "control
  changed and the journal did not".

## 2.5 The Godot protocol module, `clients/protocol/`

- `mineworld/world_client.gd` (`MineWorldClient`, `PROTOCOL := 1`): `connect_to_world(address, seat)`,
  `submit`, `disconnect_from_world`; signals `welcomed`, `observed`, `resolved`, `refused`, `disconnected`,
  `submitted_request`. No reconnect (`ADOPTION.md` §6: "a dropped connection ends; retrying is the
  client's own policy"), no token, no delta.
- `clients/3d-spike/mineworld` is a **symlink** to `../protocol/mineworld`, and
  `clients/3d-spike/scripts/slice/slice_link.gd` l. 181 calls `connect_to_world(address, seat)`. Any change
  to the module's API reaches the 3D slice immediately; S11 must keep that call site working or update it.

## 2.6 What already holds, and what does not

| Claim | Holds today? | Evidence | What is missing |
| --- | --- | --- | --- |
| **AC-3** simulation continues while a client is gone | **Partly.** | `two_clients.rs` l. 384 and `ac15_one_alice.rs` l. 438: killing one client leaves the world and the other client running. | No test reconnects. Nothing distinguishes a reconnect from a stranger joining (A-3). "Continues correctly" is weak: with no hosted controllers a hosted town does nothing while nobody is connected except fire schedule processes, so there is little to continue. |
| **AC-3** a client may reconnect | **Mechanically yes, unproven.** | A new `join` on the same seat succeeds (A-1). | No test; no seat hold or resume; a half-open old connection keeps a second subscription alive. |
| **AC-5** human takes over an NPC, Person intact | **No.** | Any roster seat is joinable, including `alice` while `--agent alice` drives her. | Joining doubles control rather than taking it over (A-1). No test of biography, relationships, inventory or employment before and after. No release back to the controller. |
| **AC-7** several humans, same NPCs | **Partly.** | `ac15_one_alice.rs` `there_is_only_one_alice`: two windows and agent-driven Alice on one server, through the real binary; `milestone_c.rs`: Bob sees the shelf Alice bought from. | Never more than two humans; no authentication, so "human clients" are indistinguishable from anything that opens a socket; the hosted town is lifeless (no walkers), so "interact with the same NPCs" is only Alice. No test is named `AC-7`. |
| **INV-9** a state-asserting message is refused | **Yes.** | `two_clients.rs` l. 433 `a_message_that_asserts_state_is_refused_and_changes_nothing`; the closed `ClientFrame` enum and `CLIENT_FRAME_TAGS`; the actor check in `WorldRuntime::submit` (`actor_not_observer`). | Must be re-proven for every frame S11 adds and for the admin surface, which is a new way in. |
| `F-13` `--agent` re-answers after restart | **Open.** | `RuleController.answered` is controller memory (§2.3). | A remedy that persists nothing (§4.6). |
| The hosted town lives | **No.** | `main.rs` `serve` runs only `--agent` seats with the reactive controller (§2.2). | Server-side paced controllers (§4.5). |

---

# 3. Scope

## 3.1 In scope

```text
authentication     server invite token + player nickname, in the join frame (§4.1)
sessions           one controller per seat; a dropped connection holds its seat for a grace period
                   and may resume it with a secret the welcome gave it (§4.2, §4.3)
takeover           a human joining a seat an in-server controller drives takes it over; leaving or
                   being released returns it (§4.4) — AC-5's mechanism
hosted controllers in-server controllers bound to seats, consulted on the world thread: the paced
                   controller for the town (--town) and the reactive one for --agent (§4.5)
F-13               the reactive controller answers only what it hears while it is bound (§4.6)
facts              each observation carries the facts this observer learned since its previous frame,
                   judged by the perception system from each fact's Visibility (§4.7)
deltas             after a keyframe, frames carry what changed; periodic keyframes (§4.8)
admin              an HTTP surface behind an admin token: list sessions and seats, kick, release; it
                   cannot change world state (§4.9)
protocol rev 2     the versioned contract S12 and S14 build against (§5)
the Godot module   clients/protocol/mineworld updated to rev 2: handshake, reconnect with resume,
                   delta application — verified from the far side (R-9)
acceptance         AC-3, AC-5, AC-7 and INV-9 as committed tests through the real binary (§10)
```

## 3.2 Non-goals

```text
accounts, matchmaking, server browser, global identity     NETWORKING.md §9, overall §1 non-goals
TLS inside the server                                      a gateway/tunnel concern (NETWORKING §7); S13
enabling/disabling a system at runtime                     needs a fourth WorldInput kind (§2.4); QS11-10
a time-scale ("--rate") or pause for hosted worlds         QS11-4; the host stays 1 s : 1 s
binary encodings, protobuf                                 QS11-1
subscribe / unsubscribe observation scope                  a connection perceives its observer, nothing
                                                           else (INV-13); no use case in MVP-0
chat outside the world                                     speech is `talk`, a System Pack action;
                                                           out-of-world chat is not an MVP requirement
an LM controller, or any asynchronous controller           S10 / MVP-1; §4.5 leaves the seat for one
structured logging (`tracing`)                             S13's observability, with deployment
the 2D and 3D clients themselves                           S12, S14 — S11 changes only the shared module
```

---

# 4. Design

The organizing idea: **who controls a Person is host state, never world state.** A seat binding, a
session, a nickname, a token, a hold — none of them is journaled, none produces a fact, and none changes
the world's revision. That is `INV-1` made checkable: if control changes and the journal does not, the
Person cannot have been altered by the change, which is exactly what `AC-5` asks.

```text
                         ┌──────────────── world thread (owns the World) ────────────────┐
HTTP /admin ─► admin ───►│ SeatTable        seat → Free | Hosted(controller) | Connected(session)│
HTTP /status ──────────► │                         | Held(until, resume)                          │
WS /ws ─► session ─────► │ HostedControllers  consulted on their own lattice; submit like a client│
          (handshake,    │ Perception         observe(observer) + learns(observer, fact)          │
           delta encode) │ runtime            clock, allocator, journal, sweep                    │
                         └───────────────────────────────────────────────────────────────────────┘
```

## 4.1 Authentication: invite token and nickname

- **Where.** In the `join` frame, not in an HTTP header or a query string. Three reasons, each a rule:
  a credential in the protocol frame survives a future transport (`NETWORKING.md` §3, `INV-14`); a browser
  WebSocket cannot set headers, so a header-only scheme would exclude a web client later; a query string is
  written to proxy and access logs.
- **Invite token.** One per server process. `mineworld server --invite TOKEN`, or the environment variable
  `MINEWORLD_INVITE` (for S13's container), or — when neither is given — 128 bits from the operating
  system's CSPRNG, printed once at startup as a ready-to-paste join line. Compared in constant time.
  Never written to a save, a World Pack, an observation, a fact or a log line other than that one startup
  line (I-5). QS11-6 asks whether loopback hosting may skip it; the recommendation is no.
- **A wrong token.** Answered `refused { code: "unauthorized" }` after a fixed 500 ms delay, then a
  `closing { reason: "unauthorized" }` and the socket is closed. One guess per connection, at most two per
  second per connection attempt; QS11-13 records why no rate-limit dependency is adopted yet.
- **Nickname.** A `Nickname` newtype: 1–32 Unicode scalar values after trimming, no control characters.
  It labels the *player*, not the Person — the Person's name is `naming`'s `display-name`, world data. The
  nickname is shown to the player's own connection (`welcome`) and to the admin surface, and to nobody else:
  it is not in any observation (QS11-7). Duplicates are allowed; the admin surface tells sessions apart by
  `SessionId`.
- **Admin token.** Separate from the invite (`--admin-token`, `MINEWORLD_ADMIN_TOKEN`). Without one the
  admin routes do not exist (§4.9).

## 4.2 Sessions and seats: one controller per seat

A new module `server/src/seats.rs` owns a `SeatTable` on the world thread. Each roster seat is in exactly
one state:

```text
Free                         nobody controls the Person; it stands
Hosted(HostedControllerId)   an in-server controller drives it (§4.5)
Connected(SessionId)         a connection drives it
Held { until, resume }       its connection dropped; nobody drives it until `until` (wall clock) or a
                             resume arrives
```

Transitions (the only ones):

```text
Free ─join─► Connected                 Hosted ─join─► Connected   (takeover, §4.4)
Connected ─socket drops─► Held         Connected ─leave frame / admin kick─► default(seat)
Held ─join with valid resume─► Connected (new session; the old one, if still open, is closed `superseded`)
Held ─until passes─► default(seat)     Held ─admin release─► default(seat)
Connected ─join on another connection without resume─► refused `seat_occupied`; state unchanged
default(seat) = Hosted(c) if a hosted controller is configured for the seat, otherwise Free
```

Every transition is a host event: no journal entry, no fact, no revision (I-2). `WorldRuntime::join` stops
pushing a subscriber for an occupied seat — that is the fix for A-1.

**The observer is still the server's.** A join names a seat; the server resolves the observer, exactly as
in revision 1. The actor check (`actor_not_observer`) is unchanged and remains the authority rule for
every submit, from a session or a hosted controller.

## 4.3 Reconnect: hold and resume

- `welcome` carries `resume`, a 128-bit secret for this seat binding, and `hold_seconds`.
- When the socket drops, the seat is `Held` for `hold_seconds` (default 30, `--hold SECONDS`; QS11-5).
  Nobody drives the Person during the hold — not even its hosted controller — so a player whose Wi-Fi
  blinks does not find their Person walked off by the town's rule.
- A `join` for that seat presenting the valid `resume` within the hold re-takes it: a new `SessionId`, a new
  `resume`, a fresh keyframe. If the old connection is somehow still open (half-open TCP), it is sent
  `closing { reason: "superseded" }` and closed — so a reconnect never needs the server to have noticed the
  drop first.
- After a server restart every session and secret is gone (host state is not persisted); a client rejoins
  with invite, nickname and seat, and the seat is free or hosted.
- The Godot module (`world_client.gd`) gains an opt-in reconnect policy: on `disconnected`, retry with the
  stored `resume` at 1 s, 2 s, 4 s, … up to `hold_seconds`, then fall back to a plain join. It is a policy
  in the module, not a rule — the server decides every outcome.

## 4.4 Takeover and release (`AC-5`)

A human joining a `Hosted` seat takes it over: the hosted controller is unbound (it is not consulted while
unbound), the seat becomes `Connected`, and the human's next observation is of the same `EntityId` with the
same components. When the human `leave`s, is kicked, or their hold expires, the seat returns to its hosted
controller, which is **rebuilt from its configuration**, not resumed from memory (§4.6).

The claim `AC-5` makes, and how it is measured (§9.2 CP-B1, decided before measuring): across a takeover and a
release, in `worlds/market-town`, the Person's `EntityId` is unchanged; the save's journal gains no entry
and its revision does not move at either binding change; the Person's biography (`mineworld biography`),
relationships (`knows` edges and values), holdings, wallet and job, read from the save before the takeover
and after the release, differ only by facts caused by actions the Person took in between — every one of
which names an `ActionId` the human's session or the controller submitted.

## 4.5 Hosted controllers: the town lives in `mineworld server`

**The seam.** `server/src/hosted.rs` declares

```rust
/// A controller the server runs on the world thread. Synchronous and bounded: `decide` must return in
/// microseconds and may not block, wait or call out — an asynchronous controller (an LM) connects as a
/// session instead (ARCHITECTURE.md §9).
pub trait HostedController: 'static {
    /// The first instant strictly after `after` at which this controller wants to be consulted. A
    /// paced adapter answers from its lattice (it is built knowing its seat's index); a reactive one
    /// answers `after + 1 s`, i.e. every tick that moves the clock.
    fn next_consult(&self, after: WorldTime) -> WorldTime;
    /// What the Person attempts, from its own observation only (INV-13). Never a world handle.
    fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest>;
}
```

The server names no controller crate; the composition root (`tools/cli`) adapts the two rule controllers
onto it, as it already adapts perception (`perceive.rs`). On every tick the runtime consults each `Hosted`
seat whose next consult is due: it computes that seat's observation through the same `Perception` call a
session gets, asks `decide`, and passes any request through **the same `submit` path a session uses** —
actor check, server-allocated `ActionId` and instant, journal first, sweep (I-8). A hosted controller is
therefore not a privileged path; it skips the socket, as `agent.rs` does today, and nothing else.

**Why on the world thread rather than as tasks** (ARC-S11-c, QS11-2). A task decides on whichever
observation the 10 Hz `try_send` stream delivered; the paced controller's "answer each line once" needs to
be consulted once per pace window at instants it can name, which only the thread that owns the clock can
guarantee. The cost is that `decide` runs on the world thread; the trait's contract is that it is a bounded
synchronous rule, and S11-B measures it (§9, CP-B4).

**The two adapters** (in `tools/cli/src/hosted.rs`, replacing `agent.rs`):

```text
--agent SEAT       ReactiveSeat: RuleController::since(bind instant), consulted once per world second
--town             PacedSeat: PacedRuleController::new(seed, pace), seat k consulted at
                   genesis + k + m·pace — ARC-27's lattice, the same formula `run` uses
--seed N           the paced controllers' seed (default 0)
--pace SECONDS     the hosted pace (default 5; QS11-4)
```

`--town` binds a `PacedSeat` to every roster seat not named by `--agent`. Player seats such as `visitor` and
`wanderer` included: while nobody plays them, they live like everybody else (QS11-3). A hosted world is
not reproducible from a seed — its instants follow the wall clock — which `ARC-27` already excludes from
`AC-12`; the save it writes still replays byte for byte, because the journal holds the intents.

## 4.6 `F-13`: the reactive controller answers only what it hears while bound

`RuleController::since(at: WorldTime)` — a controller bound at `at` treats every line heard at or before
`at` as not its to answer. A controller is rebuilt at every binding (server start, resume of a saved world,
return after a takeover), so a restarted server's Alice does not re-answer the line she answered before the
crash, and an Alice handed back after a human played her does not answer what was said to the human. No
controller state is persisted (`INV-1`, `ARC-27` option (c) rejected). The cost — a line said in the same
second the controller is bound is not answered — is the "missed the moment" limitation `ARC-27` already
accepts. `RuleController::new()` stays and answers every line, as today, so every existing test keeps
its meaning; only the hosted adapter uses `since`.

This is a change to `cognition/rule-controller/src/lib.rs` only. `PacedRuleController` and `mineworld run`
are untouched, so both 300-day digests are untouched (I-6).

## 4.7 Facts in observations (`NETWORKING.md` §5 "event notification")

- **Whose judgement.** Which facts an observer learns is perception's judgement, never the transport's
  (`ARCHITECTURE.md` §6, `INV-13`). The server's `Perception` seam gains one method with a safe default:

  ```rust
  /// Whether `observer` learns of this fact, judged against the world as it stands right after the fact
  /// was recorded. The default learns nothing — the safe direction, as `PerceivesNothing` is.
  fn learns(&self, context: &PerceptionContext<'_>, fact: &EventEnvelope) -> bool { false }
  ```

  `systems/presence` implements it from the contract's `Visibility`: `Public` → yes; `Place(p)` → the
  observer's presence is in `p`; `Participants` → the observer is a listed participant; `Entities(s)` → the
  observer is in `s`; `SystemInternal` → never. `PackPerception` forwards it.
- **When.** At record time: right after a dispatch or an advance returns its facts, the runtime asks
  `learns` for each fact and each subscriber (sessions and hosted seats), and appends the learned facts to
  that subscriber's pending queue. Evaluating later — at the next sweep — would judge a `Place` fact against
  where the observer has walked to since.
- **Delivery.** A frame's `observation.events` is exactly the facts the observer learned since the
  previous frame on this connection, oldest first. Pending facts are cleared only when a frame carrying them
  is queued (`try_send` succeeded), so a dropped frame does not drop facts; the queue is bounded (256), and
  overflow is counted in `WorldSummary.events_dropped` rather than hidden.
- **Payload.** A perceived fact is the contract's `PerceivedEvent<Value>` — the envelope with its
  `event_type`, `caused_by`, `visibility`, participants and payload in the owning pack's own JSON. Clients
  read `event_type` names from `/status`'s vocabulary (§5.5) and treat an unknown type as "something
  happened" rather than an error.
- **The run path is untouched.** `mineworld run` calls `observe`, which does not change, and never calls
  `learns`; the paced controller reads histories, not events. Both digests are unchanged (I-6).

## 4.8 Deltas

- **Where computed.** In the session task, against the last observation *this connection sent* — not on
  the world thread. The world keeps computing one whole observation per subscriber (that is perception's
  call and stays `INV-13`'s structure); the session decides how to put it on the wire. A dropped frame is
  dropped before the session sees it (`runtime::sweep`), so a delta is always against a frame the client
  actually received, over an ordered, reliable WebSocket.
- **Shape.** A typed delta keyed by contract identity (§5.4): `at`; `self_location` if changed; entities
  upserted (whole `PerceivedEntity`, by `EntityId`) and removed (by id); `relations` and `affordances` as
  whole lists when either changed (affordance order is meaningful — `PROTOCOL.md` §5, complete affordances
  are told apart by position); `events` always (they are already a since-last-frame stream, §4.7).
- **Canonical order.** Rev 2 states that `observation.entities` is in ascending `EntityId` order; the
  session sorts before encoding. Without that, "reconstructed equals whole" would depend on presence's
  iteration order.
- **Keyframes.** The first frame of a connection is a whole `observation`; so is every 50th frame
  (`--keyframe-every`, default 50 = 5 s at 10 Hz) and the first frame after a resume. A client must accept
  a whole observation at any time.
- **Measured first** (`REUSE_POLICY.md` §15, `ARC-23`). S11-C begins by recording real frames from a
  hosted `market-town` with `--town` and four sessions and compares, on the same frames, whole JSON, an
  RFC 6902 JSON Patch diff (the `json-patch` crate, as a dev-dependency of the measurement only), and the
  typed delta (§7.3). The typed delta is kept only if it beats both on bytes per client per second; the
  numbers decide DEP-S11-b either way.

## 4.9 The admin surface

HTTP, under `/admin`, JSON, `Authorization: Bearer <admin token>`, compared in constant time. Routes exist
only when an admin token is configured.

```text
GET  /admin/sessions                     every connection: session id, seat, nickname, observer,
                                         connected-at (wall), frames sent, frames dropped, last seq
GET  /admin/seats                        every seat: free | hosted(kind) | connected(session) | held(until)
POST /admin/sessions/{session}/kick      closing{kicked}; the seat returns to default (no hold)
POST /admin/seats/{seat}/release         whoever holds it is closed `kicked`; the seat returns to default
```

**No admin command touches world state** (I-4). There is no route that sets a component, moves a Person,
emits a fact, enables a system or advances the clock; S11-D's adversarial test calls every route and
asserts the save's revision and fact count unchanged. Status, which is world-composition information, stays
public at `GET /status`. A WebSocket admin frame was considered and rejected: `NETWORKING.md` §3 puts admin
on HTTP, and a frame would widen the client vocabulary that `INV-9` is defined as the absence of.

---

# 5. The wire contract: protocol revision 2

**Status: PROPOSED, part of this step's freeze.** This section is what S12 and S14 design against. On
freeze it becomes `server/PROTOCOL.md` revision 2 (written by S11-A, completed by S11-C); until then
`PROTOCOL.md` revision 1 is what `main` speaks. Everything revision 1 says and this section does not
change stays in force: identities are decimal strings, every other number is an integer, `seq` orders
frames, `action_type` appears twice, `actor_location` is a report, `result` and the `ActionResult` shapes,
the reporting rule of §6.2, and `AC-13`'s semantic core.

## 5.1 Versioning rule

- `PROTOCOL_VERSION` becomes `2` in S11-A, the first PR that changes the wire. The client states the
  revision it speaks in `join`; a server answers a different number with `refused { code:
  "protocol_mismatch" }` and closes. A client that receives a `welcome.protocol` it does not know refuses
  to continue (unchanged from revision 1).
- **Implemented incrementally, specified whole.** Revision 2 is specified completely here, and lands over
  S11-A … S11-D. A conforming revision-2 client must handle every frame in §5.3 from the start. The server
  is allowed to send a subset while the step is in flight: before S11-C merges it sends only whole
  `observation` frames, `events` is empty, and there is no `delta`; before S11-B merges there is no hold
  and `resume` is absent. Each such absence is a frame the server *may* omit, never a different meaning
  for a frame it sends, so a client written against this section is correct against every intermediate
  `main`.
- Any change to this section after freeze is a protocol change: it returns to the primary session, and to
  the S12 and S14 owners, as a revision of this step.

## 5.2 What a client may say (closed set of three)

```json
{ "t": "join", "protocol": 2, "invite": "3f9c…", "nickname": "Yue", "seat": "visitor",
  "resume": null }
{ "t": "submit", "token": "c1", "request": { … an ActionRequest, unchanged from revision 1 … } }
{ "t": "leave" }
```

| Frame | Fields | Rules |
| --- | --- | --- |
| `join` | `protocol` (integer, required), `invite` (string, required), `nickname` (string, 1–32 scalar values after trimming, no control characters, required), `seat` (entity key, required), `resume` (string or `null`, optional, default `null`) | Valid only before a seat is granted. A second `join` on a seated connection is `already_joined`. |
| `submit` | `token`, `request` | Unchanged. Before a seat is granted: `not_joined`. |
| `leave` | — | Releases the seat at once (no hold); the server answers `closing { reason: "left" }` and closes. |

There is still no frame that sets a value, names an observer, widens a scope or states a fact. Any other
`t` is `unknown_frame`, which is where a state-asserting message lands (`INV-9`). Field order is free;
unknown fields inside a known frame are `malformed_frame` (the contract denies unknown fields, so a typo is
loud).

## 5.3 What the server says

| Frame | When | Shape |
| --- | --- | --- |
| `welcome` | once, after a granted `join` | `{ "t": "welcome", "protocol": 2, "seat": "visitor", "observer": "101", "nickname": "Yue", "session": "7", "resume": "a1b2…" , "hold_seconds": 30, "took_over": "hosted", "world": WorldSummary }` |
| `observation` | the first frame; every `keyframe_every`-th frame; the first frame after a resume; and any frame the server chooses | `{ "t": "observation", "seq": 1, "revision": 7, "observation": Observation }` |
| `delta` | any other frame, from S11-C | `{ "t": "delta", "seq": 2, "base": 1, "revision": 7, "delta": ObservationDelta }` |
| `result` | one per `submit` | unchanged |
| `refused` | a frame was not accepted | unchanged shape; codes in §5.6 |
| `closing` | immediately before the server closes the socket | `{ "t": "closing", "reason": "kicked", "detail": "…" }` |

Field notes:

- `welcome.session` is a `SessionId` as a decimal string (an identity, so a string). It names this
  connection on the admin surface and nowhere else.
- `welcome.resume` is a 128-bit secret as 32 lowercase hexadecimal characters; `null` before S11-B.
  `hold_seconds` is an integer; `0` before S11-B.
- `welcome.took_over` is `"none"` when the seat was free, `"hosted"` when an in-server controller was
  driving it, `"held"` when this join resumed a held seat. It says that control changed hands, never which
  controller kind or which player: a client may tell its player "you are now playing Alice, who was living
  on her own", and learns nothing about other players.
- `observation.observation.entities` is in ascending `EntityId` order (new in revision 2).
- `observation.observation.events` is the facts this observer learned since the previous frame on this
  connection, oldest first; empty before S11-C. A fact is a `PerceivedEvent` — the event envelope the
  contract already serializes, with `event_type`, `caused_by`, `visibility`, participants and the owning
  pack's JSON payload.
- `closing.reason` is one of `left`, `kicked`, `superseded`, `unauthorized`, `protocol_mismatch`,
  `world_stopped`, `server_stopping`. `detail` is for a developer; a client branches on `reason`.

## 5.4 `ObservationDelta`

```json
{ "at": 4112,
  "self_location": { … a Location … },
  "entities": { "upsert": [ PerceivedEntity, … ], "remove": [ "9007199254740995" ] },
  "relations": [ Relation, … ],
  "affordances": [ Affordance, … ],
  "events": [ PerceivedEvent, … ] }
```

Applying a delta whose `base` is the `seq` of the observation the client holds yields the observation of
`seq`, defined field by field:

| Field | Present | Meaning |
| --- | --- | --- |
| `at` | always | replaces |
| `self_location` | only if changed | replaces; `null` means the observer now has no location |
| `entities.upsert` | only if non-empty | each replaces the entity with the same id, or is added; the result is re-sorted by id |
| `entities.remove` | only if non-empty | ids no longer perceived; removing an id not held is a client-side protocol error |
| `relations` | only if changed | replaces the whole list |
| `affordances` | only if changed | replaces the whole list, order preserved |
| `events` | always | replaces (events are a since-last-frame stream, never accumulated) |

`observer` never changes on a connection and is not in a delta. A delta whose `base` is not the `seq` the
client holds cannot occur on one WebSocket (ordered, reliable, dropped frames are dropped before they are
numbered); a client that sees one must drop the connection and resume, which yields a keyframe.

## 5.5 HTTP

```text
GET /health            unchanged, but `protocol: 2`
GET /status            WorldSummary, revision 2 (below); public
GET /ws                the live connection
/admin/...             §4.9; exists only with an admin token; Authorization: Bearer
```

`WorldSummary`, revision 2:

```json
{ "protocol": 2, "instance": "1a2b…", "at": 4112, "entities": 41,
  "systems": [ { "system": "conversation", "enabled": true,
                 "provides": [ "talk" ], "states": [ "spoke", "conversation-started" ] } ],
  "seats": [ "visitor", "wanderer", "alice" ], "clients": 2,
  "observations_dropped": 0, "events_dropped": 0, "faults": 0, "revision": 7 }
```

- Removed: `deferrals_unscheduled` (always `0` since S4; step-04 L-2).
- Added: per system `provides` (action types) and `states` (event types it emits, including another
  system's vocabulary it is declared to state), read from the kernel's `SystemDeclaration` — composition,
  not state, so it stays public. A client uses it to know whether to offer a verb at all and to name an
  `event_type` it receives.
- Added: `events_dropped`, facts lost to a full pending queue (§4.7).
- `clients` counts seated connections only (hosted controllers are not clients).

## 5.6 Refusal codes, revision 2

Revision 1's nine codes stand. Added:

| `code` | Meaning |
| --- | --- |
| `protocol_mismatch` | `join.protocol` is not a revision this server speaks. Followed by `closing`. |
| `unauthorized` | Wrong or missing invite token, after a fixed delay. Followed by `closing`. |
| `invalid_nickname` | Empty after trimming, longer than 32, or containing a control character. |
| `seat_occupied` | Another connection holds the seat, or it is held and no valid `resume` was given. |
| `invalid_resume` | A `resume` that does not match the seat's hold (expired, or for another seat). The client may retry without it. |

`refused` keeps meaning "this frame was not a request"; a `Rejection` inside a `result` keeps meaning "the
world considered the request and said no". Being closed is `closing`, never a refusal.

## 5.7 What revision 2 deliberately does not have

```text
a subscribe / scope frame      a connection perceives its observer (INV-13); nothing to widen
admin frames on the socket     admin is HTTP (§4.9)
binary or compressed frames    QS11-1; Godot's WebSocketPeer does not support permessage-deflate (§7.3)
an observer-naming field       unchanged from revision 1: there is none
other players' nicknames       not world data (QS11-7)
```

---

# 6. Invariants of this step

Each is checked by a test or a diff gate in the PR that could break it (§9).

| ID | Invariant | How it is held |
| --- | --- | --- |
| **I-1** | No kernel, contract or persistence change: `kernel/`, `contracts/`, `persistence/` have no diff in any S11 PR. | Diff gate per PR. Events use the existing `Observation.events` and `Visibility`; the vocabulary uses existing `SystemDeclaration` accessors. A needed edit there is a material stop. |
| **I-2** | Control is host state. A binding change (join, takeover, leave, hold, resume, release, kick) adds no journal entry and no fact and does not move the revision. | `AC-5` test reads the save's revision and fact count across both binding changes. |
| **I-3** | At most one controller drives a seat at any instant. | `SeatTable` is the only writer of bindings; a test races two joins and a resume on one seat and asserts exactly one `welcome`. |
| **I-4** | The client vocabulary is closed and asserts nothing; the admin surface changes no world state. | `INV-9` table test over rev 2's frames; S11-D calls every admin route against a saved world and asserts revision and facts unchanged. |
| **I-5** | No secret is persisted or disclosed: the invite, admin token and resume secrets appear in no save, World Pack, observation, fact, `/status` or log line, except the one startup line that prints a generated invite. | A test greps the save file's bytes and the captured stdout/stderr of a hosted run for each secret. |
| **I-6** | `mineworld run` is untouched: the 300-day seed-7 digests of `social-cafe` and `market-town` are identical before and after every S11 PR (against that PR's base, since S15's 12d re-baselines them). | Digest comparison per PR. |
| **I-7** | Facts reach an observer only by perception's `learns`; `SystemInternal` facts reach nobody. | A test with a pack fixture stating each `Visibility` and two observers in different places. |
| **I-8** | A hosted controller takes the session's authority path: the same actor check, server-allocated `ActionId` and instant, journal-before-answer. | Shared `submit` function; a test drives a hosted seat and a session seat and finds both requests journaled identically. |
| **I-9** | The server names no System Pack and no controller crate. | `ac1_composability` check 2 and the I-2 vocabulary scan (both already bite, step-10 11f); `server/Cargo.toml` gains no `mineworld-*` pack or cognition dependency. |
| **I-10** | A delta-reconstructed observation equals the whole observation the server computed for that frame — in Rust and in Godot. | Keyframe comparison in both appliers (§9, CP-C3, CP-E2). |
| **I-11** | The world never waits for a client or a controller: no `await` on the world thread, `try_send` for every delivery, `decide` bounded. | Review, plus CP-B4's measured tick duration. |
| **I-12** | `AC-13` and `AC-15` stay green: `ac13_semantic_parity.rs` and `ac15_one_alice.rs` pass on every S11 PR head, updated only for the rev 2 handshake. | Per-PR gate. |

---

# 7. Reuse: every infrastructure piece, both directions (`REUSE_POLICY.md` §§1–4, 11, 12, 15, 17)

Each table lists real candidates and "build our own", compared on fit with MineWorld's model (single
ownership, determinism, server authority, the pack model), licence, maturity and maintenance, and cost.
**Evidence status** is stated per row: *verified* means read in this session (local Cargo registry,
`Cargo.lock`, or a primary source linked in §7.9); *to verify* means the PR that adopts or rejects it must
confirm it before freeze, and the verdict is conditional on that.

The transport itself — `axum` on `tokio` — is not reopened: `DEP-3` selected it after a spike, and nothing
in S11 strains it. Every new route and frame lives inside it.

## 7.1 Wire encoding of revision 2

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **JSON text over WebSocket (keep, `DEP-3`)** | Contract types serialize through their own `serde`; ids already decimal strings (PR 04); Godot parses it natively; human-readable evidence logs. | n/a (serde_json MIT/Apache, in use) | in use, proven by `AC-13`/`AC-15` | none | **Recommended.** |
| Protocol Buffers: `prost` (Rust) + `godobuf` (GDScript) | `D-4` named this route for the first cross-language boundary. A parallel `.proto` schema mirrors every contract type, including opaque pack payloads that would stay JSON-in-bytes anyway. | prost Apache-2.0; godobuf BSD-3-Clause *(verified, §7.9)* | prost mature; godobuf a single-maintainer GDScript generator *(to verify: maintenance)* | a second schema for every contract type, a code generator in both builds, and the far-side proof redone | Rejected for MVP-0: the schema duplication is the drift `R-3` warns about, and bandwidth is answered by §7.3, not by encoding. Operator-material because it departs from `D-4`'s timing (QS11-1). |
| MessagePack: `rmp-serde` + a GDScript msgpack addon | Works through `serde` with no schema; binary frames. | MIT *(to verify)* | rmp-serde mature; Godot side an addon *(to verify)* | a binary path in the Godot module; unreadable evidence logs | Rejected now; the natural first step if a measured need for binary arises, since it keeps `serde` as the single source of truth. |
| CBOR: `ciborium` | As MessagePack. | Apache-2.0 *(to verify)* | mature in Rust | no maintained GDScript decoder found | Rejected: no far-side implementation. |
| Build our own binary encoding | Total control. | — | — | the most code, both sides | Rejected: commodity infrastructure (`REUSE_POLICY.md` §4). |

## 7.2 Authentication: credential carriage, comparison and secret generation

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Invite and nickname in the `join` frame, checked by a small `Admission` type in the server (own, ~60 lines)** | Transport-independent (`INV-14`), works for a browser client later, never in a URL; the check sits where the frame is decoded. | — | — | small | **Recommended** for players. |
| `tower-http` `ValidateRequestHeaderLayer` (bearer) on the `/ws` upgrade | A header on the HTTP upgrade. Browsers cannot set it on a WebSocket; it authenticates the transport rather than the protocol. | MIT | mature (tower-rs) — but its bearer helper's deprecation status *(to verify)*; `tower-http` is not yet in `Cargo.lock` *(verified)* | one new crate | Rejected for players; considered again for `/admin` below. |
| `axum-extra` `TypedHeader<Authorization<Bearer>>` | A typed header extractor; fits HTTP admin routes. | MIT | mature (tokio-rs) | one new crate (`axum-extra`, `headers`) | **Considered for `/admin`**; the PR compares it with a ten-line extractor over `axum`'s own `HeaderMap` and adopts it if it removes code rather than adds a dependency for one header. |
| JWT (`jsonwebtoken`) / PASETO session tokens | Stateless signed tokens; designed for accounts and expiry across services. | MIT | mature | key management, expiry policy, clock skew | Rejected: MVP has one shared invite and no accounts (`NETWORKING.md` §9); a signed token answers a question nobody asked yet. Revisit with durable player identity. |
| `axum-login` / `tower-sessions` | User/session frameworks around cookies and a user store. | MIT | maintained | adopts a session store and a user model | Rejected: architecture mismatch — cookie sessions over HTTP, and a user model MineWorld does not have (`REUSE_POLICY.md` §3, framework lock-in). |

| Constant-time comparison | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **`subtle` (`ConstantTimeEq`)** | Exactly the operation needed; no allocation. | BSD-3-Clause *(to verify)* | the dalek-cryptography crate, very widely depended on | one small crate | **Recommended** (DEP-S11-a). |
| `constant_time_eq` | Same operation, smaller API. | CC0 / MIT-0 / Apache-2.0 *(to verify)* | widely used | one tiny crate | Acceptable alternative; `subtle` preferred for its review history. |
| Hash both sides (`sha2`) and compare digests | Equality of digests leaks nothing useful about the secret. | MIT/Apache | mature; `sha2` is cached locally but not in `Cargo.lock` *(verified)* | one crate, and an indirect argument a reviewer must re-derive | Rejected: more cleverness than the problem. |
| Own loop with `fold`/`|` | Possible, and easy for an optimizer to undo. | — | — | — | Rejected: the classic way to get it wrong. |

| Secret generation (invite, resume) | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **`getrandom` 0.3 (OS CSPRNG)** | 16 bytes from the OS; nothing else. | MIT / Apache-2.0 | rust-random; **already in `Cargo.lock` at 0.3.4 through `tungstenite` → `rand`** *(verified)* | a direct edge to a crate already compiled | **Recommended** (DEP-S11-a). |
| `rand` 0.9 (`rand::rng()`) | A full RNG API for 16 bytes. | MIT / Apache-2.0 | already in the lock at 0.9.5 *(verified)* | larger surface than needed | Acceptable; not preferred. |
| `uuid` v4 | A UUID is an identifier, not a secret format; 122 random bits. | MIT / Apache-2.0 | mature; not in the lock | one crate | Rejected: wrong concept. |
| `WorldInstanceId::allocate` style (clock ⊕ pid ⊕ ordinal) | Guessable. | — | — | — | Rejected: an identity, not a secret (`protocol.rs` says so itself). |

## 7.3 Observation deltas

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Typed delta keyed by contract identity (own, §5.4)** | Diffs the four list-shaped fields by `EntityId` / whole-list replacement; the reconstruction rule is one table; the GDScript applier is a dictionary merge. | — | — | ~150 lines Rust, ~80 GDScript, plus tests | **Recommended, conditional on S11-C's measurement.** |
| RFC 6902 JSON Patch, `json-patch` crate (`diff` + `patch`) | Generic. On `entities`, a list ordered by id, one entity appearing shifts every later index, so patches are index operations that are long and order-fragile; the client needs an RFC 6902 applier in GDScript (none adopted). | MIT OR Apache-2.0, v4.2.0 *(verified, §7.9)* | maintained (idubrov) | one crate server-side; an applier client-side | **Measured, not adopted by default**: S11-C records patch sizes on real frames as a dev-dependency of the measurement only. Adopted if it is within 10 % of the typed delta, because then a standard beats our own. |
| RFC 7396 JSON Merge Patch (same crate) | Cannot address an array element: any changed entity replaces the whole `entities` array — no saving on the field that dominates. | as above | as above | as above | Rejected on the shape of the data; the measurement shows it. |
| WebSocket `permessage-deflate` (transport compression) | Zero protocol change and the largest likely saving on repetitive JSON. | — | **`tungstenite` 0.29 declares no deflate feature** *(verified, local registry manifest)*; **Godot's `WebSocketPeer` drops compressed frames and cannot negotiate the extension** (godot#103230, godot-proposals#13179, *verified §7.9*) | would need a different server WebSocket stack and an engine change | Rejected now: the far side cannot speak it. Recorded as the preferred route the day Godot supports it, since it would let deltas be deleted. |
| Snapshot-delta replication from game networking (Quake/Valve snapshot deltas; Colyseus `@colyseus/schema`) | Designed for authoritative state replication; Colyseus's schema is a binary per-field delta with its own type system. | Colyseus MIT | mature (TypeScript) | adopting a schema system and a TypeScript-native encoder | **Reference only**: confirms the shape (keyframe + per-entity deltas against the last acknowledged state); adopting it would put a second type system beside the contracts. |

## 7.4 Sessions, seat holding and reconnect

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Own `SeatTable` state machine on the world thread (§4.2)** | Seats are MineWorld's concept (a Person, `INV-1`); the binding must be on the thread that owns the world (A-9). | — | — | one module, ~250 lines with tests | **Recommended.** |
| Colyseus reconnection (`allowReconnection` + `reconnectionToken`) | The same pattern — hold the seat on drop for a window, rejoin with a token refreshed on every connection. | MIT | mature, widely used | TypeScript; a room model | **Reference only (ADAPT the pattern)**: §4.3's hold, refreshed `resume` and supersede-the-stale-connection follow it, including its known pitfall that a token must be validated before the grace timer is torn down (colyseus#962, §7.9). |
| Nakama sessions | Session tokens, presence, matchmaking, a Go server with its own runtime. | Apache-2.0 | mature | adopting a server framework | Rejected: framework lock-in; MineWorld's server is the world runtime (`REUSE_POLICY.md` §§3, 8). |
| `tower-sessions` | HTTP cookie sessions with a store. | MIT | maintained | a store | Rejected: a seat binding is not a cookie session. |

## 7.5 Hosted controller scheduling

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Own consult lattice on the world thread (§4.5), the formula `ARC-27` already uses** | Consults at exact world instants; no channel drops; no new concept. | — | proven by `run` since S7 | ~120 lines | **Recommended.** |
| One tokio task per controller (today's `agent.rs`) | Decides on whichever observation arrived; cannot name its instants. | — | in use | none | Kept only as the pattern for future asynchronous controllers (sessions); rejected for rule controllers. |
| The kernel's scheduler (`DEP-6`) via a `Process` | A controller would become world state — rejected by `INV-1` and `ARC-27` option (c). | — | — | — | Rejected. |
| Job schedulers (`tokio-cron-scheduler`, `apalis`) | Wall-clock cron and job queues. | MIT | maintained | a runtime beside the world's | Rejected: wrong clock (wall, not world) and wrong concept. |

## 7.6 Admin surface

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **`axum` routes under `/admin`, JSON, bearer token** | `NETWORKING.md` §3 puts admin on HTTP; same stack (`DEP-3`); curl-able. | (in use) | (in use) | one module | **Recommended.** |
| WebSocket admin frames | Widens the client vocabulary `INV-9` is the absence of. | — | — | — | Rejected. |
| gRPC (`tonic`) admin service | Typed RPC, a second protocol stack and codegen. | MIT | mature | large | Rejected: dependency larger than the problem. |
| An off-the-shelf admin panel | Nothing found models seats and controllers; any would be a UI over the same four routes. | — | — | — | Rejected for MVP-0; a GUI is Phase 2 (`MVP.md` §7.4). |

## 7.7 Abuse limits on joining

| Option | Fit | Licence | Maturity | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **One guess per connection + fixed 500 ms delay (own, a few lines)** | Enough for a 128-bit invite on a LAN or a tunnel. | — | — | trivial | **Recommended for MVP-0.** |
| `governor` / `tower_governor` (GCRA rate limiting per IP) | Correct tool for a public server. | MIT / Apache-2.0 *(to verify)* | maintained | one or two crates; IP keying behind proxies needs care | Deferred: the adopt route when public hosting is in scope; recorded so it is not rebuilt (QS11-13). |

## 7.8 Event visibility

| Option | Fit | Verdict |
| --- | --- | --- |
| **Perception's `learns`, implemented in `presence` from the contract's `Visibility` (§4.7)** | The judgement is perception's (`ARCHITECTURE.md` §6); the inputs are MineWorld's own types. | **Recommended.** Nothing external models MineWorld's `Visibility`. |
| A generic pub/sub broker (NATS, Redis pub/sub, `tokio::sync::broadcast`) | Moves facts, but cannot decide who may learn them; a broadcast channel would be a world frame filtered per client — the shape `INV-13` forbids. | Rejected on architecture; in-process delivery already exists (the per-subscriber channel). |

## 7.9 Sources consulted in this session

- `json-patch`: [crates.io](https://crates.io/crates/json-patch), [idubrov/json-patch](https://github.com/idubrov/json-patch) — RFC 6902 + RFC 7396, MIT OR Apache-2.0, 4.2.0.
- Godot WebSocket compression: [godotengine/godot#103230](https://github.com/godotengine/godot/issues/103230), [godot-proposals#13179](https://github.com/godotengine/godot-proposals/issues/13179) — no `permessage-deflate`; compressed frames are dropped.
- godobuf: [oniksan/godobuf](https://github.com/oniksan/godobuf) — BSD-3-Clause, GDScript protobuf generator.
- Colyseus reconnection: [docs.colyseus.io/room/reconnection](https://docs.colyseus.io/room/reconnection), [colyseus#962](https://github.com/colyseus/colyseus/issues/962).
- Local: `~/.cargo/registry/.../tungstenite-0.29.0/Cargo.toml` (no `deflate` feature); `Cargo.lock` (`getrandom` 0.3.4, `rand` 0.9.5 present; `tower-http`, `subtle`, `sha2` absent).

## 7.10 Decision records this step proposes (placeholders)

```text
DEP-S11-a  secrets and their comparison: getrandom (already in the build graph) for invite and resume
           secrets; subtle for constant-time comparison. Rejected alternatives above.
DEP-S11-b  observation deltas: typed delta vs json-patch, decided by S11-C's measurement; records the
           permessage-deflate dead end (Godot) so nobody retries it blind.
ARC-S11-a  control is host state: seat bindings, sessions, holds and nicknames are never journaled, and a
           binding change moves no revision (I-2). AC-5 is measured on that.
ARC-S11-b  protocol revision 2 is specified whole and implemented incrementally under the "may omit,
           never redefine" rule (§5.1).
ARC-S11-c  hosted controllers run on the world thread behind a synchronous, bounded seam; asynchronous
           controllers connect as sessions.
ARC-S11-d  facts reach observers through perception's `learns`, judged at record time, delivered as a
           since-last-frame stream; JSON stays the wire encoding for MVP-0 (supersedes D-4's timing if
           the operator agrees, QS11-1).
```

---

# 8. Modularity and pluggability

## 8.1 Boundaries, and what each can be swapped for without touching the others

```text
module                     owns                                   may be replaced by           touches nothing in
server/src/protocol.rs     frames, codes, WorldSummary            a binary encoding (§7.1)     runtime, seats, packs
server/src/protocol/delta  ObservationDelta: diff + apply,        keyframes only (rev 2        runtime, perception
   (new, S11-C)            pure functions                         permits it), or json-patch
server/src/admission.rs    invite + nickname check; secrets       an account system later      runtime, seats
   (new, S11-A)
server/src/seats.rs        SeatTable: the binding state machine   —  (it is the concept)       transport, packs
   (new, S11-B)
server/src/hosted.rs       the HostedController seam; the         any synchronous controller   cognition (never named)
   (new, S11-B)            consult schedule
server/src/perception.rs   observe + learns (seam)                any perception system        transport
server/src/admin.rs        /admin routes, bearer check            removed entirely if no token runtime internals
   (new, S11-D)                                                   (routes not mounted)
server/src/session.rs      one socket: handshake, delta encoding  another transport (QUIC…)    world thread
server/src/runtime.rs      world thread: clock, allocator,        —                            sockets, HTTP
                           journal, sweep, dispatch to the above
tools/cli/src/hosted.rs    adapters RuleController/Paced →        another controller crate     server internals
   (replaces agent.rs)     HostedController
systems/presence           learns() from Visibility               any perception pack          server, cognition
clients/protocol/mineworld handshake, reconnect policy, delta     another client language      game code of S12/S14
                           application, MineWorldObservation
```

What can be removed with no edit elsewhere:

- `--town` and `--agent`: the server runs with no hosted controller and every seat `Free`; nothing in
  `server/src` names a controller (I-9).
- The admin surface: without `--admin-token` the routes are not mounted; deleting `admin.rs` deletes one
  `merge` call in `app.rs`.
- Deltas: the session sends only keyframes; rev 2 permits it and every client already handles it (§5.1).
- Event learning: a perception without `learns` uses the default and observers learn nothing — the
  revision-1 behaviour, and the safe one.
- The Godot module's reconnect policy: opt-in; a client that does not enable it gets revision 1's "a
  dropped connection ends".

## 8.2 No God object, and where growth goes instead

`runtime.rs` (435 lines) and `host.rs` (498) are at the review trigger (A-10). S11 does not grow either
into a manager: bindings go to `seats.rs`, the controller seam to `hosted.rs`, admission to
`admission.rs`, admin to `admin.rs`, the delta to `protocol/delta.rs`. `runtime.rs` gains only the calls
into them (join → `SeatTable`, tick → hosted consults, record → `learns`). Each PR's review checks the two
files' line counts and refuses to land one past 500 without the split recorded. `session.rs` keeps socket
work only; it never reads world state.

## 8.3 Nothing leaks across layers

| Leak that must not happen | Why it cannot |
| --- | --- |
| A System Pack or controller crate named by the server | `ac1_composability` check 2 and the I-2 scan fail by file and line (proven to bite in 11f); `server/Cargo.toml` gains no `mineworld-*` pack dependency (diff gate). |
| A nickname, token or binding in world state | I-2 and I-5 tests read the save. `Nickname` has no `Serialize` into any contract type. |
| A world rule in a client | Unchanged: clients render affordances; the delta applier is structural and decides nothing. |
| Transport concepts in the kernel | I-1: no kernel diff. |
| Wall-clock time in the world | The hold timer and admin timestamps are wall-clock host state; nothing on them reaches `dispatch` or a fact. |
| A rule controller blocking the world | `HostedController::decide` is documented bounded; CP-B4 measures the world thread's tick time with `--town` on `market-town`. |

---

# 9. The PRs

Five PRs. Each runs the product at its checkpoint — the real `mineworld` binary over real sockets, and
for the wire, a real Godot client — never only a unit. Adversarial criteria and their bounds are fixed
here, before anything is measured (`ARC-23`): bounds are literals derived from the requirement, never from
the implementation under test, and each PR's review plants one mutation to show its decisive test bites.

```text
S11-A  handshake, authentication, revision 2's frames ──┐
                                                        ▼
S11-B  seats, hold and resume, takeover, hosted controllers, F-13   (AC-3, AC-5, the town lives)
                                                        │
                       ┌────────────────────────────────┴───────────────┐
                       ▼                                                ▼
S11-C  facts in observations, and deltas                 S11-D  the admin surface
                       └────────────────────────────────┬───────────────┘
                                                        ▼
S11-E  the proof: AC-7, INV-9 over revision 2, and the far side (Godot) — no production change
```

## 9.1 S11-A — Handshake, authentication, and revision 2's frames

**Scope.** `PROTOCOL_VERSION = 2`. `join` with `protocol`, `invite`, `nickname`, `resume` (accepted and
ignored until S11-B, always answered `invalid_resume` if non-null); `leave` (closes the connection;
before S11-B there is no hold to skip); `closing`; the five new refusal codes; `welcome`'s new fields
(`session`, `nickname`; `resume: null`, `hold_seconds: 0`, `took_over: "none"` until S11-B);
`WorldSummary` revision 2 (`deferrals_unscheduled` removed; `provides`, `states`, `events_dropped`
added). `admission.rs` (`Admission`, `Nickname`, `InviteToken`, constant-time comparison, generation).
CLI: `--invite`, `MINEWORLD_INVITE`, generated invite printed as one join line. `PROTOCOL.md` rewritten to
revision 2 as specified in §5 (marking what lands in later PRs). The Godot module:
`connect_to_world(address, seat, invite, nickname)` and the new frames; every in-repo caller updated
(`clients/protocol/demo`, `clients/3d-spike/scripts/slice/slice_link.gd` l. 181, Rust test clients).
DEP-S11-a recorded.

**Integration checkpoint CP-A.** `mineworld server worlds/social-cafe` with no `--invite` prints a join
line; the protocol module's headless Godot run (`clients/protocol/run.sh`) joins with that invite and a
nickname, receives `welcome` with `protocol: 2`, submits a `talk` and gets its `result`; `/status` lists
`conversation` providing `talk` and stating `spoke`.

**Adversarial criteria (decided now).**
1. A join with the right invite and `protocol: 1` is refused `protocol_mismatch` and closed.
2. Missing invite, an invite differing only in its last character, and the invite with a trailing space
   are each refused `unauthorized`, each no sooner than 500 ms after the frame, then closed.
3. The generated invite appears exactly once in the server's captured stdout and in no byte of the save
   file (`--save`).
4. `INV-9`: a table of state-asserting messages (`set_state`, `move_to` with a position, `give`, a
   `submit` with another actor, a `join` carrying an `observer` field) is refused, and the world's
   revision and fact count are unchanged after the table.
5. `ac13_semantic_parity`, `ac15_one_alice`, `milestone_b`, `milestone_c` pass with only their join
   frames changed; I-6 digests unchanged.
6. Review mutation: an `Admission` that accepts any token must fail criterion 2.

**May run in parallel with:** S15 12c and 12d; S13. **Conflicts with:** S15 12e on `server/PROTOCOL.md`
and `clients/protocol/ADOPTION.md` (whoever merges second rebases; 12e edits only §6.2 and its ADOPTION
section); S14 on `slice_link.gd` (one call site).

**Detailed to the commit in §15** (PR design, awaiting the primary session's freeze).

## 9.2 S11-B — Seats, hold and resume, takeover, hosted controllers, `F-13`

**Scope.** `seats.rs` (`SeatTable`, §4.2) and `hosted.rs` (`HostedController`, consult schedule, §4.5) in
the server; `runtime.rs` routes join, leave, drop and tick through them; `welcome` gains real `resume`,
`hold_seconds`, `took_over`. CLI: `tools/cli/src/hosted.rs` (`ReactiveSeat`, `PacedSeat`) replaces
`agent.rs`; `--town`, `--seed`, `--pace`, `--hold`. `RuleController::since` (§4.6). Godot module: opt-in
reconnect with `resume`. ARC-S11-a, ARC-S11-c recorded.

**Integration checkpoints.**
- **CP-B1 (`AC-5`).** `mineworld server worlds/market-town --town --save DIR`. A client joins `alice`
  (`took_over: "hosted"`), talks to Bob, buys one item she is offered, and `leave`s; Alice returns to her
  paced controller. Read from the save: the journal length and revision are unchanged across the join
  and across the leave (I-2); Alice's `EntityId` is the one `validate` printed; her biography, `knows`
  edges and values, holdings, wallet and job before the join equal those after the leave once the facts
  caused by actions submitted in between are applied — and every such fact names an `ActionId` the
  human's session or her controller submitted.
- **CP-B2 (`AC-3`).** Same world. A client seated as `visitor` is killed (socket dropped, no `leave`).
  During the hold, `visitor`'s position in the save does not change and the world's revision advances
  (the town acts while nobody plays). A reconnect presenting `resume` inside the hold is granted
  `took_over: "held"`, the same observer, and a keyframe whose `at` is later than the last frame before
  the drop. After the hold, a reconnect without `resume` takes the seat back from the paced controller.
- **CP-B3 (`F-13`).** `mineworld server worlds/social-cafe --agent alice --save DIR`; a client talks to
  Alice and receives her answer; SIGKILL; restart on the same save; the client rejoins and waits 15 wall
  seconds without speaking. Alice states no `spoke` addressed to the client after the restart.
- **CP-B4 (the town lives).** `market-town --town --pace 5`, 120 wall seconds, two sessions connected:
  every hosted seat has at least 3 accepted `move`s; zero faults; the world thread's longest tick (a
  test-only probe) is at most 50 ms — half the 100 ms observation cadence, a bound from the cadence and
  not from the code.

**Adversarial criteria.**
1. Two joins and one resume racing on one seat produce exactly one `welcome`; the others are
   `seat_occupied` or `invalid_resume` (I-3).
2. A `HostedController` adapter that returns a request acting as another Person is refused
   `actor_not_observer` and counted, exactly as a session would be (I-8).
3. Review mutation: return a dropped seat to its hosted controller immediately (no hold) — CP-B2's
   "position unchanged during the hold" must fail. Second mutation: bind the reactive controller with
   `new()` instead of `since` — CP-B3 must fail.
4. I-6 digests unchanged; `cognition/rule-controller`'s paced tests unchanged and passing.

**May run in parallel with:** S15 12c–12e (no shared file except `ADOPTION.md`); S13; the start of S11-C's
pure commits (below). **Conflicts with:** S11-C and S11-D on `runtime.rs`/`host.rs`/`session.rs` — they
integrate after B.

## 9.3 S11-C — Facts in observations, and deltas

**Scope.** Perception seam `learns` (default false); presence implements it from `Visibility`;
`PackPerception` forwards it; per-subscriber pending facts, judged at record time, cleared on successful
queueing, bounded and counted (`events_dropped`). `protocol/delta.rs` (`ObservationDelta`, `diff`,
`apply`); the session sorts entities, emits keyframes and deltas; `--keyframe-every`. The measurement
(`json-patch` as a dev-dependency of one test). Godot module: `delta` application in
`MineWorldObservation`; `events()` documented as since-last-frame. DEP-S11-b, ARC-S11-d recorded.

Order inside the PR, so that work can start before S11-B merges: (1) the measurement harness and the pure
`delta.rs` with its Rust tests; (2) presence's `learns` with its tests; (3) runtime and session
integration, rebased on S11-B.

**Integration checkpoints.**
- **CP-C1 (measure before adopting).** Frames recorded from `market-town --town` with four sessions for
  60 s. Reported per client per second: whole JSON bytes; JSON Patch bytes; typed-delta bytes. Decision
  rule fixed now: ship the typed delta only if it is at most half the whole-frame bytes; adopt `json-patch`
  instead if its bytes are within 10 % of the typed delta's; if neither halves the bytes, ship keyframes
  only and record why.
- **CP-C2 (facts, `INV-13`).** In `social-cafe` hosted with `--town`, `visitor` and `wanderer` in the café
  and `hana` (hosted) on the street: a `talk` from `visitor` to Alice reaches `wanderer` as a `spoke`
  fact stated with `Place(café)` visibility and does not reach the street; a `Participants` fact reaches
  only its participants; a `SystemInternal` fact from a test pack reaches nobody (I-7).
- **CP-C3 (reconstruction).** For the CP-C1 run, the Rust applier's reconstruction of every frame equals
  the whole observation the server computed for it; at every keyframe the reconstructed and received
  observations are byte-equal after canonical sorting (I-10).

**Adversarial criteria.**
1. A client that stops reading for 5 s and resumes receives every fact it learned exactly once and in
   order, or `events_dropped` accounts for the difference exactly.
2. No fact appears in two frames of one connection.
3. Review mutation: ignore `entities.remove` in `apply` — CP-C3 must fail. Second: judge `Place`
   visibility at sweep time instead of record time — a CP-C2 case where the listener walks out in the same
   second must fail.
4. I-6 digests unchanged (presence's `observe` is untouched; only `learns` is added).

**May run in parallel with:** S11-D (rebase obligation on `runtime.rs`, `host.rs`); S15 12c; S13.
**Conflicts with:** anything editing `systems/presence/src` — S15's 12a–12e plan no presence edit after
12a, which must be re-checked against 12d's frozen change set before S11-C freezes.

**Detailed to the commit in §17** (DESIGN FROZEN 2026-10-08), which also carries coordination
rulings 1, 2 and 4: the `perceived` stream, `acted_through` and the module's readers. §19 states how
S11-C and S11-D share files when they run in parallel.

## 9.4 S11-D — The admin surface

**Scope.** `admin.rs`: the four routes of §4.9, bearer check (constant time), mounted only with
`--admin-token` / `MINEWORLD_ADMIN_TOKEN`; `Command` variants for listing, kicking and releasing;
`SeatTable` gains `kick` and `release`. `PROTOCOL.md` gains an admin section.

**Integration checkpoint CP-D.** Through the binary with `--town --save DIR --admin-token T`: two
sessions; `GET /admin/sessions` lists both with nicknames and seats; `GET /admin/seats` shows them
`connected` and the rest `hosted`; kicking one sends it `closing { reason: "kicked" }` and its seat shows
`hosted` at once (no hold); releasing the other's seat does the same.

**Adversarial criteria.**
1. In a hosted `social-cafe` with `--save`, no `--town`, no submitting client, inside the routine-free
   00:00–05:00 window the restart tests already rely on (so nothing is due), every admin route called
   twice leaves the save's revision and fact count exactly unchanged (I-4).
2. Without a configured token the routes answer 404; with a wrong token, 401 no sooner than 500 ms.
3. The admin token is in no save byte and no log line (I-5).
4. Review mutation: have `kick` also submit a no-op request as the kicked seat — criterion 1 must fail.

**May run in parallel with:** S11-C; S15; S13.

**Detailed to the commit in §18** (DESIGN FROZEN 2026-10-08), which adds the host clock routes
S19 needs (`step-19-time-weather.md` §7.3–§7.4, QTW-3): `GET`/`POST /admin/clock`, the `clock` frame,
`WorldSummary.paused`, the refusal `paused`, and the rule that only the holder of the admin token
changes time.

## 9.5 S11-E — The proof

No production change (as 11f was for `AC-1`). Committed acceptance tests named for their criteria, the
far-side Godot proof, and evidence.

**Integration checkpoints.**
- **CP-E1 (`AC-7`).** `mineworld server worlds/market-town --town --save DIR` with four clients under four
  nicknames as `visitor`, `wanderer`, `bob` and `carol` (two of them taken over from the town). All four
  `talk` to Alice; Alice's disclosed history holds all four lines; Alice answers each by name (she is
  driven by `--agent alice`). One client `buy`s from the café; the other three see the café's stock one
  lower in their own observations and none of the buyer's private state. All four `welcome`s and the last
  observation of each name the same instance and, after the purchase, the same revision.
- **CP-E2 (the far side, `R-9`).** A headless Godot run of the protocol module against the real binary:
  join with invite and nickname; apply deltas for 60 s and compare with every keyframe (I-10 from the far
  side); drop the socket and resume within the hold; receive a `spoke` fact; be kicked by the admin route
  and report `closing`. Evidence under `clients/protocol/evidence/rev2/`.
- **CP-E3 (`INV-9`, revision 2).** The table of §9.1 criterion 4 extended to `leave` misuse, a `join`
  after `leave`, and every admin route without a token, run against the binary.
- **CP-E4.** `ac13`, `ac15`, `milestone_b`, `milestone_c`, `ac1_composability`, the I-2 scan and both
  300-day digests pass on the PR head.

**Adversarial criterion.** The diff touches only `tools/cli/tests/`, `server/tests/`, `clients/protocol/`
(demo, run script, evidence) and Markdown; review plants one mutation — accept a second join on an
occupied seat — and CP-E1 must fail by name.

**May run in parallel with:** S12 and S14 client work, which it does not touch.

---

# 10. Acceptance mapping

| Criterion | Where it is proven | Test file (planned) |
| --- | --- | --- |
| `AC-3` disconnect, simulation continues, reconnect | S11-B CP-B2; end to end with the 2D client in S12 (`overall.md` §4) | `tools/cli/tests/ac3_reconnect.rs` |
| `AC-5` takeover with the Person intact | S11-B CP-B1 | `tools/cli/tests/ac5_takeover.rs` |
| `AC-7` several humans, the same NPCs | S11-E CP-E1 | `tools/cli/tests/ac7_many_players.rs` |
| `INV-9` state assertions refused | S11-A criterion 4; S11-E CP-E3 | `server/tests/two_clients.rs` (extended), `tools/cli/tests/inv9_closed_vocabulary.rs` |
| `F-13` closed | S11-B CP-B3 | `tools/cli/tests/restart.rs` (extended) |
| The hosted town lives | S11-B CP-B4 | `tools/cli/tests/hosted_town.rs` |
| `R-9` far side | S11-A CP-A; S11-E CP-E2 | `clients/protocol/run.sh`, evidence |
| `AC-15` not broken | every PR (I-12) | `tools/cli/tests/ac15_one_alice.rs` |

Step S11 is complete when S11-A … S11-E are merged, each checkpoint above has evidence on its PR head, and
the operator has run the short test list the primary session hands over at that milestone.

---

# 11. Parallelism and file ownership

## 11.1 Within S11

```text
S11-A  ─►  S11-B  ─►  S11-C (pure commits may start after A)  ─┐
                  └─►  S11-D                                    ├─►  S11-E
```

## 11.2 Against S12, S13, S14 and S15 (12c–12e)

| Other work | Relation | Rule |
| --- | --- | --- |
| **S15 12c** (frozen; no server, cli, presence or clients diff) | independent | fully parallel with every S11 PR |
| **S15 12d** (worlds, movement, worldpack, digests) | independent files; shares I-6's digests | S11 PRs compare digests against their own base; if 12d merges mid-PR, rebase and re-measure |
| **S15 12e** (3D client; `server/PROTOCOL.md` §6.2, `ADOPTION.md`) | **conflicts with S11-A** on two documents | whichever lands second rebases; 12e edits only §6.2 and its section of `ADOPTION.md`; 12e targets revision 2's handshake if S11-A is merged first |
| **S12** (2D client) | **consumer** | designs against §5 now; implements against `main` after S11-A merges; adopts deltas and facts after S11-C; never edits `clients/protocol/mineworld/` while S11 is open — module changes go through S11 |
| **S13** (CI, container) | consumer of the CLI contract | parallel; uses `MINEWORLD_INVITE` and `MINEWORLD_ADMIN_TOKEN` (frozen in §11.4); runs S11's new tests; decides whether CP-E2's Godot run joins CI |
| **S14** (3D client) | **consumer**; conflicts with S11-A on `slice_link.gd` l. 181 | as S12; S11-A changes that one call to pass invite and nickname |
| **Milestone E** | unknown to this step | if it needs a hosted world with players, it depends on S11-B (`--town`) and S11-A (invite) |

## 11.3 Files each PR touches

```text
S11-A  server/src/{protocol.rs, protocol/tests.rs, session.rs, app.rs, runtime.rs (summary only),
       admission.rs (new), lib.rs}; server/{Cargo.toml, PROTOCOL.md, README.md};
       server/tests/{two_clients.rs, headless.rs, support/mod.rs}; Cargo.toml (workspace deps),
       Cargo.lock; tools/cli/src/main.rs; tools/cli/tests/{support/mod.rs, server_command.rs,
       ac15_one_alice.rs, ac13_semantic_parity.rs, milestone_b.rs, milestone_c.rs, restart.rs};
       clients/protocol/{mineworld/world_client.gd, demo/demo.gd, run.sh, ADOPTION.md, README.md};
       clients/3d-spike/scripts/slice/slice_link.gd (one call); docs/MODULE_SPEC.md §8.1;
       docs/NETWORKING.md (none expected); docs/DECISIONS.md (DEP-S11-a)
S11-B  server/src/{seats.rs (new), hosted.rs (new), runtime.rs, host.rs, session.rs, protocol.rs,
       lib.rs}; server/tests/seats.rs (new); tools/cli/src/{main.rs, hosted.rs (new), agent.rs (deleted)};
       cognition/rule-controller/src/{lib.rs, tests.rs}; tools/cli/tests/{ac3_reconnect.rs,
       ac5_takeover.rs, hosted_town.rs (new), restart.rs, ac15_one_alice.rs};
       clients/protocol/{mineworld/world_client.gd, ADOPTION.md}; server/PROTOCOL.md;
       docs/MODULE_SPEC.md §8.1; docs/DECISIONS.md (ARC-S11-a, ARC-S11-c)
S11-C  server/src/{perception.rs, runtime.rs, session.rs, host.rs, protocol.rs, protocol/delta.rs (new)};
       server/{Cargo.toml (dev-dep json-patch), PROTOCOL.md}; Cargo.lock; server/tests/{facts.rs,
       deltas.rs} (new); systems/presence/src/{observe.rs or a new learns.rs, lib.rs} and its tests;
       tools/cli/src/{perceive.rs, main.rs}; tools/cli/tests/facts.rs (new);
       clients/protocol/{mineworld/observation.gd, mineworld/world_client.gd, ADOPTION.md};
       docs/DECISIONS.md (DEP-S11-b, ARC-S11-d)
S11-D  server/src/{admin.rs (new), app.rs, host.rs, runtime.rs, seats.rs}; server/tests/admin.rs (new);
       tools/cli/src/main.rs; server/{PROTOCOL.md, README.md}; docs/MODULE_SPEC.md §8.1
S11-E  tools/cli/tests/{ac7_many_players.rs, inv9_closed_vocabulary.rs} (new); server/tests/ (extended);
       clients/protocol/{demo/, run.sh, evidence/rev2/}; Markdown
```

No S11 PR touches `kernel/`, `contracts/`, `persistence/`, `worlds/`, `worldpack/`, `sdk/`, `authoring/`,
`systems/` other than `presence` (S11-C only), or any System Pack's vocabulary.

## 11.4 The CLI contract S13 and the clients' launchers build on (frozen with this step)

```text
mineworld server <world> [--listen ADDRESS] [--save DIR]
                         [--invite TOKEN]              env MINEWORLD_INVITE; generated and printed if absent
                         [--admin-token TOKEN]         env MINEWORLD_ADMIN_TOKEN; no admin routes if absent
                         [--agent SEAT]...             reactive rule controller, F-13-safe
                         [--town [--seed N] [--pace SECONDS]]   paced controllers on every other seat
                         [--hold SECONDS]              default 30
                         [--keyframe-every N]          default 50
```

The generated-invite line is `[mineworld] invite <token> — join with: <address> seat=<seat>
invite=<token>`; S13 and the launchers may parse the token after `invite ` on a line beginning
`[mineworld] invite`. A flag on the command line wins over its environment variable.

---

# 12. Questions

**Operator-material** means the answer changes an operator decision, the kernel, scope, or a frozen
contract; those need the operator, the rest the primary session.

| ID | Question | Recommendation | Material? |
| --- | --- | --- | --- |
| **QS11-1** | `D-4` (operator, 2026-09-25) says protobuf is introduced "at the first real cross-language boundary (S10 Python cognition, S11/S12 Godot)". The Godot boundary has run on JSON since S5V. Introduce protobuf now, or record that JSON stays for MVP-0? | **JSON stays for MVP-0** (§7.1): the mirror schema is the drift `R-3` names, `godobuf` is single-maintainer, and bandwidth is answered by deltas. Record ARC-S11-d as superseding `D-4`'s timing, not its direction. | **Yes** — revises an operator decision. |
| **QS11-2** | Hosted controllers on the world thread behind a bounded synchronous seam, or as tasks like `agent.rs`? | **World thread** (§4.5, ARC-S11-c); asynchronous controllers connect as sessions. | No (architecture record, primary session) — but flag to the operator because it shapes where an LM controller attaches in MVP-1. |
| **QS11-3** | Does `--town` drive the player seats (`visitor`, `wanderer`) while nobody plays them? | **Yes**: a Person is not a player (`INV-1`), and a seat taken over or handed back is the same mechanism for every seat. A world that wants a seat idle simply does not run `--town`, or names it in a later `--idle SEAT` (not planned). | No. |
| **QS11-4** | Hosted pace and time scale: the paced controller's rates are tuned for 900 s; at a 5 s pace a person greets someone about every 25 s. Keep `HostClock` at 1 s : 1 s with no `--rate`, default `--pace 5`? | **Yes for MVP-0**, measured in CP-B4; a controller tuned for real-time hosting is cognition work (S10/MVP-1), recorded as a follow-up rather than tuned here. | **Yes** — it is the felt life of the hosted world, a product judgement. |
| **QS11-5** | Seat hold after a drop: 30 s? | **30 s**, `--hold` to change it. Long enough for a Wi-Fi blink or a client restart; short enough that a quitter's Person rejoins the town. | No. |
| **QS11-6** | Is the invite required on loopback (single player on one machine)? | **Always required**: one path (`NETWORKING.md` §1); the server prints a ready join line, and launchers pass it. No `--open` flag. | **Yes** — user-facing launch flow for S12/S14 and the operator's own demos. |
| **QS11-7** | Are nicknames visible to other players? | **No**: only to the player's own connection and the admin surface; not world data. A "who is playing" UI is a later presentation feature over an explicit, separate frame. | **Yes** — product choice touching `INV-1`'s "the world cannot tell". |
| **QS11-8** | Facts in observations (presence's `learns`) in S11's scope? | **Yes**: the brief names it, S12 needs other people's speech and purchases, and presence's own doc assigns it to S11. | No. |
| **QS11-9** | Deltas in S11, gated by CP-C1's measurement? | **Yes**, with the rule fixed in CP-C1, including "ship keyframes only" as a legitimate outcome. | No. |
| **QS11-10** | Admin: read, kick, release only — no runtime enable/disable of systems (it needs a fourth `WorldInput` kind)? | **Yes**: enable/disable is a persistence-contract change and out of scope. | **Yes, only if the operator wants enable/disable in MVP-0** (scope + persistence contract). |
| **QS11-11** | Fix `F-13` here by `RuleController::since` (an edit in `cognition/`), rather than leave it to S10? | **Yes**: one constructor, no persisted controller state, `run` untouched. | No (moves a finding between steps; record in `overall.md` §7). |
| **QS11-12** | PR identifiers. | The primary session assigns real numbers; placeholders `S11-A … S11-E` here. | No. |
| **QS11-13** | Join brute-force protection: own "one guess per connection, 500 ms delay", or `governor` now? | **Own now**; `governor` recorded as the adopt route for public hosting. | No. |
| **QS11-14** | The invite and resume secrets cross a LAN in clear over `ws://`. Accept for MVP-0, with TLS from a gateway or tunnel (`NETWORKING.md` §7) documented by S13? | **Accept and document**; no TLS inside the server. | **Yes** — security posture. |
| **QS11-15** | One protocol revision (2) specified whole and implemented incrementally under "may omit, never redefine", or one revision number per PR? | **One revision** (§5.1, ARC-S11-b): S12/S14 build once. | No. |
| **QS11-16** | `took_over` in `welcome` tells a player the Person was living on its own. Keep? | **Keep**: it is the player's own binding history, not another player's data. | No. |

---

# 13. Edits proposed for the primary session (not made here)

```text
overall.md §3 S11      add "Design: step-12-server.md (DRAFT)"; Output gains "hosted controllers
                       (--town) and facts and deltas in observations"; acceptance unchanged
overall.md §4          AC-3 "S11 (CP-B2), end-to-end in S12"; AC-5 "S11 (CP-B1)" — no longer "with S10";
                       AC-7 "S11 (CP-E1)"
overall.md §5          on QS11-1: record D-4's timing superseded for MVP-0 (if the operator agrees)
overall.md §7          F-13 moves from S10 to S11 (S11-B); S11's remaining line becomes the five PRs
overall.md §6          add R-S11-1 … R-S11-6 (§14)
MVP_STATUS.md          S11 row: "step planned (step-12-server.md), five PRs"; Networking row unchanged
                       until S11-B merges
DECISIONS.md           DEP-S11-a, DEP-S11-b, ARC-S11-a … d, with real numbers at freeze, each landed in
                       the PR named in §7.10 rather than in advance
NETWORKING.md §5       after S11-E: note that admin commands are HTTP (§3) rather than WebSocket frames,
                       which §5's "Client → Server" list currently implies
MODULE_SPEC.md §8.1    the server line, per §11.4, in S11-A/B/D
```

---

# 14. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| **R-S11-1** | Hosted controllers and per-consult perception load the world thread; with `--town` on `market-town` and four clients, ticks lengthen and observation delivery stalls. | CP-B4's 50 ms bound, measured; pace raised before seats are dropped, as `ARC-27`'s rule does for `run`. |
| **R-S11-2** | A race between a drop, a resume and a takeover leaves two controllers on one seat (A-1 reintroduced). | One writer (`SeatTable` on the world thread); §9.2 criterion 1's race test. |
| **R-S11-3** | The Rust and GDScript delta appliers diverge; a client renders a world that never existed. | I-10 checked from both sides (CP-C3, CP-E2); periodic keyframes bound the damage. |
| **R-S11-4** | Protocol churn under S12/S14 while they build. | §5 frozen with this step; "may omit, never redefine"; any change is a revision of this step, sent to their owners. |
| **R-S11-5** | A persisted hosted world with `--town` journals every NPC request with power-loss durability; fsync rate limits the town. | CP-B4 runs with `--save`; if the bound fails, the remedy is a durability setting decided by the operator, not silently relaxed. |
| **R-S11-6** | Making seats exclusive breaks a test that relied on two connections per seat (e.g. `ac15_one_alice.rs`'s "cannot act as Alice" case, which may join Alice's seat). | Audited in S11-B's design before freeze; the test's claim is preserved by asserting `seat_occupied` or a takeover explicitly. |
| **R-S11-7** | The paced controller's chattiness at a 5 s pace makes the hosted town feel wrong. | QS11-4 puts it before the operator; measured counts in CP-B4's evidence. |
| **R-S11-8** | `PROTOCOL.md` and `ADOPTION.md` edited concurrently by S11-A and S15 12e. | §11.2's rebase rule. |

---

# 15. PR S11-A — handshake and authentication (full design)

**Lifecycle:** `MERGED` — GitHub #76, merge commit `f842c52` (2026-10-08), PR head `cc1f428`, final
executable head `76be4d2`. The primary session re-ran the gates on the merge (682 passed, 0 failed; scope
clean), planted its own mutation (an `Admission` admitting any invite) and saw both
`only_the_invite_itself_is_admitted` and the socket test `a_wrong_or_missing_invite_is_refused…` fail;
on `main` afterwards AC-1 13/13 and the I-2 scan 4/4. D-SA8 (the client closes on `closing`; the server
waits up to 2 s) and D-SA10 (S11-B moves `serve` out of `main.rs`) accepted. Before that: `READY FOR
OPERATOR REVIEW`; `DESIGN FROZEN (2026-10-08), primary session`; `PR DESIGN — DRAFT`.

**Freeze record.** The primary session's freeze message (2026-10-08) accepts §15 as written — SD-A1 …
SD-A14, SA-1 … SA-12 with their mutations, A-C1 … A-C8, the `protocol.rs` split first — and all six
points of §15.8 as recommended, and leaving `spike/unreal/probe.py` untouched. It adds one coordination
rule for item 4: S12's 13a (new `clients/2d/`, one `connect_to_world` call in its `link.gd` and one
launcher) and 16b (`clients/protocol/checks/`) will probably merge first; at every rebase S11-A merges
`origin/main` and updates every in-repo `connect_to_world` call site and launcher found on `main` at that
moment, including `clients/2d/` and `clients/protocol/checks/`, and shows that `git grep connect_to_world`
on the final head finds only the new signature (added to SA-9 and §15.5). Godot runs one window at a time.
Material stops: any kernel or contract change, any path outside the frozen change set.
Execution contract: §15.9, endpoint "implementation + local validation" now authorized by this message.
**Author:** the S11-A implementing session, 2026-10-08, worktree
`/Users/yuema137/mineworld-worktrees/impl-s11a`, branch `mvp0/pr-s11a-handshake`.
**Binding parents:** this file §§4.1, 5, 6, 7.1, 7.2, 7.7, 8, 9.1, 11.3, 11.4; `overall.md` "Parallel build-out,
2026-10-08" (operator decisions and coordination rulings 1, 4, 6, 7, 9, 10), which override this file where
they differ.

Evidence goes into §15.10 as `E-SA<n>`; deviations into §15.11 as `D-SA<n>`. A planned commit may become
several coherent commits; the mapping is recorded. Line counts are estimates, never targets.

## 15.1 Identity, base, approved scope

```text
PR            S11-A — handshake, authentication, and revision 2's frames (S11, first of five);
              PR number assigned at freeze (ruling 7)
base          main @ 47c81d1. The shared-module PR 16b is frozen and lands BEFORE S11-A (coordinator,
              2026-10-08): S11-A rebases over it before its Godot commit (A-C7) if it has merged by then,
              and in any case before the PR is marked ready. Re-audit of clients/protocol/ on that rebase.
branch        mvp0/pr-s11a-handshake, worktree impl-s11a, held by this session only
audit         §15.2 (files and symbols read on 47c81d1)
scope         §9.1 as amended by the rulings: protocol revision 2's handshake and frames (§5.2, §5.3
              minus `delta`, §5.5, §5.6), the admission module, the CLI's invite, the Godot module's join
              credentials and every in-repo call site, golden frames (S10's R-S11-7), PROTOCOL.md
              revision 2, DEP-14 and ARC-41
```

**Applied coordination rulings.**

- Decision numbers are ARC-40 … ARC-44 and DEP-14 … DEP-15 only (ruling 6). The placeholders of §7.10 map
  as follows. S11-A records only the first two rows; each other record lands in the PR named.

  | Placeholder | Number | Content | Recorded in |
  | --- | --- | --- | --- |
  | DEP-S11-a | **DEP-14** | secrets: `getrandom` for invite and resume secrets, `subtle` for constant-time comparison | S11-A |
  | ARC-S11-b + the JSON half of ARC-S11-d | **ARC-41** | revision 2 is specified whole and implemented incrementally ("may omit, never redefine"); JSON text stays the wire encoding for MVP-0, superseding `D-4`'s timing but not its direction (operator, QS11-1) | S11-A |
  | ARC-S11-a | **ARC-40** | control is host state; a binding change moves no revision | S11-B |
  | ARC-S11-c | **ARC-42** | hosted controllers on the world thread behind a bounded synchronous seam | S11-B |
  | ARC-S11-d (facts half) | **ARC-43** | facts reach observers through perception's judgement, at record time | S11-C |
  | DEP-S11-b | **DEP-15** | observation deltas, decided by CP-C1's measurement | S11-C |
  | — | ARC-44 | unassigned reserve | — |

  ARC-S11-b was assigned to no PR in §9; S11-A is the PR that raises `protocol` to 2 and writes the
  revision rule into `PROTOCOL.md`, so it records it. The JSON half moves from ARC-S11-d to ARC-41 because
  S11-A is the PR that puts revision 2 on the wire in JSON; S11-C's ARC-43 then carries only facts.
- An invite is required even on loopback, and the server prints a ready join line (QS11-6). There is no
  `--open` flag and no "no invite" mode. S12's R-S11-1 asked that "a server started without a token keeps
  accepting a join without one"; the operator's decision overrides it — localhost play stays one command
  because the server prints the join line and the launchers read it (SD-A11).
- Nicknames are hidden from other players (QS11-7): a nickname reaches only the player's own `welcome`
  (and, from S11-D, the admin surface). It is in no observation, no `/status`, no fact, no save and no
  server log line (SA-5).
- JSON on the wire (QS11-1): no encoding change; ARC-41 records it.
- The shared GDScript module (ruling 4): 16b lands first with the read-only additions
  (`MineWorldObservation.affordances(action_type, target)` widened, `complete_affordances`,
  `is_complete`, `affordances_about`, `component_value`; `MineWorldClient.submit_affordance`, `submit`'s
  payload widened to `Variant`, `revision` read from frames; `clients/protocol/checks/`). S11-A changes
  only the join credentials (`PROTOCOL`, `connect_to_world`, the join frame, the welcome's new fields,
  `closing`, `leave`) and every in-repo call site — **including any 16b check under
  `clients/protocol/checks/` that connects to a server** — and keeps 16b's names exactly as they are.
- Names: S11's names win (ruling 1). S14's R-S11-6 asked the 3D client to take `--token=`; it takes
  `--invite=`. S12's R-S11-1 suggested `bad_token`; the code is `unauthorized`.
- The other rulings' additions to revision 2 are **not** S11-A's: `take_over` in `join` and `time_scale`
  (S11-B), `acted_through` and the `perceived` stream with `cursor_unavailable` and `lagged` (S11-C).
  S11-A's `PROTOCOL.md` names each with its owning PR; the owning PR specifies its shape and semantics
  in `PROTOCOL.md` when it lands it. Until then a `join` carrying `take_over` or `perceived` is
  `malformed_frame` (§5.2's unknown-field rule), so a client sends a field only to a server whose
  `PROTOCOL.md` lists it as landed (SD-A2).
- Test hygiene (ruling 10): every new test removes its own scratch data (`SaveDir` drops its directory;
  a test that captures output writes it to memory, never to `target/`).

## 15.2 Source audit (`main @ 47c81d1`)

| File / symbol | Finding | Consequence for S11-A |
| --- | --- | --- |
| `server/src/protocol.rs` (513 lines) | `PROTOCOL_VERSION = 1`; `ClientFrame { Join { seat }, Submit }` with **no** `deny_unknown_fields` (an extra field is silently ignored today); `CLIENT_FRAME_TAGS = ["join", "submit"]`; `ServerFrame { Welcome, Observation, Result, Refused }`; `RefusalCode` (9); `WorldSummary` with `deferrals_unscheduled`; `SystemSummary { system, enabled }`. **Already past the 500-line review trigger.** | Split before growth (A-C2, SD-A13); `deny_unknown_fields` added (SD-A2); new variants. |
| `server/src/session.rs` (209) `run`, `handshake`, `stream`, `submit` | The handshake loops until `host.join(seat)` succeeds; every failure is a `refused` and the loop continues; a closed socket ends it. No delay, no closing frame, no credential. | The handshake gains the ordered checks of SD-A1, the fixed delay and `closing`. |
| `server/src/app.rs` (94) `router(host)`, `serve`, `serve_with_shutdown`, `health` | Router state is the `WorldHost` alone. | State becomes `{ host, admission, session ids }`; three public signatures gain an `Admission` (SD-A8). |
| `server/src/runtime.rs` (435) `summary` l. 410–434 | Builds `WorldSummary` from `world.systems()` (`order()`, `is_enabled()`); `deferrals_unscheduled: 0` hard-coded. | `summary` reads `Registry::declaration(id)` → `provides()`, `emits()` (`kernel/src/system.rs` l. 206, 211; `kernel/src/registry.rs` l. 104). No kernel change: both accessors are public. `emits()` already includes event types owned by other systems (`emitting::<E>` pushes every type, l. 169–174), which is §5.5's "states". |
| `server/src/host.rs` (498) `WorldHost::join(seat)` | Takes a seat only. | Unchanged in S11-A: the world thread never sees an invite or a nickname (SD-A8). |
| `server/Cargo.toml` | No `getrandom`, no `subtle`. | Two dependencies (DEP-14). |
| `Cargo.lock` | `getrandom 0.3.4` present as a **normal** dependency of `mineworld-server` already (`cargo tree -i getrandom@0.3.4`: rand_core → rand → tungstenite → tokio-tungstenite → axum → mineworld-server). `subtle` absent. | `getrandom` adds an edge, not a package; `subtle` adds one package. |
| `subtle` 2.6.1 (`cargo info`, 2026-10-08) | BSD-3-Clause; dalek-cryptography; no dependencies; default features `std`, `i128`. | §7.2's "to verify" closed: verified. |
| `constant_time_eq` 0.6.1 (`cargo info`) | CC0-1.0 OR MIT-0 OR Apache-2.0. | The recorded alternative; verified. |
| `getrandom` 0.3.4 (`cargo info`) | MIT OR Apache-2.0; latest is 0.4.3. | Use 0.3 (the version already compiled) rather than add a second copy. |
| Root `Cargo.toml` l. 72 | `clap = { version = "4.6.6", features = ["derive"] }`. | Add clap's `env` feature (no new crate) for `MINEWORLD_INVITE` with `hide_env_values` (SD-A9). |
| `tools/cli/src/main.rs` (466) `serve` l. 286–372 | Prints the pack line, listening line, entities/seats line, instance line; spawns `agent::drive` per `--agent` (in-process, through `host.join`, no socket). | Invite resolution and the join line go to a new `tools/cli/src/invite.rs` so `main.rs` stays under 500. `agent::drive` needs no invite: admission gates the socket, and an in-process controller is the composition root's own code. |
| `tools/cli/tests/support/mod.rs` `Server::start` l. 49, `Client::join` l. 194 | Every CLI acceptance test (ac13, ac15, milestone_b, milestone_c, restart, bodies_yard*, market_*) starts the binary and joins through these two. | Changing these two is "only the join frames changed" (§9.1 criterion 5); the test files themselves need no edit. |
| `tools/cli/tests/server_command.rs` l. 165 | One raw join frame (`"otto"`). | Updated. |
| `server/tests/two_clients.rs` | Own `start()` (l. 48, `app::serve`), own `Client::join` (l. 95), raw joins l. 532, 545; l. 569–638 asserts `after.deferrals_unscheduled == 0`. | `start` passes an admission; the deferral test keeps its claim (the deferred fact arrives, with its cause, later) and drops the dead field's assertion — the field no longer exists. |
| `server/tests/headless.rs` | No socket, no `WorldSummary` field asserted that changes. | Expected unchanged; touched only if the compiler says so. |
| `clients/protocol/mineworld/world_client.gd` | `PROTOCOL := 1`; `connect_to_world(address, seat_name)`; join sent in `_process` l. 213; `_receive` handles four kinds and warns on unknown ones. | SD-A10. |
| Call sites of `connect_to_world` | `clients/protocol/demo/demo.gd` l. 100; `clients/3d-spike/scripts/slice/slice_link.gd` l. 181 (from `slice_main.gd` l. 382); `clients/3d-spike/mineworld` is a symlink to the module. | SD-A11. |
| Launchers that start a server | `clients/protocol/run.sh` `start_server`; `mineworld-slice --world` (l. 69–80; its log is git-ignored, `clients/3d-spike/.gitignore` l. 6). | Both read the join line (SD-A11). |
| `clients/protocol/evidence/` | Committed transcripts and server logs; `tools/cli/tests/ac13_semantic_parity.rs` l. 42 reads `request-2d.json` / `request-3d.json` (submit frames only). | Regenerated by CP-A with the invite line kept out of every committed log (SD-A11). |
| `spike/unreal/probe.py`, `spike/**` | A hand-run probe of revision 1 with captured evidence; the spike server is a separate binary. | **Not updated**: frozen revision-1 evidence, not a client of `main`. Recorded, not a call site. |
| `docs/MODULE_SPEC.md` §8.1 l. 699 | The `server` line lacks `--invite`. | Updated in A-C1. |
| `docs/DECISIONS.md` | Highest ids on `main` and every `origin/*` branch: ARC-39, DEP-13; no branch holds ARC-40…44 or DEP-14…15 (checked 2026-10-08). | ARC-41 and DEP-14 are free. |
| `.github/workflows` | Absent at the base (S13's 13a is in flight). | CI evidence: see the contract (§15.9). |

## 15.3 Design decisions (SD-A1 … SD-A14)

| ID | Decision | Why |
| --- | --- | --- |
| **SD-A1** | **Order of the handshake checks**, in the session: (1) `protocol` — not 2 → `refused protocol_mismatch`, `closing { protocol_mismatch }`, close; (2) `invite` — wrong or missing → wait `UNAUTHORIZED_DELAY` (500 ms) from the frame's arrival, then `refused unauthorized`, `closing { unauthorized }`, close; (3) `nickname` — invalid → `refused invalid_nickname`, the connection stays in the handshake; (4) `resume` — non-null → `refused invalid_resume`, stays (S11-B gives it meaning); (5) `seat` → `host.join`, refusals unchanged from revision 1. | The protocol is checked before the credential, so a revision-1 client is told what is wrong rather than that its (absent) invite is wrong, and a mismatch reveals nothing secret. The invite before everything that consults the world, so an unauthenticated connection learns nothing about seats. One guess per connection (§7.7). |
| **SD-A2** | **`join`'s wire shape, decoded structurally.** `protocol` defaults to `1` when absent, so the literal revision-1 frame `{"t":"join","seat":"visitor"}` decodes and is answered `protocol_mismatch`, not `malformed_frame`. `invite` and `nickname` default to `""` when absent and are then answered by SD-A1's checks (missing invite → `unauthorized`, as §9.1 criterion 2 requires). `seat` is required. `resume` is optional, default `null`. **Every client frame denies unknown fields** (`#[serde(deny_unknown_fields)]`): §5.2's rule, which is what turns a `join` carrying `observer` into `malformed_frame`. The offered invite is held in an `OfferedInvite` newtype whose `Debug` is redacted. `take_over` and `perceived` are not accepted until S11-B and S11-C land them. | §5.2. Without `deny_unknown_fields` today an extra field is ignored, so a client that typed `obsever` would never know. If serde cannot combine `deny_unknown_fields` with the internally tagged enum, the bounded fallback is an explicit per-variant key list in `decode` (recorded as a deviation). |
| **SD-A3** | **`leave`**: on a seated connection, the session releases the subscription (`host.leave`), sends `closing { left }` and closes. Before a seat, `leave` is answered the same way (there is nothing to release, and a client asking to go is let go). No hold exists before S11-B. | §5.2. Leaving is always honoured; a `not_joined` refusal to a client that wants to go would leave it holding a socket for nothing. |
| **SD-A4** | **`closing`**, a `ServerFrame` variant with `reason: ClosingReason` (`left`, `kicked`, `superseded`, `unauthorized`, `protocol_mismatch`, `world_stopped`, `server_stopping`) and an optional `detail`, sent immediately before the server closes the socket. S11-A sends `left`, `unauthorized`, `protocol_mismatch`, and `world_stopped` (the world's observation channel ended). `kicked` and `superseded` arrive with S11-B/S11-D; `server_stopping` with the S11 PR that wires graceful shutdown into sessions. | §5.3, §5.1's "may omit, never redefine": every reason is defined now and sent only when true. |
| **SD-A5** | **`welcome`, revision 2**: `protocol`, `seat`, `observer`, `nickname` (as trimmed and accepted), `session` (a `SessionId`, decimal string), `resume: null`, `hold_seconds: 0`, `took_over: "none"`, `world`. `SessionId` is allocated per connection by the router state (an `AtomicU64` behind an `Arc`), never reused within a process, and has no other use until S11-D. `took_over` is a `TookOver` enum (`none`, `hosted`, `held`). **Known limitation until S11-B:** seats are not exclusive (A-1), so a human joining a seat an `--agent` drives is told `"none"` while the agent keeps acting — the defect S11-B removes, stated in `PROTOCOL.md`. | §5.3, §9.1. |
| **SD-A6** | **`WorldSummary`, revision 2**: `deferrals_unscheduled` removed; each `systems[]` entry gains `provides` (`ActionTypeId`s) and `states` (`EventTypeId`s) from the kernel's declaration; `events_dropped: 0` added (no fact is delivered before S11-C, so none is dropped — the value is true, not a placeholder); `clients` keeps its revision-1 meaning in S11-A (every subscription, `--agent` included) until S11-B makes hosted controllers non-clients. | §5.5. Composition, not state: public. |
| **SD-A7** | **`server/src/admission.rs`**: `InviteToken` — an operator-given token is 8–128 bytes of printable ASCII without whitespace (it survives a shell, an environment variable and the join line); a generated one is 16 bytes from `getrandom::fill` as 32 lowercase hexadecimal characters. `Debug` prints `InviteToken(<redacted>)`; no `Serialize`, no `Display` except an explicit `reveal()` the CLI calls once. `Nickname` — trimmed, 1–32 Unicode scalar values, no control character; serializes as its text (it appears in `welcome`). `Admission { invite }`, `admit(&self, &OfferedInvite) -> Result<(), Unauthorized>` by `subtle::ConstantTimeEq` over the bytes; `UNAUTHORIZED_DELAY = 500 ms`, applied by the session. `subtle` returns early on unequal lengths: the invite's length is not secret (a generated one is always 32), and this is documented beside the call. | §4.1, §7.2 (DEP-14), §7.7. One module, no world access (§8.1). |
| **SD-A8** | **Admission is the transport's, never the world's.** `app::router(host, admission)`, `app::serve(listener, host, admission)`, `app::serve_with_shutdown(listener, host, admission, shutdown)`. `WorldHost::join(seat)` keeps its signature: the world thread never holds an invite or a nickname, so neither can reach a journal, a fact or a save (I-5 by construction). | A-9 (the world thread owns only world-touching state); §8.1. |
| **SD-A9** | **CLI.** `--invite TOKEN`, with `env = "MINEWORLD_INVITE"` and `hide_env_values = true` (clap's `env` feature; the flag wins over the variable, which is clap's own precedence). A supplied token that is not a legal `InviteToken` stops the server with a message that does **not** echo it. When neither is given, a token is generated and printed once, after the listening line, exactly in §11.4's frozen form: `[mineworld] invite <token> — join with: <address> seat=<seat> invite=<token>`. When one is supplied, the server prints `[mineworld] join with: <address> seat=<seat> invite=<the invite you gave>` and never the token. `<address>` is the bound `host:port`; `<seat>` is the first roster seat no `--agent` drives, or the first seat if every one is. Resolution and the line live in `tools/cli/src/invite.rs`; `main.rs` gains the flag and one call. | §4.1, §11.4, QS11-6. The supplied-token line starts `[mineworld] join`, not `[mineworld] invite`, so §11.4's parsing rule never mistakes it for a token line. `--help` must not print the environment value. |
| **SD-A10** | **The Godot module.** `PROTOCOL := 2`. `connect_to_world(address: String, seat_name: String, invite: String, nickname: String)` — typed, required parameters rather than M-5's `credentials := {}`: S11 owns the shape (ruling 1) and the invite is always required, so a default would only move the failure to a server refusal. The join frame sends `protocol`, `invite`, `nickname`, `seat` and `resume: null`. `_welcome` keeps `seat`, `observer`, `world` and adds members `nickname`, `session`, `took_over`, `hold_seconds`, `resume` (stored, unused until S11-B's reconnect policy). A new signal `closing(reason: String, detail: String)` is emitted for a `closing` frame, and `close_reason` holds the reason, before `disconnected`. `leave_world()` sends `leave`; `disconnect_from_world()` still only drops the socket. The module never prints, logs or emits the invite. No reconnect and no delta (S11-B, S11-C). 16b's API is untouched. | §5, ruling 4, R-9. |
| **SD-A11** | **Call sites and launchers.** `demo.gd`: `--invite VALUE` and `--nickname VALUE` (default nickname `demo`). `run.sh`: starts each server **without** `--invite`, captures its stdout in a scratch file under `$TMPDIR`, waits for the `[mineworld] invite` line, extracts the token, passes it to every Godot client; the committed `server*.log` files receive the server's output with that one line removed (`grep -v '^\[mineworld\] invite '`), so no token — even a dead one — enters the repository history before the repository goes public. `slice_link.gd`: `invite_from_args()` (`--invite=`), `nickname_from_args()` (`--nickname=`, default `player`), `start(address, seat, invite, nickname)`; `slice_main.gd` passes them. `mineworld-slice --world`: reads the join line from its (git-ignored) log and passes `--invite=`; `--server=host:port` is documented to need `--invite=`. Every 16b check under `clients/protocol/checks/` that joins a server is updated the same way after the rebase. | §9.1 "every in-repo caller"; I-5. |
| **SD-A12** | **Golden frames** (S10's R-S11-7, carried by ruling 1). `server/tests/frames/<kind>.json` for every frame S11-A defines: client `join`, `submit`, `leave`; server `welcome`, `observation`, `result`, `refused`, `closing`. `server/tests/frames.rs` builds one canonical example of each from the Rust types (fixed ids, `WorldInstanceId::from_raw`), and fails, naming the file, if the example's serialization differs from the file (compared as JSON values) or the file does not decode back to the example (client frames through `ClientFrame::decode`). Tests never write files; a developer who changes a frame updates its file by hand, which is the reviewable act. S11-B … S11-D add their frames' files. | R-S11-7: the Python SDK (S10 P3, after S11-A) reads these. The example is reviewed JSON in the repository, not the implementation's output compared with itself (rules §25): the file is the oracle. |
| **SD-A13** | **Split `protocol.rs` before it grows** (it is 513 lines): `protocol/request.rs` (`WirePayload`, `into_kernel_request`, `CorrelationToken`, `MAX_TOKEN_LENGTH`), `protocol/summary.rs` (`WorldInstanceId`, `WorldSummary`, `SystemSummary`); `protocol.rs` keeps the frames, `RefusalCode`, `Refusal`, `ProtocolError`, and re-exports the moved items so every path `mineworld_server::protocol::X` and every `lib.rs` re-export is unchanged. A pure move, in its own commit. | `ENGINEERING_STANDARDS.md` §10; §8.2's "refuses to land one past 500 without the split recorded". |
| **SD-A14** | **`PROTOCOL.md` revision 2**, rewritten in A-C1 from §5: the three client frames, the server frames (with `delta` and `ObservationDelta` specified and marked "from S11-C"), the handshake order of SD-A1, the closing reasons, the refusal codes, `WorldSummary` revision 2, HTTP, what revision 2 does not have, and a **landing table** — for each frame and field, the PR that lands it and what the server sends before that (§5.1). The rulings' additions (`take_over`, `time_scale`, `acted_through`, the `perceived` stream) are listed in the landing table with their owning PR and specified there by that PR. §6 (submitting) and §6.2 (the reporting rule) are carried verbatim, so S15's 12e edit of §6.2 rebases cleanly. | §5.1, §15.1's ruling notes; `CLAUDE.md` §2.2 (spec before code). |

## 15.4 Acceptance (decided before measuring, `ARC-23`)

Every bound below is a literal from the requirement (§§4.1, 5, 9.1, 11.4), never from the implementation.
Each guard names the mutation that must turn it red (planted on the working tree, never committed,
reverted, recorded in §15.10).

```text
SA-1  Protocol first. A join with the right invite and "protocol": 1 is answered refused
      protocol_mismatch, then closing {reason: protocol_mismatch}, then the socket closes; so is the
      literal revision-1 frame {"t":"join","seat":"bob"} (no protocol field). No welcome on either.
      [server/tests/two_clients.rs]
      Mutation M-SA1: skip the protocol check  → a welcome arrives; the test fails.

SA-2  Invite. Three joins, each on its own connection: invite missing; the invite with its last
      character changed; the invite with a trailing space. Each is answered refused unauthorized no
      sooner than 500 ms after the frame was sent (measured by the test's own clock), then closing
      {reason: unauthorized}, then the socket closes; none receives a welcome.
      [server/tests/two_clients.rs]
      Mutation M-SA2a (§9.1 criterion 6): Admission::admit returns Ok for any token → fails.
      Mutation M-SA2b: remove the delay → the 500 ms assertion fails.

SA-3  The generated invite is a secret. `mineworld server worlds/social-cafe --save DIR` with no
      --invite and no MINEWORLD_INVITE: exactly one stdout line contains the token, and it is the
      §11.4 line (begins "[mineworld] invite ", contains "join with: ", "seat=", "invite=<token>");
      a client joining with it is welcomed; after SIGKILL, the token is in no byte of stderr and no
      byte of any file under DIR. With MINEWORLD_INVITE=T: a join with T is welcomed, and T is in no
      byte of stdout or stderr. With both --invite F and MINEWORLD_INVITE=T: F is welcomed, T is
      refused unauthorized (the flag wins). An illegal supplied invite ("short") stops the server
      with a non-zero status and a message that does not contain it.
      [tools/cli/tests/server_command.rs, the real binary]
      Mutation M-SA3: print the token a second time (on the "listening" line) → the one-line count
      fails.

SA-4  INV-9 over revision 2, through the real binary on a persisted world. `mineworld server
      worlds/social-cafe --save DIR` (no --agent), inside the routine-free first minutes the restart
      tests already rely on. One seated client sends, in order: {"t":"set_state",…},
      {"t":"move_to","position":{…}}, {"t":"give",…} → each unknown_frame; a submit whose actor is
      another Person → actor_not_observer. A second, unjoined connection sends a join carrying an
      "observer" field → malformed_frame and no welcome. Afterwards GET /status's revision equals its
      value before the table; after SIGKILL, `mineworld inspect DIR` reports exactly as many facts as
      `mineworld validate worlds/social-cafe` reports genesis facts (an oracle independent of the
      server: nothing but genesis was ever recorded).
      [tools/cli/tests/server_command.rs]
      Mutation M-SA4: remove deny_unknown_fields from ClientFrame → the observer-carrying join is
      welcomed; the test fails.

SA-5  Nicknames are hidden. Client A joins as nickname "  Zephyrine-7 " and is welcomed with
      "Zephyrine-7" (trimmed). Client B, on another seat, then reads its welcome, two seconds of its
      observations, its refusals and a result, and GET /status: none contains "Zephyrine-7"; nor does
      the server's stdout or stderr (real binary). Invalid nicknames — "   ", 33 scalar values, one
      containing U+0007 — are each answered invalid_nickname with the connection left open, and a
      retry with a valid nickname on the same connection is welcomed. Exactly 32 scalar values
      (including multi-byte ones, "é" × 32) is accepted.
      [server/tests/two_clients.rs; the output check in tools/cli/tests/server_command.rs]
      Mutation M-SA5: the length check counts bytes instead of scalar values → "é" × 32 is refused;
      the test fails. (Hiding itself is structural — no path carries a nickname to the world thread,
      SD-A8 — so no mutation can plant a leak without adding that path; the test is the regression
      guard, said honestly per the test rules §24.)

SA-6  Welcome and leave, revision 2. A welcome carries protocol 2, the seat, the observer, the
      nickname, a session that is a decimal string distinct from every other connection's,
      resume null, hold_seconds 0, took_over "none". A join with a non-null resume is answered
      invalid_resume and the same connection then joins without it. `leave` on a seated connection
      → closing {reason: left}, the socket closes, and /status's clients falls back by one; `leave`
      before a seat → closing {left}. A second join on a seated connection → already_joined (as in
      revision 1). A submit before a seat → not_joined.
      [server/tests/two_clients.rs]
      Mutation M-SA6: do not release the subscription on leave → the clients count assertion fails.

SA-7  /status, revision 2, through the real binary on social-cafe: protocol 2; no key
      "deferrals_unscheduled"; the "conversation" entry's provides contains "talk" and its states
      contains "spoke"; events_dropped is 0. GET /health says protocol 2. (CP-A's status half.)
      [tools/cli/tests/server_command.rs]
      Mutation M-SA7: take states from subscribes() instead of emits() → "spoke" missing; fails.

SA-8  Golden frames. Eight files under server/tests/frames/; server/tests/frames.rs passes; renaming
      one field of one frame (hold_seconds → hold) fails it naming welcome.json.
      Mutation M-SA8: exactly that rename.

SA-9  The far side (R-9, CP-A). `clients/protocol/run.sh evidence`, on a server started with no
      --invite: the script reads the printed line; each headless Godot 4.7.2 client joins with that
      invite and a nickname, logs a welcome with protocol 2, walks, submits `talk` and logs its
      result accepted; Alice (--agent) answers. The two request-*.json files AC-13 reads are
      regenerated and `ac13_semantic_parity` passes on them. No committed evidence file contains a
      line beginning "[mineworld] invite ". `./mineworld-slice --world --link` (headless) reports its
      round trip passing with the new start() call. `git grep -n connect_to_world` on the final head
      finds only calls with the four-argument signature (freeze coordination rule).
      Negative far-side run (once, evidence only): the demo with a wrong --invite logs refused
      unauthorized, then closing unauthorized, and never a welcome.

SA-10 Nothing else moved. Every existing test passes; the only edits to existing tests are (a) the
      join helpers in tools/cli/tests/support/mod.rs, server/tests/two_clients.rs and the raw join in
      server_command.rs, (b) two_clients.rs's deferral test losing its assertion on the removed
      field, (c) new tests. ac13_semantic_parity, ac15_one_alice, milestone_b, milestone_c and
      restart pass with no edit to their own files. The 300-day seed-7 `run` digests of social-cafe
      and market-town at the PR head equal those at the base (E-SA0; I-6).

SA-11 Scope (I-1, I-9). `git diff --name-only 47c81d1...HEAD` (or the rebased base) ⊆ §15.5's change
      set; nothing under kernel/, contracts/, persistence/, worlds/, worldpack/, systems/,
      cognition/, authoring/, sdk/, tests/acceptance/. server/Cargo.toml gains exactly `subtle` and
      `getrandom`; Cargo.lock gains exactly the `subtle` package and those two edges of
      mineworld-server; no mineworld-* pack or controller dependency enters the server.
      ac1_composability and the I-2 scan pass.

SA-12 Size. At the head, protocol.rs, session.rs, runtime.rs, host.rs and tools/cli/src/main.rs are
      each under 500 lines, or the split is recorded as a deviation.
```

## 15.5 Change set

```text
server/src/{protocol.rs, protocol/request.rs (new), protocol/summary.rs (new), protocol/tests.rs,
            admission.rs (new), session.rs, app.rs, runtime.rs (summary only), lib.rs}
server/{Cargo.toml, PROTOCOL.md, README.md}
server/tests/{two_clients.rs, frames.rs (new), frames/*.json (new, eight)}
            (headless.rs and support/mod.rs only if the compiler requires it)
Cargo.toml (workspace: subtle, getrandom; clap's env feature), Cargo.lock
tools/cli/src/{main.rs, invite.rs (new)}
tools/cli/tests/{support/mod.rs, server_command.rs}
clients/protocol/{mineworld/world_client.gd, demo/demo.gd, run.sh, ADOPTION.md, README.md,
                  evidence/* (regenerated by run.sh)}
clients/protocol/checks/** (only 16b's checks that join a server, after the rebase)
clients/2d/** (only S12 13a's connect_to_world call site and its launcher, after the rebase; freeze)
clients/3d-spike/scripts/slice/{slice_link.gd, slice_main.gd}; mineworld-slice
docs/{DECISIONS.md (DEP-14, ARC-41), MODULE_SPEC.md §8.1}
.structured-coding/plans/mvp0/{step-12-server.md §15, handoff.md}
```

## 15.6 Commit plan

### A-C0 — Design (this section) — docs only

- [x] Implementation: §15 and the pointer in §9.1, from the audit in §15.2.
- [x] Validation: Markdown only; `git diff --stat` shows this file alone.
- [x] Review: every finding in §15.2 cites a file and line, a command or a measurement; every ruling of
  `overall.md` "Parallel build-out" that bears on S11-A is applied in §15.1; each acceptance guard has a
  named mutation or an honest statement why it has none (SA-5's hiding, SA-9, SA-10, SA-11 are checks,
  not guards). Self-review only; the primary session's freeze is pending.

### A-C1 — Specs before code: `PROTOCOL.md` revision 2, DEP-14, ARC-41, MODULE_SPEC §8.1

**Goal.** Revision 2 and its two records exist as reviewable specifications before code relies on them
(`CLAUDE.md` §2.2). **Scope.** `server/PROTOCOL.md` rewritten per SD-A14; `docs/DECISIONS.md` gains DEP-14
(§7.2's three tables with the licences verified in §15.2, the isolating interface — `admission.rs` is the
only file naming either crate — and the revisit trigger: durable player identity or public hosting, with
`governor` named as the adopt route for rate limiting) and ARC-41 (§5.1's rule; JSON stays for MVP-0;
what it supersedes of `D-4` and why, and that `ARCHITECTURE.md` §13.1's stale sentence is S10 P3's G-1 edit,
recorded here so the disagreement is not silent); `docs/MODULE_SPEC.md` §8.1's `server` line gains
`[--invite TOKEN]` and its table row the environment variable and the join line. **Depends on:** freeze.
**Non-goals:** no code; no edit to `NETWORKING.md` (§13 leaves its §5 note to S11-E).

- [x] Implementation: the three documents (layout: D-SA2).
- [x] Validation: `python3 scripts/check_decision_ids.py`; `python3 scripts/check_doc_headings.py`;
  ARC-41 and DEP-14 are the only new ids (E-SA1).
- [x] Review: `PROTOCOL.md` against §5 line by line (every field of §5.2–§5.6 present, every "from S11-x"
  absence stated as an omission, never a different meaning); §6 and §6.2 byte-identical to the base;
  terminology per `CORE_CONCEPTS.md` (seat, observer, Person; a nickname labels a player, not a Person).

**Acceptance.** As validation and review. **Commit boundary.** Documentation only.

### A-C2 — `protocol.rs` split (pure move)

**Goal.** SA-12 before growth (SD-A13). **Scope.** `server/src/protocol.rs`, `protocol/request.rs`,
`protocol/summary.rs`; `protocol/tests.rs` imports only if a path changes. **Non-goals:** no behaviour,
signature or public path changes.

- [x] Implementation: move the items; `pub use` them from `protocol`; module docs say what lives where.
- [x] Validation: `cargo test -p mineworld-server` — the same test names and count as at the base, all
  passing; `cargo clippy -p mineworld-server --all-targets -- -D warnings`; `git diff -M --stat` shows the
  moves.
- [x] Review: `lib.rs`'s re-export list unchanged; `cargo doc`-visible paths unchanged (the crate's public
  items are listed before and after and compared). *Done as:* `lib.rs` untouched by A-C2 and every
  moved item re-exported from `protocol` under its old name; the workspace (cli, tests) compiled with no
  import edit, which is the dependents' own check. No `cargo doc` listing was made (E-SA2).

**Failure case.** Any test change needed means the move was not pure: fix the move.

### A-C3 — `admission.rs`: invite, nickname, constant-time admission

**Goal.** SD-A7 as a unit with no transport. **Scope.** `server/src/admission.rs`, `lib.rs` (module and
re-exports `Admission`, `InviteToken`, `Nickname`, `OfferedInvite`, `UNAUTHORIZED_DELAY`),
`server/Cargo.toml` (`subtle`, `getrandom`, both `workspace = true`), root `Cargo.toml`
(`subtle = "2.6"`, `getrandom = "0.3"`), `Cargo.lock`. **Depends on:** A-C1.

- [x] Implementation: the four types and the constant, with module documentation of what each secret
  is, where it may appear (the one join line) and where it may not (I-5).
- [x] Validation (unit, in the module): a generated invite is 32 lowercase hexadecimal characters and two
  generations differ; an operator token of 7 bytes, of 129 bytes, with a space, with a control character,
  with a non-ASCII character is refused and the refusal's text does not contain the token; `admit` accepts
  exactly the token and refuses a one-character change, a trailing space, a prefix, the empty string;
  `format!("{:?}")` of an `InviteToken` and of an `OfferedInvite` does not contain the token; nickname
  rules as SA-5 (trim, 1–32 scalar values with "é" × 32 accepted and × 33 refused, U+0007 refused).
  `cargo clippy -D warnings`. `cargo tree -i subtle` shows `mineworld-server` alone depending on it.
- [x] Review: `subtle` and `getrandom` are named nowhere but `admission.rs`; no `Serialize` on a secret;
  no `Display` on a secret but the explicit `reveal()`.

### A-C4 — Revision 2's frames, `WorldSummary` revision 2, golden frames

**Goal.** The protocol layer of revision 2 (SD-A2, SD-A4, SD-A5's types, SD-A6, SD-A12), without the
session. **Scope.** `protocol.rs`: `PROTOCOL_VERSION = 2`; `ClientFrame::Join { protocol, invite:
OfferedInvite, nickname: String, seat, resume: Option<String> }` with the defaults of SD-A2, `Leave`,
`deny_unknown_fields`, `CLIENT_FRAME_TAGS = ["join", "submit", "leave"]`; `ServerFrame::Welcome` gains
`nickname: Nickname`, `session: SessionId`, `resume: Option<String>`, `hold_seconds: u32`, `took_over:
TookOver`; `ServerFrame::Closing { reason: ClosingReason, detail: Option<String> }`; `RefusalCode` gains
`ProtocolMismatch`, `Unauthorized`, `InvalidNickname`, `SeatOccupied` (defined; sent from S11-B),
`InvalidResume`. `summary.rs`: `WorldSummary` revision 2, `SystemSummary { system, enabled, provides,
states }`. `runtime.rs` `summary()`: reads the declarations; `deferrals_unscheduled` gone;
`events_dropped: 0`. `protocol/tests.rs` updated; `server/tests/frames.rs` and `server/tests/frames/*.json`.
The session is adapted only as far as needed to compile (it sends the new welcome fields with fixed
values); the handshake logic is A-C5. **Depends on:** A-C2, A-C3.

- [x] Implementation: as scoped.
- [x] Validation: unit tests in `protocol/tests.rs` for decode — the literal revision-1 join decodes with
  `protocol == 1`; a join without `invite`/`nickname` decodes with empty values; a join with `observer`,
  a submit with an extra field and a `leave` with a field are `malformed_frame`; `move_to`/`give`/`set_state`
  are `unknown_frame`; `leave` decodes. SA-8 with M-SA8. `cargo test -p mineworld-server`; clippy.
- [x] Review: no field of a contract type re-declared (welcome still carries `EntityId`, `WorldSummary`
  by the contract's serde); `SeatOccupied` is defined and documented as "from S11-B" and nothing sends it;
  every code in §5.6 present with its `PROTOCOL.md` meaning; `runtime.rs` diff is `summary()` alone.

### A-C5 — The handshake: ordered checks, delay, `closing`, `leave`

**Goal.** SA-1, SA-2, SA-5 (socket half), SA-6 through a real socket. **Scope.** `session.rs` (SD-A1,
SD-A3, SD-A4, SD-A5's allocation), `app.rs` (router state, three signatures, SD-A8), `lib.rs` (doc example),
`server/tests/two_clients.rs` (`start()` builds an `Admission` from a fixed test invite; the join helpers
send `protocol`, `invite`, `nickname`; new tests for SA-1, SA-2, SA-5, SA-6; the deferral test as
§15.2 states). So that no commit leaves the CLI's acceptance tests red, this commit also carries the CLI's
invite (SD-A9): `tools/cli/src/invite.rs`, `--invite` in `main.rs`, clap's `env` feature, and
`tools/cli/tests/support/mod.rs` (`Server::start` passes a fixed test invite unless the arguments carry
one; `Client::join` sends the revision-2 join) and the raw join in `server_command.rs`. A-C6 then adds the
real-binary acceptance tests. **Depends on:** A-C4.

- [x] Implementation: as scoped. `session.rs`'s handshake is split into one function per check if it
  passes ~50 lines (standards: one conceptual operation per function).
- [x] Validation: `cargo test -p mineworld-server` (two_clients, headless, frames, unit); SA-1, SA-2, SA-5,
  SA-6 with mutations M-SA1, M-SA2a, M-SA2b, M-SA5, M-SA6, each recorded and reverted; clippy; the
  suites that start the binary — `ac13_semantic_parity`, `ac15_one_alice`, `milestone_b`, `milestone_c`,
  `restart`, `server_command`, `bodies_yard`, `bodies_yard_restart`, `market_town`, `market_composition`,
  `social_composition` — pass with no edit to their own files (SA-10's first half).
- [x] Review: the delay runs in the session task and never on the world thread (I-11); a connection that
  fails the invite can send nothing further (the socket is closed after `closing`); no `eprintln!`/`println!`
  in the server prints a frame, a nickname or an invite; `session.rs` stays socket work only (§8.2).

### A-C6 — The CLI's invite and the real-binary acceptance

**Goal.** SA-3, SA-4, SA-5 (output half), SA-7 through `mineworld server`. **Scope.**
`tools/cli/tests/support/mod.rs` (a variant of `Server::start` that sets environment variables and
captures stdout and stderr in memory), `tools/cli/tests/server_command.rs` (new tests for SA-3, SA-4,
SA-5's output check, SA-7); `tools/cli/src/invite.rs` only if a test finds a defect (recorded).
**Depends on:** A-C5.

- [x] Implementation: as scoped.
- [x] Validation: `cargo test -p mineworld-cli --test server_command`; M-SA3, M-SA4, M-SA7 recorded and
  reverted; `mineworld server --help` shows `MINEWORLD_INVITE` and no value of it; clippy.
- [x] Review: the token reaches stdout only through `invite.rs`'s one line; `main.rs` < 500 lines; the
  join line matches §11.4 character for character. *Result:* the first and third hold (SA-3's test);
  `main.rs` was 486 at A-C6 and is 523 after merging E-a's `packs` command (D-SA10).

### A-C7 — The Godot module, its call sites and the far side (CP-A)

**Goal.** SD-A10, SD-A11, SA-9. **Scope.** Rebase over 16b first if it has merged (re-run the A-C6
validation's Rust half if the rebase touched Rust; it should not). `clients/protocol/mineworld/world_client.gd`,
`clients/protocol/demo/demo.gd`, `clients/protocol/run.sh`, `clients/protocol/ADOPTION.md` (§2's API:
`connect_to_world`'s parameters, `closing`, `leave_world`, the welcome members; §6: authentication is now
done, reconnect and deltas are not yet), `clients/protocol/README.md`, `clients/protocol/evidence/*` (as
`run.sh evidence` writes it), 16b's checks that join a server, `clients/3d-spike/scripts/slice/slice_link.gd`,
`slice_main.gd`, `mineworld-slice`. **Depends on:** A-C6.

- [x] Implementation: as scoped; 16b's names untouched (`git diff` of the module shows only the
  credential, welcome, `closing` and `leave` hunks).
- [x] Validation (background, each run waited on with Monitor): `clients/protocol/run.sh evidence` (SA-9);
  `cargo test -p mineworld-cli --test ac13_semantic_parity` on the regenerated evidence;
  `grep -rn '^\[mineworld\] invite ' clients/protocol/evidence` → nothing; the negative far-side run with a
  wrong invite (transcript kept under `/tmp`, not committed); `./mineworld-slice --world --link`; any 16b
  check that runs headless against a server.
- [x] Review: the module never logs the invite (`grep -n invite` over the module shows only the frame
  field and the parameter); `took_over`, `session`, `hold_seconds`, `resume` read with their revision-2
  meaning; a `closing` frame is reported before `disconnected` with its reason; no world rule entered a
  client.

### A-C8 — Close: README, digests, scope, full gate, ledger, PR

**Goal.** SA-10, SA-11, SA-12 and the review-ready handoff. **Scope.** `server/README.md` (the join line
and the invite in the short orientation), §15.10 evidence, §15.11 deviations, `handoff.md`.

- [x] Implementation: the README; the ledger.
- [x] Validation: the 300-day seed-7 runs of both towns at the head against E-SA0 (SA-10); SA-11's diff
  and `Cargo.lock` audit; `wc -l` for SA-12; **one** full gate on the final executable head, in the
  background: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, `cargo test --workspace --no-fail-fast`, `check_decision_ids`, `check_doc_headings`; test
  counts and wall time recorded; scratch data under `target/` and `$TMPDIR` checked for leftovers.
- [x] Review: SA-1 … SA-12 each hold with recorded evidence; every mutation was planted, seen red and
  reverted (`git status` clean of it); deviations bounded; the PR body lists the conflict surfaces
  (`PROTOCOL.md` §6.2 with S15 12e, `slice_link.gd` with S14) and marks the PR **READY FOR OPERATOR REVIEW
  — DO NOT MERGE**.

**E-SA0 (the first action after the freeze, before any code changes):** build the base
binary and record the 300-day seed-7 `run` digests (sha-256 of every output line but `wall`) of
`worlds/social-cafe` and `worlds/market-town`, with fact totals and faults, as the parity reference.

## 15.7 Test ownership for S11-A

```text
STATIC      cargo fmt, cargo clippy -D warnings: formatting, unused items, exhaustive matches over the
            new RefusalCode / ClosingReason / TookOver variants
UNIT        admission: invite parsing and generation, constant-time admit's equality semantics,
            redacted Debug, nickname rules (A-C3); protocol decode: defaults, unknown fields, the new
            tags (A-C4); golden frames (A-C4)
INTEGRATION server/tests/two_clients.rs over a real socket: the handshake's order, delay, closing,
            leave, welcome fields, nickname hiding between clients (A-C5)
REAL BINARY tools/cli/tests/server_command.rs: the generated, supplied and environment invite, the
            secret-in-output/save checks, INV-9 against a persisted world, /status revision 2; every
            existing CLI acceptance test, unchanged (A-C6)
FAR SIDE    Godot 4.7.2 headless: run.sh evidence, the wrong-invite run, mineworld-slice --world --link
            (A-C7) — R-9's rule, the Gate-2-like layer for this PR
REAL RUN    the two 300-day digests (SA-10)
GATE 1      NOT REQUIRED — no language model anywhere in S11-A
CI          no workflow at the base. If S13's 13a merges a workflow before the final head, the PR's CI
            on the exact final head is the canonical full-suite evidence and the local full gate is
            not repeated (test rules §9); otherwise the one local full gate of A-C8 is
```

## 15.8 Is any of this material?

Nothing in S11-A changes `kernel/`, `contracts/` or `persistence/`, a frozen invariant of an earlier step,
or the behaviour of any world. The public wire contract changes, but that change is the step's frozen §5
under operator decisions already taken (QS11-1, -6, -7, -14). Points the primary session should confirm at
freeze, none of which is believed to need the operator:

1. **ARC-41 absorbs the JSON half of ARC-S11-d**, and S11-A records ARC-41 (ARC-S11-b had no PR). S11-C's
   ARC-43 then covers facts only.
2. **The rulings' additions to revision 2 are specified by their owning PRs** (`take_over`, `time_scale` in
   S11-B; `acted_through`, `perceived` in S11-C). S11-A's `PROTOCOL.md` lists them in the landing table,
   and its decoder refuses them as unknown fields until they land. The alternative — accepting and
   ignoring them now — would fix their shapes in S11-A's design ahead of their owners.
3. **§9.1 criterion 3 vs §11.4's line.** §11.4's frozen join line contains the token twice (`invite <token>
   — … invite=<token>`), so "appears exactly once in stdout" is read as **exactly one line**. SA-3 measures
   it that way.
4. **`connect_to_world` takes typed `invite` and `nickname` parameters**, not M-5's `credentials := {}`.
   S12 (13e) and S14 consume it as such.
5. **`took_over: "none"` while seats are not exclusive** (SD-A5): true to the frozen §9.1 text and stated in
   `PROTOCOL.md` as the defect S11-B removes.
6. **Golden frames (R-S11-7) are in S11-A's scope**, as ruling 1 requires S11 to carry S10's requirements
   and S10's P3 starts after S11-A.

**Material stops during execution:** any needed edit under `kernel/`, `contracts/`, `persistence/` or any
path outside §15.5; an existing test failing for a reason other than SA-10's listed edits; either digest
differing from E-SA0; a 16b name that S11-A would have to change; a dependency beyond `subtle` and the
`getrandom` edge.

## 15.9 Execution contract (proposed; confirmed only by the primary session's freeze)

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-A — handshake and authentication, protocol revision 2
                    part 1 (PR number assigned at freeze)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §15; evidence §15.10 (E-SA<n>);
                    deviations §15.11 (D-SA<n>)
RELATED / BINDING   this file §§4.1, 5, 6, 7.1, 7.2, 7.7, 8, 9.1, 11.3, 11.4; overall.md "Parallel
                    build-out, 2026-10-08" (decisions; rulings 1, 4, 6, 7, 9, 10); docs/NETWORKING.md
                    §§2, 3, 5, 9, 10; server/PROTOCOL.md; docs/DECISIONS.md DEP-3, ARC-23, ARC-25;
                    docs/REUSE_POLICY.md; docs/ENGINEERING_STANDARDS.md; CLAUDE.md §§2–4
IMPLEMENTATION BASE main @ 47c81d1, rebased over 16b when it merges; branch mvp0/pr-s11a-handshake;
                    worktree /Users/yuema137/mineworld-worktrees/impl-s11a, held by this session only
APPROVED SCOPE      §15.5's change set; A-C1 … A-C8; SD-A1 … SD-A14
FROZEN INVARIANTS   I-1: no diff under kernel/, contracts/, persistence/ (nor worlds/, worldpack/,
                    systems/, cognition/, authoring/, sdk/, tests/acceptance/).
                    I-5: no invite, offered invite or nickname reaches the world thread, a save, an
                    observation, a fact, /status or a log line, except the one generated-invite line.
                    I-6: both 300-day digests equal E-SA0.
                    I-9: the server names no System Pack and no controller crate.
                    I-12: ac13, ac15 and the milestone tests pass with no edit to their own files.
                    16b's GDScript names unchanged. No hold, takeover, delta or fact delivery (S11-B/C).
SEQUENCE            E-SA0 → A-C1 → A-C2 → A-C3 → A-C4 → A-C5 → A-C6 → (rebase over 16b) → A-C7 → A-C8;
                    each commit pushed when coherent
VALIDATION BUDGET   unit/integration/static unrestricted; 300-day runs (~15 s each) at most six;
                    run.sh evidence at most three runs and mineworld-slice --world --link at most three,
                    each in the background; one full workspace gate on the final head (background,
                    ~5 min); about one hour in total; real-model: NOT REQUIRED
LIVE DOCUMENTATION  §15 checkboxes; §15.10 evidence; §15.11 deviations
HANDOFF             .structured-coding/plans/mvp0/handoff.md, reinitialized for S11-A at A-C1
ENDPOINT AUTHORITY
  implementation + local validation   authorized (source: the primary session's freeze message,
                                      2026-10-08)
  semantic commits, branch push       authorized after the freeze (source: the S11-A brief, "commit and
                                      push after each step"; D-12)
  PR creation / update                authorized (source: the brief, "open a PR marked READY FOR
                                      OPERATOR REVIEW"; D-12)
  CI repair to review readiness       authorized if a workflow exists on the final head; otherwise N/A
  merge                               operator only, explicit, never inherited (source: the brief,
                                      "Do not merge"; D-12)
POST-MERGE SYNC     this session owns §15 (merge identity, evidence, deviations, remaining issues);
                    the primary session owns this step's §§1–14 and header, overall.md and MVP_STATUS
NORMAL STOP         PR S11-A READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       §15.8's list
```

## 15.10 Evidence ledger

```text
E-SA0 2026-10-08, base main @ 47c81d1 (worktree impl-s11a, before any code change). Debug build
      (`cargo build -p mineworld-cli`), binary copied to /tmp/s11a/mineworld-base.
      `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, 339 lines,
        sha-256 of every line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
        (= step-10 E-0 and step-11 E-RS0); wall 22.0 s.
      `mineworld run worlds/market-town --headless --seed 7 --days 300` → exit 0, 355 lines,
        sha-256 = 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d (= step-10 E-P0).
      Filter: `grep -v '^wall\|wall '` then `shasum -a 256`. Outputs under /tmp/s11a/base-*.out.
E-SA1 A-C1. check_decision_ids → 53 ids, all distinct (ARC-41, DEP-14 new). check_doc_headings → 176
      numbered sections across 25 documents, none duplicated. PROTOCOL.md: §6 and §6.2 carried —
      `diff` against the base shows §6.2 identical and one cross-reference in §6 ("§5" → "§5.2");
      §7 gains SessionId and the integer fields; section numbers 1–8 keep their revision-1 meaning
      (routes, client frames, identity, sequence, server frames, submitting, numbers, rates), so every
      existing "PROTOCOL.md §n" reference in the repository still points at the right section (audited
      with git grep: §1, §5, §6, §6.2, §7, §8 in code, tests and docs); §9 is now "what revision 2
      does not have" (references to "§9" in spike/unreal/probe.py and UNREAL_ADAPTER_SPIKE.md meant
      revision 1's "not in revision 1" — frozen spike evidence, left).
E-SA2 A-C2 (995c30d). protocol.rs 513 → 288 lines; request.rs 122, summary.rs 136. clippy -D warnings
      clean; `cargo test -p mineworld-server`: 19 unit + 4 headless + 9 two_clients + 1 doc, all pass,
      the same tests as the base (tests.rs untouched). Public paths re-exported unchanged.
E-SA3 A-C3. admission.rs (with 5 unit tests): `cargo test -p mineworld-server --lib admission` 5
      passed; clippy clean. `cargo tree -i subtle -e normal` → mineworld-server (→ mineworld-cli) only.
      Cargo.lock: +1 package (subtle 2.6.1) and the server's two dependency lines; nothing else.
E-SA4 A-C4 … A-C6 (one commit, D-SA3), working tree on b6075e2 + the commit's diff.
      cargo fmt --check clean; clippy --workspace --all-targets --all-features -D warnings clean.
      `cargo test -p mineworld-server -p mineworld-cli --no-fail-fast`: every binary passes —
      server: unit 26, frames 8, handshake 7, headless 4, two_clients 9, doc 1; cli: ac13 2, ac15 6,
      biography 2, bodies_yard 3, bodies_yard_restart 1, commands 4, content_kinds 3, create 2,
      inspect 3, market_composition 1, market_town 1, milestone_b 1, milestone_c 1, restart 2,
      routines 1, run 3, run_restart 2, server_command 7, social_composition 4. ~4 min wall. The CLI
      acceptance files (ac13, ac15, milestone_b/c, restart, …) have no diff: only support/mod.rs's
      join and Server::start changed (SA-10 first half).
      Flake observed once before D-SA5: "cannot listen on 127.0.0.1:50918: Address already in use";
      server_command then 3/3 runs green after D-SA5.
      Mutations (each planted on the working tree, seen red, reverted; final `git diff` free of them):
        M-SA1  protocol check disabled            → handshake a_join_of_another_revision… FAILED
        M-SA2a admit() always Ok                  → handshake a_wrong_or_missing_invite… FAILED,
                                                    server_command a_given_invite…flag_beats… FAILED
        M-SA2b delay removed                      → handshake a_wrong_or_missing_invite… FAILED
        M-SA3  join line printed twice            → server_command a_generated_invite… FAILED
        M-SA4  deny_unknown_fields removed        → server_command state_assertions… FAILED
        M-SA5  nickname length in bytes           → handshake an_invalid_nickname… FAILED
        M-SA6  no host.leave on leave             → handshake leave_gives_the_seat_up… FAILED
        M-SA7  as designed (states ← subscribes()) stayed GREEN: conversation subscribes to its own
               `spoke`, so the mutation does not change the observable. Replaced by states ← empty
               → server_command status_names… FAILED. Recorded rather than hidden (rules §24).
        M-SA8  hold_seconds renamed "hold"        → frames welcome FAILED, naming welcome.json
      Sizes (SA-12): protocol.rs 353, session.rs 349, runtime.rs 441, host.rs 498 (unchanged),
      tools/cli/src/main.rs 486 — all under 500.
E-SA5 Rebase: merged origin/main @ 889d217 (16b #66, 12c #67, docs #64/#65/#68/#69) as 9cdf1b0. One
      conflict, docs/DECISIONS.md (both appended at the end): 12c's DEP-13 note kept before ARC-41 and
      DEP-14; check_decision_ids 53 distinct, check_doc_headings clean. clients/2d is not on main yet
      (S12 13a unmerged), so the call sites on main are the module, demo.gd, 16b's
      checks/affordances_check.gd and the 3D slice.
E-SA6 A-C7, Godot 4.7.2.stable.official.ed1daf0bf, one window at a time.
      - First negative run (wrong invite, headless demo): the client logged only "disconnected: the
        connection closed (0 )" — neither the refusal nor `closing` (FAIL of SA-9's negative half).
        Second, with a 1000 close frame and a grace for the client's reply: same, code 1000. Cause:
        Godot's WebSocketPeer discards frames it has not handed out when a close frame arrives in the
        same read. Fix (D-SA8): the server sends `closing`, waits up to 2 s for the client to close,
        then closes; the module closes on `closing`, and drains frames in CLOSING/CLOSED too. Third
        run: "refused: unauthorized — that is not this server's invite", "the server is closing the
        connection: unauthorized", "disconnected: the server closed the connection: unauthorized" —
        PASS. Neither invite appears in either log (grep count 0 and 0). Transcripts under /tmp/s11a,
        not committed (§15.6 A-C7).
      - `run.sh evidence` (final module): exit 0; every transcript logs "seated: … protocol 2, as
        'demo-<flavour>-<seat>', took over: none"; the AC-13 talk is answered by Alice (agent);
        request-2d.json and request-3d.json byte-identical to the base (no diff);
        `cargo test --test ac13_semantic_parity` 2 passed on them. Servers ran on OS-chosen ports
        (55869 …), started without --invite; the script read address and invite from the join line.
        `grep -rln '^\[mineworld\] invite ' clients/protocol/evidence` → nothing.
      - `run.sh affordances` (16b's live check, now joining with the invite): "PASS — 0 failure(s)".
      - `run.sh` windowed: screenshot evidence/demo-scene.png, inspected — seat 'wanderer' = observer
        18, Alice, Bob, Wes, Vera drawn, talk answered ("accepted (4 fact(s))").
      - `./mineworld-slice --world --link` (headless, server on 127.0.0.1:0, invite from its line):
        "[link] seated as visitor [observer 17 …]", "all link checks pass", exit 0, 2 min 10 s.
      - `git grep -n "connect_to_world("` over *.gd and *.sh: only the four-argument definition and
        its three callers (slice_link.gd, checks/affordances_check.gd, demo/demo.gd).
      - Integers (16b's D-2): the join's one number, `protocol`, is the module's int constant
        `PROTOCOL`, never a parsed value, so JSON.stringify writes `2`; `ClientFrame::Join.protocol`
        is a u32 and serde_json refuses `2.0` for it, so every welcome above is evidence it went out
        as an integer. `leave` carries no number. Read side: `hold_seconds` through int().
      - After the closing change: server suites green again (handshake 7 passed, 7.5 s with the
        grace), server_command 7 passed; clippy clean.
E-SA7 A-C8, the close.
      - Second rebase: merged origin/main @ 1a1d08e (E-a #70, S17 plan #71) as 76be4d2. Conflicts:
        tools/cli/src/main.rs (`mod invite;` beside E-a's `mod packs;`, both kept) and
        docs/DECISIONS.md (ARC-41, DEP-14 kept before E-a's ARC-53, DEP-21). check_decision_ids 55
        distinct; check_doc_headings 177 sections, none duplicated. No new connect_to_world call site
        on main (clients/2d not merged yet).
      - SA-10 digests at 9cdf1b0's executable content (+ README only after): social-cafe
        ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b (339 lines), market-town
        365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d (355 lines) — both = E-SA0.
        (E-a's merge touches packages/ and the CLI's `packs` command, not `run`.)
      - SA-11: `git diff --name-only origin/main...HEAD` ⊆ §15.5 (plus D-SA1's handoff file and
        D-SA4's handshake.rs); nothing under kernel/, contracts/, persistence/, worlds/, worldpack/,
        systems/, cognition/, authoring/, sdk/, tests/acceptance/. Cargo.lock: +subtle package, the
        server's +getrandom +subtle lines, nothing else of ours. `git grep getrandom|subtle` in *.rs:
        admission.rs only.
      - SA-12: protocol.rs 353, session.rs 369, runtime.rs 441, host.rs 498, main.rs 523 (D-SA10).
      - `mineworld server --help` with MINEWORLD_INVITE set shows "[env: MINEWORLD_INVITE]" and not
        the value (grep count 0).
      - FULL GATE on the final executable head 76be4d2, 13:43–13:50 PDT: `cargo fmt --all --check`
        OK; `cargo clippy --workspace --all-targets --all-features -D warnings` clean;
        check_decision_ids 55 distinct; check_doc_headings OK; `cargo test --workspace --no-fail-fast`
        153 test binaries, 682 tests passed, 0 failed, 7 min 4 s wall. Log /tmp/s11a/full-gate.log.
        Scratch: one leftover $TMPDIR/mineworld-cli-83715-2d-i5-variantfull, not a name any S11-A
        test uses (ruling 10's bounded PR owns the sweep).
      - CI: no workflow on main at the final head (S13's 13a unmerged); the local full gate above is
        the canonical full-suite evidence (§15.7).
```

## 15.11 Deviations and discoveries

```text
D-SA1 (bounded) Handoff file. Six lanes run in parallel and the effort's single handoff.md would be
      rewritten by each; S11-A keeps its continuation aid in handoff-s11a.md and leaves handoff.md
      untouched.
D-SA2 (bounded) PROTOCOL.md layout. §15.3 SD-A14 lists the content; the numbering keeps revision 1's
      section numbers for the same subjects (E-SA1) so no reference elsewhere goes stale: joining's
      checks are §4.1, WorldSummary is §5.7, the landing table and the revision rule are §10.
D-SA3 (bounded) Commit mapping. A-C4, A-C5 and A-C6 land as one commit: the new frames cannot compile
      without the session's handshake, and the router's new signature cannot compile without the CLI
      passing an admission, so any split leaves the workspace or the CLI suites red in between.
D-SA4 (bounded) The socket tests of SA-1, SA-2, SA-5 and SA-6 live in a new
      server/tests/handshake.rs (with its own small client that sees the server close) rather than in
      two_clients.rs, which is 640 lines; two_clients.rs changes only as §15.2 says. support/mod.rs
      gains INVITE, admission() and join_frame().
D-SA5 (bounded) Test harness ports. tools/cli/tests/support's free_port() releases the port before the
      binary binds it; with more servers per run, and other worktrees' suites on the same machine, a
      run failed once with "Address already in use" (E-SA4). Server::start now restarts the binary on a
      new port, at most three times, when it exits before answering /health; a server that runs but
      stays silent still fails the test. Server::start_captured shares the same launch.
D-SA6 (bounded) The join line's seat. "The first roster seat no --agent drives" is the roster's order,
      which is key order (SeatRoster is a BTreeSet): social-cafe's line suggests `alice` with no
      --agent. SA-3's test takes the expected seat from /status rather than assuming `visitor`.
D-SA8 (bounded, a discovery from the far side — flagged to the primary session) The closing
      handshake. SD-A4 said the server sends `closing` "immediately before the server closes the
      socket". Against Godot that loses `closing` and the refusal before it (E-SA6). PROTOCOL.md §5.6
      now states: the client closes the socket on receiving `closing`; the server waits up to 2 s for
      that, then sends its own close (1000) and drops the connection. No frame or field changes; the
      module closes on `closing`; Rust test clients that do not close simply see the server close
      after the grace. Also: the module drains queued frames in CLOSING/CLOSED states, and demo.gd no
      longer treats a token-less refusal as its stride's (it matched the empty token).
D-SA9 (bounded) Launchers bind 127.0.0.1:0 and read the address as well as the invite from the join
      line (coordinator's port-collision finding): run.sh no longer pkills by name and stops only its
      own PID; mineworld-slice --world no longer uses 7979. The committed server logs therefore name
      ephemeral ports.
D-SA10 (bounded, reported) tools/cli/src/main.rs is 523 lines after the second rebase: 466 at the base,
      +20 for S11-A (the flag, one resolution call, the join line), +37 from E-a's `packs` command
      merged in. Past the ~500 review trigger, far from the 800 warning; still one responsibility
      (argument parsing, dispatch, `serve`). Not split here — moving `serve` out would touch E-a's
      code in an S11-A PR. Recommended for S11-B, which rewrites `serve` for --town anyway: extract
      `serve`/`persisted` into tools/cli/src/serve.rs.
D-SA7 (bounded) ClientFrame no longer derives Serialize: nothing serialized a client frame, and an
      OfferedInvite (a possible near miss of the secret) should not be serializable. `Leave` is an
      empty struct variant (`Leave {}`) so that deny_unknown_fields applies to it.
```

---

# 16. PR S11-B — seats, hold and resume, takeover, hosted controllers, `F-13` (full design)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session`. Superseded: `PR DESIGN — DRAFT`.

**Freeze record.** The primary session's freeze message (2026-10-08) accepts §16 as written and rules
QS11B-1 … QS11B-6 as recommended: (1) a hosted seat is taken over without a flag; (2) `took_over:
"connection"` and `closing.reason: "taken_over"` are added; (3) any invite holder may take over any seat,
held seats included — recorded in ARC-40 as an **MVP-0 limitation** (one shared invite means LAN-friends
trust; per-player identity and accounts are an explicit non-goal, "matchmaking, global accounts"; the
primary session informs the operator); (4) hold in wall seconds, pace in world seconds; (5) the shutdown
statistics line is CP-B4's probe; (6) no merge order with E-b or IL-a. Implementation is done by a fresh
session in its own worktree on `mvp0/pr-s11b-seats` (§16.9's endpoint "implementation + local
validation" is authorized by this message).
**Amendment 2026-10-08 (operator ruling QTW-13, relayed by the primary session).** QS11B-4 is reversed
for consult cadence: `--pace` is measured in **wall** seconds, not world seconds, so that changing the
time scale never speeds up NPC movement or talk (`step-19-time-weather.md` §4.4 and its QTW-13 row, on
`plan/s19-time-weather`). Applied in S11-B as follows, and only in hosted serve mode: the seam stays in
world time (SD-B5 unchanged); the CLI adapters (SD-B7) state their cadence as `cadence × time_scale`
world seconds — `PacedSeat` consults seat `k` at `genesis + (k + m·pace)·time_scale` and tells
`PacedRuleController` its pace in world seconds (`pace × time_scale`, its answering window);
`ReactiveSeat` is consulted every wall second (`time_scale` world seconds). SD-B9's `--pace` reads "wall
seconds"; `--hold` was already wall time. At `--time-scale 1` (every existing test, SB-6) nothing
changes. Headless `run`, `run.rs` and the 300-day digests are untouched (I-6). `ARC-40`'s limitation and
`MODULE_SPEC.md` §8.1 are amended with the date and the citation. The `--pace` → `--cadence` rename
QTW-13 suggests, and rescheduling pending consults on a live scale change, are S19's (TW-c); the form
`cadence × s_current` is the one S19 §4.4 reschedules. Evidence E-SB5; deviation record D-SB6.

**Author:** the S11 implementing session, 2026-10-08, worktree
`/Users/yuema137/mineworld-worktrees/impl-s11a`, branch `mvp0/pr-s11b-seats` (from `main @ f842c52`).
**Binding parents:** this file §§4.2–4.6, 5 (as `server/PROTOCOL.md` revision 2 now states it), 6, 7.4,
7.5, 8, 9.2, 11; §15 as merged (S11-A's handshake, D-SA8, D-SA10); `overall.md` "Parallel build-out,
2026-10-08" rulings 1, 3, 6, 7, 9, 10; the primary session's S11-B brief (2026-10-08). Evidence in §16.10
(`E-SB<n>`), deviations in §16.11 (`D-SB<n>`).

## 16.1 Identity, base, scope

```text
PR            S11-B — seats, hold and resume, takeover, hosted controllers, F-13 (S11, second of five)
base          main @ f842c52 (S11-A merged as #76); merge origin/main at each rebase
branch        mvp0/pr-s11b-seats, worktree impl-s11a, held by this session only
scope         §9.2 as amended by the brief:
                one controller per seat (SeatTable); a hold after a dropped socket and resume (AC-3);
                takeover with `take_over: true` and release (AC-5); in-server controllers on the world
                thread (--town paced, --agent reactive) — the hosted town lives; --time-scale reported
                in the welcome (ruling 1, S12's R-S11-6); F-13 by RuleController::since (ruling 3);
                `serve` moved out of tools/cli/src/main.rs (D-SA10); the Godot module's opt-in
                reconnect with `resume` and its `take_over` argument
not in scope  admin routes, kick and release over HTTP (S11-D); facts in observations, deltas,
              acted_through, the perceived stream (S11-C); an asynchronous (LM) controller (S10)
```

**Coordination with lanes in flight (the brief's point 3).**

| Lane | What it does to S11-B's files | Rule |
| --- | --- | --- |
| **S16 E-b** (`mvp0/pr-eb-requirements`, unmerged) | Adds `--packs` (`PackDirs`, `PackRoots`) to every command including `server`; `serve(…, roots)` reads the pack with `read_with` (`tools/cli/src/main.rs`). | B-C2 moves `serve`/`persisted` verbatim into `tools/cli/src/serve.rs` behind a `ServeRequest` struct. If E-b lands first, S11-B rebases and carries `roots` into `ServeRequest`. If S11-B lands first, E-b's `serve` hunk moves to `serve.rs` as one field and one argument. Either way the flag stays declared in `main.rs`'s `Subcommand::Server`. |
| **IL-a** (`mvp0/pr-il-a-seam`, frozen) | One configuration-drift call in `persisted` before `PersistentWorld::resume` (`step-18-interaction-list.md` §11, F-IA-8: `main.rs:424`). | Same: the call lands in `serve.rs::persisted` if S11-B is first; S11-B carries it if IL-a is first. Different lines of one function; mechanical. |
| **test hygiene** (`mvp0/pr-test-hygiene`, frozen) | A `mineworld-test-support` crate (`scratch!(name)`, `Scratch`), keeping `SaveDir::new(name)` / `.path() -> &str` source-compatible in `tools/cli/tests/support`. | S11-B's new tests use `SaveDir::new` (or `scratch!` if the crate has landed by B-C6), with names unique per test; nothing writes outside its scratch. |
| **S12 13a** (`clients/2d`, unmerged) | One `connect_to_world` call and one launcher. | The new optional fifth argument (`take_over := false`) keeps every four-argument call valid; launchers need no change. |
| **S15 12e / S14** | `PROTOCOL.md` §6.2, `slice_link.gd`. | S11-B edits neither §6.2 nor `slice_link.gd`. |

## 16.2 Source audit (`main @ f842c52`)

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| `server/src/runtime.rs` (441) `join` l. 255 | Pushes a new `Subscriber` for every join on a roster seat: **two connections may hold one seat** (A-1 still true). `Command::Leave(SubscriptionId)` removes a subscriber; a dead channel is reaped by `sweep`. | Joins and departures go through a `SeatTable`; a departure says whether the client left or dropped. |
| `runtime.rs` `HostClock` l. 64–81 | `now = epoch + elapsed wall seconds`: one simulated second per wall second, no scale. | `HostClock` gains `scale` (integer ≥ 1). |
| `runtime.rs` `submit` l. 283–319 | The one authority path: actor check, `ActionIds::allocate`, instant from `clock.now()`, advance, dispatch, `remember`, sweep. | Becomes `submit_at(observer, request, at)`; sessions pass `clock.now()`, hosted consults pass their lattice instant (I-8: one function). |
| `runtime.rs` `tick` l. 351, `sweep` l. 382 | `Command::Sweep` every `observation_interval` (100 ms) from a tokio ticker (`host.rs` l. 392–402); the tick advances then sweeps. | Hosted consults and hold expiry run in `tick`, before the sweep. |
| `server/src/host.rs` (498) | `Command`, `Seated`, `Submitted`, `Perceived`, `HostedWorld` builder (`seating`, `perceiving`), `WorldHost::{join, submit, leave}`. **At the 500-line trigger.** | Split before growth (B-C2): the handles (`Seated`, `Submitted`, `Perceived`, `SubscriptionId`) move to `host/handles.rs`. |
| `server/src/session.rs` (369) | Handshake per §4.1; `resume` non-null → `invalid_resume`; `took_over: None`, `resume: None`, `hold_seconds: 0` fixed; `host.leave(subscription)` on every ending. | Join carries `take_over` and `resume` to the world thread; the welcome carries what the table answered; a `released` channel tells a session it was taken over or superseded. |
| `server/src/protocol.rs` `ClientFrame::Join` | `protocol, invite, nickname, seat, resume`; `deny_unknown_fields`. | Gains `take_over: bool` (default `false`). |
| `protocol/connection.rs` `TookOver { None, Hosted, Held }`, `ClosingReason` | No value for "another connection held it" and no reason for "taken over". | Additions specified in §16.3 SD-B4 (QS11B-2). |
| `server/src/admission.rs` | `InviteToken::generate` (getrandom), `ct_eq` (subtle) — DEP-14 already names resume secrets. | `ResumeSecret` beside them; no new dependency. |
| `tools/cli/src/agent.rs` (82) | `drive`: a tokio task occupying a seat through `host.join` and running `RuleController::new()` on every observation it is sent; prints each outcome. | Replaced by a hosted adapter on the world thread (`tools/cli/src/hosted.rs`); deleted. |
| `tools/cli/src/main.rs` (523) `serve` l. ~320–420, `persisted` | Over the trigger (D-SA10). | Moved to `tools/cli/src/serve.rs` (B-C2). |
| `tools/cli/src/run.rs` l. 38–137 | `PACE` 900 s; consult lattice `genesis + k + m·PACE`, seat `k` in roster order; `PacedRuleController::new(seed, PACE)`; one `decide(&self, &Observation)` per consult. | The hosted paced adapter uses the same formula with `--pace`; `run.rs` is not edited (I-6). |
| `cognition/rule-controller/src/lib.rs` `RuleController` l. 83–128 | `answered: BTreeMap<EntityId, Heard>`; `decide` answers the newest unanswered line per speaker. `Heard` carries `at` (`systems/conversation/src/component.rs` l. 27–31). | `RuleController::since(at)`: a line heard at or before `at` counts as answered (F-13). `new()` unchanged. |
| `tools/cli/tests/*` | No test joins a seat an `--agent` drives, and no two connections join one seat (grep of every `join(`: ac13/ac15 join `visitor`/`wanderer`; milestone_b/c join through `support`, one connection per seat). | Exclusivity breaks no existing test (R-S11-6 audited). |
| `persistence` | `SqliteBackend::{create, open}(…, Durability::PowerLoss)` in `persisted`: one fsync per journaled input. | `--town --save` journals every NPC request; CP-B4 measures it (R-S11-5). |

## 16.3 Design decisions (SD-B1 … SD-B14)

| ID | Decision | Why |
| --- | --- | --- |
| **SD-B1** | **`server/src/seats.rs`: `SeatTable`**, owned by `WorldRuntime`, the only writer of bindings. One state per roster seat: `Free`; `Hosted(HostedSlot)`; `Connected { subscription, session, resume }`; `Held { until: Instant, resume, observer }`. Pure transitions over an injected wall-clock `Instant`, so they are unit-testable with no thread. Every transition is host state: no journal entry, no fact, no revision (I-2, ARC-40). | §4.2. One writer on the thread that owns the world (A-9). |
| **SD-B2** | **The join rules**, in this order, after S11-A's admission: (1) `resume` given → it must match the seat's `Held` secret (constant time) or the seat's live `Connected` secret (a half-open old socket): granted `took_over: "held"`, the old connection, if any, closed `superseded`; otherwise `invalid_resume`. (2) `Free` → granted, `"none"`. (3) `Hosted` → granted without any flag, `"hosted"`: the in-server controller yields to a person (§4.4, CP-B1). (4) `Connected` or `Held` by somebody else → `seat_occupied`, unless `take_over: true`: then granted, `"connection"`, and the previous connection, if live, is closed `taken_over`; a held seat's secret dies. | §4.2–4.4; ruling 1's explicit flag for taking a seat from another *connection* (S10's R-S11-4: a human displaces an LM session). An in-server controller is not a connection and needs no flag. |
| **SD-B3** | **Departures**: `leave` frame, or the session ending on `closing` it sent → the seat returns to its default at once. Socket gone without `leave` → `Held { until: now + hold, resume }`; nobody drives the Person during the hold, not even its hosted controller. Hold expiry (checked each tick) → default. `default(seat)` = `Hosted` with a controller **rebuilt** from its factory at that instant, or `Free`. | §4.3, §4.4: a blinking Wi-Fi does not hand the Person to the town; a released Person is driven by a fresh controller, never one resumed from memory (§4.6). |
| **SD-B4** | **Protocol additions inside revision 2** (owned by S11-B per the landing table; `PROTOCOL.md` updated in B-C1): `join.take_over` (bool, default `false`); `welcome.resume` a 32-hex secret, refreshed on every welcome; `welcome.hold_seconds` the server's hold; `welcome.took_over` gains **`"connection"`**; `closing.reason` gains **`"taken_over"`**; `seat_occupied` is sent; `WorldSummary.time_scale` (integer ≥ 1, in `welcome.world` and `/status`); `WorldSummary.clients` counts connections only. Golden frames updated (welcome, closing). | Ruling 1 (take_over, time scale "reported in welcome"); S10 R-S11-4 needs the evicted session told distinctly; `took_over` must not claim `"none"` when a connection was displaced. QS11B-2 asks the primary session to confirm the two new values. |
| **SD-B5** | **`server/src/hosted.rs`: the seam.** `pub trait HostedController: 'static { fn next_consult(&self, after: WorldTime) -> WorldTime; fn decide(&mut self, observation: &WireObservation) -> Option<ActionRequest>; fn answered(&mut self, _answer: &HostedAnswer) {} }` with `HostedAnswer { action_id, result } | Refused(RefusalCode)`. Registered per seat on `HostedWorld` as a factory: `HostedWorld::hosting(seat, impl Fn(WorldTime) -> Box<dyn HostedController>)` — called at server start and at every release, with the binding instant. The server names no controller crate (I-9). | §4.5, ARC-42. `answered` keeps today's per-outcome printing possible without the server printing for a controller. |
| **SD-B6** | **Consults on the world thread.** In `tick`, before the sweep: for every `Hosted` seat whose `next_consult ≤ now`, in instant order then seat order: advance the world to that instant, compute the seat's observation through the same `Perception::observe`, `decide`, and pass any request through `submit_at(observer, request, instant)` — the session's path (actor check, server `ActionId`, journal first). A seat consulted at most once per tick per due instant; consults due while a seat is `Connected`/`Held` are skipped, not queued. | §4.5; I-8; I-11 (bounded synchronous work; measured, CP-B4). |
| **SD-B7** | **CLI adapters** in `tools/cli/src/hosted.rs`: `ReactiveSeat` (`RuleController::since(bound_at)`, `next_consult = after + 1 s`) for `--agent SEAT`; `PacedSeat` (`PacedRuleController::new(seed, pace)`, `next_consult` = the next `genesis + k + m·pace` after `after`, seat `k` in roster order — `run`'s formula with `--pace`) for every other seat under `--town`. `agent.rs` is deleted. | §4.5, §4.6. `run.rs` untouched (I-6). |
| **SD-B8** | **`RuleController::since(at: WorldTime)`** in `cognition/rule-controller`: a line whose `Heard.at ≤ at` is treated as answered. `new()` keeps answering every line. | F-13, §4.6, ruling 3. |
| **SD-B9** | **CLI flags** (§11.4's frozen contract plus the ruling's time scale): `--town`, `--seed N` (default 0), `--pace SECONDS` (world seconds, default 5), `--hold SECONDS` (wall seconds, default 30, `0` = no hold), `--time-scale N` (world seconds per wall second, integer ≥ 1, default 1). `--agent` seats are reactive; `--town` drives every other seat. | §11.4, QS11-3, QS11-4, QS11-5; S12 R-S11-6 (`--time-scale`). |
| **SD-B10** | **`tools/cli/src/serve.rs`**: `serve`, `persisted` and the hosted-controller wiring move out of `main.rs` behind `ServeRequest { world, listen, invite, agents, town, seed, pace, hold, time_scale, save }` (E-b adds `roots`). `main.rs` keeps the `clap` declaration and one call. | D-SA10 (accepted); file sizes (SB-12). |
| **SD-B11** | **Operator's statistics on shutdown.** On a graceful stop (Ctrl-C / SIGINT) the server prints `[world] ticks N, longest tick M ms` and one line per hosted seat `[world] hosted <seat>: C consults, A accepted (by action type, e.g. move 7,
talk 2), R rejected, F refused` — counted by the adapter through `answered`, printed by the composition
root, so the server still names no action type. Host state, not world state; nothing secret. CP-B4 reads it. | CP-B4's "test-only probe", made an operator-visible line rather than a hidden hook (no test-only code path in the product). |
| **SD-B12** | **`ResumeSecret`** in `admission.rs`: 16 bytes from `getrandom`, 32 lowercase hex, `Debug` redacted, compared with `subtle` (DEP-14 already covers resume secrets). It is sent only in the holder's own `welcome`. | §4.3, I-5. |
| **SD-B13** | **Session plumbing**: `Command::Join` carries `JoinRequest { seat, take_over, resume: Option<ResumeSecret>, session }`; `Seated` carries `took_over`, `resume`, `hold_seconds` and a `released: oneshot::Receiver<ClosingReason>` the session selects on (taken over → `closing{taken_over}`, superseded → `closing{superseded}`); `WorldHost::leave(subscription, Departure::{Left, Dropped})`. | One writer (the table); the session holds no binding state. |
| **SD-B14** | **The Godot module**: `connect_to_world(address, seat, invite, nickname, take_over := false)`; `var reconnect := false` (opt-in). When `reconnect` is on and the connection drops without a `closing`, the module rejoins with the stored `resume` at 1 s, 2 s, 4 s … while within `hold_seconds`, then once without it; it emits `reconnecting(attempt)` and the usual `welcomed`. Every four-argument call stays valid. A new headless check `clients/protocol/checks/reconnect_check.gd` and `run.sh reconnect`. | §4.3's module policy; R-9's far side; S12 13a unaffected. |

**Size plan (SB-12).** `runtime.rs` (441) gains the seat and consult calls; its clock, `ActionIds` and the
`Hosted` dispatch wrapper move to `server/src/runtime/world.rs` in B-C2's pure move. `host.rs` (498)
splits as SD-B5/16.2 say. `session.rs` (369) gains about 40 lines. `main.rs` drops to about 350.

## 16.4 Acceptance (decided before measuring, `ARC-23`)

Bounds are literals from §§4.2–4.6, 9.2 and 11.4. Each guard names its mutation (planted on the working
tree, seen red, reverted, recorded in §16.10).

```text
SB-1  Control is host state (I-2, ARC-40). `mineworld server worlds/social-cafe --save DIR --agent alice
      --hold 3`, inside the routine-free first minutes, nobody speaking: GET /status's revision is the
      same before and after each of: a client joining `alice` (welcome took_over "hosted"); it leaving
      (alice back to her controller); a client joining `visitor`, its socket killed (hold), the client
      rejoining with its resume (took_over "held"); a second connection taking `visitor` with
      take_over: true (the first receives closing {taken_over}); that one leaving; the 3 s hold of a
      dropped seat expiring. After SIGKILL, `mineworld inspect DIR` facts = `validate`'s genesis count.
      [tools/cli/tests/ac5_takeover.rs]
      Mutation M-SB1: a binding change that dispatches (`join` submits a no-op `move` as the seat) →
      the revision moves; the test fails.

SB-2  One controller per seat (I-3). Server socket test: three connections race for one free seat —
      two plain joins and one join with a forged resume — exactly one welcome; the others are refused
      seat_occupied / invalid_resume. A fourth connection without take_over on a connected seat is
      seat_occupied, and with take_over: true is welcomed "connection" while the holder receives
      closing {taken_over} and is closed. [server/tests/seats.rs]
      Mutation M-SB2: SeatTable grants a plain join on a Connected seat → two welcomes; fails.

SB-3  Hold and resume (AC-3, CP-B2). `mineworld server worlds/market-town --town --save DIR --hold 10`:
      a client seated as `visitor` is killed without `leave`. During the hold, a second client's
      observations show visitor's location unchanged while /status's revision advances (the town acts
      while nobody plays visitor). A reconnect with the resume inside the hold is welcomed
      took_over "held", the same observer, and an observation whose `at` is later than the last one
      before the drop. A drop then a plain join after the hold expires is welcomed took_over "hosted".
      A resume presented after expiry is invalid_resume. [tools/cli/tests/ac3_reconnect.rs]
      Mutation M-SB3: return a dropped seat to its default at once (no hold) → the reconnect is
      welcomed "hosted", not "held"; fails.

SB-4  Takeover with the Person intact (AC-5, CP-B1). `mineworld server worlds/market-town --save DIR
      --agent alice` (alice hosted; her colleagues idle): before, the save's biography of alice
      (`mineworld biography --json`, read with the server stopped) and her `EntityId` from `validate`.
      Restarted on the same save, a client joins `alice` (took_over "hosted", observer = that id),
      talks to bob, submits an offered complete `buy` and leaves; the server is stopped. After: the
      biography before is a prefix of the biography after, and every added entry's cause is an
      ActionId this client was answered with; alice's own disclosed components in the client's first
      and last observations differ only in wallet (−price), holdings (+1) and conversation history.
      [tools/cli/tests/ac5_takeover.rs]
      Mutation M-SB4: rebuild the hosted controller on takeover instead of unbinding it (it keeps
      acting) → alice's controller answers bob's reply while the human holds her; an unexplained
      biography entry appears; fails.

SB-5  F-13 (CP-B3). `mineworld server worlds/social-cafe --agent alice --save DIR`: a client talks to
      alice and receives her answer; SIGKILL; restart on the save; the client rejoins and is silent for
      15 wall seconds: no `spoke` by alice addressed to the client is recorded after the restart
      (`mineworld inspect` / the client's disclosed history). [tools/cli/tests/restart.rs, extended]
      Mutation M-SB5: the reactive adapter binds `RuleController::new()` instead of `since` → alice
      re-answers after the restart; fails.

SB-6  The town lives (CP-B4). `mineworld server worlds/market-town --town --pace 5 --save DIR`, two
      sessions connected, 120 wall seconds, then SIGINT: the shutdown lines show every hosted seat with
      ≥ 3 accepted `move` requests, zero faults (/status), and ~~`longest tick ≤ 50 ms`~~ — half the
      100 ms cadence. [tools/cli/tests/hosted_town.rs]
      SUPERSEDED 2026-10-08 (operator ruling on D-SB12, option (b), on E-SB9's evidence): the bound
      is now **the p99 tick over the run ≤ 50 ms, with the maximum reported beside it**; the maximum
      no longer gates. The shutdown line is `[world] ticks N, p50 A ms, p99 B ms, longest tick M ms`
      (a fixed 0.1 ms histogram in runtime/world.rs `TickTimes`); the test prints p50, p99 and max on
      every run. The other checks (accepted moves per seat, faults, clients) are unchanged.
      Mutation M-SB6: consults never fire (next_consult always in the future) → accepted 0; fails.
      Mutation M-SB12: a 60 ms delay injected into every tick → p99 68 ms > 50 ms; fails.

SB-7  A hosted controller has no privilege (I-8). Server test: a test HostedController that returns a
      request whose actor is another Person is answered Refused(actor_not_observer) through
      `answered`, nothing is dispatched, and a request it makes as its own Person receives a
      server-allocated ActionId in sequence with the sessions'. [server/tests/seats.rs]
      Mutation M-SB7: hosted requests bypass `submit_at`'s actor check → dispatched; fails.

SB-8  Time scale. `--time-scale 60`: welcome.world.time_scale and /status's time_scale are 60, and
      two /status answers 2 wall seconds apart differ in `at` by 100 … 140 world seconds; without the
      flag, time_scale is 1. [tools/cli/tests/server_command.rs]
      Mutation M-SB8: HostClock ignores the scale → the `at` difference is ≈ 2; fails.

SB-9  Clients and secrets. /status's `clients` counts connections only (a server with --town and one
      connection reports 1). The resume secret is in no stdout/stderr byte and no save byte (I-5;
      SA-3's method). [tools/cli/tests/ac3_reconnect.rs]

SB-10 The far side (R-9). `clients/protocol/run.sh reconnect`: a headless Godot check joins `visitor`
      on a `--town --hold 10` market-town, drops its socket without `leave`, and with `reconnect = true`
      is welcomed again with took_over "held" and the same observer; `run.sh evidence`, `run.sh
      affordances` and `./mineworld-slice --world --link` still pass. [clients/protocol/checks/]

SB-11 Nothing else moved. Every existing test passes; existing tests edited only where a hosted
      `--agent` changes what they observe (listed in §16.11 if any); 300-day seed-7 digests of both
      towns equal those at the base (I-6; `run.rs` untouched); cognition's paced tests unchanged.

SB-12 Scope and size. No diff under kernel/, contracts/, persistence/, worlds/, worldpack/, systems/,
      authoring/, sdk/, tests/acceptance/; no new dependency; the server names no controller crate.
      runtime.rs, host.rs, session.rs, protocol.rs and tools/cli/src/main.rs each under 500 lines.
```

## 16.5 Change set

```text
server/src/{seats.rs (new), hosted.rs (new), runtime.rs, runtime/world.rs (new, moved), host.rs,
            host/handles.rs (new, moved), session.rs, protocol.rs, protocol/connection.rs,
            protocol/summary.rs, admission.rs, app.rs (only if a signature moves), lib.rs}
server/{PROTOCOL.md, README.md}
server/tests/{seats.rs (new), handshake.rs, frames.rs, frames/{welcome,closing,join}.json,
              support/mod.rs, two_clients.rs (only if the compiler requires)}
cognition/rule-controller/src/{lib.rs, tests.rs}
tools/cli/src/{main.rs, serve.rs (new, moved), hosted.rs (new), agent.rs (deleted)}
tools/cli/tests/{support/mod.rs, ac3_reconnect.rs (new), ac5_takeover.rs (new), hosted_town.rs (new),
                 restart.rs, server_command.rs}
clients/protocol/{mineworld/world_client.gd, checks/reconnect_check.gd (new), run.sh, ADOPTION.md,
                  README.md, evidence/* (regenerated)}
docs/{DECISIONS.md (ARC-40, ARC-42), MODULE_SPEC.md §8.1}
.structured-coding/plans/mvp0/{step-12-server.md §§15 (merge record), 16; handoff-s11b.md}
```

## 16.6 Commit plan

### B-C0 — Design (this section) and S11-A's merge record — docs only

- [x] Implementation: §15's lifecycle records the merge; §16 from the audit in §16.2.
- [x] Validation: Markdown only.
- [x] Review: every lane in §16.1's table checked against its branch (`git diff origin/main...origin/<lane>`);
  every guard in §16.4 names a mutation. Self-review only; the freeze is pending.

### B-C1 — Specs before code: `PROTOCOL.md`, ARC-40, ARC-42, MODULE_SPEC §8.1

**Scope.** `server/PROTOCOL.md`: the landing table's S11-B rows become landed; §2's `join` row gains
`take_over`; §4.1's checks gain the seat rules of SD-B2; §5.1 (`resume`, `hold_seconds`, `took_over` with
`"connection"`); §5.6 (`taken_over`); §5.7 (`time_scale`, `clients`). `docs/DECISIONS.md`: **ARC-40**
(control is host state: bindings, sessions, holds, nicknames are never journaled; a binding change moves
no revision; AC-5 is measured on that) and **ARC-42** (hosted controllers on the world thread behind a
bounded synchronous seam; asynchronous controllers connect as sessions; the comparison of §7.5).
`MODULE_SPEC.md` §8.1: the `server` line and row gain `--town`, `--seed`, `--pace`, `--hold`,
`--time-scale`.

- [x] Implementation · [x] Validation: `check_decision_ids`, `check_doc_headings` (E-SB1) · [x] Review: §6
  and §6.2 of `PROTOCOL.md` untouched (12e's surface); every new value appears in the landing table
  (E-SB1). `PROTOCOL.md` gains §4.2 for the seat rules rather than growing §4.1's table (D-SB1).

### B-C2 — Pure moves: `host/handles.rs`, `runtime/world.rs`, `tools/cli/src/serve.rs`

**Scope.** Move, re-export, no behaviour change; `serve`/`persisted` move behind `ServeRequest` with
today's fields only. **Validation:** `cargo test -p mineworld-server -p mineworld-cli` — the same tests,
all passing; clippy. **Review:** public paths unchanged; `main.rs` < 500.

- [x] Implementation · [x] Validation · [x] Review (E-SB2, commit 8882798)

### B-C3 — `SeatTable` (`seats.rs`) and `ResumeSecret`

**Scope.** SD-B1–SD-B3 and SD-B12 as pure code over an injected `Instant`. **Validation (unit):** every
transition of §4.2 and its refusals; `take_over` on `Connected` and `Held`; resume on `Held` and on a live
`Connected` (supersede); expiry at `until` and not before; `default` rebuilds through the factory;
`ResumeSecret` format and redacted `Debug`. **Review:** no world access in `seats.rs`; one writer.

- [x] Implementation · [x] Validation · [x] Review (E-SB3, commit 89069b2, with B-C4 — D-SB2)

### B-C4 — The `HostedController` seam and `RuleController::since`

**Scope.** `hosted.rs` (trait, `HostedAnswer`, the factory type, the due-consult order) and
`cognition/rule-controller` `since`. **Validation (unit):** consult order across seats and instants; a
`since(t)` controller does not answer a line heard at `t` and answers one at `t + 1`; `new()` unchanged
(existing tests untouched and passing); paced tests untouched.

- [x] Implementation · [x] Validation · [x] Review (E-SB3, commit 89069b2 — D-SB2)

### B-C5 — The runtime and the session: seats, holds, consults, time scale, revision 2's additions

**Scope.** `runtime.rs` (`submit_at`, join/departure through the table, consults and expiry in `tick`,
`time_scale`, `clients`), `host.rs`/`handles.rs` (`JoinRequest`, `Departure`, `Seated` fields,
`released`), `session.rs` (take_over, resume, the released branch, `Departure`), `protocol*` (SD-B4),
golden frames, `server/tests/seats.rs` (SB-2, SB-7), `handshake.rs` updated where S11-A asserted
`invalid_resume` for every resume. **Validation:** server suites; M-SB2, M-SB7. **Review:** I-11 (no wait
on the world thread; consults bounded); the session holds no binding state.

- [x] Implementation · [x] Validation · [x] Review (E-SB4; D-SB3, D-SB4)

### B-C6 — The CLI: hosted adapters, flags, statistics; real-binary acceptance

**Scope.** `tools/cli/src/hosted.rs`, `serve.rs` wiring, `main.rs` flags, `agent.rs` deleted; tests
`ac5_takeover.rs` (SB-1, SB-4), `ac3_reconnect.rs` (SB-3, SB-9), `hosted_town.rs` (SB-6), `restart.rs`
(SB-5), `server_command.rs` (SB-8). **Validation:** those tests and every suite that starts the binary
(SB-11's first half); M-SB1, M-SB3, M-SB4, M-SB5, M-SB6, M-SB8. **Review:** the server names no controller
crate; statistics lines carry no secret.

- [x] Implementation · [x] Validation · [x] Review (E-SB5; D-SB6 … D-SB10; full `cargo test -p
  mineworld-cli --no-fail-fast` after the QTW-13 change: every target green but ac15's second
  `clients` literal, fixed (D-SB9) and re-run 6/6)

### B-C7 — The Godot module: `take_over`, opt-in reconnect; the far side

**Scope.** SD-B14; `checks/reconnect_check.gd`; `run.sh reconnect` (port 0, own PID only, invite line kept
out of evidence); `ADOPTION.md` §2/§6; evidence regenerated. **Validation (one Godot window at a time):**
SB-10. **Review:** the module never logs the resume secret; every four-argument call unchanged.

- [x] Implementation · [x] Validation · [x] Review (E-SB6)

### B-C8 — Close: README, digests, scope, full gate, ledger, PR

- [x] Implementation: `server/README.md` (`--town`, takeover); ledger · [x] Validation: SB-11 digests,
  SB-12 scope and sizes, one full gate on the final executable head (background) — E-SB7, E-SB8: all
  green but one CP-B4 measurement under load (D-SB12, material) · [x] Review: SB-1 … SB-12 with
  evidence; the PR marked READY FOR OPERATOR REVIEW — DO NOT MERGE, with D-SB12 flagged.

**E-SB0 (first action after the freeze, before any code):** the base's two 300-day digests (expected
`ad49c723…c64b` and `365b50e0…1d1d`), and a timing of `cargo test -p mineworld-cli` at the base.

## 16.7 Test ownership

```text
STATIC      fmt; clippy -D warnings (exhaustive matches over the seat states and the new enum values)
UNIT        SeatTable transitions and refusals; ResumeSecret; consult order; RuleController::since
INTEGRATION server/tests/seats.rs over real sockets: exclusivity race, take_over, supersede, released
            sessions, hosted authority (SB-2, SB-7); handshake and golden frames updated
REAL BINARY ac5_takeover (SB-1, SB-4), ac3_reconnect (SB-3, SB-9), restart (SB-5), hosted_town (SB-6),
            server_command (SB-8); every existing CLI acceptance test unchanged
FAR SIDE    Godot reconnect check; run.sh evidence/affordances; mineworld-slice --world --link (SB-10)
REAL RUN    both 300-day digests (SB-11)
GATE 1      NOT REQUIRED — no language model
CI          S13's workflow if merged by the final head (its run on the exact head is canonical);
            otherwise one local full gate
```

## 16.8 Is any of this material?

No kernel, contract or persistence change; no new dependency. Points for the primary session's freeze:

1. **QS11B-1 — Hosted seats need no flag.** A join on a seat an in-server controller drives takes it over
   without `take_over` (§4.4, CP-B1's frozen text); the flag is required only to displace another
   *connection* or a held seat (ruling 1, S10 R-S11-4). Recommended as written.
2. **QS11B-2 — Two values added inside revision 2:** `took_over: "connection"` and `closing.reason:
   "taken_over"`. Both are S11-B's to specify under the landing table; they let a displaced client (an
   LM session) be told distinctly and stop `took_over` from claiming `"none"`. Recommended.
3. **QS11B-3 — Who may take over.** Any holder of the invite may take any seat with `take_over: true`,
   including a dropped player's held seat. MVP-0 has one trust level (`NETWORKING.md` §9); finer policy
   (e.g. an operator-only takeover of a held seat) would need accounts. Recommended: accept for MVP-0 and
   record it in ARC-40's limitations. **Operator-visible**, so flagged.
4. **QS11B-4 — Hold time base.** `--hold` is wall seconds and `--pace` world seconds; with
   `--time-scale 60` a 30 s hold is 30 minutes of world time. Recommended (a hold is about a network,
   a pace about a life).
5. **QS11B-5 — Shutdown statistics lines** (SD-B11) as CP-B4's probe instead of a hidden test hook.
6. **QS11B-6 — Merge order with E-b and IL-a** (§16.1): whichever lands second carries the other's
   `serve`/`persisted` lines; no order is required.

**Material stops:** any edit under `kernel/`, `contracts/`, `persistence/`; a path outside §16.5; an
existing test failing for a reason other than a hosted `--agent` changing what it observes (each such
edit listed and justified, or the stop); a digest change; CP-B4's 50 ms bound failing (the remedy —
pace or durability — is the operator's, R-S11-5).

## 16.9 Execution contract (proposed; confirmed only by the primary session's freeze)

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-B — seats, hold and resume, takeover, hosted
                    controllers, F-13
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §16; evidence §16.10; deviations §16.11
RELATED / BINDING   this file §§4.2–4.6, 5, 6, 7.4, 7.5, 8, 9.2, 11, 15; server/PROTOCOL.md revision 2;
                    overall.md "Parallel build-out, 2026-10-08"; docs/DECISIONS.md DEP-14, ARC-41,
                    ARC-23, ARC-25, ARC-27; NETWORKING.md; CLAUDE.md §§2–4
IMPLEMENTATION BASE main @ f842c52, merged forward at each rebase; branch mvp0/pr-s11b-seats; worktree
                    /Users/yuema137/mineworld-worktrees/impl-s11a, held by this session only
APPROVED SCOPE      §16.5; B-C1 … B-C8; SD-B1 … SD-B14 as answered by QS11B-1 … QS11B-6
FROZEN INVARIANTS   I-1 (no kernel/contracts/persistence diff); I-2/ARC-40 (a binding change moves no
                    revision); I-3 (one controller per seat); I-5 (resume secrets and invites in no
                    save, observation, fact, /status or log); I-6 (digests = E-SB0; run.rs untouched);
                    I-8 (hosted requests take submit_at); I-9 (no controller crate in server/);
                    I-11 (nothing waits on the world thread); 16b's GDScript names unchanged;
                    four-argument connect_to_world calls valid
SEQUENCE            E-SB0 → B-C1 → B-C2 → B-C3 → B-C4 → B-C5 → B-C6 → (merge origin/main) → B-C7 → B-C8
VALIDATION BUDGET   unit/integration/static unrestricted; real-binary tests as listed (hosted_town ~2.5
                    min, background); 300-day runs at most six; Godot runs one window at a time, each
                    mode at most three times; one full gate on the final head (background, ~7 min);
                    about 1.5 hours in total; real-model: NOT REQUIRED
LIVE DOCUMENTATION  §16 checkboxes; §16.10; §16.11
HANDOFF             .structured-coding/plans/mvp0/handoff-s11b.md
ENDPOINT AUTHORITY
  implementation + local validation   unresolved until the primary session's freeze message
  semantic commits, branch push       authorized after the freeze (the brief: same rules as S11-A)
  PR creation / update                authorized (same)
  CI repair                           authorized if a workflow exists on the final head
  merge                               operator only; never inherited
POST-MERGE SYNC     this session owns §16; the primary session owns §§1–14, overall.md, MVP_STATUS
NORMAL STOP         PR S11-B READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       §16.8's list
```

## 16.10 Evidence ledger

```text
E-SB0 2026-10-08, base main @ f842c52 + design commit 1a46de6 (no code change), worktree impl-s11b.
      Debug build (`cargo build -p mineworld-cli`), binary copied to /tmp/s11b/mineworld-base.
      `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → 339 lines, sha-256 of every
        line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b (= E-SA0).
      `mineworld run worlds/market-town --headless --seed 7 --days 300` → 355 lines, sha-256 =
        365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d (= E-SA0).
      Filter: `grep -v '^wall'` then `shasum -a 256`. Outputs /tmp/s11b/base-{cafe,market}.out.
E-SB1 B-C1. check_decision_ids → 57 ids, all distinct (ARC-40, ARC-42 new; ARC-40 placed before
      ARC-41 and ARC-42 after DEP-14, inside S11's region, so lanes appending at the file's end do not
      collide). check_doc_headings → 191 numbered sections across 26 documents, none duplicated.
      PROTOCOL.md: hunks only in the header, §§2, 3, 4.1, 4.2 (new), 5.1, 5.5, 5.6, 5.7, 8, 10 —
      none in §6 or §6.2 (12e's surface). New values each in the landing table: join.take_over,
      took_over "connection", closing.taken_over, WorldSummary.time_scale, clients without hosted
      controllers. ARC-40 records QS11B-3 (one trust level) and QS11B-4 (hold in wall seconds) as
      MVP-0 limitations.
      E-SB0 (continued): `cargo test -p mineworld-cli --no-fail-fast` at the base: exit 0, 55 passed,
      0 failed, 446 s wall.
E-SB2 B-C2 (pure moves). host.rs 498 → 387 (handles.rs new), runtime.rs 441 → 344 (runtime/world.rs
      new), tools/cli/src/main.rs 523 → 382 (serve.rs new). clippy -D warnings clean on server + cli;
      `cargo test -p mineworld-server`: unit 26, frames 8, handshake 7, headless 4, two_clients 9,
      doc 1 — the base's tests and counts (E-SA4); `--test server_command --test restart` 7 + 2 pass.
      lib.rs re-exports unchanged (handles re-exported from host).
E-SB3 B-C3 + B-C4 (one commit, D-SB2). seats.rs + seats/tests.rs (10 unit tests: free/occupied/
      take_over, hosted yields without a flag and is unbound, leave rebuilds through the factory at
      the release instant, hold expiry at `until` and not 1 ms before, resume on held / on a live
      connection (superseded) / wrong secret / wrong seat / secret replaced on every welcome,
      take_over kills a held secret, hold 0, due order by instant then seat, unknown seat);
      admission.rs ResumeSecret/OfferedResume (+1 test: format, freshness, exact match, redacted
      Debug). `cargo test -p mineworld-server --lib` 37 passed. rule-controller: `since` + 1 test
      (a line at the binding instant is not answered, one at +1 s is; new() unchanged); 35 passed,
      every existing test untouched. clippy on this commit alone reports only dead code (the seat
      table and seam are wired by B-C5); the workspace is clippy-clean from B-C5 on.
E-SB4 B-C5. Resumed session (D-SB4): the 13 uncommitted files were audited hunk by hunk and kept.
      `cargo test -p mineworld-server`: unit 37, frames 8, handshake 7, headless 4, seats 4 (new),
      two_clients 9, doc 1 — all pass. seats.rs: three-way race on a free seat → exactly one welcome,
      the forged resume invalid_resume, the losing plain join seat_occupied; a plain join on a
      connected seat seat_occupied, then take_over → welcome "connection" (same observer) and the
      holder closing{taken_over}; a resume on a live connection → "held", the old one
      closing{superseded}; a test HostedController (written against the public seam) asking as
      carol → Refused(actor_not_observer), no fact perceived, as alice → Accepted with an ActionId
      consecutive with bob's session's ids. clippy -D warnings clean (server + cli, all targets).
      Mutations: M-SB2 (SeatTable grants a plain join on Connected) → race test "exactly one welcome:
      left 2" and the occupied test red; reverted. M-SB7 (consult submits as request.actor()) →
      "acting as somebody else … left: Answered(… Accepted …)"; reverted. Both re-green.
      Sizes: runtime.rs 492, host.rs 432, session.rs 399, protocol.rs 358.
E-SB5 B-C6 (origin/main e98321a — test hygiene #77 — merged first; new tests use SaveDir, i.e.
      `scratch!`). tools/cli/src/hosted.rs (ReactiveSeat, PacedSeat, statistics), serve.rs wiring,
      main.rs flags, agent.rs deleted. Smoke (market-town --town, port 0, SIGINT): join line read,
      `[world] ticks 19, longest tick 0 ms` and eleven `[world] hosted …` lines.
      SB-1 ac5_takeover binding_changes_… PASS: revision 1 through join alice (hosted), leave, rejoin
      (hosted), drop, resume (held), take_over (connection; holder closing{taken_over}), leave, drop +
      3 s hold expiry (free again); inspect `facts      53` = validate's 53 genesis facts.
      M-SB1 (join submits a talk as the seat) → "taking alice from her controller moved the revision:
      left 2, right 1"; reverted.
      SB-3/SB-9 ac3_reconnect PASS (13 s): hosted → drop → revision advanced during the hold → resume
      within 10 s → "held", same observer, later `at`, same self_location; second drop + 11 s →
      resume invalid_resume, plain join "hosted"; /status clients 1 with --town; three secrets in no
      stdout/stderr line and no save byte. M-SB3 (no hold) → the inside-hold resume refused
      invalid_resume (the seat was already back with the town); reverted.
      SB-4 ac5_takeover a_player_takes_alice… PASS (D-SB7): biography before a prefix of after, every
      added event_id among the players' answered events, revision = head + 3, components changed
      {acquaintances, conversation-history, holdings, wallet}, wallet lower, holdings +1. M-SB4 (a
      hosted seat stays Hosted when a player joins) → revision 5 ≠ 4; reverted.
      SB-5 restart a_restarted_agent… PASS (16.6 s): revision 7 unchanged over 15 silent wall seconds
      after the restart, history unchanged. M-SB5 (ReactiveSeat binds RuleController::new()) →
      "left 8, right 7"; reverted.
      SB-6 hosted_town PASS (120.6 s, scale 1): `[world] ticks 1201, longest tick 13 ms`; accepted
      moves per hosted seat alice 17, bob 16, carol 10, dev 20, erin 20, felix 16, grace 16, hana 14,
      ivan 8; zero faults; clients 2. CP-B4's 50 ms bound holds with Durability::PowerLoss.
      M-SB6 (a slot's first consult a billion seconds away) → "alice made 0 accepted move(s)";
      reverted.
      SB-8 server_command time_scale… PASS: welcome and /status 60, Δat 100…140 over 2 wall s; plain 1.
      M-SB8 (HostClock ignores the scale) → "differ by 2 world seconds"; reverted.
      QTW-13 (D-SB6): hosted.rs unit tests 2 PASS; its mutation red; reverted.
      clippy -D warnings, workspace, all targets: clean. Sizes: main.rs 412, serve.rs 271,
      hosted.rs 258.
E-SB6 B-C7 (SB-10). world_client.gd: `connect_to_world(…, take_over := false)`, `var reconnect`,
      `signal reconnecting(attempt)`, `State.RECONNECTING` appended (no existing name changed),
      `took_over` "connection", resume never printed. Godot 4.7, one headless window at a time:
      `run.sh reconnect` → 6 PASS, 0 failure(s): hosted, hold 10, a resume given (not shown), one
      attempt announced [1], welcomed "held", same observer "17". `run.sh affordances` → PASS, 0
      failure(s). `run.sh evidence` → exit 0, Alice answers in every transcript, no SCRIPT ERROR;
      server logs lose the old per-outcome `agent:` lines and gain the `hold … time scale …` line
      (the agent is no longer a task printing outcomes; its counts are in the shutdown statistics).
      `./mineworld-slice --world --link` → "all link checks pass". The check drops the socket by
      closing the module's own WebSocketPeer — the event a dropped network produces; no test-only
      path in the module. Each server stopped by its own PID (run.sh's `kill "$server"`).
E-SB7 B-C8. Merged origin/main b97cb4a (S16 E-b #78 with `--packs`, S19 plan #81): conflicts in
      tools/cli/src/main.rs and MODULE_SPEC §8.1 resolved by carrying E-b's `PackRoots` into
      `ServeRequest.roots` and `WorldPack::read_with` in serve.rs (QS11B-6's rule); the merged
      main.rs is 486 lines. IL-a has not landed; its drift call will go into serve.rs::persisted.
      SB-11 digests on the merged head (debug build, `grep -v '^wall' | shasum -a 256`):
        social-cafe seed 7, 300 days: 339 lines, ad49c723…c64b = E-SB0
        market-town seed 7, 300 days: 355 lines, 365b50e0…1d1d = E-SB0
      run.rs untouched; cognition's paced tests untouched.
      SB-12: no diff under kernel/, contracts/, persistence/, worlds/, worldpack/, systems/,
      authoring/, sdk/, tests/acceptance/; no Cargo.toml/Cargo.lock change; server/ names no
      controller crate. Sizes: runtime.rs 492, host.rs 432, session.rs 399, protocol.rs 358,
      main.rs 486 — each under 500.
      check_decision_ids 60 distinct; check_doc_headings 191, none duplicated; check_scratch scan
      151 sources clean.
E-SB8 Full gate on be674d7 (the merge head; later commits are docs only): fmt --check ok, check ok,
      clippy -D warnings ok, `cargo test --workspace --no-fail-fast`: 164 test binaries ok, ONE
      failed — hosted_town: `[world] ticks 1202, longest tick 160 ms` > 50 ms. The machine's load
      average was ~21 during the gate (other worktrees' 300-day runs and Godot checks running
      concurrently). CP-B4 measurements so far: 13 ms (E-SB5, isolated), PASS inside the full CLI
      suite run, 160 ms (this gate, load ~21), 1 ms (re-run on the same head right after, load ~7).
      Classification: CP-B4 FAIL once under load; PASS ×3. Material per §16.8 — see D-SB12.
E-SB9 D-SB12 located before counted (primary-session ruling: no bound change, no option chosen;
      ARC-23). A scratch probe — a local uncommitted patch to server/src/runtime.rs, lib.rs,
      server/Cargo.toml (libc, for thread CPU time) and persistence/src/world.rs, reverted after
      (`git status` clean) — timed every tick's phases in wall and thread-CPU time: expire; per
      hosted consult its advance, observe (perception), decide and submit, the submit split into
      kernel dispatch, row encode, snapshot (every 64th revision) and the SQLite commit
      (Durability::PowerLoss); the tick's own advance; the sweep (perception + try_send fan-out).
      Any tick over 20 ms was logged with its breakdown, and every new maximum. The CP-B4 scenario
      (hosted_town: market-town --town --pace 5 --save, two sessions, 120 s, SIGINT) was run:
        run  conditions                                      p50      p99      max     >20 ms
        1    machine as found, load 8.1→7.5                  0.29 ms  1.94 ms  3.58 ms  0
        2    as found, load 7.5→2.5                          0.30 ms  2.20 ms  10.87 ms 0
        3    as found, load 2.5→5.4                          0.27 ms  2.11 ms  5.34 ms  0
        4    + `cargo test --workspace` in this worktree,    0.08 ms  0.89 ms  3.29 ms  0
             load 14.8 (the hog itself passed, exit 0)
        5    + 16 CPU-bound 300-day runs (no save), load 17.7 0.11 ms 1.21 ms  3.46 ms  0
        6    + 8 fsync-heavy hosted servers (--town --pace 1  0.08 ms 0.96 ms  13.55 ms 0
             --save, PowerLoss), load ~15
      (Runs 5 and 6 go beyond the ruling's list to provoke each hypothesis separately; every hog
      was this session's own PIDs, stopped afterwards.) Tick start-to-start: p50 100 ms, max
      119 ms in every run — the tick is never starved of its turn by more than ~20 ms.
      The slowest ticks and where their time went:
        run 4  3.3 ms wall / 0.6 ms CPU: consult.decide 2.6 ms wall, 0.1 ms CPU (descheduled)
        run 5  3.5 ms wall / 0.5 ms CPU: sqlite-commit 3.2 ms wall (I/O wait)
        run 6  13.5 ms wall / 0.2 ms CPU: consult.observe 8.4 ms wall, 0.0 ms CPU (descheduled)
               + sqlite-commit 5.1 ms wall (I/O wait)
      Steady state: a tick's own CPU is ≤ 0.6 ms; a commit is 0.1–0.4 ms; no snapshot was ever in a
      slowest tick. The 160 ms tick was not reproduced in six probed runs. Every outlier has wall ≫
      CPU: the world thread was off-CPU (descheduled, or waiting on the commit's I/O), never busy.
      Neither stall source reached 15 ms under the loads this session could make; the 160 ms one
      coincided with a load of ~21 from other worktrees' concurrent runs.
      GitHub runner (ubuntu-24.04, CI from #63): a scratch branch `scratch/s11b-ticks` (8ecee01 plus
      one commit that fails hosted_town on purpose to print its lines; run 37856777357, deleted
      after use) recorded `[world] ticks 1202, longest tick 8 ms`, with accepted moves alice 17,
      bob 16, carol 10, dev 20, erin 20, felix 16, grace 16, hana 14, ivan 8. On #83 itself, CI at
      8ecee01 (run 37856771059): fast success, test success — hosted_town passed on the runner.
E-SB10 CP-B4 as ruled (operator, 2026-10-08: option (b)). Merged origin/main 77a8717 first (IL-a #80,
      13a #82, #85): IL-a's configuration-drift check moved with `persisted` into serve.rs (same
      place: after `SqliteBackend::open`, before `compose`), `saved_genesis` shared from main.rs;
      ADOPTION §6's "reconnect arrives with S11-B" row dropped and §6.1 told of the opt-in; ARC-40
      and 13a's ARC-45 both kept. `TickTimes` keeps a 0.1 ms histogram (fixed 80 KB) and prints
      p50/p99/max; unit test `the_p99_ignores_one_stall_in_a_hundred_and_not_two` (99 × 1.05 ms + one
      160 ms → p99 1.1, max 160; a second stall → p99 160.1; past 1 s → the longest). hosted_town
      asserts p99 ≤ 50 ms and prints `CP-B4 tick p50 0.3 ms, p99 2.3 ms, max 5 ms` (local, PASS).
      M-SB12: `std::thread::sleep(60 ms)` injected at the end of every tick (the harness cannot slow
      the server's world thread, so the delay was planted in runtime.rs `tick`) → "the p99 tick took
      68 ms, over 50 ms (p50 64.9 ms, max 78 ms)"; reverted, `git diff` clean of it.
      ARC-42's measured paragraph and MODULE_SPEC §8.1's shutdown line updated to match.
      CI at 39d02ea: fast success, test FAILURE — IL-a's structural test tools/cli/tests/configure.rs
      looked for `fn persisted(` in main.rs, where it no longer is. One line now points it at
      serve.rs (D-SB14); the check itself (drift check before `PersistentWorld::resume`) holds there.
```

## 16.11 Deviations and discoveries

```text
D-SB1 (bounded) PROTOCOL.md layout. SD-B2's join rules are §4.2 (new), with §4.1's check table gaining
      "4 seat" and "5 control" in place of S11-A's "4 resume, 5 seat": the resume is now judged on the
      world thread with the seat, after the roster check, as SD-B2 orders. Section numbers 1–10 keep
      their meaning (D-SA2).
D-SB2 (bounded) Commit mapping. B-C3 (SeatTable) and B-C4 (the seam) land as one commit: a Hosted
      seat holds a bound HostedController, so the table cannot compile without the trait. Held
      carries no `observer` (SD-B1 listed one): the runtime resolves a seat's observer from the
      roster whenever it needs it, so the table stays free of world identities.
D-SB3 (bounded) The first binding instant (F-13). A world resumed with history binds its controllers
      at its own instant (its last input); a world holding only its genesis binds them one second
      before it, because the host clock stamps a line said in the first wall second with the genesis
      instant itself and `since(t)` treats `at ≤ t` as answered (runtime.rs `first_binding`).
D-SB4 (bounded) Resumed session. The previous implementation session was cut off by an API rate limit
      with 13 files uncommitted (B-C5's runtime, host, session, protocol and golden-frame hunks).
      Audited and kept, not discarded: coherent with SD-B1…SD-B13 and compiling. Finished by
      server/tests/seats.rs, a one-line edit to protocol/tests.rs (its "field a later PR defines"
      example was `take_over`, now defined; replaced by `scope`), and a one-argument edit to
      agent.rs (`Departure::Left`) so the workspace compiles until B-C6 deletes the file.
      `WorldHost::join(seat)` is kept as a plain join labelled session 0 for in-process callers.
D-SB5 (forward constraint, S19) Live time scale and pause. The operator's later requirements (4 h /
      2 h / 1 h per day, host-only, live; calendar time only; pause/resume) are not implemented here
      and not precluded: HostClock is `epoch + scale × elapsed wall time`, so a live change is a
      rebase (epoch := now, started := Instant::now(), scale := new) and a pause is a rebase with
      the clock frozen until resume; both stay on the world thread. Holds are wall time and
      consults skip (never queue) missed instants, so neither misbehaves across a rebase.
      `welcome.world.time_scale` is a snapshot; a live change would need an announcement frame
      (S19's to specify). Hosted consult cadence is wall time from the QTW-13 amendment (D-SB6); on
      a live change S19 must reschedule each slot's pending `next` (`now + w × s₁`, its §4.4) — the
      slot already re-asks `next_consult` after every consult, so a rescale needs one pass over the
      table and no seam change. Nothing in this PR scales movement or dialogue.
D-SB6 (ruling) QTW-13 applied (the dated amendment under §16's freeze record): `--pace` in wall
      seconds, the reactive second in wall time, both stated to the world-time seam × time_scale, in
      tools/cli/src/hosted.rs and serve.rs only. Files touched are inside §16.5 (hosted.rs, serve.rs,
      main.rs, docs/DECISIONS.md ARC-40, MODULE_SPEC.md §8.1); no invariant changes — I-6 holds
      because run.rs is untouched. Guard: hosted.rs `cadence_is_wall_time_whatever_the_scale`; its
      mutation (pace not multiplied by the scale) → "left [1120, 1125, 1130], right [1120, 1420,
      1720]"; reverted. Seat offsets are k wall seconds as well (k × scale world seconds), so seats
      stay spread across ticks at any scale.
D-SB7 (bounded) SB-4's evidence. (a) `mineworld biography --json` carries `event_id`, not a cause, so
      "every added entry's cause is an ActionId this client was answered with" is checked as every
      added entry's event_id being among the events the players' accepted requests returned (the
      server's own answer ties those events to those ActionIds). (b) `talk` is not biographical, so a
      controller answering while alice is played would not show in her biography: M-SB4 was seen
      green on the biography check alone. The test therefore adds a second session playing bob who
      speaks to alice while she is played, and asserts the persisted revision advanced by exactly
      the players' three requests; under M-SB4 it is head+4. (c) Alice's changed components are
      wallet (lower), holdings (+1), conversation-history and **acquaintances** — a first talk makes
      two people acquainted, which SB-4's literal omitted; listed, not hidden.
D-SB8 (bounded) SB-3's "visitor's location unchanged during the hold" is read from the resumed
      client's own first observation (equal `self_location` to the last one before the drop) rather
      than from a second client's: the visitor is not guaranteed to be in a second client's
      perception, and its own observation is the stronger witness. The revision advancing during the
      hold is read from /status as written.
D-SB9 (bounded) Existing test edited because a hosted `--agent` changes what it observes (§16.8):
      tools/cli/tests/ac15_one_alice.rs `clients` 3 → 2 (there_is_only_one_alice) and 2 → 1
      (killing_one_window_…) — an in-server controller is no longer a client (SD-B4). No other
      existing CLI test assertion changed.
D-SB11 (bounded) Paths beside §16.5's list, each the companion of a listed one: server/src/seats/
      tests.rs (seats.rs's unit tests), clients/protocol/checks/reconnect_check.gd.uid (Godot's
      generated id for the listed check), clients/protocol/evidence/README.md (names the two new
      evidence files), tools/cli/tests/ac15_one_alice.rs (D-SB9, the §16.8 exception). The old
      `--agent` printed one `[mineworld] agent: …` line per outcome; the hosted adapter prints none
      while running and its counts in the shutdown statistics instead (SD-B11) — the regenerated
      evidence logs show the difference.
D-SB12 (MATERIAL — operator's decision) CP-B4's 50 ms longest-tick bound failed once (160 ms) in
      the full workspace gate on a machine at load ~21, and held in three other runs (13 ms, suite
      PASS, 1 ms). The longest tick is a single maximum over ~1200 ticks, so one scheduler or fsync
      stall (Durability::PowerLoss, one fsync per hosted request) decides it. Nothing was changed to
      make it pass. Options for the operator (R-S11-5): (a) accept as a load outlier and keep the
      bound, running hosted_town alone in CI; (b) bound a high percentile (e.g. p99) instead of the
      maximum, with the maximum reported; (c) Durability::Commit for hosted saves; (d) a longer
      default pace. The PR is opened for review with this item flagged; not merged.
      Proposal after locating (E-SB9): (b) — bound the tick's p99 at ≤ 50 ms and report the max
      beside it. Every probed run's p99 is ≤ 2.2 ms, an order of magnitude under the bound with any
      hog, so the p99 bound keeps CP-B4's meaning (the hosted work fits the cadence) while not
      failing on one off-CPU stall the server cannot control. Not (c): commits stayed ≤ 5.1 ms even
      under eight fsync-heavy neighbours, and the outliers were as often descheduling as I/O, so
      weaker durability would trade crash safety for a stall it does not remove. Not (d): the
      world thread's own CPU per tick is ≤ 0.6 ms, so a longer pace removes no measured cost.
      (a) alone leaves a maximum that one scheduler hiccup can fail. CP-B4's ≤ 50 ms maximum stays
      frozen until the operator decides.
D-SB13 (bounded) CI on main (#63) ran #83's `test` job red at 0c9dc02: hosted_town's
      `Server::interrupt` spawned a `kill` binary, which the runner image does not have ("kill
      runs: NotFound"). It now sends SIGINT through `sh -c 'kill -INT <pid>'` (the shell builtin),
      with no new dependency; re-run locally green (longest tick 3 ms).
D-SB14 (bounded) IL-a (#80) landed with a structural test, tools/cli/tests/configure.rs, that
      reads `fn persisted(` from main.rs. S11-B moved `persisted` to serve.rs (D-SA10), so the
      merge points that one `checks_before` at serve.rs — the file QS11B-6 already named for the
      drift call. A path outside §16.5, edited because the frozen move requires it; the assertion
      is unchanged.
D-SB10 (bounded) The paced lattice's `genesis` is `HostConfig::epoch`, the instant every world this
      command creates begins at, so it is the same after a resume; hosted instants follow the wall
      clock anyway (ARC-42's accepted limitation).
```

---

# 17. PR S11-C — facts in observations, the `perceived` stream, `acted_through`, deltas (full design)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session`. Superseded: `PR DESIGN — READY FOR FREEZE
REVIEW`. A fresh implementation session executes it under §17.11, starting only once S11-B (#83) has
merged (C-C1, C-C3 and C-C3b excepted, §17.11's sequence).

**Freeze record (2026-10-08).** Relayed by the coordinator:
- **Operator.** QS11C-6 accepted: overhearing is place-level for MVP-0. A hearing range comes later as a
  configurable World Interaction List rule behind the same `EventPerception` seam. D-SB12 ruled: CP-B4's
  bound is **p99 tick ≤ 50 ms with the maximum reported**, and `hosted_town` asserts that form (CA-13
  uses it).
- **Primary session.** QS11C-1 … QS11C-5 and QS11C-7 … QS11C-9 accepted as recommended. S10's R-S11-9 and
  R-S11-10 are adopted, as the coordinator states them:
  - S11-C commits golden frames for `perceived`, `delta`, an observation carrying events, and the new
    refusals and closing reason (`refused-cursor_unavailable.json`, `refused-lagged.json`,
    `closing-lagged.json`), in SD-C14;
  - `mineworld perceived` lands in an early commit, **C-C3b**, right after the audience function.
- **New binding requirement (operator): "我们要保证支持全平台，mac linux windows都可以"** (macOS, Linux and
  Windows must all be supported). It is applied in §17.14.

**Author:** the S11-C/D planning agent, 2026-10-08, worktree
`/Users/yuema137/mineworld-worktrees/plan-s11cd`, branch `plan/s11-cd` (from `main @ 77a8717`).
**Binding parents:** this file §§4.7, 4.8, 5, 6, 7.3, 7.8, 8, 9.3, 11; §15 (S11-A, merged as #76); §16
(S11-B, PR #83 on `mvp0/pr-s11b-seats @ 7811e06`, in review, **not merged**); `overall.md` "Parallel
build-out, 2026-10-08" rulings 1 (S14's R-S11-4 → `acted_through`), 2 (event perception has one owner,
S11-C: S10's P1 folded in), 4 (the module's `events`, `perceived` and `acted_through` readers land here),
6 (ARC-43, DEP-15), 9, 10; "The World Interaction List" QIL-8 (audience narrows at emission; this
function is unchanged by it); `step-17-cognition.md` §§3.3, 3.4.3, 10 (IC-1, IC-9), 11.1 (R-S11-1 …
R-S11-3, R-S11-7, R-S11-8); `step-13-client-2d.md` R-S11-4, R-S11-7; `step-15-demo-3d.md` R-S11-1, R-S11-3,
R-S11-4, M-4, M-6. Evidence goes to §17.12 as `E-SC<n>`, deviations to §17.13 as `D-SC<n>`.

Identifiers in this section are placeholders: commits `C-C0 …`, decisions `SD-C1 …`, acceptance `CA-1 …`,
questions `QS11C-1 …`. The two decision records are already numbered by ruling 6: **ARC-43** (facts reach
observers through perception's judgement) and **DEP-15** (observation deltas, decided by measurement).

## 17.1 Identity, base, scope, preconditions

```text
PR            S11-C — facts in observations, the perceived stream, acted_through, deltas
              (S11, third of five; runs in parallel with S11-D, §19)
base          main after S11-B (#83) merges; designed against main @ 77a8717 + mvp0/pr-s11b-seats @ 7811e06
branch        mvp0/pr-s11c-facts (proposed), its own worktree, one session
scope         one audience function (presence) used for three deliveries:
                observation.events   each frame carries the facts this observer learned since its
                                     previous frame (best effort, bounded, drops counted)
                perceived            an opt-in, reliable, ordered, cursor-resumable stream of the same
                                     facts (S10's R-S11-1/2/3), served live and from the save
                mineworld perceived  the same function over a save, offline
              acted_through on observation frames (S14's R-S11-4); entities in id order;
              the delta measurement (CP-C1) and, if it decides so, delta frames and keyframes;
              the Godot module's readers and delta application (ruling 4; S12's R-S11-7)
not in scope  the admin surface and the host clock (S11-D); a hearing range (a later perception
              system behind the same seam); events for in-server controllers (SD-C9); any kernel,
              contract or persistence change (I-1)
```

**Preconditions (unmerged code this design depends on).** Each is on `mvp0/pr-s11b-seats` and must be on
`main` before C-C2 begins; C-C0 and C-C1 (documents) and C-C3 (presence, no server code) may start before.

| Precondition | What S11-C uses | Where on the S11-B branch |
| --- | --- | --- |
| P-C1 Seat exclusivity | `acted_through` is per **connection**; one connection per seat makes "this connection's requests" well defined | `server/src/seats.rs` `SeatTable::join`, `decide` |
| P-C2 The split world thread | `runtime.rs` (492 lines) with `runtime/world.rs`; the sweep, `remember`, `submit_at`, `consult` as audited in §17.2 | `server/src/runtime.rs`, `runtime/world.rs` |
| P-C3 `Seated`, `Perceived`, `Departure`, the `released` channel | the session's stream loop gains branches beside these | `server/src/host/handles.rs`; `server/src/session.rs` `stream` |
| P-C4 Resume with `resume` | a perceived client reconnects with `resume` **and** its cursor; the hold keeps its seat | `PROTOCOL.md` §4.2 (S11-B) |
| P-C5 `tools/cli/src/serve.rs` | the event perception and the history source are wired beside `PackPerception` | `tools/cli/src/serve.rs` `serve`, `persisted` |
| P-C6 The module's reconnect policy | the perceived cursor rides on it | `clients/protocol/mineworld/world_client.gd` `_schedule_reconnect`, `_send_join` |
| P-C7 CP-B4's ruling (D-SB12) | `hosted_town` is one of this PR's regression tests; its bound must be decided | §16.11 D-SB12, operator |

If S11-B changes any of these in review, C-C2's first act is a re-audit of §17.2 and a bounded amendment
recorded as a deviation; a change to a seam's meaning (not its location) is a stop.

## 17.2 Audit anchors

Read on `main @ 77a8717` and `mvp0/pr-s11b-seats @ 7811e06` (marked **B**). Re-verified at C-C2.

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| **B** `server/src/runtime.rs` (492) `remember` l. 394, `advance` l. 333, `submit_at` l. 274, `sweep` l. 413 | Every recorded fact passes through `remember` (from `advance` and `submit_at`) into a bounded recent window. `sweep` computes one `Perception::observe` per subscriber and `try_send`s it; a full channel drops the frame and counts it; a closed one departs as `Dropped`. | The audience fold and the per-subscriber fan-out attach at `remember` (record time), never at `sweep` (SD-C3). `runtime.rs` is 8 lines under the trigger, so the delivery code moves out first (SD-C12). |
| **B** `runtime.rs` `consult` l. 362 | Hosted controllers get `Perception::observe` directly and submit through `submit_at`. | Hosted seats get no event queue (SD-C9); their observations stay exactly as S11-B made them. |
| **B** `server/src/host.rs` `Command::Submit { observer, request, reply }` l. 255 | A submit names the observer, not the connection. | It gains the `SubscriptionId`, so `acted_through` is per connection (SD-C7). |
| **B** `server/src/host/handles.rs` `Perceived { revision, observation }` l. 44; `Seated` l. 55 | What crosses to a session per sweep, and the seated handle. | `Perceived` becomes the stream item enum of SD-C5; `Seated` gains the perceived head (SD-C6). |
| **B** `server/src/session.rs` (399) `stream` l. 245, `seq` l. 253 | One task; `biased` select on `released`, observations, the socket; `seq` counts observation frames from 1. | Backfill, `perceived` frames, delta encoding and `acted_through` go into the stream loop; the encoder is a pure function in `protocol/delta.rs` (SD-C10). |
| `server/src/perception.rs` (118) `Perception`, `PerceptionContext::recent_events` | One seam, `observe`; `recent_events` exists and nothing reads it. | A second seam beside it, `EventPerception` (SD-C2); `recent_events` stays for `observe`. |
| `systems/presence/src/observe.rs` l. 28 ("Events … nothing") | `observe` leaves `Observation.events` empty and names S10/S11 as the owners. | `observe` is **not** edited; events are added by the server from the audience function (SD-C4), so `mineworld run` and both digests are untouched (I-6). |
| `systems/presence/src/event.rs` `Arrived`, `arrival`, `arrivals` l. 32–169 | Presence states `arrived` (at genesis, on a stride, on a displacement) with `Visibility::Place`; `person-entered-place`, `stopped-short` likewise. | The `Whereabouts` fold reads only presence's own `arrived` (ARC-28 point 3; S10 §3.3.2). |
| `contracts/src/observation.rs` `PerceivedEvent<P>(EventEnvelope<P>)` l. 140, `Observation::with_events` l. 426 | The contract type for a perceived fact exists, transparent over the envelope. | Used as is (I-1). |
| `contracts/src/event.rs` `EventEnvelope` l. 383, `EventRecord::new<E: Event>` l. 205 | An envelope's payload record can only be built from a static `Event` type; there is no `map_payload`. Deserialization goes through `EventEnvelopeFields` with the agreement check. | The server cannot construct an `EventEnvelope<Value>` directly without a contract change; it renders one through serde (SD-C4). A contract helper is proposed, not taken (QS11C-3). |
| `docs/DECISIONS.md` DEP-5 note | "event and action payloads remain bytes a system encodes and the kernel never interprets". Every in-tree pack's `codec.rs` encodes JSON (13 files, `serde_json::to_vec`). | JSON is a convention of the in-tree packs, not a contract; a non-JSON payload must have a defined wire form (SD-C4). |
| `persistence/src/world.rs` `recent_facts(count)` l. 181; `persistence/src/backend.rs` `last_facts`, `facts_of(revision)` | Reading every fact of a save already exists (`biography.rs` l. 92 reads `last_facts(all)`). No "facts after id" read. `SqliteBackend` runs in WAL mode (`sqlite.rs` l. 132). | Resume and `mineworld perceived` read through existing public API (I-1). A second, read-only connection may read while the world thread writes (WAL). No persistence edit. |
| **B** `tools/cli/src/perceive.rs` `PackPerception` | The composition root adapts presence onto `Perception`. | `PackEventPerception` and `SavedHistory` live beside it (SD-C2, SD-C6). |
| **B** `tools/cli/src/main.rs` (486) | Under the trigger by 14 lines. | `perceived`'s arguments live in `perceived.rs` (`clap::Args`); `main.rs` gains one variant and one arm (SD-C13). |
| **B** `server/PROTOCOL.md` §5.2, §5.3, §10 landing table | `events`, entity order, `delta`, `acted_through`, `perceived` with `cursor_unavailable`/`lagged` are listed as S11-C's, shapes left to it. | §17.4 specifies them; C-C1 writes them. |
| **B** `clients/protocol/mineworld/world_client.gd` (569), `observation.gd` (284) `events()` l. 152 | The module is past 500 lines; `events()` reads `observation.events`; unknown frame kinds are warned about and ignored. | Delta application goes into a new `delta.gd`; the client grows only by the frame arms and the cursor (SD-C11). |
| `tests/acceptance/tests/seam_vocabulary.rs` l. 115, `precursor_vocabulary.rs`, `configuration_vocabulary.rs`, `ac1_composability.rs` | Presence's sources are scanned for physics words and other packs' vocabulary. | `audience.rs` names no other pack's event type and no physics word; every scan passes unedited (CA-14). |
| `step-17-cognition.md` §3.3.4 | S10 planned `Observation.events` to stay empty, perceived facts only in their own frame, and IC-9 to check transcripts byte-identical. | Superseded by ruling 2 ("delivers both … from the one function"); IC-9 reduces to the digests (QS11C-1). |

## 17.3 Design decisions (SD-C1 … SD-C14)

| ID | Decision | Why |
| --- | --- | --- |
| **SD-C1** | **One audience rule, owned by presence**: `systems/presence/src/audience.rs`. `Whereabouts` (a `BTreeMap<EntityId, PlaceId>`), `Whereabouts::from_world(&WorldRead)` (seeded from presence's own `Presence` components), `apply(&EventEnvelope)` (folds presence's own `arrived` facts, ignores every other type), `admits(&Whereabouts, &EventEnvelope, observer) -> bool`, and `perceived_by(facts, observer, since) -> impl Iterator` (a fresh fold from the first fact, used offline and for resume). The rule is S10 §3.3.1's, verbatim: `SystemInternal` never; `Public` always; `Participants` iff the observer is a participant; `Entities(S)` iff in `S`; `Place(p)` iff a participant or subject, or the observer's whereabouts **after applying this fact** is `p`. | Ruling 2; S10 §3.3; QIL-8: the Interaction List narrows a fact's `Visibility` at emission, so this function reads only the envelope and needs no change for it. One rule in one place is what lets live, resume and offline agree (I-4 of S10). |
| **SD-C2** | **A second server seam, `EventPerception`**, in `server/src/perception.rs`: `fn record(&mut self, fact: &EventEnvelope)` (fold) and `fn admits(&self, fact: &EventEnvelope, observer: EntityId) -> bool`. Default `PerceivesNoEvents` admits nothing. `HostedWorld::perceiving_events(impl EventPerception)` sets it. The CLI's `PackEventPerception` wraps presence's `Whereabouts`, seeded with `from_world` on the world thread at start. The server names no pack (I-9). | The same shape as `Perception`/`PackPerception`. Separate from `Perception` because `observe` is a pure read while the audience is a fold that must see every fact in order. |
| **SD-C3** | **Judged at record time, fact by fact.** In `remember`, for each new fact in log order: `record(fact)`, then `admits(fact, observer)` for each subscriber, queueing admitted facts on that subscriber. Never re-judged at sweep. | §4.7: a `Place` fact is judged against where people were when it happened, not where they walked to before the next 100 ms sweep. Folding per fact (not per dispatch) is what makes "a person who arrives perceives their own arrival" true inside one request. |
| **SD-C4** | **A fact on the wire** is the contract's `PerceivedEvent<serde_json::Value>`: the envelope as the contract serializes it, with `payload.payload` the pack's payload parsed as JSON. Built in one function, `protocol/fact.rs` `wire_fact(&EventEnvelope) -> PerceivedEvent<Value>`, through serde (serialize the envelope to a `Value`, replace `payload.payload` with the parsed bytes, deserialize as `EventEnvelope<Value>`, which re-runs the contract's agreement check). A payload whose bytes are not JSON is delivered with `payload.payload: null`, and the server prints one `[world] event type <t> has a payload that is not JSON` line per event type per process. | I-1 forbids a contract helper; this needs none and the contract still validates the result. Every in-tree pack encodes JSON; the null form is defined rather than left to chance, and visible rather than silent (QS11C-3 proposes `EventEnvelope::map_payload` for a later contract PR). |
| **SD-C5** | **One ordered stream per subscriber**: the subscriber channel carries `Streamed::Facts { through, events }` and `Streamed::Observation(Perceived)`. Per subscriber the world thread keeps (a) `pending_events`: facts for the next observation frame, at most `HostConfig::event_backlog` (default 256); overflow drops the oldest and counts `events_dropped`; cleared only when the observation carrying them was queued; (b) if the connection asked for `perceived`, `pending_perceived`: at most `HostConfig::perceived_backlog` (default 4096) facts; on each sweep it is flushed first, with `try_send`; if the channel is full it stays pending **and that subscriber's observation is skipped this sweep** (counted as dropped); if it exceeds its bound the subscriber is released with `lagged`. | Ordering by construction (S10 R-S11-1: every `perceived` frame covering revision R precedes any observation of R); no silent drop of a reliable fact; the world never waits (I-11). A single channel makes the order the channel's order; two channels would need the session to re-establish it. |
| **SD-C6** | **The `perceived` stream.** `join.perceived = { "since": <EventId> \| null }`, optional. At a granted join the world thread records the subscriber's `head` = the newest `EventId` the world has recorded, and from then on queues live admitted facts with ids `> head`. `Seated` returns `head`. If `since` is `null` or `< head`, the session first sends the backfill — the facts in `(since, head]` this observer perceived — from `HostedWorld::with_history(impl PerceivedHistory)`, read off the world thread with `spawn_blocking`, in `perceived` frames of at most 256 events, the last with `through = head`; only then does it forward the live stream. A world with no history source (an ephemeral world) answers a `since` older than `head` with `cursor_unavailable`; so does any `since` greater than `head` (not a cursor of this world). The CLI's `SavedHistory` opens the save read-only (`SqliteBackend::open`, WAL), reads every fact with `last_facts`, and runs `audience::perceived_by` — the function `mineworld perceived` runs. | S10 R-S11-1/2; one function for live, resumed and offline (I-4). The world thread never reads history (I-11): a 300-day market-town log is ~373 000 facts. An ephemeral world keeping a re-foldable history would be a second log; S10 needs resume only for persisted worlds (QS11C-2). |
| **SD-C7** | **`acted_through`** on every `observation` (and `delta`) frame: the newest `ActionId` the server allocated to a request submitted **on this connection** that was dispatched (any `ActionResult`) before the observation was computed; `null` before the first. Per subscriber, set in `submit` after `dispatch` returns; a new connection on the seat (resume, takeover) starts at `null`. `Command::Submit` carries the `SubscriptionId`. Requests refused before allocation (`actor_not_observer`, `paused`) never set it. | S14's R-S11-4 (the 3D correction rule), ruling 1. Per connection, not per observer, because a resumed or taken-over seat is a new client whose earlier requests it never made. |
| **SD-C8** | **Entities in ascending `EntityId` order**, sorted by the session before encoding (or by the delta encoder). | §5.3 field note; makes "reconstructed equals whole" independent of presence's iteration order. |
| **SD-C9** | **In-server controllers get no events.** `consult` keeps calling `observe` directly; no queue, no fold read. | No in-tree controller reads `events` (the paced controller reads histories, `ARC-27`; the reactive one reads its conversation history, F-13 closed by `since`). A queue nobody drains is a cost with no consumer; the seam is there when a hosted controller needs it (QS11C-4). |
| **SD-C10** | **Deltas, measured first (CP-C1), with all three outcomes specified now.** `server/src/protocol/delta.rs`: `diff(prev, next) -> ObservationDelta` and `apply(prev, &delta) -> Observation`, pure, over §5.4's shape. The measurement (C-C7) records real frames and computes, per client per second, whole-JSON bytes, the typed delta's bytes and RFC 6902 patch bytes (`json-patch` 4.2, a dev-dependency of `mineworld-cli` only). The frozen rule (§9.3 CP-C1): **typed** if ≤ ½ of whole bytes and `json-patch` is not within 10 %; **json-patch** if within 10 % of typed and ≤ ½ of whole — then `delta.patch` is an RFC 6902 array against the JSON of the base observation (shape specified in §17.4 so no re-freeze is needed); **keyframes only** otherwise — `delta.rs` is deleted, §5.3 is marked "not shipped in revision 2", and DEP-15 records why. Keyframes: the first frame, every `--keyframe-every` (default 50) frames, the first after a backfill. Computed in the session against the last frame *this connection sent*. | §4.8, §7.3, ARC-23. Pre-specifying the json-patch shape keeps the decision mechanical rather than a mid-PR protocol stop. |
| **SD-C11** | **The Godot module** (ruling 4): `MineWorldObservation.acted_through()`, `events_of(event_type)`; `MineWorldClient.perceive_from(cursor: Variant)` (opt-in before `connect_to_world`; `null` = from the beginning), `signal perceived(events: Array, through: String)`, `var perceived_cursor` (the last `through`), and the reconnect policy rejoins with `perceived: { since: perceived_cursor }`; delta frames are applied inside the module by `mineworld/delta.gd` (`apply(base: Dictionary, delta: Dictionary) -> Dictionary`), so `observed` always emits a whole observation (S12's R-S11-7); a base mismatch or a `remove` of an id not held disconnects and, with `reconnect` on, resumes (which yields a keyframe). Every existing call and name unchanged. | Ruling 4; S12 R-S11-7; S14 M-4, M-6; R-9 (verified from the far side, CA-15). |
| **SD-C12** | **`runtime.rs` split before growth**: `Subscriber`, `sweep`, `release`, `depart` and the new fan-out move to `server/src/runtime/delivery.rs` in a pure-move commit (C-C2). | ENGINEERING_STANDARDS §10; §8.2; §19's file ownership (S11-D splits a different part). |
| **SD-C13** | **`mineworld perceived <world> --save DIR --person KEY [--since ID] [--json]`** in `tools/cli/src/perceived.rs` (`clap::Args` there; one variant in `main.rs`). Reads the save and the pack, nothing else (as `biography`, ARC-29); prints one line per perceived fact (`<id> <at> <event_type> place=<key> caused_by=<…>`), or with `--json` one `PerceivedEvent` per line in the wire form of SD-C4. | Ruling 2; S10 IC-1's offline half; MODULE_SPEC §8.1. |
| **SD-C14** | **Protocol additions inside revision 2** (the landing table already names them S11-C's): `join.perceived`; server frames `perceived` and (per SD-C10) `delta`; `observation.acted_through`; refusal codes `cursor_unavailable` (the connection stays in the handshake) and `lagged` (followed by `closing`); closing reason `lagged`; `events_dropped` non-zero; entity order. **Golden frames (S10 R-S11-9/10, freeze):** `perceived.json`; `observation.json` with `acted_through` and at least one event; `delta.json` (under the keyframes-only outcome, the file is not added and §5.3 says so); `join.json` with `perceived`; `refused-cursor_unavailable.json`; `refused-lagged.json`; `closing-lagged.json`. | ARC-41's "may omit, never redefine"; R-S11-7 (golden frames for S10's Python SDK). |

## 17.4 Wire additions, exactly (written into `PROTOCOL.md` by C-C1)

```json
{ "t": "join", "protocol": 2, "invite": "…", "nickname": "cognition", "seat": "alice",
  "resume": null, "take_over": false, "perceived": { "since": "1873" } }

{ "t": "perceived", "through": "1907",
  "events": [ { "id": "1890", "at": 4100, "event_type": "spoke", "subjects": ["5"], "participants": ["5","7"],
                "place": "3", "caused_by": { … }, "payload": { "event_type": "spoke", "schema_version": 1,
                "payload": { "utterance": "hello" } }, "visibility": { "place": "3" }, "provenance": { … } } ] }

{ "t": "observation", "seq": 12, "revision": 7, "acted_through": "41", "observation": { …, "events": [ … ] } }

{ "t": "delta", "seq": 13, "base": 12, "revision": 7, "acted_through": "41", "delta": ObservationDelta }
   — or, under SD-C10's json-patch outcome: "patch": [ { "op": "replace", "path": "/at", "value": 4113 }, … ]
```

- `perceived` (object, optional on `join`): `since` is required inside it, an `EventId` string or `null`.
  Unknown fields inside it are `malformed_frame`. Without it no `perceived` frame is ever sent.
- `perceived` frame: `events` admitted for this observer, ascending `EventId`, never reordered, never
  dropped, never duplicated within a connection; `through` the newest `EventId` the server has
  considered for this connection (admitted or not), which a client stores as its cursor. A frame is sent
  when it has at least one event, and once at the end of a backfill even if empty.
- Order: every `perceived` frame carrying facts of revision R is sent before any `observation`/`delta` frame
  whose `revision` is R or later.
- `cursor_unavailable` (refusal, connection stays): `since` older than this world can serve (no history
  source) or newer than its head. `lagged` (refusal, then `closing { reason: "lagged" }`): the
  connection's perceived backlog overflowed; it resumes with its cursor.
- `observation.events`: the facts this observer learned since the previous frame on this connection,
  oldest first; best effort — `WorldSummary.events_dropped` counts facts lost to a full queue. The
  reliable form is `perceived`.

## 17.5 Acceptance (decided before measuring, `ARC-23`)

Bounds are literals from the requirement (§§4.7, 4.8, 9.3; S10 §§3.3, 10, 11.1; S14 R-S11-4). Each guard
names the mutation that must turn it red; mutations are planted on the working tree, seen red, reverted,
and recorded in §17.12. Oracles are independent of the code under test.

```text
CA-1  The rule (unit, presence). For each Visibility over a fixture of envelopes and a hand-written
      Whereabouts: SystemInternal admits nobody; Public everybody; Participants exactly the
      participants; Entities(S) exactly S; Place(p) the participants, the subjects, and exactly those
      whose whereabouts after the fact is p — including a person whose own arrival into p is the fact,
      and excluding a person whose departure from p was folded before the fact.
      [systems/presence/src/audience.rs tests]
      M-CA1  Place(p) admits everyone → fails.

CA-2  One function: live + resumed = offline (S10 I-4, IC-1). `mineworld server worlds/social-cafe
      --town --save DIR --hold 10`: a client joins `wanderer` with perceived {since: null}, records every
      perceived frame for 20 wall s, drops its socket without leave, rejoins inside the hold with its
      resume and perceived {since: <last through>}, records 20 wall s more, leaves; the server is
      killed (`Child::kill`, portable, §17.14). The ids
      received, concatenated, equal exactly the ids of `mineworld perceived worlds/social-cafe --save
      DIR --person wanderer --json` (no gap, no duplicate, ascending), and that export is non-empty
      beyond genesis (≥ 1 `spoke` or `arrived` stated after genesis).
      [tools/cli/tests/perceived.rs]
      M-CA2  resume serves (since + 1, head] → one fact missing; fails.

CA-3  Record-time judgement and INV-13 (in-process host, deterministic, no wall-clock race). On
      social-cafe through `WorldHost` with PackPerception + PackEventPerception: three connections on
      seats chosen from the authored `location:` lines so that two are in one place and the third in
      another (verified from people/*.yaml at C-C6 and written into the test as literals). Script:
      A says a line to B's place-mate → B's next observation.events holds that `spoke`, stated
      Place(that place); C's never does. Then, with no sweep between: A speaks, then B moves out of
      the place → B still receives the line said before it left. Then A speaks again → B does not.
      A `conversation-started` (Participants) reaches only its two participants.
      [tools/cli/tests/facts.rs]
      M-CA3  admits evaluated after the whole batch is folded (sweep-time judgement) → the
             "said before it left" case fails.

CA-4  observation.events is exact or accounted. A connection with perceived opted in stops reading
      its socket for 5 s on a hosted market-town --town, then reads on. Over the run: no fact id
      appears in two frames' events; the multiset of ids over all frames' events plus the increase of
      /status events_dropped attributable to it equals the perceived stream's ids for the same
      connection (the reliable oracle, CA-2's function). [server/tests/facts.rs with a stub
      EventPerception that admits by a fixed table, plus the real-binary variant in
      tools/cli/tests/facts.rs]
      M-CA4  pending_events cleared even when try_send fails → ids missing and uncounted; fails.

CA-5  Flow control. In-process server with perceived_backlog = 8 and a stub perception admitting
      every fact: a perceived connection that does not read while 20 facts are recorded receives
      refused {lagged} then closing {lagged}; rejoining with resume and its last cursor delivers the
      missing ids exactly (history from a stub PerceivedHistory over the same facts). Never a gap.
      [server/tests/facts.rs]
      M-CA5  overflow drops the oldest silently instead of releasing → gap; fails.

CA-6  Order. On CA-2's recorded stream, joined with the save after the run (persistence's
      facts_of(revision) read in the test): for every observation frame with revision R, every
      admitted fact recorded at revision ≤ R arrived in a perceived frame before it.
      M-CA6  sweep sends the observation before flushing pending_perceived → fails.

CA-7  acted_through. Server socket test: null on every frame before the first submit; after a
      submit answered with action_id a, the first observation frame computed after that answer
      carries a, and no later frame carries less; a second connection's submits never appear in the
      first's acted_through; after the seat is resumed (or taken over) the new connection's frames
      start at null. [server/tests/facts.rs]
      M-CA7  acted_through kept per observer instead of per subscription → the resume case fails.

CA-8  Measure before adopting (CP-C1, a check, not a guard). `market-town --town`, four sessions
      (visitor, wanderer, and two taken-over seats), 60 wall s, `--keyframe-every 1` so every recorded
      frame is whole: per client per second, whole bytes; typed-delta bytes (server delta::diff);
      RFC 6902 bytes (json_patch::diff). The outcome of SD-C10's rule is recorded in E-SC with the
      numbers, and DEP-15 states it. [tools/cli/tests/deltas.rs, #[ignore], run explicitly]

CA-9  Reconstruction (I-10, Rust), if deltas ship. Over CA-8's recorded whole frames (≥ 2 000
      consecutive pairs): for every pair, apply(prev, diff(prev, next)) serializes byte-equal to next
      after canonical entity order. Golden delta cases under server/tests/frames/deltas/
      (hand-reviewed: an entity added, removed, changed; self_location to null; affordances reordered;
      events only) pass in Rust. [server/tests/deltas.rs]
      M-CA9a apply ignores entities.remove → fails. M-CA9b upserts not re-sorted → fails.

CA-10 Deltas on the wire, if shipped. Through the binary: frame 1 is an observation; every delta's
      base is the previous frame's seq; frames 50, 100, … are observations; the first frame after a
      resume and after a backfill is an observation; a client's Rust applier over 60 s raises no
      base mismatch and no unknown remove. [tools/cli/tests/deltas.rs]

CA-11 INV-9 over S11-C's surfaces. Through the binary on a persisted social-cafe, no --town, in the
      routine-free first minutes: {"t":"perceived","events":[…]} and {"t":"delta",…} from a seated
      client → unknown_frame; a join whose perceived carries an extra field ("events", "observer")
      → malformed_frame, no welcome; a submit carrying "acted_through" → malformed_frame; a join with
      perceived {since: "<head + 1000>"} → cursor_unavailable and the same connection then joins with
      perceived {since: null}. Afterwards /status's revision equals its value before, and after
      SIGKILL `mineworld inspect DIR` reports exactly validate's genesis fact count.
      [tools/cli/tests/server_command.rs]
      M-CA11 drop deny_unknown_fields from the perceived object → the extra-field join is welcomed.

CA-12 cursor_unavailable, both worlds. Ephemeral server: a join with perceived {since: null} after
      any fact was recorded → cursor_unavailable; {since: <current head>} → welcomed, live only.
      Persisted server: {since: null} serves from genesis (the first perceived event id is the
      smallest admitted genesis fact). [server/tests/facts.rs; tools/cli/tests/perceived.rs]

CA-13 A resume of a long save does not stall the world. A 300-day market-town save (`mineworld run
      --days 300 --save`), hosted with --town; a client joins with perceived {since: null}: the
      backfill completes without lagged, and the world thread's p99 tick in the shutdown line is
      ≤ 50 ms (CP-B4 as ruled on D-SB12; the maximum is reported beside it and recorded). Backfill
      wall time and fact count recorded.
      [tools/cli/tests/perceived.rs, #[ignore] if > 2 min, run in the close commit]
      M-CA13 history read on the world thread → the p99 tick bound fails.

CA-14 Nothing else moved; scope. Both 300-day seed-7 digests at the head equal those at the base
      (I-6; presence's observe and run.rs untouched); every existing test passes, the only edits to
      existing tests being join/observation shapes (golden frames, a helper) listed in §17.13; hosted
      controllers' observations unchanged (consult code untouched beyond the move). No diff under
      kernel/, contracts/, persistence/, worlds/, worldpack/, sdk/, authoring/, cognition/; under
      systems/ only systems/presence/src/{audience.rs, lib.rs} and its tests; the server gains no
      mineworld-* pack dependency; precursor_vocabulary, seam_vocabulary, configuration_vocabulary,
      ac1_composability pass unedited. runtime.rs, session.rs, host.rs, protocol.rs, main.rs under
      500 lines.

CA-15 The far side (R-9). Headless Godot 4.7: `clients/protocol/run.sh perceived` — a check joins with
      perceive_from(null), receives perceived frames, logs a `spoke` it overheard in its own
      observation.events, submits a talk and sees acted_through equal its result's action_id, drops
      its socket and resumes with its cursor (no duplicate id, no gap against the server's own
      `mineworld perceived` after stop); if deltas ship, `run.sh deltas` applies 60 s of deltas with no
      base mismatch, and delta.gd passes every golden case of CA-9. `run.sh evidence`, `affordances`,
      `reconnect` and `./mineworld-slice --world --link` still pass.
```

## 17.6 Change set

```text
systems/presence/src/{audience.rs (new), lib.rs}; systems/presence/tests/audience.rs (new, if not inline)
server/src/{perception.rs, runtime.rs, runtime/delivery.rs (new, moved + fan-out),
            runtime/status.rs (one line, only if S11-D's move has landed; else runtime.rs), host.rs,
            host/handles.rs, session.rs, protocol.rs, protocol/connection.rs, protocol/fact.rs (new),
            protocol/delta.rs (new; deleted again under the keyframes-only outcome), lib.rs}
server/{PROTOCOL.md, README.md}
server/tests/{facts.rs (new), deltas.rs (new), frames.rs, frames/{join,observation,perceived,delta}.json,
              frames/deltas/*.json (new), support/mod.rs}
tools/cli/src/{main.rs, serve.rs, perceive.rs, perceived.rs (new), history.rs (new)}
tools/cli/{Cargo.toml (dev-dependency json-patch)}; Cargo.lock
tools/cli/tests/{facts.rs (new), perceived.rs (new), deltas.rs (new), server_command.rs, support/mod.rs}
clients/protocol/{mineworld/{world_client.gd, observation.gd, delta.gd (new)},
                  checks/{perceived_check.gd, delta_check.gd} (new, with .uid), run.sh, ADOPTION.md,
                  README.md, evidence/* (regenerated)}
docs/{DECISIONS.md (ARC-43, DEP-15), MODULE_SPEC.md §8.1}
.structured-coding/plans/mvp0/{step-12-server.md §17, handoff-s11c.md}
```

## 17.7 Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its evidence.

### C-C0 — Design (this section) — docs only

- [x] Implementation: §17, and the pointer in §9.3, from the audit in §17.2.
- [x] Validation: Markdown only; `check_doc_headings`, `check_decision_ids` (E-SC0 at freeze).
- [x] Review: every anchor cites a file and line or a document section; every ruling bearing on S11-C is
  applied in §17.1; every guard has a mutation or says why it has none (CA-8, CA-14 are checks).
  Self-review; the freeze is pending.

### C-C1 — Specs before code: `PROTOCOL.md`, ARC-43, MODULE_SPEC §8.1

**Goal.** §17.4 and SD-C1 … SD-C14 exist as reviewable specification before code (`CLAUDE.md` §2.2).
**Scope.** `server/PROTOCOL.md` §2 (`join.perceived`), §5.2 (events, entity order, `acted_through`
landed), §5.3 (delta: "per DEP-15", all three outcomes stated), new §5.8 (`perceived`), §5.5
(`cursor_unavailable`, `lagged`), §5.6 (`lagged`), §10's rows; `docs/DECISIONS.md` **ARC-43** (one
audience function owned by presence, judged at record time, delivered three ways; supersedes S10
§3.3.4's "events stays empty"; rejected: judging at sweep, a broadcast channel, an ephemeral re-foldable
history); `MODULE_SPEC.md` §8.1 (`mineworld perceived`). **Depends on:** freeze; preconditions not needed.
**Non-goals:** DEP-15 (written in C-C7 with the numbers).

- [ ] Implementation · [ ] Validation: `check_decision_ids`, `check_doc_headings`; §6 and §6.2 of
  `PROTOCOL.md` untouched · [ ] Review: every new value in the landing table; terminology per
  `CORE_CONCEPTS.md` (fact, Visibility, observer, PerceivedEvent).

### C-C2 — Pure move: `runtime/delivery.rs`

**Goal.** SD-C12. **Scope.** Move `Subscriber`, `sweep`, `release`, `depart` to `runtime/delivery.rs`; no
behaviour change. **Depends on:** S11-B merged (P-C2). **Failure case:** any test edit means the move was
not pure.

- [ ] Implementation · [ ] Validation: `cargo test -p mineworld-server` — same names and counts as the
  base; clippy `-D warnings` · [ ] Review: `runtime.rs` < 450 lines; public paths unchanged.

### C-C3 — Presence's audience

**Goal.** SD-C1, CA-1. **Scope.** `systems/presence/src/audience.rs`, `lib.rs` (`pub mod audience`).
**Non-goals:** `observe` untouched; no new event type; no other pack named.

- [ ] Implementation · [ ] Validation: CA-1 with M-CA1; a fold-equals-components check — over a 30-day
  social-cafe `run --save`, `Whereabouts` folded from every fact equals `from_world` of the resumed
  world (S10's A-1; failing it is a stop: the live seed and the offline fold would disagree); presence's
  existing tests, `seam_vocabulary`, `precursor_vocabulary`, `configuration_vocabulary` pass unedited
  · [ ] Review: the module decodes only `arrived`; no physics or market word; `admits` is total.

### C-C3b — `mineworld perceived`, early (S10 R-S11-10, freeze)

**Goal.** SD-C13, landed before the server work so that S10's P4 can build memory fixtures on it.
**Scope.** `tools/cli/src/perceived.rs` (the `clap::Args` struct and the command), one variant and one
match arm in `main.rs`, `tools/cli/tests/perceived.rs` (the offline half), MODULE_SPEC §8.1's line.
**Depends on:** C-C3 only, so it may land on `main` before S11-B merges. If it does, S11-B's rebase
carries the one `main.rs` variant, which is mechanical. Path handling uses `std::path` only, with no Unix
assumption (§17.14).

- [ ] Implementation · [ ] Validation:
  - a 30-day social-cafe `run --save`, exported for one person:
    - every exported id is a fact in the save;
    - every `Place(p)` fact exported is one the person was in `p` for, checked against an independent
      scripted case with known placements (CA-3's literals);
    - `--since X` exports exactly the suffix after X;
    - an unknown person is refused by name;
    - `--json` lines decode as `PerceivedEvent<Value>`.
  · [ ] Review:
  - it reads the save and the pack and nothing else;
  - it opens the save, reads it, and closes it, so no handle outlives the command (Windows locking,
    §17.14).

### C-C4 — The server seams and the wire form of a fact

**Goal.** SD-C2, SD-C4, SD-C6's `PerceivedHistory` trait. **Scope.** `perception.rs` (`EventPerception`,
`PerceivesNoEvents`, `PerceivedHistory`, `HistoryUnavailable`), `protocol/fact.rs` (`wire_fact`),
`host.rs` builders, `lib.rs` re-exports. **Validation (unit):** `wire_fact` of a JSON payload equals a
hand-written expected frame; a non-JSON payload yields `payload.payload: null`; the contract's agreement
check still rejects a tampered type (round-trip of a mismatched envelope fails).

- [ ] Implementation · [ ] Validation · [ ] Review: the server names no pack; no `Serialize` added to a
  contract type; defaults are the safe direction.

### C-C5 — The world thread and the session: fan-out, `perceived`, `acted_through`

**Goal.** SD-C3, SD-C5, SD-C6, SD-C7, SD-C14 through a real socket. **Scope.** `runtime.rs`
(`remember` fans out; `submit` sets `acted_through`), `runtime/delivery.rs` (queues, ordered flush,
`lagged`), `host.rs` (`Command::Submit` gains the subscription; `JoinRequest` gains `perceived`;
`HostConfig.event_backlog`, `perceived_backlog`), `host/handles.rs` (`Streamed`, `Perceived` gains
`acted_through`, `Seated` gains `head`), `session.rs` (backfill via `spawn_blocking`, `perceived`
frames, `lagged`), `protocol*` (frames, codes, reason), golden frames, `server/tests/facts.rs` (CA-4
stub half, CA-5, CA-7, CA-12 ephemeral). **Depends on:** C-C2, C-C4.

- [ ] Implementation · [ ] Validation: server suites; M-CA4, M-CA5, M-CA7; clippy · [ ] Review: I-11
  (no history read, no wait on the world thread); the session holds no world state; `consult` untouched.

### C-C6 — The composition root: event perception, history, `mineworld perceived`; real-binary tests

**Goal.** SD-C2's adapter, SD-C6's `SavedHistory`, SD-C13; CA-2, CA-3, CA-4 (binary), CA-6, CA-11,
CA-12 (persisted). **Scope.** `tools/cli/src/{perceive.rs, history.rs, perceived.rs, serve.rs, main.rs}`;
`tools/cli/tests/{perceived.rs, facts.rs, server_command.rs, support/mod.rs}`.

- [ ] Implementation · [ ] Validation: those tests; M-CA2, M-CA3, M-CA6, M-CA11; every suite that starts
  the binary passes (ac13, ac15, milestone_b, milestone_c, restart, ac3_reconnect, ac5_takeover,
  hosted_town) · [ ] Review: `SavedHistory` opens the save read-only and runs only on a blocking task;
  `perceived` reads nothing but the save and the pack.

### C-C7 — Entity order, the pure delta, and the measurement (CP-C1)

**Goal.** SD-C8, SD-C10's measurement; CA-8; DEP-15. **Scope.** `protocol/delta.rs` (`diff`, `apply`),
the session's sort, `server/tests/deltas.rs` (golden cases, CA-9 over recorded frames when they exist),
`tools/cli/tests/deltas.rs` (`#[ignore]` measurement), `tools/cli/Cargo.toml` dev-dependency `json-patch`
(licence and version verified with `cargo info` first), `docs/DECISIONS.md` DEP-15 with the numbers and
the outcome. **Gate spec (before running):** claim — which encoding halves the bytes; owner — real
binary measurement; evidence — the three byte rates; counterfactual — none (a measurement); cost — one
60 s run, at most two.

- [ ] Implementation · [ ] Validation: CA-8 recorded (E-SC); CA-9 with M-CA9a, M-CA9b · [ ] Review: the
  outcome follows the frozen rule mechanically; DEP-15 records the `permessage-deflate` dead end.

### C-C8 — Deltas on the wire, or their removal (by C-C7's outcome)

**Typed or json-patch:** the session's encoder, keyframes, `--keyframe-every` (`main.rs`, `serve.rs`,
`HostConfig`), golden `delta.json`, CA-10. **Keyframes only:** `delta.rs` and its tests deleted,
`PROTOCOL.md` §5.3 marked "not shipped in revision 2", no flag added; this commit is then documentation.

- [ ] Implementation · [ ] Validation: CA-10 (or N/A with the outcome cited) · [ ] Review: a client
  holding any whole observation can always continue; `observation` remains acceptable at any time.

### C-C9 — The Godot module and the far side

**Goal.** SD-C11; CA-15. **Scope.** `world_client.gd` (perceived opt-in, cursor, `perceived`/`delta`
arms, reconnect with cursor), `observation.gd` (`acted_through`, `events_of`), `delta.gd`,
`checks/{perceived_check.gd, delta_check.gd}`, `run.sh perceived|deltas`, `ADOPTION.md` §§2, 3.4, 6,
`README.md`, regenerated evidence (invite line kept out). **Validation (one Godot window at a time):**
CA-15. **Review:** every existing name and call valid; no rule in the module (`check_client_rules.py`
where it applies); `world_client.gd` growth only frame arms and the cursor.

- [ ] Implementation · [ ] Validation · [ ] Review

### C-C10 — Close: README, digests, scope, sizes, full gate, ledger, PR

- [ ] Implementation: `server/README.md`; ledger · [ ] Validation: CA-13 (the long-save resume, once);
  CA-14 (digests at head vs base, scope diff, sizes, scans); **one** full gate on the final head (PR CI if
  the workflow runs it, else local in the background) · [ ] Review: CA-1 … CA-15 with evidence; every
  mutation planted, red, reverted; the PR marked READY FOR OPERATOR REVIEW — DO NOT MERGE.

**E-SC0 (first action after the freeze):** the base's two 300-day digests and test counts.

## 17.8 Test ownership

```text
STATIC      fmt; clippy -D warnings (exhaustive matches over Streamed, the new codes and reason)
UNIT        presence audience (CA-1); wire_fact; delta diff/apply and golden delta cases (CA-9)
INTEGRATION server sockets with stub seams: queues, order, lagged, acted_through, cursor (CA-4, -5, -7, -12)
REAL BINARY perceived = offline (CA-2), record-time judgement (CA-3), order (CA-6), INV-9 (CA-11),
            long-save resume (CA-13), deltas (CA-10); every existing CLI acceptance test
MEASUREMENT CP-C1 (CA-8): the one Gate-2-like run whose numbers decide DEP-15
FAR SIDE    Godot: perceived, events, acted_through, resume with cursor, deltas (CA-15)
REAL RUN    both 300-day digests (CA-14)
GATE 1      NOT REQUIRED — no language model
CI          the PR's CI on the exact final head is canonical; no local full suite besides C-C10's
```

## 17.9 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-SC1 | The live fold seeded from components and the offline fold from facts disagree (S10 A-1 false), so live ≠ offline. | C-C3's fold-equals-components check before any wiring; CA-2 end to end. A disagreement is a stop (it would be a presence defect). |
| R-SC2 | A full-log resume is slow on long saves, and the live backlog overflows while the backfill runs. | Backfill off the world thread; `perceived_backlog` 4096; CA-13 measures a 300-day save; a snapshot-seeded fold is the recorded optimization (QS11C-8). |
| R-SC3 | Market-town's street is one large `Place`: every line spoken on it reaches everyone on it. | It is `Visibility::Place` as the packs declare it; a hearing range is a later perception refinement behind the same seam (QS11C-6, operator-material). |
| R-SC4 | Per-frame events plus 10 Hz whole frames raise bandwidth before deltas land. | CP-C1 measures with events present; keyframes-only is still a conforming outcome. |
| R-SC5 | The serde round trip in `wire_fact` costs CPU per admitted fact per subscriber. | Rendered once per fact (cached on the fact as it is fanned out), not per subscriber; CP-B4's tick bound re-measured in C-C10. |
| R-SC6 | S11-D lands first and edits the same hunks (`runtime.rs`, `session.rs`, `protocol.rs`, the module). | §19's ownership and merge rule; evidence regenerated, never hand-merged. |
| R-SC7 | The Godot applier diverges from Rust's. | Shared golden delta cases (CA-9) read by both; the live run's invariants (CA-15). |

## 17.10 Questions

**[OPERATOR]** marks an operator-material question; the rest the primary session may rule.

| ID | Question | Recommendation |
| --- | --- | --- |
| QS11C-1 | Ruling 2 delivers both per-frame `events` and the `perceived` stream; S10 §3.3.4 planned `events` empty and IC-9 "transcripts byte-identical". Amend S10's IC-9 to "300-day digests unchanged" and S10 §3.3.4 accordingly? | **Yes** — ruling 2 already decided it; ARC-43 records the supersession; S10's text is edited by its owner at its next PR. |
| QS11C-2 | An ephemeral world serves `perceived` only from the join on (`cursor_unavailable` for older cursors), not "from its recent window" (S10 R-S11-2). | **Yes**: a re-foldable window is a second log; S10's milestone runs on persisted worlds. |
| QS11C-3 | A fact's payload reaches the wire by a serde round trip, `null` if not JSON; propose `EventEnvelope::map_payload` as a later contract PR? | **Yes, as recorded**; I-1 holds now; the contract helper is reviewed on its own when a second caller needs it. |
| QS11C-4 | In-server controllers get no events. | **Yes** (SD-C9); revisit when a hosted controller reads facts. |
| QS11C-5 | All three delta outcomes, including the RFC 6902 shape, specified now so C-C7's measurement decides without a re-freeze. | **Yes.** |
| **QS11C-6 [OPERATOR]** | Overhearing is place-level: a player on market-town's street sees every line said anywhere on the street, and NPC memory (S10) follows it. Accept for MVP-0? | **Accept**; a hearing range is a perception refinement behind `EventPerception` with no server change. Product-visible, so the operator's. |
| QS11C-7 | Backlogs: 256 facts per frame queue, 4096 perceived. | **Yes**, both in `HostConfig`; CA-5 uses a small value to test the mechanism. |
| QS11C-8 | A resume reads the whole fact log (≈373 000 facts at 300 days); seed the fold from a snapshot later? | **Accept now**, measured in CA-13; the snapshot seed is an optimization when measured necessary. |
| QS11C-9 | `lagged` is both a refusal code (the ruling's word) and a closing reason, as `unauthorized` is. | **Yes**, consistent with S11-A's pattern. |

**Rulings (freeze, 2026-10-08):**
- every question above is accepted as recommended;
- QS11C-6 was accepted by the operator, with a hearing range to come later as a World Interaction List
  rule.

## 17.11 Execution contract (frozen 2026-10-08)

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-C — facts in observations, the perceived stream,
                    acted_through, deltas (PR number assigned at freeze)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §17; evidence §17.12; deviations §17.13
RELATED / BINDING   this file §§4.7, 4.8, 5, 6, 7.3, 7.8, 8, 9.3, 15, 16, 19; server/PROTOCOL.md rev 2;
                    overall.md rulings 1, 2, 4, 6, 9, 10 and QIL-8; step-17 §§3.3, 10, 11.1;
                    docs/DECISIONS.md ARC-23, ARC-25, ARC-28, ARC-40, ARC-41, ARC-42; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after #83 (S11-B) has merged (precondition); branch mvp0/pr-s11c-perception;
                    worktree /Users/yuema137/mineworld-worktrees/impl-s11c, held by one session only
APPROVED SCOPE      §17.6; C-C1 … C-C10 with C-C3b; SD-C1 … SD-C14 as ruled; §17.14 (all platforms)
FROZEN INVARIANTS   I-1 (no kernel/contracts/persistence diff); I-6 (digests = E-SC0; observe and run.rs
                    untouched); I-7 (facts only through EventPerception; SystemInternal to nobody);
                    I-9 (no pack in server/); I-10 (if deltas ship); I-11 (no history read or wait on the
                    world thread); I-12 (ac13, ac15 green); every existing module name and call valid
SEQUENCE            E-SC0 → C-C1 → C-C3 → C-C3b (these three may precede S11-B's merge) → [S11-B merged]
                    → C-C2 → C-C4 → C-C5 → C-C6 → C-C7 → C-C8 → C-C9 → C-C10; each commit pushed when
                    coherent
COMMANDS            as S11-B's contract: cargo (fmt, check, clippy -D warnings, test, run, info), git,
                    gh (no merge), python3 scripts/*, the repository's Godot and run.sh scripts, and
                    `cargo check --target x86_64-pc-windows-msvc` if that target is installed (§17.14)
VALIDATION BUDGET   unit/integration/static unrestricted; CP-C1 at most two 60 s runs; CA-13 once (≤ 10
                    min with the 300-day save); 300-day digests at most four; Godot one window at a time,
                    each mode at most three runs; one full gate; about two hours; real-model NOT REQUIRED
LIVE DOCUMENTATION  §17 checkboxes; §17.12; §17.13
HANDOFF             .structured-coding/plans/mvp0/handoff-s11c.md
ENDPOINT AUTHORITY
  implementation + local validation   authorized (the primary session's freeze, 2026-10-08)
  semantic commits, branch push       authorized
  PR creation / update                authorized
  CI repair to review readiness       authorized
  merge                               operator only; never inherited
NORMAL STOP         PR S11-C READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       a needed kernel/contract/persistence edit; R-SC1 (fold ≠ components); a digest change;
                    a path outside §17.6; CP-B4's bound failing because of S11-C's work
```

## 17.12 Evidence ledger

```text
(empty until the freeze)
```

## 17.13 Deviations and discoveries

```text
(empty until the freeze)
```

## 17.14 macOS, Linux and Windows (operator requirement, 2026-10-08)

The audit found that CI runs only on `ubuntu-24.04` (`.github/workflows/ci.yml`). The test support
interrupts a server with `sh -c 'kill -INT <pid>'`, from S11-B's D-SB13, which is Unix only. The
requirement binds in four ways:

- **No platform assumption in product code.**
  - Paths use `std::path`.
  - Nothing in S11-C's server or CLI code signals, forks or reads `/proc`.
  - `spawn_blocking` and tokio are portable as used.
- **The resume history read on Windows.** `SavedHistory` opens its own connection for each backfill
  (`SqliteBackend::open`) and drops it before returning. No handle outlives the read, so on Windows a
  save directory can still be removed or renamed once the server stops.
  - The second reader runs beside the world thread's writer in WAL mode. SQLite arbitrates that on
    Windows as on Unix, under the existing `busy_timeout` of 5 s.
  - A read that still fails (`SQLITE_BUSY`, a sharing violation) is answered `cursor_unavailable`, with
    the cause in `detail`. The connection stays and may rejoin. It never stalls the world.
  - `mineworld perceived` likewise opens, reads and closes.
- **Tests that use signals.**
  - Wherever the property under test is not a graceful stop, a test ends a server with `Child::kill`,
    which is portable (TerminateProcess on Windows). Every commit is durable before anyone is told, so a
    kill loses nothing a test reads.
  - CA-2 and CA-6 read the save after a kill, never after an interrupt.
  - CA-13 needs the shutdown statistics line, so it needs a graceful stop. Its interrupt helper is
    `#[cfg(unix)]`, and **its Windows path is owned by S13**: a graceful-stop helper for the Windows
    lane (`GenerateConsoleCtrlEvent` or an equivalent), recorded as R-S13-W1.
  - No other S11-C test needs a signal.
- **Evidence of portability.**
  - C-C10 runs `cargo check -p mineworld-server -p mineworld-cli --target x86_64-pc-windows-msvc` if
    that target is installed. If it is not, the check is recorded as INCONCLUSIVE, with the owner below.
  - The Godot far side (`run.sh`, bash) runs on macOS here and on Linux in CI.
  - **A Windows and macOS CI matrix is S13's** (proposed requirement R-S13-W1: build, unit and server
    tests on `windows-latest` and `macos-latest`). Until it exists, Windows behaviour is designed and
    compiled, not exercised, and this PR's handoff says so.

---

# 18. PR S11-D — the admin surface and the host clock routes (full design)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session`. Superseded: `PR DESIGN — READY FOR FREEZE
REVIEW`. A fresh implementation session executes it under §18.11, once S11-B (#83) has merged (D-C1
excepted).

**Freeze record (2026-10-08).** Relayed by the coordinator:
- **Operator.** QS11D-3 accepted as a recorded limitation: there is no ban list; to ban someone, rotate
  the invite (restart the server with a new `--invite`). D-SB12: CP-B4's bound is p99 tick ≤ 50 ms with
  the maximum reported.
- **Primary session.** QS11D-1: a live scale change is TW-c's, and S11-D answers `409
  time_scale_fixed`. QS11D-2 and QS11D-4 … QS11D-8 accepted as recommended. QS11D-6: ruling 4 is amended
  so that S11-D adds the one `clock` reader to the module. QS11D-7: S11-D's record is **ARC-44**, which
  replaces the placeholder record id throughout §§18–19.
- **New binding requirement (operator):** macOS, Linux and Windows must all be supported. It is applied in
  §18.14 and SD-D13.

**Author:** the S11-C/D planning agent, 2026-10-08, worktree `plan-s11cd`, branch `plan/s11-cd`.
**Binding parents:** this file §§4.1, 4.9, 5, 6 (I-4, I-5), 7.2, 7.6, 7.7, 8, 9.4, 11.4; §15 (S11-A);
§16 (S11-B, PR #83, **not merged**); `step-19-time-weather.md` §§4.1–4.3, 7.1–7.4, 10 (INV-TW-2, -8, -9),
14.1 (QTW-2: full pause; QTW-3: the routes inside S11-D; QTW-13: wall cadence), 15.3 (the I-4
clarification); `overall.md` rulings 1, 6, 9, 10. Evidence `E-SD<n>` in §18.12, deviations `D-SD<n>` in
§18.13. Placeholders: commits `D-C0 …`, decisions `SD-D1 …`, acceptance `DA-1 …`, questions `QS11D-1 …`,
and one decision record, numbered **ARC-44** at the freeze (S11's reserve).

## 18.1 Identity, base, scope, preconditions

```text
PR            S11-D — the admin surface and the host clock routes (S11, fourth of five; parallel with
              S11-C, §19)
base          main after S11-B (#83) merges; designed against main @ 77a8717 + mvp0/pr-s11b-seats @ 7811e06
branch        mvp0/pr-s11d-admin (proposed), its own worktree, one session
scope         HTTP under /admin, bearer admin token, mounted only when a token is configured:
                GET  /admin/sessions                   every seated connection
                GET  /admin/seats                      every seat's binding
                POST /admin/sessions/{session}/kick    closing {kicked}; the seat returns to its default
                POST /admin/seats/{seat}/release       whoever holds it is closed; the seat returns
                GET  /admin/clock                      { at, time_scale, paused }
                POST /admin/clock                      { "paused": bool } — pause and resume
              the host clock's pause (HostClock segments); the `clock` server frame; WorldSummary.paused;
              the refusal `paused`; the rule that only the admin-token holder changes time;
              --admin-token / MINEWORLD_ADMIN_TOKEN; the module's `clock` reader
not in scope  a live time-scale change (TW-c, QS11D-1); the host journal, world.yaml hosting.time_scale,
              --cadence and FX-24 (TW-c); bans or per-player identity (QS11D-3); enabling or disabling
              a system (QS11-10); a WebSocket admin frame (§4.9); any world-state change (I-4)
```

**Preconditions (unmerged code this design depends on).**

| Precondition | What S11-D uses | Where on the S11-B branch |
| --- | --- | --- |
| P-D1 `SeatTable` with `Connected { subscription, session, resume }` and `Held` | kick finds a session's seat; release rebinds a seat's default; the seats report | `server/src/seats.rs` l. 61–79 (`session` is `#[allow(dead_code)]` "for the admin surface (S11-D)") |
| P-D2 `release(subscription, reason)` and the session's `released` branch | a kicked connection is told `closing {kicked}` exactly as a taken-over one is told `taken_over` | `runtime.rs` l. 237; `session.rs` l. 128, 260 |
| P-D3 `HostClock { epoch, started, scale }` in `runtime/world.rs` | pause re-anchors it (S11-B's D-SB5 says how) | `runtime/world.rs` l. 27–51 |
| P-D4 `tick` and `consult` order | pause skips consults and advances | `runtime.rs` l. 342–391 |
| P-D5 `ClosingReason::Kicked` defined, never sent | sent from here | `protocol/connection.rs` l. 80 |
| P-D6 `serve.rs` and `ServeRequest` | `--admin-token` is one field | `tools/cli/src/serve.rs` l. 30–53 |
| P-D7 CP-B4's ruling (D-SB12) | `hosted_town` is a regression test here | §16.11, operator |

## 18.2 Audit anchors

| File / symbol | Finding | Consequence |
| --- | --- | --- |
| **B** `server/src/app.rs` (124) `router(host, admission)`, `serve`, `serve_with_shutdown`; `Hosting { host, admission, sessions }` | Three routes; `SessionId`s allocated by an `AtomicU64`; no admin. | Routes are merged in only with a token; the three public signatures accept `impl Into<Access>` so every existing `router(host, admission)` call compiles unchanged (SD-D2). |
| **B** `server/src/session.rs` `run` l. 90 | The nickname exists only in the session task (S11-A SD-A8: the world thread never holds it). | `GET /admin/sessions` reads a transport-side registry the session writes, never the world thread (SD-D5). |
| **B** `server/src/runtime.rs` `submit` l. 257, `tick` l. 342, `consult` l. 362, `summary` l. 440, 492 lines | One authority path; the tick expires holds, consults, advances, sweeps; `summary` builds `WorldSummary`. At the trigger. | Pause is checked in `submit` and `tick` (SD-D7); admin commands go to a new `runtime/control.rs`; `summary` and `first_binding` move to `runtime/status.rs` first (SD-D11). |
| **B** `runtime/world.rs` `HostClock::now` | `epoch + elapsed_ms × scale / 1000`; no pause, no re-anchor. | Becomes a segment: `(world_anchor, wall_anchor, scale, paused)`, `now_at(Instant)` injectable (SD-D6). |
| **B** `server/src/seats.rs` `SeatTable::{join, depart, expire, due}` | Pure transitions over injected time; no kick or release. | Gains `kick(session)`, `release(seat)`, `report(now)` (SD-D4). |
| **B** `server/src/admission.rs` `InviteToken::given` l. 70, `ct_eq` (subtle), `UNAUTHORIZED_DELAY` l. 41 | The operator-token rules (8–128 printable ASCII, no whitespace), constant-time comparison and the 500 ms delay exist. | `AdminToken` reuses them; no new dependency (DEP-14 covers it) (SD-D3). |
| **B** `server/src/protocol/summary.rs` `WorldSummary` l. 92 | `time_scale` present; no `paused`. | Gains `paused: bool` (SD-D8). |
| **B** `server/PROTOCOL.md` §5.6 (`kicked` "S11-D"), §9 ("admin frames on the socket … admin is HTTP (S11-D)"), §10 last row | The landing table names S11-D's rows. | §11 (new) specifies the surface; §5.9 the `clock` frame. |
| `step-19-time-weather.md` §7.2–§7.4, §14.1 QTW-2, QTW-3 | Full pause: the clock stops, no Process wakes, hosted controllers are not consulted, client requests refused `paused`, observers stay connected. Routes inside S11-D. `clock { at, time_scale, paused }` after `welcome` and on every change. Host-only via the admin token. | All in scope except a live scale change, whose hosted rescheduling and host journal are TW-c's (QS11D-1). |
| **B** `tools/cli/src/hosted.rs` `ReactiveSeat::bound(at, scale, …)`, `PacedSeat::new(pacing, …)` | Adapters capture `time_scale` at construction (QTW-13). | A live scale change would leave every hosted cadence wrong until TW-c's seam change — the reason QS11D-1 recommends leaving it to TW-c. A pause needs no adapter change: with the clock frozen nothing is due, and on resume each pending instant is the same wall distance away. |
| `docs/NETWORKING.md` §3, §5 | Admin is on the HTTP control plane; §5 lists "admin command (authorized clients only)" among client→server messages. | HTTP, not a frame (§4.9); §13's proposed `NETWORKING.md` §5 note stays S11-E's. |

## 18.3 Design decisions (SD-D1 … SD-D12)

| ID | Decision | Why |
| --- | --- | --- |
| **SD-D1** | **`server/src/admin.rs`**: the routes, the bearer check, JSON bodies typed per route (`deny_unknown_fields` on every request body), errors as `{ "error": <code>, "detail": … }` with codes `unauthorized` (401), `unknown_session` / `unknown_seat` (404), `malformed` (400), `time_scale_fixed` (409, QS11D-1). Handlers only send commands to the world thread or read the session registry; none touches a `World`. | §4.9, §7.6. One module; removable by not mounting it (§8.1). |
| **SD-D2** | **Mounted only with a token**: `app::Access { admission, admin: Option<AdminToken> }` with `From<Admission> for Access` (no admin); `router`/`serve`/`serve_with_shutdown` take `impl Into<Access>`. Without a token no `/admin` path exists and every one answers 404 (axum's fallback). | §11.4 ("no admin routes if absent"); existing call sites unchanged. |
| **SD-D3** | **The bearer check**: `Authorization: Bearer <token>` read from `axum`'s `HeaderMap` (about 15 lines), compared with `subtle` through admission's `ct_eq`; any failure — header absent, another scheme, wrong token, a token in the query string (never read) — sleeps until 500 ms after the request arrived, then answers 401. `AdminToken` lives in `admission.rs` with `InviteToken::given`'s rules, a redacted `Debug`, no `Serialize`, no `Display`. `axum-extra`'s `TypedHeader` is not adopted: it adds `axum-extra` and `headers` to read one header (§7.2's comparison, closed here). | §7.2, §7.7, I-5. The delay is per request, in the handler's task, never on the world thread. |
| **SD-D4** | **`SeatTable` gains** `kick(session) -> Option<(SubscriptionId, EntityKey)>` (a `Connected` seat bound to that session returns to its default **with no hold**), `release(seat) -> Released` (`Connected` → default, the connection to close; `Held` → default, its secret dies; `Free`/`Hosted` → unchanged, `released: false`), and `report(now: Instant) -> Vec<SeatReport>` (`free`, `hosted`, `connected { session }`, `held { seconds_left }`). The runtime closes a displaced connection with `release(subscription, ClosingReason::Kicked)`. A kicked or released connection's resume is dead: the binding it named no longer exists. | §4.2's transitions plus the two §4.9 adds; the one writer of bindings stays the world thread (I-3). |
| **SD-D5** | **The session registry** (`server/src/admin/registry.rs`): a transport-side map `SessionId → { nickname, seat, observer, connected_at (wall, Unix seconds), seq (AtomicU64) }`, written by the session task at `welcome` and removed when the session ends; `observations_dropped` per session comes from the world thread (a counter on each subscriber, keyed by session). `GET /admin/sessions` merges the two. | The nickname never reaches the world thread (SD-A8, I-5); the drop count is only known there. |
| **SD-D6** | **The host clock pauses.** `HostClock` holds a segment `(world_anchor, wall_anchor, scale, paused)`: running, `now = world_anchor + ⌊elapsed_ms × scale / 1000⌋`; paused, `now = world_anchor`. `pause()` sets `world_anchor := now, paused := true`; `resume()` sets `wall_anchor := Instant::now(), paused := false`. Every method takes the wall instant as an argument (`now_at(Instant)`) so the property test injects it. Monotonic and jump-free by construction (INV-TW-8). | S19 §4.2 option A, the form S11-B's D-SB5 anticipated. Kernel untouched: the kernel's clock still moves only by advance and dispatch. |
| **SD-D7** | **What pause means** (QTW-2, full pause): while paused the tick still expires holds (wall time) and sweeps (observers stay connected, frames keep flowing), but does not consult hosted controllers and does not advance; `submit` refuses every session request with `refused { code: "paused" }` before allocating an `ActionId` (nothing journaled); joins, leaves, resumes, takeovers, kicks and releases still work (host state). | S19 §7.2. A consult skipped while paused is not owed afterwards: its instant is not passed while paused, so it falls due after resume at the same wall distance. |
| **SD-D8** | **Telling clients.** `WorldSummary.paused: bool` (in `/status` and `welcome.world`; public: pacing, not state). A server frame `clock { at, time_scale, paused }`, sent right after `welcome` and on every pause and resume, through a `tokio::sync::watch` channel the world thread writes and every session reads — newest wins, never dropped, never waited on. | S19 §7.4. A `watch` has exactly the clock's semantics; the observation channel's `try_send` could drop a clock change. |
| **SD-D9** | **Only the host changes time**: there is no client frame for it (a socket frame `pause`, `clock`, `kick`, `release` is `unknown_frame`); the only way is `POST /admin/clock` with the admin token. "The host" is whoever holds that token: the operator, or a single-player launcher that generated one for its own client (S19 §7.3; TW-e). An invite holder is not the host. | INV-9 (the client vocabulary is closed); S19 §7.3. |
| **SD-D10** | **`POST /admin/clock`** accepts `{ "paused": bool }` (other fields refused, `malformed`); a body with `time_scale` is answered `409 time_scale_fixed` until TW-c lands live scaling (QS11D-1); no field can name an instant, so no route moves the clock to an instant (the I-4 clarification of S19 §15.3). Repeating the current state is `200` and changes nothing. `GET /admin/clock` answers `{ at, time_scale, paused }`. | S19 §7.3, §15.3; "may omit, never redefine" (ARC-41) for `time_scale`. |
| **SD-D11** | **Size**: `summary` and `first_binding` move from `runtime.rs` to `runtime/status.rs` (pure move, D-C2); admin and clock command handling lives in `runtime/control.rs`. | `runtime.rs` is 492 lines; §19's file ownership (S11-C moves a different part). |
| **SD-D12** | **CLI**: `--admin-token TOKEN` with `env = "MINEWORLD_ADMIN_TOKEN"`, `hide_env_values`; an illegal token, or one equal to the invite, stops the server with a message that echoes neither; with a token the server prints `[mineworld] admin surface: http://<address>/admin (bearer token as given)`; without one, `[mineworld] no admin surface (no --admin-token)`. The token is never printed. | §11.4's frozen CLI contract; I-5; an admin token equal to the invite would make every player the host. |
| **SD-D13** | **Stopping the server on every platform** (operator requirement, §18.14). The shutdown future in `serve.rs` completes on the first of `tokio::signal::ctrl_c()` (Ctrl-C on Unix; Ctrl-C and Ctrl-Break on Windows) and, under `#[cfg(windows)]`, `tokio::signal::windows::ctrl_close()` and `ctrl_shutdown()` (closing the console window, or logging off) — **amended 2026-10-09 by ruling QW-1 (D-SD0): and `ctrl_break()`, because `ctrl_c()` on Windows does not catch Ctrl-Break**. Each leads to the same graceful path: stop accepting connections, `WorldHost::shutdown` (checkpoint, statistics), the hosted report. On a Windows close event the OS allows about 5 s; the checkpoint is one snapshot write, measured in E-SD. Under `#[cfg(unix)]` nothing changes. | S19 §7.5: closing the game saves and pauses, including on Windows, where a closed console sends no SIGINT. |

## 18.4 The surface, exactly (written into `PROTOCOL.md` §11 and §5.9 by D-C1)

```json
GET  /admin/sessions   → 200 { "sessions": [ { "session": "7", "nickname": "Yue", "seat": "visitor",
                                  "observer": "101", "connected_at": 1791441600, "seq": 312,
                                  "observations_dropped": 0 } ] }
GET  /admin/seats      → 200 { "seats": [ { "seat": "alice", "state": "hosted" },
                                  { "seat": "visitor", "state": "connected", "session": "7" },
                                  { "seat": "wanderer", "state": "held", "seconds_left": 21 },
                                  { "seat": "bob", "state": "free" } ] }
POST /admin/sessions/7/kick     → 200 { "session": "7", "seat": "visitor", "state": "hosted" }
POST /admin/seats/wanderer/release → 200 { "seat": "wanderer", "released": true, "state": "hosted" }
GET  /admin/clock      → 200 { "at": 4112, "time_scale": 1, "paused": false }
POST /admin/clock { "paused": true } → 200 { "at": 4112, "time_scale": 1, "paused": true }

server frame: { "t": "clock", "at": 4112, "time_scale": 1, "paused": true }
refusal:      { "t": "refused", "token": "c4", "code": "paused", "detail": "…" }
```

Every `/admin` request without the right bearer token: `401 { "error": "unauthorized" }`, no sooner than
500 ms after it arrived. No token configured: every `/admin` path is `404`. A request body field the
route does not define: `400 malformed`. `connected_at` is wall-clock Unix seconds (host state, never a
`WorldTime`). Identities are decimal strings (§7).

## 18.5 Acceptance (decided before measuring, `ARC-23`)

```text
DA-1  Absent without a token. `mineworld server worlds/social-cafe` with no --admin-token and no
      MINEWORLD_ADMIN_TOKEN: GET /admin/sessions, GET /admin/seats, POST …/kick, POST …/release,
      GET /admin/clock, POST /admin/clock each answer 404; stdout says "no admin surface".
      [tools/cli/tests/admin.rs]
      M-DA1  mount the routes unconditionally → 401 instead of 404; fails.

DA-2  Permission refusals. With --admin-token T (a fixed test token): each of — no Authorization
      header; "Bearer <the invite>"; "Bearer <T with its last character changed>"; "Bearer <T> "
      with a trailing space; "Basic <base64 of T>"; T only in the query string (?token=T); "bearer"
      lower-case scheme with a wrong token — on POST /admin/clock {"paused": true} and on
      POST /admin/sessions/<a live session>/kick, answers 401 {"error":"unauthorized"} no sooner than
      500 ms after the request was sent (the test's clock); afterwards GET /admin/clock (with T) says
      paused false and the targeted client is still seated and received no closing.
      [server/tests/admin.rs]
      M-DA2a the check accepts any bearer → fails. M-DA2b no delay → the 500 ms assertion fails.

DA-3  No admin route changes world state (I-4, S19 INV-TW-9). `mineworld server worlds/social-cafe
      --save DIR --admin-token T` (no --town), inside the routine-free first minutes, one idle seated
      client: every route called twice, in order sessions, seats, clock GET, pause, resume, release of a
      free seat, kick of the client. /status's revision is the same before and after; after a kill
      (`Child::kill`, portable, §18.14)
      `mineworld inspect DIR` reports exactly `mineworld validate`'s genesis fact count.
      [tools/cli/tests/admin.rs]
      M-DA3  kick also submits a no-op `move` as the kicked seat → revision moves; fails.

DA-4  Kick (CP-D). `mineworld server worlds/market-town --town --save DIR --admin-token T`: client A
      as visitor (nickname "Avery"), client B joins alice (took_over "hosted", nickname "Bea").
      GET /admin/sessions lists exactly A and B with those nicknames, seats and observers; GET
      /admin/seats shows visitor and alice connected with their sessions and every other seat hosted.
      Kicking B's session: B receives closing {kicked} and is closed; GET /admin/seats immediately
      after shows alice "hosted" (no hold); B's resume is then refused invalid_resume; B is absent
      from /admin/sessions. A kick of an unknown session → 404 unknown_session.
      [tools/cli/tests/admin.rs]
      M-DA4  kick departs as Departure::Dropped (a hold) → alice shows "held"; fails.

DA-5  Release. On DA-4's server: releasing visitor while A is connected → A receives closing {kicked},
      visitor returns to "hosted"; a client C seated as wanderer drops its socket (hold) → release
      wanderer → "hosted", and C's resume is refused invalid_resume; releasing a hosted seat → 200
      {"released": false} and GET /admin/seats still shows it "hosted"; release of an unknown seat →
      404 unknown_seat. [tools/cli/tests/admin.rs]
      M-DA5  release leaves a held seat held → C's resume is welcomed; fails.

DA-6  Pause (S19 §7.2, CP-TW-c's pause half). `market-town --town --save DIR --admin-token T`, two
      clients: POST /admin/clock {"paused": true} → 200 paused true; both clients receive
      clock {paused: true} within 1 wall s; /status paused true; three /status readings 2 wall s
      apart have equal `at` and equal revision (the town does not act: no hosted consult, no Process);
      a client submit → refused paused with its token, and the revision is unchanged; a dropped
      connection's hold still expires during the pause (wall time). POST {"paused": false} → clients
      receive clock {paused: false}; the first /status after resume has at ≥ the frozen at and
      ≤ frozen at + (wall seconds since resume + 1) × time_scale; the town acts again (revision moves
      within 30 wall s). [tools/cli/tests/admin.rs]
      M-DA6a submit not refused while paused → fails. M-DA6b consults not skipped → revision moves
      during the pause; fails. M-DA6c resume without re-anchoring the wall instant → `at` jumps by
      the paused duration; fails.

DA-7  The clock is monotonic and jump-free (unit, INV-TW-8). HostClock under 10 000 random steps of
      pause, resume and wall advances of 0–5000 ms (seeded): now never decreases; while paused it is
      constant; immediately after resume it equals the frozen value. [runtime/world.rs tests]
      Mutation: as M-DA6c → fails.

DA-8  Only the host changes time; INV-9. A seated client (invite holder) sends {"t":"pause"},
      {"t":"clock","paused":true}, {"t":"kick","session":"1"}, {"t":"release","seat":"alice"} → each
      unknown_frame; a join carrying "admin_token" → malformed_frame, no welcome. With T:
      POST /admin/clock {"at": 999999} → 400 malformed; {"paused": true, "at": 5} → 400 and not
      paused; {"time_scale": 12} → 409 time_scale_fixed and /status time_scale unchanged; {} → 400.
      Afterwards /status's at advances normally and the revision is unchanged by the table.
      [server/tests/admin.rs; the binary half in tools/cli/tests/admin.rs]
      M-DA8  ClockChange without deny_unknown_fields → the "at" body answers 200; fails.

DA-9  Secrets and names (I-5). With --admin-token T: T is in no stdout/stderr byte and no save byte;
      MINEWORLD_ADMIN_TOKEN=T behaves as the flag and is not echoed; `--help` names the variable and
      no value; --admin-token equal to --invite stops the server, non-zero, echoing neither; an
      illegal token ("short") likewise. A nickname appears in /admin/sessions and the holder's own
      welcome only — not in /status, another client's frames, or any log line (SA-5 extended).
      [tools/cli/tests/admin.rs, server_command.rs]

DA-10 The clock frame on the wire and from the far side. Golden server/tests/frames/clock.json; every
      welcome is followed by a clock frame before the first observation (server socket test).
      Headless Godot: `clients/protocol/run.sh admin` — a check joins, sees clock_changed(paused
      false), is paused by the script's POST and sees clock_changed(paused true), submits and is
      refused paused, is resumed, then is kicked and reports closing "kicked" before disconnected.

DA-11 Nothing else moved. Both 300-day seed-7 digests equal the base's (run.rs untouched; HostClock
      is not used by run); every existing test passes, edits to existing tests limited to welcome.json
      (world.paused) and transcripts that now carry clock frames, listed in §18.13; ac13, ac15,
      milestone_b, milestone_c, ac3_reconnect, ac5_takeover, hosted_town green.

DA-12 Scope and size. No diff under kernel/, contracts/, persistence/, systems/, worlds/, worldpack/,
      cognition/, sdk/, authoring/; no new dependency; runtime.rs, session.rs, host.rs, protocol.rs,
      app.rs, main.rs under 500 lines; the server names no pack and no controller crate.
```

## 18.6 Change set

```text
server/src/{admin.rs (new), admin/registry.rs (new), app.rs, admission.rs, seats.rs, seats/tests.rs,
            runtime.rs, runtime/world.rs, runtime/status.rs (new, moved), runtime/control.rs (new),
            host.rs, host/handles.rs, session.rs, protocol.rs, protocol/summary.rs, lib.rs}
server/{PROTOCOL.md, README.md}
server/tests/{admin.rs (new), frames.rs, frames/{clock.json (new), welcome.json}, support/mod.rs}
tools/cli/src/{main.rs, serve.rs}
tools/cli/tests/{admin.rs (new), server_command.rs, support/mod.rs}
clients/protocol/{mineworld/world_client.gd (the clock arm and signal only),
                  checks/admin_check.gd (new, with .uid), run.sh, ADOPTION.md, README.md,
                  evidence/* (regenerated)}
docs/{DECISIONS.md (ARC-44), MODULE_SPEC.md §8.1}
.structured-coding/plans/mvp0/{step-12-server.md §18, handoff-s11d.md}
```

## 18.7 Commit plan

### D-C0 — Design (this section) — docs only

- [x] Implementation: §18 and the pointer in §9.4, from the audit in §18.2.
- [x] Validation: Markdown only; `check_doc_headings`, `check_decision_ids`.
- [x] Review: S19's five asks (routes, frame, `paused` field, refusal, host-only rule) each map to a
  decision and a criterion (SD-D6 … SD-D10; DA-6, DA-8, DA-10); every guard names a mutation. Self-review;
  the freeze is pending.

### D-C1 — Specs before code: `PROTOCOL.md` §§5.9, 11; ARC-44; MODULE_SPEC §8.1

**Scope.** `PROTOCOL.md`: §1 (the `/admin` routes exist only with a token), §5 table (`clock`), new
§5.9 (`clock`), §5.5 (`paused`), §5.6 (`kicked` landed), §5.7 (`paused`), §9 (admin is HTTP, confirmed),
new §11 (the admin surface, §18.4), §10's rows (S11-D landed; `time_scale` in `POST /admin/clock` "from
TW-c"). `DECISIONS.md` **ARC-44** (the admin surface is HTTP behind a bearer token, mounted only when
configured, and changes no world state; pause is host pacing; the I-4 clarification of S19 §15.3;
rejected: socket frames, `tonic`, `axum-extra`; limitation: one admin token, no per-player bans).
`MODULE_SPEC.md` §8.1: `--admin-token`, `MINEWORLD_ADMIN_TOKEN`.

- [x] Implementation · [x] Validation: the two doc checks; §6/§6.2 untouched · [x] Review: every route
  and code in §18.4 present; S19's ARC-69 left to TW-c and cited, not pre-empted (E-SD1).

### D-C2 — Pure move: `runtime/status.rs`

**Scope.** `summary`, `first_binding` move; no behaviour change. **Depends on:** S11-B merged.

- [x] Implementation · [x] Validation: `cargo test -p mineworld-server` same names and counts; clippy
  · [x] Review: `runtime.rs` < 450 lines (E-SD2).

### D-C3 — `AdminToken`, the pausable `HostClock`, the seat table's kick, release and report

**Scope.** `admission.rs` (`AdminToken`), `runtime/world.rs` (SD-D6), `seats.rs` + `seats/tests.rs`
(SD-D4). **Validation (unit):** DA-7 with its mutation; `kick` on a connected seat returns to default
with no hold, on any other state is `None`; `release` on each of the four states; `report` per state;
`AdminToken` rules and redacted `Debug`.

- [x] Implementation · [x] Validation (E-SD3; DA-7's mutation in E-SD-M) · [x] Review: no world access
  in `seats.rs`; the clock never reads `Instant::now()` inside its arithmetic.

### D-C4 — The world thread and the session: control commands, pause, the clock frame, the registry

**Scope.** `runtime/control.rs` (handlers for seats report, kick, release, clock get/set), `runtime.rs`
(match arms; `paused` in `submit`; skip consult and advance in `tick` while paused), `host.rs`
(`Command::Control(ControlCommand, reply)`, `WorldHost` admin methods), `host/handles.rs` (`Seated` gains
the clock `watch::Receiver`), `session.rs` (clock frame after welcome and on change; registry
registration), `admin/registry.rs`, `protocol.rs` (`ServerFrame::Clock`, `RefusalCode::Paused`),
`protocol/summary.rs` (`paused`), golden `clock.json`, `welcome.json`. **Validation:** server suites;
the socket half of DA-10.

- [x] Implementation · [x] Validation (E-SD4) · [x] Review: I-11 (no wait on the world thread;
  `send_replace` never waits and never fails); kick/release rebind the seat and release the
  connection inside one world-thread command, so no consult or join can see two drivers.

### D-C5 — The HTTP routes

**Scope.** `admin.rs` (SD-D1, SD-D3, SD-D10), `app.rs` (SD-D2), `lib.rs`; `server/tests/admin.rs`: DA-2,
DA-8 (socket and HTTP halves), DA-5's in-process half. **Validation:** M-DA2a, M-DA2b, M-DA8.

- [x] Implementation · [x] Validation (E-SD4; mutations E-SD-M) · [x] Review: handlers hold no
  binding state; the one body type denies unknown fields; the delay is in the request's own task
  (a route_layer middleware in front of every handler).

### D-C6 — The CLI and the real-binary acceptance

**Scope.** `main.rs` (`--admin-token`), `serve.rs` (SD-D12), `tools/cli/tests/{admin.rs, support/mod.rs,
server_command.rs}`: DA-1, DA-3, DA-4, DA-5, DA-6, DA-9. **Validation:** M-DA1, M-DA3, M-DA4, M-DA5,
M-DA6a–c; every suite that starts the binary.

- [x] Implementation · [x] Validation (E-SD5, E-SD-M) · [x] Review: the token reaches nothing but the
  router (AdminToken has no Display/Serialize; the startup line prints none); `main.rs` 498 < 500.

### D-C7 — The Godot module's `clock` reader and the far side

**Scope.** `world_client.gd`: a `"clock"` arm, `signal clock_changed(at: int, time_scale: int, paused:
bool)`, `var paused`, `var time_scale` — nothing else (ruling 4 is amended for this one arm, QS11D-6);
`checks/admin_check.gd`; `run.sh admin`; `ADOPTION.md` §§2, 6; regenerated evidence (clock frames now
appear in transcripts). **Validation:** DA-10, one Godot window at a time; `run.sh evidence`,
`affordances`, `reconnect`, `./mineworld-slice --world --link` still pass.

- [x] Implementation · [x] Validation (E-SD6) · [x] Review: no rule in the module (the arm stores and
  emits); every existing name and call valid.

### D-C8 — Close

- [x] Implementation: `server/README.md` (admin, pause); ledger · [x] Validation: DA-11 digests; DA-12
  scope and sizes; one full gate on the final executable head (E-SD7) · [x] Review: DA-1 … DA-12 with
  evidence (E-SD1 … E-SD7); mutations planted, red, reverted, one equivalent survivor recorded
  (E-SD-M, M-DA6b); PR READY FOR OPERATOR REVIEW — DO NOT MERGE.

**E-SD0 (first action after the freeze):** the base's two 300-day digests and test counts.

## 18.8 Test ownership

```text
STATIC      fmt; clippy -D warnings (exhaustive matches over SeatReport, ControlCommand, the new code)
UNIT        HostClock property (DA-7); SeatTable kick/release/report; AdminToken
INTEGRATION server sockets + HTTP: permission refusals, INV-9 and body refusals, clock frame order
            (DA-2, DA-8, DA-10)
REAL BINARY absent routes, no world-state change, kick, release, pause, secrets (DA-1, -3, -4, -5, -6,
            -9); every existing CLI acceptance test
FAR SIDE    Godot admin check (DA-10)
REAL RUN    both 300-day digests (DA-11)
GATE 1      NOT REQUIRED
CI          the PR's CI on the exact final head is canonical
```

## 18.9 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-SD1 | A live scale change is wanted before TW-c, and S11-D's 409 reads as a missing feature. | QS11D-1 makes the split explicit; the field is specified and its landing row names TW-c. |
| R-SD2 | Pause interacts with holds: a player whose socket drops during a long pause loses the seat (holds are wall time). | Intended (a hold is about a network, QS11B-4); stated in `PROTOCOL.md` §4.2 and §11. |
| R-SD3 | Pause is not persisted until TW-c's host journal: a server restarted while paused starts running. | Stated; S19 §7.5 "closing saves and pauses" already holds because time does not pass while no host runs. |
| R-SD4 | Unlimited parallel wrong-token requests: each waits 500 ms but nothing caps their number. | `governor` recorded as the adopt route for public hosting (QS11-13); MVP-0 is LAN plus a gateway (QS11-14). |
| R-SD5 | The admin token in a launcher's process arguments is visible to other local users (`ps`). | The environment variable is the documented path for launchers (S19 §7.3, TW-e); `--admin-token` stays for operators. |
| R-SD6 | S11-C lands first and edits the same hunks. | §19. |

## 18.10 Questions

| ID | Question | Recommendation |
| --- | --- | --- |
| QS11D-1 | A live `time_scale` change through `POST /admin/clock`: in S11-D, or in TW-c? A live change needs every hosted adapter's cadence rescheduled (S19 §4.4) — a seam change — and the host journal. | **TW-c.** S11-D specifies the field and answers `409 time_scale_fixed` until TW-c lands it; pause needs neither the seam change nor the journal. Cross-lane, primary session. |
| QS11D-2 | Pause is checked in `submit` (sessions) and `tick` (hosted, Processes); kicks, joins and releases still work while paused. | **Yes** (S19 §7.2's full pause concerns the world, not host state). |
| **QS11D-3 [OPERATOR]** | A kicked player may rejoin at once with the invite: there is no ban without per-player identity. Accept for MVP-0? | **Accept**, recorded in ARC-44's limitations beside ARC-40's one trust level; a ban list waits for durable player identity. Security posture, so the operator's. |
| QS11D-4 | An admin token equal to the invite is refused at start. | **Yes.** |
| QS11D-5 | Releasing a free or hosted seat is a `200` no-op (`released: false`), not an error and not a controller rebuild. | **Yes.** |
| QS11D-6 | Ruling 4 says no PR but 16b, S11-A and S11-C edits the module; S11-D needs one `clock` arm so clients stop warning on a frame every server now sends. Amend ruling 4? | **Yes**, for that arm only; the alternative (S11-C carries D's arm) couples the two lanes. |
| QS11D-7 | Decision record: S11-D records ARC-44 (S11's reserve). S19's ARC-69 ("pause and scale are host commands") is left to TW-c and cited. | **Yes** — ruled: ARC-44. |
| QS11D-8 | `paused` is public in `/status` (pacing, like `time_scale`). | **Yes.** |

**Rulings (freeze, 2026-10-08):**
- every question above is accepted as recommended;
- QS11D-3 was accepted by the operator as a recorded limitation: to ban someone, rotate the invite;
- QS11D-6 amends ruling 4;
- QS11D-7 assigns ARC-44.

## 18.11 Execution contract (frozen 2026-10-08)

```text
PROJECT / PR        MVP-0 · Step 12 (S11) / PR S11-D — the admin surface and the host clock routes
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-12-server.md §18; evidence §18.12; deviations §18.13
RELATED / BINDING   this file §§4.9, 5, 6, 7.2, 7.6, 7.7, 9.4, 11.4, 15, 16, 19; server/PROTOCOL.md rev 2;
                    step-19-time-weather.md §§4.2, 7, 10, 14.1, 15.3; overall.md rulings 1, 6, 9, 10;
                    docs/DECISIONS.md ARC-23, ARC-40, ARC-41, DEP-14; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after #83 (S11-B) has merged (precondition); branch mvp0/pr-s11d-admin;
                    worktree /Users/yuema137/mineworld-worktrees/impl-s11d, held by one session only
APPROVED SCOPE      §18.6; D-C1 … D-C8; SD-D1 … SD-D13 as ruled; §18.14 (all platforms)
FROZEN INVARIANTS   I-1; I-2/ARC-40 (no admin action moves the revision); I-3 (one controller per seat);
                    I-4 (no route changes world state or names an instant); I-5 (admin token in no
                    save, frame, /status or log; nicknames only in /admin/sessions and the holder's
                    welcome); I-6 (digests = E-SD0); I-9; I-11 (handlers and the delay never on the
                    world thread); INV-TW-8 (the clock never decreases and never jumps)
SEQUENCE            E-SD0 → D-C1 (may precede S11-B's merge) → [S11-B merged] → D-C2 → D-C3 → D-C4 →
                    D-C5 → D-C6 (with SD-D13) → D-C7 → D-C8
COMMANDS            as S11-B's contract: cargo (fmt, check, clippy -D warnings, test, run, info), git,
                    gh (no merge), python3 scripts/*, the repository's Godot and run.sh scripts, and
                    `cargo check --target x86_64-pc-windows-msvc` if that target is installed (§18.14)
VALIDATION BUDGET   unit/integration/static unrestricted; real-binary tests as listed (each < 2 min);
                    300-day digests at most four; Godot one window at a time, at most three runs per
                    mode; one full gate; about 1.5 hours; real-model NOT REQUIRED
LIVE DOCUMENTATION  §18 checkboxes; §18.12; §18.13
HANDOFF             .structured-coding/plans/mvp0/handoff-s11d.md
ENDPOINT AUTHORITY
  implementation + local validation   authorized (the primary session's freeze, 2026-10-08)
  semantic commits, branch push       authorized
  PR creation / update                authorized
  CI repair to review readiness       authorized
  merge                               operator only; never inherited
NORMAL STOP         PR S11-D READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       any world-state change by an admin path; a kernel/contract/persistence edit; a new
                    dependency; a digest change; a path outside §18.6
```

## 18.12 Evidence ledger

```text
E-SD0 2026-10-09, base origin/main @ ec38570 (S11-B #83 merged at 15b05a9), worktree impl-s11d, machine
      load ~260 (other worktrees' jobs). Debug build `cargo build -p mineworld-cli`.
      `mineworld run worlds/social-cafe --headless --seed 7 --days 300` → 339 lines, sha-256 of every
        line but `wall` = ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b (= E-SB0).
      `mineworld run worlds/market-town --headless --seed 7 --days 300` → 355 lines, sha-256 =
        365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d (= E-SB0).
      Outputs /tmp/s11d/base-{cafe,market}.out; filter `grep -v '^wall' | shasum -a 256`.
      Audit anchors re-verified on ec38570: runtime.rs 492 lines (submit l. 257, tick l. 342, consult
      l. 362, summary l. 440, first_binding l. 482); seats.rs `Connected.session` still
      `#[allow(dead_code)]`; HostClock { epoch, started, scale } in runtime/world.rs; ClosingReason::Kicked
      defined, never sent; app.rs `router(host, admission)`; WorldSummary without `paused`. One anchor
      moved: tools/cli/src/main.rs is 501 lines at the base (S11-B left 486; IL-a and later merges
      added), already over DA-12's 500 — see D-SD1.
      Windows target: `rustup target list --installed` → aarch64-apple-darwin, x86_64-apple-darwin only.
E-SD1 D-C1. check_decision_ids → 72 ids, all distinct (ARC-44 placed after ARC-42, inside S11's
      region). check_doc_headings → 191 numbered sections, none duplicated. PROTOCOL.md hunks: §1
      (/admin route row), §4.1 (nickname shown on the admin surface), §5 table (`clock`), §5.5
      (`paused`), §5.6 (`kicked` landed), §5.7 (`paused`), §5.9 (new), §9, §10 (S11-D landed; TW-c row
      for `time_scale` in POST /admin/clock), §11 (new). §§5.8 and 6/6.2 untouched (S11-C's and 12e's).
      Every route and code of §18.4 is in §11.2; ARC-69 (S19) cited as TW-c's, not pre-empted.
E-SD2 D-C2 (pure move). `summary` and `first_binding` moved verbatim to server/src/runtime/status.rs;
      runtime.rs 492 → 436. clippy -D warnings (server, all targets) clean. `cargo test -p
      mineworld-server`: unit 38, frames 8, handshake 7, headless 4, seats 4, two_clients 9, doc 1 —
      all pass, the base's names and counts.
E-SD3 D-C3. admission.rs `AdminToken` (the invite's rules through two shared predicates; `admits`
      in constant time; `is_invite`; redacted Debug; no Serialize/Display; its own two error
      variants so a refusal names the admin token, never the invite). runtime/world.rs HostClock as a
      segment (world_anchor, wall_anchor, scale, paused) with `now_at(Instant)`, `pause_at`,
      `resume_at`; `now()` is the only Instant::now() and is outside the arithmetic. seats.rs `kick`,
      `release` → `Released`, `report`/`state_of` → `SeatReport { seat, SeatState }` (serde: state
      tag, session as a decimal string, held seconds rounded up); `Connected.session` no longer dead
      code. `cargo test -p mineworld-server --lib`: 43 passed (38 + AdminToken 1 + DA-7 1 + seats 3).
      DA-7 property test: 10 000 seeded LCG steps, scale 60, pause/resume/advance 0–5000 ms: never
      decreases, constant while paused, a resume reads the frozen value, never ahead of the running
      wall time.
E-SD4 D-C4 + D-C5 (one commit, D-SD3). runtime/control.rs (ControlCommand: Sessions, Seats, Kick,
      Release, Clock, Pause → ControlAnswer); runtime.rs: Subscriber gains session and a per-session
      drop count; Command::Control arm; `submit` refuses `paused` before any ActionId; `tick` while
      paused expires holds and sweeps but neither consults nor advances; a `watch::Sender<ClockState>`
      written with send_replace on each pause/resume. host.rs Command::Control + WorldHost::control;
      handles.rs Seated carries the watch receiver (subscribed at join, so a change after the
      welcome is never missed) and `clock_at_welcome()`; `streams()` returns a `Streams` struct.
      session.rs: welcome, then `clock` from the welcome's own summary, then the stream; a select
      branch on `clock.changed()`; registry registration (admin/registry.rs) after the welcome,
      dropped as soon as the stream ends (before the closing handshake); `seq` from the registry's
      counter. protocol: ServerFrame::Clock, RefusalCode::Paused, ClockState, WorldSummary.paused;
      golden clock.json (new), welcome.json (+paused). admin.rs: six routes, a route_layer bearer
      middleware (`Bearer ` prefix, exact bytes, constant time, sleep_until(arrived + 500 ms) then
      401), typed JSON errors, ClockChange { paused, time_scale } with deny_unknown_fields
      (time_scale → 409), GET /admin/sessions = registry ⋈ world counts. app.rs Access with
      From<Admission>; router/serve/serve_with_shutdown take impl Into<Access>; every existing call
      compiles unchanged. clippy -D warnings (server, all targets) clean.
      `cargo test -p mineworld-server --no-fail-fast`: unit 43, admin 5 (new), frames 9 (+clock),
      handshake 7, headless 4, seats 4, two_clients 9, doc 1 — all pass (load ~250).
      admin.rs: DA-2 (7 header cases × {POST /admin/clock, POST …/kick}: 401 {"error":
      "unauthorized"}, each ≥ 500 ms after sending; afterwards paused false and the client still
      seated, only observations), DA-8 (pause/clock/kick/release frames → unknown_frame; join with
      admin_token → malformed_frame; bodies {"at"}, {"paused","at"}, {} , not-json → 400,
      {"time_scale"} and {"paused","time_scale"} → 409; paused false, time_scale and revision
      unchanged; `at` still advances), DA-10 socket half (every welcome followed by a clock frame
      equal to the welcome's world before any observation; pause announced to both clients; submit
      refused `paused` with its token; `at` frozen over 1.2 s; resume answers the frozen `at` and is
      announced), DA-5 in-process (sessions listed with nicknames, seats, connected_at; kick → 200
      state free, closing{kicked}, gone from /admin/sessions at once; unknown 404, non-numeric
      400; release connected → released true + closing{kicked}; repeat → released false; unknown
      seat 404), and no token → every /admin path 404.
E-SD-M Mutations, each planted on the working tree, seen red, reverted (`git checkout`/re-edit,
      `git diff` clean of it):
      M-DA2a  bearer check admits any non-empty bearer → "the invite on /admin/clock … left: 200,
              right: 401"; red.
      M-DA2b  no sleep before 401 → "no Authorization header on /admin/clock was answered after
              93.542µs, under 500 ms"; red.
      M-DA8   ClockChange without deny_unknown_fields → `{"paused": true, "at": 5}` answered 200
              (the clock paused); red. As predicted, `{"at": 999999}` alone stays 400 under the
              mutation (no `paused`), so the second body carries the guard.
      Probe   DA-2's literal trailing space → admitted 200 (D-SD4).
      M-DA6c (DA-7, unit) resume_at does not re-anchor the wall instant → "a resume continues where
              it stopped … left: WorldTime(1523)"; red.
      M-DA1   routes mounted without a token (a never-matching token) → "GET /admin/sessions … left:
              401"; red.
      M-DA3   kick also advances the world a day (an admin path changing world state; the frozen
              text's no-op `move` needs a request the server cannot build without naming an
              action, so the same failure class is planted through `advance`) → "revision left: 2,
              right: 1"; red.
      M-DA4   kick departs as Departure::Dropped → alice "held", seconds_left 30; red.
      M-DA5   release leaves a held seat held → wanderer "held"; red.
      M-DA6a  submit not refused while paused → "a submit while paused was answered Result { … }";
              red.
      M-DA6b  consults and advance not skipped while paused → test PASSES: SURVIVED, equivalent in
              what any client can observe. With the host clock frozen, `now` stands still, so no
              slot becomes due and no deferral fires; the only window is the first paused tick, for
              slots due between the last running tick and the pause instant, which completes before
              any client can read a frozen revision. The skip is kept as the stated rule (QTW-2: "no
              hosted consult") and as defence for a future clock that is not frozen by pausing;
              recorded rather than over-tested.
E-SD5 D-C6. main.rs `--admin-token` (env MINEWORLD_ADMIN_TOKEN, hide_env_values) and serve.rs:
      AdminToken::given + is_invite before anything binds (refusals name `--admin-token`, echo
      neither secret); Access into serve_with_shutdown; the two startup lines of SD-D12;
      `stop_requested()` = ctrl_c, and under #[cfg(windows)] ctrl_break (QW-1), ctrl_close,
      ctrl_shutdown. Sizes main.rs 498 (D-SD1), serve.rs 371. clippy -D warnings (cli, all targets)
      clean. tools/cli/tests/admin.rs (6 tests) PASS: DA-1, DA-3 (+DA-8 binary half; inspect facts =
      validate's genesis count), DA-4 + DA-5 (one market-town --town server), DA-6 (clock frames
      after the welcome and within 1 s of the pause; at and revision equal over two 2 s waits;
      submit refused `paused` with token p1; a 3 s hold ended during the pause; resume's `at` within
      [frozen, frozen + (wall s + 1) × scale]; the town acted again within 30 s), DA-9 (token and
      nickname in no stdout/stderr line and no save byte; nickname only on /admin/sessions and the
      holder's welcome; MINEWORLD_ADMIN_TOKEN works and is not echoed; the invite is no admin
      token), SD-D13's #[cfg(unix)] graceful stop (exit 0, "[mineworld] stopping", "[world] ticks").
      server_command.rs DA-9 half PASS (--help names the variable, never its value; equal-to-invite
      and "short" stop the server non-zero, echoing neither). One DA-6 test defect fixed before it
      passed: the first `next_clock` read the welcome's own clock (paused false); the test now
      asserts that one first.
      `cargo test -p mineworld-cli --no-fail-fast` (every suite that starts the binary): exit 0, 28
      targets ok — ac13, ac15 (6), ac3_reconnect, ac5_takeover (2), milestone_b, milestone_c,
      hosted_town (120 s, p99 bound asserted), restart, server_command (10) among them; client_2d's
      8 are #[ignore] (Godot-gated) as at the base. Load ~250 throughout.
E-SD6 D-C7 (DA-10's far side). world_client.gd: one `"clock"` arm, `signal clock_changed(at,
      time_scale, paused)`, `var paused`, `var time_scale`, and the `closing` doc names `kicked` —
      nothing else (QS11D-6); every existing name and call unchanged. checks/admin_check.gd (+ .uid
      Godot generated): the host's HTTP calls are made by the check itself through Godot's
      HTTPRequest with the bearer token read from MINEWORLD_ADMIN_TOKEN (no curl, no argument);
      run.sh `admin` generates a token per run, exports it to the server and the check, and fails if
      the token reaches either evidence file. Godot 4.7.2, one headless window at a time:
        run 1  FAIL at "POST /admin/clock was sent: 44" (ERR_BUSY): the `refused paused` frame came
               back before the pause's HTTP answer had finished, and one HTTPRequest carries one
               request. A check defect, not a module or server one: requests now queue in the
               check and go one at a time.
        run 2  PASS — 8 checks, 0 failures: clock after welcome (paused false, scale 1); pause
               announced and held in `paused`; submit refused `paused` with its token "c1"; resume
               announced; kicked → closing "kicked" before disconnected.
      `run.sh evidence` exit 0, Alice answers in every transcript, no SCRIPT ERROR and no "unknown
      kind" warning in any log (the module now reads the clock frame every server sends);
      `run.sh affordances` PASS 0 failures; `run.sh reconnect` PASS 0 failures;
      `./mineworld-slice --world --link` → "all link checks pass". Evidence regenerated (server logs
      gain "[mineworld] no admin surface (no --admin-token)"; admin-social-cafe.log and
      server-admin.log new); the generated token is in no evidence file.
E-SD7 D-C8, full gate on the merged head 7e1188e (origin/main a30755e merged: docs only — the 13w
      plan; 13w's `interrupt` helper is not on main, so SD-D13's check keeps its #[cfg(unix)] gate,
      QW-3). Load ~250.
      fmt --check exit 0; clippy --workspace --all-targets -D warnings exit 0;
      `cargo test --workspace --no-fail-fast` exit 0: 174 test targets ok, 783 passed, 0 failed,
        9 ignored (client_2d's Godot-gated and one bodies_yard, as at the base).
      DA-11 digests (debug build, `grep -v '^wall' | shasum -a 256`):
        social-cafe seed 7, 300 days: 339 lines, ad49c723…c64b = E-SD0
        market-town seed 7, 300 days: 355 lines, 365b50e0…1d1d = E-SD0
        tools/cli/src/run.rs untouched (no diff from ec38570).
      check_decision_ids 72 distinct; check_doc_headings 191, none duplicated; check_scratch scan
        162 sources clean (2 exempt, as at the base); check_scratch left: none.
      `bash clients/protocol/run.sh evidence` exit 0 again on this head, no SCRIPT ERROR or
        unknown-kind warning; the regenerated logs are committed with this ledger.
      DA-12: no diff under kernel/, contracts/, persistence/, systems/, worlds/, worldpack/,
        cognition/, sdk/, authoring/; no Cargo.toml or Cargo.lock change; server/ names no pack and
        no controller crate. Sizes: runtime.rs 480, session.rs 442, host.rs 451, protocol.rs 372,
        app.rs 163, main.rs 498, admission.rs 498, admin.rs 263 — each under 500. Paths beyond
        §18.6: D-SD5 only.
      Windows: `cargo check --target x86_64-pc-windows-msvc` INCONCLUSIVE — the target is not
        installed here (E-SD0); SD-D13's #[cfg(windows)] branch (ctrl_c, ctrl_break, ctrl_close,
        ctrl_shutdown) is written and not compiled on this machine. Owner: S13 (13w's Windows CI
        lane, R-S13-W1).
```

## 18.13 Deviations and discoveries

```text
D-SD0 (ruling, SD-D13 amended) QW-1, relayed by the coordinator 2026-10-09 from S13's 13w design
      (finding F-13w-2): in tokio 1.53.1 on Windows `tokio::signal::ctrl_c()` catches Ctrl-C only,
      not Ctrl-Break; without a handler Ctrl-Break kills the server with 0xC000013A (no stopping
      line, no checkpoint, no statistics). SD-D13 assumed ctrl_c() caught both. Amended: under
      #[cfg(windows)] the shutdown future also completes on `tokio::signal::windows::ctrl_break()`,
      beside ctrl_close() and ctrl_shutdown(), all on the same graceful path. QW-3: whichever of
      S11-D and 13w lands second removes the #[cfg(unix)] gate on SD-D13's graceful-stop check; 13w's
      `interrupt` helper is not on main (origin/main @ ec38570 at this session's resume), so the gate
      stays.
D-SD1 (bounded) tools/cli/src/main.rs is 501 lines at the base, already over DA-12's < 500 before
      S11-D adds `--admin-token`. Resolved inside §18.6's paths: `saved_genesis`, used by
      `serve::persisted`, `replay` and run.rs, moves from main.rs to serve.rs, re-exported at the crate
      root by one `pub(crate) use` so run.rs keeps its `crate::saved_genesis` import untouched (I-6);
      the flag's `#[arg]` takes clap's default value name (`<ADMIN_TOKEN>`) so it fits one line.
      main.rs 501 → 498. tools/cli/tests/configure.rs's structural check (`fn replay(` →
      `verify(&backend` in main.rs) is unaffected.
D-SD2 (bounded) Existing tests edited because every welcome is now followed by a `clock` frame
      (DA-11's "transcripts that now carry clock frames"): each test file's own "next frame that is not
      an observation" helper now also skips `clock` — server/tests/{handshake.rs, seats.rs (and one
      match arm), two_clients.rs}; server/src/protocol/tests.rs gains `paused: false` in one
      WorldSummary literal (a compile requirement); frames.rs gains `paused` in its world() and the
      `clock` golden test. No assertion changed.
D-SD3 (bounded) Commit mapping: D-C4 and D-C5 land as one commit. The session registry is created
      by the router (app.rs) and handed to both sessions and admin routes, so the world-thread and
      session half cannot be exercised without the routes that read it.
D-SD4 (bounded) DA-2's case "Bearer <T> " with a trailing space cannot be told apart by any server:
      HTTP field values exclude leading and trailing whitespace (RFC 9110 §5.5), and hyper's parser
      strips it before axum's HeaderMap is built. Probed, not assumed (E-SD-M): with the literal case
      the request was admitted (200) because the bytes the server receives are exactly `Bearer <T>`.
      The case is exercised as "Bearer <T> x" — the token followed by whitespace and more — which is
      refused 401 after 500 ms. Not a weakening: the credential the client sent is the token itself.
D-SD5 (bounded) Paths beside §18.6's list, each a companion of a listed one or required by the
      frozen behaviour: server/src/protocol/tests.rs (one `paused: false` in a WorldSummary literal,
      to compile), server/tests/{handshake.rs, seats.rs, two_clients.rs} (D-SD2's helper edits — the
      clock frame every welcome is now followed by), clients/protocol/checks/admin_check.gd.uid
      (Godot's generated id for the listed check), clients/protocol/evidence/README.md (names the two
      new evidence files). As S11-B's D-SB11.
D-SD6 (bounded) The design's §18.6 names `server/tests/support/mod.rs`; nothing in it needed to
      change (the admin socket tests keep their HTTP helper in server/tests/admin.rs, the only file
      that uses it). Not edited.
```

## 18.14 macOS, Linux and Windows (operator requirement, 2026-10-08)

CI runs only on `ubuntu-24.04` (`.github/workflows/ci.yml`). The test support's `Server::interrupt` uses
`sh -c 'kill -INT'`, which is Unix only (S11-B D-SB13). Applied to S11-D:

- **The admin token on Windows, from the environment and the CLI.**
  - clap's `env` feature reads `MINEWORLD_ADMIN_TOKEN` through `std::env` on every platform.
  - The token rules (8–128 printable ASCII, no whitespace) avoid every character that cmd.exe or
    PowerShell would need to quote.
  - DA-9's environment and flag cases use `Command::env` and `Command::arg`, never a shell, so they run
    unchanged on Windows.
  - Launchers on every platform pass the token through the environment, not the command line (R-SD5).
- **Stopping on Windows**: SD-D13 covers Ctrl-C, Ctrl-Break and closing the console window, ahead of
  SIGINT. `HostClock` and the delays use `std::time::Instant` and tokio timers, which are portable.
- **Tests that use signals.**
  - DA-3, DA-4, DA-5 and DA-6 end their servers with `Child::kill`, which is portable (TerminateProcess
    on Windows), and read the save after the kill.
  - No S11-D test needs a graceful stop, with one exception: SD-D13's own graceful-stop check is
    `#[cfg(unix)]`. **Its Windows path is owned by S13** as proposed requirement R-S13-W1: a
    Windows/macOS CI matrix, and a Windows graceful-stop helper (`GenerateConsoleCtrlEvent`, or
    `windows::ctrl_close` exercised in a console test).
  - The same owner covers the existing `hosted_town` interrupt.
- **Evidence.**
  - D-C8 runs `cargo check -p mineworld-server -p mineworld-cli --target x86_64-pc-windows-msvc` if that
    target is installed, which is what compiles SD-D13's `#[cfg(windows)]` branch. If it is not
    installed, the check is recorded INCONCLUSIVE with S13 as the owner.
  - Until S13's matrix exists, Windows behaviour is designed and compiled, not exercised, and the
    handoff says so.

---

# 19. Running S11-C and S11-D in parallel: file ownership and merge order

Both start only after S11-B merges (their preconditions, §17.1 and §18.1), each in its own worktree on its
own branch, held by one session (`CLAUDE.md` §3.1). S11-C's C-C1 and C-C3, and S11-D's D-C1, are documents
or presence-only code and may start before S11-B merges.

## 19.1 Files each lane owns alone

```text
S11-C   systems/presence/src/audience.rs (+ lib.rs line); server/src/perception.rs;
        server/src/runtime/delivery.rs; server/src/protocol/{fact.rs, delta.rs};
        server/tests/{facts.rs, deltas.rs, frames/{perceived,delta,join,observation}.json, frames/deltas/};
        tools/cli/src/{perceive.rs, perceived.rs, history.rs}; tools/cli/tests/{facts.rs, perceived.rs,
        deltas.rs}; tools/cli/Cargo.toml (dev json-patch); clients/protocol/mineworld/{observation.gd,
        delta.gd}; clients/protocol/checks/{perceived_check.gd, delta_check.gd}; DECISIONS ARC-43, DEP-15
S11-D   server/src/{admin.rs, admin/registry.rs, app.rs, admission.rs, seats.rs, seats/tests.rs};
        server/src/runtime/{world.rs, status.rs, control.rs}; server/src/protocol/summary.rs;
        server/tests/{admin.rs, frames/{clock,welcome}.json}; tools/cli/tests/admin.rs;
        clients/protocol/checks/admin_check.gd; DECISIONS ARC-44
```

## 19.2 Files both edit, and the hunks each owns

| File | S11-C's hunks | S11-D's hunks |
| --- | --- | --- |
| `server/src/runtime.rs` | moves `Subscriber`/`sweep`/`release`/`depart` out (C-C2); `remember` fans out; `submit` sets `acted_through` | moves `summary`/`first_binding` out (D-C2); `run`'s match arms for control; the paused checks in `submit` and `tick` |
| `server/src/runtime/status.rs` (D's move) | `events_dropped` from the counter instead of `0` (one line, wherever `summary` lives at C's head) | created by the move; `paused` |
| `server/src/runtime/delivery.rs` (C's move) | created by the move; queues, order, `lagged` | `release(…, Kicked)` and per-session drop counts call into it (wherever `release` lives at D's head) |
| `server/src/host.rs`, `host/handles.rs` | `Command::Submit` gains the subscription; `JoinRequest.perceived`; `HostConfig` backlogs; `Streamed`, `Perceived.acted_through`, `Seated.head` | `Command::Control`; `WorldHost` admin methods; `Seated`'s clock receiver |
| `server/src/session.rs` | backfill, `perceived` frames, `lagged`, delta encoding, the submit's subscription | the clock frame and its select branch; registry registration |
| `server/src/protocol.rs`, `protocol/connection.rs` | `Perceived`, `Delta` frames; `acted_through`; `join.perceived`; `CursorUnavailable`, `Lagged`; reason `Lagged` | `Clock` frame; `Paused` |
| `server/src/lib.rs`, `server/README.md`, `server/tests/frames.rs`, `server/tests/support/mod.rs` | its re-exports, sections, frame files | its re-exports, sections, frame files |
| `server/PROTOCOL.md` | §§2, 5.2, 5.3, 5.5, 5.6, 5.8, 10 | §§1, 5 (table), 5.5, 5.6, 5.7, 5.9, 9, 10, 11 |
| `tools/cli/src/main.rs`, `serve.rs` | one `Perceived` variant and arm; `--keyframe-every`; event perception and history wiring | `--admin-token`; the token into `Access` |
| `tools/cli/tests/support/mod.rs`, `server_command.rs` | a perceived join helper; CA-11 | an admin request helper; DA-9's half |
| `clients/protocol/mineworld/world_client.gd` | `perceived`/`delta` arms, cursor, reconnect with cursor | the `clock` arm and signal only |
| `clients/protocol/{run.sh, ADOPTION.md, README.md, evidence/*}` | `run.sh perceived|deltas`; §§2, 3.4, 6 | `run.sh admin`; §§2, 6 |
| `docs/MODULE_SPEC.md` §8.1, `docs/DECISIONS.md` | `perceived` command; ARC-43, DEP-15 | `--admin-token`; ARC-44 |

## 19.3 Rules

- **No merge order is required.** Recommended: S11-D first if both are ready together, because S19's TW-c
  builds on its clock routes and it is the smaller diff.
- **The second to merge rebases** (merges `origin/main`), keeps the first's hunks, and re-runs its own
  validation's affected half; the shared table above is the review checklist for that merge.
- **Evidence is regenerated, never hand-merged**: `clients/protocol/evidence/*` and golden frames touched by
  both (`welcome.json` is D's, `observation.json` C's) are rebuilt by the second lane's own run.
- **Sizes are per lane and per head**: each lane's pure move keeps `runtime.rs` under 500 alone; together
  they leave room for both. `world_client.gd` (569 lines at S11-B) grows only by frame arms; new logic goes
  to new files (`delta.gd`).
- **Decision ids**: ARC-43 and DEP-15 are S11-C's (ruling 6), and ARC-44 is S11-D's (freeze). Neither
  lane writes the other's record.
- **Worktrees** (freeze): S11-C in `/Users/yuema137/mineworld-worktrees/impl-s11c` on
  `mvp0/pr-s11c-perception`; S11-D in `/Users/yuema137/mineworld-worktrees/impl-s11d` on
  `mvp0/pr-s11d-admin`. Each is held by one session.

