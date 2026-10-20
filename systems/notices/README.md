# notices

Public notices that expire. A world declares **topics**. Named people post notices on them. Everyone
who subscribes to a topic gets each notice in their inbox, and sees it on their board until it expires.

A price clerk posting suggested prices, a newsstand posting the day's headline, a market posting its
opening hours: each is a notice on a topic. The pack does not know what a price or a headline means. It
knows who may post, who listens, and how long a notice stays valid.

## What a world writes

```yaml
# configure/notices.yaml
topics:
  prices.ingredients:
    body: prices                 # prices (item and amount lines) or text (one line)
    publishers: [carol]          # who may post on it
    valid_for: { min: "1h", max: "48h" }
    per_day: 4                   # at most 4 posts by one publisher in any 24 hours
defaults:
  by_tag: { barista: [prices.ingredients] }   # who subscribes when their file says nothing
```

```yaml
# people/alice.yaml
notices: { subscribe: [prices.ingredients] }  # replaces the defaults; [] means none
```

## What happens

- `publish-notice { topic, body, valid_for }`: a listed publisher posts. The notice reaches exactly the
  topic's subscribers at that moment, and the publisher.
- `subscribe { topic }` and `unsubscribe { topic }`: a person's own choice. Nobody else sees it.
- The **board** shows a person the notices that are current now on the topics they subscribe to or
  publish. A notice leaves the board at its end time, with nothing recorded. The inbox and memory keep it.

Every action is offered without a ready-made payload, so a generic NPC never posts or subscribes at
random. A refusal is always named: `unknown-topic`, `body-mismatch`, `body-too-large`, `body-invalid`,
`validity-out-of-range`, `over-daily-limit`, `already-subscribed`, `not-subscribed`, or
`PermissionDenied` for someone who may not post.

## Where the rules are

- The decision: [`docs/decisions/ARC-115.md`](../../docs/decisions/ARC-115.md).
- Every field: the generated [`docs/reference/`](../../docs/reference/).
- The design: [`pr-s31b-notices.md`](../../.structured-coding/plans/mvp0/pr-s31b-notices.md).
