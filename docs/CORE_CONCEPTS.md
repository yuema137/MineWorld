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

Traits, together with the identity facts (§4.1) and the views of mutable state (§4.3) that a reader needs by
name, are declared as **attributes** (§4.5).

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

## 4.5 Attributes

Specified by [`DECISIONS.md`](DECISIONS.md) `ARC-120` (ownership), `ARC-122` (the model), `ARC-123`
(generation), `ARC-125` (appearance descriptors) and `ARC-126` (backends and masks). The formats an author
writes are [`MODULE_SPEC.md`](MODULE_SPEC.md) §2 (data directories of an Entity Pack), §3.1 (what a System
Pack declares) and §4.1 ("Attributes"). Introduced by step S30
(`.structured-coding/plans/mvp0/step-30-person-attributes.md`).

**Definition.** An **Attribute** is a declared, typed property of a Person. Every attribute belongs to one
of the first three classes of state above:

```text
identity   §4.1   core.birth, core.natal-sex                 set at genesis
trait      §4.2   core.stature, core.chronotype, core.diet   slow-changing
view       §4.3   economy.wealth-band, employment.occupation  a projection of another System's state
```

A relationship (§4.4) is never an attribute: it is an edge between two entities, owned by the System that
models it (§9).

Every attribute is:

- **declared as data**, in an `attributes/` file of an Entity Pack, a World Pack, or a System Pack that
  ships it, never in code;
- **named by a qualified id**, `<namespace>.<slug>`, whose namespace is the declaring pack's id (`core` is
  the built-in catalogue's, `entities/person-attributes`);
- **owned by exactly one System** (INV-7), named in its declaration. The default owner is the `attributes`
  System Pack.

`Attribute` is a defined term with this meaning only.

**An attribute is not a Component.** A Component (§3.1) is one owning System's typed state. Attributes are
the declared vocabulary over such state:

- **Plain traits** — attributes no specialised System models, such as stature, chronotype or diet — are
  owned by the `attributes` System Pack, which holds a Person's values in one component, `traits`.
- **An attribute another System already models** stays with that System. `economy.wealth-band` is a
  projection of `economy`'s own state, computed by `economy` when the attribute is read or disclosed. Nobody
  else stores it: an owner's attribute is **a view, never a copy**.
- **No owner, no attribute.** An attribute whose owner the world does not enable does not exist in that
  world; declaring one is refused at load by name.

**Kinds.** Every attribute has one value kind, fixed by its declaration: `category` (one of a closed list),
`level` (one of an ordered list), `quantity` (an integer at a declared scale in a declared unit; never a
float), `flag`, `set` (a subset of a closed list or of a declared vocabulary), `text` (bounded prose, never
readable by a System) or `instant` (a world instant). A value is checked against its declaration where it is
written. There is no `unknown` value: an attribute that nothing states is **absent**.

**Visibility.** Disclosure is by perception only, never by what the observer knows (INV-13):

| Visibility | Disclosed to the holder | Disclosed to another observer who perceives the holder |
| --- | --- | --- |
| `perceivable` | the value | the declaration's coarser `perceived_as` wording if it has one, else the value |
| `learnable` | the value | nothing; it may be learned only in conversation, and what is learned is the learner's memory (INV-4) |
| `private` | the value | nothing, ever |
| `hidden` | nothing | nothing |

A System that declares an attribute it reads may read it whatever its visibility, through a typed handle
obtained when it is installed; no System reads a `text` attribute.

**Mutability.**

| Mutability | Meaning |
| --- | --- |
| `fixed` | set at genesis; never changes |
| `evolving` | changes only through its owner, on an event the owner accepts, and every change cites its cause (INV-15) |
| `derived` | computed when it is read, from stored state and the observation's instant; never stored and never stated as a fact |

Another System changes a plain trait only by proposing the change to `attributes`, which accepts or refuses
it; a specialised owner changes its own attribute through its own actions.

**Generation.** A world's people receive the attribute values their files do not state from a **population
profile**: per-world data, sampled deterministically from the world's **population seed**, which is world
content and never the run's seed. The biography wins: a value a person's file states, or that a casting
ruling pinned, is never re-drawn, and nothing reads a person's name. Which catalogue and which way of
generating people a world uses is its **backend**, chosen as data; the default backend is MineWorld's own,
and a backend runs no code.

**What is not an attribute.**

- **A name.** A display name is identity owned by `naming` (§4.1); it is never input to generation.
- **A memory.** What someone said about themselves is the listener's subjective memory (INV-4, §5.3). It is
  never world state and never widens an observation.
- **A look key.** `appearance`'s look key is opaque and is the only thing a renderer binds. Apparent
  descriptors (stature, hair colour, apparent age band) are `perceivable` attributes, and a descriptor never
  selects art (`ARC-125`).
- **Provenance.** Where a value came from (authored, ruled, sampled, defaulted) is authoring metadata,
  reported by tools and never saved as world state (§3).

**The kernel learns nothing** (INV-12). The attribute vocabulary, its values, handles and sampler live above
the contracts and below the System Packs; no type, word or id of it is in the kernel or the contracts.

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

### Forgetting: the world log never forgets, subjective memory may

Subjective memory may forget. The world's Event Log may not: it is the single source of truth
(INV-11), so it is compacted or archived and never forgotten. Every Event ID stays resolvable in the
save for as long as the save exists.

Forgetting happens in one of two places, and ownership decides which:

1. **System-owned person state** — a relationship's familiarity and regard, an acquaintance list, a
   conversation's remembered lines — is a Component of a System. The owning System forgets it by a
   pure rule inside its own reduction, so replay reproduces it exactly (ARC-25). Decay writes no
   fact and starts no Process; a level change caused by decay is observed lazily, and a biographical
   fact about it is emitted only by a reduction that touches the pair. No model may touch this place.
2. **Controller-owned memory** — a seat's store of what it perceived — is not world state. Its
   controller forgets it through a memory policy, under bounds the host enforces outside any policy.
   The world never reads this store, so forgetting in it changes no byte of any save.

A controller cannot forget world state for the world; it can only stop reading it.

Whichever place forgets, what the person no longer holds is recorded as the **forgotten set**: the
Event ID ranges that were perceived and are no longer held, kept sorted and bounded. A forgotten
Event ID is never shown to the person again and is never cited by anything the person says, but it
still resolves in the save, so tools may say that a person has forgotten a span of time. Memory
remembers or forgot; it never deletes the truth. Its bounds are stated in
[`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3, and the decisions in `DECISIONS.md` `ARC-86` and `ARC-87`.

### Shared storage is not shared knowledge

Storing a perceived fact once for several persons is a storage detail. Each person still remembers
only what they perceived, in their own rendering. Shared storage is not shared knowledge.

A fact record may therefore be held in one place for every person who perceived that fact, but a
person's access to it is their perception and nothing else: no scope, policy, query, cache or
service may grant a person access to a fact they did not perceive. How a controller's memory is
stored, shared and sized is stated in [`ARCHITECTURE.md`](ARCHITECTURE.md) §9.3, and the decisions
are `DECISIONS.md` `ARC-98` and `ARC-99`.

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

A controller normally reads the compression of its own perceived history; the objective biography's
compression is a tool's view ([`DECISIONS.md`](DECISIONS.md) `ARC-59`). Debugging and evaluation
tools follow the Event IDs back to the original facts. Compression therefore reduces **context**, never historical
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

## 6.1 The spatial frame, stated because two clients cannot guess it alike

`LocalPosition` and `Orientation` fix units and ranges but said nothing about the frame, and the
renderer spike showed what that costs: Godot 3D is −Z forward and Godot 2D is +Y down, so each
client invented a convention and wrote its own sign flips. Two independently written clients
would disagree, and the symptom is NPCs facing backwards.

```text
+x  east        +y  north        +z  up        right-handed
yaw  measured from +y toward +x, so 0 mdeg faces north and 90 000 mdeg faces east
pitch  positive looks up
```

Worked example: a person at `LocalPosition { x: 2000, y: 0, z: 0 }` with
`Orientation { yaw: 180_000 }` stands two metres east of the place origin, facing south. A 2D
top-down client maps `+y` to screen-up and flips its own axis; a 3D client maps `+y` to −Z. Both
conversions live in the adapter, and neither is the contract's business — but the frame is.

## 6.2 Location: semantic place, optionally refined

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

## 6.3 SpatialRequirement: what an action needs of space

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

## 6.4 Sit point: where a person may sit, as place data

A **sit point** is a place-local, authored position at which one Person may sit. It is semantic
place data, owned by the `posture` System Pack (`DECISIONS.md` `ARC-147`). It is not an Entity, not
an Item, and not geometry: a bench is drawn by a Presentation Pack and is solid to bodies, and the
sit point is what the server knows about where a person may sit on it.

```text
SitPoint
    key       slug, 1 to 48 bytes of a–z, 0–9, '-' and '.'; unique within its place
    at        x and y in millimetres in the place's frame: the seated person's position, on free floor
    facing    yaw in millidegrees: the direction a seated person faces
    group     optional slug naming the furniture the point belongs to, for presentation to name once
```

- **Sit point is the term; "seat" is not.** "Seat" already names a player's or an agent's seat in
  other specifications. A sit point is never called a seat in a contract, a key, an event or a
  document that defines one.
- **The position is free floor.** `at` is the centre of the seated person's footprint in front of the
  seat's front edge, clear of every solid by a person's radius and the gap bodies states. The server
  never learns a height: the pelvis over the seat, the knees and the seat's height belong to a
  Presentation Pack. Two sit points of one place stand at least two person radii and the gap apart,
  so two seated people never overlap.
- **Sitting is a state of a person, not of a sit point.** A seated person holds `Seated { place, point }` (a
  row on the Person; the place is kept because a key is unique only within its place); a person who
  stands holds no such row. Whether a sit point is taken is derived
  from the people in its place, never stored on the point.
- **A seated person is at their point.** Their Presence is their sit point's place and position.
  Facing may change while seated; moving away from the point stands the person up.
- **Nothing pushes a seated person.** Other people's movement stops short of a seated person, and
  `shove` is refused for one (`ARC-147`; bodies' part is the record's second step, S34-b). Until that
  step lands, a seated person moved by anybody is stood up, so a seated person is still always at
  their point.

Each client draws a sit point's bench, its seated figure and its settling animation from the same
disclosure; what the server answers about sitting is the same for a 2D click and a 3D look.

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

In MVP-0, an Item authored in a World Pack is an item kind, and what anybody holds of it is a count
of that kind: stacked items only ([`DECISIONS.md`](DECISIONS.md) `ARC-36`).

An item kind may carry an authored **reference value**, its typical worth in the world's currency. It is never a
price: nobody is charged it, and what a kind costs is always the asking price of the seller one buys from
(`ARC-113`, `ARC-112`).

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

An Organization sells through a shop: a Place where its asking prices are on display and can be bought without
the Organization being present. The Persons its shop names as keepers change those prices while standing in the
shop (`ARC-112`). A Person sells for itself, face to face.

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

Every EventType has exactly one **owner** — the System that defines its vocabulary (schema and
public constructor) and the **only** System that reduces it into owned state. Usually the owner is
also the only System that emits it. Another System may state a fact of that type only if it
declares the emission, declares a dependency on the owner, and builds the payload with the owner's
constructor; the kernel refuses the first two at installation otherwise. The owner still decides:
it may refuse a fact its state may not take, at construction and again at reduction. Provenance
then names the System that *stated* the fact, and the type's owner names whose vocabulary it is
(`DECISIONS.md` `ARC-26`). Example: `MovementSystem` decides that a move is legal and states
`PresenceSystem`'s `Arrived`; only `PresenceSystem` writes where the person now is.

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

The event that carries a cross-domain effect is normally the requester's own (`WageDue` is
`EmploymentSystem`'s). When the owner of the state cannot depend on the requester — because the
requester already depends on the owner to read its state — the requester instead states a fact in
the owner's vocabulary, under the conditions of §11 (`ARC-26`). Either way, only the owner writes
its components.

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

For example, a perceived Person's entry carries, among its components, the `traits` component filtered for
this observer (§4.5): the holder's own observation lists every non-hidden attribute value, and another
observer's lists only the `perceivable` ones, in their `perceived_as` wording. Nothing in the filtered
component can be widened into the rest.

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
    payload               the complete request, when the offering system can state one
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

An Affordance that carries a `payload` is a **complete affordance**: the owning System has stated
the exact request it would accept, so a requester — a Controller or a client — may submit it
unchanged, labelled with the Affordance's own action type and target, without knowing the action's
payload shape. Only the owning System makes an offer complete, and the payload cannot be attached
to an offer of another action type. The server still validates whatever is submitted, and a
Controller that submits a complete affordance has not created an interaction: it has attempted one
the world offered, and a world without the owning System offers none (INV-10). An action whose
payload is free-form — an utterance, a position — cannot be offered complete and is attempted by a
requester that knows it by name ([`DECISIONS.md`](DECISIONS.md) `ARC-34`).

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

---

# 17. Time

A world has one clock, `WorldTime`: whole simulated seconds from the world's epoch. Four terms say how
it relates to real time ([`DECISIONS.md`](DECISIONS.md) `ARC-67`). They are not synonyms of one another,
nor of `Process` or `Event`.

```text
calendar time   WorldTime. Every Event, every Process wake-up and every Observation is stamped in it;
                dates, the sun, weather, routines, shifts and wages are functions of it. A System Pack
                gives it a meaning (a calendar pack says which date an instant is); the kernel does not.
embodied time   real (wall-clock) time as a body experiences it: how fast a player walks, how long a
                line takes to type, how long an animation or a physics impulse plays. NOT a world
                quantity: no Event, Component, Process or rule carries it. It exists only as the
                cadence at which embodied inputs reach the server.
time scale      world seconds per wall second while a hosted world runs (an integer ≥ 1). Host pacing,
                not world state.
paused          the host pacing state in which world seconds do not pass. Not world state.
```

Rules that follow:

1. No System Pack, Controller decision or Event depends on the time scale or on paused. Only the host
   and the clients see them.
2. A per-request rule — a stride, an impulse, a spoken line — is resolved at the instant its
   `ActionIntent` arrives, so the time scale cannot reach it.
3. Headless `run` has no time scale: it advances calendar time as fast as the machine allows.
4. A calendar's local midnight is every multiple of 86 400 s from instant 0, the convention every
   calendar-driven pack shares.
