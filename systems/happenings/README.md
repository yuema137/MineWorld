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

Enabled without that file, the pack does nothing.

## Specification

The design, its facts and its rules are in
[`pr-we-a-happenings.md`](../../.structured-coding/plans/mvp0/pr-we-a-happenings.md) and
[`docs/DECISIONS.md`](../../docs/DECISIONS.md) `ARC-88`.
