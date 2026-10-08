# Step 17 — Cognition: the language-model half of S10, and Milestone D

**Role:** step document for the part of **S10 — Cognition layer** that the operator deferred to MVP-1
on 2026-09-25, and for **Milestone D — LM-native persistent characters**. It records the requirement,
the audit, the design, the reuse comparison, the invariants, a PR split with integration checkpoints,
the risks, the requirements this step places on S11, and the open questions. It holds no frozen PR
design and authorizes no implementation (`CLAUDE.md` §3.1).
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S10, and the "Original scope, retained
for MVP-1" list), §4 (`AC-4`, `AC-10`), §7.
**Lifecycle:** `DRAFT — awaiting the primary session's review`.
**Base:** `main @ 0fd0be3`, branch `plan/s10-cognition`.
**Schedule (operator, 2026-10-08):** designed now, in parallel with S11 (server and protocol), S12
(2D), S13 (CI), S14 (3D) and Milestone E; **implemented after the clients**. §9 marks which PRs could
start earlier if the operator chooses, and which must wait.

**Parallel-safety rules this document was written under.** It is the only file this planning session
writes. It edits no spec, no `overall.md`, no `MVP_STATUS.md` and no `DECISIONS.md`; every edit those
need is *proposed* in §12. Decision identifiers are placeholders (`ARC-S10-a`, `DEP-S10-a`, …) that the
primary session numbers at freeze. Everything this step needs from the server or the protocol is
stated as a requirement on S11, with its exact shape, in §11.

## Why the file is `step-17`, and the step is S10

Step files are numbered in the order they were opened, not by the S-number of the step they plan
(`step-11-bodies.md` is S15). This is the seventeenth step document of the effort; the step it plans
is S10's language-model half.

## 0. The answer in brief

```text
Rust server (authoritative)                          Python cognition process (one per operator)
──────────────────────────                          ─────────────────────────────────────────────
kernel · systems · persistence                       one SeatSession per LM-driven seat
presence: who perceived which fact   ── perceived ─► memory   (subjective; derived only from what
  (one audience function, used live     events,         this seat perceived; L0–L3 with Event IDs)
   and when resuming from a cursor)     reliable,   ─► LMController
                                        resumable       routine: deterministic policy (no model)
perception: the 10 Hz observation   ── observation ─►   social: ModelBackend (provider-neutral)
                                                        budget gate · recorder (cassettes)
dispatch at the current state       ◄── submit ─────   ActionRequest built only from what the
  (revalidation is structural)          (same frame     observation offers, or `talk` by name
                                         a client uses)
```

1. **The process boundary is the public client protocol.** An LM-driven Person is a client of the
   server, joined to a seat over WebSocket like a 2D or 3D client, with the same `join`/`submit`
   frames, the same actor check and the same server-allocated identities. There is no second, more
   privileged path for cognition (`INV-9`, `ARC-S10-a`).
2. **The world revalidates every intent by construction.** Every `submit` is dispatched against the
   world's state at the moment it arrives, never against the observation the controller decided on.
   Cognition adds a pre-submit staleness check and records what each decision was based on (§3.6).
3. **Memory is subjective and derived only from perception.** A new, reliable, cursor-resumable
   stream of *perceived events* (facts whose `Visibility` admitted this observer, decided by one
   presence-owned audience function) is the only input to an NPC's memory, besides its own
   observations. The world never reads memory back (`INV-4`, `INV-13`, `ARC-S10-b`).
4. **Compression L0–L3 keeps Event IDs**, and is deterministic and model-free by default. A model may
   write prose summaries only through the recorder, so a summary is replayable (`AC-10`, `ARC-S10-c`).
5. **Providers are adapters behind one provider-neutral `ModelBackend`.** One OpenAI-compatible adapter
   covers Ollama's `/v1` endpoint and any compatible server. Provider names, model names, URLs and
   keys live only in the operator's cognition configuration, never in a World Pack and never in a
   contract (`AC-4`, `DEP-S10-a`).
6. **Every model call is recorded and replayable.** The key is a hash of the provider-neutral request,
   so a cassette recorded against one backend replays under any other. Core tests run against
   cassettes or a scripted backend and never reach a model (`ENGINEERING_STANDARDS.md` §24,
   `ARC-S10-e`).
7. **Natural speech changes no world determinism.** What Alice says is a `talk` payload. The world
   journals the request and records a `spoke` fact. A world replay re-executes the journal and never
   calls a model (`ARC-25`).
8. **Milestone D** is shown by one persisted world, with Alice on an LM-driven seat. A player speaks to
   her from the 2D client. The server and the cognition process are killed and restarted. The same
   player then walks up to her in the 3D client, and her reply draws on the 2D conversation. The test
   names identity (one world instance, one Alice `EntityId`) and provenance: the Event IDs of the 2D
   lines appear in the context her reply was decided from (§3.14).
9. **A world with every model removed stays valid.** `LMController` is one Controller Pack among
   others. Unplugging it leaves its seats to the Rust rule controllers, to a human, or empty, with no
   change to any World Pack, system, contract or client (§3.16).

---

# 1. The requirement

## 1.1 Milestone D, verbatim

From [`docs/HUMAN_REVIEW_QUEUE.md`](../../../docs/HUMAN_REVIEW_QUEUE.md), "Framework milestones":

> | **D** | LM-native persistent characters | speak to Alice in 2D, meet her in 3D, and she reacts
> consistently with what happened | ❌ |

The same document, on what Milestone D is for (its record of the operator's 2026-10-06 finding that
the dialogue did not read as natural language):

> **What this does not fix: the wording of Alice's reply.** "I remember you. You said "Hello! A
> coffee, please.". You are the first person to speak to me here." (and, once others have spoken,
> "Earlier, <name> said …") is the fixed template of the town's **rule controller**. The client now
> shows it cleanly, but cannot make it natural. Since S8 PR 10c the template names people rather than
> numbering them. Natural dialogue needs a language-model controller, which is Milestone D.

## 1.2 The acceptance criteria, verbatim

From [`docs/MVP.md`](../../../docs/MVP.md) §9:

> | **AC-4** | Cognition independence | Replacing the LM backend requires no change to the World Pack. |

> | **AC-10** | Bounded cognition context | After 100 simulated days, character history does not require
> feeding all historical events to a model; biography compression stays bounded while original event
> provenance is retained. |

`AC-15`, which Milestone D extends from "remembers" to "reacts consistently":

> | **AC-15** | **There is only one Alice** | A 2D client, a 3D client and an agent-driven Person are
> connected to **one running server** at the same time. Something a player does in the 2D client is
> visible in the 3D client, and an NPC carries the consequence forward: speak to Alice in 2D, then walk
> up to her in 3D, and Alice knows it happened. |

`MVP.md` §9.2, which names the path this step grows:

> Alice knowing *"Player A spoke to me three minutes ago about X"* is enough. Episodic memory,
> summarized biography, retrieval and forgetting are the MVP-1 evolution of this path, and
> `CORE_CONCEPTS.md` §5's distinction between world truth, biography and subjective memory is already
> the design they grow into.

## 1.3 The model-free rule, verbatim

[`overall.md`](overall.md) §3, S10's retained acceptance checkpoint:

> **Acceptance checkpoint:** swapping the model backend changes no World Pack (`AC-4`); after 100
> simulated days a character's context stays bounded while event provenance is retained (`AC-10`); the
> full core test suite still passes with no model reachable (§24).

[`ENGINEERING_STANDARDS.md`](../../../docs/ENGINEERING_STANDARDS.md) §24:

> Core integration tests should not require live external LM APIs. […] Separately, maintain optional
> live-model integration tests when useful. MineWorld correctness must not depend on whether an
> external provider responds.

[`VISION.md`](../../../docs/VISION.md) §1.1:

> **Runtime is LM-independent** | A running world must not require a model. Rule controllers, human
> players and deterministic systems stand alone, and a world with every model unplugged is still a
> valid MineWorld world.

## 1.4 The scope decisions this step sits between

- **2026-09-25 (operator).** MVP-0 delivers the controller abstraction with `HumanController` and
  `RuleController`, plus perception and `Observation` production. `LMController` over Ollama and one
  OpenAI-compatible endpoint, subjective memory and hierarchical biography compression move to MVP-1,
  and `AC-4` and `AC-10` are deferred with them (`overall.md` §3 S10).
- **2026-10-08 (operator).** S10's language-model half is **designed now, in parallel**, and
  implemented after the clients. This document is that design. It does not by itself reverse the
  09-25 decision: whether this work lands inside MVP-0 or opens MVP-1 is QS10-1 (§13), and is the
  operator's.

## 1.5 The operator's reuse and modularity directive (2026-10-08), verbatim

> 保持代码干净整洁，模块化，可插拔。遇到问题要积极寻找已有的开源方案，不要重复发明轮子，而且要对比多种实现方案，
> 不要找到一个就觉得万事大吉。当然如果没有现成的能满足我们的大目标的，那我们自己改进或者发明轮子也是可以的

As relayed in English: keep code clean, modular and pluggable; search for open-source solutions first;
compare several implementations instead of stopping at the first; build our own only if nothing
existing serves the goal. §4 is the comparison it asks for, one table per infrastructure piece; §3.16
is the modularity and pluggability section.

## 1.6 Binding constraints restated

Each is judged against the design below; none is relaxed by it.

```text
INV-1    Person ≠ Controller; rebinding a seat changes no Person state
INV-4    Memory ≠ Biography ≠ World Truth
INV-6    controllers submit requests; they never mutate
INV-9    the server is authoritative; every controller is a requester
INV-10   a controller cannot create an interaction; "I shoot Bob" is Unavailable
INV-13   controllers receive Observations, never global state; omniscience impossible by construction
INV-14   simulation semantics never depend on the model provider
ARCHITECTURE §9    the server never blocks on a controller; the world revalidates every intent
ARCHITECTURE §9.1  cost is first-class; the framework requires no paid API
ARCHITECTURE §9.2  credentials belong to the operator and never appear in a World Pack
MODULE_SPEC §5     a Controller Pack consumes Observations only, emits ActionIntents only, keeps
                   provider details behind the backend interface
ENGINEERING_STANDARDS §3     Python for LM integration and cognition policies
ENGINEERING_STANDARDS §4     cross-language communication through explicit contracts only
ENGINEERING_STANDARDS §§22–24  headless, deterministic, LM-independent core tests
REUSE_POLICY       adopt → adapt → extend → build; both adopting and declining are recorded
```

---

# 2. Audit — what exists, from source (`main @ 0fd0be3`)

## 2.1 The controllers that exist

`cognition/` holds one Rust crate, `cognition/rule-controller` (`mineworld-rule-controller`), and a
`.gitkeep`. There is no Python anywhere under `cognition/`, no `pyproject.toml` in the repository, and
`sdk/` holds only `sdk/rust`. `.structured-coding/standards.md` declares `ruff` and `pyright` and keeps
them disabled "until the first Python module under `cognition/`".

The crate depends on `mineworld-contracts` and on five System Pack crates whose actions or components
it uses by name: `conversation` (`Talk`, `ConversationHistory`), `movement` (`Move`, `Passages`,
`MAX_STRIDE`), `group-activity`, `naming` (`DisplayName`) and `schedule` (`Agenda`). It depends on no
transport, server or kernel.

**`RuleController`** (`src/lib.rs`), the reactive controller:

- `decide(&mut self, &Observation<Value>) -> Option<ActionRequest>`.
- Reads the observer's own `ConversationHistory` from the observation, takes the newest line per
  speaker, and answers the lowest-`EntityId` speaker whose newest line it has not yet answered, if the
  observation offers an *available* `talk` against that speaker.
- Its only state is `answered: BTreeMap<EntityId, Heard>`, which line it answered per speaker.
- It never attempts a complete affordance and takes no initiative (`ARC-34` point 5).

**`PacedRuleController`** (`src/paced.rs`), the headless controller (`ARC-27`):

- `decide(&self, …)`: a pure function of seed, pace and observation, enforced by `&self`.
- Consulted at `genesis + k + m·P` with P = 900 s (`ARC-27` note).
- Its draws are SplitMix64 mixes of `(seed, observer, instant)`, with fixed draw indices per band:

```text
0        the walking roll: greet, approach, doorway, wander, stand
1        answer an in-window line (75 in 100)
2, 3     greet: whom, and which of six fixed greetings
4        approach: whom
5, 6     wander offsets
8 … 12   social.rs: answer an invitation, initiative, invitee, kind, joined
13       agenda.rs: follow the agenda (90 in 100)
14, 15   offered.rs: attempt an available complete affordance (20 in 100), and which
```

- Order inside `decide`: invitation answer → reply in window → agenda → social → offer band →
  walking roll.
- "Answer once" holds with no record, because a line heard at `h` lies in exactly one window
  `(t − P, t]` of its listener.

**The offer band** (`src/offered.rs`, `ARC-34`): filters the observation's affordances to those
available *and* carrying a `payload`, then submits the chosen one through `Affordance::request`. It
names no action type and imports no pack type. Constants frozen for S9: `ATTEMPTS_OFFERED = 20`,
`OFFER_DRAW = 14`, `OFFERED_CHOICE_DRAW = 15`.

**The agenda** (`src/agenda.rs`, `ARC-32`): decodes the observer's own `Agenda` and, when away from its
place, heads there through the disclosed passage whose `to` is that place, 90 times in 100.

**Free-form actions stay known by name** (`ARC-34` point 6): `talk`'s utterance, `move`'s position
and `invite`'s kind cannot be offered complete. A controller must know them by name to use them.
`ARC-34` says it explicitly: *"A new pack whose actions are free-form is usable headless only through
a language-model controller (S10)"*.

## 2.2 How a controller attaches to a seat

**`mineworld run`** (`tools/cli/src/run.rs`) steps the world itself, synchronously, on one thread, and
consults a `PacedRuleController::new(seed, PACE)` per seat. No server, no transport, no wall clock.

**`mineworld server <world> --agent SEAT`** (`tools/cli/src/main.rs` `serve`, `tools/cli/src/agent.rs`):

- Each `--agent` seat is checked against the pack's seats, then `tokio::spawn(agent::drive(host, seat))`.
- `drive` calls `host.join(seat)`, then loops on `seated.observations().recv()` and passes each
  observation to `RuleController::decide`. Any request is sent with `host.submit(observer, request)`.
- These are the same two calls a WebSocket session makes. The driver skips only the JSON framing and
  the TCP hop. `agent.rs` records that limit and anticipates this step: *"A controller in another
  process, speaking the wire protocol, is a driver swap and no architectural change."*
- The wiring lives in the CLI, the only crate that depends on both the server and a controller. The
  server crate depends on no controller.

## 2.3 Observation production

- `server/src/perception.rs` is the one seam: `trait Perception { fn observe(&self, &PerceptionContext)
  -> WireObservation }`. The context carries `&World`, the observer, the instant, and
  `recent_events: &[EventEnvelope]`, a bounded window. A persisted world restores that window from
  the save with `recent_facts`.
- `tools/cli/src/perceive.rs` `PackPerception` calls `mineworld_presence::observe(world, observer, at,
  providers)`.
- `systems/presence/src/observe.rs` fills `self_location`, `entities` (the observer's place and
  everybody in it), the components each pack's `PerceptionProvider::discloses`, `present-in`
  relations, and `affordances` with the server's verdict.
- **`events` is always empty.** The module says why, and names this step:

> **Events.** [`Observation::with_events`] exists and this function leaves it empty. Deciding which
> recorded facts a person learned of means keeping a per-observer position in the log and honouring
> [`Visibility`] over time; that is a perception system's job (S10) and the transport's (S11), and a
> first cut here would be a second implementation to delete.

- `server/src/runtime.rs` `sweep` makes one perception call per subscriber, at the host's cadence
  (10 Hz by default) and straight after any request that recorded facts. Delivery uses `try_send` on a
  bounded channel (`observation_backlog`), so a slow reader **loses** observations, by design.
  Every observation is whole; there are no deltas.
- A hosted world's clock advances **one simulated second per wall second** (`runtime.rs`, "the pacing").
  A model that thinks for two wall seconds lets two simulated seconds pass.

## 2.4 The contract types cognition will read

- `Observation<P>` (`contracts/src/observation.rs`): `observer`, `at`, `self_location`, `entities`,
  `relations`, `events: Vec<PerceivedEvent<P>>`, `affordances: Vec<Affordance<P>>`.
- `PerceivedEvent<P>` wraps an `EventEnvelope<P>`. Its constructor is documented as *"Called by
  whatever made that judgement — a perception system — and by nothing else."*
- `EventEnvelope` (`contracts/src/event.rs`) carries:

```text
id · at · event_type · subjects · participants · place
caused_by · payload · visibility · provenance
```

- `Provenance` carries `emitted_by` and `controller_decision: Option<ActionId>`. Its documentation:
  *"`controller_decision` is what connects a fact back to the controller that asked for it, which is
  what makes an LM controller's effect on a world reviewable after the fact."*
- `Visibility` is `Public | Place(PlaceId) | Participants | Entities(BTreeSet<EntityId>) |
  SystemInternal`.
- `Affordance<P>` carries `action_type`, `target`, `available`, `unavailable_reason`, `requirement` and,
  on a complete affordance, `payload` (`ARC-34`).

## 2.5 How facts declare their audience today

Every System Pack already states a `Visibility` on every emission. A sample relevant to memory:

| Fact | Owner | Visibility | So, perceived by |
| --- | --- | --- | --- |
| `spoke` | conversation | `Place(place)` | everyone present, so **overhearing exists** |
| `conversation-started` | conversation | `Participants` | the two speakers |
| `arrived`, `person-entered-place` | presence | `Place(place)` | everyone in that place |
| `stopped-short` | presence | `Place(place)` | everyone in that place |
| `became-acquainted`, `relationship-changed` | relationships | `Participants` | the two people |
| group-activity facts | group-activity | `Participants` or `Place(place)` | members, or the place |
| wage and shift facts | employment | `Entities({employee})` or `Participants` | the employee |
| `money-transferred` (buy) | economy | `Place(place)` | the shop's occupants |
| movement facts, naming, item kinds | movement, naming, item | `Public` | everyone |

`Visibility::Place` needs to know who was in the place at the moment of the fact. That is presence's
state, and presence's own facts (`arrived`, `person-entered-place`, `stopped-short`, genesis) are the
record of it.

## 2.6 The reply template the operator saw

`RuleController::reply_to` (`cognition/rule-controller/src/lib.rs`):

```rust
format!("I remember you. You said \"{mine}\". Earlier, {who} said \"{theirs}\" to me.")
format!("I remember you. You said \"{mine}\". You are the first person to speak to me here.")
```

Quotations are cut at 80 characters (`QUOTED_AT_MOST`). An utterance is at most 480 bytes
(`UTTERANCE_MAX_BYTES`). The paced controller's own initiative is one of six fixed greetings.

The template is not a defect. It is the `AC-15` evidence, a sentence only a Person who holds both
conversations could produce. It reads as a template because it is one, and the crate says so: quoting
*"is deliberately literal […] because summarizing is interpretation and interpretation is what MVP-1's
cognition layer is for."*

## 2.7 What an NPC could remember today

| Source | What it holds | Bound | Who may read it |
| --- | --- | --- | --- |
| `ConversationHistory` (conversation) | `Heard { speaker, at, utterance }`, lines **heard** by this person, oldest first | `REMEMBERED_AT_MOST = 32` | the person, through their own observation |
| `Acquaintances` (relationships, `ARC-28`) | per counterpart: familiarity, regard, exchanges, activities shared, first met, last contact | none | the holder only |
| `Agenda` (schedule, `ARC-32`) | today's place and label | one entry | the person |
| `display-name` (naming) | names of people perceived | n/a | whoever perceives them |
| the objective biography (`ARC-29`) | `{ at, event id, event type, place, counterparts }` per biographical fact | none ("L0 only: a long world's biography is long") | a **tool** reading a save: `mineworld biography` |

What is missing for Milestone D:

- What the person **said** is not in `ConversationHistory`, only what they heard.
- Nothing older than 32 heard lines survives.
- Facts other than speech are not perceived by a controller at all, because `events` is empty.
- The objective biography is a CLI projection over the save. No controller can read it, and it must
  not, because it is world truth rather than perception (`INV-4`, `INV-13`). §3.9 resolves the
  sentence in `CORE_CONCEPTS.md` §5.4, *"A controller normally reads the summary"*, against `INV-13`.

## 2.8 F-13, precisely

`overall.md` §7, S5:

> Open, outside S5 (F-13): a restarted `--agent` rule controller re-answers its last line, because what
> it has answered lives in controller memory, not world state. Belongs with S10 (cognition); recorded
> so Milestone D does not rediscover it.

`ARC-27` closed it for `run` (stateless pace windows) and left it open for `--agent`: *"Its remedy is a
perception or cognition change (S10)."* The cause: `RuleController.answered` starts empty, and the
restored `ConversationHistory` still holds the last heard line, so the restarted controller answers it
again. The controller cannot see that it already answered, because its own `spoke` fact is never
delivered to it.

## 2.9 Protocol facts that bind this design

From `server/PROTOCOL.md` revision 1 and `server/src/runtime.rs`:

- A client may send exactly `join { seat }` and `submit { token, request }`. There is no third frame.
- The server sends `welcome`, `observation { seq, revision, observation }`, `result { token,
  action_id, result }` and `refused { code }`.
- Identities are decimal strings, and every other number is an integer (§7).
- No authentication: the seat name alone.
- **Several connections may join the same seat**: `join` refuses only an unknown seat, so two
  controllers could drive one Person at once.
- `submit` checks `actor == observer`, allocates the `ActionId` and the instant, advances the world to
  now, then dispatches against **current** state.
- JSON text frames only.

## 2.10 Specification gaps found, recorded rather than worked around (`CLAUDE.md` §2.1(4))

| # | Where | The gap | Proposed resolution |
| --- | --- | --- | --- |
| G-1 | `ARCHITECTURE.md` §13.1 (D-4) | Protobuf "introduced at the first real cross-language boundary … first boundary Python cognition, then the Godot client". In fact the Godot client crossed first, over the JSON protocol (`server/PROTOCOL.md`), and no Protobuf exists. | The spec is stale. Record that the first cross-language boundary is the JSON client protocol, mirrored for Python as typed models checked against golden frames the Rust tests produce (§3.4.2). Protobuf/gRPC is declined for now (§4.6). Proposed edit in §12. |
| G-2 | `CORE_CONCEPTS.md` §5.4 | "A controller normally reads the summary" of the *biography*, while §5.2 defines biography as derived from the **global** event history and `INV-13` forbids controllers world truth. | A controller reads the compression of **its own perceived history** (§3.9). The objective biography's compression is a tool's view. Proposed edit in §12. |
| G-3 | `presence/src/observe.rs`, `PerceptionContext` | Event perception is named as S10's and S11's job and done by nobody. | `ARC-S10-b`, §3.3. |
| G-4 | `MODULE_SPEC.md` §4.1 | `cognition_profile` is refused by name. `CORE_CONCEPTS.md` §16 shows `cognition_profile: default: local_agents` in a World Pack. | Keep it refused. A World Pack binds no cognition (`AC-4`, §3.7.4). QS10-6. |
| G-5 | people files | `note` is "authoring provenance, never gameplay state". No source of a person's character exists for a prompt. | A `persona` System Pack owning a `persona:` section (§3.8.5). QS10-7. |
| G-6 | `ARCHITECTURE.md` §8 | `save/cognition_cache/` is listed as "later (S10 / MVP-1)". | The cognition store lives under the operator's cognition directory, not in the world's save, because it is not world state (§3.8.4). Proposed edit in §12. |
| G-7 | `server` `join` | No seat exclusivity: an LM controller and a human could both drive Alice. | A requirement on S11 (§11, R-S11-4). |
