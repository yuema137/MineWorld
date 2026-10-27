# mineworld-price-reporter

The price clerk's deterministic brain. A clerk is an ordinary Person in a seat; give that seat the model
`price-reporter` and it:

1. takes the `survey` the world offers wherever asks are on display, so its price book fills;
2. at a few times of day (08:00, 12:00, 16:00, 20:00 by default) posts one `prices` notice of suggested
   prices, computed from its own book;
3. otherwise lives exactly as the `rule` controller does.

```yaml
# decisions.yaml
models:
  price-reporter: { parameters: { topic: prices.ingredients, first_minute: 480, last_minute: 1200, every_minutes: 240 } }
assign:
  - { seat: carol, model: price-reporter }
```

To drive the same clerk with a language model instead, assign `carol` to `lm` with `fallback: price-reporter`
and configure `[fill.publish-notice]` in `cognition.toml`. Nothing in the world changes.

The rules and every parameter: [`docs/MODULE_SPEC.md`](../../docs/MODULE_SPEC.md) §5.1;
[`ARC-116`](../../docs/decisions/ARC-116.md), [`ARC-117`](../../docs/decisions/ARC-117.md),
[`ARC-183`](../../docs/decisions/ARC-183.md).
