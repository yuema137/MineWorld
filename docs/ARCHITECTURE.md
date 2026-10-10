# MineWorld — Architecture

**Status:** frozen architecture for v0.1 planning
**Audience:** coding agents. Vocabulary is defined in [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md);
module packaging in [`MODULE_SPEC.md`](MODULE_SPEC.md); transport and hosting in
[`NETWORKING.md`](NETWORKING.md). Engineering rules that govern implementation are in
[`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md).

---

# 1. The five layers

```text
             World Definition
                   │
                   ▼
          ┌──────────────────┐
          │ MineWorld Kernel │
          └────────┬─────────┘
                   │
            Interaction Systems
                   │
        ┌──────────┴──────────┐
        ▼                     ▼
   Controllers            Presentation
 Human / LM / RL       2D / 3D / Headless
        │
        ▼
   ActionIntent
        │
        └──────────→ Kernel
```

```text
Simulation creates reality.
Controllers propose actions.
Presentation observes reality.
```

Expanded, with the authority boundary made explicit:

```text
┌─────────────────────────────────────────────┐
│               WORLD PACK                    │
│ people / places / items / organizations     │
│ enabled systems / initial state / config    │
└────────────────────┬────────────────────────┘
                     ▼
┌─────────────────────────────────────────────┐
│          AUTHORITATIVE WORLD SERVER         │
│ Entity Registry                             │
│ Component State                             │
│ Scheduler                                   │
│ Interaction Systems                         │
│ Processes                                   │
│ Event Log                                   │
│ Persistence                                 │
│ Networking                                  │
└───────────┬───────────────────┬─────────────┘
            │                   │
     observations         state updates
            ▼                   ▼
┌────────────────────┐   ┌─────────────────────┐
│ COGNITION LAYER    │   │ PRESENTATION LAYER  │
│ Human              │   │ Godot 2D            │
│ Rule-based         │   │ Godot 3D            │
│ LM                 │   │ Unreal              │
│ RL                 │   │ Web                 │
│ Custom agent       │   │ Custom              │
└─────────┬──────────┘   └─────────────────────┘
          │
      ActionIntent
          ▼
   World validation
```

The world server is always authoritative. Clients and controllers may **request** actions.
They may never modify world state (INV-5, INV-6, INV-9).

---

# 2. Kernel scope

The kernel's responsibilities are exactly these:

```text
Identity
Component storage
World clock
Spatial semantics
Action dispatch
System registry
Process scheduling
Event sourcing
Relationship graph
Persistence
Networking
Permissions
Plugin lifecycle
```

And, per INV-12, the kernel does **not** know:

```text
how romance works
what employment is
how eating works
what coffee costs
that people need to sleep
```

All of that is a System Pack. A change that puts domain knowledge into the kernel is rejected
on architectural grounds, not stylistic ones.

---

# 3. Action pipeline

The central contract of the framework:

```text
ActionIntent  (from a controller: human, rule, LM, RL, script)
     ↓
Dispatch      kernel routes the intent to the system that provides the action
     ↓
Validate      the owning system decides admissibility against current state
     ↓
Resolve       the owning system decides the outcome
     ↓
Event(s)      immutable facts appended to the event log
     ↓
Reducer(s)    each owning system applies events to the components it owns
     ↓
New World State
```

Properties that must hold:

1. Dispatch of an action no enabled system provides returns `ActionUnavailable` (INV-10).
2. Validation reads current authoritative state, never the state the controller believed.
3. Only the owning system resolves its own actions; only owning systems reduce into their own
   components (INV-7).
4. Every resulting state change is attributable to an event, and every event carries
   `CausedBy` (INV-15).
5. Rejection is a normal outcome and is reported to the requester as a first-class result.

---

# 4. Single-writer rule

```text
EmploymentSystem
    owns EmploymentComponent
    must not write BankAccount.balance

EmploymentSystem  ──WageDue──►  EconomySystem
                                    owns AccountBalance
                                    emits MoneyTransferred
```

This is what makes systems independently installable, removable, and replaceable. A system
that writes another system's component has coupled the two permanently, and the composability
claim in [`VISION.md`](VISION.md) is lost.

---

# 5. Time, scheduling, and processes

The world clock is kernel-owned. Simulation advances in discrete-event / semantic ticks, not
in render frames.

Processes ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §10) are scheduled entities with duration.
The scheduler:

- advances `WorldTime`;
- wakes processes at their scheduled boundaries;
- delivers subscribed events to systems;
- routes interruption *requests* to the owning system, which decides the outcome.

Distinct rates, which must never be conflated:

```text
Rendering:            60+ FPS, renderer's concern
Local physics:        renderer-dependent, never authoritative
World simulation:     discrete-event / semantic ticks
Network replication:  state-delta based
Agent cognition:      event triggered, asynchronous
```

---

# 6. Observation and perception

Controllers receive `Observation`s produced by the world (INV-13). The kernel provides the
mechanism; *what* is perceivable is decided by the perception systems a world enables
(`VisionSystem`, `HearingSystem`, `RumorSystem`, `PhoneSystem`, `InternetSystem`,
`NewsSystem`, …).

Event `Visibility` is the input to that filtering. An LM controller that can read global state
is an architecture defect, not a tuning problem.

---

# 7. Event sourcing and snapshots

World history is append-only at the semantic level.

```text
#10921 AliceEnteredCafe
#10922 BobAskedForJob
#10923 AliceRejectedRequest
#10924 RelationshipChanged
```

Snapshots are generated periodically, and a world state is reconstructed as:

```text
snapshot at revision R  +  the journal after R, re-executed  →  checked against the facts after R
```

Stated precisely, because two different things are authoritative for two different questions
([`DECISIONS.md`](DECISIONS.md) `ARC-25`):

```text
history   the fact log: every Event, append-only, never rewritten and never derived (INV-11)
state     the kernel's one pipeline applied to the journal — the recorded inputs that moved the
          world: genesis, every ActionIntent with its instant, every advance of the clock that
          fired something
```

A snapshot is a checkpoint of that function, never an authority on its own. Reconstruction loads the
newest snapshot and re-executes the journal after it through the same pipeline a live world uses; each
re-executed input must regenerate its logged facts byte for byte, or the load is refused. Facts are
**not** re-applied through the reducers: a System reduces and emits in the same call and may write
while resolving or waking a process, so re-applying a fact would re-emit its consequences. The log is
instead the check every reconstruction must pass, and a full re-execution from genesis must reproduce
every stored snapshot.

Each journaled input is one **revision** of the world: a monotonic number, committed before any client
is told it, and the persisted state revision `MVP.md` §9.1 asks `AC-15` evidence to name.

What this buys, and why it is non-negotiable:

```text
replay
debugging
agent evaluation
biography generation
save migration
causal explanation
deterministic testing
```

Recorded LM outputs are replayable, so regression scenarios do not pay for inference twice —
and core tests never depend on a live provider
([`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §24).

---

# 8. Persistence

```text
PersistenceBackend
├── SQLiteBackend      local default
└── PostgresBackend    cloud, later
```

Local save layout:

```text
save/
├── world.sqlite       manifest · journal · facts · snapshots, one transaction per revision
└── cognition_cache/   later (S10 / MVP-1)
```

The manifest — save format version, world instance, composition — is a table inside `world.sqlite`
rather than a `manifest.json` beside it, so that it is committed in the same transaction as the
revisions it describes ([`DECISIONS.md`](DECISIONS.md) `DEP-2`). A save whose format, composition, system
versions or component schemas differ from the running code is refused by name, never decoded on a guess.

World semantics must not depend on the selected database (INV-14). A behavior that appears
only under one backend is a defect in that backend, not a property of the world.

---

# 9. Controller runtime

The authoritative server must never block on a controller — least of all on a model call.

```text
World event
    ↓
Controller receives Observation
    ↓
async cognition worker
    ↓
model thinks for 2 seconds
    ↓
ActionIntent returned
    ↓
WORLD REVALIDATES CURRENT STATE
    ↓
action executes or fails
```

The revalidation step is mandatory: the world may have changed while the controller was
deciding. An intent validated against a stale observation must fail, not succeed on the
strength of having been requested.

## 9.1 Cognition budget

Cost is a first-class concept in the architecture, not an operational afterthought.

```yaml
cognition:
  routine:
    policy: deterministic
  ordinary_decision:
    model: local-small
  social:
    model: local-medium
  major_decision:
    model: frontier

limits:
  max_calls_per_sim_hour: 20
  max_tokens_per_day: 30000
```

The server operator decides who pays for inference. The framework itself must require no paid
API.

## 9.2 Model backends

First-class interfaces:

```text
OpenAI-compatible endpoint
Ollama
llama.cpp
vLLM
custom HTTP
```

Credentials belong to the server operator and never appear in a World Pack. A world declares
capabilities, not secrets:

```yaml
requires:
  structured_generation: true
  context_window: ">=16k"
```

---

# 10. Presentation boundary

Rendering choices never leak into simulation contracts (INV-5, INV-14).

Two consequences are stated in full in [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§4–9 and are
binding here. First, **semantic space is not render geometry**: the simulation knows
`Cafe.Counter`, while `(x, y)`, `(x, y, z)`, a navmesh point and an animation anchor belong to a
client; the kernel touches engine geometry only through the generic spatial contract. Second,
**a renderer reports intent and never evaluates a rule**: a client detects that the player pressed
interact while targeting Alice and sends an intent; the owning system answers with the resolved
action or with `Unavailable`, `Busy`, `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction`.
Both the 2D and the 3D reference client are first-class, and the same semantic `ActionIntent` must
result whichever one initiated it.

World state:

```text
Alice
location          = Cafe
activity          = serving_customer
position_semantic = counter
```

2D renderer:

```text
AliceSprite
animation = serve
```

3D renderer:

```text
AliceCharacter
navigation_target = CounterAnchor
animation         = ServeCoffee
```

Photorealistic renderer:

```text
high-fidelity character model
motion-matched animation
high-fidelity cafe asset
```

Text client:

```text
08:32 Alice starts making coffee.
```

The simulation does not care, and must not be able to tell.

---

# 11. Deployment

One server binary, four targets:

```text
Local PC
Cloud VM
Docker
Dedicated community server
```

There is no separate single-player logic. Single-player is a client talking to a local server
over localhost. Details in [`NETWORKING.md`](NETWORKING.md).

---

# 12. Plugin security

```text
v0     trusted Rust systems, trusted Python extensions
later  WASM component plugins
```

Trusted in-process Rust may come from outside this repository: a System Pack compiled into the build
from a commit pinned in the installed set, reviewed like any dependency (`DECISIONS.md` `ARC-66`).

A plugin declares the capabilities it needs:

```text
world.read.person
world.read.place
world.emit.event
```

Network and filesystem access are never implicit.

---

# 13. Technology stack

| Layer | Choice |
| --- | --- |
| Authoritative kernel | Rust |
| Async runtime | Tokio |
| Public server API | HTTP + WebSocket |
| Internal service RPC | gRPC / Protobuf where useful |
| Contracts | Rust types in `mineworld-contracts`; other languages mirror them as typed models held to the server's golden frames over the JSON client protocol (decision D-4; `ARC-41`; §13.1) |
| Local persistence | SQLite |
| Cloud persistence | Postgres, later |
| Cognition runtime | Python |
| LM interfaces | provider-neutral |
| World authoring | YAML + schema validation |
| Reference client | Godot (MIT); 3D default renderer **Forward+** — `DEP-9` |
| High-end client | Unreal adapter, later |
| Local deployment | native binary / Docker |
| Cloud deployment | Docker first |
| Future plugin sandbox | WASM Component Model / WIT |

Language boundaries are governed by [`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§3–4:
language choice is local, world semantics are global, and cross-language communication goes
through explicit contracts only.

## 13.1 Contract representation: Rust types first (decision D-4)

This row previously read "Protobuf + generated language bindings", which described a
Protobuf-first pipeline. The decision, recorded when the contract layer was first written, is
the other way round:

```text
source of truth   Rust types in contracts/  (crate mineworld-contracts)
Protobuf          introduced at the first real cross-language boundary and
                  mirrored from those types, never the reverse
first boundary    Python cognition, then the Godot client
```

The reasons, so that a later session does not reverse it by accident:

1. A Protobuf-first pipeline costs a second schema language, code generation and a build step
   from the first commit, while the first non-Rust consumer arrives much later.
2. The guarantees the kernel relies on are expressible in Rust's type system and not in
   Protobuf: distinct identifier types that cannot be substituted for one another, component
   ownership carried as part of a component's type, one payload-erasure boundary per contract
   family and nowhere else. Generating the contracts from a `.proto` would flatten all three into
   integers and byte strings and move every check to runtime.
3. Choosing a wire encoding early would also choose it for component payloads, which the
   contract layer deliberately leaves to the persistence layer.

Consequences, which are binding:

- The Rust types are authoritative. A `.proto` file added later is a mirror, and keeping it in
  step with the Rust types is part of the change that adds it.
- Until that boundary exists, there is no cross-language contract to maintain, and adding one
  "for later" is the premature work this decision exists to avoid.

**What the first boundaries turned out to be (2026-10-08, S10 PR P3; step-17 G-1).** The
direction above stands; the timing in the block above it does not, and `ARC-41` records why:

```text
first boundary    the JSON client protocol (server/PROTOCOL.md): the Godot client first (S5V),
                  then Python (mineworld-sdk, sdk/python, S10)
how it is kept    each language mirrors the Rust types as typed models (GDScript; Pydantic in
                  Python), and each mirror is tested against the golden frames the Rust tests
                  keep in server/tests/frames/, so the Rust types stay the one source
Protobuf / gRPC   declined until an encoding need is measured (ARC-41, ARC-56); a binary
                  encoding, if one is ever needed, starts from serde (MessagePack), not .proto
```

- A pull request that changes a golden frame updates every mirror in the same change (step-17
  R-S11-9), so a mirror cannot drift silently: its own golden test fails first.
- The Python mirror is hand-written while the vocabulary is small enough to review; generation
  (JSON Schema from the Rust types) is the upgrade when it is not.

---

# 14. Repository structure

```text
mineworld/
├── docs/          specifications
├── contracts/     entities, components, actions, events, observations, networking
├── kernel/        entity, components, scheduler, process, actions, world snapshots
├── persistence/   the journal, the fact log and the save: PersistenceBackend, SqliteBackend
│                  (the kernel layer, kept a separate crate so no SQL type enters the kernel)
├── systems/       time, places, movement, conversation, relationships, inventory,
│                  group_activity, economy, employment, …
├── cognition/     runtime, controllers, memory, biography, models, budgets
├── server/        the process that hosts the kernel
├── clients/       godot, admin-web
├── sdk/           python, rust
├── packages/      what a package is: package identity, pack.yaml, versions and licences
│                  (a leaf crate, DECISIONS.md ARC-53)
├── worlds/        sample World Packs
├── tools/         world-validator, replay, inspector, benchmark
└── tests/         conformance and acceptance tests
```

Dependency direction is one-way and enforced by review:

```text
World Pack  →  Systems  →  Kernel Contracts
```

A circular dependency between those layers is an architectural warning to be resolved, never
worked around.
