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

`PacedRuleController` is what `mineworld run` puts in every seat. It is consulted every fifteen simulated
minutes and takes initiative: it answers what was said to it since it was last asked, and otherwise —
by a seeded draw — greets somebody, walks toward somebody, wanders, or heads for a door. Which door is
a draw it keeps for six simulated hours, and on a street, which many places open onto, it mostly walks
on.

It also does things with other people. An open invitation is answered first, mostly with yes. When it
is part of nothing, it sometimes invites somebody or joins what somebody nearby is doing. When it is
part of an activity, it sometimes leaves, and it never walks out of the door. Every one of these
choices is limited to what the server's affordances say it may do (`src/social.rs`). It keeps
no memory at all (`decide(&self)`), so the same seed always makes the same choices, and a run that is
killed and restarted continues exactly as if it had not been. The decision is
[`DECISIONS.md` `ARC-27`](../../docs/DECISIONS.md).

It keeps its day. When its agenda says to be somewhere else, it mostly heads there, through the door
that leads there. When it is where the agenda says, it does not walk out (`src/agenda.rs`;
[`ARC-32`](../../docs/DECISIONS.md)). Being spoken to still comes first. With no agenda in the
observation, it decides exactly as it did before agendas existed.

Both controllers name people by the name the world discloses, so Alice says "Earlier, Vera Lindgren
said …". When the observation discloses no name, she says "someone else", never an id.

```sh
cargo test -p mineworld-rule-controller
```
