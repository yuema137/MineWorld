# MineWorld MVP-0 — Composable Worlds

> **MVP success is not measured by how intelligent an NPC appears. It is measured by whether
> materially different games can be constructed by composing independently installable systems
> without modifying the kernel.**

**Status:** frozen MVP definition for v0.1 planning
**Audience:** coding agents. Vocabulary: [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md). Layering:
[`ARCHITECTURE.md`](ARCHITECTURE.md). Packaging: [`MODULE_SPEC.md`](MODULE_SPEC.md).

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

Official MVP client: a **simple Godot 2D top-down renderer**. It is deliberately not the final
visual target. Its only job is to prove:

```text
renderer ≠ world
```

The world must also run completely headless, so the MVP already ships two valid presentation
modes:

```text
headless
Godot 2D
```

A 3D renderer comes only after kernel contracts stabilize.

---

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

Everything after step 8 — the Godot 2D client, the LM controller, Social Café, Market Town,
and the AC-1 transformation test — builds on a kernel that already runs, persists, and replays
without a renderer or a model.
