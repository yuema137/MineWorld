## ARC-43 — Facts reach observers through one audience rule, judged when they are recorded

**Date** 2026-10-08 · **Status** accepted; implemented in S11 PR S11-C · **Approved by** the primary
session at S11-C's design freeze (QS11C-1 … QS11C-5, QS11C-7 … QS11C-9) and the operator (QS11C-6) ·
**Relates to** `INV-13`, `ARC-28`, `ARC-29`, `ARC-40`, `ARC-41`, `ARC-42`, coordination rulings 1, 2 and
4 of `.structured-coding/plans/mvp0/overall.md` "Parallel build-out, 2026-10-08", and "The World
Interaction List" QIL-8 · **Design** `.structured-coding/plans/mvp0/step-12-server.md` §§4.7, 17 ·
**Specification** [`server/PROTOCOL.md`](../server/PROTOCOL.md) §§5.2, 5.8; [`MODULE_SPEC.md`](MODULE_SPEC.md)
§8.1 (`mineworld perceived`)

**Problem.** A fact states who *could* have learned of it — its `Visibility` — and nothing yet decides
who *did*. Three consumers need that answer: a client's observation frame (`observation.events`), a
cognition process that must not miss a fact and must resume after a dropped socket (S10's R-S11-1 …
R-S11-3), and an offline reader building a Person's memory from a save (S10's IC-1). If each answered it
for itself, a resumed stream, a live one and an offline export would disagree, and nobody could say
which was right.

**Choice.**

1. **One rule, owned by the perception system.** `systems/presence/src/audience.rs`:
   `Whereabouts` (which place each person is in — a fold of presence's own `arrived`, seeded from its
   `Presence` components or from a world's first fact), `admits` and `perceived_by`. `SystemInternal`
   reaches nobody; `Public` everybody; `Participants` exactly the fact's participants; `Entities(S)`
   exactly `S`; `Place(p)` the fact's participants and subjects, and everybody whose whereabouts *after
   the fact is applied* is `p`. Presence owns it because where people are is presence's state and
   perception is presence's job (`INV-13`); the server names no pack (it asks an `EventPerception` seam
   the composition root fills).
2. **Judged at record time, fact by fact.** As each fact is recorded the fold advances past it and every
   connection's observer is asked about it then. A `Place` fact is heard by whoever was there when it
   happened, not by whoever is there at the next 100 ms sweep; a person's own arrival is heard by them; a
   line said just before somebody leaves reaches them.
3. **Delivered three ways from that one function.** `observation.events`, best effort and bounded,
   drops counted in `events_dropped`; the `perceived` stream, reliable, ordered before the observations
   it explains, cursor-resumable, backfilled for a persisted world from its save by `perceived_by`; and
   `mineworld perceived`, the same `perceived_by` over a save, offline. Live, resumed and offline agree
   by construction, and the acceptance tests check that they do.
4. **The audience narrows at emission, not here.** A World Interaction List rule that makes a fact
   quieter states a narrower `Visibility` when the fact is built (QIL-8); this function reads only the
   envelope and needs no change for it. A hearing range is a later refinement behind the same seam; for
   MVP-0 overhearing is place-level (the operator's ruling on QS11C-6): a player on market-town's street
   hears every line said anywhere on the street.

**Supersedes.** S10's plan (`step-17-cognition.md` §3.3.4) for `Observation.events` to stay empty, with
perceived facts only in their own frame, and its check IC-9 "transcripts byte-identical": coordination
ruling 2 delivers both from the one function, so IC-9 reduces to "the 300-day digests are unchanged" —
`mineworld run` calls `observe`, which this decision does not touch. S10's text is amended by its owner
at its next pull request (QS11C-1).

**Rejected.**
- *Judging at sweep time* against the world as it stands: answers where people are now, not where they
  were when the fact happened; the "said before they left" case fails.
- *A broadcast channel of every fact, filtered per connection*: the filter needs the whereabouts at
  record time, which only the world thread has; and a lagging receiver on a broadcast channel loses
  facts silently.
- *An ephemeral world keeping a re-foldable window of recent facts* so it can serve old cursors: a
  second log. An unsaved world serves the stream from the join on and answers older cursors
  `cursor_unavailable` (QS11C-2).
- *In-server controllers receiving facts* now: none reads them (the paced controller reads histories,
  the reactive one its conversation history), and a queue nobody drains is a cost with no consumer
  (QS11C-4). The seam is there when one does.

**Limitations accepted.** A resume of a persisted world folds its whole fact log off the world thread (a
300-day market town is about 373 000 facts); seeding the fold from a snapshot is the recorded
optimization, taken when measured necessary (QS11C-8). A payload that is not JSON reaches the wire as
`null` (`PROTOCOL.md` §5.2); `EventEnvelope::map_payload` is a proposed later contract change, not taken
here (QS11C-3).
