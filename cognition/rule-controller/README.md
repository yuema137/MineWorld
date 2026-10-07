# cognition/rule-controller/

**`mineworld-rule-controller`** — a Person whose actions are decided by a rule, with no model.

```text
Observation  ─►  decide()  ─►  ActionRequest
```

That is the whole interface. It reads the observation it is given, and returns at most one request
per observation. It has no handle to the world, no clock, no network and no model — and it could not
acquire one, because `decide` takes an `Observation` and returns an `ActionRequest`.

## What it is for

`AC-15` needs three participants against one server: a 2D client, a 3D client, and a Person driven by
an agent. This is the third. The point is not the rule's cleverness — it is that the world cannot
tell the difference: the same seat, the same actor check, the same server-allocated identity as a
human client's request (`INV-1`).

## The rule

Answer the newest thing each person has said to me, once, when the server says I may — and say
something that proves I remember the rest.

The memory is not this crate's. It is the conversation pack's `ConversationHistory`, projected off the
event log and disclosed in this observer's own observation
([`../../docs/MVP.md`](../../docs/MVP.md) §9.2). Whether Alice may speak to somebody is the server's
answer, read out of an affordance. Nothing here measures a distance.

## The paced rule

`PacedRuleController` is what `mineworld run` puts in every seat. It is consulted every ten simulated
minutes and takes initiative: it answers what was said to it since it was last asked, and otherwise —
by a seeded draw — greets somebody, walks toward somebody, wanders, or heads out of the door. It keeps
no memory at all (`decide(&self)`), so the same seed always makes the same choices, and a run that is
killed and restarted continues exactly as if it had not been. The decision is
[`DECISIONS.md` `ARC-27`](../../docs/DECISIONS.md).

```sh
cargo test -p mineworld-rule-controller
```
