## DEP-9 — The frozen technology boundary

**Date** 2026-09-26 · **Consolidates** `DEP-2`, `DEP-3`, `DEP-4`, `DEP-5`, `DEP-6`, `DEP-7`,
`ARC-8`

One table, so the question stops being reopened per subsystem:

| Layer | Choice |
| --- | --- |
| Core runtime, world state, systems, scheduler, persistence, networking, contracts | **Rust** |
| Reference rendering, 2D and 3D | **Godot** (MIT) |
| 3D renderer default | **Godot Forward+** |
| GPU, physics, animation, camera, input, navigation, audio, platform export | **Godot's** — MineWorld writes none of it |
| Canonical 3D asset format | **glTF / GLB** (`DEP-7`) |
| Cognition and ML | **Python** or an external service where useful |
| Long-term system and plugin extension | **WASM Component + WIT** (`ARC-8`) |
| Persistence | **SQLite** locally, Postgres later (`DEP-2`) |
| Transport | **tokio + axum**, HTTP and WebSocket (`DEP-3`) |

### Why Godot rather than Unreal or Unity

Not because it renders best. Because the licence matches the project: MIT, forkable, patchable,
no vendor whose terms can change under an open-source framework, and 2D and 3D in **one** engine
— which matters more here than anywhere, since MineWorld's claim is that both clients render one
semantic world. Its Forward+ renderer already provides the GI, shadows, volumetric fog and
tonemapping a warm semi-realistic lakeside town needs; the target was never cinematic fidelity.

Unreal is stronger visually and stays welcome as an optional high-end renderer
(`renderers/unreal/`), but it is proprietary, carries a revenue royalty, and its build system
would absorb exactly the engineering attention this project cannot spare. Unity's licensing
controversy has eased, yet it offers no advantage over Godot for a project whose contributors
must be able to fork the engine.

**A Rust engine such as Bevy was considered and rejected**, despite the appeal of an all-Rust
stack. It would hand us renderer maturity, asset pipeline, animation tooling, character
controllers and editor workflow as *our* problems — precisely the work this decision exists to
avoid. Technical purity is not the objective; shortest path to an extensible framework is.

```text
Rust  → the world           Godot → the pixels
```

### What this buys

Engineering attention concentrates on the differentiating problem — **how an LM generates a
modular, verifiable, composable world that runs for a long time** — instead of on making a
street lamp three percent faster on the GPU.
