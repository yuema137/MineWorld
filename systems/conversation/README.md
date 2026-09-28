# systems/conversation/

**`mineworld-conversation`** — speaking to somebody, and remembering that they spoke to you.

```text
owns        ConversationHistory   who spoke to this person, when, and about what
provides    talk                  one person speaks to another, in reach of being heard
emits       conversation-started  the first exchange after a silence
            spoke                 every exchange
subscribes  spoke                 so the history above is a projection of the event log
depends on  presence              because space is that pack's state, not this one's
```

## Alice's memory is a projection, and must stay one

`ConversationHistory` is the whole of what a person remembers in MVP-0: a bounded list, written in
one place only — while reducing this pack's own `spoke` fact. That is enough for *"somebody spoke to
me three minutes ago about X"*, which is exactly what [`../../docs/MVP.md`](../../docs/MVP.md) §9.2
asks for, and it is deliberately not a cognition architecture.

Because it is a reduction of the log rather than a cache kept alongside it, a world replayed from
its events remembers the same conversations. Episodic memory, summarization and forgetting are a
later step built on that property; they are not a larger number in this file.

A controller reads it through the observation it is given, never from the world: this pack discloses
a person's history **in that person's own observation and nowhere else**. So Alice's controller
learns that somebody spoke to her, a player learns what Alice said to them out of their own history,
and neither can read a stranger's memory — because there is no request that asks for one.

## Distance is checked, and not by arithmetic written here

`talk` declares a `SpatialRequirement` — the same place, within three metres, of somebody available
— and the contract layer's single `SpatialRequirement::evaluate` is what decides it. There is no
distance comparison anywhere in this crate, which is what makes the affordance a client is shown and
the answer the server gives the *same* answer. A test pins that agreement; mutating either path
turns it red.

The positions come from `Presence`, which this pack reads and can never write. Hence the declared
dependency: a world with conversation and no presence is refused when it is composed, naming what is
missing, rather than failing later as a `talk` that cannot find anybody.

```sh
cargo test -p mineworld-conversation        # the two packs in one world, and AC-2
cargo doc -p mineworld-conversation --open  # the crate docs are the reference
```

[`tests/conversation_and_presence.rs`](tests/conversation_and_presence.rs) is the integration
checkpoint: a talk accepted and remembered, the same talk refused `TooFarAway` with nothing written,
and two worlds that differ by one boolean — in one of which there is no talking at all, and which is
not broken.
