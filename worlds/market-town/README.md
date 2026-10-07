# `market-town`

Social Café, plus installed systems and configuration. This is `AC-1`'s demonstration: a materially
different world made from the same people and places, without editing the kernel, a controller or a
renderer ([`docs/MVP.md`](../../docs/MVP.md) §2, [`DECISIONS.md` `ARC-35`](../../docs/DECISIONS.md)).

So far it adds owning and giving things (`ARC-37`):
- `item` declares twenty kinds of things (`items/*.yaml`, each with an `item:` section);
- `inventory` gives each person what they carry at the start (each person file's `holdings:`) and is
  the only system that changes it; a person carries at most six things;
- `item-transfer` lets people give each other things, one at a time, within reach.

Work, money and shops arrive next (step-10 PR 11e).

Every file of `worlds/social-cafe` was copied unchanged; the only differences are this README, the
world's id and name, three appended systems, the `items` list and `items/`, and one `holdings:` block at
the end of each person file.

Run it headless — everybody walks, talks, and now gives:

```sh
mineworld validate worlds/market-town
mineworld run worlds/market-town --headless --seed 7 --days 30
```

Nothing is used up yet, so things only change hands. The headless controller gives because each `give`
is offered as a complete request it can submit unchanged; it was never taught what `give` is
([`ARC-34`](../../docs/DECISIONS.md)).
