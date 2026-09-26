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

## Status

Specification only — no kernel code yet. The next three PRs define the contracts.

The one criterion the first milestone must meet:

> Materially different games can be built by composing independently installable systems,
> without modifying the kernel.

## Where to look

- [`docs/`](docs/) — the specifications, starting with [`docs/VISION.md`](docs/VISION.md)
- [`docs/MVP.md`](docs/MVP.md) — what the first milestone must prove
- [`CLAUDE.md`](CLAUDE.md) — how development runs here, and the rules code must follow

## License

MIT
