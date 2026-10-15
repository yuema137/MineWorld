# PR S30-a — Person attributes: the specifications and the decision records

## DESIGN FROZEN — 2026-10-15 (operator approved the freeze in the primary session: "同意冻结并实施 S30-a/b")

```text
Design revision:        revision 2 (2026-10-15: §14's rulings applied), FROZEN 2026-10-15
Author:                 S29-a / S30-a / S30-b PR-design session (Opus), worktree
                        /Users/yuema137/mineworld-worktrees/design-s29a-s30ab, branch docs/pr-designs-s29a-s30ab,
                        planning base origin/main @ e6030a36 (#225, the S30 amendment, merged)
Approved by / evidence: the operator, directly in the primary session, 2026-10-15: "同意冻结并实施 S30-a/b";
                        Q-S30a-1, Q-S30a-2 (and S30-b's Q-S30b-1) ruled as recommended (§14.1), relayed by the
                        coordinator
Implementation base:    origin/main at C0; C0 records the commit and the merge state of S26-a (#206), P6a and S31's
                        step
Start condition:        none beyond this design's freeze and the primary's dispatch: step-30 §13.3 "S30-a starts
                        on step approval" (approved 2026-10-15)
Execution contract:     §12
Implementer:            Opus-class (step-30 §13.1, the S30-a row); reviewer the primary session
Lifecycle:              DESIGN FROZEN 2026-10-15
```

**Effort:** `mvp0` · **Step:** [`step-30-person-attributes.md`](step-30-person-attributes.md) (APPROVED 2026-10-15,
amended the same day; Q-S30-1…9 ruled, RR-S30-1…5 recorded, §16) §§0, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13 (the S30-a
row), 15 · **Parent:** [`overall.md`](overall.md) · **Followed by:** S30-b (the authoring seam), whose design is
frozen with this one; S30-c … S30-k · **Coordinates with:** step-26 §7.4b (C-1…C-7, `ARC-133`), step-31 §§7.4, 9
(`ARC-114`, `economy.wealth-band`), S26-a (#206, `ARC-91`), P6a (`ARC-74`, unpushed) · **Binding parents:**
`CLAUDE.md` §§2.1, 2.2, 4 (items 1, 2, 5, 7, 10, 11, 12, 16); `docs/CORE_CONCEPTS.md` §§2, 3.1, 4, 15;
`docs/MODULE_SPEC.md` §§2, 3, 4.1; `docs/DECISIONS.md` `ARC-26`, `ARC-31`, `ARC-61`, `ARC-71`.

**Decision ids** (confirmed free on `origin/main @ e6030a36`: `git grep -n "^## ARC-12[0-9]\|^## DEP-6[6-9]"
docs/DECISIONS.md` is empty). This PR writes `ARC-120` … `ARC-126` and `DEP-66` … `DEP-68`, as step-30 §15.5
assigns them. `ARC-127` … `ARC-129` and `DEP-69` stay reserved and unused.

Documentation only. No code, no world, no test, no model call.

---

## 0. Binding inputs, exactly restated

- **Scope (step-30 §13.1, S30-a row).** "`CORE_CONCEPTS.md` §4 (new §4.5 "Attributes", §15); `MODULE_SPEC.md`
  §§2, 3, 4.1 (the `attributes:` section, `configure/attributes.yaml`, data directories); `DECISIONS.md`
  (`ARC-120`…`ARC-126`, `DEP-66`…`DEP-68`, the dated amendment to `ARC-91` item 3); `overall.md` S30 entry."
- **Checkpoint (same row).** "documents agree with each other (the doc checks pass; a review against §§4–11)."
  Adversarial criteria: "none (docs only)". Implementer: Opus.
- **Order (step-30 §13.3).** "S30-a starts on step approval. It is a design-doc-only PR, merged after review under
  the standing design-doc rule." S30-b follows it.
- **Rulings (step-30 §16).** Q-S30-1 (`ARC-125`), Q-S30-2 (profiles in a data pack), Q-S30-4 (P6a merges as
  frozen), Q-S30-5, Q-S30-6 (`economy` owns the wealth band), Q-S30-8 (sensitive attributes private by default),
  Q-S30-9 (an optional `log-linear` mode of the MatrAIx backend only), and the re-rulings RR-S30-2 (everything
  before the release), RR-S30-3 (≤ 100 built-ins), RR-S30-5 (backends; MatrAIx optional). None is re-asked.
- **`CLAUDE.md` §2.1.** Every non-README `.md` here is a specification: complete, exact, terminologically fixed.
  `Attribute` becomes a defined term (step-30 §15.1); it is never a synonym for `Component`.

---

## 1. Goal

Write the specifications and the decision records that every later S30 PR implements against, before any of them
is designed to the commit: the concept (`CORE_CONCEPTS.md` §4.5), the formats and refusals a world author and a
pack author write against (`MODULE_SPEC.md` §§2, 3, 4.1), and the ten decisions with their alternatives
(`DECISIONS.md`). After this PR a reader holding only the repository knows what an attribute is, who owns it, how
it is declared, typed, seen, sampled, persisted and generated, and why — with the step's rulings applied and with
the contradictions the audit found (§3.2) decided, not carried.

---

## 2. Scope

### 2.1 In scope

1. **`docs/CORE_CONCEPTS.md`.** A new `## 4.5 Attributes` (step-30 §15.1, stated in full: the definition, the
   four classes of §§4.1–4.3 and which an attribute may be, one owner, visibility, mutability, derived values,
   "a view, never a copy", and what an attribute is not — a component, a memory, a name). §4.2's list gains a
   pointer ("declared as attributes, §4.5"). §15.1 gains the example line of an observer-filtered `traits`
   component. INV references (INV-4, INV-7, INV-12, INV-13, INV-15) are cited, not restated.
2. **`docs/MODULE_SPEC.md` §2 (Entity Pack).** The MVP-0 subset widens from "item kinds" to "item kinds and the
   data directories an installed System Pack owns" (`ARC-121`): `attributes/`, `populations/`, `vocabularies/`,
   `masks/`, `mappings/`, `backend.yaml`, `labels/`. The "item kinds only" sentence is amended with a dated note.
   The built-in catalogue (`entities/person-attributes`), the profiles pack (`entities/populations-us`) and the
   optional backend pack (`entities/attributes-matraix`) are described as Entity Packs of that shape.
3. **`docs/MODULE_SPEC.md` §3 (System Pack).** What a pack declares about attributes: `reads_attributes`,
   `proposes`, `AttributeOwner`; the general authoring seam of `ARC-121` stated without the word "attribute" (a
   pack may own data directories; a section may be decoded in its owner's context; an owner may resolve across
   files before seeding; a System Pack may ship data files for another owner's directory, as Q-S30b-1 rules).
4. **`docs/MODULE_SPEC.md` §4.1 (World Pack).** The person file's `attributes:` section; a world's own
   `attributes/`, `populations/`, `masks/`; `configure/attributes.yaml` (backend, mask, profile, seed, salt, lock);
   the lock file; every refusal of step-30 §§5, 6.4, 6.5, 7, 8 in one table (name, when, what it names).
5. **`docs/DECISIONS.md`.** `ARC-120` … `ARC-126` and `DEP-66` … `DEP-68`, each in the file's record format
   (Date · Status · Approved by · Relates to · Design; Problem; Options considered; Choice; Consequences;
   Revisit), with the step's rulings and re-rulings as their approval and the superseded texts kept visible in
   `DEP-66` (step-30 §15.5). The `ARC-91` amendment as §3.2 F-S30a-1 resolves it.
6. **`overall.md`.** The S30 entry (step-30 §15.6), unless the primary has written it by C0, in which case only a
   reconciliation.

### 2.2 Not in scope, and where it goes

| Item | Where |
| --- | --- |
| `MODULE_SPEC.md` §8.1 rows for `mineworld attributes catalogue | show | check | resolve | lock | backends` | S30-c, S30-d (the section lists commands *as implemented*) |
| Any code, crate, pack, world, profile or test | S30-b … S30-k |
| Step-26, step-27/S27-d, step-28, step-31 edits | their owners (step-30 §15.3–15.4); listed for the primary in §11 |
| The guide chapter of the MatrAIx demo | S30-h |

### 2.3 Dependencies

None for the start. `ARC-91` (S26-a, #206) and `ARC-74` (P6a) are **not on `main`** at the audit base; §3.2
F-S30a-1 states how the records are written either way.

---

## 3. Audit (`origin/main @ e6030a36`, 2026-10-15)

### 3.1 Anchors

| # | Fact | Source | Consequence |
| --- | --- | --- | --- |
| A-1 | `CORE_CONCEPTS.md` §4 has §4.1 Identity, §4.2 Traits, §4.3 Mutable state, §4.4 Relationships; §15.1 lists an Observation's parts | `docs/CORE_CONCEPTS.md:121-182, 772-789` | §4.5 is the next free number; `check_doc_headings.py` holds it unique |
| A-2 | `MODULE_SPEC.md` §2 says "MVP-0 reads **item types** only … component schemas belong to System Packs in MVP-0 (their sections, `ARC-31`)" and shows `modern-goods/{pack.yaml, items/}` | `docs/MODULE_SPEC.md:40-92` | the sentence is amended, with a dated note citing `ARC-121` |
| A-3 | `MODULE_SPEC.md` §3 / §3.1 list what `SystemPack` declares (`PACKAGE`, `BIOGRAPHICAL`, `SECTION`, `CONFIGURATION`, `INTERACTIONS`) | `docs/MODULE_SPEC.md:95-170` | the seam's declarations are added to that list as specified, marked "implemented by S30-b" |
| A-4 | `MODULE_SPEC.md` §4.1 shows `world.yaml`, the person and place files and every section; there is no §4.3 (S29-a adds one) | `docs/MODULE_SPEC.md:476-940` | attributes go into §4.1's file blocks and a subsection after the existing ones; S29-a's §4.3 and this PR's text do not collide (§13 R-a-2) |
| A-5 | `DECISIONS.md`'s last records are `ARC-105` … `ARC-108`, `DEP-60`; the format is the heading `## ARC-n — title`, then the bold metadata line, Problem, Options, Choice, Consequences, Revisit | `docs/DECISIONS.md:8305-8410` | new records go at the end, in that format; `check_decision_ids.py` holds ids distinct |
| A-6 | `ARC-91` (appearance, "no descriptors", item 3 and its revisit clause) is on `origin/mvp0/pr-s26a` (#206, OPEN), not on `main`; `ARC-74` (persona) is on the unpushed P6a branch | `git grep -n "^## ARC-91\|^## ARC-74" origin/main -- docs/DECISIONS.md` (empty); step-30 A-4, A-5 | F-S30a-1 |
| A-7 | Step-30 §6.1 lists 94 built-ins; row 32 `occupation` is "view of `employment` where installed", row 37 `wealth-band` "view, owned by `economy`"; §7.1 says an owner pack's declarations use **its** pack id as namespace; §9.4 says "No existing System is changed by S30" | step-30 §§6.1, 7.1, 9.4 | F-S30a-2 |
| A-8 | Step-31 §7.4: "`economy` declares `economy.wealth-band` in its pack's `attributes/`"; "S30's owner updates that example (§19.6)" | `step-31-wealth-prices.md:589-623` | F-S30a-2: the spec says `economy.wealth-band` |
| A-9 | Step-30 §8.2's profile conditions `core.stature`, `core.build` and `core.facial-hair` on `core.natal-sex`, and §4.5 names `natal-sex` as `fixed`; §6.1's 94 rows contain no `natal-sex` | step-30 §§4.5, 6.1, 8.2 | F-S30a-3, Q-S30a-1 |
| A-10 | Step-30 §4.3 item 3 says the seam "adds an optional `type Context`"; the pinned toolchain is Rust 1.97.1, where associated-type defaults are unstable, and a new required associated type would break every `AuthoredSection` impl (seven in `systems/`, one in `tests/acceptance`, and the pinned external `acme-fishing`) | `rust-toolchain.toml`; `authoring/src/section.rs:172-203`; `grep -rln "AuthoredSection for" systems tests` | F-S30a-4: the spec states the seam's **semantics**; its Rust form is S30-b's (a sibling trait) |
| A-11 | A System Pack is a Rust crate; the loader reads data from World Packs and Entity Packs only. S31's notices pack and `economy` declare attributes "in its pack's `attributes/`" | `worldpack/src/read.rs:8-35`; step-30 §9.6 N-2; step-31 §7.4 | F-S30a-5: the seam needs a way for a System Pack to ship data; Q-S30b-1 (S30-b) rules its form, and this PR's §3 text follows the ruling |
| A-12 | Step-30 §2's outcome table is malformed: row O-12 continues into backend text, and `AC-S30-14…17` rows sit inside it | step-30:249-265 | F-S30a-6: a parent edit (§11), not this PR's file |
| A-13 | `overall.md` names neither S29 nor S30 nor S31 | `grep -n "S29\|S30\|S31" overall.md` (empty) | the S30 entry is new (C5), coordinated with the primary |
| A-14 | Step-26 §7.4b.1 states C-1…C-7 as `ARC-133`'s consumer contract | `step-26-cast.md:942-951` | `ARC-123`'s Consequences cite C-1…C-7 by step section; `ARC-133` is not this PR's |

### 3.2 Findings and their resolution

- **F-S30a-1. `ARC-91` and `ARC-74` are not on `main`.** `ARC-125` is "a dated amendment to `ARC-91` item 3".
  **Resolution (bounded).** If #206 has merged by C4, C4 writes `ARC-125` and a dated note under `ARC-91` pointing
  to it. If not, C4 writes `ARC-125` in full (it states item 3's amended text itself and names `ARC-91` as the
  record it amends "on S26-a's merge"), and §10 R-S26a-a-1 asks whichever of S26-a and this PR merges second to
  add the note under `ARC-91`. The same for `persona`'s fields and `ARC-74` (R-P6a-a-1).
- **F-S30a-2. Two built-ins are other packs' views and cannot be `core.` attributes.** Step-30 §7.1 (an owner
  pack's declarations use its own namespace) and step-31 §7.4 (`economy.wealth-band`) agree; §6.1's rows 32 and
  37 imply `core.occupation` and `core.wealth-band`. **Resolution (bounded by §7.1's rule):** the spec names them
  `employment.occupation` and `economy.wealth-band`, declared by their owners; the built-in catalogue
  (`entities/person-attributes`, the `core` namespace, ≤ 100) declares the other 92 rows plus `core.natal-sex` (Q-S30a-1), 93 in all; the table
  stays the list of player-noticeable attributes with their readers. The MatrAIx mapping's target becomes
  `economy.wealth-band ← socioeconomic_band`. **Who makes `employment` an owner** is not settled by the step,
  because §9.4 forbids S30 from changing an existing System: **Q-S30a-2, ruled 2026-10-15: S30-f, as a named
  exception to §9.4** (§14.1); the spec says so in `MODULE_SPEC` §3 and `ARC-120`'s Consequences.
- **F-S30a-3. `natal-sex` is used but not catalogued.** The San Diego profile's conditionals and §4.5's example
  need `core.natal-sex`; §6.1 omits it. **Q-S30a-1.**
- **F-S30a-4. The seam's "optional `type Context`" is semantics, not syntax.** The spec says: "a section may be
  decoded in its owner's context — the owner's decoded data directories and its world configuration — so a value
  is checked against a schema declared in data, at the value's own line and column". It does not prescribe an
  associated type; S30-b's design fixes the form (a sibling trait, `pr-s30b-authoring-seam.md` §5.3).
- **F-S30a-5. System Packs need to ship data.** See A-11. Q-S30b-1, ruled 2026-10-15: compiled in, through
  `SystemPack::SHIPS` / `ships_data!`. `MODULE_SPEC` §3 states it as seam item 4 (§5.3).
- **F-S30a-6.** Step-30 §2's table is malformed (A-12). Listed for the primary (§11).

---

## 4. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-a-1 | Only `docs/CORE_CONCEPTS.md`, `docs/MODULE_SPEC.md`, `docs/DECISIONS.md`, `.structured-coding/plans/mvp0/overall.md`, this file and its handoff change | C5's `git diff --name-only` |
| I-a-2 | Every rule the step states in §§4–11 appears in a specification, and nothing a specification says contradicts the step as ruled, except the findings of §3.2, each stated as decided | the C5 review table (§15): one row per step section |
| I-a-3 | `Attribute` is defined once (`CORE_CONCEPTS` §4.5) and used in one sense; `Component`, `Observation`, `System`, `Entity Pack`, `World Pack` keep their meanings | review; `grep -n "attribute" docs/CORE_CONCEPTS.md` read in full |
| I-a-4 | The kernel learns nothing: no specification says that `kernel/` or `contracts/` holds an attribute word, type or id | review (I-S30-8) |
| I-a-5 | The MatrAIx backend is optional and named only in its own places: `MODULE_SPEC` names it once, as an example of an optional backend pack, with the attribution line | review against step-30 §6.6's isolation list |
| I-a-6 | Every record carries its approval by ruling id and date; superseded rulings stay visible in `DEP-66` | review |

---

## 5. Design: what each document says

### 5.1 `CORE_CONCEPTS.md` §4.5 Attributes (C1)

The text of step-30 §15.1, expanded to a full specification:

1. **Definition.** An attribute is a declared, typed property of a Person. It is one of §§4.1–4.3's classes:
   identity (`birth`), a trait (`stature`, `chronotype`), or a view of mutable state another System owns (a wealth
   band). It is declared as data, and it has exactly one owning System.
2. **Not a Component.** A component is one owner's typed state; attributes are the declared vocabulary over that
   state. The `attributes` System Pack holds plain traits in one component (`traits`); an owner pack exposes its
   own state as attributes through a projection (a view, never a copy).
3. **Visibility** (`perceivable`, `learnable`, `private`, `hidden`), stated as step-30 §5.4's table; disclosure is by
   perception only (INV-13); `perceived_as` is what another observer is told.
4. **Mutability** (`fixed`, `evolving`, `derived`), step-30 §4.5's table; a derived attribute is computed, never
   stored or stated (INV-15 for the others: every change cites its cause).
5. **What is not an attribute.** A name (identity, `naming`'s; never sampler input). A memory of what someone said
   about themselves (INV-4; S28's private layer). A look key (`appearance`'s; a renderer binds it, a descriptor
   never selects art, `ARC-125`).
6. **Generation.** A world's people get their unstated attributes from a population profile, sampled
   deterministically from the world's population seed, never from the run's seed; the biography wins (authored
   and ruled values are never re-drawn). Generation is pluggable by backend (`ARC-126`); the default backend is
   MineWorld's.

§15.1's example: `entities  … each with only the components this observer is entitled to — for a Person, the
traits component filtered for this observer (§4.5)`.

### 5.2 `MODULE_SPEC.md` §2 (C2)

- The amended MVP-0 sentence: "MVP-0 reads **item types** and the **data directories an installed System Pack
  owns** (`ARC-121`)." The old sentence is kept struck through with the date.
- A layout block:

  ```text
  person-attributes/            the built-in catalogue (backend `mineworld`; namespace `core`)
    pack.yaml                   id, type: entity-pack, version, mineworld, license (CC0-1.0), authors
    backend.yaml                the backend contract (ARC-126)
    attributes/<group>.yaml     declarations (§4.1 "Attribute declarations")
    masks/{core,appearance}.yaml
  populations-us/
    pack.yaml
    populations/san-diego-county.yaml
  ```

- One paragraph per directory with its owner (`attributes`), its format's home (§4.1), and the rule that an
  Entity Pack may carry a data directory only of a pack the build installs; an unknown directory is refused,
  listing the directories the build knows (`ARC-121`).
- The optional backend example (`entities/attributes-matraix`), with step-30 §6.6's attribution line and the
  statement that nothing outside it names it.

### 5.3 `MODULE_SPEC.md` §3 / §3.1 (C2)

- **Attributes a pack reads or owns.** `reads_attributes: [<qualified id>, …]` (a typed handle obtained at
  install, refused by name: `Missing`, `WrongKind`, `MissingUnderBackend`; a `text` attribute can never be read by
  a System); `proposes: [<qualified id>, …]` (the attributes it may propose changes to, through `attributes`'
  vocabulary, `ARC-26`); `AttributeOwner` (an owner names the attributes it owns, projects each from its own
  component, and seeds their resolved initial values in its own vocabulary). Marked "specified by `ARC-120`,
  `ARC-122`, `ARC-124`; implemented by S30-c".
- **The authoring seam** (`ARC-121`), stated generally: (1) a pack may own data directories in World Packs and
  Entity Packs, collected in composition order and decoded by the owner with line and column; (2) a section may be
  decoded in its owner's context (its decoded data and its world configuration); (3) an owner may resolve across
  every file of the kinds its section is carried by, once, before any per-subject seeding, the seeding order
  unchanged; (4) as Q-S30b-1 rules, a System Pack may ship data files for an owner's directory. Each item names
  its refusal. Marked "implemented by S30-b".

### 5.4 `MODULE_SPEC.md` §4.1 (C3)

- In the person-file block: `attributes:` (a section owned by `attributes`), with an example of a category, a
  quantity, a set and an `allowed:` restriction (step-30 §8.5 item 2).
- **Attribute declarations** (a new paragraph and block): step-30 §5.1's fields table, §5.2's kinds and units,
  §5.6's rendering, §7.1's namespaces and short slugs, §7.2's overrides table.
- **A world's configuration** `configure/attributes.yaml`: step-30 §§6.4, 8.1 (backend, mask, profile, seed, salt,
  lock), with "the population seed is not the run's `--seed`".
- **Population profiles**: step-30 §8.2's format and rules table; §8.3's semantics; `verify` marks and `authored`
  sources (Q-S30-5's release rule).
- **The lock file**: step-30 §8.4, with the precedence order.
- **Masks and backends**: step-30 §§6.4–6.5, including `MaskedOut`.
- **One refusal table** for everything above, columns *Refusal*, *When*, *Names*: `UnknownAttribute`,
  `DuplicateAttribute`, `ForeignNamespace`, `AmbiguousSlug`, `WrongKind` (at a value), `OutOfDomain`,
  `OutOfRange`, `OverrideNotAllowed`, `OwnerNotInstalled`, `DependencyCycle`, `MissingCondition`,
  `BadDistribution`, `BadAdjustment`, `BadMask`, `EmptySupport`, `UncitedDistribution`, `MissingSupport`,
  `Unsatisfiable`, `BackendMalformed`, `UnknownMask`, `MaskNamesUnknownAttribute`, `MaskEntryUnjustified`,
  `MaskedOut`, `UnsupportedSampling`, `ProvenanceMismatch`, `MissingUnderBackend`, `UnknownTopic` (a `vocabulary:`
  value). Each row cites the step section it comes from; a name not in the step (`AmbiguousSlug`,
  `OverrideNotAllowed`, `OwnerNotInstalled`, `OutOfDomain`, `OutOfRange`, `UnknownTopic`) is the step's
  prose given a name, and the row says which sentence.
- **Persistence**: step-30 §11.1's facts table and §11.2's rule (a world without `attributes` is byte-identical).

### 5.5 `DECISIONS.md` (C4)

One record per id, in this order at the end of the file, each with step-30's section as its Design:

| Id | Title (heading) | Approval line |
| --- | --- | --- |
| `ARC-120` | Person attributes are declared data with one owner each; `attributes` owns plain traits; an owner's attribute is a view, never a copy | step approval 2026-10-15 |
| `ARC-121` | The authoring seam: data directories, decoding in the owner's context, owner resolution before seeding | step approval; Q-S30b-1 (S30-b) for item 4 |
| `ARC-122` | The attribute model: kinds, typed values and handles, visibility, mutability, rendering | step approval |
| `ARC-123` | Population profiles and the deterministic stratified sampler; the population seed is not the run seed; the lock file | step approval; Q-S30-2, Q-S30-5 |
| `ARC-124` | The owner interface, with S31's subscriptions as its first user | step approval; §1.2 item 3 |
| `ARC-125` | Apparent descriptors are perceivable attributes; `look` stays opaque (amends `ARC-91` item 3) | Q-S30-1 |
| `ARC-126` | Person-generation backends and masks are data; the MineWorld backend is the default; a backend runs no code | RR-S30-5 (final) |
| `DEP-66` | MatrAIx: adopted as an optional backend pack (licence check, pinned commit and digest, update procedure, attribution, isolation); earlier texts kept | RR-S30-1 … RR-S30-5 |
| `DEP-67` | The sampler is our own; pgmpy, pomegranate, PopulationSim and MatrAIx's sampler code declined; MatrAIx's DAG as an optional `log-linear` mode | Q-S30-9 |
| `DEP-68` | Population data sources: ACS and NHANES (public domain), the Monk Skin Tone scale (CC BY 4.0) | step approval |

Each record's Options table is the step's (§4.1 for `ARC-120`; §8.6 for `DEP-67`; §6.4's rejected alternatives
and RR-S30-1…5 for `ARC-126`/`DEP-66`). F-S30a-2's names (`economy.wealth-band`, `employment.occupation`) and
Q-S30a-1/2's rulings are applied in the records' text.

---

## 6. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Static | unique section numbers (`check_doc_headings.py`); unique decision ids (`check_decision_ids.py`); the docs CI layer | each commit; CI `docs` |
| Review (LLM, the implementer's own; the primary's) | I-a-2 … I-a-6: every step rule present, terminology fixed, no contradiction left unrecorded | C5's review table |
| Unit, Gate 1, Gate 2 | N/A: documentation only; nothing executes | — |

---

## 7. Acceptance criteria (fixed before measuring)

| ID | Criterion |
| --- | --- |
| AC-a-1 | `python3 scripts/ci_layer.py docs` PASS on the final head (the canonical check), and CI's `docs` layer green |
| AC-a-2 | The C5 review table has one row for each of step-30 §§4, 5, 6, 7, 8, 9, 10, 11, 12 naming where the specification states it, or `N/A` with the reason (for example §13, the PR split, which is plan, not specification) |
| AC-a-3 | `ARC-120` … `ARC-126` and `DEP-66` … `DEP-68` exist, each with Problem, Options, Choice, Consequences and Revisit, and `ARC-127` … `ARC-129`, `DEP-69` do not |
| AC-a-4 | F-S30a-1 … F-S30a-5 are each decided in the text, as §3.2 says, and Q-S30a-1/2 as ruled |
| AC-a-5 | I-a-1's diff scan lists only the files it names |

---

## 8. Commit plan

Each commit tracks implementation, validation and review separately; `[x]` needs the work and its evidence.

### 8.0 The procedure for every commit

Work in `/Users/yuema137/mineworld-worktrees/impl-s30-a`. Re-read the step sections a commit cites and the whole
section of the document you edit before editing it. Validate with the two doc scripts. Review against the step,
answering the commit's questions in §15. Stage by path; commit with the message given and the attribution line.

#### C0 — Base and ledger

- **Files:** this file (§15), `.structured-coding/plans/mvp0/handoff-s30a.md` (new).
- Steps: record the base; confirm the ids free (`git grep`); record whether #206 (S26-a) and P6a have merged
  (F-S30a-1's branch); record whether the primary has written S30 into `overall.md` (A-13); confirm Q-S30a-1,
  Q-S30a-2 and Q-S30b-1 are ruled in §14.1 / S30-b's §14.1 (absent → stop for the affected commit).
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: the base and every ruling quoted.
- **Commit message:** `docs(plan): S30-a — open the ledger at the base`

#### C1 — `CORE_CONCEPTS.md`: the term

- **Files:** `docs/CORE_CONCEPTS.md`.
- Steps: §4.5 as §5.1; the §4.2 pointer; the §15.1 example line.
- [ ] Implementation. [ ] Validation: `check_doc_headings.py`. [ ] Review: `Attribute` is not used for a
  `Component`; INV-4, INV-12, INV-13, INV-15 cited; nothing implies kernel knowledge.
- **Commit message:** `docs(spec): CORE_CONCEPTS — a Person's attributes, defined (S30-a)`

#### C2 — `MODULE_SPEC.md` §§2–3: packs

- **Files:** `docs/MODULE_SPEC.md`.
- Steps: §5.2 and §5.3; the seam text exactly as S30-b's design states it (`pr-s30b-authoring-seam.md` §5) and
  Q-S30b-1 rules it.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: the seam paragraph contains no attribute word; the
  MatrAIx example carries the attribution line and its optionality; F-S30a-2's names used.
- **Commit message:** `docs(spec): MODULE_SPEC — Entity Pack data directories and the owner seam (S30-a)`

#### C3 — `MODULE_SPEC.md` §4.1: what a world author writes

- **Files:** `docs/MODULE_SPEC.md`.
- Steps: §5.4, every block and the refusal table.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: every refusal of step-30 §§5, 6.4, 6.5, 7, 8 is in
  the table; `population` is never used as a key name (F-S30-8); every example value lies inside its declared
  domain.
- **Commit message:** `docs(spec): MODULE_SPEC — attributes, profiles, masks and the lock in a World Pack (S30-a)`

#### C4 — The decision records

- **Files:** `docs/DECISIONS.md`.
- Steps: §5.5's ten records; F-S30a-1's `ARC-91` handling.
- [ ] Implementation. [ ] Validation: `check_decision_ids.py`. [ ] Review: each record's Options are the step's;
  approvals cite ruling ids and dates; `DEP-66` keeps the superseded texts.
- **Commit message:** `docs(decisions): ARC-120 … ARC-126, DEP-66 … DEP-68 — person attributes (S30-a)`

#### C5 — `overall.md` and close-out

- **Files:** `.structured-coding/plans/mvp0/overall.md`, this file (§15), the handoff.
- Steps: the S30 entry (step-30 §15.6) or its reconciliation; the AC-a-2 review table in §15; `python3
  scripts/ci_layer.py docs`; push `docs/pr-s30a-attribute-specs`; `gh pr create`; CI on the exact head.
- [ ] Implementation. [ ] Validation: AC-a-1. [ ] Review: AC-a-2 … AC-a-5.
- **Commit message:** `docs(plan): S30-a close-out — overall.md S30 entry`
- **Then:** READY FOR OPERATOR REVIEW — DO NOT MERGE (the standing design-doc rule lets the primary merge after
  review; this session never merges).

---

## 9. R-LOAD-1

Documentation only: the two doc scripts and `ci_layer.py docs` run locally; no build, no test suite. CI on the
exact head is the gate.

---

## 10. Requirements this PR places on other PRs

- **R-S26a-a-1 (S26-a or this PR, whichever merges second).** A dated note under `ARC-91` item 3: "amended by
  `ARC-125` (2026-10-15)".
- **R-P6a-a-1 (P6a or this PR, whichever merges second).** A dated note under `ARC-74`: `persona`'s three fields are
  catalogued as `persona`-owned `text` attributes (`ARC-120`), reviewed after S30-f (Q-S30-4).
- **R-S30b-a-1 (S30-b).** The seam's code matches `MODULE_SPEC` §3's text; a divergence found in implementation is
  fixed in the code or recorded as a dated spec correction, never left (`CLAUDE.md` §2.1 rule 4).
- **R-S30e-a-1 (S30-e).** The catalogue declares 93 `core` rows (the 94 of step-30 §6.1, plus `core.natal-sex` per
  Q-S30a-1, minus the two owner-declared views); X-S30-20's cap applies to `core`.
- **R-S30f-a-1 (S30-f).** `employment` becomes the owner of `employment.occupation` (Q-S30a-2): a projection of
  `Employment { job }` and a declaration shipped with `ships_data!`; nothing else in `employment` changes.
- **R-S31-a-1 (S31).** `economy.wealth-band` as `ARC-124` states it.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `step-30-person-attributes.md` §2 | repair the outcome table (F-S30a-6). **Applied in PR #234, 2026-10-15** |
| `step-30-person-attributes.md` §§6.1, 6.6, 9.6 | `economy.wealth-band`, `employment.occupation` (F-S30a-2); `natal-sex` per Q-S30a-1; row 32's owner per Q-S30a-2. **Notes applied in PR #234 (§§6.1, 9.4); the §6.6 and §9.6 wording stays the primary's to reconcile** |
| `step-30-person-attributes.md` §4.3 item 3 | "optional `type Context`" → "decoded in its owner's context" (F-S30a-4). **Note applied in PR #234** |
| `step-31-wealth-prices.md` §19.6 | acknowledged: S30's example updated |
| `overall.md` | S30-a merged |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S30-a — person attributes: specifications and decision records
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s30a-attribute-specs.md
RELATED / BINDING DOCS:  step-30-person-attributes.md (§§0, 4–13, 15, 16); pr-s30b-authoring-seam.md §5;
                         step-26-cast.md §7.4b; step-31-wealth-prices.md §§7.4, 9; overall.md; CLAUDE.md §2
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s30-a, its own, held by one session
BRANCH:                  docs/pr-s30a-attribute-specs, created from origin/main
START CONDITION:         this design frozen and the primary's dispatch (step-30 §13.3: on step approval,
                         approved 2026-10-15)
IMPLEMENTATION BASE:     origin/main at C0; C0 records it
APPROVED SCOPE:          §2.1 items 1–6
FROZEN INVARIANTS:       §4 I-a-1 … I-a-6; step-30 I-S30-1 … I-S30-10 as specified text
SEQUENCE:                C0 … C5 (§8)
IMPLEMENTER:             Opus-class (step-30 §13.1); reviewer: the primary session
ALLOWED COMMANDS:        python3 scripts/check_doc_headings.py, check_decision_ids.py, ci_layer.py docs; git;
                         gh (pr create/view; never merge); grep; sed -n
NEVER:                   any code, world or test edit; python3 -c; sed -i; heredoc writes; curl; git add -A
MATERIAL STOPS:          a rule the step states that the specification cannot express without changing it;
                         Q-S30a-1, Q-S30a-2 or Q-S30b-1 unruled when its commit is reached; a conflict with S31's
                         or S26's frozen text that §3.2 does not resolve
PLATFORMS:               N/A (documentation; CI's docs layer)
VALIDATION BUDGET:       the doc scripts, unrestricted. Gate 1 and Gate 2: NOT REQUIRED
LIVE DOCUMENTATION:      this document (§15)
HANDOFF:                 .structured-coding/plans/mvp0/handoff-s30a.md
ENDPOINT AUTHORITY:      implementation + local validation: authorized   (task brief 2026-10-15: "endpoint: PR
                                                                          ready for operator review; no merge")
                         semantic commits: authorized                     (working rules §14)
                         branch push: authorized                          (working rules §21)
                         PR creation / update: authorized                 (working rules §21)
                         CI repair to review readiness: authorized        (working rules §21)
                         endpoint: the PR READY FOR OPERATOR REVIEW
                         merge: NOT authorized for this session (working rules §22); the standing design-doc
                         rule lets the primary merge after review
POST-MERGE SYNC OWNER:   primary: step-30, overall.md (beyond C5's entry), the coordinated steps; this session:
                         this ledger
STOP CONDITION:          S30-a READY FOR OPERATOR REVIEW — DO NOT MERGE
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-a-1 | The specification fixes a detail S30-b … S30-k then find wrong | the text states semantics and refusals, not Rust forms (F-S30a-4); a later correction is a dated note, reviewed |
| R-a-2 | `MODULE_SPEC.md` is edited by S29-a (§4.3) and S31 at the same time | separate sections; whichever merges second rebases; `check_doc_headings.py` catches a number collision |
| R-a-3 | `ARC-91` lands after this PR and its note is forgotten | R-S26a-a-1, listed in both PRs |
| R-a-4 | The 94-row table is read as 94 `core` declarations | F-S30a-2's wording in §2 and `ARC-122`; R-S30e-a-1 |

---

## 14. Operator questions (each with a recommendation; questions the step settled are not re-asked)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S30a-1 [operator — the built-in catalogue, RR-S30-3]** | `natal-sex` conditions stature, build and facial hair in the San Diego profile (step-30 §8.2) and is §4.5's example of a `fixed` attribute, but it is not among §6.1's 94 built-ins (F-S30a-3). Add it, or reword the profile to condition on `body-form`? | **Add `core.natal-sex` as a built-in** (`category`: female, male; `private`, `fixed`; reader C: the profile parent of stature, build and facial hair, like `ancestry`, row 26). The cited sources (ACS, NHANES) tabulate by sex, not by body form, so conditioning on `body-form` would no longer match the data the figures come from. The cap still holds: 93 `core` rows of ≤ 100 |
| **Q-S30a-2 [operator — `employment` as an owner; step-30 §9.4]** | `occupation` (row 32) is a view of `employment`'s state, so `employment` must implement the owner interface and declare `employment.occupation`. Step-30 §9.4 says no existing System is changed by S30, and no PR is assigned to change `employment` (F-S30a-2). Who adds it? | **S30-f, as a named, reviewed exception to §9.4**, because S30-f ships `occupation`'s readers (ME, OV-learnable). It is the smallest change: `employment` gains a projection of its own `Employment { job }` to the job's label and an `attributes/` declaration, and nothing else in `employment` changes. The alternative — leave `occupation` out until `employment` is next opened for its own reasons — leaves a reader without its attribute at the release, which RR-S30-2 forbids |

### 14.1 Rulings, 2026-10-15 (relayed by the coordinator; binding)

| Question | Ruling | By |
| --- | --- | --- |
| Q-S30a-1 | **Add `core.natal-sex` as a built-in** (`category`: female, male; `private`, `fixed`; reader C). Applied: F-S30a-3, R-S30e-a-1 (93 `core` rows), step-30 §6.1's dated note | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S30a-2 | **`employment.occupation` is added by S30-f, as a named exception to step-30 §9.4.** Applied: F-S30a-2, R-S30f-a-1, step-30 §9.4's dated note | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Q-S30b-1 (S30-b's, binding here for `MODULE_SPEC` §3) | **Declarations are compiled in through `SystemPack::SHIPS` / `ships_data!`.** Applied: F-S30a-5, §5.3 seam item 4 | Ruled 2026-10-15 (operator answered directly in the primary session) |
| Freeze | "同意冻结并实施 S30-a/b" | Operator, directly in the primary session, 2026-10-15 |

No question remains open.

---

## 15. Ledger (live during implementation)

### Commit ledger

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| C0 base and ledger | this commit | [x] base, ids, merge states and rulings recorded below; handoff opened | [x] `python3 scripts/ci_layer.py docs`: PASS (3 commands, 0.1 s) | [x] every ruling quoted from §14.1 and S30-b §14.1; branch of F-S30a-1 chosen from evidence |
| C1 `CORE_CONCEPTS` §4.5 | this commit | [x] §4.5 "Attributes" (definition and the three classes, qualified ids, one owner; not a Component; kinds; visibility table; mutability table; generation; what is not an attribute; kernel ignorance); §4.2 pointer; §15.1 example paragraph | [x] `check_doc_headings.py`: "240 numbered sections across 33 documents, none duplicated" | [x] `grep -n -i attribute docs/CORE_CONCEPTS.md` read in full (29 lines): `Attribute` never names a Component; `traits` is named as the component that holds plain traits; INV-4, INV-7, INV-12, INV-13, INV-15 cited; nothing places attribute vocabulary in the kernel or contracts; F-S30a-2's `economy.wealth-band` and `employment.occupation` used |
| C2 … C5 | — | [ ] | [ ] | [ ] |

### C0 — the base (recorded 2026-10-15)

- **Session.** Implementation session (Opus), dispatched by the primary session on 2026-10-15.
- **Worktree and branch, as dispatched.** `/Users/yuema137/mineworld-worktrees/impl-s30a`, branch `mvp0/pr-s30a`,
  from `origin/main`. §8.0 and §12 name `impl-s30-a` and `docs/pr-s30a-attribute-specs`; the dispatch's names
  are used (bounded deviation D-a-1 below). `git worktree list`: no other worktree holds this branch.
- **Implementation base.** `origin/main @ 8c5d3598` (Merge pull request #234, `docs/pr-designs-s29a-s30ab`: S30-a
  and S30-b DESIGN FROZEN, step-30 notes applied). The audit base of §3 was `e6030a36`; the step text this PR
  reads is the one #234 left.
- **Ids free.** `git grep -n "^## ARC-91\|^## ARC-74\|^## ARC-12[0-9]\|^## DEP-6[6-9]" origin/main --
  docs/DECISIONS.md` printed nothing: `ARC-120` … `ARC-129` and `DEP-66` … `DEP-69` are free, and `ARC-91` and
  `ARC-74` are absent.
- **F-S30a-1's branch.** S26-a (#206, `mvp0/pr-s26a`) is **OPEN** (`gh pr list`), and P6a's `ARC-74` is not on
  `main`. C4 therefore writes `ARC-125` in full, stating `ARC-91` item 3's amended text itself and naming
  `ARC-91` as the record it amends on S26-a's merge; R-S26a-a-1 and R-P6a-a-1 stand for whichever merges second.
- **`overall.md` (A-13).** Superseded by the tree: `overall.md` §"S26–S33" already lists S30 (row "S30 …
  APPROVED 2026-10-15 … person attributes and dependency-aware population sampling") and states that S29–S33 are
  MVP-0 and release-blocking. C5 is therefore a **reconciliation** (§2.1 item 6), not a new entry.
- **Rulings confirmed.** Q-S30a-1 ("Add `core.natal-sex` as a built-in"), Q-S30a-2 ("`employment.occupation` is
  added by S30-f, as a named exception to step-30 §9.4") and, in `pr-s30b-authoring-seam.md` §14.1, Q-S30b-1
  ("Declarations are compiled in through `SystemPack::SHIPS` / `ships_data!`") — each "Ruled 2026-10-15 (operator
  answered directly in the primary session)". No commit is blocked.
- **Scope added by the dispatch (D-a-2).** The primary's dispatch makes one parent reconciliation part of this
  PR: step-30 §§6.6 and 9.6 still name `core.wealth-band` / `core.occupation`; they become `economy.wealth-band`
  / `employment.occupation` with a dated note (F-S30a-2). §11 lists that edit as the primary's; the primary
  delegated it here. It is commit C4b, and I-a-1's file list gains `step-30-person-attributes.md` for it.

### AC-a-2 review table (one row per step section)

(none yet)

### Deviations and findings

F-S30a-1 … F-S30a-6 (§3.2), recorded at design time.

- **D-a-1 (bounded).** Deviation: worktree `impl-s30a` and branch `mvp0/pr-s30a` instead of §12's `impl-s30-a`
  and `docs/pr-s30a-attribute-specs`. Reason: the primary's dispatch names them. Impact: none on scope; the PR's
  head branch is `mvp0/pr-s30a`. Validation: `git status` in the worktree.
- **D-a-2 (bounded, primary-authorized).** Deviation: `step-30-person-attributes.md` is edited (§§6.6, 9.6, and
  the §4.5 example the same finding touches), which I-a-1 did not list. Reason: the primary's dispatch, which
  owns §11's parent edits, assigned the reconciliation to this PR. Impact: one extra file, one extra commit
  (C4b); no frozen invariant changes. Validation: C5's `git diff --name-only`.
