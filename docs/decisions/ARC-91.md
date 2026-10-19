## ARC-91 — Appearance is a perceivable trait of a Person, owned by the `appearance` System Pack

**Date** 2026-10-14 · **Status** accepted (step-26 freeze, operator Q-S26-11); implemented by S26-a · **Approved by** the operator (Q-S26-11, Q-S26-4, Q-S26-9) · **Relates to** `ARC-31` (sections), `ARC-33` (installed set), `ARC-46`, `ARC-92`, [`CORE_CONCEPTS.md`](CORE_CONCEPTS.md) §4.2 (Traits), [`MODULE_SPEC.md`](MODULE_SPEC.md) §4.1 · **Design** `.structured-coding/plans/mvp0/step-26-cast.md` §§4.1–4.2; `pr-s26a-appearance.md`

**Problem.** Every person must be drawn as themselves in both reference clients, and in any third client. A client can read only a person's entity id, tags, disclosed components and display name. None of them identifies an appearance: an id is a creation-order counter that renumbers when a place is added; a display name is absent without `naming`, can be shared, and would make a face change when a name changes or, if names are ever gated by acquaintance (`ARC-31`), when a stranger is introduced; a tag is semantic content that Systems and controllers read. Nobody owned an appearance, so each client invented one from a hash, and nine sprites came to serve twelve people.
**Options considered.**

| Option | Verdict |
| --- | --- |
| (a) `hash(entity id)` modulo a pool | Declined: distinct only by luck; cannot say "Alice looks like this" |
| (b) The entity id listed in a Presentation Pack | Declined: ids are creation-order counters; an unreviewable list of numbers |
| (c) The display name | Declined: absent without `naming`; may collide; makes appearance depend on what the observer knows |
| (d) A presentation tag (`look:alice`) | Declined: pollutes a semantic vocabulary every consumer reads, with no owner |
| (e) A disclosed `look` component owned by a System Pack | **Chosen** |

**Choice.**

1. **`appearance` is a System Pack** on the `naming` pattern: it owns a person file's `appearance:` section (`{ look: <key> }`, unknown keys refused), states a public `appeared` fact at genesis, reduces it into the `look` component (`{ "look": "<key>" }`, schema 1) and discloses that to every observer who perceives the person. It depends on nothing and declares no action or process.
2. **A look is an opaque key**: 1–48 bytes of `a`–`z`, `0`–`9` and `-`. It names an appearance; it is not a mesh, a file, a texture, a colour, a height or any other renderer concept. A Presentation Pack resolves it (`ARC-92`).
3. **No descriptors.** Apparent age, stature, build, skin and hair are not world facts: no System consumes them. They live in the casting brief, in the Presentation Pack. A System that later needs stature makes it a `bodies` section.
4. **Appearance never depends on what the observer knows.** Disclosure is gated by perception only.
5. **Single ownership.** A later clothing, ageing or disguise System changes a look by having this pack state `appeared` again; no other pack writes `look`.

**Consequences.** A world without the pack, or a person without the section, has no look, and every client draws them with its fallback. The default worlds' keys are name slugs for review readability. A controller sees `look` as it sees any component; the memory renderer stores `appeared` generically and never shows the key as a name.

**Revisit** if names become acquaintance-gated (the default keys then become opaque codes in one data migration per world, R-S26-9), or when a System needs an appearance descriptor.
