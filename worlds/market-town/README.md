# `market-town`

Social Café, plus installed systems and configuration. This is `AC-1`'s demonstration: a materially
different world made from the same people and places, without editing the kernel, a controller or a
renderer ([`docs/MVP.md`](../../docs/MVP.md) §2, [`DECISIONS.md` `ARC-35`](../../docs/DECISIONS.md)).

It adds owning and giving things (`ARC-37`):
- `item` declares twenty kinds of things (`items/*.yaml`, each with an `item:` section);
- `inventory` gives each person what they carry at the start (each person file's `holdings:`) and is
  the only system that changes it; a person carries at most six things;
- `item-transfer` lets people give each other things, one at a time, within reach.

And work, money, shops and eating (`ARC-38`):
- `economy` gives everyone a wallet (`economy:`), runs the café's and the store's shops for the two
  organizations in `organizations/`, and is the only system that moves money;
- `employment` gives Alice a job at the café and Felix one at the store (`job:`); being there during
  the shift is the work, paid by the hour and restocking the shop;
- `consumption` lets people eat the food and drink the drinks they carry.

Every file of `worlds/social-cafe` was copied unchanged; the only differences are this README, the
world's id and name, six appended systems, the `items` and `organizations` lists with their files,
and blocks appended at the end of each person file (`holdings:`, `economy:`, and `job:` for two).

Run it headless — everybody walks, talks, gives, buys, eats and drinks, and two of them work:

```sh
mineworld validate worlds/market-town
mineworld run worlds/market-town --headless --seed 7 --days 30
```

The headless controller buys, eats and gives because each is offered as a complete request it can
submit unchanged; it was never taught what they are ([`ARC-34`](../../docs/DECISIONS.md)). People
without a job live on what they start with, sized for a 300-day run.
