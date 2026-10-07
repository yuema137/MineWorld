# `social-cafe`

A small town around a café, and the world the MVP-0 vertical slice is set in. It has five places
joined by a street, twelve people, and the three systems that make being there mean anything:
`presence`, `movement` and `conversation`.

```text
world.yaml           what this world is, what it enables, who is in it, and which seats may be occupied
places/cafe.yaml     the café, laid out as the 3D slice draws it (reference 03): the door at the west end
                     of the shopfront, the counter across the back of the room
places/street.yaml   the street every other place opens onto
places/*.yaml        the apartments, the park, the convenience store and the office: one door each
people/*.yaml        Alice behind the counter, Bob at it, two visitors, and the town's other eight people
```

Every Person but Otto is a seat, so a client, an agent or a headless rule controller can drive any of
them. The café's interior follows the 3D slice's own geometry, so a person standing at the counter
there can talk to Alice behind it ([`step-09-social.md`](../../.structured-coding/plans/mvp0/step-09-social.md)
§2.5).

Host it, with Alice driven by a rule controller:

```sh
mineworld server worlds/social-cafe --agent alice
```

Then connect two clients, on the `visitor` and `wanderer` seats. Speak to Alice from one and walk up
to her from the other, and she knows it happened. That is `AC-15`, the criterion this world exists to
demonstrate ([`docs/MVP.md`](../../docs/MVP.md) §9). Walking is `move`, in strides of at most 2 m that
the server accepts or refuses `too_far_away`. Walk through a door and the world records that you
entered the next place ([`server/PROTOCOL.md`](../../server/PROTOCOL.md) §6.2).

Or run the whole town headless, every seat driven by a seeded rule:

```sh
mineworld run worlds/social-cafe --headless --seed 7 --days 300
```

It is content, not code: no rule of the simulation is written here, and none can be. What a pack may
say is specified in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.
