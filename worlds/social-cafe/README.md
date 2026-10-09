# `social-cafe`

A small town around a café, and the world the MVP-0 vertical slice is set in. It has five places
joined by a street, twelve people, four loose objects, and eight systems:
- `presence`, `movement` and `conversation` make being there mean anything;
- `group-activity` lets people do things together;
- `relationships` lets them come to know each other;
- `naming` gives everybody a name (each person file's `name:`);
- `schedule` gives everybody a day (each person file's `routine:`) that they may follow;
- `bodies` gives every place walls and furniture (each place file's `body:`) and puts a box and a ball
  in the café and on the pavement outside it (`items/`). Nobody walks through a wall, a counter or
  somebody else, and leaving a room is through its door. People may `kick`, `throw` and `shove`.

Every part of a day lasts at least four hours. People walk slowly in a headless run, and a shorter
part of the day would be over before anybody got there. No part of anybody's day begins between
00:00 and 05:00, and the restart tests check that (`people/alice.yaml` explains why).

Run it headless, then read Alice's life out of the log:

```sh
mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save cafe-save
mineworld biography worlds/social-cafe --save cafe-save --person alice
```

```text
world.yaml           what this world is, what it enables, who is in it, and which seats may be occupied
places/cafe.yaml     the café, laid out as the 3D slice draws it (reference 03): the door at the west end
                     of the shopfront, the counter across the back of the room, its walls and tables
places/street.yaml   the street every other place opens onto, with the slice's lamps, trees and tables
places/*.yaml        the apartments, the park, the convenience store (the slice's Flower Room) and the
                     office: one door each, onto a door the 3D slice draws
items/*.yaml         four loose objects: a box and a ball in the café, a crate and a ball outside it
people/*.yaml        Alice behind the counter, Bob at it, two visitors, and the town's other eight people,
                     each with a name and a day
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
the server accepts or refuses `too_far_away`; a stride into a wall or a table stops short at it. Walk
through a door and the world records that you entered the next place
([`server/PROTOCOL.md`](../../server/PROTOCOL.md) §6.2).

Or run the whole town headless, every seat driven by a seeded rule:

```sh
mineworld run worlds/social-cafe --headless --seed 7 --days 300
```

It is content, not code: no rule of the simulation is written here, and none can be. What a pack may
say is specified in [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.
