## ARC-29 — A biography is a projection of the fact log, selected by what each pack declares biographical

**Date** 2026-10-07 · **Implements** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§5.2, 5.4 · **Relates to**
`INV-4`, `INV-11`, `ARC-25`, [`MODULE_SPEC.md`](MODULE_SPEC.md) §8.1 · **Design**
`.structured-coding/plans/mvp0/step-09-social.md` SD-12, §10.1 Q10, §4.2 (S8, PR 10b)

**Problem.** Milestone B asks that Alice and Bob "survive a restart with their history", and
`CORE_CONCEPTS.md` §5.2 says a Person's objective biography is derived from the event log and can
always be regenerated. Something must derive it. It must not become a second account of history, and
adding a pack must not mean writing biography code.

**Options considered.** (a) A stored biography component, maintained by reducers: a second truth that
can disagree with the log. (b) Per-pack narrator functions returning typed entries: more expressive,
and an abstraction with one shape so far (`CLAUDE.md` §4 rule 11). (c) **A generic projection over
fact envelopes**, with each pack declaring which of its own event types are biographical.

**Choice: (c).**
- An entry is `{ at, event id, event type, place, counterparts }`, read only from the envelope's
  kernel fields.
- A fact belongs to Person P's biography when P is among its subjects or participants **and** its
  type is in the composition's biographical set.
- Each System Pack exports `pub const BIOGRAPHICAL: &[EventTypeId]`, its own judgement over its own
  vocabulary, and the World Pack catalog aggregates them (`Capability::biographical`). A pack that
  declares none contributes none: presence, movement and conversation contribute nothing in S8.
- `mineworld biography <world> --save DIR --person KEY` reads a save's fact table and nothing else. It
  never resumes or writes the world, and every line carries its event id (§5.4).

**Why it is honest.** The biography is never stored, so it cannot drift. It is tested against the
owner packs' typed payloads rather than against the envelope rule it implements, so a biography that
invented or dropped an entry fails. It is regenerated identically from a restarted world's save.

**Accepted limitations.** L0 only: a long world's biography is long. Compression (L1–L3) is `AC-10`'s,
in S10. Generated prose is display, never state (§4.4). A pack that states a fact without naming its
people in the envelope is invisible to biographies; that is the pack's defect, not the projection's.

**Note, 2026-10-08 (S17, PR IL-b; `ARC-65` item 4, QIL-15).** The selection is now
`mineworld_sdk::interactions::biography::selected`. A fact whose owner has a configured section is
biographical when that section's consequence for the fact's roles says so (`biography: on | off`, only
where the owner allows); otherwise the compiled set above decides. `mineworld biography` therefore also
reads the save's genesis `*-interactions-configured` facts and each entity's type and tags (from the
World Pack it already reads for keys and names), and each composed capability's declared facts. It still
writes nothing and resumes nothing. With no configured section it is this decision exactly.
