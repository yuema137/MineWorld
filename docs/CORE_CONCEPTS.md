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

## 6.1 Location: semantic place, optionally refined

Where something is, is one type with an optional refinement:

```text
Location
    place     PlaceId              always present, authoritative
    local     LocalPosition?       optional: where inside the place
    facing    Orientation?         optional: which way it is turned
```

```text
LocalPosition       x, y, z          millimetres from the place's own origin
Orientation         yaw              millidegrees, canonicalized into [0, 360000)
                    pitch?           millidegrees, optional, within ±90000
```

Both refinements are optional because a world may not model them, and a world that does not
loses nothing: a headless world and a 2D client work with the place alone, an embodied 3D
client fills both in, and every System is written against the same type either way. One
`Location` therefore describes both *"standing 1.2 m from Alice, facing her, inside the café"*
and *"in the café, position irrelevant"*
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§5–6, §§11–12).

Three properties are not negotiable:

1. **Fixed-point, never floating-point.** Positions are `i32` millimetres and angles are `i32`
   millidegrees. Positions reach the Event Log, the log is replayed, and floating-point
   arithmetic is not reproducible across platforms — so a float here would break the
   determinism the framework requires of a seeded run.
2. **No engine concept.** No mesh, navmesh, collider, camera, scene node, animation, skeleton
   or physics ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §12). A client maps a `Location`
   onto whatever its engine uses; the simulation never learns what that was.
3. **Hierarchy is a Relation, not a field.** That a kitchen is inside a café is a fact about two
   Places and lives as a Relation (§9), not as a parent field on this type.

## 6.2 SpatialRequirement: what an action needs of space

Whether an action needs proximity belongs to the contract of the System that provides it, and no
renderer decides it ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §7). The requirement is
therefore **data** a System declares, not code it hides:

```text
SpatialRequirement
    place                       Any | SamePlaceAsActor | Specific(PlaceId)
    within_range                Millimetres?     interaction radius
    requires_line_of_access     bool
    requires_target_available   bool
```

`talk`, `give item`, `open door`, `sit` and `use machine` declare what they need. `send message`,
`make phone call` and `apply for remote job` declare that they need nothing, which is as much a
declaration as the others.

Because it is data, it has two consumers rather than one: the authoritative server evaluates it,
and a client can be *told* it — inside an Affordance (§15) — without implementing the check.
The kernel supplies one evaluation of it, which answers `TooFarAway`, `TargetUnavailable` or
`PreconditionFailed`, and which a renderer never performs itself
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §8).

Two boundaries of that evaluation are stated rather than hidden:

- **Line of access is declared and not yet evaluated.** Deciding whether a wall stands between
  two positions needs world geometry that no layer owns yet. The declaration is carried so that
  a geometry provider can answer it later; until then the kernel does not pretend to have
  checked it.
- **A range requirement in a world with no continuous position means *same place*.** That is the
  finest proximity such a world can express. Passing everything would let a three-metre range
  reach another town; failing everything would make an action declared by a 3D-capable System
  unusable in a 2D world, which §§6.1 and 9 of this document forbid.

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

`CausedBy` is what makes causality traceable (INV-15), and it is never absent: an Event is
caused by an `ActionIntent`, by a `Process`, by another Event, by a System's own tick, or by the
world coming into existence. The last exists so that a world's initial facts are *explained*
rather than uncaused.

`Visibility` is what lets perception systems decide who could have learned of the event, and it
is never absent either: an Event that did not state its audience would leave a perception system
to choose a default, and the only available defaults are omniscience and silence (INV-13).
Declaring it is the emitting System's job, because only that System knows whether the fact was
shouted across a room or noticed by nobody.

`Provenance` records which system emitted it and, where relevant, which controller decision led
to it.

`Location` here is the **semantic** location: the Place the fact happened in (§6). It is not a
`Location` with a continuous refinement, because a millimetre position and an orientation are
properties an *entity* has rather than properties a *fact* has. A System whose events genuinely
carry continuous geometry — a movement system recording a new position — puts it in that
System's own typed Payload, where it is that System's contract rather than the kernel's.

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

## 12.1 What the world answers

An `ActionIntent` is answered with exactly one of three things:

```text
Accepted        with the Events the request caused
Rejected        with a reason
Unavailable     no enabled system provides this action at all (INV-10)
```

The reasons are kernel vocabulary, because every client must be able to show them
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §8):

```text
Busy                    TooFarAway              PermissionDenied
NoSupportedInteraction  TargetUnavailable       PreconditionFailed
Unavailable             the action exists and this attempt was refused without
                        further classification — distinct from the answer above,
                        which says the action does not exist in this world
```

A System may add a reason of its own, as a code it owns together with an optional note, so that
installing a System Pack never requires editing the kernel's list. A client that does not
recognize a System's code falls back to showing the request as refused.

None of these carries display text. What a player reads is presentation, and which language they
read it in is a Presentation Pack's decision, so a client maps a reason to its own wording
([`ART_DIRECTION.md`](ART_DIRECTION.md), [`MODULE_SPEC.md`](MODULE_SPEC.md)).

An intent may also report where the actor was when it was made. That is a *report* from a
client, not authoritative state: the server holds the authority and is free to evaluate against
its own ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §8, INV-9).

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

## 15.1 What an Observation contains

```text
Observation
    observer          which Person's view this is
    at                the world time it was taken
    self_location     where the observer itself is
    entities          the entities exposed to it, each with only the
                      components this observer is entitled to
    events            the Events this observer is entitled to have learned of
    affordances       what this observer may attempt, and the world's answer
```

"Structural" means specifically this: an Observation is a **value listing what was exposed**. It
holds no component store, no entity registry, no world handle and no query, so there is nothing
in it to widen. Asking it about an entity it does not list answers *nothing*, and there is no
second call that would answer more. A Controller that was not shown something cannot distinguish
that from the thing not existing, and that asymmetry is the feature (INV-13).

## 15.2 Affordance: the world's answer about what can be attempted

A Controller or a client still has to know what it may try. A 3D client shows a prompt over the
person the camera is pointing at; a 2D client greys out a menu entry; a language-model controller
is told what is possible instead of guessing. If the Observation did not carry that answer, each
of them would compute it from whatever it could see — which is the duplicated rule logic
[`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§8–9 identifies as an architectural failure.

```text
Affordance
    action_type           which action
    target                what it would be directed at, if anything
    available             whether it can be attempted right now
    unavailable_reason    why not, when it cannot
    requirement           the action's SpatialRequirement (§6.2), unevaluated
```

The server computes it; a client renders it. Three consequences follow, and all three are
contracts rather than conventions:

1. **The reason is carried, not just the refusal.** A client told only *that* something is
   unavailable can grey it out; one told `TooFarAway` can say so, and a player who is told
   nothing learns nothing.
2. **No display text.** The Affordance names the action type and the target; the client maps the
   type to its own wording and reads the target's *name* from a component of that target in the
   same Observation. A name is world data and must not be invented by a client, while wording and
   language are presentation and must not be invented by the kernel.
3. **The requirement travels unevaluated.** A client can show what an action needs — a reach, a
   place — without checking it. Checking remains the server's, in one implementation shared by
   every client (§6.2).

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
