# MineWorld

> **An open-source framework for persistent, modular, playable game worlds.**

You build a world by composing people, places and objects with **installable systems**, then
choosing your own rules, art style and AI. You don't fork a game and edit it. The café town in
this repository is **one default composition** of the framework, playable out of the box. It is
not the product.

```text
Simulation creates reality.      the server owns every rule and every fact
Controllers propose actions.     humans, scripted rules, or language models
Presentation observes reality.   2D, 3D, or anything else that can read a frame
```

## What MineWorld gives you, and what you bring

| MineWorld provides (the framework) | You provide (your game) |
| --- | --- |
| A kernel that knows only entities, components, time, places, actions, events and persistence. It does not know what money or a job is. | **Your world:** people, places, items and organizations as plain YAML, a *World Pack* |
| Deterministic, event-sourced simulation: every change traces to a cause, and a world survives a crash and replays byte for byte | **Which systems are on:** list them in your world's `systems:` |
| A server that owns all state; clients only send *intents* and render what they observe | **New systems:** a Rust crate in `systems/`, added with two lines in `systems/installed` |
| A library of ready-made **System Packs**: presence and movement, conversation, relationships, group activities, names, daily schedules, items, inventory, giving, money and shops, jobs, eating and drinking, bodies and physical interaction | **Your art style:** a *Presentation Pack* of assets and bindings. The 2D and 3D reference clients read it. |
| *Affordances*: the server tells each client what is possible right now, so clients and AI can use systems they have never seen | **Your interaction rules:** the "physics list" deciding how kinds of object interact. Its design is in progress, see below. |
| Reference 2D and 3D clients (Godot), plus a protocol module any client can use | **Your characters' minds:** rule-based controllers today; language-model controllers are in progress |

The rule that holds it together: **installing a system never edits the kernel, the other systems,
or the clients.** It is a tested property, not a promise. `tests/acceptance/tests/ac1_composability.rs`
proves that `worlds/market-town` is `worlds/social-cafe` plus six installed packs and configuration,
with no other change.

## Play the default demo

You need [Rust](https://rustup.rs) (the pinned toolchain installs itself) and
[Godot 4.7](https://godotengine.org) on your `PATH`.

```sh
git clone https://github.com/yuema137/MineWorld && cd MineWorld

# 3D: walk the town in first or third person (V switches camera, Shift runs, Space jumps)
./mineworld-slice

# 3D, connected to a live world: enter the café, walk to the counter, press E to talk to Alice
./mineworld-slice --world

# No graphics: run the market town for 30 simulated days and watch people live, work, buy and eat
cargo run -p mineworld-cli -- run worlds/market-town --headless --seed 7 --days 30
```

The connected 2D client, multiplayer with invites, and full collision in 3D are being built now.
[`docs/MVP_STATUS.md`](docs/MVP_STATUS.md) says exactly what works today.

## Make your own world

```sh
cargo run -p mineworld-cli -- create my-world      # a minimal World Pack to start from
cargo run -p mineworld-cli -- validate my-world    # what it contains, and anything wrong with it
cargo run -p mineworld-cli -- run my-world --headless --seed 1 --days 7
```

Then grow it:

- **Content.** Add people, places and items as YAML files, and turn systems on in `world.yaml`.
  `worlds/social-cafe` and `worlds/market-town` are worked examples.
- **A new system.** Read [`systems/README.md`](systems/README.md), section "Adding a pack". A pack
  owns its state, declares the actions it offers and the facts it records, and never writes
  another pack's state.
- **The formats.** See [`docs/MODULE_SPEC.md`](docs/MODULE_SPEC.md) and
  [`docs/PACKAGE_FORMAT.md`](docs/PACKAGE_FORMAT.md).

## Status

MVP-0 is in progress. Its main claim, that different games come from composing the same core with
different installable systems, is demonstrated and tested. Persistent social life and an everyday
economy are working and accepted. Work in progress:

- the 2D and 3D clients' full interaction;
- multiplayer with invites and reconnects;
- physical interaction in the towns;
- versioned third-party packs;
- language-model characters.

Details: [`docs/MVP_STATUS.md`](docs/MVP_STATUS.md) and [`docs/MVP.md`](docs/MVP.md).

## Where to look

- [`docs/VISION.md`](docs/VISION.md): why this exists
- [`docs/CORE_CONCEPTS.md`](docs/CORE_CONCEPTS.md): the vocabulary (Person, Place, System, Affordance…)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md): how the pieces fit
- [`CLAUDE.md`](CLAUDE.md) and [`docs/ENGINEERING_RULES.md`](docs/ENGINEERING_RULES.md): the rules
  code here must follow

## License

MIT, except the Blender scripts in `clients/3d-spike/tools/blender/`, which use Blender's GPL API
and are GPL-2.0-or-later. Asset provenance and AI-generated content are listed in
[`NOTICE`](NOTICE).
