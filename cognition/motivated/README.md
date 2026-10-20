# mineworld-motivated

A decision model for MineWorld that needs no language model. It scores what a person can do, and its
daily rhythm is calibrated from published time-use data. It is optional: `rule` stays the default.

To try it on Social Café, run this from the repository root:

```sh
mineworld run worlds/social-cafe --headless --days 1 \
  --decisions cognition/motivated/examples/social-cafe-all-motivated.yaml
```

In a world with the `needs` and `sleep` packs, a person's own hunger and tiredness weigh in: a tired person
scores going to sleep high at night, a hungry one scores eating and buying food high, and a tired one is
slower to start small talk. A sleeper decides nothing until it is time to leave for work. The curves are
parameters (`hungry`, `sated`, `tired`, `wakeful`; [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §5.1).

Status: the rhythm acceptance AC-DLb-4 is a recorded FAIL. With the default weights it speaks rarely, and
only in the evening. The design and the diagnosis are in
[`pr-dl-b-motivated.md`](../../.structured-coding/plans/mvp0/pr-dl-b-motivated.md) §15 (F-DLb-5).

The calibration data is Table A-3 of the **American Time Use Survey** (U.S. Bureau of Labor Statistics,
2025 annual averages). The table is in the public domain. Credit: *Source: U.S. Bureau of Labor
Statistics, American Time Use Survey.* See [`data/NOTICE`](data/NOTICE) for the details.
