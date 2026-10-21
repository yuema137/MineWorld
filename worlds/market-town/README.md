# `market-town`

Social Café, plus installed systems and configuration. This is `AC-1`'s demonstration: a materially
different world made from the same people and places, without editing the kernel, a controller or a
renderer ([`docs/MVP.md`](../../docs/MVP.md) §2, [`DECISIONS.md` `ARC-35`](../../docs/DECISIONS.md)).

It adds owning and giving things (`ARC-37`):
- `item` declares twenty kinds of things (`items/*.yaml`, each with an `item:` section naming its
  category and its display name, such as `Croissant`), and tells whoever is in a place the whole list
  of kinds and their names;
- `inventory` gives each person what they carry at the start (each person file's `holdings:`) and is
  the only system that changes it; a person carries at most six things;
- `item-transfer` lets people give each other things, one at a time, within reach.

And work, money, shops and eating (`ARC-38`):
- `economy` gives everyone a wallet (`economy:`), runs the café's, the store's and the station newsstand's shops
  for the three organizations in `organizations/`, and is the only system that moves money; the money is US
  dollars, counted in cents, as `configure/economy.yaml` declares;
- `employment` gives Alice a job at the café, Felix one at the store and Ivan one at the station
  newsstand (`job:`); being there during
  the shift is the work, paid by the hour and restocking the shop;
- `consumption` lets people eat the food and drink the drinks they carry.

It lives in San Diego: `calendar` gives it a date and a sun, and `weather` San Diego's real weather of
2015–2024, replayed day by day from NOAA's station record in `data/weather/` (see its `NOTICE`), both
configured in `configure/`.

Things happen in town (`happenings`, `ARC-88`): a market on the street every Saturday from 08:00 to 14:00,
a harvest festival in the park on 31 October, announced a week ahead, and a street musician on some days.
They come from the calendar and a seeded draw in `configure/happenings.yaml`; no language model is involved.
While the market runs, the corner store charges 10 % less. Economy decides that itself, from
`configure/economy.yaml`.

People get hungry and sleep (step-35): `sleep` lets them lie down in the apartments and wake by alarm,
when spoken to or when they act; `needs` keeps each person's hunger and tiredness, which meals and
sleep lower. Both are configured in `configure/`, every person file ends with `needs: {}`, and each food
and drink says how much one eats away of hunger.

Every file of `worlds/social-cafe` was copied unchanged — its walls, furniture and four loose objects
(`bodies`) included; the only differences are this README, the world's id and name, six appended
systems (and the five generic packs above, with their configuration, and economy's
configuration), the twenty kinds and the three
organizations with their files, and blocks appended at the end of each person file (`holdings:`,
`economy:`, `needs:`, and `job:` for three).

## Food and money

How the town was sized, so that an author of another world can repeat it (`pr-s35-nd-content.md` §2):

1. **Demand.** A person's hunger rises 120 an hour awake and 30 asleep: 16 × 120 + 8 × 30 = 2 160 a day.
   Twelve people need 25 920 points of nourishment a day.
2. **Supply.** Alice's and Felix's shifts make, each day, 8 croissants, 6 cakes, 10 sandwiches, 10
   soups, 24 apples, 18 loaves, 6 milks and 6 juices: 33 000 points, 127 % of demand, so a short shift
   or uneven buying still feeds everyone.
3. **Prices.** A person without a job starts with 200 000 minor units. To eat for 300 days they may spend
   about 600 a day, so food costs at most 0.28 a point: soup 160, sandwich 175, bread 130, cake 110,
   croissant 95, apple 55. A day of the dearest food per point costs 605.

The numbers are content in this world's files. No code reads them.

Run it headless — everybody walks, talks, gives, buys, eats and drinks, and three of them work:

```sh
mineworld validate worlds/market-town
mineworld run worlds/market-town --headless --seed 7 --days 30
```

The headless controller buys, eats and gives because each is offered as a complete request it can
submit unchanged; it was never taught what they are ([`ARC-34`](../../docs/DECISIONS.md)). People
without a job live on what they start with, sized for a 300-day run.

## The proof

That this world is Social Café plus installed packs and configuration is checked, not claimed:

```sh
cargo test -p mineworld-acceptance --test ac1_composability   # AC-1, three checks (needs full git history)
cargo test -p mineworld-cli --test market_town                # the market lives 300 days, then determinism
cargo test -p mineworld-cli --test market_composition         # each market pack can be removed
cargo test -p mineworld-cli --test milestone_c                # work, earn, buy, seen by another client, restart
```

"Installed" means a directory, two lines in `systems/installed` and a rebuild
([`ARC-33`](../../docs/DECISIONS.md)). To see Milestone C yourself, follow
[`HUMAN_REVIEW_QUEUE.md`](../../docs/HUMAN_REVIEW_QUEUE.md).
