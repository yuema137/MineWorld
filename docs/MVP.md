# MineWorld MVP-0 — Composable Worlds

> **MVP success is not measured by how intelligent an NPC appears. It is measured by whether
> materially different games can be constructed by composing independently installable systems
> without modifying the kernel.**

**Status:** frozen MVP definition for v0.1 planning
**Audience:** coding agents. Vocabulary: [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md). Layering:
[`ARCHITECTURE.md`](ARCHITECTURE.md). Packaging: [`MODULE_SPEC.md`](MODULE_SPEC.md). Mandatory
pre-implementation rules: [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md).

**Revision 2026-09-25.** [`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §§2–3 and §10 promote an
embodied 3D client from "after the kernel stabilizes" to a required architecture target with its
own reference demo. This document is revised accordingly: §7 now defines three presentation
modes and two reference demos, and §9 gains `AC-13` and `AC-14`. The primary criterion `AC-1` is
unchanged.

---

# 1. What the MVP is for

The MVP is not an attempt to build a life-simulation game, and not an attempt to demonstrate
convincing NPC dialogue. Generative-agent work has demonstrated conversational plausibility
many times over; repeating it proves nothing about this architecture.

The MVP exists to answer one question:

> Can two materially different games be built by composing modules, without touching the
> kernel, `Person`, the renderer, or the controller implementations?

---

# 2. The primary acceptance test

## World A — Social Café

```text
Entities:
    Person
    Place

Systems:
    Time
    Movement
    Conversation
    Relationship
    GroupActivity
```

Available behavior:

```text
walk
talk
get to know people
invite
do something together
```

## World B — Market Town

Same kernel. Same `Person` schema. Same renderer. Same controllers. The only change is
installed systems and world configuration:

```text
+ Item
+ Inventory
+ Economy
+ Employment
```

Available behavior becomes:

```text
own items
exchange items
work
earn money
spend money
run a shop
```

## The test

```text
World A
+ install economy
+ install inventory
+ change world config
= World B
```

The MVP succeeds only if that transformation requires **no modification** to:

```text
kernel
Person
renderer
controller implementations
```

This is the highest-level acceptance criterion of the project, and it is frozen.

---

# 3. MVP world content

One very small town, sized for iteration speed rather than richness.

```text
10–15 Persons

5 Places:
    apartments
    cafe
    convenience store
    park
    workplace

~20 Item Types

2 Organizations:
    cafe
    store

2 Jobs
```

---

# 4. MVP systems

Enable only:

```text
Time
Places
Movement
Schedule
Conversation
Relationships
Inventory
ItemTransfer
GroupActivity
BasicEconomy
BasicEmployment
```

Do **not** implement yet:

```text
romance            children           health
crime              combat             vehicles
property market    education          construction
business ownership complex interiors  travel between cities
```

Each omission is a System Pack that a later world installs. None of them may require a kernel
change to become possible — that is exactly what the MVP is testing.

---

# 5. MVP interactions

A Person can:

```text
move            talk             work
eat             buy              give item
invite          accept invitation  reject invitation
join group activity  leave group activity
sleep
```

That set is sufficient to observe whether interesting social trajectories emerge. Emergence is
a welcome observation, not an acceptance criterion.

---

# 6. MVP controllers

```text
8–12 LM-driven NPCs
up to 4 human players
```

Controller kinds:

```text
HumanController
RuleController
LMController
```

`LMController` supports Ollama plus one generic OpenAI-compatible endpoint. No sophisticated
multi-agent framework is required, and none should be introduced.

Per the implementation sequence in [`../CLAUDE.md`](../CLAUDE.md) §5, **no language model is
attached before the headless demo exists**. A seeded rule-based configuration must be able to
run the whole world with no model present.

---

# 7. MVP presentation

Three presentation modes, all first-class, all shipping within this effort
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §2):

```text
headless        no renderer at all; the simulation's own truth
2D              Demo A — fast architectural validation
3D              Demo B — the experience the project actually exists to enable
```

Headless is the baseline: the world runs, persists, and replays with no client attached. The two
clients then prove the same semantic world can be inhabited two different ways.

## 7.1 Demo A — 2D living world

A simple Godot 2D client — isometric in the default Presentation Pack (`DECISIONS.md` `ARC-14`), a
north-up plan with no pack; nothing semantic depends on the projection (operator, 2026-10-08). Its
purpose is speed: it is the cheapest place to validate world
state, movement, interaction, multiplayer, persistence, agent behavior, and system composition.
It is a permanent integration testbed, **not** a temporary mock UI to be discarded once 3D
exists.

Must demonstrate: multiple Persons, Places, movement, conversation, relationships, basic items,
basic group activity, persistence, human and agent controllers, multiplayer connectivity.

## 7.2 Demo B — 3D walking world

An embodied first-person client. Visual fidelity is explicitly not the goal; interaction
correctness and embodiment are ([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §10).

This does not conflict with [`ART_DIRECTION.md`](ART_DIRECTION.md), which defines the default
*Presentation Style Pack* and its own acceptance criteria (§21). Those criteria judge the default
style pack once it is populated with assets; they are not gates on `AC-14`. MVP-0 ships the pack's
skeleton — manifest, art direction, references — and a Demo B built from simple geometry. Making
Demo B actually look like the references is MVP-1 work, and the reason the direction is fixed now
is so asset selection never has to be redone.

Must demonstrate:

```text
first-person or embodied player movement
camera / look controls
collision / basic navigation
walkable environment
enterable Place
NPC represented physically
approach NPC
spatially initiate conversation
interact with at least one Item / Object
human and AI-controlled Persons sharing the same world
server-authoritative state
```

## 7.3 Why both, from the start

The 2D client is easier, and that is the danger. Left alone it invites contracts that assume
tile-only movement, click-only interaction, instant movement, single-room places, no orientation,
and no local geometry — each of which quietly forecloses the 3D experience
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §11). Every spatial or interaction contract in
this effort is therefore reviewed against one question before it merges:

> Can this reasonably support a Minecraft-like embodied 3D client later without redesigning the
> kernel?

The converse rule is equally binding: no mesh, animation, physics, skeleton, camera, scene tree,
or navmesh concept enters a generic MineWorld contract
([`ENGINEERING_RULES.md`](ENGINEERING_RULES.md) §12).

# 7.4 What MVP-0 actually ships

MineWorld is not an application. It is **an installable world runtime, a set of developer tools,
and some reference clients** — closer in shape to Godot, a Minecraft server, or ROS than to a
game executable. The split that matters:

```text
             MineWorld
                 │
        ┌────────┴────────┐
        │                 │
   Infrastructure     Applications
        │                 │
   World Server       2D Client
   Kernel             3D Client
   Systems            Launcher, editor, studio — all later
   SDK
```

The left column is the project. The right column is what the project ships to show the left
column works, and a community creator may replace all of it.

MVP-0's deliverable is therefore a set of artefacts, not a demo video:

| Artefact | Invocation | What it is |
| --- | --- | --- |
| World server | `mineworld server worlds/social-cafe` | the authoritative runtime: state, systems, events, processes, persistence, networking, controllers. Runs with nothing attached. |
| 2D client | `mineworld-2d` | a real program you open and play: walk, see NPCs, click a person, talk |
| 3D client | `mineworld-3d` | a real program you open and play: WASD, mouse look, walk up to an NPC, aim, press E |
| Developer CLI | `mineworld create \| validate \| run \| inspect` | how a world author first touches MineWorld |
| Sample World Packs | `worlds/social-cafe`, `worlds/market-town` | copy one, change people, places and enabled systems, get your own world |

Explicitly **not** MVP-0: a launcher, a world editor GUI, a MineWorld Studio shell. Those are the
product surface over this runtime and are Phase 2 work; building a GUI before the runtime is
proven would be building the shell of a thing that does not exist yet.

# 8. MVP networking

One binary, three deployments:

```text
single player:         localhost
private multiplayer:   LAN / direct server
cloud:                 Docker deployment on a VPS
```

No matchmaking, no global account service, no server browser. Authentication is a server
invite token plus a player nickname ([`NETWORKING.md`](NETWORKING.md) §9).

---

# 9. Acceptance criteria

The MVP is successful only when **all** of the following hold. Each is stated as an observable
test, because an unobservable criterion is not a criterion.

| ID | Criterion | Observable test |
| --- | --- | --- |
| **AC-1** | Composability (primary) | World A plus installed Inventory/Economy/Employment plus config change yields World B, with no edit to kernel, `Person`, renderer, or controllers. |
| **AC-2** | Modularity | Disabling `ItemTransferSystem` removes item-giving interactions with no change to `Person` code, and the world still runs. |
| **AC-3** | Renderer independence | The simulation continues correctly while the Godot client is disconnected, and a client may reconnect to the running world. |
| **AC-4** | Cognition independence | Replacing the LM backend requires no change to the World Pack. |
| **AC-5** | Controller independence | A human takes control of an existing NPC without destroying that Person's biography, relationships, inventory, or employment. |
| **AC-6** | Persistence | Restarting the server preserves the world exactly. |
| **AC-7** | Networking | Several human clients inhabit the same world simultaneously and interact with the same NPCs. |
| **AC-8** | Local/cloud parity | The same World Pack runs on a laptop and inside a cloud Docker container with no semantic differences. |
| **AC-9** | Causality | Every significant state mutation traces to an `ActionIntent`, a `Process`, or a system `Event`. |
| **AC-10** | Bounded cognition context | After 100 simulated days, character history does not require feeding all historical events to a model; biography compression stays bounded while original event provenance is retained. |
| **AC-11** | Headless stability | A seeded rule-based configuration runs at least hundreds of simulated days with no renderer and no model. |
| **AC-12** | Determinism | The same initial state, inputs, system versions, and seed reproduce the same run, excluding explicitly non-deterministic external controller calls. |
| **AC-13** | 2D / 3D semantic parity | `Talk` initiated by clicking an NPC in Demo A and by approaching, looking at, and pressing interact in Demo B produce an **identical semantic core** — `actor`, `action_type`, `target`, payload — resolved by the same system to the same result. Neither client implements any validity rule: distance, availability, permission and willingness are all decided server-side. |

**Corrected 2026-09-26.** This criterion previously demanded a *byte-identical* `ActionIntent`,
which the renderer spike proved unachievable — and wrong to want. Three fields legitimately
differ: `action_id` and `issued_at` are per-request, and `actor_location` is *designed* to differ,
since a 2D client that models no position sends none. Demanding byte equality would have forced
the two clients to converge on things the contract deliberately lets them disagree about. The
comparison is defined once, in the server, so the S12/S14 test cannot quietly compare a different
set of fields.
| **AC-14** | Embodiment | In Demo B a player moves in first person through a walkable environment, enters a Place, approaches an NPC, spatially initiates a conversation, and interacts with one object, with all state authoritative on the server. |
| **AC-15** | **There is only one Alice** | A 2D client, a 3D client and an agent-driven Person are connected to **one running server** at the same time. Something a player does in the 2D client is visible in the 3D client, and an NPC carries the consequence forward: speak to Alice in 2D, then walk up to her in 3D, and Alice knows it happened. |

`AC-15` is the criterion that distinguishes this project from a game. `AC-13` proves the two
clients *ask* the same question; `AC-15` proves they are looking at **the same world** rather than
at two consistent copies of one. It is also the most convincing thing the MVP can show a person in
thirty seconds:

```text
2D client ───────┐
                 │
3D client ───────┼──►  one MineWorld Server
                 │
agent  ──────────┘
```

## 9.1 What `AC-15` must record, and the false success it exists to exclude

"Both clients showed the same thing" is **not** evidence. Two clients each holding their own
Alice, kept in step well enough to look identical, would pass that reading and would mean the
opposite of what this project claims. The evidence must therefore name identity, not appearance:

```text
same world instance id
same Alice EntityId
same authoritative event sequence
same persisted state revision
```

The claim being tested is that **there is only one Alice**.

## 9.2 How Alice remembers, in MVP-0

`AC-15` must not become a reason to build a cognition architecture first. The minimum that
satisfies it is a projection, not a memory system:

```text
objective event log
      ↓
a conversation-history projection for Alice
      ↓
her controller's context
```

Alice knowing *"Player A spoke to me three minutes ago about X"* is enough. Episodic memory,
summarized biography, retrieval and forgetting are the MVP-1 evolution of this path, and
`CORE_CONCEPTS.md` §5's distinction between world truth, biography and subjective memory is
already the design they grow into. Nothing here contradicts it; the projection is simply the
first and thinnest member of that family.

AC-11 and AC-12 are what make the rest of the criteria testable in CI
([`ENGINEERING_STANDARDS.md`](ENGINEERING_STANDARDS.md) §§16, 22–24).

---

# 10. Sample worlds are integration fixtures

`Social Café` and `Market Town` are not demos. They are the project's primary integration
fixtures, maintained runnable throughout development, and major architectural changes are
evaluated by actually running them (§21 of the standards).

---

# 11. Explicit non-goals

The MVP is not:

```text
The Sims
inZOI
GTA
a Minecraft replacement
a photorealistic game
a massively multiplayer world
a demonstration of clever NPC dialogue
```

It is:

> A minimal proof that independent people, places, items, and organizations can inhabit a
> persistent networked world whose possible interactions are determined entirely by composable
> systems.

---

# 12. Delivery sequence

Each entry is at least one PR under the Structured Coding workflow
([`../CLAUDE.md`](../CLAUDE.md) §3).

```text
1  Vision + architecture + MVP specifications        (done)
2  Entity / Component contracts
3  ActionIntent / Event contracts
4  System interface
5  World clock + scheduler
6  Persistence
7  First trivial system
8  Headless demo
```

Everything after step 8 — the CLI and the server binary, the 2D client, Social Café, Market Town,
the 3D walking world, and the `AC-1` transformation test — builds on a kernel that already runs,
persists, and replays without a renderer or a model.

The 3D demo comes after the kernel, but the **contracts it needs do not**: spatial position,
orientation, interaction range, and the distinction between a move intent and rendered movement
are designed while the contracts are being written, not retrofitted once a 3D client exists.
