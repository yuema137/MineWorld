# systems/

**System Packs**: the modules that decide what is *allowed to happen* in a world. This is where
MineWorld's composability lives — a world is its systems, and installing or disabling one changes
what the world is without changing anything else.

```text
presence/        where people are, what each of them perceives, and what they may attempt
movement/        walking: whether a `move` is allowed, and which places open onto each other
conversation/    speaking to somebody, and remembering that they spoke to you
group-activity/  inviting, answering, joining and leaving something done together (a Process)
relationships/   who knows whom, and how well — changed only by reacting to the others' facts
naming/          what people are called, from a person file's `name:` section
schedule/        a person's day, from their `routine:` section — an agenda kept by a Process
item/            what kinds of things exist, from an item file's `item:` section
inventory/       who holds how many of each kind — the only writer of holdings, with a person's capacity
item-transfer/   `give`: offered complete for each kind held; states inventory's fact, owns nothing
employment/      jobs from a person's `job:` section; a shift is a Process, work is being there; wage-due
economy/         wallets and shops from the `economy:` section, `buy`, and wages — the only mover of money
consumption/     `eat` food and `drink` drinks a person carries; states inventory's fact, owns nothing
```

All thirteen are real packs, not examples. The first seven are what the vertical slice runs on; the
last six are Market Town ([`DECISIONS.md` `ARC-37`, `ARC-38`](../docs/DECISIONS.md)). The first
three are also the
worked example every later pack copies, so they are written to be read in this order —
`src/lib.rs` first, then the action, the event, the component, and `src/system.rs` last.

## The four rules a pack lives by

1. **It writes only the components it owns.** Anything cross-domain travels as an event, and the
   owner decides what it means. The compiler enforces this, not a reviewer.
2. **It never assumes a system it has not declared as a dependency.** `conversation` declares
   `presence`, because space is presence's state; a world composed without it is refused by name.
   A pack may state a fact in another pack's vocabulary only if it depends on that pack — `movement`
   states presence's `arrived`, and presence still decides what its state may hold
   ([`DECISIONS.md` `ARC-26`](../docs/DECISIONS.md)); the kernel refuses anything else at install.
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

## Adding a pack

A pack is a crate in its own directory here. It implements `System`, `PerceptionProvider` and
`mineworld_sdk::SystemPack` — the last says, once, what the build needs to know about it (see
[`../sdk/rust/README.md`](../sdk/rust/README.md)), and its first line is always its package identity,
`const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();` (`ARC-53`). It depends on a
sibling pack by path: `mineworld-presence = { path = "../presence" }`.

Then install it into the build with two lines in [`installed/`](installed/), and rebuild:

```text
installed/Cargo.toml    mineworld-<name> = { path = "../<name>" }
installed/src/lib.rs    <Variant> => mineworld_<name>::<System>,
```

Nothing else is edited — not the root `Cargo.toml`, not the World Pack loader. A world uses the pack
by naming it in its `systems:` list. Installing always means a rebuild in MVP-0; the full rules are
[`../docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §3.1 and
[`DECISIONS.md` `ARC-33`](../docs/DECISIONS.md).

A pack that changes what an arrival achieves — where a person ends up, or who else is moved — does
not edit `movement` or `presence`. It implements presence's `ArrivalResolver`, is listed on
presence's `extension` line in the installed set as well as on its own line, and calls
`require_registered` in `install` ([`../docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §3.1,
[`DECISIONS.md` `ARC-39`, `ARC-62`](../docs/DECISIONS.md)). Any pack can open such a catalog for a
trait it owns with one more `extension` line; a pack can also take a world-level configuration file
(`configure/<id>.yaml`, `ARC-61`).

A pack whose numbers or rules a world may choose has a **section** of the World's Interaction List
(`configure/<id>.yaml`, `ARC-63`): it implements `mineworld_sdk::interactions::InteractionSection`
(its parameters with their bounds and defaults, its actions and facts with their roles), writes
`mineworld_sdk::interactions!();` in its `impl SystemPack`, and reads the answers with `permits`,
`parameters` and `consequence`. Its README documents its section. The schema is
[`../docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §4.2.

```sh
cargo test -p mineworld-presence
cargo test -p mineworld-movement          # walking, TooFarAway, and AC-2 with its negative control
cargo test -p mineworld-conversation      # talking where people are, and AC-2
cargo test -p mineworld-group-activity    # invitations, and an activity that is a Process
cargo test -p mineworld-relationships     # values changed only by other packs' facts
cargo test -p mineworld-naming            # a name, seeded from a pack file, disclosed to perceivers
cargo test -p mineworld-schedule          # a day kept by a Process that moves nobody
cargo test -p mineworld-item              # a kind declared from an item file's section
cargo test -p mineworld-inventory         # holdings, refused facts, capacity, a restart
cargo test -p mineworld-item-transfer     # give through the unchanged controller, and AC-2
cargo test -p mineworld-employment        # shifts paid for the time present, and a restart mid-shift
cargo test -p mineworld-economy           # buy through the unchanged controller, wages, AC-2 both ways
cargo test -p mineworld-consumption       # eat food, drink drinks, never goods, and AC-2
cargo doc -p mineworld-conversation --open
```

What a System Pack must declare is specified in
[`../docs/MODULE_SPEC.md`](../docs/MODULE_SPEC.md) §3; the interaction and spatial rules are
[`../docs/ENGINEERING_RULES.md`](../docs/ENGINEERING_RULES.md) §§4–9.
