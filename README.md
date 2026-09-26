# MineWorld

> **An open-source framework for building persistent, modular living game worlds.**

Worlds are composed from independent entities, simulation systems, controllers, and
presentation layers — not implemented as monolithic games.

```text
Simulation creates reality.
Controllers propose actions.
Presentation observes reality.
```

## The idea

You install modules and compose a world, instead of forking a game and editing it.

```text
Entity Pack        what exists
System Pack        what is allowed to happen between the things that exist
World Pack         a specific world: people, places, organizations, initial state
Controller Pack    who decides what a character does — human, rules, LM, RL
Presentation Pack  how it is rendered — pixel 2D, 3D, photorealistic, text
```

`Person` never changes. Enabling `Conversation` makes talking possible; adding `Inventory` and
`Economy` turns the same town into a market; adding `Crafting` and `Survival` turns it into a
survival game. The kernel never learns what a job or a coffee is.

A world with every language model unplugged is still a MineWorld world. LM cognition is one
kind of controller, not the point.

## Worlds you walk around in

The point is a world you are *inside*: you move through space, explore, enter places, meet
people, interact, and travel — and it keeps living after you log out. Two reference clients are
maintained side by side, a lightweight 2D one for fast validation and an embodied first-person 3D
one for the real experience. Clicking an NPC in 2D and walking up to them in 3D produce the same
action; neither client decides whether it is allowed.

## Status

Specification only — no kernel code yet. The next three PRs define the contracts.

The one criterion the first milestone must meet:

> Materially different games can be built by composing independently installable systems,
> without modifying the kernel.

## Where to look

- [`docs/`](docs/) — the specifications, starting with [`docs/VISION.md`](docs/VISION.md)
- [`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md) — read before touching production code
- [`docs/MVP.md`](docs/MVP.md) — what the first milestone must prove
- [`CLAUDE.md`](CLAUDE.md) — how development runs here, and the rules code must follow

## License

MIT
