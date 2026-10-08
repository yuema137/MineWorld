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
