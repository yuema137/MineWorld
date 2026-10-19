## ARC-65 — Consequence routing: a list narrows a fact's audience and switches its biography, within the owner's bounds

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-8,
QIL-10, QIL-15) and the primary session at PR IL-b's freeze (QIB-11, QIB-12) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to** `INV-4`, `INV-11`, `INV-13`, `ARC-29`, `ARC-63`
· **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.7, §12 SD-IB-11, SD-IB-13

**Problem.** "Servants' lines are not history; nobody overhears in this world." A world must be able to
say what a fact means for history and perception without ever stopping a fact from being recorded,
leaking private state, or telling a mind what to forget.

**Choice.**

1. **What a list may govern, per fact type and role selectors:** its **audience**, narrowing along
   `Public ⊇ Place ⊇ Participants`, never below the owner's declared narrowest and never wider than its
   default (a widening is refused at load); its **biography** flag, `on` or `off`, only where the owner's
   `FactDecl` allows; and the pack's own typed knobs. `Entities(…)` and `SystemInternal` are never
   produced by a list: a list cannot name entities.
2. **What it may never do:** widen an audience; stop a fact from being recorded (`INV-11`); change a
   payload, an owner or a fact's subjects; route another pack's fact. A fact stated through another pack's
   constructor takes its consequence from its owner's section.
3. **"Enters history", layer by layer:**

   ```text
   fact log     always: every fact is recorded in every world (INV-11)
   biography    the ARC-29 projection asks the save's configured sections (item 4)
   in-world     the owning pack's own state (conversation's Remembered), through that pack's knobs
   memory       cognition's, from perceived facts only: the list reaches it only through audience
   ```

   No list writes into a mind; a world cannot tell cognition what to forget (QIL-10).
4. **`ARC-29` amended.** The biography projection selects through
   `mineworld_sdk::interactions::biography::selected(fact, person, compiled, configured)`. `Configured`
   is assembled from the save's genesis `*-interactions-configured` facts, each composed capability's
   `FactDecl`s, and each entity's type and tags. With nothing configured it is `ARC-29` exactly.
5. **The audience is chosen by the owner at emission**, through `consequence`, and carried in the
   envelope's `Visibility`; perception reads only the envelope, so it applies a list's narrowing with no
   change and no knowledge of lists.

**Accepted limitations.** In IL-b no installed pack declares a configurable fact: the routing is proven
with a test-only pack, and through the binary when IL-e converts conversation (QIB-11). Perceiving a
narrowed fact as a bystander is proven by S11-C's audience function and `mineworld perceived` when they
exist (QIB-12); IL-b proves the envelope's `Visibility`.

**Note, 2026-10-10 (S17, PR IL-e; QIE-3, QIE-5, QIE-6, QIE-10 … QIE-12) — the first configurable facts of
installed packs.** `conversation` declares `spoke` (owner default `Place`, narrowest `Participants`) and
`conversation-started` (`Participants`), both with configurable biography, compiled off, and the knob
`remember` (item 3's "in-world" layer: whether the listener's `ConversationHistory` keeps the line; it
never stops the fact being recorded, perceived or reduced by another pack). `group-activity` declares its
seven facts — the three invitation facts `Participants`, compiled non-biographical; `group-activity-started`
and `-ended` by `place` only, since their members list has no fixed positions; `joined-` and
`left-group-activity` by `actor` and `place` — each narrowable to `Participants`, biography configurable.
`relationships` declares `became-acquainted` and `relationship-changed` (already `Participants`;
biography configurable, compiled on). Each owner writes its default audience once, in its `FactDecl`, and
passes it as `owner_default` at emission.

The accepted limitations above are closed: the biography projection through the binary (QIB-11) and a
narrowed `spoke` not perceived by a bystander through `audience::admits` and `mineworld perceived`
(QIB-12) are proven by IL-e's tests. **Biography is chosen per fact, not per reader** (QIE-6, accepted for
MVP-0): `selected` fills the roles from the envelope, so its answer is the same for every person the fact
names — `{ fact: spoke, actor: servant, biography: off }` removes a servant's line from the servant's and
the listener's biography alike. Per-reader selection would change the SDK and this record. No line is
biographical by default in MVP-0 (QIE-12): a world that wants lines in biographies writes
`{ fact: spoke, biography: on }` first.
