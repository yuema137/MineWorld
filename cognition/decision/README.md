# mineworld-decision

How each seat decides, as a choice a world makes rather than code it forks.

- `DecisionModel` is the trait every in-process decision model implements: a pure function of its
  parameters, the run's seed, the pace and what the seat observes.
- `Draw` is the seeded randomness every model shares.
- `decisions.yaml` is the decision list: which model drives each seat, with typed parameters.

Write a model, add one line to [`cognition/installed`](../installed), rebuild, and name it in a world's
`decisions.yaml`.

The specification is [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §4.1 and §5.1.
