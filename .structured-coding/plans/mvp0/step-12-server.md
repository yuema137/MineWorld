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
      ≥ 3 accepted `move` requests, zero faults (/status), and `longest tick ≤ 50 ms` — half the 100 ms
      cadence. [tools/cli/tests/hosted_town.rs]
      Mutation M-SB6: consults never fire (next_consult always in the future) → accepted 0; fails.

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

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C3 — `SeatTable` (`seats.rs`) and `ResumeSecret`

**Scope.** SD-B1–SD-B3 and SD-B12 as pure code over an injected `Instant`. **Validation (unit):** every
transition of §4.2 and its refusals; `take_over` on `Connected` and `Held`; resume on `Held` and on a live
`Connected` (supersede); expiry at `until` and not before; `default` rebuilds through the factory;
`ResumeSecret` format and redacted `Debug`. **Review:** no world access in `seats.rs`; one writer.

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C4 — The `HostedController` seam and `RuleController::since`

**Scope.** `hosted.rs` (trait, `HostedAnswer`, the factory type, the due-consult order) and
`cognition/rule-controller` `since`. **Validation (unit):** consult order across seats and instants; a
`since(t)` controller does not answer a line heard at `t` and answers one at `t + 1`; `new()` unchanged
(existing tests untouched and passing); paced tests untouched.

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C5 — The runtime and the session: seats, holds, consults, time scale, revision 2's additions

**Scope.** `runtime.rs` (`submit_at`, join/departure through the table, consults and expiry in `tick`,
`time_scale`, `clients`), `host.rs`/`handles.rs` (`JoinRequest`, `Departure`, `Seated` fields,
`released`), `session.rs` (take_over, resume, the released branch, `Departure`), `protocol*` (SD-B4),
golden frames, `server/tests/seats.rs` (SB-2, SB-7), `handshake.rs` updated where S11-A asserted
`invalid_resume` for every resume. **Validation:** server suites; M-SB2, M-SB7. **Review:** I-11 (no wait
on the world thread; consults bounded); the session holds no binding state.

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C6 — The CLI: hosted adapters, flags, statistics; real-binary acceptance

**Scope.** `tools/cli/src/hosted.rs`, `serve.rs` wiring, `main.rs` flags, `agent.rs` deleted; tests
`ac5_takeover.rs` (SB-1, SB-4), `ac3_reconnect.rs` (SB-3, SB-9), `hosted_town.rs` (SB-6), `restart.rs`
(SB-5), `server_command.rs` (SB-8). **Validation:** those tests and every suite that starts the binary
(SB-11's first half); M-SB1, M-SB3, M-SB4, M-SB5, M-SB6, M-SB8. **Review:** the server names no controller
crate; statistics lines carry no secret.

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C7 — The Godot module: `take_over`, opt-in reconnect; the far side

**Scope.** SD-B14; `checks/reconnect_check.gd`; `run.sh reconnect` (port 0, own PID only, invite line kept
out of evidence); `ADOPTION.md` §2/§6; evidence regenerated. **Validation (one Godot window at a time):**
SB-10. **Review:** the module never logs the resume secret; every four-argument call unchanged.

- [ ] Implementation · [ ] Validation · [ ] Review

### B-C8 — Close: README, digests, scope, full gate, ledger, PR

- [ ] Implementation: `server/README.md` (`--town`, takeover); ledger · [ ] Validation: SB-11 digests,
  SB-12 scope and sizes, one full gate on the final executable head (background) · [ ] Review: SB-1 …
  SB-12 with evidence; the PR marked READY FOR OPERATOR REVIEW — DO NOT MERGE.

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
```

