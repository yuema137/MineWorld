# MineWorld Engineering Rules

**Status:** mandatory pre-implementation rules
**Audience:** coding agents and contributors. Read this before modifying production code.

## How this document relates to the others

Sections **1–12** are the authoritative product and spatial-architecture requirements. They are
stated only here, and they bind every contract, system, and client. In particular §§2–3 promote
a playable 3D client from "later, optional" to a required architecture target, which supersedes
the earlier statement in [`MVP.md`](MVP.md) §7 that a 3D renderer comes only after kernel
contracts stabilize.

Sections **13–22** are a deliberately short pre-implementation checklist over rules whose
complete statement lives in [`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md). Where the two
differ in detail, that document governs; nothing here relaxes it.

Before modifying production code, read and follow:

- `docs/VISION.md`
- `docs/ARCHITECTURE.md`
- `docs/ENGINEERING_STANDARDS.md`
- relevant module contracts/specifications

These rules are mandatory unless an approved design document explicitly overrides them.

---

## 1. Product North Star: MineWorld Is a Playable Living World

MineWorld is not primarily:

- a social-science simulator;
- an agent benchmark;
- an NPC chatbot framework;
- a backend world-state database;
- a dashboard for watching agents move.

MineWorld is fundamentally infrastructure for building **playable walking / exploration / travel-oriented worlds**.

The intended player experience is:

```text
enter world
→ move through space
→ explore
→ enter places
→ encounter people
→ interact with people and objects
→ participate in activities
→ travel elsewhere
→ continue living in the same persistent world
```

The player should feel that they are **inside the world**, not observing a simulation from outside.

Agent simulation exists to make that world alive.

It is not the product by itself.

---

## 2. Both 2D and 3D Must Remain First-Class

MineWorld must maintain reference implementations for both:

```text
2D playable world
3D playable world
```

They serve different purposes.

### 2D Reference World

The 2D client is the fastest environment for validating:

- world state;
- entities;
- movement;
- interactions;
- multiplayer;
- persistence;
- agent behavior;
- system composition.

It should remain lightweight and easy to run.

It is an architectural and integration testbed, not merely a temporary mock UI.

### 3D Reference World

The 3D client validates that MineWorld can support the intended final category of experience:

> a first-person / embodied walking and exploration game inside a persistent living world.

The 3D implementation must therefore be treated as a required architecture target, not an optional cosmetic renderer added much later.

Core contracts must not make assumptions that prevent a real-time 3D client.

---

## 3. 3D Interaction Reference: Minecraft-Like Embodiment

For basic 3D movement and interaction semantics, use **Minecraft as an interaction reference**, not as a visual or gameplay template.

The important properties are:

```text
player has a physical position in the world

player directly controls movement

player looks around using a camera

player approaches things spatially

player targets nearby world entities

player initiates interactions from the world view

distance and spatial accessibility matter
```

Typical interaction flow:

```text
walk toward café
→ enter café
→ look at NPC
→ interact
→ conversation begins
```

or:

```text
walk toward object
→ target object
→ interact
→ available world action executes
```

The player should generally not interact with the world primarily by choosing locations or entities from abstract menus.

Menus may supplement the experience but should not replace spatial interaction.

---

## 4. MineWorld Is More Semantically Complex Than Minecraft

Minecraft is useful as a reference for movement and direct world interaction, but MineWorld's semantic world is substantially richer.

A MineWorld interaction may depend on:

```text
identity
location
distance
orientation
permissions
relationships
ownership
organization membership
current Process
schedule
world time
object state
social context
enabled Systems
```

For example:

```text
Player looks at Alice
        ↓
ConversationSystem available?
        ↓
Alice reachable?
        ↓
Alice currently available?
        ↓
Alice willing to interact?
        ↓
conversation begins
```

The 3D renderer must not implement these rules itself.

It only gathers the player's physical interaction intent.

The authoritative server determines whether the interaction is valid.

---

## 5. Separate Semantic Space From Render Geometry

MineWorld must distinguish:

### Semantic world position

Used by simulation.

Examples:

```text
Cafe
Cafe.Kitchen
Cafe.Counter
LakePark
Apartment_12
```

### Render-space position

Used by a specific client.

Examples:

```text
(x, y)
(x, y, z)
navigation mesh location
animation anchor
```

The kernel must not depend on Godot, Unreal, meshes, colliders, scene nodes, or specific coordinates unless provided through a generic spatial contract.

A world should be representable as:

```text
2D
3D
headless
```

without changing its underlying social/economic state.

---

## 6. Movement Is a Core World Capability

Movement is not merely a rendering animation.

MineWorld should distinguish:

```text
MoveIntent
Travel Process
Spatial State
Rendered Movement
```

Example:

```text
Human input:
move forward

3D client:
interprets physical controller input

World:
updates authoritative spatial state

Renderer:
shows corresponding movement
```

For longer travel:

```text
Player chooses to travel to another town
        ↓
TravelSystem creates Process
        ↓
world time advances
        ↓
arrival Event
        ↓
new Place becomes current location
```

Walking across a room and traveling between cities are therefore different scales of the same broader spatial model.

Do not collapse all movement into either:

- renderer-local physics only; or
- abstract teleportation between locations.

MineWorld must eventually support both continuous local movement and higher-level travel.

---

## 7. Interaction Must Be Spatially Grounded Where Appropriate

Actions should declare whether they require spatial conditions.

Examples:

```text
talk
give item
open door
sit
use machine
pick up object
```

may require:

```text
same Place
within interaction radius
line of access
target available
```

Other actions such as:

```text
send message
make phone call
apply for remote job
```

do not require physical proximity.

This requirement belongs to the relevant System contract.

The renderer does not decide it independently.

---

## 8. Renderers Request Actions; They Do Not Implement World Rules

A 3D client may detect:

```text
player pressed interact
target = Alice
```

It sends something equivalent to:

```text
InteractIntent(
    actor = player,
    target = Alice
)
```

The server may resolve this into:

```text
StartConversation
```

or return:

```text
Unavailable
Busy
TooFarAway
PermissionDenied
NoSupportedInteraction
```

The client must not directly start or modify semantic world processes.

The same rule applies to the 2D client.

---

## 9. Shared Interaction Semantics Across 2D and 3D

2D and 3D reference clients must exercise the same core world actions whenever possible.

For example:

```text
Move
Talk
GiveItem
Buy
Invite
JoinActivity
Work
UseObject
```

A 2D renderer might initiate `Talk` by clicking an NPC.

A 3D renderer might initiate `Talk` by approaching the NPC, looking at them, and pressing an interaction key.

Both must ultimately produce the same semantic ActionIntent.

This is an important architecture test.

If the 2D and 3D clients require separate implementations of business rules, the architecture is wrong.

---

## 10. Reference Demo Requirements

MineWorld development must maintain two evolving integration demos.

### Demo A — 2D Living World

Purpose:

> fast architectural validation.

Must demonstrate:

```text
multiple Persons
Places
movement
conversation
relationships
basic items
basic group activity
persistence
human + agent controllers
multiplayer connectivity
```

---

### Demo B — 3D Walking World

Purpose:

> validate actual MineWorld gameplay.

The initial 3D demo does not need high visual fidelity.

It must demonstrate:

```text
first-person or embodied player movement
camera/look controls
collision / basic navigation
walkable environment
enterable Place
NPC represented physically
approach NPC
spatially initiate conversation
interact with at least one Item/Object
human + AI-controlled Persons sharing the same world
server-authoritative state
```

The first 3D demo should prioritize **interaction correctness and embodiment**, not graphics quality.

---

## 11. Do Not Optimize Only for the 2D Demo

The 2D client is easier to implement.

That must not cause core architecture to accumulate assumptions such as:

```text
tile-only movement
click-only interaction
instant movement
single-room locations
no orientation
no local geometry
```

Any major spatial or interaction contract should be reviewed against this question:

> **Can this reasonably support a Minecraft-like embodied 3D client later without redesigning the kernel?**

If not, reconsider the contract before merging it.

---

## 12. Do Not Optimize Only for the 3D Demo Either

Conversely, do not encode:

```text
mesh
animation
physics engine
skeleton
camera
scene tree
navmesh
```

into generic MineWorld entity contracts.

The 2D client and headless server must remain fully valid implementations.

The correct architecture is:

```text
Semantic World
      │
      ├── Headless
      ├── 2D
      └── 3D
```

not:

```text
3D Game Engine
      └── everything else attached to it
```

---

## 13. Architecture First

MineWorld is modular infrastructure, not a monolithic game.

Before implementing a feature, identify:

1. which module owns it;
2. what state it owns;
3. what contracts it exposes;
4. which existing modules it depends on;
5. how it will be integration-tested;
6. how the feature behaves in both headless and playable contexts where relevant.

If a feature requires invasive changes across unrelated modules, stop and report an architecture problem rather than patching around it.

---

## 14. Rust First

Prefer Rust for long-lived infrastructure, deterministic simulation, state, networking, scheduling, persistence, and core systems.

Use another language only when it provides a meaningful ecosystem or implementation advantage.

Typical exceptions include Python for cognition/ML and engine-specific languages for rendering adapters.

Cross-language boundaries must remain explicit and typed.

---

## 15. Preserve Module Boundaries

Do not directly mutate another system's owned state.

Communicate through typed:

- actions;
- events;
- processes;
- public interfaces.

Do not introduce hidden global state or cross-module implementation dependencies.

Avoid god objects and central managers accumulating unrelated responsibilities.

---

## 16. Strong Types

Do not pass arbitrary dictionaries, `Any`, or unstructured JSON through core boundaries when a domain type can be defined.

Important domain concepts and cross-module messages must use explicit schemas/types.

Make invalid states difficult to represent when practical.

---

## 17. Keep Code Understandable

Prefer simple, readable implementations.

Warning thresholds for handwritten production code:

- functions around 50 lines deserve review;
- functions around 100+ lines are a strong warning;
- files around 500 lines deserve review;
- files around 800+ lines are a strong warning;
- multi-thousand-line handwritten files are not acceptable.

These are architectural warnings, not mechanical limits.

---

## 18. Integration Tests First

Do not optimize for unit-test count or coverage percentage.

Testing priority:

1. integration tests;
2. contract tests;
3. regression tests;
4. end-to-end smoke tests;
5. meaningful unit tests.

Do not add trivial tests merely to increase test count.

---

## 19. Test Real Execution

A feature is not validated merely because isolated tests pass.

Core validation should exercise flows such as:

```text
create world
→ start server
→ connect client
→ move through world
→ interact with entity
→ submit semantic action
→ resolve system behavior
→ emit event
→ update world
→ persist
→ restart
→ continue
```

For rendering-related work, actually run the relevant reference client.

Do not infer that the 3D experience works merely because the server tests pass.

---

## 20. Change Amplification Check

Before finalizing a feature, ask:

> How many unrelated existing modules had to change?

Adding a new MineWorld capability should primarily require:

```text
new/updated module
contracts
registration/configuration
integration tests
```

If adding one feature requires widespread unrelated edits, revisit the architecture.

---

## 21. Definition of Done

A substantial feature is done only when:

- module ownership is clear;
- boundaries remain clean;
- types/contracts are explicit;
- automated checks pass;
- meaningful integration tests pass;
- the relevant runnable scenario has actually been exercised;
- existing reference worlds continue to work;
- 2D/3D compatibility implications have been considered where relevant;
- documentation is updated where public behavior changed.

---

## 22. Primary Maintainability Test

Always ask:

> **Does this change make the next similar feature easier to add without modifying unrelated code?**

And for world-facing features:

> **Can the same semantic capability be used by both 2D and 3D clients without duplicating game logic?**

If either answer is no, reconsider the design.

MineWorld should grow through independently composable capability, not increasing global coupling.
