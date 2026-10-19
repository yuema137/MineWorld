## ARC-82 — A doorway discloses the tags of the place it leads to: a door sign, never its contents

**Date** 2026-10-10 · **Approved by** the operator (Q1 of PR 13c's design, `.structured-coding/plans/mvp0/pr-13c-doorway-names.md` §11.1) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md), the movement system's disclosure · **Relates to** `ARC-26`, `ARC-75`, `ARC-62`, `ARC-45`

**Problem.** A person perceives only the place it stands in, and its doorways (`ARC-26`); the other places
are not perceived (`A-20`). A client therefore cannot tell which doorway leads to the café, cannot choose a
façade before entering, and labels every neighbour alike. The 2D client named a doorway by the last tag in
sorted order, which for most places is `public`.

**Choice.** Each entry of the `passages` disclosure gains one field, `to_tags`: the destination place's tag
set, in `Tag` order, always present (`[]` when the destination has none). It is built from current state
when the place is disclosed (`discloses`), by a disclosure-only type; the stored `Passage` and `Passages`
are unchanged, so no fact, persisted row or genesis changes. `component_type` stays `passages` and
`schema_version` stays `1`.

**The line.** The field carries tags and nothing else. It never carries the destination's identity beyond
`to`, its contents, occupants or activity. The audience of the disclosure does not widen: a person still
perceives no place as an entity except the one they stand in (pinned by the movement disclosure tests).

**Alternatives.** (a) Learn tags by entering the place (the fallback in step-13 QS12-7): keeps the street's
doorways generic, which the client needs to name them before entry. (b) Disclose the destination's full
record: widens the audience to what is inside. (c) Store `to_tags` in `Passage`: changes persisted state and
the genesis facts for a value that can be read from the destination at any time.

**Why.** Tags are authored world data, already shown to anyone who enters, and a door sign is what a
doorway shows from outside. Consumers that decode `passages` ignore unknown fields (the paced controller,
the agenda, the 3D slice, the protocol demo), so no controller decision changes; the 300-day digests of
`social-cafe` and `market-town` are the evidence (PR 13c).

**Consequences.** The 2D client names a doorway by the first tag, in order, that the shared wording names
(`place.<tag>`), else the destination's display name, else `outside`. A doorway without modelled positions is
not glued and its destination's tags are not learned by the 2D client; no drawn label is added on the street
(a follow-up finding).

**Revisit** if a place's tags must be hidden from people outside it, or if a door sign needs more than tags.
