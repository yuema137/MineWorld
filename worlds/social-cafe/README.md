# `social-cafe`

The world the MVP-0 vertical slice is set in: one café, four people, and the two systems that make
being there mean anything.

```text
world.yaml        what this world is, what it enables, who is in it, and which seats may be occupied
places/cafe.yaml  the one room
people/*.yaml     Alice behind the counter, Bob at a table, and two visitors by the door
```

Run it, with Alice driven by a rule controller:

```sh
mineworld server worlds/social-cafe --agent alice
```

Then connect two clients, on the `visitor` and `wanderer` seats. Speak to Alice from one and walk up
to her from the other, and she knows it happened — which is `AC-15`, the criterion this world exists
to demonstrate ([`docs/MVP.md`](../../docs/MVP.md) §9).

It is content, not code: no rule of the simulation is written here, and none can be. What a pack may
say is specified in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.
