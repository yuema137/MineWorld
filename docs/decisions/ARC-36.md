## ARC-36 — An authored Item is a kind; items and organizations are content kinds of a World Pack

**Date** 2026-10-07 · **Approved by** the operator at S9's design freeze (step-10 QS-5, QS-6) ·
**Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1, [`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 ·
**Relates to** [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §§7–8, `ARC-15`, `ARC-31`, `ARC-35` · **Design**
`.structured-coding/plans/mvp0/step-10-market.md` §2.4, SD-7, SD-8, §4.2 (S9, PR 11b)

**Problem.** `CORE_CONCEPTS.md` defines four kinds of entity that a world is authored with: `Person`,
`Place`, `Item` and `Organization`. `MODULE_SPEC.md` §4's frozen World Pack layout already lists
`items/` and `organizations/`, and the contracts already have `EntityType::Item` and
`EntityType::Organization`. But the MVP-0 loader read only `places` and `population`, so no World Pack
could declare an Item or an Organization. A System Pack therefore could not own state on one the way
`naming` and `schedule` own state on a person (`ARC-31`).

There are three ways to give a world such entities:

```text
(a) a System Pack creates them at genesis      no system can create an entity: the view a system is
                                               handed has no create, and adding one is a kernel change
(b) no entities: kinds as slugs inside some    a kind would have no identity and no tags, and nothing
    pack's state, organizations as tags        for several packs to share; an Organization is a core
    on places                                  primitive (CORE_CONCEPTS §8), not a tag
(c) complete §4's frozen layout                chosen
```

**Choice: (c).**

1. **Two content kinds.** `world.yaml` gains `items:` and `organizations:`. Each key names a file,
   `items/<key>.yaml` or `organizations/<key>.yaml`, which may carry `tags`, `note` and sections. Both
   lists are optional, and a World Pack that declares neither is read exactly as before.
2. **An authored Item is a kind.** `CORE_CONCEPTS.md` §7 separates a type from an instance and allows
   "unique items, stacked items, or abstract resources". MVP-0 implements **stacked items only**. An
   Item entity declared in `items/lantern.yaml` *is* the kind `lantern`, and a quantity held of it is a
   count of that kind, kept by whichever System Pack owns holdings. This reads the defined term one
   of the two ways the ontology permits. It does not redefine it (`CLAUDE.md` §2.1(3)).
3. **Instances are out of MVP-0.** A unique instance, with its own owner, place or condition, would be
   an Item entity created while the world runs, by the pack that owns it. No system may create an
   entity (option (a)), so instances need a kernel decision of their own.
4. **Identities are allocated after people.** The order is places, then people, then items, then
   organizations, each in key order. Every identity a world had before it declared items or
   organizations stays where it was, and so does every event id of its genesis.
5. **Keys are one namespace across the four lists.** A key declared in two lists is refused, naming
   both. A section names another entity by key together with the kind it requires, so one key must
   never mean two entities.
6. **Item and organization files may carry sections.** This extends `ARC-31`'s person-or-place wording
   unchanged in every other respect:
   - the owner declares which kinds may carry its section;
   - the loader decodes the section with the owner's type;
   - a reference must name a declared key of the required kind, and that now includes items and
     organizations.
7. **Genesis order.** Passages and locations are seeded first, unchanged. Then sections are seeded:
   items', organizations', places', people's, each in key order, and within one file in composition
   order. What people's and places' sections are likely to refer to (a kind, an organization) is
   therefore seeded before them. The worlds that existed before this decision declare neither kind,
   so their genesis is unchanged.

**Accepted limitations.**

- There are no item instances (item 3).
- Items and organizations have no display names. `naming` carries people only, and a place has none
  either (`ARC-31`).
- No installed System Pack owns a section on an item or organization file yet. The first owners
  arrive with the packs that need them (step-10 PR 11d). Until then, the order and reference rules of
  items 5–7 are proven with a section owner that exists only in the loader's own tests.
- Organization membership, roles and accounts (`CORE_CONCEPTS.md` §8) are not authored fields. They
  are the state of whichever System Pack owns them, carried as its section.

**Note, 2026-10-08 (S15, PR 12c; step-11 QB-3, decided by the operator; §18.0, SD-O3) — an item file
with a `body:` section is one physical object.** Item 2 reads `Item` as a kind. The `bodies` System
Pack adds the second reading `CORE_CONCEPTS.md` §7 already allows, "unique items", for exactly one
case, and nothing else changes:

1. **An item file that carries the `bodies` pack's `body:` section declares one physical object**, not
   a kind: the Item entity *is* that object, lying in one place at one position, with one shape. It is
   created at genesis like every other authored Item (item 4); no entity is created while a world runs.
2. **It is never a declared kind, so it is never held.** It carries no `item:` section: `bodies`
   refuses, at genesis, an Item that is both a declared kind and an object (`bodies-held-kind`). What
   a person or an organization may hold is a declared kind (`ARC-37`), so the pack that owns holdings
   refuses every holding of an object by its own rule, at genesis and while the world runs. Lying in a
   place and being held are disjoint by construction.
3. **Instances in general stay out of MVP-0** (item 3). An object is authored, not created; a kind
   still has no instances; and how a unique object could one day be held — what carrying needs — is
   the first decision of the step that adds carrying, not this note's.
