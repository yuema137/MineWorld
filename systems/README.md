# systems/

**System Packs**: the modules that decide what is *allowed to happen* in a world. This is where
MineWorld's composability lives — a world is its systems, and installing or disabling one changes
what the world is without changing anything else.

```text
presence/       where people are, what each of them perceives, and what they may attempt
conversation/   speaking to somebody, and remembering that they spoke to you
```

Both are real packs, not examples: they are what the vertical slice runs on. They are also the
worked example every later pack copies, so they are written to be read in this order —
`src/lib.rs` first, then the action, the event, the component, and `src/system.rs` last.

## The four rules a pack lives by

1. **It writes only the components it owns.** Anything cross-domain travels as an event, and the
   owner decides what it means. The compiler enforces this, not a reviewer.
2. **It never assumes a system it has not declared as a dependency.** `conversation` declares
   `presence`, because space is presence's state; a world composed without it is refused by name.
3. **It names no renderer, transport or model provider.** A pack does not know it is being drawn.
4. **Removing it removes its actions and its state contribution**, and nothing else — no edit to
   `Person`, to another pack, or to the kernel.

## The one thing worth understanding before adding a pack

A client is told what a player may attempt, with the server's verdict for each, so that no
renderer ever decides whether an interaction is valid. Perception produces those answers **without
knowing what any action is**: a pack implements `PerceptionProvider` and offers its own actions
with the spatial requirement it declared, and `presence` prices each offer through the kernel's
route map and the contract layer's one evaluator.

The same trait carries the other half — which of a pack's **components** an observer may know about
a given entity. The owning pack answers, per observer, so state reaches an observation because
somebody named it rather than because it exists.

A pack that would need an edit to `systems/presence/` in order to be playable has not been written
correctly. That is the difference between a framework and a hardcoded game, and
[`presence/src/interaction.rs`](presence/src/interaction.rs) is where it is documented in full.

```sh
cargo test -p mineworld-presence
cargo test -p mineworld-conversation      # the two packs in one world, and AC-2
cargo doc -p mineworld-conversation --open
```

What a System Pack must declare is specified in
[`../docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §3; the interaction and spatial rules are
[`../docs/ENGINEERING_RULES.md`](../docs/ENGINEERING_RULES.md) §§4–9.
