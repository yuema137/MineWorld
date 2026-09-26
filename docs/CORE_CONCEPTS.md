# MineWorld — Core Concepts

**Status:** frozen ontology for v0.1 planning
**Audience:** coding agents. This document defines MineWorld's vocabulary and its hard
invariants. Every other document, contract, and implementation must use these terms with
exactly these meanings.

Terminology discipline: never introduce a synonym for a term defined here, and never reuse a
term defined here for a different concept. Extending the ontology is a change to this
document, reviewed as such.

---

# 1. The core primitives

The core stays at roughly ten concepts. Growth happens in modules, not here.

```text
Entity
├── Person
├── Place
├── Item
└── Organization

Relation
Process
ActionIntent
Event
System
Controller
World
```

Supporting concepts that are part of the kernel vocabulary but not themselves entities:

```text
Component        typed state attached to an entity, owned by exactly one system
Observation      the filtered view of the world delivered to a controller
WorldTime        the simulation clock
```

Deliberately **not** core: `Vehicle`, `Building`, `Job`, `Money`, `Food`, `Pet`, `School`,
`Hospital`, `Restaurant`, and every other domain noun. They are compositions:

```text
Building = Place  + StructureComponent
Vehicle  = Entity + TransportComponent + InventoryComponent
Job      = Relation(Person, Organization) + EmploymentComponent
Money    = EconomySystem's owned component plus its own action set
```

---

# 2. Hard invariants

These are contracts, not guidelines. Code that violates one is defective even if it works.

| ID | Invariant |
| --- | --- |
| **INV-1** | `Person ≠ Controller`. A Person exists independently of whatever decides its actions, and control may change at runtime without altering the Person. |
| **INV-2** | `ActionIntent ≠ Event`. An intent is a request that may be rejected. An event is an immutable fact that already happened. |
| **INV-3** | `Process ≠ Event`. A Process is something happening over time and is mutable while it runs. An Event is instantaneous and immutable. |
| **INV-4** | `Memory ≠ Biography ≠ World Truth`. Subjective belief, derived life history, and the authoritative event log are three distinct things. |
| **INV-5** | The renderer does not own world state. Presentation observes; it never mutates. |
| **INV-6** | The controller does not own world state. Controllers submit `ActionIntent`s; they never mutate. |
| **INV-7** | Only systems mutate state, and only the components they own. Cross-system change happens through events. |
| **INV-8** | No entity may directly modify another entity. All change is mediated by a system. |
| **INV-9** | The server is authoritative. Every client and every controller is a requester. |
| **INV-10** | An action that no enabled system provides does not exist. The kernel answers `ActionUnavailable`, regardless of who asked or how convincingly. |
| **INV-11** | The Event Log is the single source of truth for history. Every other historical view is derived and reconstructible from it. |
| **INV-12** | The kernel contains no domain semantics. It does not know what a job, money, romance, hunger, or sleep is. |
| **INV-13** | Controllers receive `Observation`s, never global world state. Omniscience must be impossible by construction, not by convention. |
| **INV-14** | Simulation semantics never depend on the renderer, the transport, the persistence backend, the deployment target, or the model provider. |
| **INV-15** | Every significant state mutation is traceable to an `ActionIntent`, a `Process`, or a system-emitted `Event`. Untraceable mutation is a defect. |

INV-10 deserves an example, because it is the invariant that makes controllers safe:

```text
CombatSystem = disabled

LM controller output:  "I shoot Bob."
Kernel response:       ActionUnavailable

There is no shoot() in this universe.
```

---

# 3. Entity

The universal identity primitive. Everything persistent is an Entity.

```text
EntityID          stable, unique, never reused
EntityType        Person | Place | Item | Organization
Tags              open set of semantic labels
LifecycleState    e.g. active, dormant, destroyed / archived
Metadata          authoring provenance; never gameplay semantics
```

An Entity carries almost no semantics on its own. Semantics come from Components, and
Components are owned by systems. Removing a system removes the meaning it contributed without
invalidating the Entity.

## 3.1 Component

A Component is typed state attached to an Entity by exactly one system.

```text
ComponentType     declared by its owning system
OwnerSystem       the single writer (INV-7)
Schema            explicit and versioned
Data              typed fields, never an untyped blob
```

Reading a component may be permitted broadly. Writing it is the owning system's exclusive
right.

---

# 4. Person

A Person is an Entity representing an individual. Per INV-1, a Person is not an agent: a
Person may be controlled by a `HumanController`, `LMController`, `RuleController`,
`ScriptController`, or `RLController`, and that binding may change during runtime without
touching the Person's identity, biography, relationships, inventory, or employment.

A Person's state falls into four classes.

## 4.1 Identity

Usually immutable.

```text
entity_id
origin
birth information
```

## 4.2 Traits

Slow-changing characteristics. They may change, but only through an explicit system.

```text
personality
preferences
skills
values
habits
```

## 4.3 Mutable state

Fast-changing properties. Each field is owned by whichever system provides it; a world that
does not enable that system simply does not have the field.

```text
location
money
hunger
energy
mood
employment
inventory
health
current_activity
```

## 4.4 Relationships

Relationships are structured typed edges, never prose.

```text
Alice --friendship=0.72--> Bob
Alice --trust=0.83------> Bob
Alice --coworker--------> Bob
```

Narrative text may be *generated* from these values for display or for an LM prompt. The
structured value remains authoritative; generated prose is never read back as state.

---

# 5. Biography, Memory, and World Truth

Per INV-4 these are three distinct layers. Conflating them is one of the most damaging
mistakes available in this architecture.

## 5.1 World truth

The Event Log. Append-only, immutable, complete, authoritative (INV-11).

## 5.2 Objective biography

What actually happened to this person, derived from the global event history.

```text
2027-03-02   Alice joined Lakeside Cafe.
2027-04-14   Alice met Bob.
2027-06-01   Alice became cafe manager.
```

Biography is a compressed index over the Event Log, not a source of truth. It can always be
regenerated from events.

## 5.3 Subjective memory

What a character saw, heard, believes, remembers, forgot, or misunderstood. Memory belongs to
cognition, not to the world.

```text
World truth:      Bob lost his job.
Alice's memory:   Bob told Alice he quit voluntarily.
Charlie's memory: Charlie knows nothing about it.
```

This is what makes limited knowledge and conflicting beliefs representable. A cognition
implementation that reads world truth instead of memory has destroyed the distinction and
violates INV-13.

## 5.4 Hierarchical biography compression

Biography must not grow without bound in an LM context window. Layers:

```text
L0 — recent structured events        last hours / days
L1 — episode summaries               "Started working at the cafe and became close to Bob."
L2 — life chapters                   "Early adulthood in Lakewood"
L3 — stable biography                major long-term facts only
```

Every summary retains references to the underlying Event IDs:

```text
Early Career Summary

events:
    1732-2188

summary:
    Alice began working at Lakeside Cafe,
    developed a close friendship with Bob,
    and was promoted after six months.
```

A controller normally reads the summary. Debugging and evaluation tools follow the Event IDs
back to the original facts. Compression therefore reduces **context**, never historical
truth.

---

# 6. Place

A Place represents semantic space. Places form an optional hierarchy.

Detailed world:

```text
World
└── Lakewood
    ├── Downtown
    │   └── Lakeside Cafe
    │       ├── Main Room
    │       ├── Kitchen
    │       └── Bathroom
    └── Lake Park
```

Simple world:

```text
Lakewood
├── Cafe
├── Apartment
├── Park
└── Store
```

Room-level topology exists only when a world enables `InteriorSystem`, `RoomSystem`,
`DoorSystem`, or their equivalents. **Interior granularity is a simulation-system choice, not
a graphics setting.**

Possible mutable Place state, each field owned by the system that provides it:

```text
owner
occupants
open / closed
capacity
inventory
condition
access permissions
temporary events
```

Geometry belongs to the renderer. Semantic location belongs to the simulation (INV-5, INV-14).

---

# 7. Item

Items are objects that participate in simulation. Type and instance are separate concepts.

```text
ItemType
    coffee

ItemInstance
    coffee_18517
    owner:       Alice
    location:    cafe_table_3
    temperature: 61 C
```

Not every world needs individual instances. A system may support unique items, stacked items,
or abstract resources. This is what prevents a world from being forced to instantiate millions
of forks, plates, and coins.

---

# 8. Organization

An Organization is a core primitive. Without it, companies, schools, and families degenerate
into Person-to-Person hacks.

Examples: company, restaurant, school, university, family, club, government, sports team,
gang, research laboratory.

Possible Organization state:

```text
members
roles
owned places
accounts
inventory
rules
schedule
relationships with other organizations
```

Expressed as relations:

```text
Alice         --employee---> Lakeside Cafe
Lakeside Cafe --occupies---> Building_12
Bob           --student----> UCSD
```

---

# 9. Relation

A Relation is a first-class typed edge between entities — not only Person ↔ Person.

```text
Person ↔ Person          friend, parent, partner
Person ↔ Organization    employee, member
Organization ↔ Place     owns, operates_at
Person ↔ Item            owns
Place ↔ Place            connected_to
```

A system may attach its own structured state to a relation, subject to INV-7: the attaching
system owns that state.

---

# 10. Process

A Process is something that happens **over time**. It is not an Event.

Examples: working, sleeping, driving, having dinner, talking, attending class, traveling,
watching a movie, running a business meeting.

```text
ProcessID
type
participants
location
start_time
expected_end
state
progress
interruptibility
owning_system
```

Example:

```text
DinnerProcess

participants:  Alice, Bob
location:      Restaurant_03
started:       19:04
state:         eating_main_course
```

If Bob receives an emergency call, another system may *request* interruption. The owning
system — `DinnerSystem` — decides whether and how the process ends. No other system terminates
it directly (INV-7).

---

# 11. Event

Events are immutable facts. They are the canonical historical record (INV-11).

Examples:

```text
PersonEnteredPlace
ConversationStarted
ItemTransferred
EmploymentStarted
RelationshipChanged
DinnerInterrupted
PersonMovedHome
```

Schema:

```text
EventID
Timestamp
EventType
Subjects
Participants
Location
CausedBy
Payload
Visibility
Provenance
```

`CausedBy` is what makes causality traceable (INV-15). `Visibility` is what lets perception
systems decide who could have learned of the event. `Provenance` records which system emitted
it and, where relevant, which controller decision led to it.

---

# 12. ActionIntent

Controllers never create events. They create `ActionIntent`s (INV-2, INV-6).

```text
actor:   Bob
action:  give_item
target:  Alice
item:    coffee_18517
```

The owning interaction system validates it:

```text
Does Bob possess the coffee?
Is Alice reachable?
Does ItemTransferSystem exist in this world?
Is transferring this item permitted?
```

Only then is it resolved. The pipeline is one of the central contracts of the framework:

```text
ActionIntent
     ↓
Validate
     ↓
Resolve
     ↓
Event(s)
     ↓
Reducer(s)
     ↓
New World State
```

Rejection is a normal, first-class outcome, and `ActionUnavailable` is the answer whenever no
enabled system provides the action (INV-10).

---

# 13. System

A System is analogous to an enabled physics process. It declares:

```text
ID
Version
Dependencies

Owned Components
Provided Actions
Emitted Events
Subscribed Events

Configuration Schema
Migration Schema
```

Example:

```text
InventorySystem

provides:
    InventoryComponent

actions:
    pick_up
    drop
    give

events:
    ItemPickedUp
    ItemDropped
    ItemTransferred
```

## 13.1 Single-writer rule

A system may only directly modify components it owns (INV-7). Cross-domain effects travel as
events:

```text
EmploymentSystem  ──WageDue──►  EconomySystem  ──MoneyTransferred──►  balance changed
```

`EmploymentSystem` must never write `BankAccount.balance`. This is what keeps systems
independently installable and removable.

---

# 14. Controller

A Controller decides what a Person attempts. It is outside the authoritative world.

```text
HumanController
RuleController
BehaviorTreeController
LMController
RLController
ScriptController
```

Within one world simultaneously:

```text
Alice   → human
Bob     → local Qwen
Charlie → cloud model
David   → deterministic NPC
Emma    → RL policy
```

A Controller cannot create new interactions (INV-10). It receives `Observation`s (INV-13) and
emits `ActionIntent`s (INV-6). Rebinding a Person's controller at runtime must preserve that
Person's biography, relationships, inventory, and employment (INV-1).

---

# 15. Observation

Controllers must never read arbitrary global world state. The world produces an Observation.

```text
Alice is in Cafe.

Visible:
    Bob
    Cafe counter
    5 customers

Audible:
    Bob says hello

Known from memory:
    Bob cancelled dinner yesterday
```

Which channels exist depends on which perception systems a world enables. Future modules may
include `VisionSystem`, `HearingSystem`, `RumorSystem`, `PhoneSystem`, `InternetSystem`,
`NewsSystem`. The purpose is structural: an omniscient controller must be impossible, not
merely discouraged.

---

# 16. World

A World is the composed, running whole: an entity population, a set of enabled systems,
configuration, a clock, an event log, and persistence.

Its declarative definition is a World Pack ([`MODULE_SPEC.md`](MODULE_SPEC.md)). Its runtime is
the authoritative server ([`ARCHITECTURE.md`](ARCHITECTURE.md)).

```yaml
world:
  id: lakewood
  calendar: modern

systems:
  - movement
  - conversation
  - relationships
  - inventory
  - group_activity
  - economy
  - employment

cognition_profile:
  default: local_agents

presentation_profile:
  default: lakewood_realistic

network_profile:
  default: private_server
```

A world's identity is its semantics. Renderer, transport, deployment target, database, and
model provider are all interchangeable around it (INV-14).
