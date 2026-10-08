# Step 17 — Cognition: the language-model half of S10, and Milestone D

**Role:** step document for the part of **S10 — Cognition layer** that the operator deferred to MVP-1
on 2026-09-25, and for **Milestone D — LM-native persistent characters**. It records the requirement,
the audit, the design, the reuse comparison, the invariants, a PR split with integration checkpoints,
the risks, the requirements this step places on S11, and the open questions. It holds no frozen PR
design and authorizes no implementation (`CLAUDE.md` §3.1).
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S10, and the "Original scope, retained
for MVP-1" list), §4 (`AC-4`, `AC-10`), §7.
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions and coordination rulings in `overall.md` "Parallel build-out, 2026-10-08", which bind and override this document where they differ (decision numbers, protocol ownership, event perception, the shared module, digests). Superseded wording below: `DRAFT — awaiting the primary session's review`.
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

---

# 3. Design

## 3.1 Module map and ownership

Every piece has one owner. Nothing in this table changes the kernel or `mineworld-contracts`
(invariant I-1, §5).

| Piece | Language | Location (proposed) | Owner | Depends on |
| --- | --- | --- | --- | --- |
| Audience rule and `Whereabouts` fold: who perceived which fact | Rust | `systems/presence/src/audience.rs` | presence, which owns where people are | contracts, presence's own facts |
| `EventPerception` seam: perceived events per observer | Rust | `server/src/perception.rs`, beside `Perception` | server, as a seam | contracts |
| Perceived-event stream, cursor resume, seat exclusivity, tokens | Rust | `server/` | **S11** (requirements in §11) | the seam above |
| `mineworld perceived` (offline export of one Person's perceived facts) | Rust | `tools/cli/src/perceived.rs` | CLI, like `biography` | presence's audience, persistence |
| F-13 fix: the reactive rule controller answers statelessly | Rust | `cognition/rule-controller/src/lib.rs` | rule-controller | perceived events |
| `persona` System Pack: a Person's authored character, disclosed to self | Rust | `systems/persona/` | persona | sdk/rust, presence (disclosure) |
| Python protocol SDK: typed frames, seat session, `Controller` protocol | Python | `sdk/python/` (`mineworld-sdk`) | sdk | the client protocol only |
| LM cognition: memory, compression, context, `LMController`, backends, recorder, budgets | Python | `cognition/lm-controller/` (`mineworld-cognition`) | cognition | `mineworld-sdk` only |
| Operator configuration: which seats, which backend, budgets, secrets by env-var name | TOML | outside every World Pack; path given on the command line | the server operator | — |

The dependency direction is one way: `mineworld-cognition → mineworld-sdk → the wire protocol`. The
server depends on neither, and no Rust crate depends on Python. Removing `cognition/lm-controller`
deletes nothing any other module needs (§3.16).

## 3.2 The process boundary (`ARC-S10-a`)

**Problem.** The cognition layer is Python (`ENGINEERING_STANDARDS.md` §3), the world is Rust, and the
server must never block on a model (`ARCHITECTURE.md` §9). Something has to carry observations and
perceived events out, and requests back.

**Options considered.** The full comparison, with licences and maturity, is §4.6.

```text
(a) LMController in Rust, inside the server     Rust's model-client ecosystem is thinner; a model stall
                                                shares the world's process; against §3's Python rule
(b) Python embedded in the server (PyO3)        the world process hosts a Python runtime; a cognition
                                                crash or GIL stall is a world crash or stall; the server
                                                crate would depend on a controller (agent.rs forbids it)
(c) Python sidecar behind the Rust --agent      a second protocol (gRPC or stdio JSON-RPC) beside the
    driver (the driver keeps the seat, asks     client protocol; two boundaries to keep in step; the
    Python what to say)                         "controller" is split across two processes
(d) Python process as a client of the public    chosen
    protocol, one WebSocket per seat
```

**Choice: (d).** An LM-driven Person is a client. It joins a seat, receives the observations and
perceived events that seat is entitled to, and submits `ActionRequest`s in the same `submit` frame a
2D or 3D client sends.

- **`INV-9` and `INV-1` by construction.** The server cannot tell a model from a person, and has no
  extra path a model could use. `AC-5` takeover is symmetric: a human joining Alice's seat and an LM
  joining it are the same operation (R-S11-4).
- **One boundary, already verified from the far side.** The JSON protocol is proven against Godot
  (`overall.md` R-9). Python becomes its third consumer, checked by the same golden frames (§3.4.2).
- **The server never waits.** Observations already go out with `try_send`, and a model call is
  something that happens in another process.
- **Placement is free.** Cognition can run on the machine with the GPU, or not at all.

**What (d) costs, and where it is paid.**

1. The client protocol must carry perceived events reliably, and must let a seat resume from a cursor
   (§3.3, R-S11-1 … R-S11-3).
2. A seat needs exclusivity and a credential (R-S11-4, R-S11-5).
3. A model's latency is wall time, and a hosted world's clock runs one simulated second per wall
   second (§2.3). A two-second decision is a two-second-late reply. That is realistic, and §3.6
   handles what the world did meanwhile.

`agent.rs` predicted (d): *"A controller in another process, speaking the wire protocol, is a driver
swap and no architectural change."* This design takes it at its word.

## 3.3 Event perception: what a Person perceived (`ARC-S10-b`)

### 3.3.1 The rule

A fact is perceived by an observer when the fact's declared `Visibility` admits them at the moment
it happened:

```text
SystemInternal   never
Public           always
Participants     observer ∈ envelope.participants
Entities(S)      observer ∈ S
Place(p)         observer ∈ envelope.participants ∪ envelope.subjects
                 or the observer's whereabouts, after this fact, is p
```

"Whereabouts after this fact" is read from the `Whereabouts` fold (§3.3.2) once it has applied this
fact. So a person who arrives perceives their own arrival, and the people already in the place
perceive it too. A person who has just left `p` does not perceive what happens in `p` afterwards.

Approximations, recorded so they are not mistaken for intent:

- **Place-level, not distance-level.** Anyone in the café overhears a `spoke` in the café, as
  `Visibility::Place` says. A hearing range is a later perception system's refinement, behind the same
  seam.
- **One request, one instant.** A request's facts share one instant, and the fold applies them in log
  order.

### 3.3.2 Where it lives

- **`mineworld_presence::audience`** holds the rule and `Whereabouts`, a `BTreeMap<EntityId, PlaceId>`
  folded over presence's own facts: genesis placement, `arrived`, displaced arrivals.
  - `apply(&EventEnvelope)` updates the fold.
  - `admits(&EventEnvelope, observer) -> bool` applies the rule.
  - It decodes only presence's own published event types. It names no other pack's vocabulary
    (`ARC-28` point 3).
- **`server::perception::EventPerception`** is the seam:
  - shape: `fn perceived(&self, fact: &EventEnvelope, observer: EntityId) -> bool` plus
    `fn record(&mut self, fact: &EventEnvelope)`;
  - default: `PerceivesNoEvents`, so a world with no perception system leaks nothing;
  - the presence implementation is wired by `PackPerception` in the CLI, exactly as `Perception` is
    today;
  - a future `HearingSystem` replaces or refines it with no server change.

### 3.3.3 Live, resumed and offline: one function

- **Live.** The world thread folds every newly recorded fact, then queues each admitted fact on every
  subscriber that asked for perceived events (R-S11-1).
- **Resumed.** A seat that joins with a cursor gets every admitted fact after the cursor. For a
  persisted world, the server reads the facts after the cursor from the save and folds `Whereabouts`
  from genesis. Cost: one pass over the fact table, about 373 000 rows after 300 days of Market Town
  (`overall.md` §7, 11e). Seeding the fold from the newest snapshot's `Presence` components is an
  optimization, deferred until measured. A world without a save can resume only within its recent
  window. Otherwise it refuses with `cursor_unavailable` (R-S11-2).
- **Offline.** `mineworld perceived <world> --save DIR --person KEY [--since ID] [--json]` runs the
  same fold over a save and prints the facts the person perceived. It reads the save and nothing else,
  as `biography` does (`ARC-29`).

**Invariant I-4 (§5):** for any save, the facts a subscriber received live, followed by those it
received on resume, equal `mineworld perceived` over that save. Checkpoint IC-1 (§10) tests it.

### 3.3.4 What does not change

- `Observation.events` in the 10 Hz `observation` frame **stays empty**. Perceived events travel in
  their own frame (R-S11-1), so every existing observation frame, transcript and 300-day digest is
  byte-identical (I-7).
- `PerceivedEvent` and `Visibility` are used as they are. No contract changes.
- A human client may opt in to perceived events, for example to show a conversation log. Nothing
  requires it.

### 3.3.5 Why presence, and not a new pack

`Visibility::Place` is a question about where people were, and presence owns where people are. A new
"perception" pack would have to reduce presence's facts into a second copy of presence's state. That
is a second account of one truth, which `ARC-26` and `CORE_CONCEPTS.md` §13.1 exist to prevent.
Presence already does observation (`observe.rs`). Event audience is the other half of the same job,
and `observe.rs` names it as missing.

## 3.4 The Python runtime

### 3.4.1 Two packages, split on the line a non-LM controller would need

```text
sdk/python/                    mineworld-sdk          any Python controller: scripted, RL, LM
  src/mineworld_sdk/
    wire.py        the protocol's frames as typed models (Pydantic), ids as distinct string types
    session.py     one seat: join, observations (newest wins), perceived events (reliable), submit
    controller.py  the Controller protocol and the Decision types (§3.4.4)
    requests.py    Decision → ActionRequest, only from what the observation offers
cognition/lm-controller/       mineworld-cognition    the language-model Controller Pack
  src/mineworld_cognition/
    memory/        subjective memory store, ingestion, retrieval (§3.8)
    compress/      L0–L3, structural summarizer, model summarizer (§3.9)
    context.py     assembles one decision's context, deterministically (§3.5)
    controller.py  LMController: triggers, routine policy, social decisions (§3.4.5, §3.5)
    backend/       ModelBackend, the OpenAI-compatible adapter, ScriptedBackend (§3.7)
    record.py      cassettes: record, replay, strict misses (§3.11)
    budget.py      the budget gate and its ledger (§3.10)
    config.py      operator configuration (TOML), secrets by env-var name (§3.7.4)
    __main__.py    `python -m mineworld_cognition --config FILE`
```

`overall.md` §3's cross-cutting list already places "sdk/python with S10". Python tooling:

- `uv` for environments and locking (`DEP-S10-e`);
- `ruff` and `pyright` in strict mode, enabled in `.structured-coding/standards.md` in the same PR
  that adds the first module (`standards.md` says so);
- `pytest`;
- Python ≥ 3.12. The toolchain line in `overall.md` §7 reads 3.14.7.

### 3.4.2 Typed wire models, held to the Rust types by golden frames

- `mineworld_sdk.wire` mirrors `server/PROTOCOL.md` as Pydantic models.
- Identities are distinct `NewType`s over `str` (`EntityId`, `EventId`, `ActionId`), never ints
  (`PROTOCOL.md` §7).
- Every other number is a strict int, so a float is refused, as the Rust contract refuses one.
- **The Rust types stay authoritative** (D-4). Drift is caught by golden frames:
  - S11's server tests write every frame kind they produce to `server/tests/frames/*.json`
    (R-S11-7);
  - the SDK's tests parse each one, re-serialize it, and compare it semantically;
  - a field added in Rust and missing in Python fails the Python suite in CI.
- No code generator is adopted (§4.6). One alternative stays open: `schemars` emitting JSON Schema,
  with `datamodel-code-generator` emitting the Pydantic models. It becomes worth its build step if
  the frame vocabulary grows past what golden frames review comfortably (QS10-12).

### 3.4.3 The seat session

One asyncio task per seat, and one WebSocket per seat (the protocol's one-seat-per-connection rule
stands):

```text
connect → join { seat, token, perceived_since: cursor | null }   (R-S11-1, R-S11-5)
  welcome                  → the observer id; the world instance
  observation              → replaces the session's newest observation (older ones are worthless)
  perceived { events, through }
                           → memory.ingest(events), and only then the cursor advances to `through`
  result { token, … }      → completes the pending submit with that token
  refused / closed         → seat_taken: stop and leave the seat (R-S11-4);
                             lagged or disconnected: reconnect with the cursor, with backoff
```

The cursor is the id of the last perceived event that **memory has durably ingested**. A crash
between receipt and ingestion re-delivers rather than loses. Ingestion is idempotent by `EventId`.

### 3.4.4 The Controller protocol, and the only decisions there are

```python
class Controller(Protocol):
    def perceive(self, events: Sequence[PerceivedEvent]) -> None: ...
    def observe(self, observation: Observation) -> None: ...
    async def decide(self, trigger: Trigger) -> Decision | None: ...
    def outcome(self, decision: Decision, result: ActionResult) -> None: ...

Decision = Say(to: EntityId, words: str)          # `talk`, known by name (ARC-34 point 6)
         | Attempt(offer: OfferRef)               # a complete affordance, submitted unchanged
         | Step(to: Location)                     # `move`, known by name; the routine policy's only
```

- `requests.py` turns a `Decision` into an `ActionRequest` **only** from what the newest observation
  offers:
  - `Say` needs an available `talk` affordance targeting `to`;
  - `Attempt` needs that affordance still offered, and resubmits its `payload` unchanged
    (`ARC-34`);
  - `Step` needs an available `move`.
- Anything else has no representation. A model that "decides" to shoot Bob produces nothing a
  request can be built from. Had a request been built, the world would answer `Unavailable`
  (`INV-10`).
- `OfferRef` is the affordance's position in the observation it came from, plus its action type and
  target, re-located in the newest observation by all three. A complete affordance is identified by
  position (`PROTOCOL.md` §5), so a moved offer that no longer matches is dropped, not guessed.

### 3.4.5 Triggers and asynchronous workers

Cognition is "event triggered, asynchronous" (`NETWORKING.md` §4). A seat decides when one of these
arrives, not on every 10 Hz observation:

| Trigger | From | Default handling |
| --- | --- | --- |
| T1 somebody spoke to me | perceived `spoke` with me as listener (`subjects`) | social decision (model, within budget) |
| T2 an invitation to me | perceived group-activity fact naming me | routine policy (deterministic) |
| T3 somebody I remember entered my place | perceived `arrived` and memory | social decision, rate-limited per counterpart |
| T4 my agenda changed or I am away from it | observation (`Agenda`) | routine policy |
| T5 heartbeat | every `H` simulated minutes (default 15, the headless pace) | routine policy |

- **At most one decision in flight per seat.** A trigger that arrives meanwhile marks the seat
  dirty, and the seat decides again once, on the newest state.
- **A world-wide limit on in-flight model calls** (§3.10) queues seats fairly by trigger time.
- **The server never waits.** The world's only interaction with a slow seat is a request arriving
  late (§3.6).

### 3.4.6 The routine policy is deterministic and model-free

`ARCHITECTURE.md` §9.1: `routine: policy: deterministic`. In an LM seat, walking to the agenda's place,
answering invitations and idling are decided by a small deterministic policy, and never by a model.
*"LM-native does not mean an LM produces every frame"* (`DECISIONS.md`, Microverse note).

The policy reads the server's verdicts and never computes one. It proposes strides toward the
disclosed passage that leads to the agenda's place, never longer than `MAX_STRIDE`, and the movement
system decides. It repeats a behaviour the Rust `PacedRuleController` already has, in another language.
That is a duplicated **policy**, not a duplicated world rule. `ENGINEERING_RULES.md` §9's prohibition
is on rules a renderer or controller evaluates instead of the server, and the policy evaluates none.
The alternative is a Rust routine controller sharing the seat, which would put two controllers on one
Person. QS10-8 asks whether the Python port should stay this minimal.

## 3.5 The LMController's social decision

One social decision is a pure function of four inputs:

```text
(the newest observation, the memory store, the persona, the trigger)
  → context (deterministic text and structure)
  → CompletionRequest (provider-neutral, §3.7.1)
  → budget gate (§3.10) → recorder (§3.11) → ModelBackend
  → Completion → parsed SocialChoice → Decision → request (§3.4.4) → pre-submit check (§3.6)
```

**Context, assembled deterministically.** Sorted, bounded, with no wall clock and no randomness, so
one state always yields the same `CompletionRequest`, and a cassette key is stable (§3.11):

1. **Who I am.** The disclosed display name, the `persona` section (§3.8.5), and today's `Agenda`.
2. **Where I am and who is here.** The place, and each perceived person by display name, with my
   relationship values for each, disclosed to me alone (`ARC-28` point 6).
3. **What I remember.** The memory sections chosen by retrieval (§3.8.6), each line carrying its Event
   IDs.
4. **What just happened.** The trigger and the newest perceived lines, quoted as data.
5. **What I can do.** The offered affordances: `talk` to whom, and the complete affordances numbered
   by position.

The model sees **names, never entity ids**. Names are mapped back to ids from the same observation.

**Structured output, one schema.** `SocialChoice` is a Pydantic model whose JSON Schema goes into the
request as `output_schema`:

```text
SocialChoice
  act      "say" | "attempt" | "nothing"
  to       a display name from the context's "who is here", when act = say
  words    ≤ 480 bytes, when act = say   (UTTERANCE_MAX_BYTES; the conversation pack refuses longer)
  offer    a number from "what I can do", when act = attempt
  recalls  the Event IDs from "what I remember" this choice draws on (may be empty)
```

- **`recalls` is how "reacts consistently" becomes checkable.** Every cited id must be one the context
  actually contained, or the choice is invalid. The Milestone D test asserts that the 2D lines are
  cited (§3.14).
- **One repair attempt.** An invalid completion gets one re-ask that carries the validation error.
  This is the useful idea from Instructor (§4.2). If the repair fails too, the result is `nothing`
  plus a recorded failure. Every raw attempt is recorded.

## 3.6 Mandatory revalidation

`ARCHITECTURE.md` §9: *"An intent validated against a stale observation must fail, not succeed on the
strength of having been requested."* Three layers, and only the first is authoritative:

1. **The world, structurally.** `runtime.submit` advances the world to *now* and dispatches against
   the current state. The owning system validates against what is true when the request arrives, not
   what was true when the model was asked. Cognition has no way to bypass this, because there is no
   other path into the world.
2. **The seat, before submitting.** A decision records `based_on = (observation.at, observation seq,
   perceived cursor)`. The request is dropped, unsent, if any of these holds:
   - the newest observation no longer offers the needed affordance as available. This reads the
     server's verdict and computes nothing;
   - a newer line from the same speaker arrived after `based_on`. The seat re-decides once, on the
     newest state;
   - the decision is older than `max_decision_age` (wall seconds; default 15).
3. **The record.** Each decision's `based_on`, its request, the server's `action_id` and its result
   go to the decision log (§3.11.4). A rejection is perceived and remembered like any other outcome:
   "I tried to answer Bob; he had gone".

**Adversarial checkpoint IC-3 (§10):** a scripted backend that answers after three wall seconds. During
the delay the test client walks out of `talk`'s range. Expected: either the seat drops the request
before sending it, or the world rejects it `TooFarAway`. **No `spoke` fact from Alice appears after
the departure.** Both outcomes are asserted from the save, not from the seat's log.

## 3.7 The provider-neutral model interface

### 3.7.1 `ModelBackend`

```python
class ModelBackend(Protocol):
    async def complete(self, request: CompletionRequest) -> Completion: ...

CompletionRequest
  purpose          "decide" | "summarize"
  messages         [(role: "system" | "user" | "assistant", text: str)]
  output_schema    JSON Schema | None          from a Pydantic model
  sampling         temperature, max_output_tokens, seed | None
Completion
  text             str
  finish           "complete" | "length" | "refused" | "error"
  usage            input_tokens, output_tokens   (as the backend reports them; estimated if absent)
```

Deliberately absent: model names, provider names, URLs, keys, provider-specific parameters, tool-call
formats and streaming. Those are adapter configuration. The request carries no `tier` either: the
router binds a purpose and tier to a backend **before** the request is built, so the request, and
therefore the cassette key, does not change when the operator changes provider (§3.11.1).

### 3.7.2 Adapters

| Adapter | Covers | Built on |
| --- | --- | --- |
| `OpenAICompatibleBackend(base_url, model, api_key_env, extra)` | Ollama's `/v1` endpoint; any OpenAI-compatible server (vLLM, llama.cpp server, LM Studio, a hosted endpoint) | the `openai` Python SDK (`DEP-S10-a`), its `base_url`, and `response_format` with a JSON Schema |
| `ScriptedBackend(script)` | tests; no model at all | a pure function of the request, written in Python |
| `ReplayBackend(cassette)` | tests and regression replays | the recorder (§3.11) |

- **One adapter covers both backends `MVP.md` §6 names.** Ollama documents its OpenAI-compatible
  endpoint with `response_format`, `seed` and `temperature` among its supported fields. Its
  structured-outputs announcement demonstrates JSON-Schema parsing through that endpoint.
- **What the endpoint cannot do.** It cannot set the context size; Ollama needs a Modelfile
  `num_ctx` for that. Its `json_schema` support is demonstrated rather than specified.
- **The fallback** is an `OllamaNativeBackend` over `ollama-python`'s `format=<schema>`, an adapter of
  under a hundred lines. It is added only if the compatible endpoint proves insufficient in the
  P5 spike (QS10-3). Either way, it is an adapter file, and no other module changes.

### 3.7.3 Routing by purpose and tier

`ARCHITECTURE.md` §9.1's tiers are kept as named slots that the operator binds to backends:

```text
routine            deterministic policy, never a model (§3.4.6)
ordinary_decision  e.g. "local-small"
social             e.g. "local-medium"     ← Milestone D's dialogue
major_decision     e.g. "frontier"         (unused until a pack offers a major decision)
summarize          e.g. "local-small"      (only when model summaries are enabled, §3.9.4)
```

A slot bound to nothing means "no model for this purpose". Social decisions then fall back to the
deterministic speech of §3.10.3. A world with every slot unbound is a valid world with a polite,
formulaic Alice.

### 3.7.4 `AC-4`, credentials and what a World Pack never says

- A World Pack names no backend, model, endpoint, tier binding or key. `cognition_profile` stays
  refused (`MODULE_SPEC.md` §4.1; QS10-6).
- Which seats are LM-driven, and by which backend, is the **operator's** cognition configuration:

```toml
# cognition.toml — the operator's, never inside worlds/
server   = "ws://127.0.0.1:7878/ws"
seats    = ["alice"]
store    = "~/.local/share/mineworld/cognition/social-cafe"   # §3.8.4
mode     = "live"                                             # live | record | replay | scripted

[tiers]
social    = "local"
summarize = "none"

[backends.local]
kind     = "openai-compatible"
base_url = "http://127.0.0.1:11434/v1"
model    = "qwen3:8b"        # an example, not a decision (QS10-2)
key_env  = ""                # Ollama needs none

[backends.hosted]
kind     = "openai-compatible"
base_url = "https://api.example.com/v1"
model    = "…"
key_env  = "OPENAI_API_KEY"  # the NAME of a variable; the value never appears in any file we write
```

- **Keys are read from the process environment by the variable name configured.** The runtime never
  reads `~/.config/mineworld/secrets.env` itself. An operator may `source` it before starting
  cognition. A key is never logged, never recorded in a cassette (§3.11 records at our interface,
  below HTTP headers) and never echoed in errors. When the configured variable is unset, the
  failure names the variable and never prints a value.
- **`AC-4` test (IC-6, §10).** The Milestone D scenario is run twice from one cassette, under two
  backend bindings: the cassette was recorded through `local`, and is replayed with the binding
  renamed and repointed. The test asserts that:
  - `worlds/` is byte-identical;
  - the cassette keys are identical;
  - the submitted requests are identical.

  The keys are provider-neutral by construction, so this test is meaningful rather than trivial. A
  live variant with two real endpoints is optional and operator-run (QS10-4).

## 3.8 Subjective memory

### 3.8.1 What it is

`CORE_CONCEPTS.md` §5.3: *"What a character saw, heard, believes, remembers, forgot, or
misunderstood. Memory belongs to cognition, not to the world."* Here that means:

- **Inputs:** only what the seat was given. Its perceived events (§3.3), its own observations, and the
  outcomes of its own requests.
- **Never:** the fact log, a save, the objective biography, another seat's memory, or any server
  endpoint that is not this seat's stream.
- **Not world state.** The world never reads it. Deleting it changes nothing in the world (I-6, §5).

### 3.8.2 Ingestion

Each perceived event is rendered into one **L0 record**:

```text
L0 record
  event_id          the fact's id                          (provenance, always)
  at                the fact's world time
  kind              the fact's event type, as a string      (no interpretation)
  place             the fact's place, by display name if known
  who               counterparts by display name; ids kept alongside, never shown to a model
  gist              a deterministic one-line rendering of the payload, by a per-event-type renderer
  mine              whether I was the actor (provenance.controller_decision is one of my action ids)
```

**Renderers** are small pure functions keyed by event type: `spoke` → `Bob said to me: "…"`, or
`I said to Bob: "…"`, or `Bob said to Carol: "…"` when overheard. An unknown event type renders
generically from envelope fields, so a new System Pack needs **no** cognition code to be remembered,
only less eloquently (I-9). A renderer for a pack's events is an optional plug-in, registered by event
type (§3.16).

### 3.8.3 Beyond L0

The store also holds the compression levels (§3.9) and **impressions**: per counterpart, the latest
relationship values disclosed to me and the Event IDs of the facts that changed them. An impression is
a cache of what observations said, never a judgement the world reads.

### 3.8.4 The store and how it is rebuilt

- **Store.** One SQLite file per seat (Python's `sqlite3`, `DEP-S10-d`). It lives under the operator's
  cognition directory (`store` in §3.7.4), keyed by world instance and seat. It is **not** placed in
  the world's save directory: the save holds the world's truth, and putting a controller's memory
  beside it would invite the coupling `INV-4` forbids. `ARCHITECTURE.md` §8's `cognition_cache/` line
  is amended accordingly (§12).
- **Rebuild.** Memory is a derivation. With the store deleted, re-joining with `perceived_since: null`
  re-delivers the seat's whole perceived history; ingestion and structural compression reproduce it
  exactly (I-5).
  - Model-written prose summaries are reproduced from the recorder, or omitted when no recording
    exists. They are an embellishment, never the index (§3.9.4).
  - A store whose world instance differs from the server's `welcome.world.instance` is refused by
    name, never merged.
- **Restart.** Nothing in memory is lost on a restart, because memory is durable before the cursor
  advances (§3.4.3).

### 3.8.5 Who I am: the `persona` System Pack

`CORE_CONCEPTS.md` §4.2 lists traits among a Person's state: *"personality, preferences, … They may
change, but only through an explicit system."* No pack provides them, and `note` is explicitly not
gameplay state (G-5). Proposed:

```yaml
# people/alice.yaml — a section owned by the `persona` System Pack (ARC-31)
persona:
  summary: Runs the café counter. Warm with regulars, dry with strangers. Remembers orders.
  traits: [warm, observant, dry-humoured]
  speech: Short sentences. Never more than two at a time.
```

- **Shape.** `persona` owns a `Persona` component of bounded strings (summary ≤ 280 bytes, ≤ 8
  traits, speech ≤ 160 bytes).
- **Disclosure.** It discloses the component **to its holder only**, the same rule as relationship
  values (`ARC-28` point 6). It provides no action and runs no process.
- **What reads it.** A rule controller ignores it. The LM controller puts it in "who I am". It is a
  Person trait any controller may use, not a prompt and not a provider concept.
- **Composition.** Installing `persona` and authoring sections is the `AC-1` path: `systems/**` and
  `worlds/**` only.
- **Scope.** It ships in P6 with content for social-cafe's NPCs. Whether Market Town gets it too is
  QS10-7.

### 3.8.6 Retrieval

Retrieval is deterministic and structured, with no embeddings in the first cut. The model is shown:

1. **L3**, whole (bounded by construction, §3.9).
2. Every **L1/L2** entry that names a counterpart present now or named in the trigger, newest first,
   up to its section's bound.
3. **L0**, newest first, up to its bound.
4. When the trigger quotes words, the top `k` L1 entries by SQLite FTS5 lexical match on the trigger's
   words, ties broken by recency then by Event ID.

The generative-agents scoring of recency + importance + relevance (Park et al. 2023) is the reference
design for a later retrieval upgrade. Its *importance* is a model-assigned score, and its *relevance*
needs embeddings. Both would put a model or an embedding service on the memory path, so both are
deferred (§4.3, QS10-10).

## 3.9 Hierarchical compression L0–L3 with retained Event IDs (`ARC-S10-c`)

### 3.9.1 The levels

| Level | Unit | Rule (deterministic) | Event IDs kept | Context bound |
| --- | --- | --- | --- | --- |
| L0 | one perceived fact | the newest records, within the last 24 simulated hours | its own id | ≤ 48 records |
| L1 | an **episode** | consecutive L0 records sharing a place and a counterpart set, with gaps ≤ 30 simulated minutes; an episode closes on a gap, a place change or a day boundary | every member id, as sorted ranges | ≤ 8 episodes, chosen by §3.8.6 |
| L2 | a **chapter** | one simulated week of episodes | the episode ids it covers, and the ranges they span | ≤ 4 chapters, newest first |
| L3 | **stable facts** | per counterpart: first met, last seen, current level, times met; my places by time of day; my work, if any | for each line, the ids of the facts that establish it (first meeting, latest level change) | ≤ 16 lines, top counterparts by times met |

- Every summary line is length-bounded, so each level's contribution to context is bounded by its
  count and its line bound. The whole memory section therefore has a fixed ceiling that does not depend
  on the world's age: 6 000 bytes by default (QS10-11).
- The *store* grows, because it keeps every level. The *context* does not. `AC-10` asks for exactly
  that distinction: *"does not require feeding all historical events to a model"*.

### 3.9.2 Two summarizers behind one interface

```python
class Summarizer(Protocol):
    def episode(self, records: Sequence[L0Record]) -> Summary: ...
    def chapter(self, episodes: Sequence[Summary]) -> Summary: ...
```

- **`StructuralSummarizer`** is the default and is model-free. It produces templated text from counts
  and quotes, for example: *"Day 12, 09:10–09:40, café: talked with Bob Achterberg (6 lines; he
  mentioned rain); joined a coffee with Bob."* It is a pure function of its inputs. `AC-10`'s test
  uses it, so `AC-10` holds with no model.
- **`ModelSummarizer`** rewrites a structural summary into prose through the `summarize` tier and the
  recorder. It keeps the **structural summary's id list unchanged**: the model may never add or drop
  provenance. It is off by default.

### 3.9.3 Provenance, and why compression never loses truth

- Every Event ID in any summary resolves to a fact in the save, and that fact is one this seat
  perceived, as `mineworld perceived` confirms.
- Every perceived Event ID is covered by exactly one L0 record or one L1 episode. Nothing is silently
  dropped from the index.
- *"Compression therefore reduces context, never historical truth"* (`CORE_CONCEPTS.md` §5.4): the
  facts stay in the log, and the ids lead back to them.

### 3.9.4 Model prose is an embellishment, never the index

A summary's ids, bounds and coverage come from the structural layer. Prose may replace the *text* of a
line in the context, never its ids. With prose absent, because there is no model, no recording, or it
is disabled, the context is the structural text and is still complete. This is what keeps `AC-10`
model-independent.

### 3.9.5 `AC-10`, as a test decided before measuring (IC-4, §10)

1. `mineworld run worlds/social-cafe --headless --seed 7 --days 100 --save DIR`, with the paced rule
   controllers. No model is involved, and the history is long, about 1 400 talks per seat per 30 days
   (`overall.md` §7, S7).
2. `mineworld perceived … --person alice --json` exports Alice's perceived facts.
3. The cognition package ingests them and compresses with `StructuralSummarizer`, then assembles the
   context for a T1 trigger at days 10, 30, 60 and 100.

Assertions, all fixed before the first run:

- **(a) Bounded.** Every assembled memory section is ≤ 6 000 bytes. Day 100's is no more than 10 %
  larger than day 30's.
- **(b) Provenance resolves.** Every Event ID in every summary resolves in the save's fact table, and
  is in Alice's perceived set.
- **(c) Coverage.** The ids covered by L0 ∪ L1 equal Alice's perceived set exactly.
- **(d) No omniscience.** Five facts Alice did not perceive appear in no record and no summary. They
  are located, not counted (`ARC-23`): `spoke` facts in the park while Alice's whereabouts were the
  café.
- **(e) Determinism.** Two runs of steps 2–3 give byte-identical stores.

**Mutations the test must catch.** Feeding the objective log instead of the perceived set fails (b)
and (d). Disabling the roll-up from L0 into L1 fails (a). Dropping one episode's ids fails (c).

## 3.10 Cognition budgets (`ARC-S10-d`)

### 3.10.1 What is limited

Defaults are taken from `ARCHITECTURE.md` §9.1:

```text
per seat     calls_per_sim_hour          20
             tokens_per_sim_day          30 000   (input + output, as reported or estimated)
per process  max_in_flight               2        model calls at once
             call_timeout                20 s wall
             max_decision_age            15 s wall (§3.6)
per backend  requests_per_minute         unset    (an operator ceiling for hosted endpoints)
```

### 3.10.2 Where it is enforced

The gate is enforced in `budget.py`, **before** a request reaches the recorder or a backend. The
ledger is keyed by **simulated** time, read from the observation's `at`, and kept in the seat's store,
so a restart neither resets nor double-counts it. A replay uses the same ledger rules, so a budget
refusal replays identically.

### 3.10.3 Exhaustion, timeouts and unreachable models

These all fall back to the same place: the routine policy for movement, and **deterministic speech**
for a social decision.

- Deterministic speech is a short fixed phrase set, chosen by a seeded draw over `(seat, trigger
  event id)`. Examples: "One moment.", "Sorry — busy right now.", "Good to see you again."
- It never claims a memory it does not have.
- Each fallback is logged with its cause. Whether a fallback should instead be silence is QS10-9.

### 3.10.4 Cost

- The framework requires no paid API (`ARCHITECTURE.md` §9.1). The defaults bind `social` to a local
  Ollama.
- A hosted endpoint is the operator's choice, and so is its cost. The runtime reports tokens per seat
  per simulated day at shutdown and in its status line, so an operator can price it. It never
  computes money, because prices are provider concepts.

## 3.11 Recorded and replayable model outputs (`ARC-S10-e`)

### 3.11.1 The key is provider-neutral

```text
key = sha256( canonical_json( CompletionRequest ) )     sorted keys, no floats except sampling
                                                        temperature written as a decimal string
```

- No model name, no URL and no tier is in the key (§3.7.1). A cassette recorded through Ollama
  therefore replays when the operator's configuration names a hosted endpoint. That is half of
  `AC-4`'s test (IC-6).
- The backend and model that produced an entry are kept as **metadata** in the entry, for audit, and
  are never matched on.

### 3.11.2 The cassette

- A JSON Lines file per scenario, under `cognition/lm-controller/tests/cassettes/`. One entry per call:
  `{ key, request, completion, meta: { backend, model, recorded_at, latency_ms } }`.
- Recorded at **our** interface, not at HTTP. It therefore never contains headers, keys or provider
  wire formats, and it survives an SDK upgrade. §4.5 compares this with VCR-style HTTP cassettes.
- The request is stored in full beside its key. A miss can then show the nearest recorded request and
  the first differing field, so a prompt change is diagnosable rather than a bare hash mismatch.

### 3.11.3 Modes

```text
live       call the backend; record nothing
record     call the backend; append every call to the cassette     (operator-run; may need a key)
replay     never call a backend; a missing key is a CassetteMiss    (core tests; CI)
scripted   ScriptedBackend; no cassette                             (most core tests)
```

- `replay` is strict. There is no "fall through to live", because a test that can silently reach a
  model is a test that can depend on one (§24).
- `scripted` is a deterministic function of the request. For example, it answers with a sentence that
  quotes the newest remembered line and cites its Event ID. It is what most cognition tests use,
  because it needs no re-recording when a prompt changes.

### 3.11.4 The decision log, and how it meets the world's log

Each decision appends `{ seat, trigger, based_on, key, choice, request, action_id, result }` to the
seat's store. The world's facts caused by that request carry `provenance.controller_decision ==
action_id`, which the kernel already sets (`kernel/src/dispatch.rs`). So, from a fact in the save, a
reviewer finds the decision, the completion and the exact context that produced it. This is the
"reviewable after the fact" property `Provenance` was designed for (§2.4).

### 3.11.5 Two replays, never confused

- **World replay** (`ARC-25`) re-executes the journal. The journal holds each `talk` request with its
  utterance bytes, so a world replay **never** needs a model or a cassette, and is byte-identical by
  the existing tests.
- **Cognition replay** re-runs a seat against a recorded scenario, using the cassette. It checks
  cognition, not the world.

## 3.12 Natural dialogue without breaking the world's determinism

What a model writes enters the world in exactly one way: as the utterance of a `talk` request, which
the conversation system validates and records as a `spoke` fact.

```text
model completion ──► SocialChoice.words ──► talk { utterance } ──► journal (the request, verbatim)
                                                               └─► spoke fact (the event log)
```

- **The world records what was said.** The `spoke` fact holds the words, and the journal holds the
  request that produced them. `AC-12` already scopes determinism to the inputs: *"excluding explicitly
  non-deterministic external controller calls"*. The model's choice of words is such an input; once
  journaled it is fixed, and every replay reproduces it (I-8).
- **The world never interprets the words.** No system reads meaning from an utterance. `relationships`
  counts exchanges and does not judge them (`ARC-28`). A model cannot change world state by phrasing.
- **The template is retired only for LM seats.** `RuleController` keeps its template, since it is the
  `AC-15` evidence and the model-free fallback. `PacedRuleController` keeps its greetings. Natural
  speech is what an LM seat adds, not a change to any rule controller.
- **Bounds.** `UTTERANCE_MAX_BYTES = 480` is the conversation pack's, and is enforced by it. The
  schema states it to the model, and an over-long completion is repaired once, then dropped. It is
  never truncated mid-character.

## 3.13 Safety and limits

| Risk | Control | Where |
| --- | --- | --- |
| The model invents an action | Only `Say`, `Attempt` and `Step` exist, each built from an offered affordance; anything else is unrepresentable, and the world answers `Unavailable` regardless (`INV-10`) | `requests.py`, the server |
| The model acts as someone else | The seat's session can only submit as its observer; the server refuses `actor_not_observer` | the server (existing) |
| The model learns what it should not | The context is built only from this seat's observation, perceived events and memory; no other input exists (`INV-13`) | `context.py`; I-3 |
| Prompt injection through what players say | Heard words are quoted as data inside delimiters and never concatenated into instructions; the choice is schema-validated; a choice can only pick among offered affordances, so injected text cannot widen what is possible; `recalls` must cite ids that were in the context | `context.py`, validation |
| Leaking ids or internals in speech | Names only; a reply matching an id pattern or the delimiter tokens is repaired once, then replaced by deterministic speech | validation |
| Over-long or empty speech | Schema bound plus the pack's own refusal | schema, conversation |
| Secrets | Key by env-var name; never logged, recorded or echoed; cassettes recorded above HTTP | `config.py`, `record.py` |
| Runaway cost or load | Budgets per seat and per process; timeouts; no retry loop beyond one repair | `budget.py` |
| A wedged model blocks a seat | Call timeout; the seat falls back and stays responsive; the world is never waiting | `controller.py` |
| Offensive output | Out of scope for MVP-0's private worlds. A moderation hook (a `Filter` protocol before submit) is reserved and empty; whether to fill it is QS10-13 | `controller.py` |
| Cognition writes world state | Impossible: it holds a WebSocket that accepts only `join` and `submit` | protocol |

## 3.14 Milestone D, demonstrated

### 3.14.1 The claim

*Speak to Alice in 2D, meet her in 3D, and she reacts consistently with what happened.* The phrase
"the same Person and the same memory, through both clients" is made checkable as four statements:

```text
same world        one world instance id, across both clients and a restart
same Alice        one Alice EntityId, found by tag, never by a literal
same memory       the context Alice's 3D reply was decided from contains the Event IDs of the 2D lines
consistent        her 3D reply's SocialChoice.recalls cites at least one of those Event IDs, and the
                  words it produced are a spoke fact in the save, caused by her seat's action
```

### 3.14.2 The automated test (`tools/cli/tests/milestone_d.rs`, P7)

Run as Milestones B and C are, against the real binaries:

1. Start `mineworld server worlds/social-cafe --save DIR` and `python -m mineworld_cognition
   --config TEST.toml`, with `seats = ["alice"]` and `mode = "replay"` against
   `milestone_d.jsonl`. **No model is reachable.** The test sets an unroutable `base_url` and asserts
   that no connection is attempted.
2. **"2D".** A protocol client tagged `2d` joins `visitor`, walks to the counter, and says: "Hi, I'm
   new in town. I left a red umbrella here yesterday."
3. Alice's seat triggers (T1), decides, and replies. Assert a `spoke` from Alice to the visitor,
   caused by an action of her seat.
4. **SIGKILL both** the server and the cognition process. Restart both. Assert the same world instance.
   Assert that Alice's memory was not re-ingested from scratch: the store's cursor resumed.
5. **"3D".** A protocol client tagged `3d` joins `visitor`, approaches from the door, faces Alice, and
   says: "Hello again."
6. Alice decides. Assert:
   - the decision's context holds the Event IDs of step 2's and step 3's `spoke` facts;
   - `recalls` cites at least one of them;
   - the resulting `spoke` fact is in the save;
   - with the recorded completion, her words mention the umbrella.
7. `inspect` resolves every cause in the save.
8. **The same run with Alice's seat unplugged** (no cognition process). The world runs, the visitor's
   lines are recorded, and Alice says nothing. This is the "world without models" line.

The protocol-level clients are tagged as `AC-13`'s harness is (`overall.md` §3). The real 2D and 3D
clients are the operator's demo (§3.14.3). The automated test never needs a GPU, a key or a model.

### 3.14.3 The operator's run (live, optional)

```sh
ollama serve &                                 # a local model; no key
mineworld server worlds/social-cafe --save /tmp/d
python -m mineworld_cognition --config cognition/lm-controller/examples/local.toml   # proposed in P6
mineworld-2d   # as visitor: walk to the counter, talk to Alice about the umbrella
# Ctrl-C both processes; start both again
mineworld-3d   # as visitor: walk in, look at Alice, press E, say hello
```

What to look at: her 3D reply refers to the 2D conversation, and `mineworld biography` together with
the cognition decision log show which Event IDs her reply cited.

### 3.14.4 What would be a false success

- **Alice "remembers" because the model was given the whole log.** Excluded by I-3 and IC-4 (d): the
  context's ids must be a subset of her perceived set.
- **Two Alices, one per client.** Excluded by `AC-15`'s identity evidence, carried into step 6.
- **The reply only *looks* consistent.** Excluded by `recalls`: the cited ids must be the 2D lines' ids,
  not merely a plausible sentence.

## 3.15 F-13, closed for `--agent`

With perceived events, the reactive controller can know what it said. Its own `spoke` facts are
perceived, since a speaker is a participant. So "have I answered X's newest line?" becomes a question
about perceived facts, not about controller memory:

```text
answer X  ⇔  newest line heard from X  is later than  newest spoke from me to X
```

- `RuleController` reads this from a bounded window of perceived events, which `agent.rs` passes in
  beside the observation. Its `answered` map is deleted.
- A controller restarted against a resumed world reads the same facts, so it does not re-answer.
- The headless `run` path and `PacedRuleController` are unchanged (`ARC-27`): their digests stay
  byte-identical (I-7).
- Test (P2): restart the server mid-conversation with `--agent alice`. Assert that Alice's `spoke`
  count after the restart equals the count of lines heard after the restart, not one more. This is
  the regression test F-13 never had.

## 3.16 Modularity and pluggability

The operator's directive, as structure (§1.5).

### 3.16.1 `LMController` is one Controller Pack among others

```text
Controller Packs a seat may be driven by        lives in                     needs a model?
  HumanController     a 2D or 3D client          clients/                     no
  RuleController      reactive, --agent          cognition/rule-controller    no
  PacedRuleController headless run               cognition/rule-controller    no
  LMController        python -m mineworld_cognition   cognition/lm-controller no — degrades to
                                                                              deterministic speech
  (any)               anything speaking the protocol, e.g. an RL policy on mineworld-sdk
```

- **Binding is configuration.** Which pack drives which seat is a command-line or `cognition.toml`
  fact. The World Pack says only which Persons are seats (`world.yaml` `seats`).
- **Removal is deletion.** Delete `cognition/lm-controller/` and every Rust crate, World Pack, System
  Pack, client and test outside that directory still builds and passes. IC-7 (§10) checks this
  mechanically: the Rust workspace and the SDK's tests run with the directory absent.
- **A world without it is valid.** Milestone D's step 8 runs Alice's seat unplugged. `AC-11`'s 300-day
  runs never involve it. The `--agent` rule controller still drives Alice for `AC-15`.

### 3.16.2 Providers are swappable without touching a contract (`AC-4`)

- `ModelBackend` is the only interface the controller calls. Adapters are files in `backend/`, chosen
  by `kind` in operator configuration.
- Adding a provider means adding one adapter file, plus one line in the adapter registry. Nothing in
  `controller.py`, `context.py`, `memory/`, `compress/`, `record.py`, `mineworld-sdk`, the server, any
  World Pack or any contract changes.
- IC-6 is the test: the same scenario, a different backend binding, identical world, keys and
  requests.

### 3.16.3 Every inner piece is a seam with a default

| Seam | Protocol | Default | Alternatives that plug in without other edits |
| --- | --- | --- | --- |
| Event audience (Rust) | `EventPerception` | presence's place-level rule | a hearing-range system |
| Model | `ModelBackend` | OpenAI-compatible adapter | Ollama native, llama.cpp, a hosted API, scripted, replay |
| Summaries | `Summarizer` | `StructuralSummarizer` | `ModelSummarizer` |
| Memory store | `MemoryStore` | SQLite per seat | in-memory (tests); another engine |
| Retrieval | `Retriever` | structured + FTS5 | embedding-based (§4.3, later) |
| Event rendering | renderer per event type | generic envelope renderer | a pack's own renderer, registered by event type |
| Speech filter | `Filter` | none | a moderation hook (QS10-13) |
| Routine | `RoutinePolicy` | agenda + invitations | anything deterministic |

Each protocol has exactly the implementations listed when it lands. None is introduced for a single
hypothetical implementation (`CLAUDE.md` §4 rule 11). `MemoryStore`'s in-memory implementation exists
because the tests need it, and `Summarizer` has two from the start.

---

# 4. Reuse analysis, in both directions

`REUSE_POLICY.md` and the operator's directive (§1.5): compare several real candidates, plus building
our own, for every infrastructure piece; adopt what fits; record declining as carefully as adopting.

## 4.0 How to read the tables

**Fit** is judged on the three properties this step cannot trade away:

```text
N   provider-neutral   no provider concept reaches a contract or a cassette key (AC-4, MODULE_SPEC §5.5)
D   deterministic      replayable with no model; the same inputs yield the same requests (§§23–24)
V   valid without      the world, and the core tests, work with the dependency's model path absent
    a model            (VISION §1.1)
```

**Verdict** uses the vocabulary of `REUSE_POLICY.md` §19: `REUSE` · `ADAPT` · `REFERENCE ONLY` ·
`REJECT`.

**Provenance of the facts.** Licences, versions and capabilities were verified against primary
sources on 2026-10-08: GitHub repositories and licence notices, PyPI metadata, and the projects' own
documentation. Where a fact could not be confirmed, the table says *not verified*. Star counts and
commit counts are omitted as noise. A release date is given where it was read verbatim.

## 4.1 LM provider abstraction

| Candidate | Licence | Maturity (verified) | N | D | V | Cost | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **LiteLLM** (`litellm.completion`, in-process SDK) | MIT for the core; `enterprise/` is under a proprietary BerriAI licence for production use | 1.104.1 on 2026-10-07; very active | ✗ provider is encoded in the model string (`ollama_chat/…`, `openai/…`); its exact-match cache keys on the whole request, model included | ✓ with our own recorder in front | ✓ | a large dependency tree; 17 GitHub security advisories, almost all in its proxy/gateway server, which we would not run | **REJECT for now; REFERENCE ONLY.** It solves "many providers", and we need two, which one OpenAI-compatible client already covers. Its model-string prefixes would have to be kept out of our keys anyway. Revisit when a third backend is needed that has no OpenAI-compatible endpoint (QS10-3). |
| **`openai` Python SDK** pointed at compatible endpoints | Apache-2.0 | 3.26.0; maintained by OpenAI | ✓ behind `ModelBackend`; `base_url` is configuration | ✓ | ✓ | small; typed errors, timeouts and retries included | **REUSE**, isolated in one adapter file (`DEP-S10-a`). `base_url` covers Ollama's `/v1` and any compatible server; `chat.completions.parse` with a Pydantic `response_format` gives structured output. |
| **`ollama-python`** | MIT | 0.6.3 on 2026-09-29; infrequent releases | ✓ behind `ModelBackend` | ✓ | ✓ | small (httpx) | **REFERENCE ONLY now; REUSE if needed.** It is the fallback adapter if the P5 spike shows the compatible endpoint is insufficient (no `num_ctx` control; `json_schema` demonstrated rather than specified). |
| **Our own thin adapter** over `httpx` | n/a | n/a | ✓ | ✓ | ✓ | about 150 lines, plus retries, timeouts and error mapping that we would own | **REJECT.** The SDK already does this and is maintained. "Cleaner" is not a reason (`REUSE_POLICY.md` §12). |

**Recommendation.** The `openai` SDK behind our own provider-neutral `ModelBackend`. The interface is
ours, because no library's interface is provider-neutral in the sense `MODULE_SPEC.md` §5 requires.
The client is theirs.

## 4.2 Structured output

| Candidate | Licence | Maturity (verified) | N | D | V | Cost | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **The endpoint's JSON-Schema mode** (`response_format`; Ollama `format=`) + **Pydantic** | Pydantic MIT | Pydantic 2.13.5 on 2026-08-28; Ollama documents `response_format` | ✓ the schema is ours, generated by `model_json_schema()` | ✓ validation is local and deterministic | ✓ | none beyond Pydantic | **REUSE** (`DEP-S10-b`). Schema from Pydantic, enforcement by the endpoint where supported, validation by Pydantic always. |
| **Instructor** | MIT | 1.17.0 on 2026-09-09 | ~ `from_provider("ollama/…")` names providers; arbitrary `base_url` *not verified* | ~ it retries with the validation error, and the raw attempts are not ours to record | ✓ | small, but it wraps or patches the client | **REFERENCE ONLY.** We adopt its idea, one re-ask carrying the validation error (§3.5), in about thirty lines. We must record every raw attempt (§3.11), which its wrapper hides. |
| **Outlines** | Apache-2.0 | 1.3.3 on 2026-08-06 | ✓ | ✓ | ✓ | heavy for local models | **REJECT for this step.** Against Ollama and OpenAI-style servers it supports JSON Schema only, which the endpoint already does. Its strength, constrained decoding over local logits, needs us to host inference. Revisit if MineWorld ever runs models in-process. |
| **guidance** | MIT | 0.3.2 (year not verified); slower releases | ✓ | ✓ | ✓ | full grammars only with local backends | **REJECT for this step**, for the same reason as Outlines. Its Ollama and generic `base_url` support are *not verified*. |
| **Our own validator** without Pydantic | n/a | n/a | ✓ | ✓ | ✓ | re-implementing schema generation and validation | **REJECT.** Pydantic is the ecosystem standard and also types our wire models (§3.4.2). |

## 4.3 Agent memory

| Candidate | Licence | Maturity (verified) | N | D | V | Cost | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| **Letta** (formerly MemGPT) | Apache-2.0 | `letta` 0.34.5 on PyPI is now a CLI. The repository points to `letta-code`; the Python API server is archived as "V1"; the Docker server is deprecated | ✗ it is an agent runtime that owns the loop and calls the model itself | ✗ its memory edits are model tool calls | ✗ without a model it does nothing | Postgres with pgvector; a server to run | **REJECT; REFERENCE ONLY.** It would own the decision loop, which is the Controller's, and the memory store, which must derive from perceived events. Reference: its tiering, pinned "core" blocks versus searchable archive, matches our L3 versus L1/L2. |
| **mem0** | Apache-2.0 | 2.2.1 | ✗ `add()` sends messages through a model to extract facts by default (`infer=False` opts out) | ✗ extraction is a model call | ~ with `infer=False` it is a vector store wrapper | a vector database (Qdrant by default) | **REJECT.** With inference on, memory would depend on a model and on its non-determinism. With it off, what remains is a vector store we do not yet need. Neither keeps Event-ID provenance. |
| **LangChain / LangGraph memory** | MIT | legacy memory moved to `langchain-classic`; LangGraph checkpointers (`SqliteSaver`) and stores; `SummarizationMiddleware` in v1 | ~ | ~ | ✓ | the LangChain/LangGraph framework and its execution model | **REJECT; REFERENCE ONLY.** Its memory is chat-thread state inside a graph runtime. Adopting it forces our loop into theirs (`REUSE_POLICY.md` §3). The checkpointer/store split is the same split we have: cursor plus store. |
| **Graphiti** (Zep's open-source core) | Apache-2.0 | 0.30.2 | ✗ model extraction by default (OpenAI default) | ✗ | ✗ | Neo4j or FalkorDB (FalkorDB-lite embedded on Python ≥ 3.12); telemetry on by default | **REFERENCE ONLY.** Conceptually the closest fit: bi-temporal facts and every derived fact traced to an "episode", which is our Event-ID provenance. But extraction is a model call and it needs a graph database. Revisit for MVP-1's beliefs and contradictions (QS10-10). |
| **Generative Agents** (Park et al. 2023, paper) | paper | UIST '23 | ✓ | ~ its importance and relevance need model scores and embeddings | ✓ | n/a | **REFERENCE ONLY.** Memory stream, retrieval by recency + importance + relevance, reflection. Our L0 is its memory stream; L1/L2 are deterministic stand-ins for reflection; its scoring is the documented upgrade path (§3.8.6). |
| **Our own, over the perceived-event stream** (SQLite) | n/a | n/a | ✓ | ✓ a pure function of perceived events | ✓ | about one module; owned by us | **CHOSEN.** No candidate derives memory from a world's event log with id provenance and no model on the ingest path. That property is MineWorld's core value (`REUSE_POLICY.md` §4), not commodity infrastructure. |

## 4.4 Summarisation and compression

| Candidate | Licence | N | D | V | Verdict |
| --- | --- | --- | --- | --- | --- |
| **LangChain `SummarizationMiddleware`** | MIT | ~ | ✗ a model writes the summary | ✗ | **REJECT.** It compresses a chat thread with a model and keeps no source-id list. It ties summaries to the LangChain runtime. |
| **mem0 / Graphiti extraction** | Apache-2.0 | ✗ | ✗ | ✗ | **REJECT** for compression: model extraction, discussed in §4.3. Graphiti's episode provenance is **REFERENCE ONLY**. |
| **Generative-agents reflection** | paper | ✓ | ✗ model-written | ✗ | **REFERENCE ONLY.** It is the model of `ModelSummarizer`, which is optional and recorded. |
| **Extractive summarizers** (e.g. `sumy`, TextRank) | *not verified* | ✓ | ✓ | ✓ | **REJECT.** Our input is structured facts, not prose, so extracting sentences from rendered lines loses structure we already have. |
| **Our own `StructuralSummarizer`**, with optional `ModelSummarizer` | n/a | ✓ | ✓ | ✓ | **CHOSEN.** Grouping by place, counterpart and time over typed facts is a few hundred lines. Ids are kept by construction, which no candidate does. `AC-10` holds with no model. |

## 4.5 Recording and replaying model outputs

| Candidate | Licence | Maturity (verified) | N | D | V | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| **vcrpy** (HTTP cassettes, supports `httpx`) | MIT | 8.3.0; requires Python ≥ 3.10 | ✗ cassettes hold provider wire formats, URLs and headers, so they are provider-specific and must be scrubbed of keys | ✓ | ✓ | **REJECT for the cognition recorder; allowed for adapter tests.** A cassette recorded against Ollama would not replay under another provider, which makes `AC-4`'s test impossible. It breaks on SDK upgrades that change the wire. Useful for one thing: the adapter's own contract test, which records one real exchange per backend (QS10-4). |
| **pytest-recording** | MIT | 0.14.0 on 2026-10-01 | as vcrpy, which it wraps | ✓ | ✓ | **REJECT**, as above. If vcrpy is used for adapter tests, this is how: `--block-network` and record mode `none` by default. |
| **respx** (httpx mocking) | BSD-3-Clause | maintained | ✗ HTTP-level | ✓ | ✓ | **REJECT.** It mocks rather than records. `ScriptedBackend` mocks at our interface instead. |
| **LiteLLM caching** | MIT | n/a | ✗ the key includes the provider-prefixed model | ~ | ✓ | **REJECT.** Its key changes when the provider changes. |
| **inline-snapshot** | MIT | maintained | ✓ | ✓ | ✓ | **REFERENCE ONLY.** General snapshot testing. Its review-the-diff workflow is the model for re-recording (§3.11.2). |
| **Our own interface-level cassette** | n/a | n/a | ✓ keyed on the provider-neutral request | ✓ strict replay | ✓ | **CHOSEN.** About one module, JSON Lines. It records above HTTP, so it holds no secrets and survives SDK changes, and one cassette serves every backend. |

## 4.6 The process boundary to the Rust server

| Candidate | Licence | Maturity (verified) | Fit | Cost | Verdict |
| --- | --- | --- | --- | --- | --- |
| **The existing WebSocket client protocol** (`websockets` on the Python side) | `websockets` BSD-3-Clause; server `axum` (`DEP-3`) | `websockets` 17.2 on 2026-10-03, requires Python ≥ 3.11 | ✓ one boundary, already proven from Godot; `INV-9` by construction; `AC-5` takeover symmetric | needs perceived events, a cursor and seat exclusivity, all useful to every client (§11) | **REUSE** (`ARC-S10-a`, `DEP-S10-c`). |
| **gRPC** (`grpcio` / `tonic`) | Apache-2.0 / MIT | `grpcio` 1.84.0 on 2026-09-14; tonic's master is preparing breaking changes | ~ typed streams, but a second protocol beside the client one; Protobuf mirrors of the contracts (D-4) | `.proto` files, codegen in two languages, a second server port | **REJECT for this step.** Two boundaries to keep in step, for a gain (binary encoding, codegen) that the JSON protocol's rates do not need (`PROTOCOL.md` §8). `DEP-3` already says a compact encoding is replaceable behind the same boundary. |
| **stdio JSON-RPC** (the Rust `--agent` driver spawns Python) | n/a | n/a | ~ the Rust driver keeps the seat and asks Python what to say | a second protocol; cognition must run on the server's machine; one controller split across two processes | **REJECT.** It is option (c) of §3.2. |
| **PyO3 embedding** | dual MIT / Apache-2.0 (not re-verified) | mature | ✗ a Python runtime inside the world's process; the server crate would depend on a controller | a GIL and crash domain shared with the world | **REJECT.** It is option (b) of §3.2. |

## 4.7 Smaller pieces

| Piece | Candidates | Verdict |
| --- | --- | --- |
| Memory store | **`sqlite3` (standard library) with FTS5** · SQLAlchemy · DuckDB · LangGraph `SqliteSaver` | **`sqlite3`** (`DEP-S10-d`): no dependency, FTS5 for lexical retrieval, one file per seat. SQLAlchemy adds an ORM we do not need. DuckDB is analytical. `SqliteSaver` brings LangGraph. |
| Python environments and locking | **`uv`** · pip with venv and pip-tools · Poetry | **`uv`** (`DEP-S10-e`): one tool for environments, locking and running, fast in CI (S13). Its licence (dual MIT / Apache-2.0) is *not verified*; verify at P3. Poetry is heavier; pip-tools needs two tools. |
| WebSocket client | **`websockets`** · `aiohttp` · `httpx-ws` | **`websockets`**: asyncio-native, BSD-3-Clause, maintained. `aiohttp` is a whole HTTP framework; `httpx-ws` is *not verified*. |
| Wire models | **Pydantic, hand-mirrored, with golden frames** · `schemars` → JSON Schema → `datamodel-code-generator` | **Hand-mirrored** for now (§3.4.2). Generation is the documented upgrade, QS10-12. |
| Configuration | **TOML via `tomllib` (standard library)** + Pydantic · `pydantic-settings` · YAML | **`tomllib`** plus a Pydantic model: no dependency. Env vars are read only for the key named by `key_env`. |

## 4.8 Records this section asks for (`REUSE_POLICY.md` §§11–12)

Proposed `DECISIONS.md` entries, drafted in §12:

- **Adopt:**
  - `DEP-S10-a` the `openai` SDK as the OpenAI-compatible adapter;
  - `DEP-S10-b` Pydantic;
  - `DEP-S10-c` `websockets`;
  - `DEP-S10-d` `sqlite3` with FTS5;
  - `DEP-S10-e` `uv`.
- **Decline**, in one record, `DEP-S10-f`, each with the reason above and a re-evaluation trigger:
  LiteLLM, Instructor, Outlines, guidance, Letta, mem0, LangChain/LangGraph, Graphiti, vcrpy as the
  cognition recorder, gRPC.

---

# 5. Invariants

Each one is checked by a named test (§10), not by review alone.

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-1 | No change to `kernel/` or `contracts/` anywhere in this step. | diff check in every PR's review |
| I-2 | The server depends on no controller and no Python. No Rust crate depends on `cognition/lm-controller` or `sdk/python`. | `cargo tree`, IC-7 |
| I-3 | A seat's cognition reads only its own frames (observations, perceived events, results), its own store and the operator's configuration. It never reads a save, the fact log, the objective biography or another seat's store. | IC-4 (d), IC-8; a test that runs cognition with the save directory unreadable |
| I-4 | Perception is one function: the facts a subscriber receives live, followed by those it receives on resume, equal `mineworld perceived` over the same save. | IC-1 |
| I-5 | Memory is a derivation: with the store deleted and the seat re-joined from a null cursor, the structural store is byte-identical. | IC-4 (e), P6 test |
| I-6 | The world never reads cognition state. Deleting every cognition store changes no byte of any save. | P6 test |
| I-7 | Byte-identity: social-cafe's and market-town's 300-day digests, and every existing observation frame and transcript, are unchanged by every PR of this step. The reference digest is the one current on `main` when the PR merges, since S15's 12d re-baselines them. | IC-9 |
| I-8 | A world replay never calls a model: the journal holds every utterance. | the existing restart and replay tests, run with cognition absent |
| I-9 | A new System Pack needs no cognition code to be perceived and remembered (generic renderer). To be *attempted* by an LM seat it needs complete affordances (`ARC-34`), or an action known by name. | P4 test with a synthetic event type |
| I-10 | No provider concept appears in a World Pack, a contract, `mineworld-sdk`, a cassette key, the memory store or the decision context. A provider concept is a provider name, a model name, an endpoint or a key. | IC-10 |
| I-11 | The full core suite, Rust and Python, passes with no model reachable. Cognition tests run with outbound network blocked except to the test's own server. | IC-5 |
| I-12 | Every summary carries Event IDs, and every one resolves to a fact the seat perceived. | IC-4 (b) |
| I-13 | The context a model sees is bounded independently of the world's age. | IC-4 (a) |
| I-14 | At most one controller drives a Person at a time. | R-S11-4's test; P6 |
| I-15 | A model decides only social choices. Movement and routine are deterministic. | P6 test: `Step` decisions never pass through a backend |
| I-16 | No secret is written anywhere: log, cassette, store or error message. | IC-10 includes a planted fake key that must not appear in any artefact |

---

# 6. Assumptions to verify in implementation, not in planning

| ID | Assumption | Verified in | If false |
| --- | --- | --- | --- |
| A-1 | Every person's place can be folded from presence's own facts: genesis placement, `arrived` and displaced arrivals. | P1, first commit: fold the 300-day save; compare with the `Presence` components at every snapshot | Seed the fold from the newest snapshot's `Presence` components, and fold forward from there |
| A-2 | Every fact caused by a request carries `provenance.controller_decision`, including facts caused by reactions. `kernel/src/dispatch.rs:696` sets it on one path. | P6 | The decision log links by `caused_by` instead, which `inspect` already resolves |
| A-3 | Ollama's `/v1` honours `response_format` with a JSON Schema for the chosen model. | P5 spike | Add `OllamaNativeBackend` (`format=`), as QS10-3 provides |
| A-4 | A local model answers a social decision within 15 s at the 95th percentile on the operator's machine. | P5 spike, on the operator's hardware | Raise `max_decision_age` within the spike's evidence, or choose a smaller model (QS10-2) |
| A-5 | Facts per seat per simulated day stay small enough that SQLite ingestion keeps pace with a hosted world, which runs one simulated second per wall second. | P4: measure ingestion over the 100-day export | Batch ingestion per `perceived` frame; it is already per frame |
| A-6 | `spoke` lists the speaker among `participants`, so a speaker perceives their own words. **Verified:** `conversation/src/system.rs` `.with_participants(both)`. | — | — |

---

# 7. Change amplification and the two gate questions

**Change amplification** (`ENGINEERING_STANDARDS.md` §8). Adding LM control adds two modules,
`sdk/python` and `cognition/lm-controller`, plus one System Pack (`persona`) and contracts that are
seams rather than edits. It edits:

- **presence**: one new file. The job `observe.rs` already names as missing.
- **server**: one new seam beside `Perception`.
- **the CLI**: one new command; `agent.rs` passes perceived events.
- **the rule controller**: F-13.

It edits no `Person` code, no unrelated system, no renderer, no scheduler and no kernel. The routine
policy in Python repeats a policy, not a rule (§3.4.6). If a later PR finds it must edit an unrelated
system to make an LM seat work, that is a stop and an architecture question, not a patch.

**Gate 1: a Minecraft-like 3D client without redesigning the kernel?** Yes. Cognition never sees
geometry beyond the observation's `Location`. The 3D client's `talk` is the same request, and Alice's
reply is a `spoke` fact that any client renders.

**Gate 2: 2D and 3D without duplicating game logic?** Yes. Neither client knows a controller is a
model. Both show `spoke` lines as they do today. Milestone D's test has the two clients do identical
things after acquisition, as `AC-13` requires.

**`AC-1` and the I-2 vocabulary scan.** P1's presence and server changes are framework changes. They
must name no market concept, and `precursor_vocabulary` checks that. `persona` is a pack under
`systems/` with content under `worlds/`, the path `AC-1` measures, and it touches no transformation
range.

---

# 8. Files touched, by PR (proposed; each PR's design confirms against source)

| PR | Adds | Edits |
| --- | --- | --- |
| P1 | `systems/presence/src/audience.rs`; `tools/cli/src/perceived.rs`; `tools/cli/tests/perceived.rs` | `systems/presence/src/lib.rs`; `server/src/perception.rs` (the `EventPerception` seam, default `PerceivesNoEvents`); `server/src/lib.rs` (re-export); `tools/cli/src/main.rs`, `tools/cli/src/perceive.rs`; `persistence/src/world.rs` (`facts_after`, if `biography`'s direct read is not reusable); `docs/MODULE_SPEC.md` §8.1 (`perceived`) |
| P2 | — | `cognition/rule-controller/src/lib.rs` and its tests; `tools/cli/src/agent.rs`; `tools/cli/tests/` (F-13 regression) |
| P3 | `sdk/python/**` (`pyproject.toml`, `uv.lock`, `src/mineworld_sdk/**`, `tests/**`, `README.md`) | `.structured-coding/standards.md` (enable `ruff`, `pyright`; add `pytest`); `.gitignore` (Python caches) |
| P4 | `cognition/lm-controller/**` skeleton; `memory/`, `compress/`; `tests/test_ac10.py`; a fixture generator that runs `mineworld run` and `mineworld perceived` | — |
| P5 | `backend/`, `record.py`, `budget.py`, `config.py`; `tests/cassettes/`; the provider-concept scan | — |
| P6 | `context.py`, `controller.py`, `__main__.py`, `examples/local.toml`; `systems/persona/**`; `persona:` sections in `worlds/social-cafe/people/*.yaml` | `systems/installed` (one line per manifest); `worlds/social-cafe/world.yaml` (`systems:` gains `persona`); `docs/MODULE_SPEC.md` §4.1 (`persona:` section) |
| P7 | `tools/cli/tests/milestone_d.rs`; `cognition/lm-controller/tests/cassettes/milestone_d.jsonl`; the removal check (IC-7) | `docs/HUMAN_REVIEW_QUEUE.md` (Milestone D's review package) |

---

# 9. PR split, dependencies and parallelism

PR labels are placeholders (P1 … P7). The primary session numbers them.

```text
            S11 protocol revision (R-S11-1 … R-S11-7)
                         │
P1 perception (Rust) ────┼──► P2 F-13 (Rust)
   │                     │
   │                     ▼
   │                P3 Python SDK ──► P5 backends · recorder · budgets
   │                     │                         │
   └──► P4 memory · compression · AC-10 ◄──────────┘ (P4 needs P3's package skeleton only)
                         │
                         ▼
                    P6 LMController · persona ◄── P5
                         │
                         ▼
                    P7 Milestone D ◄── S12 (2D) and S14 (3D) for the operator's run
```

| PR | Scope | Depends on | Can start before the clients? | Integration checkpoint |
| --- | --- | --- | --- | --- |
| **P1** Event perception | presence's audience rule and `Whereabouts`; the `EventPerception` seam; `mineworld perceived` | nothing new | **Yes**, now. No S11 or client dependency | IC-1 (offline half), IC-9 |
| **P2** F-13 closed | `RuleController` answers from perceived facts; `agent.rs` passes them | P1; the in-process host's perceived channel (R-S11-1, host API half) | Yes, after S11's host-API change | IC-2, IC-9 |
| **P3** Python SDK | `uv`, `ruff`, `pyright`, `pytest`; wire models with golden frames; the seat session; the `Controller` protocol; an echo controller against a spawned server | S11's protocol revision (tokens, perceived frames, golden frames). Revision 1 frames may be done first | Yes, after S11's protocol PR | IC-1 (live and resumed halves), IC-5 |
| **P4** Memory and compression | store, renderers, L0–L3, structural summarizer, retrieval; `AC-10` | P1 (offline export); P3's skeleton | **Yes.** It needs no server and no client: it runs on a `run` save | IC-4 |
| **P5** Backends, recorder, budgets | `ModelBackend`, the OpenAI-compatible adapter, scripted and replay backends, cassettes, budget gate, the provider-concept scan; a live spike on the operator's machine (optional, QS10-2) | P3 | Yes | IC-5, IC-10; the P5 spike's A-3 and A-4 |
| **P6** `LMController` and `persona` | triggers, routine policy, context, `SocialChoice`, revalidation, decision log; the `persona` pack and social-cafe content | P3, P4, P5; S11 (seat exclusivity, tokens) | Yes. Tested through protocol-level clients | IC-3, I-5, I-6, I-14, I-15 |
| **P7** Milestone D | `milestone_d.rs` (protocol-level `2d`- and `3d`-tagged clients, SIGKILL, restart), the `AC-4` swap, the removal check; Milestone D's review package | P6; **S12 and S14** for the operator's run with the real clients; S13 for the Python CI job | **The automated test can. The operator's demonstration must wait** for both clients | IC-6, IC-7, IC-8 |

**Under the operator's 2026-10-08 schedule** (implementation after the clients), every PR waits.
What the table adds is that P1 and P4 depend on nothing in flight, so if the operator wants risk
retired early, they are the ones to bring forward (QS10-1).

**Parallelism with the other steps.**

| Step | What it means for S10 | What S10 needs from it |
| --- | --- | --- |
| S11 server/protocol | owns every server and protocol change S10 needs | R-S11-1 … R-S11-8 (§11), ideally in its protocol revision, so the protocol is revised once |
| S12 2D client | renders `spoke` lines as today; may opt in to perceived events for a conversation log | nothing new; a runnable client for P7's operator demo |
| S13 CI | a Python job: `uv sync`, `ruff`, `pyright`, `pytest` with network blocked; the Rust `milestone_d` test needs Python on the runner | R-S13-1: the Python job (§11.2) |
| S14 3D client | nothing new | a runnable client for P7's operator demo |
| S15 bodies | 12d re-baselines the 300-day digests; I-7 compares against `main` at merge time | nothing |
| Milestone E (package composition) | `persona` (a System Pack) and `LMController` (a Controller Pack) are natural examples of independently installable packs | coordination only: if E introduces a Controller Pack manifest, `LMController` adopts it then |

---

# 10. Integration checkpoints and adversarial criteria (decided before measuring)

Each checkpoint states its pass criterion and the mutation that must make it fail. A checkpoint
whose mutation does not fail it has not tested anything (`ARC-23`).

| ID | Checkpoint | Pass criterion | Mutation that must fail it |
| --- | --- | --- | --- |
| IC-1 | Perception is one function | **P1:** `mineworld perceived` over a 30-day social-cafe save includes five **located** overheard `spoke` facts in the café (Alice not a participant) and excludes five located `spoke` facts in the park while Alice was in the café. **P3:** a protocol client's live stream, killed at a random revision and resumed with its cursor, concatenates to exactly the offline export | `Place(p)` admits everyone: the park facts appear. The cursor is resumed one id late: a fact is missing |
| IC-2 | F-13 regression | `--agent alice`, the server SIGKILLed after a player's line and Alice's answer, then restarted. Alice's `spoke` facts after the restart equal the player's lines after the restart: zero re-answers | restore the `answered` map in place of the perceived check: one extra `spoke` |
| IC-3 | Revalidation under delay | the scripted backend answers after 3 s; the client leaves `talk`'s range during the delay; no `spoke` from Alice after the departure, in the save; the seat's log shows a pre-submit drop or a `TooFarAway` | the pre-submit check is removed **and** the server is made to skip validation in a scratch build: the test fails. The first alone must still pass, which proves the world's check is the one that holds |
| IC-4 | `AC-10` | §3.9.5 (a)–(e) | feed the objective log (fails b, d); disable the L1 roll-up (fails a); drop one episode's ids (fails c) |
| IC-5 | No model reachable | the full `cargo test` and the full `pytest` pass with outbound network blocked, except to the test's own server on localhost | a test that constructs `OpenAICompatibleBackend` against a public URL fails loudly with a network-guard error, not with a timeout |
| IC-6 | `AC-4` | the Milestone D scenario replayed from one cassette under two backend bindings: `worlds/` byte-identical, keys identical, submitted requests identical | put the backend's model name into the key: the replay under the second binding misses |
| IC-7 | Removability | with `cognition/lm-controller/` deleted in a scratch tree: `cargo test` passes, the SDK's tests pass, and `milestone_d.rs`'s step 8 (Alice unplugged) passes | make any Rust crate or the SDK import from `mineworld_cognition`: the build fails |
| IC-8 | Milestone D | §3.14.2 steps 1–8 | the store is deleted between steps 4 and 5 **and** re-ingestion disabled: the 2D ids are absent from the context, and the test fails naming them |
| IC-9 | Byte-identity | social-cafe and market-town 300-day digests equal `main`'s at merge time; every recorded observation transcript is unchanged | put perceived events into `Observation.events`: the transcripts differ |
| IC-10 | No provider concept, no secret | a scan of `mineworld-sdk`, `cognition/lm-controller` outside `backend/` and `config.py`, every cassette key, the store schema and `worlds/` finds no provider or model name. A fake key planted in the environment appears in no artefact of a full P7 run | plant `"ollama"` in `context.py`: the scan fails at that file and line |

---

# 11. Requirements on other steps

## 11.1 On S11 (server and protocol), with exact shapes

Proposed for S11's protocol revision. S11 owns the server and the protocol, so S10 implements none of
these itself, which keeps the protocol to one revision (QS10-15). Shapes are JSON as `PROTOCOL.md`
writes them, and S11 may rename fields while preserving the semantics.

**R-S11-1 — Perceived events: an opt-in, reliable, ordered stream.**

```json
{ "t": "join", "seat": "alice", "token": "…", "perceived": { "since": "1873" } }
{ "t": "join", "seat": "alice", "token": "…", "perceived": { "since": null } }
```

- Without `perceived`, no `perceived` frame is ever sent. Every existing client is unchanged.
- The server frame:

```json
{ "t": "perceived", "through": "1907",
  "events": [ { …an EventEnvelope, as PerceivedEvent<serde_json::Value>… } ] }
```

- `events` holds facts admitted by the `EventPerception` seam for this observer, in ascending
  `EventId`, never reordered, never dropped, never duplicated within a connection.
- `through` is the highest `EventId` the server has *considered* for this subscriber, whether
  admitted or not. A client uses it as its cursor, so the cursor advances through facts it was not
  shown.
- **Ordering guarantee:** every `perceived` frame covering the facts of revision R is sent before any
  `observation` frame carrying revision R. A controller then never sees a world newer than its memory.
- **Flow control:** a separate bounded queue. On overflow the server sends
  `{ "t": "refused", "code": "lagged" }` and closes. It never drops silently, and the client resumes
  with its cursor.
- Ids are decimal strings and payloads JSON values, as `PROTOCOL.md` §7 and §5 already state.
- **The host API half.** The in-process `Seated` handle gains the same stream, so the `--agent`
  driver (P2) receives what a WebSocket client receives.

**R-S11-2 — Resume.** `since: "<id>"` delivers every admitted fact with an id greater than `<id>`,
then continues live. `since: null` delivers from the beginning of what the world holds.

- A persisted world serves from genesis (its fact table). A world without a save serves from its
  recent window.
- A cursor older than what can be served is refused `{ "code": "cursor_unavailable" }`. The server
  never sends a partial history.
- Needs a streaming fact read after an id from the save. P1 adds it to `persistence` if `biography`'s
  read is not reusable (§8); S11 uses it.

**R-S11-3 — The audience is decided by the seam, never by the server.** The server calls
`EventPerception` (P1) and decides nothing itself. Default `PerceivesNoEvents`. `PackPerception`
wires presence's.

**R-S11-4 — Seat exclusivity and takeover (`AC-5`).**

- At most one live connection holds a seat. A second `join` is refused `seat_taken`, unless it asks
  to take over: `{ "t": "join", "seat": "alice", "token": "…", "take_over": true }`.
- The evicted holder receives `{ "t": "released", "reason": "taken_over" }` and is closed.
- Who may take over is S11's authentication policy. S10 needs only that the evicted seat is told
  distinctly, and that a re-join by the evicted cognition is refused `seat_taken` while the human
  holds the seat. The LM seat then backs off and retries at a slow interval, so the NPC resumes when
  the human leaves (`AC-5`'s spirit: the Person persists across controllers).

**R-S11-5 — Authentication without privilege.** Cognition authenticates exactly as a player does
(`NETWORKING.md` §9: an invite token, plus a nickname, e.g. `cognition`). No controller-only
credential and no controller-only frame exist. The token is read from an environment variable the
operator names.

**R-S11-6 — No change to pacing or cadence is required.** One simulated second per wall second, and
10 Hz observations, are fine. Cognition reads `at` and never assumes a rate (`PROTOCOL.md` §8).

**R-S11-7 — Golden frames.** S11's tests write one example of every frame kind, client and server, to
`server/tests/frames/<kind>.json`, and fail if a frame type changes without its example. The Python
SDK's conformance tests read them (§3.4.2). This is `overall.md` R-9's rule, applied to Python: every
cross-language boundary is verified from the far side.

**R-S11-8 — The `--agent` driver keeps working.** `mineworld server --agent SEAT` stays, now passing
perceived events to `RuleController` (P2). Under R-S11-4 it holds the seat exclusively, like any
client.

## 11.2 On S13 (CI)

**R-S13-1.** A Python job: `uv sync --locked`, `ruff check`, `ruff format --check`, `pyright`,
`pytest`, with outbound network blocked except localhost. The Rust job that runs
`tools/cli/tests/milestone_d.rs` needs the same Python environment on the runner. Neither job may need
a model or a key (I-11).

## 11.3 On S12 and S14 (the clients)

Nothing new. They render `spoke` lines as today, and the 2D client may opt in to perceived events.
Milestone D's operator demo (§3.14.3) needs both clients runnable against a persisted world.

---

# 12. Proposed edits to documents this session may not edit

Applied by the primary session at freeze, or by the PR named.

| Document | Edit | When |
| --- | --- | --- |
| `overall.md` §3 S10 | Add: "**2026-10-08 (operator):** the LM half is designed in `step-17-cognition.md` and implemented after the clients, as PRs P1–P7", with QS10-1's outcome; link the step | at this step's freeze |
| `overall.md` §4 | `AC-4` → "S10 (step-17), P7"; `AC-10` → "S10 (step-17), P4"; a row for Milestone D → "S10 (step-17), P7, with S12 and S14" | at freeze |
| `overall.md` §7 | The S10 line, under "Remaining"; F-13 "closed by P2" once merged | at freeze, then at P2's merge |
| `MVP_STATUS.md` | An S10 row: designed, and implementation scheduled after the clients | at freeze |
| `DECISIONS.md` | `ARC-S10-a` process boundary (§3.2) · `ARC-S10-b` event perception (§3.3) · `ARC-S10-c` compression with retained ids (§3.9) · `ARC-S10-d` budgets (§3.10) · `ARC-S10-e` recorded model outputs (§3.11) · `DEP-S10-a` … `DEP-S10-f` (§4.8) | each with the PR that implements it; `ARC-S10-a` at freeze |
| `ARCHITECTURE.md` §13.1 | G-1: the first cross-language boundary is the JSON client protocol; Python mirrors it as typed models held by golden frames; Protobuf/gRPC declined until an encoding need is measured | P3 |
| `ARCHITECTURE.md` §8 | G-6: `cognition_cache/` leaves the save layout; the cognition store lives in the operator's cognition directory, because it is not world state | P4 |
| `ARCHITECTURE.md` §6 | Name the `EventPerception` seam and presence's audience rule as the mechanism for event `Visibility` | P1 |
| `CORE_CONCEPTS.md` §5.4 | G-2: "A controller normally reads the summary" → "A controller normally reads the compression of **its own perceived history**; the objective biography's compression is a tool's view" | P4 |
| `MODULE_SPEC.md` §5 | `LMController` is a Controller Pack running as a protocol client; operator configuration binds seats and backends; a World Pack binds none | P6 |
| `MODULE_SPEC.md` §8.1 | the `mineworld perceived` command | P1 |
| `MODULE_SPEC.md` §4.1 | the `persona:` section, owned by `persona` | P6 |
| `server/PROTOCOL.md` | revision 2 (S11's document) | S11 |
| `.structured-coding/standards.md` | enable `ruff`, `pyright`; add `pytest` | P3, as that file instructs |
| `docs/HUMAN_REVIEW_QUEUE.md` | Milestone D's review package (`ACCEPTANCE.md` §6 shape) | P7 |

---

# 13. Questions

Operator-material questions are marked **[operator]**: provider and cost choices, anything needing a
paid API, scope, and changes to the 2026-09-25 decision. The rest the primary session may decide, and
each carries a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QS10-1 [operator — scope, changes the 09-25 decision]** | Where does this land? (a) Inside MVP-0, as its last step after S14, with `AC-4`, `AC-10` and Milestone D re-entering MVP-0's gates. (b) As MVP-1's first step, leaving the 09-25 decision as it stands. Also: bring P1 and P4 forward, since they depend on nothing in flight? | **(a)**, with P1 and P4 brought forward. Milestone D is already in the framework milestone list beside A–C and E. P1 and P4 are headless and retire the two largest unknowns (perception's correctness; whether `AC-10`'s bound holds over 100 days) without touching the clients' critical path. |
| **QS10-2 [operator — provider and cost]** | Which local model, and on what hardware, is the default `social` binding? | Decided by the P5 spike on the operator's machine. Criteria fixed now: schema-valid ≥ 95 % of decisions on the scenario set; 95th-percentile latency ≤ 15 s; Ollama; no key. The model name lives in `examples/local.toml` only. |
| QS10-3 | Ollama through `/v1` only, or also a native adapter? | `/v1` only, through the `openai` SDK; add `OllamaNativeBackend` only if A-3 fails in the spike. |
| **QS10-4 [operator — paid API]** | Is a hosted OpenAI-compatible endpoint ever exercised live: to record cassettes, or for a live `AC-4` variant? It needs the operator's key and spend. | Optional and operator-run only, never in CI. At most one recording session per cassette change, with the spend reported in tokens. The default demonstration needs no paid API. |
| **QS10-5 [operator — provider]** | `MVP.md` §6 says "Ollama plus one generic OpenAI-compatible endpoint". Does Ollama's own `/v1` count as the second? | No. Demonstrate the generic adapter against a second, free, local OpenAI-compatible server (for example llama.cpp's server; its compatibility is *not verified* here), so `AC-4` is shown across two real servers without a paid API. A hosted endpoint is QS10-4's optional extra. |
| QS10-6 | Admit `cognition_profile` or `requires:` capabilities in `world.yaml`? | No. Keep them refused until a world needs to declare a capability. A World Pack binds no cognition (`AC-4`). |
| **QS10-7 [operator — scope]** | Add a `persona` System Pack, and author personas for social-cafe's NPCs (and Market Town's)? | Yes for social-cafe in P6. Market Town only if Milestone D's demo uses it. Character is world content (`CORE_CONCEPTS.md` §4.2), not controller configuration. |
| QS10-8 | How much routine policy does an LM seat get in Python? | Minimal: agenda following, invitation answers, idling. Never a model. No port of the paced bands, and no second controller on the seat. |
| QS10-9 | With no model (unbound, exhausted, unreachable), should an LM seat speak deterministic phrases or stay silent? | Deterministic short phrases that never claim a memory. Silence reads as a broken NPC. The Rust template is `AC-15`'s evidence and stays in the rule controller. |
| QS10-10 | Embedding retrieval and beliefs (Graphiti- or generative-agents-style), now or later? | Later, after Milestone D, behind `Retriever`. Both put a model or an embedding service on the memory path. |
| QS10-11 | The context ceilings (memory ≤ 6 000 bytes; L0 ≤ 48, L1 ≤ 8, L2 ≤ 4, L3 ≤ 16) | Accept as stated. P4 may tune them on the 100-day export **before** IC-4 is run, then freeze them; never after measuring. |
| QS10-12 | Wire models hand-mirrored with golden frames, or generated (`schemars` → JSON Schema → Pydantic)? | Hand-mirrored with golden frames now; generation when the frame vocabulary outgrows review. |
| QS10-13 | A moderation filter before submit? | An empty `Filter` seam now; private worlds only in MVP-0. Fill it before any public world (Phase 4). |
| **QS10-14 [operator — scope]** | Should `mineworld run` (headless) ever drive seats with an LM, for example from a cassette? | No. `AC-10` is shown on a rule-driven history, and an LM in headless runs adds cost, not acceptance evidence. |
| QS10-15 | Who implements R-S11-1 … R-S11-8: S11 in its protocol revision, or S10 later? | S11. It owns the server and the protocol, and doing it once avoids a second protocol revision. If S11 declines, P3 inherits them, and the protocol revises twice. |
| QS10-16 | A Python job in CI (R-S13-1)? | Yes, in S13. Without it, I-11 and IC-5 are review promises rather than gates. |
| QS10-17 | Where does the cognition store live? | In the operator's cognition directory, keyed by world instance and seat, never inside the world's save (§3.8.4). |

---

# 14. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-S10-1 | A local model's dialogue is poor or slow, and Milestone D reads worse than the template. | P5's spike measures quality and latency against fixed criteria (QS10-2) **before** P6 builds on them. The fallback is deterministic speech. The demo's acceptance is `recalls` plus the operator's judgement, never a score the model gives itself. |
| R-S10-2 | Small models produce invalid structured output often. | Endpoint JSON-Schema mode, one repair, then fallback. The spike's validity rate is a criterion. |
| R-S10-3 | Prompt changes invalidate cassettes, and tests fail with misses. | Most tests use `ScriptedBackend`. Cassettes are few. A miss shows the first differing field. Re-recording is an operator task, since it needs a model. |
| R-S10-4 | Perceived-event volume: every stride in the café is an `arrived` fact visible to everyone there, so Alice perceives tens of thousands over 100 days. | Ingestion collapses within-place strides into episodes cheaply. SQLite holds them. Context stays bounded by I-13. A hearing-range or salience perception system is the later refinement, behind the seam. |
| R-S10-5 | Two controllers drive Alice before R-S11-4 lands. | P6 refuses to run without seat exclusivity: it checks the server's protocol revision at `welcome`. |
| R-S10-6 | A hosted world runs one simulated second per wall second, so 100 simulated days cannot be shown hosted. | `AC-10` is shown on a `run` save through the offline export, the same audience function (I-4). |
| R-S10-7 | An observation newer than memory makes a seat answer from a stale past. | R-S11-1's ordering guarantee, and the pre-submit check (§3.6). |
| R-S10-8 | The project drifts toward the "AI NPC demo" its vision rejects (`VISION.md` §1.1; `MVP.md` §11: *"a demonstration of clever NPC dialogue"* is a non-goal). | `LMController` is one Controller Pack. Milestone D's test includes the unplugged run. No world rule, contract or client depends on a model (§3.16, I-1, I-2, IC-7). |
| R-S10-9 | Persona text is used to smuggle rules ("always gives free coffee"). | It can only shape speech: an LM seat can attempt only what systems offer (`INV-10`). `MODULE_SPEC.md` §4.1's `persona` entry says so. |
| R-S10-10 | The model's words claim memories it was not given, even when `recalls` is valid. | A known limit of free text. The context carries ids and the prompt asks for no invention. Evaluation is by the operator's run; the test checks citation, not truth of every clause. |
| R-S10-11 | S15's 12d re-baselines the 300-day digests while S10 PRs are open. | I-7 compares against `main` at merge time, never against a digest frozen in this document. |
| R-S10-12 | Python enters a Rust repository and its tooling rots. | `uv` lock; `ruff` and `pyright` strict and enabled the day the first module lands; a CI job (R-S13-1); two small packages with one-way dependencies. |
