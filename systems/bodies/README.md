# systems/bodies/

**`mineworld-bodies`** — people and things have bodies: nobody walks through a wall, a table, an
object or anybody else, and people can push, kick, throw and shove.

```text
section     body              a place file's floor and solids, or an item file's one loose object
owns        PlaceShape        a place's walls and furniture
            BodyShape         a loose object's box or ball
            LooseObjects      where the objects of a place lie
provides    kick, throw       an object within 800 mm (target-less; the object is in the payload)
            shove             a person within 1 000 mm, half a metre
emits       place-shaped, body-formed, object-placed, object-moved, person-shoved,
            and presence's arrived and stopped-short (for shove only)
resolves    arrivals          into a shaped place: walls stop, people are nudged, objects pushed
depends on  presence          (and reads mineworld_item::is_declared, nothing else)
```

Everything is decided on the server, on whole millimetres. Rapier sweeps, casts and flies, behind
one module (`src/rapier.rs`); the checks that keep the world consistent are integers. A place without
a `body:` section is untouched.

The rules are in [`docs/DECISIONS.md`](../../docs/DECISIONS.md) (`ARC-39` and its notes, `ARC-36`'s
note, `DEP-13`); the section in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1; the design in
`.structured-coding/plans/mvp0/step-11-bodies.md` §§17–18. Try it on
[`worlds/bodies-yard`](../../worlds/bodies-yard).
