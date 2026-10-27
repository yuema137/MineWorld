# PR PDN — Places have display names, authored in the World Pack and localisable

## DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended) 2026-10-26 (not frozen; implementation needs an operator freeze)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

```text
Lifecycle:              DRAFT 2026-10-26. Not frozen. No implementation before an operator ruling on §9 and a
                        DESIGN FROZEN header.
Parent:                 operator ruling of 2026-10-26 ("all as recommended") on the milestone F+D finding that
                        no place has a name; D-P6-43 (pr-s10-p6-lm-controller.md §15, PR #472)
Audit base:             origin/main as of 2026-10-26 (this branch's base)
Decision records:       a new ARC record (id from the coordinator's range), written in C1; it extends ARC-31
                        (names for places were "a later option") and adds a note to ARC-70 (world-authored
                        forms are selected by a client, never translated by one)
Implementation branch:  to be named at freeze
Implementation worktree: to be named at freeze, one under /Users/yuema137/mineworld-worktrees/
Execution contract:     §8
```

This document is the PR design for place display names. It is the single authority for this PR's scope,
contracts and acceptance once frozen. Until then every section is a proposal.

---

## 1. Problem

A place has no name anywhere in the shipped worlds. The `naming` System Pack owns a `name:` section, but
only a person's file may carry it (`systems/naming/src/section.rs`, `CARRIED_BY = [ContentKind::Person]`).
`ARC-31` recorded names for places as "a later option". Three readers feel the gap:

```text
2D client, I panel     readers.gd place(id): tags joined by ", ", else Words "ui.place" → "place 12"
2D client, door label  places.gd _place_name: a tag the shared wording names (place.cafe …), else the
                       destination's display name (never present), else "outside"
LM controller prompt   situation.py _learn: display names first (none for places), then tag_label →
                       "cafe/public" (D-P6-43); a place known only from `ways` stays "somewhere"
```

Two concrete examples of the cost. In Market Town, the I panel shows a place as its tags joined
("cafe, public") or, for a place whose tags this client never saw, as "place 7". A model asked where Alice
works reads "cafe/public", so it says "I work at cafe/public".

The tag fallback was the right bounded fix for D-P6-43. It is not a name. Tags are a world's vocabulary
for rules (`cafe`, `public`); a name is what people call the place ("The Copper Kettle"). A world with two
cafés has two places with the same tags, and only a name tells them apart.

## 2. Goal and non-goals

**Goal.** A World Pack may give any place a display name in the place's own file, with optional forms per
language. The server discloses it in Observations under exactly the rules that disclose the place's tags.
The 2D client, the 3D client and the LM controller show the name, in the viewer's language when the world
wrote a form for it, and fall back to today's behaviour when the world wrote none.

**Non-goals.**

```text
kernel change              the kernel stays ignorant of names; a name stays a component the naming pack owns
renaming at runtime        nothing renames a place in this PR (a later event, same fact type, if ever needed)
names in `ways`            a place known only from `ways` stays unnamed, as its tags are (§9 OQ-4)
person name forms          a person's name stays one string; the localisable forms are available to persons
                           by the same type, but no person file uses them in this PR (§9 OQ-5)
machine translation        a client never translates a name; it only selects among forms the author wrote
item names                 items are named by item kinds' catalogue (systems/item/src/name.rs); unchanged
```

## 3. Constraints that bind the design

1. **Kernel ignorance** (`CLAUDE.md` §4 rule 2). The kernel's entity record carries a key, a type and tags.
   It gains nothing. A name is `naming`'s component state.
2. **Single ownership** (rule 1). `naming` owns `display-name` for places as it does for persons. `movement`
   owns `passages`. Neither writes the other's state.
3. **No world-specific code** (lane charter, 2026-10-18). No place name, tag or world key appears in code.
   Names live in `worlds/*/places/*.yaml`.
4. **Disclosure as tags** (the operator's brief). A place's tags reach an observer in two ways today: the
   place's own entity record, to whoever stands in it (`ARC-26`); and the destination's tags on a doorway,
   `to_tags`, to whoever perceives the doorway's place (`ARC-82`). A place's name reaches exactly the same
   observers by the same two routes, and no others.
5. **Wording and translation rule** (`ARC-70`). Client UI text is keys plus gettext catalogs in packs;
   world content is never translated by a client. A world-authored name is world content. The localisable
   forms are therefore written by the World Pack's author, and a client only picks one by its language
   setting, with the fallback order of `ARC-70`'s note (chosen language, then English/default).
6. **Strong typing** (rule 7). Locale keys and names are named types, validated when the file is read.
7. **Determinism.** Genesis facts are a function of the pack; a world with no place names produces the same
   facts and digests as before.

## 4. Design

### 4.1 Authoring: the `name:` section on a place file

`naming`'s section is carried by places as well as persons. The section accepts two shapes:

```yaml
# worlds/<world>/places/cafe.yaml — the short form: one name, no other language
name: The Copper Kettle

# the long form: the default plus forms per language
name:
  default: The Copper Kettle
  forms:
    zh_Hans: 铜壶咖啡馆
    fr: La Bouilloire de Cuivre
```

Rules, all checked when the section is decoded (decoding is `naming`'s validation, `ARC-31` rule 1):

- `default` (or the scalar) is a `Name`: 1 to 64 bytes, no control character, no surrounding space.
- Each key of `forms` is a `Locale`: the stem of a `.po` file in `ARC-70`'s catalogs (`en`, `zh_Hans`,
  `fr`, `pt_BR`), matching `^[a-z]{2,3}(_[A-Z][a-z]{3})?(_[A-Z]{2})?$`. Using the catalog stem, not a BCP 47
  tag, means a client compares it directly with the locale its settings module reports
  (`clients/shared/settings/text.gd`, `TranslationServer.get_locale()`), with no conversion table.
- Each value of `forms` is a `Name`.
- At most 16 forms. A form equal to `default` is refused, so the file has one spelling per meaning.
- An unknown field is refused with its line and column (`DEP-10`).

A person file keeps the scalar form. The long form is accepted for a person too, because it is the same
type, but no shipped person uses it (§9 OQ-5).

### 4.2 Facts and state

A new fact, `place-named`, owned by `naming`:

```text
place-named   { place: PlaceId, name: Name, forms: { Locale: Name } }   public, about the place
              forms omitted from the bytes when empty
```

`named` (persons) is unchanged byte for byte, so every existing save's person facts and the 300-day digests
of worlds without place names are unchanged (§9 OQ-2 compares one generalised fact).

`naming` reduces `place-named` into the place's `DisplayName`. `DisplayName` gains an optional `forms`
field, omitted from the payload when empty:

```json
{ "name": "The Copper Kettle", "forms": { "zh_Hans": "铜壶咖啡馆" } }
```

`schema_version` stays 1: the field is optional, a person's payload is byte-identical, and every reader
that decodes `display-name` reads `name` and ignores unknown fields (`observation.gd`, `situation.py`,
`clients/agent/src/mineworld_agent/view.py`, `cognition/rule-controller/src/act.rs`). This is `ARC-82`'s
precedent. Seeding order follows `ARC-31` rule 4: places' sections before people's, so a world that names
places shifts the event ids of its people's `named` facts. That world's baseline digests are regenerated in
C4, by name.

### 4.3 Disclosure

Route 1, **the place one stands in.** `naming`'s existing `discloses` returns the subject's `DisplayName`
to every observer who perceives the subject. A person perceives no place except the one they stand in
(`ARC-26`, pinned by movement's disclosure tests), so the place's own name reaches exactly those who see its
tags. No code change beyond the component now existing on places.

Route 2, **the door sign.** `to_tags` is built by `movement` from kernel tags. A name is not kernel state,
so `movement` cannot read it without depending on `naming`. The design keeps the dependency pointing from
the optional pack to the fundamental one: `naming` depends on `movement`'s public `Passages` component (read
only), and discloses a disclosure-only record on the place the observer stands in:

```text
component_type  door-names      owner naming      never stored (a disclosure-only type, as DisclosedPassage)
payload         { "leads_to": [ { "to": PlaceId, "name": Name, "forms": { Locale: Name } } ] }
entries         one per doorway of that place whose destination has a DisplayName, in Passages order
audience        whoever perceives the place: the same observers who receive `passages` with `to_tags`
```

The record carries the destination's name and nothing else: never who or what is inside (`ARC-82`'s line).
A world without `movement` has no doorways and no `door-names`. A world without `naming` has no names and
every reader falls back as today. Neither pack is forced on a world.

Rejected routes: putting `to_name` in `movement`'s `passages` (movement then depends on naming, so every
world with movement must build naming); putting a name in the kernel's entity record beside tags (breaks
kernel ignorance); disclosing the destination's `display-name` record about the destination itself (the
destination is not perceived, and perception records are about perceived subjects, `ARC-26`).

### 4.4 Readers

**Shared protocol module** (`clients/protocol/mineworld/observation.gd`, both Godot clients):

- `display_name(id)` keeps its meaning (the default name) for callers that log or compare names.
- New `shown_name(id, locale)`: the form for `locale`, else the form for its language part (`zh` for
  `zh_Hans`, only when exactly one form has that language), else the default name, else `""`.
- New `door_names()`: `{to_id: {name, forms}}` from the observer's place's `door-names` record.

**2D client.**

- `readers.gd` `place(id)`: shown name (current locale) → remembered shown name → tags as today →
  `ui.place`. Door names from `door-names` are remembered in presentation memory, as `town.tags` is.
- `places.gd` `_place_name(to)`: the destination's shown name first, then today's tag wording
  (`PLACE_NAME_KEYS`), then `ui.outside`. A name the world wrote is more specific than a tag's wording.
- A language switch re-labels live, because the label is computed from the stored forms on each draw.

**3D client.** `slice_link.gd` `shown_label` and `things.gd` already go through `display_name`; they switch
to `shown_name` with the settings module's locale. Door labels read `door_names()`.

**LM controller** (`cognition/lm-controller`).

- `situation.py` `_learn`: learn display names (default name, §9 OQ-3), then `door-names` entries, then tag
  labels. A name always wins over a tag label, and a door name over a tag label of the same place.
- `names.py`: docstring updated; `tag_label` stays the fallback for unnamed places.
- No prompt template changes: the label flows through `NameBook` as today.

**Python SDK and agent client.** `sdk/python` gains the `door-names` reader beside its `display-name` one;
`clients/agent/src/mineworld_agent/view.py` names places through it. The MCP agent sees the same names a
human sees (`agents-first-class-players`).

### 4.5 Content

Every shipped world gives every place a default name in its file: Market Town, Social Café, Lakeside,
Bodies Yard. Market Town also gives a `zh_Hans` form for each place, so the localised path is exercised by
a real world. Names are chosen by the content author; this design names none.

### 4.6 Affected contracts

```text
naming section `name:`          CARRIED_BY += Place; Authored = NameSpec (scalar | {default, forms})
naming fact `place-named`       new, schema 1
naming component `display-name` optional `forms`; schema stays 1; person bytes unchanged
naming disclosure `door-names`  new, disclosure-only
naming Cargo dependency          += mineworld-movement (read Passages); movement unchanged
server/PROTOCOL.md               documents `door-names` and `forms`
docs/MODULE_SPEC.md §4.1         section table: `name` carried by person and place
docs/PACKAGE_FORMAT.md §8        place file may carry `name:`
docs/decisions                    new ARC record; ARC-31 note; ARC-70 note
observation.gd (protocol)        shown_name, door_names
sdk/python wire readers          door-names reader
```

Unchanged: the kernel, `mineworld-contracts`, `movement`, `presence`, the save format, every action and
rejection code.

## 5. Acceptance criteria

```text
AC-PDN-1  A place file with `name:` (both shapes) loads; the 2D/3D/LM readers show it. A file with an
          invalid locale, an empty name, a duplicate form or an unknown field is refused naming file, line,
          column and the rule.
AC-PDN-2  Disclosure equals tags' audience: in Market Town and Social Café, for every seat over a 1-day
          headless run, an observation contains a place's display-name iff it contains that place's entity
          record with tags; and a door-names entry for place P iff a passages entry with to = P is present.
          No observation names a place by any other route.
AC-PDN-3  Default parity: a world with no place names (a generated variant of Social Café with every place
          `name:` removed) produces the same facts, the same 300-day digest and the same LM prompts as main.
AC-PDN-4  Person parity: every person's `named` fact bytes and `display-name` payload bytes are unchanged.
AC-PDN-5  2D: the I panel and door labels show "The Copper Kettle"-style names in Market Town and Lakeside;
          with the language set to zh_Hans, Market Town shows the zh_Hans forms and Lakeside (no forms)
          shows its default names. 3D: the same for door labels in the Market Town slice.
AC-PDN-6  LM: the recorded market-town street observation yields prompts that name the café by its name,
          not "cafe/public"; the tag label remains for an unnamed place in the C3 fixture.
AC-PDN-7  No code file contains a place name or world key (grep check over systems/, clients/, cognition/,
          sdk/ for each authored place name, run in C4).
```

## 6. Commit plan

Each commit tracks implementation, deterministic validation, and LLM logic review as separate items.

### C1 — `naming`: place names, forms, `place-named`, `door-names`, decision record

**Goal.** The server side: authoring, fact, state and both disclosure routes. It stands alone because every
reader depends on it and it is testable headless.

**Scope.** `systems/naming/src/{name.rs, section.rs, event.rs, component.rs, system.rs, lib.rs, codec.rs}`,
a new `systems/naming/src/door.rs` (disclosure-only type), `systems/naming/Cargo.toml`
(+ `mineworld-movement`), `systems/naming/tests/naming.rs` and a new `tests/places.rs`,
`systems/naming/README.md`, `server/PROTOCOL.md`, `docs/MODULE_SPEC.md` §4.1, `docs/PACKAGE_FORMAT.md` §8,
the new ARC record and notes on ARC-31 and ARC-70, `docs/decisions/INDEX.md`. Unchanged: `named`, movement,
presence, the kernel.

**Implementation.**
- [ ] `Locale` and `NameSpec` types (scalar or `{default, forms}`), validated on decode.
- [ ] `CARRIED_BY` += `ContentKind::Place`; `seed` emits `named` for a person, `place-named` for a place.
- [ ] `place-named` fact; `react` reduces it into `DisplayName { name, forms }`.
- [ ] `DisplayName.forms` optional, skipped when empty.
- [ ] `door-names` disclosure-only record from the observer's place's `Passages`.
- [ ] `names_in` also projects `place-named` (for `mineworld biography` and `inspect`).
- [ ] Decision record, notes, docs, `scripts/check_decision_ids.py --write-index`.

**Validation.**
- [ ] Section decode: both shapes accepted; empty name, bad locale, duplicate-of-default form, 17 forms,
      unknown field each refused with line and column (negative tests).
- [ ] A person with the scalar form: `named` bytes equal to a recorded literal from main (AC-PDN-4).
- [ ] Two-place fixture world: the observer inside A sees A's `display-name`; the observer in B (doorway to
      A) sees `door-names` for A and not A's `display-name`; an observer in C (no doorway to A) sees neither
      (AC-PDN-2 at unit scale).
- [ ] Mutation: drop the `door-names` audience check (disclose for every place) and see the C-observer test
      fail.
- [ ] `cargo fmt`, `cargo clippy -p mineworld-naming -D warnings`, `cargo test -p mineworld-naming
      -p mineworld-movement`.

**Review.** - [ ] An LLM review of the diff against §3 constraints 1, 2, 4 and the rejected routes in §4.3.

**Acceptance.** AC-PDN-1 (server side), AC-PDN-4, AC-PDN-2 at unit scale.

**Failure cases.** A place file names a place with a person-only shape: none exists, both shapes are valid
for both. A `door-names` destination without a name: omitted from the list, never `""`. A world without
`movement`: no `Passages`, no record. Malformed section: load refused, world does not start.

### C2 — protocol module, 2D and 3D clients

**Goal.** Both Godot clients show the names, in the chosen language.

**Scope.** `clients/protocol/mineworld/observation.gd`, `clients/2d/scripts/hud/readers.gd`,
`clients/2d/scripts/scene/places.gd`, `clients/3d-spike/scripts/slice/slice_link.gd`, `things.gd`, the
existing client harness tests (`tools/cli/tests/client_2d*.rs`, `client_3d_market.rs`) and their fixtures.
Depends on C1.

**Implementation.**
- [ ] `shown_name(id, locale)` and `door_names()` in the protocol module, with the fallback of §4.4.
- [ ] 2D `place(id)` and `_place_name(to)` precedence changes; door names remembered.
- [ ] 3D labels through `shown_name`.

**Validation.**
- [ ] Protocol-module unit scene: payload with and without `forms`, locale exact, language-part, absent.
- [ ] 2D harness on Market Town and Lakeside, `en` and `zh_Hans` (AC-PDN-5); a world without names still
      prints `place <id>` / tag labels as before.
- [ ] 3D Market slice probe prints door labels by name.

**Review.** - [ ] An LLM review: no rule in the renderer, no world key in client code (§3 constraint 3).

**Acceptance.** AC-PDN-5.

### C3 — LM controller, Python SDK, agent client

**Goal.** Prompts and the MCP agent name places by their names.

**Scope.** `cognition/lm-controller/src/mineworld_cognition/controller/situation.py`, `memory/names.py`,
`sdk/python/src/mineworld_sdk/` (door-names reader), `clients/agent/src/mineworld_agent/view.py`,
`cognition/lm-controller/tests/test_place_labels.py` and a recorded observation fixture with names.
Depends on C1.

**Implementation.**
- [ ] `_learn` order: display names, door names, tag labels.
- [ ] SDK reader for `door-names`; agent view uses it.

**Validation.**
- [ ] `test_place_labels.py`: named café → name; unnamed place → tag label (AC-PDN-6); a door name never
      replaced by a later tag label.
- [ ] Scripted-model prompt snapshot for a seat in Market Town (no live model).
- [ ] `ruff`, `pyright`, the affected pytest modules.

**Review.** - [ ] An LLM review: no tag or world named in code (D-P6-43's rule kept).

**Acceptance.** AC-PDN-6.

### C4 — content: every shipped world names its places

**Goal.** The worlds use the mechanism; baselines move by name.

**Scope.** `worlds/{market-town,social-cafe,lakeside,bodies-yard}/places/*.yaml`, `scripts/baselines.txt`
(the digests of worlds whose genesis changed), the generated no-names variant used by AC-PDN-3.

**Implementation.**
- [ ] A default name per place in every world; `zh_Hans` forms for Market Town.
- [ ] Regenerate the affected baselines; record old and new digests in §7.

**Validation.**
- [ ] AC-PDN-3 on the no-names variant of Social Café; AC-PDN-2 over 1 day for Market Town and Social Café.
- [ ] AC-PDN-7 grep.
- [ ] `market_town.rs` ASRB-1/ASRB-2 still pass (genesis grows by a few facts only).

**Review.** - [ ] An LLM review that only data files and baselines changed.

**Acceptance.** AC-PDN-2, AC-PDN-3, AC-PDN-7.

## 7. Evidence

None yet (DRAFT).

## 8. Execution contract

To be filled at freeze: branch, worktree, authorised validation budget (per the lane charter, local checks
through `cast-slots/with-slot.sh`, full tests at the milestone), and merge path (the coordinator merges).

## 9. Open questions (each with a recommendation)

```text
OQ-1  Where do the localised forms live?
      (a) inline in the place file's `name:` section (§4.1)
      (b) gettext files in the World Pack, i18n/<locale>.po with msgctxt place.<key>, read by the loader
      Recommend (a). The server must disclose the forms anyway, because a remote client has no World Pack
      files, so (b) still needs a Rust .po reader (a new dependency and a DEP record) plus a loader change
      that hands per-key tables to sections, which ARC-31 rule 2 keeps out of the loader. (a) needs neither.
      The disclosed shape is the same, so (b) can be added later without touching any reader.

OQ-2  One fact or two? Generalise `named` to { subject: EntityId, … } or add `place-named`.
      Recommend `place-named`. It keeps every existing person fact byte-identical, so no save or digest of a
      world without place names moves (AC-PDN-3, AC-PDN-4). One extra event type is cheap.

OQ-3  Which form goes into the LM prompt?
      Recommend the default name in MVP-0. The controller has no language setting today; a seat's prompt
      language is a later controller-configuration field, and the forms are already disclosed for it.

OQ-4  Should `ways` (places reachable but not adjacent) disclose names?
      Recommend no, in this PR. The brief binds names to tags' disclosure, and `ways` discloses no tags
      either. A place known only from `ways` keeps "somewhere" / "place N" until a later PR widens `ways`
      for tags and names together.

OQ-5  Should person names accept forms too?
      Recommend the type accepts them (it is one type) but no shipped person uses them, and clients show a
      person's form by the same `shown_name`. Whether a person's name is translated at all is a content
      question for the world author.

OQ-6  Does a door name outrank a tag's shared wording on a 2D door label ("The Copper Kettle" vs "Café")?
      Recommend yes: the author's name is more specific, and the tag wording remains the fallback.

OQ-7  Dependency direction for door names: naming → movement (this design) or movement → naming.
      Recommend naming → movement. The optional pack depends on the fundamental one, so a world with
      movement and no naming builds as today.
```
