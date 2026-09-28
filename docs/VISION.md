# MineWorld — Vision

**Status:** frozen positioning for v0.1 planning
**Audience:** coding agents and contributors. This is a specification of intent; every other
document must remain consistent with it.

This document answers exactly three questions: what MineWorld is, what MineWorld is not, and
what a creator can do with it.

---

# 1. One-sentence definition

> **MineWorld is an open-source, LM-native framework for building persistent, modular, playable
> game worlds.**

Second sentence, equally binding:

> **Worlds are composed from independent entities, simulation systems, controllers, and
> presentation layers rather than implemented as monolithic games.**

MineWorld is not intended to be one fixed game. It is infrastructure that lets people — assisted
heavily by language and generative models — create their own worlds by composing reusable
modules. The long-term goal:

> **Anyone should be able to build a game with LMs, without a traditional studio pipeline.**

## 1.1 LM-native means authored, not dependent

These two statements are both binding, and reading either without the other produces a different
project:

| | |
| --- | --- |
| **Authoring is LM-native** | Generated content is a first-class input. Worlds, characters, assets, styles and even systems may be produced by models, and the framework is shaped to receive that. |
| **Runtime is LM-independent** | A running world must not require a model. Rule controllers, human players and deterministic systems stand alone, and a world with every model unplugged is still a valid MineWorld world. |

The distinction is *creation versus execution*. "LM-native" describes how a world comes to
exist; it is never licence for the runtime to depend on a model. If unplugging the models makes
a **running** world meaningless, MineWorld has degenerated into an AI-NPC demo — the failure
this project was defined against. If unplugging them makes a world impossible to **build without
a studio**, MineWorld has failed at its purpose.

Three-line architectural slogan, used throughout the repository:

```text
Simulation creates reality.
Controllers propose actions.
Presentation observes reality.
```

MineWorld deliberately positions one abstraction level above "AI NPC framework". An LM-driven
character is one kind of `Controller`, not the point of the project.

And the worlds it builds are meant to be **played**, from inside:

```text
enter world → move through space → explore → enter places → encounter people
→ interact → participate in activities → travel elsewhere
→ continue living in the same persistent world
```

The player is a participant standing in the world, not an observer watching a simulation from
outside. Agent simulation exists to make that world alive; it is not the product by itself. The
binding form of this requirement is [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§1–12.

---

# 2. What MineWorld is

## 2.1 Infrastructure, not a game

MineWorld owns the **world**: identity, state, time, causality, interaction rules,
persistence, and authority. It does not own the renderer and it does not own the language
model.

Think of it as an operating system for game worlds:

```text
                    ┌──────────────────┐
                    │    MineWorld     │
                    │      Kernel      │
                    └────────┬─────────┘
                             │
       ┌──────────────┬──────┼──────┬──────────────┐
       ▼              ▼      ▼      ▼              ▼
    Entities       Systems  Time   Events      Persistence
       │              │
       ▼              ▼
 Person/Place       Social
 Item/Org           Economy
                    Employment
                    Combat
                    ...

             nothing above requires an LM
```

and then, attached from outside:

```text
Controllers                    Presentation

Human                          2D pixel
Rule policy                    stylized 3D
Behavior tree                  photorealistic 3D
RL agent                       text
Local LM                       custom engine
Cloud LM
```

## 2.2 Composition, not modification

A creator builds a world by installing and configuring independent modules. There are exactly
five publishable module kinds, specified in [`MODULE_SPEC.md`](MODULE_SPEC.md):

```text
Entity Pack        what exists in the world
System Pack        what is allowed to happen between the things that exist
World Pack         a specific world: people, places, organizations, initial state, config
Controller Pack    who decides what a character does
Presentation Pack  how the world is rendered
```

Composition example:

```text
Modern Life Entity Pack
+ Conversation System
+ Relationship System
+ Employment System
+ Local LM Controller
+ Pixel Presentation Pack
= a game
```

Replace only the last line with `Realistic 3D Presentation Pack` and the simulation is
bit-for-bit the same world.

## 2.3 The particle-physics principle

Entities exist independently. Interactions exist only when the corresponding system is
enabled.

```text
Person + Person

without a social system
→ no social interaction exists

with ConversationSystem
→ talking becomes possible

with InventorySystem + ItemTransferSystem
→ giving becomes possible

with GroupActivitySystem
→ invite / join / leave becomes possible

with EmploymentSystem
→ hire / work / fire becomes possible

with RomanceSystem
→ romantic relationships become possible
```

Therefore:

> Entities define what exists.
> Systems define how things can interact.
> Events record what happened.

`Person` is identical in all of those worlds. What changes is the set of permitted
interactions. That is what composability means in MineWorld.

## 2.4 A world with no language model still runs

The runtime half of §1.1, stated as the hard constraint it is:

> **If unplugging every language model makes a running world pointless, then it is not
> infrastructure — it is an AI-NPC demo.**

Rule-driven, scripted, RL-driven, and human-driven worlds must all be first-class. LM cognition
is an optional, replaceable controller implementation.

## 2.5 What LMs are expected to author

The creation half. Users should be able to generate and iterate on, with human judgement in the
loop at every step:

```text
world concepts          characters and biographies      places and buildings
visual assets           3D models, textures, materials   lighting configurations
animations              dialogue, quests, events         gameplay rules
interaction systems     economies, jobs, organizations   UI and presentation styles
world / asset / presentation / system / controller packs
```

Generation output is a **candidate**, never automatically content
([`DECISIONS.md`](DECISIONS.md) `ARC-9`). The workflow that makes it safe:

```text
LM generation → human curation → modular packaging → runtime validation → iterative refinement
```

This applies to code, art, world definitions, mechanics, characters, systems and presentation
alike. The more powerful a generated module, the stronger its validation: schema and type
checks, dependency and licence validation, integration tests, sandboxing, smoke runs, human
inspection. A module is not trusted because it compiles.

## 2.6 MineWorld is not a game engine

MineWorld must not reimplement mature engine technology. It does **not** build a GPU renderer,
graphics API, physics engine, animation system, audio engine, input framework, camera system,
shader compiler, scene editor, navmesh generator or asset importer.

```text
                     MineWorld
        ┌────────────────┴────────────────┐
   WE BUILD                          WE REUSE
        │                                 │
 Entity · Component                 GPU rendering
 Systems · Actions · Events         2D and 3D renderers
 Process · Person · Controller      physics · collision
 Persistence · networking semantics animation · audio · input
 World / Asset / Style contracts    camera · navmesh · shaders
 LM generation pipeline             particles · platform export
 Validation · package system
```

> **MineWorld does not build a rendering engine. It builds the semantic world layer above
> existing ones.**

The official reference client is **Godot** — MIT, cross-platform, 2D and 3D in one engine, and
extensible without vendor permission, which matches what this project is
([`DECISIONS.md`](DECISIONS.md) `DEP-9`). But Godot is not MineWorld: no generic contract may
expose a `Node`, `SceneTree`, `MeshInstance`, `Camera3D` or `NavigationMesh`. Those live in
renderer adapters, and the semantic world runs headless without any of them.

Performance follows the same discipline. A rendering problem is answered by profiling, engine
configuration, LOD, culling, asset optimization, then existing extensions — custom GPU code
last, if ever.

## 2.7 The kernel stays small

The kernel is responsible for identity, component storage, the world clock, spatial
semantics, action dispatch, the system registry, process scheduling, event sourcing, the
relationship graph, persistence, networking, permissions, and plugin lifecycle.

The kernel does not know how romance works, what employment is, how eating works, what coffee
costs, or even that people need to sleep. All of that lives in System Packs. Core primitives
stay at roughly ten concepts ([`CORE_CONCEPTS.md`](CORE_CONCEPTS.md)); `Vehicle`, `Building`,
`Job`, `Money`, and `Food` are explicitly **not** core:

```text
Building = Place  + StructureComponent
Vehicle  = Entity + TransportComponent + InventoryComponent
Job      = Relation(Person, Organization) + EmploymentComponent
```

If the core ontology starts listing `Dog`, `Car`, `House`, `Restaurant`, `School`,
`Hospital`, MineWorld has become a monolithic life simulator.

---

# 3. What MineWorld is not

```text
not The Sims, not inZOI
not GTA, not a Minecraft replacement
not a photorealistic game
not an "AI NPC framework"
not a starter kit you fork and edit in place
not a single monolithic life simulator

not a social-science simulator
not an agent benchmark
not an NPC chatbot framework
not a backend world-state database
not a dashboard for watching agents move
```

The last five matter because they are what MineWorld decays into if the playable-world
requirement is ever treated as optional. A world you can only watch has failed even if every
simulation test passes.

## 3.1 The boundary against demo / starter-kit projects

The difference is the abstraction boundary, not the scale. A demo separates its code into
folders yet still expects extension by editing source: adding one gameplay element means
touching game logic, agent code, and frontend at once; character definitions mix personality,
plans, and sprite references in one file; changing that data requires wiping the database.
Demo engines also carry demo assumptions — a world that must fit in memory each step, an
active-state budget measured in tens of kilobytes, conversations fixed at two participants.

Those are reasonable choices for a demo. They are disqualifying for infrastructure.

```text
demo / starter kit          MineWorld
fork source            →    install modules
modify game code       →    compose world
modify agent code      →    configure
modify frontend        →    run
```

## 3.2 What Minecraft is a reference for

Two separate things, neither of them visual style or gameplay.

**Ontology.** A minimal core plus explicit interaction rules plus enormous extension space:
`Block`, `Entity`, `Item`, `World`, `Recipe`, `Event` compose into far more than they enumerate.
MineWorld's domain is more complex, which is exactly why its core must stay small and its
complexity must live in optional systems.

**Embodiment.** The player has a physical position, controls movement directly, looks around with
a camera, approaches things spatially, targets nearby entities, and initiates interaction from the
world view. Distance and spatial accessibility matter. Interaction happens in the world, not
primarily through abstract menus — menus may supplement spatial interaction but never replace it
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §3).

What MineWorld adds on top is semantic depth: whether `Talk` succeeds can depend on identity,
distance, orientation, permissions, relationships, ownership, organization membership, the current
`Process`, a schedule, world time, object state, social context, and which systems are enabled.
The renderer evaluates none of that. It reports that the player pressed interact while targeting
Alice; the server decides what, if anything, that means.

---

# 4. What a creator can do with it

## 4.1 Independent choices

A creator chooses each of these independently, without forking the framework:

- what exists in the world;
- which interactions and simulation rules exist;
- how characters make decisions;
- how the world is rendered;
- whether the world is 2D or 3D;
- whether buildings have interiors, and at what granularity;
- which LM backend is used, if any;
- whether the world runs locally or on a cloud server;
- whether the world is single-player, private multiplayer, or public multiplayer.

The same semantic world runs as:

```text
headless simulation
        │
        ├── 2D Godot client        Demo A — architectural testbed, permanent
        ├── 3D Godot client        Demo B — embodied walking world, required
        ├── Unreal client
        └── custom client
```

Both reference clients are first-class and both are maintained. 2D is not a placeholder for 3D,
and 3D is not a cosmetic layer added late: a `Talk` initiated by a click and a `Talk` initiated by
walking up to someone and pressing a key must produce the same `ActionIntent`, or the architecture
is wrong ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §9).

The same character is controllable by:

```text
Rule-based policy
Local LM
Cloud LM
Human player
Scripted scenario controller
RL policy
Custom community controller
```

## 4.2 Same entities, different games

```yaml
# World A
systems: [time, movement, conversation, relationships]
```

Walking and talking. Nothing can be owned, bought, or exchanged.

```yaml
# World B
systems: [time, movement, conversation, relationships,
          inventory, item_transfer, economy, employment, property]
```

A life simulation.

```yaml
# World C
systems: [time, movement, inventory, crafting, survival, combat]
```

A survival game.

`Person` never changed.

## 4.3 Installing capability instead of building it

```bash
mineworld install modern-life
mineworld install employment
mineworld install restaurant
mineworld install relationships
mineworld install godot-realistic
mineworld create my-town
mineworld add-system university
```

After the last command the world gains `University`, `Student`, `Professor`, `Course`,
`Enroll`, `AttendClass`, `TakeExam`, `Graduate` — without the world author implementing a
university simulation. That leverage is the point of the module system.

---

# 5. Roadmap beyond the MVP

The MVP validates the architecture and nothing else; see [`MVP.md`](MVP.md).

## Phase 2 — World Creator

A visual creator experience replaces hand-editing YAML: world editor, character editor,
system selector, interaction configuration, asset binding, packaging — and style authoring, where
a creator uploads four to ten reference images, writes two or three sentences, picks 2D or 3D, and
the system generates an initial style manifest for them to correct
([`ART_DIRECTION.md`](ART_DIRECTION.md) §23).

```text
Create World

Name:            Lakewood
Style:           realistic / stylized / pixel
Dimension:       2D / 3D
Spatial detail:  exterior only / semantic interiors / room-level interiors

Simulation systems:
  ☑ conversation     ☑ relationships     ☑ inventory
  ☑ economy          ☑ employment        ☑ group activities
  ☐ romance          ☐ education         ☐ vehicles
  ☐ crime            ☐ combat            ☐ health

Agents:
  background NPC: local 8B
  major NPC:      local 32B
  human players:  enabled

Rendering:       Lakewood Realistic 3D

            [ Create World ]
```

### The creator experience this is all for

A creator should be able to say:

> *"A realistic small Japanese coastal town where I run a café and meet persistent characters."*

and have MineWorld tooling turn it into a World Pack, Asset Packs, a Presentation Pack, System
Packs and controller configuration — runnable without implementing every subsystem by hand. When
they ask for a university, an LM generates `Course`, `Enrollment`, `Professor`, `Student`,
`AttendClass`, `TakeExam`, `Graduate` **against explicit MineWorld contracts**, and MineWorld
validates the module before it runs.

Humans keep what matters: intent, taste, selection, evaluation, direction. LMs expand what one
person can build. The loop MineWorld optimizes is the one between human intent and generated
implementation — not the removal of the human from it.

## Phase 3 — Ecosystem

The community publishes Entity Packs, System Packs, World Packs, Controller Packs, and
Presentation Packs: 1990s American small town, modern Tokyo, university campus, space colony,
medieval village, research station, cruise ship. One framework runs them all.

## Phase 4 — Public living worlds

Cloud-hosted persistent servers that advertise what they are:

```text
World:        Lakewood
Players:      37 / 100
NPCs:         142
Systems:      social, economy, employment, property, vehicles
Presentation: realistic-3D
Cognition:    local-hosted-model
```

Players join an already-running society instead of starting a save. NPCs may have existed for
months before a player arrives.

---

# 6. Final vision

```text
Server starts.

Nobody logs in.

The world continues living.

People wake up.
People go to work.
Businesses open.
People meet.
Relationships change.
People move.
Organizations evolve.

A player connects.

They enter this existing world.

They do not become the center of the universe.

They simply become another participant in it.

They log out.

The world continues.
```

The long-term product is therefore not an AI NPC framework. It is:

> **an open runtime for persistent, composable, multiplayer living worlds.**
