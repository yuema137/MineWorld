# Step 12 — S11: Server and networking

**Effort:** `mvp0` · **Parent:** [`overall.md`](overall.md) §3 S11, §4, §7
**Lifecycle:** `DRAFT — awaiting the primary session's review`
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

