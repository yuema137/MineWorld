## ARC-31 — A System Pack owns a section of an authored person or place file

**Date** 2026-10-07 · **Implements** [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 (sections, rule 6),
[`PACKAGE_FORMAT.md`](PACKAGE_FORMAT.md) §8 · **Relates to** `INV-7`, `INV-13`, `ARC-15`, `ARC-26`,
`DEP-10`, [`MVP.md`](MVP.md) §9 `AC-2` · **Design** `.structured-coding/plans/mvp0/step-09-social.md`
§10.1 Q9, §4.3 (S8, PR 10c)

**Problem.** Authored content becomes state only as a genesis fact its owner reduces (`ARC-15`), and
until now the loader knew each such field by name: a person's `location` is presence's, a place's
`passages` are movement's. A third and a fourth arrived together — a person's routine (`schedule`)
and a person's name. Nobody owned a name, so none reached a client: the 3D slice showed only tags,
and Alice said "Earlier, person 4 said …". Adding each one as another loader field would make every
new System Pack an edit to the World Pack format and its loader, the change-amplification
`CLAUDE.md` §4 rule 5 forbids, and the pattern finding F-1 already names. The repeated concept is now
observed twice over (`CLAUDE.md` §4 rule 11).

**Options considered.**

```text
(a) one loader field per pack (`routine:`), as `location`    each new pack edits the format, the
                                                            reader and the loader
(b) free items per pack (a const, two functions) matched     lighter; the shape is unchecked by the
    by the catalog, as `biographical` is                    compiler
(c) a trait a System Pack implements, in a crate of its     chosen
    own below the packs
```

**Choice: (c).**

1. **The contract.** `mineworld-authoring` (`authoring/`) defines `AuthoredSection`, implemented by a
   System Pack:
   - `SECTION`: the key it owns, one word;
   - `CARRIED_BY`: the content files that may carry it;
   - `Authored`: the section's own type. Deserializing it *is* the owner's validation, so an
     invalid section cannot be constructed;
   - `references`: the other entities it names by key, each with the entity type it must be;
   - `seed`: its genesis facts, in its own vocabulary, built with its own codec.

   The crate depends on the kernel's contracts only. It lives below the packs because the kernel and
   contracts know nothing of authored files, and a pack cannot depend on the World Pack loader,
   which depends on every pack.
2. **The loader never learns what a section means.** It decodes a section straight from the YAML
   stream into the owner's type, so a refusal keeps the line and column `DEP-10` chose `serde-saphyr`
   for. It checks only what every section shares:
   - the owner is enabled (otherwise refused, naming the pack, as an unowned `location` is);
   - the file kind may carry it;
   - every reference is a declared key of the required type.

   An unknown key is still refused, now listing the sections this build knows beside the fields.
3. **A seeded fact must be the owner's own.** World genesis attributes a fact to its event type's
   owner and checks no dependency, so a section that seeded another pack's vocabulary would bypass
   `ARC-26`'s rule. The loader refuses it.
4. **Order.** Sections are seeded after passages and locations, whose event ids therefore do not
   move: places' before people's, each in key order, and within one file in composition order.
5. **`location` and `passages` stay fields.** Moving them would edit `presence` and `movement` and
   change refusals authors already see, so it was not the no-op the step required. They are the
   pre-seam special cases, and their move is a later candidate, made when those packs are next
   opened.
6. **Section names are one namespace.** Each pack chooses its word. No two packs may claim one, and
   none may shadow a field, which a structural test over the build's catalog enforces. This follows
   the precedent of action types, which are already one namespace across packs.

**The first two users.**

- **`name`, owned by `naming`.** It owns `DisplayName` (component `display-name`, payload
  `{ "name": … }`), seeded by a genesis `named` fact.
  - The pack is called `naming`, not `identity`, because identity is the kernel's word for
    `EntityId` (`CLAUDE.md` §2.1(3)).
  - Names are **public**: disclosed to whoever perceives the person, oneself included.
  - A controller reads a name only from its observation. When the person it means is not
    perceived, it says "someone else" — never an id.
  - Readers of a save (`mineworld biography`) use naming's published projection of its own facts.
- **`routine`, owned by `schedule`** (`ARC-32`).

**Accepted limitations.**
- The catalog is still a closed list compiled into the build (F-1). The seam removes the format
  edit, not the registration.
- Places carry no names yet.
- Names are not gated on acquaintance. That would couple naming to relationships' state, and it is
  a decision for the step that wants strangers to stay nameless.
- There is no rename action.
