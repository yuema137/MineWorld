# price-survey

A price notebook for one person. Give a person a `price-survey:` section and they become a
**surveyor**. Their **price book** fills with the prices they saw themselves: sales that happened in
front of them, and the asks on display where they took `survey`. Nothing else gets in.

The book is world state, shown only to its holder. So a deterministic price clerk, a language model, a
human player and an MCP agent driving that person all read the same book. This pack computes no
suggested price and publishes nothing; a decision model reads the book and does that.

## What a world writes

```yaml
# people/carol.yaml
price-survey:
  kinds: { categories: [food, drink] }   # required: 1 to 16 item categories
  depth: 32                              # optional: notes kept per kind, 1 to 64
  window: "7d"                           # optional: the oldest note kept, "1h" to "30d"
  resurvey_after: "1h"                   # optional: per place, "1m" to "1d"
```

The world must enable `economy`, `item` and `presence` too. A category that no item kind has is
refused at genesis (`survey-category-unknown`).

## What happens

- **Witnessing.** Carol stands in the store and Bob buys bread for 300. Her book gains a `sale` note:
  bread, the store's operator, 300, the store, the time. If Carol is out on the street, nothing changes.
  "Did she perceive it" is presence's own audience rule, the same one every inbox uses.
- **Surveying.** `survey` copies every tracked ask economy shows Carol where she stands: the shop's
  listing with its stock count, and each present person's asks with no count. It is offered ready to
  take, and a second survey of the same place within `resurvey_after` is refused `surveyed-recently`.
  Where nothing tracked is on display, it is not offered and is refused `nothing-to-survey`.
- **Bounds.** Each kind keeps at most `depth` notes, oldest dropped first. A note older than `window`
  leaves the book exactly when it ages out.
- **Privacy.** Only the holder sees the book (`survey-book`). Its facts, `survey-book-opened` and
  `price-noted`, reach nobody's inbox.

## Where the rules are

- The decision: [`docs/decisions/ARC-116.md`](../../docs/decisions/ARC-116.md).
- Every field: the generated [`docs/reference/`](../../docs/reference/).
- The design: [`pr-s31c-price-survey.md`](../../.structured-coding/plans/mvp0/pr-s31c-price-survey.md).
