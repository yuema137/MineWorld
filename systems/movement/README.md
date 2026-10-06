# systems/movement/

**`mineworld-movement`** — whether a person may walk where they ask, and which places open onto
which.

```text
owns        Passages          which places a place opens onto, and where the doorway is
provides    move              a stride within a place, or through a doorway into the next
emits       passage-opened    the genesis fact that gives places their passages
            arrived           presence's fact, built by presence's own constructor
depends on  presence          which owns where people are
```

The server decides every move, against where the world says the person is. Within a place a move
may carry a person at most **2 m** (`MAX_STRIDE`); into another place only through a doorway, within
2 m of it on both sides. Anything else is refused `too_far_away`. A client walks its own body and
reports `move` before it has gone 2 m since its last accepted position.

This pack decides; presence records. Disabling it removes `move` and changes nothing else.

```sh
cargo test -p mineworld-movement
cargo doc -p mineworld-movement --open      # the crate docs are the reference
```

Read [`src/lib.rs`](src/lib.rs) first, then [`src/system.rs`](src/system.rs). The decision is
[`DECISIONS.md` `ARC-26`](../../docs/DECISIONS.md); the wire rule is
[`server/PROTOCOL.md`](../../server/PROTOCOL.md) §6.2.
