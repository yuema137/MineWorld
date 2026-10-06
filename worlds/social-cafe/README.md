# `social-cafe`

The world the MVP-0 vertical slice is set in: a café and the street outside it, four people, and
the three systems that make being there mean anything — `presence`, `movement`, `conversation`.

```text
world.yaml          what this world is, what it enables, who is in it, and which seats may be occupied
places/cafe.yaml    the room, and its front door onto the street
places/street.yaml  the pavement outside
people/*.yaml       Alice behind the counter, Bob at a table, and two visitors by the door
```

Run it, with Alice driven by a rule controller:

```sh
mineworld server worlds/social-cafe --agent alice
```

Then connect two clients, on the `visitor` and `wanderer` seats. Speak to Alice from one and walk up
to her from the other, and she knows it happened — which is `AC-15`, the criterion this world exists
to demonstrate ([`docs/MVP.md`](../../docs/MVP.md) §9). Walking is `move`, in strides of at most
2 m that the server accepts or refuses `too_far_away`; walk through the front door and the world
records that you entered the street ([`server/PROTOCOL.md`](../../server/PROTOCOL.md) §6.2).

It is content, not code: no rule of the simulation is written here, and none can be. What a pack may
say is specified in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.
