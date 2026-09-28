# PR 05d — The client protocol, the agent-driven Person, and `AC-15`

## DESIGN FROZEN

Design revision: `step-05-vertical-slice.md` §2 "PR 05d — The clients, and `AC-15`", plus §1.3's
frozen invariants and `docs/MVP.md` §9 `AC-13`, `AC-15`, §9.1 and §9.2. This file is the PR-level
ledger for that frozen scope.
Approved by / evidence: operator kickoff of this implementation session (2026-09-27), which states
the deliverables ("the client protocol module, a minimal scene that uses it, and the `AC-15`
acceptance test"), the two open decisions it delegates (how Alice's history reaches a controller;
where the `AC-13` comparison lives), the objective gates, and the publication authority verbatim.
Implementation base: `main` @ `cc40ad5`
Branch: `mvp0/pr-05d-clients-ac15`
Lifecycle: **READY FOR OPERATOR REVIEW** — implementation, validation and review complete; branch
pushed; no PR opened and no merge, as the kickoff directs. The exact final head is the tip of
`mvp0/pr-05d-clients-ac15`, which `git log --oneline` is authoritative for: a commit cannot carry its
own hash, and §9.1's gate results were produced at that tip.

Continuation state: **this file**. `.structured-coding/plans/mvp0/handoff.md` is still PR 02's;
PR 05c flagged it as stale and asked whoever started PR 05d to replace it rather than read it. This
session read it, confirmed it describes PR 02, and replaced it (§6 C7).

---

## 1. What this PR is

Three artefacts and two decisions.

```text
clients/protocol/        a Godot 4 client protocol module — drop-in for the 2D and the 3D client,
                         with a tiny demonstration scene of its own that runs against the real server
cognition/rule-controller/  a RuleController: the agent-driven Person AC-15's third participant is
tools/cli/               `--agent <seat>`, which runs that controller against the hosted world
tools/cli/tests/         the AC-15 and AC-13 acceptance tests, against the real binary and the real pack
```

The two decisions the kickoff delegates:

1. **How Alice's history reaches a controller** (§4). `ConversationSystem` already owns
   `ConversationHistory` and writes it only while reducing `spoke`, so the memory exists; what does
   not exist is a way for a controller to read it, because `mineworld_presence::observe` exposes no
   component records at all. The answer has to expose *this* state without turning an `Observation`
   into a window onto everything (`INV-13`).
2. **Where the `AC-13` comparison lives** (§5). `MVP.md` §9 requires it to be defined once, in the
   server, so that a later test cannot quietly compare a different set of fields.

What this PR is judged on, restated from the step document and `MVP.md` §9.1:

```text
A1  two clients and an agent-driven Person alive at once against ONE server
A2  something done in one window is carried forward by an NPC met in the other
A3  the evidence names identity, not appearance:
        same world instance · same Alice EntityId · same authoritative event sequence
A4  a Talk from a 2D click and a Talk from a 3D walk-up-look-press have an identical semantic
    core — actor, action_type, target, payload — resolved by the same system to the same result
A5  the client protocol module is drop-in for both visual clients, and a scene of this PR's own
    proves it against the real server
```

## 2. Implementation contract

```text
PROJECT / PR:        MVP0 PR 05d — The clients, and AC-15
PRIMARY DESIGN DOC:  this file
RELATED / BINDING:   .structured-coding/plans/mvp0/step-05-vertical-slice.md §1.3, §2
                     .structured-coding/plans/mvp0/pr-05a-conversation-presence.md
                     .structured-coding/plans/mvp0/pr-05b-server.md
                     .structured-coding/plans/mvp0/pr-05c-world-packs.md
                     docs/MVP.md §7, §9 (AC-13, AC-15, §9.1, §9.2)
                     docs/ACCEPTANCE.md §§1, 4.1
                     docs/CORE_CONCEPTS.md §6.1 (the spatial frame), §14 (Controller), INV-1/6/13
                     docs/MODULE_SPEC.md §5 (Controller Pack) and its five hard constraints
                     docs/ENGINEERING_RULES.md §§3, 7-9 (a client reports intent and evaluates no rule)
                     docs/NETWORKING.md, server/PROTOCOL.md
                     spike/FINDINGS.md F1-F9 — what a real renderer already measured
                     contracts/src/{observation,action,spatial,ids,component}.rs
                     systems/presence/src/{observe,interaction}.rs, systems/conversation/src/*
                     server/src/{protocol,session,host,runtime,perception}.rs
                     worldpack/src/{catalog,load}.rs, tools/cli/src/{main,perceive}.rs
IMPLEMENTATION BASE: main @ cc40ad5 — 05a, 05b, 05c merged; 237 tests green at that HEAD
BRANCH:              mvp0/pr-05d-clients-ac15

APPROVED SCOPE:
  clients/protocol/ (new), cognition/rule-controller/ (new), the AC-15 and AC-13 acceptance tests
  in tools/cli/tests/, the `--agent` flag and its driver in tools/cli/, the component-disclosure
  seam in systems/presence + systems/conversation and the two call sites it touches
  (worldpack/src/catalog.rs, tools/cli/src/perceive.rs), the world-instance identity and the
  AC-13 comparison in server/, worlds/social-cafe/ content (a second visitor, and Alice as a
  seat), the workspace manifest, and the documentation those force
  (server/PROTOCOL.md, the new crates' READMEs, worlds/social-cafe/README.md).

NON-GOALS:
  the 2D and the 3D client themselves      → vis/2d-generated-assets and vis/3d-human-pipeline,
                                             which this PR must not merge and must not edit
  events in an Observation                  → §4.4: not needed for AC-15 and left to S10
  a scheduler, durable persistence, travel  → S4, S5, later
  an LM controller                          → MVP-1; the kickoff says a RuleController is enough
  a `controllers:` field in a World Pack    → the pack format is PR 05c's frozen contract; §7.3
  authentication beyond a seat name          → PROTOCOL.md §9

FROZEN INVARIANTS:
  - AC-15 evidence names identity, never appearance: same world instance, same Alice EntityId,
    same authoritative event sequence. "Both clients showed the same thing" is not evidence.
  - Alice remembers through a projection of the event log, not a memory system. Nothing in this
    PR may write ConversationHistory anywhere but ConversationSystem::react.
  - No world rule in a client. Distance, availability, permission and system presence are
    server-decided; the Godot module evaluates none of them and computes no affordance.
  - A client may act and may never assert: the protocol's two frames stay two.
  - The wire carries 64-bit ids as decimal strings; the module must parse them as strings and
    never as numbers, and must never re-implement the encoding.
  - INV-13: a component reaches an observation because its owning pack named it and named who
    may see it — never because it happens to exist.
  - INV-1: the agent-driven Person is reached through the same seat, the same actor check and
    the same server-allocated ActionId a human client's request goes through.
  - Dependencies point one way. The server crate gains no dependency on a System Pack, on the
    loader or on a controller.
  - fmt, check, clippy -D warnings and cargo test --workspace clean at the final HEAD.

ENDPOINT AUTHORITY:
  implementation + local validation  authorized   source: operator kickoff, this session
  semantic commits                   authorized   source: operator kickoff ("Commits and pushes
                                                   authorized")
  branch push                        authorized   source: operator kickoff
                                                   ("git push -u origin mvp0/pr-05d-clients-ac15")
  PR creation / merge                FORBIDDEN    source: operator kickoff ("No merge, no PR")

TEST OWNERSHIP:
  STATIC       fmt, check, clippy -D warnings. clippy.toml's HashMap/HashSet ban keeps the
               controller's own bookkeeping order-deterministic.
  UNIT         the AC-13 semantic-core comparison (which fields it reads, and that it reports
               actor_location as the one permitted difference); the world-instance identity's
               string form; the RuleController's decision over hand-built observations, including
               the cases where it must NOT act (no available affordance, nothing new heard).
  INTEGRATION  the disclosure seam, in systems/presence and systems/conversation: a person's own
               history is disclosed to that person and to nobody else, and a disabled pack
               discloses nothing.
  GATE 2       (this PR's real evidence, and the only honest owner of AC-15) the real `mineworld`
               binary hosting the real World Pack, with two real WebSocket clients and the agent
               alive at once, and the identity evidence read off the frames they receive.
               Separately: the real Godot 4.7.2 client protocol module, run by the real engine
               against that binary, headless for the transcript and windowed for the scene.
  GATE 1 (real LLM): NOT REQUIRED — nothing here is LLM-facing, and MVP-0 has no model.
  MUTATION     the AC-15 assertions specifically: an evidence test that still passes when there
               are two Alices is worthless, so §9.4 records what was broken to prove it fails.

NORMAL STOP CONDITION:
  PR 05d implemented, validated, committed and pushed. NO PR. NO MERGE.
```

---

## 3. Source audit

Read in full before writing: `CLAUDE.md`; the `structured-coding` SKILL, `agent-workflow.md`,
`adaptation.md`, `implementation-working-rules.md`, `test-ci-gate-rules.md`;
`.structured-coding/standards.md`; `docs/ACCEPTANCE.md`; `step-05-vertical-slice.md`;
`docs/MVP.md` §9; `spike/FINDINGS.md`; `server/PROTOCOL.md`; every source file of `server/src`,
`systems/presence/src`, `systems/conversation/src`, `tools/cli/src`, `worldpack/src/catalog.rs`;
`contracts/src/observation.rs`, `contracts/src/spatial.rs`, `contracts/src/ids.rs`;
`worlds/social-cafe/**`; `server/tests/support/mod.rs`; `tools/cli/tests/server_command.rs`;
`docs/CORE_CONCEPTS.md` §6.1 and §14; `docs/MODULE_SPEC.md` §5; `docs/ENGINEERING_RULES.md` §§1-9.

Findings that changed the plan:

| # | Finding | Consequence |
| --- | --- | --- |
| F-A | `mineworld_presence::observe` exposes **no** component records and **no** events, and says why: exposing a component because it happens to exist is how an observation becomes a window onto everything. | §4's seam has to be an owner-declared disclosure, not a widening of perception. |
| F-B | `observe` returns `Observation<Vec<u8>>`, and `tools/cli/src/perceive.rs::wire()` converts it to `Observation<Value>` by a serde round trip — with a comment recording that the day a component *is* exposed, its payload would reach a client as F8.2's array of byte integers, and that the fix belongs in the perception system. | The disclosure produces `ComponentRecord<Value>` and `observe` returns `Observation<Value>`; `wire()` is deleted rather than extended. |
| F-C | `WorldSummary` carries no world identity at all. | `AC-15`'s first evidence line — *same world instance* — was unassertable. §5.1 adds it. |
| F-D | `worlds/social-cafe/world.yaml` offers exactly one seat (`visitor`), and Alice is deliberately not one — `tools/cli/tests/server_command.rs` uses her as its example of a refused seat. | Two clients need two seats, and the agent needs Alice to be one. §7.3 decides this and §6 C4 corrects that test to name a seat the world genuinely does not have. |
| F-E | `ConversationSystem::resolve` emits `Spoke` with `Visibility::Place`, and `react` writes `Heard` **on the listener**. | A client can see what an NPC said to *it* by reading its own disclosed history; nothing needs events in the observation (§4.4). |
| F-F | `systems/presence/tests/presence.rs` scans `presence/src/**` for the words `talk`, `conversation`, `spoke`, `utterance`, `f32`, `f64` and fails if any appears. | Every sentence the disclosure seam adds to that crate has to be written without naming the pack that uses it. That is the architecture claim, so the constraint is welcome. |
| F-G | `contracts` has no `ActionId`/`issued_at` on `ActionRequest` (PR 04), so of `AC-13`'s three legitimately-differing fields only `actor_location` survives — exactly as the kickoff says. | §5.2's comparison reports one permitted difference, and a unit test pins that it is one and not three. |
| F-H | `docs/CORE_CONCEPTS.md` §6.1 fixes the spatial frame (+x east, +y north, +z up, right-handed; yaw from +y toward +x). `spike/FINDINGS.md` F5 is the reason it exists, and the spike's two clients each invented their own conversion with a sign flip. | The module owns the conversion for both dimensions, once, so the two visual clients cannot disagree (§8.3). |
| F-I | `clients/2d-spike/` and `clients/3d-spike/` are two separate Godot projects on two branches. Godot cannot reference `res://` above a project root. | The module is a self-contained folder that a project copies or symlinks in, and `clients/protocol/` is itself a Godot project so the folder has a runnable home and a demonstration scene (§8.1). |
| F-J | `clippy.toml` forbids `HashMap`/`HashSet` workspace-wide. | The controller's own memory is a `BTreeMap`. |

---

## 4. DECISION — how Alice's history reaches a controller

### 4.1 The requirement, and the failure to avoid

`MVP.md` §9.2 fixes the shape:

```text
objective event log  →  a conversation-history projection for Alice  →  her controller's context
```

The first two arrows exist and are PR 05a's. The third does not: a controller receives an
`Observation` (`INV-13`), and `observe` puts no component records in one. So the question is not
*how does Alice remember* — she already does — but *how does what she remembers become something a
controller is allowed to read*, without the answer generalizing into omniscience.

The failure to avoid is named in `observe`'s own documentation: exposing *some* component because
it happens to exist. A perception system that walked the component stores and serialized what it
found would satisfy this PR and defeat `INV-13` for every pack that comes after.

### 4.2 The answer: the owning pack declares the disclosure, per observer

`systems/presence` already asks each pack a question it cannot answer itself —
`InteractionProvider::offers`, *which of your actions may this observer attempt against that
target*. State is the same shape of question with a different noun, so it becomes a second method
on the same seam, which is renamed to say what it is:

```rust
pub trait PerceptionProvider {
    fn offers(&self, read, observer, target) -> Vec<Offer> { Vec::new() }
    fn discloses(&self, read, observer, subject) -> Vec<ComponentRecord<Value>> { Vec::new() }
}
```

`ConversationSystem` implements the second method with one rule:

> a person's record of what they have been told is disclosed **only in that person's own
> observation**.

So Alice's history reaches Alice's controller and nobody else's, and a client reads what an NPC
said *to it* out of its own history — which is the same mechanism, not a second one.

Four properties, and each is why this is the answer rather than a convenient one:

```text
the owner decides        the pack that owns the component names it; presence cannot expose a
                         component it does not own, and does not know this one exists
the observer decides     the disclosure is computed per (observer, subject) pair, so "what I have
                         been told" cannot be read off a stranger
adding a pack adds a     no edit to perception, the server, the loader or a client
  method
removing a pack removes  a disabled pack is not asked, so its state leaves every observation in
  the disclosure         the world with no edit anywhere (AC-2, the same route the affordances take)
```

`INV-13` holds for the reason it holds for `entities` and `relations`: an `Observation` is a list of
what was exposed deliberately, and there is nothing in it to widen and nothing to ask again.

### 4.3 What this forces, and what it deletes

`ComponentRecord<Value>` cannot go into an `Observation<Vec<u8>>`, so `observe` now returns
`Observation<Value>`: a perceived payload is **self-describing data**, because its reader is a
controller or a client that does not have the Rust type. `Vec<u8>` is the right default for a log
and the wrong one for an observation, which is what `FINDINGS.md` F8.2 measured. That deletes
`tools/cli/src/perceive.rs::wire()` and the hazard its comment recorded, rather than extending it.

`serde_json::Value` in a System Pack is a serialization datum, not a transport concept: every pack
already encodes its own payloads with `serde_json` (`codec.rs`, `DEP-5`), and no simulation
semantics depend on it (`INV-14`). The alternative — making `observe` generic over the payload type
with a per-pack codec trait — is one instantiation pretending to be two, which
`ENGINEERING_STANDARDS.md` §28 forbids until the second real one exists.

### 4.4 Why events are still not in an observation

`Spoke` is emitted with `Visibility::Place`, so both clients in the café are entitled to it, and
putting perceived events in an observation would be a legitimate second answer. It is not needed:
`react` writes the `Heard` entry on the **listener**, so everything a client must see about what was
said to it is in its own disclosed history, and everything the `AC-15` evidence needs about the
event log is in the `EventId`s the server returns on each `result` frame. Events stay S10's, and
§10 records it as a follow-up rather than as an omission.

---

## 5. DECISION — the server owns the two things the evidence is made of

### 5.1 A world instance has an identity (`AC-15`'s first evidence line)

`WorldSummary` gains `instance: WorldInstanceId`, allocated once when a world's thread starts, and
reported identically by `GET /status` and by every `welcome`. It is not a UUID, not a secret and not
a name: it distinguishes *this running world* from another one, which is exactly what
"same world instance" has to mean for the evidence to exclude two servers pretending to be one.
It reaches the wire as a hexadecimal **string**, for the reason every identity does (`DEP-3`).

`MVP.md` §9.1's fourth line, *same persisted state revision*, has no referent until S5: this server
holds its world in memory. Recorded in §10, not faked.

### 5.2 The `AC-13` comparison is defined once, in `server/src/parity.rs`

```rust
pub struct SemanticCore { actor, action_type, target, payload }   // PartialEq
pub fn semantic_core(request: &ActionRequest<WirePayload>) -> SemanticCore;
pub fn differing_fields(a: &ActionRequest<WirePayload>, b: &ActionRequest<WirePayload>)
    -> Vec<RequestField>;
```

`semantic_core` is the four fields `MVP.md` §9 names and no fifth. `differing_fields` is the whole
request, field by field, so a test states which differences it is *permitted* to see rather than
silently ignoring the ones it forgot to compare. `RequestField` is a closed enum, so
`[RequestField::ActorLocation]` is the assertion and a new field on `ActionRequest` makes the
comparison fail to compile rather than quietly drop it.

It lives in the server because the server is what both clients talk to, and because a comparison
defined in a test can be redefined by the next test. `MVP.md` §9's correction asked for exactly
this.

---

## 6. Commit plan

Every commit: implementation, deterministic validation, and LLM logic review, tracked separately.

### C1 — the world instance identity, and the `AC-13` comparison

- [x] Implementation: `WorldInstanceId` and `WorldSummary.instance` in `server/src/protocol.rs`;
      allocation in `server/src/runtime.rs`; `server/src/parity.rs` with `SemanticCore`,
      `RequestField`, `semantic_core`, `differing_fields`; re-exports in `server/src/lib.rs`;
      `server/PROTOCOL.md` §5, §6 and §10 updated.
- [x] Validation (`cargo test -p mineworld-server`: 19 lib + 8 integration + 1 doc, all pass at
      this commit): unit tests in `server/src/protocol/tests.rs` and `server/src/parity.rs` — the
      instance id round-trips as a string and two allocations differ; the semantic core ignores
      `actor_location` and notices actor, action type, target and payload. `server/tests/two_clients.rs`
      gains the assertion that two connections to one world are told the same instance.
- [x] Review: the three `WorldSummary` constructions (`runtime::summary`, the welcome test, the
      two-clients helper) and its three consumers (`app::status`, `session::run`, `Seated::world`)
      all carry the instance unchanged; `parity::differing_fields` matches `RequestField`
      exhaustively, so a fifth field on `ActionRequest` is a compile error rather than a silent
      omission. Checked that `parity` names no domain action: its fixture action is
      `example-action`, owned by `example-system` (`INV-12`).

### C2 — the disclosure seam

- [x] Implementation: `InteractionProvider` → `PerceptionProvider` with `discloses`, in
      `systems/presence/src/interaction.rs`; `observe` returns `Observation<Value>` and calls the
      disclosure per perceived entity, in `systems/presence/src/observe.rs`;
      `ConversationSystem::discloses` in `systems/conversation/src/system.rs`; the rename at its two
      call sites (`worldpack/src/catalog.rs`, `worldpack/src/load.rs`); `wire()` deleted from
      `tools/cli/src/perceive.rs`.
- [x] Validation (`cargo test --workspace`: 244 tests pass, up from 237; three new tests in the
      conversation suite): `systems/conversation/tests/conversation_and_presence.rs` — a person's own history
      is disclosed to them, is absent from a stranger's observation of them, and disappears when the
      pack is disabled. `systems/presence/tests/presence.rs` still passes, including the structural
      scan that forbids this crate from naming another pack's vocabulary.
- [x] Review: `git grep` for `insert(.*ConversationHistory` finds exactly one write, in `react`;
      `discloses` takes a `WorldRead` and so cannot write at all. The disclosure is called only from
      `perceived`, once per entity the observation already lists, so it cannot be used to learn about
      an unperceived subject. DISCOVERY recorded in §7.6: a disabled pack was still disclosing,
      because `observe`'s `AC-2` filter was on the action route map only.

### C3 — the `RuleController`

- [x] Implementation: `cognition/rule-controller/` — a decision over an `Observation<Value>`
      returning at most one `ActionRequest`, with its own bookkeeping and no I/O.
- [x] Validation (`cargo test -p mineworld-rule-controller`: 6 tests pass): unit tests over
      hand-built observations: it replies to a new utterance, does not
      reply twice to the same one, does not act when the server's affordance is unavailable, and
      names the earlier speaker when there is one.
- [x] Review: `decide` takes one argument and it is an `Observation`; the crate's manifest depends
      on contracts, the conversation pack and `serde_json` and on no server, kernel or transport, so
      there is nothing else reachable to read or write. Availability comes from
      `Affordance::is_available` and there is no arithmetic over positions anywhere in the crate.
      `ActionRequest` has no `action_id` and no `issued_at` to set. The one piece of state it keeps
      is a `BTreeMap` of what it has already answered, which is about its own past actions rather
      than about the world, and is order-deterministic (`AC-12`).

### C4 — the agent driver, and the world it runs in

- [x] Implementation: `tools/cli/src/agent.rs` and `--agent <seat>`; `worlds/social-cafe/` gains
      `people/wanderer.yaml`, a second seat, and Alice as a seat; READMEs; the corrected seat-refusal
      test in `tools/cli/tests/server_command.rs`.
- [x] Validation: `mineworld validate worlds/social-cafe` prints the four people, three seats and
      four genesis facts; a live `mineworld server worlds/social-cafe --agent alice` logged
      `agent: driving 'alice' as entity 2` and `/status` reported `clients: 1` with the agent alone
      connected; `cargo test --workspace` 256 tests pass (was 244), with the pack's new population
      reflected in the worldpack and CLI suites.
- [x] Review: `agent::drive` calls exactly `host.join` and `host.submit`, which are the two calls
      `server/src/session.rs` makes; it holds no other handle. The seat check before spawning uses the
      pack's own roster, so a mistyped `--agent` is an error before a socket is bound. A test pins that
      a client cannot act as Alice even while an agent drives her
      (`one_window_cannot_act_as_the_other_nor_as_alice`).

### C5 — the Godot client protocol module and its scene

- [x] Implementation: `clients/protocol/` — the module (`mineworld/world_client.gd`,
      `observation.gd`, `space.gd`), the demonstration scene, `run.sh`, `README.md` and
      `ADOPTION.md`. The transcript is the demonstration scene under `--autopilot` rather than a
      second script (§7.7).
- [x] Validation: Godot 4.7.2 (`4.7.2.stable.official.ed1daf0bf`) headless and windowed against
      `mineworld server worlds/social-cafe --agent alice`, four runs, all in
      `clients/protocol/evidence/`. The client joined a seat, was told the world instance, perceived
      five entities, was refused `too_far_away`, walked, waited for the server's verdict to change,
      spoke, and read Alice's reply out of its own disclosed history. The windowed run saved
      `demo-scene.png`, which was read back and inspected.
- [x] Review: no comparison of two positions anywhere in `clients/protocol` — every verdict comes
      from `observation.may()` / `unavailable_reason()`, which read the server's answer; no `int()`
      or arithmetic is applied to any identity (they are `String` everywhere, and the demo uses them
      as dictionary keys and labels); every number the module sends goes through
      `MineWorldSpace.millimetres` / `millidegrees`, and the two fixtures show integer millimetres on
      the wire.

### C6 — `AC-15` and `AC-13`

- [x] Implementation: `tools/cli/tests/ac15_one_alice.rs` and `tools/cli/tests/ac13_semantic_parity.rs`.
- [x] Validation: six AC-15 tests against the real binary, the real pack and real sockets; §9.3
      records the identities they read off the frames.
- [x] Review: the assertions name a world instance, an `EntityId` and an `EventId` sequence, and not
      one of them compares two renderings. The counterfactual is a committed test rather than a note
      (§9.4): two servers, two Alices with the *same* `EntityId` and the same authored position, told
      apart by the instance and by the carry-forward that cannot happen.

### C7 — the ledger

- [x] Implementation: this file's §§7-10, `handoff.md` replaced, `docs/MVP_STATUS.md` updated where
      this PR changed the answer.
- [x] Validation: the four gates at the final HEAD (§9.1).
- [x] Review: §9 separates what ran from what did not, and §10 states nine limitations including the
      two that bound the central claim (the agent's transport, and the absent persistence line).

---

## 7. Discoveries and deviations

Recorded as they were found.

### 7.1 `Observation<Value>` reaches further than expected — bounded, accepted

Previous assumption: only `observe`'s return type and `wire()` would change.
Audit evidence: `systems/presence/tests/presence.rs:81` builds `Observation` by name, and
`Observation::new`'s inference has nothing to fix `P` to once no `Vec<u8>` payload is ever
constructed.
Corrected understanding: the test helper and two `Observation` type annotations in the presence and
conversation suites also name the payload type.
Implementation consequence: three annotations changed, no behaviour.
Validation consequence: none — the suites assert the same facts.

### 7.2 The rename is not cosmetic

`InteractionProvider` with a `discloses` method would be a name that lies, and a reviewer reading
`catalog.rs` would see a pack registered as an *interaction* provider and disclosing state. The
rename to `PerceptionProvider` is mechanical (three crates, one method name each) and
`ENGINEERING_STANDARDS.md` §12 is explicit that a wrong early interface is changed rather than
wrapped while no public stable contract exists.

### 7.3 The agent occupies a seat, which is why Alice is now one

Bounded decision, recorded because it changes a World Pack a previous PR authored.

The kickoff requires the agent to act "through the same `ActionRequest` path a human client uses".
`WorldHost::join` resolves a seat from the world's roster and there is no second way to acquire an
observer — deliberately, and adding one would be exactly the privileged path `INV-1` is about. So
`worlds/social-cafe/world.yaml` offers three seats: `visitor`, `wanderer` and `alice`.

That makes a seat what it always was — *a Person a client or a controller may occupy* — and it makes
mixed control (`MODULE_SPEC.md` §5: "Alice → human, Bob → local Qwen") a property of the world's
configuration rather than of the person. It also means a human client may occupy Alice, which is
`AC-5`'s direction rather than a defect, and which §10 records as a limitation because two
controllers on one Person are not arbitrated in MVP-0.

The pack format is **not** changed: no `controllers:` field. Which Person an agent drives is a
launch argument (`--agent alice`) until the pack format gains configuration in S7, because changing
the format is PR 05c's frozen contract and not a bounded discovery.

### 7.4 The agent runs in the server process, over `WorldHost` rather than a socket

Decision, with the alternative stated. The driver calls `host.join(seat)` and `host.submit(observer,
request)` — the same two calls `server/src/session.rs` makes for a WebSocket client, so the seat
resolution, the actor check, the server-allocated `ActionId`, the server-allocated instant and the
per-observer perception are all identical. What it skips is JSON framing and a TCP hop.

The alternative — a second process speaking the wire protocol — would need `tokio-tungstenite` as a
runtime dependency, which `Cargo.toml` currently records as a dev-dependency only, and would put a
fourth terminal in front of the operator for a demo whose point is that there is one server. The
honest statement of what was proved is in §10: the agent is on the client *authority* path, not
across the client *transport*. Nothing in the architecture prevents the socket version; it is a
driver swap.

### 7.6 DISCOVERY — a disabled pack was still disclosing its state

Found by a test written to fail if it did.

```text
Previous assumption:
  perception's AC-2 filter covers everything a pack contributes, because an offer from a
  disabled pack is already dropped by the kernel's route map.

Audit evidence:
  `observe`'s route-map check is `world.systems().provider(action_type)`, which is about ACTIONS.
  A component has no route, so `a_disabled_pack_discloses_nothing_and_still_holds_its_state`
  failed: the history was still in the observation of a world whose conversation pack was disabled.

Corrected understanding:
  the analogue of a route for state is the kernel's component DECLARATION table, which says who
  owns a component type, plus the registry's `is_enabled`. `owned_by_an_enabled_system` asks those
  two, so the decision stays the kernel's and this crate still never learns what the component is.

Implementation consequence:
  `disclosed` filters on that, and `observe` takes the `&World` it already had for the same reason
  the affordance path does.

Validation consequence:
  the failing test passes, and it is the reason the filter exists rather than a decoration on it.
  Without it, disabling a pack would remove what a player may *do* and leave what a player may
  *know* — the half of AC-2 nobody would have noticed.
```

### 7.5 The `AC-13` fixtures come from the real Godot client, not from the test

A Rust test that writes both request frames itself proves the comparison and nothing about the
clients. So `clients/protocol/`'s headless script submits a `talk` in each of its two flavours — one
that reports a position and one that reports none, which is the whole of what a 3D and a 2D client
differ by — and writes the exact JSON it sent to `clients/protocol/evidence/`. The `AC-13` test
reads those two files and compares them with the server's own definition. Frozen real evidence
rather than a self-referential fixture (`test-ci-gate-rules.md` §25).

### 7.7 The transcript is the demonstration scene, not a second script

Bounded deviation from §6 C5, which planned a separate `headless.gd`.

A second script would have been a second client, and the thing most worth proving is that *the one a
person watches* is the one the evidence came from. So `demo/demo.gd` takes `--autopilot` and runs the
same code path with nobody at the keyboard — the spike's discipline, and the reason the transcripts
and the screenshot describe the same scene.

### 7.8 DISCOVERY — a Godot project must be imported before a headless run

`godot --headless --path clients/protocol` failed with *Identifier "MineWorldClient" not declared*.
Godot registers `class_name` in `.godot/global_script_class_cache.cfg`, which it builds when it
imports a project; a first headless run has no cache and no editor to build one.

`godot --headless --path <project> --import` builds it, takes about a second, and is in `run.sh`,
the README and `ADOPTION.md` §1 — because every adopting project will hit this. The cache is
gitignored: a clean checkout builds its own, which is what `ACCEPTANCE.md` §4.1 requires.

### 7.9 Godot 4.7 treats an inferred-from-Variant declaration as an error

`var x := something_returning_Variant()` is a *warning treated as an error* in this Godot. Fixed by
typing those declarations explicitly. Worth recording because it will hit the visual clients too, and
because the resulting style — an explicit type wherever a dictionary is read — is the right one for a
client reading JSON.

---

## 8. The client protocol module

### 8.1 Shape, and why it is a folder inside a project

```text
clients/protocol/
  project.godot           a Godot 4.7 project, so the module has a runnable home
  mineworld/              THE MODULE — the folder an adopting project copies or symlinks
    world_client.gd         MineWorldClient: the connection, the two frames, the four signals
    observation.gd          MineWorldObservation: reading a frame without deciding anything
    space.gd                MineWorldSpace: the one conversion between world and engine axes
  demo/
    demo.tscn / demo.gd     the demonstration scene: who is here, what may be attempted, what
                            was said, and two buttons that submit
    headless.gd             the transcript and the AC-13 request fixtures
  README.md                 human orientation
  ADOPTION.md               the specification an adopting client is written against
  evidence/                 what the real engine produced
```

The two visual clients are two separate Godot projects, and Godot cannot reference `res://` above a
project root, so "drop-in" means *one self-contained folder with no project-setting dependencies*.
`ADOPTION.md` states the two supported ways to take it and what each costs.

### 8.2 The division of knowledge

```text
the module knows      the frames, the handshake, the seat, the sequence number, the correlation
                      token, that an identity is a string, that every other number is an integer,
                      that `action_type` appears twice and must agree, and that the actor is this
                      connection's observer
the client knows      which action it is submitting and what its payload means, how the world
                      looks, and what a key press or a click means
NEITHER knows         whether an action is allowed. The module surfaces the server's affordances
                      and results; it computes no distance and no availability.
```

`MineWorldClient.submit(action_type, target, payload, actor_location)` is the whole submission API:
a client cannot forget the envelope rules because it does not write the envelope, and it cannot
invent an `ActionId` because there is no parameter for one.

### 8.3 The conversion nobody should write twice

`CORE_CONCEPTS.md` §6.1 fixes the frame; `MineWorldSpace` is that paragraph as code, in both
directions and for both dimensions, so the sign flips the spike's two clients each invented
separately (`FINDINGS.md` F5) exist once.

---

## 9. Validation record

Every command below was actually run on this host, at the commit named. A result is classified from
what it printed, never from an exit code alone.

### 9.1 The four gates, at the final executable HEAD

```text
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check                                              clean
cargo check --workspace --all-targets                                clean
cargo clippy --workspace --all-targets --all-features -- -D warnings clean
cargo test --workspace                                               258 passed, 0 failed
```

258 tests, up from 237 at `main @ cc40ad5`. The 21 new ones: 4 in the server (`parity`, the world
instance, the shared instance across two connections), 3 in the conversation pack (the disclosure,
its absence for a stranger, its absence when the pack is disabled), 6 in the rule controller, 6 in
`ac15_one_alice.rs`, 2 in `ac13_semantic_parity.rs`. No test was weakened; four existing tests in
`worldpack` and `tools/cli` were updated for the pack's new population and seats, which §7.3 records.

### 9.2 Gate 2 — the real thing, running

Godot `4.7.2.stable.official.ed1daf0bf`, Metal, Apple M5. `mineworld` built by cargo from this
branch, hosting `worlds/social-cafe` from disk.

| Run | Command | Result |
| --- | --- | --- |
| R1 | `mineworld server worlds/social-cafe --agent alice` | `PASS` — loads, listens, prints its world instance, and the controller occupies the `alice` seat: *agent: driving 'alice' as entity 2* |
| R2 | `godot --headless --path clients/protocol -- --autopilot --flavour 2d --seat visitor` | `PASS` — joined, was told instance and observer, perceived 5 entities, was refused `too_far_away`, walked, saw the server's verdict change, spoke, and read Alice's reply out of its own disclosed history |
| R3 | the same with `--flavour 3d --seat wanderer` | `PASS` — and Alice's reply to it quotes *what the 2D client said*, naming person 4 |
| R4 | `godot --path clients/protocol` (windowed) with `--screenshot` | `PASS` — `clients/protocol/evidence/demo-scene.png`, read back and inspected: the world instance, the people at their authored positions, the server's four verdicts in three colours, and the disclosed history |
| R5 | two Godot clients **at once** against one server | `PASS` — both told instance `0000000018d95a32bba34d4899250000`; the second was told `3 client(s)`; Alice answered both and told the first what the second had said |
| R6 | `cargo test -p mineworld-cli --test ac15_one_alice` | `PASS` — six tests, §9.3 |
| R7 | `cargo test -p mineworld-cli --test ac13_semantic_parity` | `PASS` — two tests, §9.5 |

Transcripts, the two request fixtures and the screenshot are in `clients/protocol/evidence/`, with a
README saying which run produced which file and how to regenerate them (`clients/protocol/run.sh`).

### 9.3 `AC-15`, and the identities it actually recorded

`there_is_only_one_alice`, printed by the test itself:

```text
AC-15 evidence, read off the frames two clients received:
  same world instance          0000000018d958c063e6efd85a7a0000
  same Alice EntityId          2
  one event sequence           [6, 7, 10, 11]
  one action-id allocator      2, 5
  2D window (4) said   "hello Alice, this is the 2D window"
  3D window (5) said   "hello Alice, this is the 3D window"
  Alice told the 3D window     "I remember you. You said \"hello Alice, this is the 3D window\".
                                Earlier, person 4 said \"hello Alice, this is the 2D window\" to me."
```

Read as evidence, in the terms `MVP.md` §9.1 sets:

```text
same world instance       both welcomes and GET /status carry one instance id
same Alice EntityId       entity 2 is the barista BOTH windows perceive, and is the speaker of the
                          reply each of them received — found by her tag, never by a literal
one event sequence        [6, 7] are the 2D talk's facts and [10, 11] the 3D talk's; 8 and 9 are
                          Alice's own reply in between. One monotonic sequence across three
                          participants. Two worlds would each have started at 1.
the carry-forward         she repeats to the second window what the first one said. Two
                          synchronised copies cannot produce that sentence: neither was told the
                          other's conversation.
```

`MVP.md` §9.1's fourth line, *same persisted state revision*, is **not** recorded: this server holds
its world in memory and persistence is S5's (§10).

### 9.4 The counterfactual, which makes §9.3 evidence rather than decoration

`two_servers_are_two_worlds_and_the_evidence_can_tell`, a committed test rather than a note. Two
servers, one client on each:

```text
alice_here == alice_there      the SAME EntityId, because two loads of one pack resolve the same
                               keys to the same ids (AC-12 working)
same authored position         she is standing in the same place in both
different instance             which is what tells them apart
no carry-forward               the second Alice says "You are the first person to speak to me
                               here", which is true of her world
```

An "appearance" test passes there and is wrong. That is the false success `MVP.md` §9.1 exists to
exclude, built on purpose so that the real evidence is demonstrably discriminative.

### 9.5 `AC-13`, from what the real client sent

The two `talk` requests in `clients/protocol/evidence/`, submitted by the Godot module in its two
flavours as the same seat:

```text
              2D                                  3D
actor         "4"                                 "4"
action_type   "talk"                              "talk"
target        "2"                                 "2"
payload       {"utterance": "hello Alice, …"}     {"utterance": "hello Alice, …"}
actor_location null                               {place 1, local (1200, 1400, 0), yaw 0}
```

`semantic_core(2d) == semantic_core(3d)`, and `differing_fields` is exactly
`[RequestField::ActorLocation]` — the one field `MVP.md` §9's correction names, and the only one left
after PR 04 removed `action_id` and `issued_at` from what a client submits. Replayed against a server
each, both were answered identically: `Accepted` with two facts.

### 9.6 The objective gates the kickoff names

```text
clean checkout                 ✅ `.godot/` is gitignored and was deleted and rebuilt during this
                                  work; `git status` is clean at HEAD
no hand-preserved caches       ✅ `run.sh` builds the Godot import cache itself; nothing in
                                  `evidence/` is required to run anything
reproducible relaunch          ✅ the server was started and stopped at least eight times over this
                                  work, and `run.sh` is the scripted form
no fatal errors                ✅ no panic, no `faults` above 0, no `observations_dropped` above 0,
                                  and no error line in any transcript
two clients at once            ✅ R5 (two Godot clients) and `there_is_only_one_alice` (two sockets
                                  plus the agent). `/status` reported 3 clients.
each sees only its own         ✅ `a_window_is_told_what_it_heard_and_never_what_somebody_else_heard`
                                  — a window perceives Alice and is not shown her memory
killing one leaves the rest    ✅ `killing_one_window_leaves_the_world_and_the_other_window_running`
```

---

## 10. Limitations and follow-ups

Stated plainly, because a precise account of what was not proved is worth more than a claim.

1. **The agent is on the client authority path, not across the client transport.** It calls
   `host.join` and `host.submit` — the same two calls a WebSocket session makes, with the same seat
   resolution, actor check and server-allocated identity — but in the server's process, without JSON
   framing or a TCP hop. §7.4 states the alternative and why it was not taken. What is *not* proved is
   that a controller in another process behaves identically; nothing in the architecture prevents it,
   and it is a driver swap.
2. **`AC-15`'s fourth evidence line has no referent.** `MVP.md` §9.1 also asks for *same persisted
   state revision*, and this server holds its world in memory. S5 gives that line something to name;
   until then it is recorded as missing rather than approximated.
3. **An `Observation` still carries no events.** Not needed here (§4.4), because what an NPC said to
   you arrives in your own disclosed history, and the `AC-15` event evidence comes from the `EventId`s
   on each `result`. A client that wants to *overhear* two other people — `Spoke` is emitted with
   `Visibility::Place` — cannot yet. S10 owns it, and the seam it would use is the one this PR added.
4. **The two reference clients have not adopted the module.** By instruction: they are under active
   visual development on their own branches, and this PR must not merge or edit them. The module is
   drop-in and `ADOPTION.md` is written for exactly that handover; nothing has yet proved it inside
   *their* scene graphs.
5. **The demonstration scene draws dots.** It is a protocol demonstration and not a style candidate,
   and no fidelity claim is made from it (`ACCEPTANCE.md` §3.1).
6. **Two controllers may occupy one Person.** Alice is a seat, so a human client could join as her
   while the agent drives her, and MVP-0 arbitrates nothing. That is `AC-5`'s direction rather than a
   defect, and it wants a decision — exclusive seats, or explicit hand-over — before a world is
   exposed to strangers.
7. **`WorldInstanceId` is not a security boundary.** It distinguishes one running world from another,
   which is what the evidence needs. It is not random enough to be unguessable and nothing
   authenticates with it.
8. **The `AC-13` fixtures are frozen evidence and can go stale.** If the demonstration scene's wording
   or walk changes, `run.sh evidence` regenerates them and the change belongs in the same commit. A
   test reading a stale fixture would still be testing two real frames, but not the current client's.
9. **`docs/MVP_STATUS.md` was updated only where this PR changed the answer.** Parts of it describe
   work owned by branches this session must not touch; the rest of its staleness is recorded here
   rather than silently fixed.
