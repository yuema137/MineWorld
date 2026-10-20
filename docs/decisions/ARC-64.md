## ARC-64 — Entity classes are named tag selectors, fixed during play

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-3,
QIL-6, QIL-7; QIL-2 overruled, so classes live in `configure/classes.yaml`) and the primary session at PR
IL-b's freeze (QIB-8) · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to**
`ARC-36`, `ARC-61`, `ARC-63`, `A-1` · **Design** `.structured-coding/plans/mvp0/step-18-interaction-list.md`
§4.2, §12 SD-IB-4

**Problem.** A rule about "nobles" needs to know who is a noble, in every pack, without a new owner of
state and without changing a world that never mentions nobles.

**Choice.**

1. **A class names a tag over one entity type.** `configure/classes.yaml` is a list of
   `{ class, of, tag }`: `class` has the system-id grammar (`authoring::ClassName`); `of` is `person`,
   `place`, `item` or `organization`; `tag` is a `Tag`. It is decoded by
   `mineworld_authoring::EntityClasses`.
2. **One class per entity, by priority.** An entity's class is the first entry whose `of` is its type
   and whose tag it carries; otherwise its type's **implicit class**, named by the type. Every entity
   also matches the selector of its type's implicit class. The four implicit names are reserved.
3. **Classes are not state.** Tags are a taxonomy fixed after genesis (`A-1`), so a class needs no fact,
   no component and no owner, and does not change while a world runs. `classes.yaml` is not seeded on
   its own: each section's resolved fact copies the class entries it references, and the entries of the
   same type listed before one of them (which could shadow it). So editing a referenced class is drift
   (`ARC-61` item 7), and editing an entry no section can see is not.
4. **`classes` is a framework key of `configure:`**, not a System Pack id: listed to be read, read
   before any section is decoded, refused as `ClassesInvalid` (its line and column) when a class is
   defined twice, an implicit name is reused, or `of` is not one of the four types. An unlisted
   `configure/classes.yaml` is refused as any undeclared file is.

**Options considered.** (a) A `classes` pack owning a mutable `Class` component and facts —
rejected for MVP-0 (QIL-7): an owner, facts and a migration for what tags already say. (b) **Named tag
selectors** — chosen: byte-identical when unused, already disclosed to clients, already immutable.

**Accepted limitations.** One class per entity and no inheritance (QIL-6); no promotion during play
(QIL-7); a tag no class names has no effect, and `mineworld interactions` prints each entity's class so
an author sees which applies (R-IL-7).
