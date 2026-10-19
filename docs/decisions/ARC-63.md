## ARC-63 — The World's Interaction List: one section shape for every pack, typed and enforced by its owner

**Date** 2026-10-08 · **Approved by** the operator (`overall.md` "The World Interaction List": QIL-2
overruled, QIL-4 … QIL-6, QIL-9, QIL-11 … QIL-16) and the primary session at PR IL-b's design freeze
(step-18-interaction-list §12; QIB-1 … QIB-14 accepted as recommended) · **Implements**
[`MODULE_SPEC.md`](MODULE_SPEC.md) §4.2 · **Relates to** `ARC-25`, `ARC-31`, `ARC-33`, `ARC-34`,
`ARC-61`, `ARC-64`, `ARC-65`, `DEP-28`, `INV-13` · **Design**
`.structured-coding/plans/mvp0/step-18-interaction-list.md` §4.4 … §4.6, §4.10, §12 (S17, PR IL-b)

**Problem.** A world author wants to say, without code, that nobles and commoners do not talk, that a
conversation lasts longer in this world, that an heirloom cannot be given. Every such rule today is a
compiled constant or a hard-coded branch inside one System Pack (step-18 §2.2). `ARC-61` gave every pack
a world-level file it types; it did not say what that file looks like. If each pack invented its own
shape, an author would learn twelve dialects, a tool could print none of them, and the first rule that
needed a class of person would be written twelve times.

**Choice.**

1. **One shape for every pack.** A pack's part of the list is its **section**: the file
   `configure/<pack id>.yaml`, carried by `ARC-61`'s seam unchanged. Every section has the same six
   optional keys and refuses any other (`deny_unknown_fields` throughout):

   ```yaml
   extends: default            # one of this pack's reference lists (item 4)
   default: permit             # permit | forbid: what an action no rule matches gets; permit unless said
   rules:                      # { action, <role>: <selector>…, effect: permit | forbid }
     - { action: talk, actor: noble, target: commoner, effect: forbid }
   parameters:                 # { <role>: <selector>…, <field>: <value>… }; unscoped = the base
     - { gap: 600 }
     - { actor: guard, gap: 1200 }
   consequences:               # { fact, <role>: <selector>…, audience?, biography?, <knob>… }
     - { fact: spoke, actor: servant, biography: off }
   regions:                    # by place key: rules, parameters and consequences for that place only
     library: { parameters: [ { gap: 60 } ] }
   ```

   A **selector** is an entity class (`ARC-64`), a type's implicit class, or `*`. A **role** is a position
   the owning pack declares per action and per fact: `actor`, `target`, `object`, `place`.
2. **The SDK supplies the shape; the pack supplies the meaning.** A pack implements
   `mineworld_sdk::interactions::InteractionSection` and writes `mineworld_sdk::interactions!();` inside
   its `impl SystemPack`:

   ```text
   type Parameters      made by parameters!: each field's type, bound (L0) and default, with an
                        all-optional partial twin for scoped entries
   type Knobs           pack-specific consequence fields (`()` for none)
   ACTIONS              each action: its roles, and whether a region may scope it
   FACTS                each fact type: which envelope position fills each role, its default audience,
                        the narrowest audience a list may choose, whether biography is configurable
   PARAMETER_ROLES      the roles a parameter entry may scope by
   CONFIGURED, COMPONENT   the names of its configured fact and per-place component
   reference_lists()    the compiled lists; always `default` (today's behaviour)
   encode, decode       the configured fact's payload, with the pack's own codec
   ```

   `interactions!()` makes the section the pack's configuration (`ARC-61`'s `CONFIGURATION`,
   `CONFIGURATION_FACTS`, `decode_configuration`), so a pack with a section has no other configuration,
   and states `INTERACTIONS` for the installed set's `Capability::interaction_section()`, which the tools
   read. The pack's `declaration()` and `install()` call `interactions::declare` and
   `interactions::install`, and its `react` calls `interactions::reduce` first.
3. **What decoding refuses, at its line and column** (the section's own type, `DEP-10`): an action or fact
   the pack does not declare; a role the pack does not declare for that action, fact or parameter
   block; a parameter outside its bound or unknown; a widened audience or one below the owner's
   narrowest (`ARC-65`); `biography` on a fact whose owner does not allow it; a rule in a region for an
   action that is not regional; an `extends` naming no reference list of the pack, or one whose chain
   is cyclic or longer than four. After decoding, once the world's classes are read, the loader refuses
   by name, naming the file, the list and the index of each entry: a selector naming a class that is
   neither declared nor implicit (`ClassUndefined`), and two **ambiguous** entries (`AmbiguousEntries`,
   item 5). A region naming no declared place is `ARC-61`'s unknown-entity refusal.
4. **Levels, highest last** (`L0` is never overridden):

   ```text
   L0  the pack's bounds         code; a value outside them is refused at load
   L1  the pack's `default`      compiled; today's behaviour
   L2  extends                   a named reference list of this pack (chains of at most four, no cycle)
   L3  the world's section       configure/<pack>.yaml
   L4  a region                  that section's regions.<place> entries, for that place only
   ```

   A higher level's entry replaces a lower level's entry with the same key — the action or fact plus its
   selectors — field by field for parameters and consequences; entries with different selectors
   coexist.
5. **Then specificity, then forbid.** Among the entries that apply to a request, the one naming the most
   roles wins (an implicit class counts as named). Among rules of equal specificity, `forbid` overrides
   `permit` (Cedar's rule, `DEP-28`). Two parameter entries or two consequence entries of equal
   specificity that **overlap** and give one field different values are refused at load: so ambiguity
   never reaches run time, and a lookup's answer does not depend on the order an author wrote entries
   in. Two entries overlap when, for every role, their selectors are equal, or one is `*`, or one is an
   implicit class whose type the other's class selects.
6. **Storage.** A configured section is resolved at genesis, by a pure function of the files, into one
   genesis fact `<pack>-interactions-configured`, `Visibility::SystemInternal`, no subjects, holding
   every level merged, each region, and the classes the section references. The pack reduces it into a
   component `<pack>-interactions` on every Place, holding the base and that place's region. The drift
   check (`ARC-61` item 7) compares that fact, so an edited section, an edited referenced class or a
   changed attachment is refused at resume.
7. **Lookups are pure and total.** `permits` (`Err(PermissionDenied)` when forbidden), `parameters`
   and `consequence` read the component on the place the pack names (a request's actor's place, a
   reaction's fact's place; without a place, the base, which every copy holds). With no component —
   the world configures nothing for that pack — each returns the compiled default and reads nothing
   else: `Ok`, the default parameters, the owner's default audience and compiled biographical flag. So
   a world that configures nothing produces byte-identical facts, pack by pack.
8. **Enforcement stays in the owning pack.** In `validate`, `permits` after the payload, actor and
   target exist and before the spatial requirement; in its offers through the same call, as
   `Offer::refused(PermissionDenied)` (`ARC-34` note); `consequence` at emission; `parameters` where it
   decides. The kernel sees nothing new.
9. **A list cannot grant.** It names only declared actions and facts; `permit` is a filter AND-ed with
   the pack's own validation; parameters stay within bounds; audience only narrows and biography changes
   only where the owner allows (`ARC-65`); a section for a pack the world does not enable is refused
   (`ARC-61`).
10. **`mineworld interactions <world> [--place KEY] [--json]`** prints each configured section resolved
   — base, then each region — each entity's class, and "default (compiled)" for a pack with a section
   that the world does not configure. It reads the World Pack only and writes nothing. Its JSON has
   sorted keys and is stable.

**Options considered.** (a) Each pack's own file format — rejected: twelve dialects, no tool. (b) A
general policy engine — rejected (`DEP-28`). (c) **One SDK shape, the meaning per pack** — chosen.

**Accepted limitations.**
- No attribute conditions (`when regard > 50`, QIL-11): data only, no expressions (QPL-1).
- A reference list is written in Rust by its pack, not in YAML: neither the SDK nor a pack depends on a
  YAML parser, and the loader's parser is not handed to them.
- A pack with a section has no other configuration file; a pack that needs a `data:` attachment
  (`ARC-61` note) and a section at once is a later amendment, when one exists.
- Converting a pack raises its `SystemVersion` (its declaration grows), so older saves are refused by
  name (`ARC-25`); no fact digest moves. A pack's `default` is pinned to its version by one test per
  pack.
- In IL-b only `conversation` and `group-activity` have sections, and they declare parameters only.
  Rules and consequences of real packs arrive with IL-e … IL-g.

**Note, 2026-10-10 (S17, PR IL-e; QIE-1, QIE-2, SD-IE-2, SD-IE-8) — the first rules of real packs, and a
rule decided in a reaction.** `conversation` declares the action `talk`; `group-activity` declares
`invite`, `accept-invitation` and `join-group-activity` (never `decline-invitation` or
`leave-group-activity`: a list can stop people coming together, not trap anyone in an invitation or an
activity; `accept-invitation` is governed because accepting joins the inviter's activity, so a
`join-group-activity` rule alone would leak). Each pack asks `permits` in `validate` after the payload is
readable, the actor and target exist and are Persons and the actor is located, and before any condition
that depends on the pack's own state (an open invitation, `Busy`) and before the spatial requirement; its
offers ask the same `permits` and answer `Offer::refused(PermissionDenied)` (item 8).

Item 1 says a rule is "an action and role selectors". A pack may also declare a **rule name for a yes/no
decision it makes in a reaction**: `relationships` declares `acquaint` (roles `actor`, the person who
would come to know, and `target`, the counterpart; not regional), asked per directed change while it
reduces another pack's fact, before any state is written; forbidden, that direction forms no
acquaintance, no `knows` edge and no fact. The name is declared through the same `ActionDecl` (the SDK
keys rules by `ActionTypeId`); it **names no action type a client can send**, and a test pins that no
installed pack provides an action type `acquaint`. No SDK type is added.

**Note, 2026-10-13 (S24, PR FG-a; QFGA-2, D-FGA-8) — four additions to the shape.** Item 2's "every
field with a bound" is generalised and three seams are added, each generic and none naming a pack
(`MODULE_SPEC.md` §4.2):

1. **A bound is a value.** `parameters!` declares each field as `name: Type = default, bound;`, where the
   bound implements `ParameterBound<Type>` (`admits`, `describe`). `RangeInclusive` implements it with
   today's message byte for byte, so every existing section decodes and refuses unchanged; a pack may
   supply a named-choice bound (`relationships`' curve catalog).
2. **Per-field roles.** `InteractionSection::FIELD_ROLES` lists, per field, the subset of
   `PARAMETER_ROLES` that may scope an entry setting it (unlisted: all of them). The decoder refuses, at
   the entry's line and column, an entry that sets such a field under another role, and a region entry
   that sets a field whose roles exclude `place`.
3. **A place-free lookup**, `parameters_unplaced`: the base resolution with the place role unfilled, or
   the compiled defaults reading nothing when unconfigured. It is correct for exactly the fields item 2
   makes unplaceable.
4. **A section's required systems.** `InteractionSection::requires(section)` (default none) is what the
   held section answers to the loader's existing requires check (`ARC-61`), so a section can require
   the packs whose code it selects.

New fields grow a pack's configured fact, so the pack's version is raised (`relationships` 3,
`conversation` 4), as item "Converting a pack raises its `SystemVersion`" already states.
