## ARC-28 — Relationships are a `knows` edge with values its owner reduces from other systems' facts

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§4.4, 9, 13.1 ·
**Relates to** `INV-7`, `INV-13`, `DD-6`, `ARC-25`, `ARC-26`, [`MVP.md`](MVP.md) §9 `AC-2` ·
**Design** `.structured-coding/plans/mvp0/step-09-social.md` SD-5 … SD-9, §10.1 Q5/Q6, §4.2 (S8,
PR 10b)

**Problem.** S8 adds the first System Pack whose state changes *only* because of what other packs
did: people get to know each other because they spoke, accepted an invitation, or spent an hour
together. Four questions follow:
1. Where per-relation state lives, when the specification says "a component keyed by the triple"
   (`contracts/src/relation.rs`, `DD-6`) and the kernel keys components by `EntityId` alone.
2. Whether the pack must depend on the packs whose facts it reads.
3. How it reads their payloads.
4. Which changes are worth a fact of their own.

**Options considered.**

```text
state     (a) a kernel change: components keyed by a Relation          kernel learns a new key kind
          (b) a component on the `from` Person, keyed by the counterpart  chosen — same identity, no
              under the one relation type this pack declares               kernel change
dependency (a) depend on conversation and group-activity                 disabling either is refused
                                                                         while relationships is on
          (b) no system dependency; subscribe                             chosen
decoding  (a) a local struct shaped like the other pack's payload        drifts silently; bypasses the
                                                                         schema-version refusals
          (b) the owner crate's published event type, through            chosen
              EventRecord::payload_for
facts     (a) one per value change (~+43 000 per 300 days)               doubles the log, names nothing
          (b) one when a derived level crosses a boundary                 chosen (Q5)
```

**Choice.**
1. `RelationshipsSystem` declares a directed relation type `knows` (Person → Person, no self edges)
   and owns `Acquaintances` on the `from` Person: counterpart → `RelationshipValues { familiarity
   0..=1000, regard −1000..=1000, exchanges, activities_shared, first_met, last_contact }`. The edge
   and the entry are written together, in one reduction, by the one owner. That realizes `DD-6`'s
   "keyed by the triple" without a kernel change.
2. **No system dependency.** Subscribing is not emitting. `ARC-26` requires a dependency only to
   *state* another system's vocabulary. A world with relationships and no conversation installs; it
   simply hears no speech. `ARC-26` also calls the owner "the only system that reduces it into owned
   state". Read with `CORE_CONCEPTS.md` §13.1, that sentence is about the state the fact describes:
   only presence turns `arrived` into a `Presence`. It does not stop another system reacting to the
   fact by writing **its own** state, which is how `EconomySystem` answers `WageDue`. Relationships
   writes only `Acquaintances` and `knows`, never `ConversationHistory`. This sentence records the
   reading, so the two decisions do not appear to disagree (`CLAUDE.md` §2.1(4)).
3. **Decoding goes through the owner's published type**, a Cargo dependency on its vocabulary and never
   a registry dependency. A local mirror of another pack's payload is refused in review, because it
   would bypass `EventSchemaTooNew` / `EventSchemaOutdated`.
4. **Facts at level crossings only.** `became-acquainted` when an edge first forms (once per
   direction); `relationship-changed { from, to }` when the level (`Acquaintance`, `Friendly`,
   `Friend`, `Close`, from familiarity and regard) changes, up or down. Each is caused by the fact that
   formed or crossed it. Fine-grained values are reductions of logged facts, so a replay reproduces
   them (`ARC-25`).
5. **It provides no action, runs no process and has no wake.** Its state changes only by reducing
   `spoke`, `invitation-accepted`, `invitation-declined` and `group-activity-ended`. That is
   `CLAUDE.md` §4 rule 1 in its plainest form: the owner reacts, nobody else writes.
6. Values are disclosed to their holder only (`INV-13`): how Alice regards Bob is Alice's to know.

**Accepted limitations.** Values never decay, so a long-running world's social graph saturates: every
pair that keeps meeting reaches its top level within weeks and stops changing. That is a living-world
gap, recorded for a later step, not hidden by the tests (`step-09-social.md` QB-2). The constants are
published and not world configuration yet (the `ARC-26` note's rule). Only `knows` exists; romance and
typed relationships are later packs (`MVP.md` §4).

**Note, 2026-10-13 (S24, PR FG-a; `ARC-86`) — values may now decay, by configuration.** The limitation
"values never decay" is closed for a world that configures it: `configure/relationships.yaml` may name a
`DecayCurve` and a time constant and floor for familiarity and for regard, read lazily from the stored
`(value, last_contact)` (`ARC-86`), and a per-person cap `known_at_most` (default 150) evicts the lowest
current familiarity and states `acquaintance-forgotten`. Item 4's "`became-acquainted` … once per
direction" becomes **once per formation**: a counterpart evicted by the cap and met again forms a new
entry and is stated again. The defaults (`none`, 150) leave every reference world's facts unchanged, so a
world that configures nothing still saturates as described above. `relationships` is version 3.
