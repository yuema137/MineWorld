# happenings

A Saturday market on the street, a festival in the park announced a week ahead, a street musician on
one day in seven: **happenings** make a MineWorld town change beyond its people and its weather. They
come from the world's calendar and a seeded integer draw. No language model is involved.

## Turn it on

Enable `calendar` and `happenings` in `world.yaml`, list `happenings` under `configure:`, and write
`configure/happenings.yaml`:

```yaml
seed: 7
kinds:
  - { id: market-day, title: Saturday market, places: [street], hours: "08:00-14:00" }
calendar:
  - { kind: market-day, place: street, weekdays: [saturday] }
```

Enabled without that file, the pack does nothing. [`worlds/market-town`](../../worlds/market-town)
has a full example.

## Make other things follow

`happenings` only announces, begins and ends happenings. Another pack decides what they change, in its
own configuration. For example, `configure/economy.yaml` lowers a shop's prices while one runs:

```yaml
price_modifiers:
  - { during: market-day, shop: store, permille: 900 }
```

To decide *when* things happen in code, write a System Pack that implements `HappeningGenerator`,
add it to the installed set's `happenings` extension line, and list its id under `generators:`.

## Specification

The design, its facts and its rules are in
[`pr-we-a-happenings.md`](../../.structured-coding/plans/mvp0/pr-we-a-happenings.md) and
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-88`.
