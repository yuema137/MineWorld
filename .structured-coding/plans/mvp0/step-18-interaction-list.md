# Step 18 — The World's Interaction List: interactions between any entities, as content and pluggable code

**Role:** the step design for one concept that replaces two: **S17 — Physics list** and **S18 —
Configurable rules** (`overall.md`, "Physics list and configurable rules (operator, 2026-10-08)"). It
records the operator's broadened requirement, an audit of every pack's hard-coded rules, a reuse
comparison, the unified design, a PR split replacing PL-a … PL-d and S18, invariants, risks and
questions (QIL-1 …). It holds no frozen PR design.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md); siblings: [`step-11-bodies.md`](step-11-bodies.md)
(S15), [`step-16-packages.md`](step-16-packages.md) (S16), [`step-17-cognition.md`](step-17-cognition.md)
(S10), [`step-12-server.md`](step-12-server.md) (S11-C perception).
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing here authorizes implementation.
Edits to `overall.md`, `docs/DECISIONS.md` and the specifications are proposed in §9 and §10 and not applied.
**Authority.** This document is the one authority for the merged step.
[`step-18-physics-list.md`](step-18-physics-list.md) (S17, frozen at step level) is **superseded** by it.
Its header says so. The following parts are incorporated by reference **as the design of the `bodies`
section**, unchanged except where §4.9 here says otherwise:
- §2.1 (audit of bodies' constants and rules);
- §3 (the physics reuse comparison);
- §4.3–§4.5 (the physics document, parameters and engine constants, resolution);
- §4.7 (interaction kinds and consequence packs);
- §4.9 items 3–7 (determinism).

Every other part of that file is replaced here.
**Branch:** `plan/interaction-list` from `main @ 21f96ff`, worktree
`/Users/yuema137/mineworld-worktrees/plan-physics-list`, one writer.
**Source audited:** `main @ 21f96ff` (12c merged: `systems/bodies` VERSION 2).
**Decision ids:** the numbers already assigned to S17 and S18 are reused (§9): ARC-61 (configuration
seam), ARC-62 (extension catalogs), ARC-63 (the Interaction List), ARC-64 (entity classes), ARC-65
(consequence routing), DEP-28 (build our own, on Cedar's semantics). The primary session confirms them.

## Why a new file

The concept changed: the old file's title, vocabulary ("body class") and scope (bodies only) would be
wrong in place. A successor file plus a one-line "superseded" header in the old one keeps the frozen
record readable and leaves exactly one authority.

---

# 1. The requirement

## 1.1 Verbatim (operator, 2026-10-08, second statement)

> 我们这里说的physics list，并非只是物理规律，而是不同的object之间的相互作用，比如，人物是否可以对话，对话内容是否进入npc的biography和history，物品是否可以交易（改变人物归属）等等，是广义的虚拟object之间的相互作用

```text
R-IL-1  The "physics list" is not only physical law: it is the interactions between virtual objects in
        general — people, items, places, organizations.
R-IL-2  Examples: whether persons can talk; whether what is said enters an NPC's biography and history;
        whether items can be traded (ownership change).
```

The first statement (R-PL-1 … R-PL-4 in the superseded file: modular, a framework, a high-level
abstraction layer, user-added lists) still binds.

## 1.2 Operator decisions that bind this design

From `overall.md` (2026-10-08), kept unchanged:
- everything here is in MVP-0, including the ice/fragile demonstration (QPL-13);
- a list is data, never scripts; new behaviour comes from installed packs (QPL-1);
- the generic `configure:` seam, typed by its owning pack (QPL-2, ARC-61);
- resume is refused on configuration drift (QPL-12);
- no runtime entity creation by a System (QPL-11).

Primary-session rulings kept: the timing of QPL-4 (the seam beside 12d after S16 E-a; bodies after
12d); QPL-10 (presence's `resolution:` line migrates to the generic extension form in the seam PR).

## 1.3 The answer in brief

```text
Classes        Every entity may have a class: a name the World's Interaction List gives to a tag
               selector (`noble` = a person tagged `noble`). Tags already exist on every entity, are
               authored in content, are disclosed, and are "a taxonomy, not state" (contracts/src/entity.rs).
               So classes add no fact and no component. Unclassed worlds are byte-identical.

One list       The World's Interaction List is one document in sections. `classes` is shared. Each other
               section is owned and typed by one System Pack. The SDK provides the schema: rules (which
               actor class may do which action to which target and object class), parameters (ranges,
               capacities, amounts, typed by the pack, bounded by the pack), and consequences (each fact's
               audience, narrowing only, and whether it is biographical). The pack provides the meaning and
               enforces it. Bodies' physics is one section; every pack gets one.

Routing        A list may narrow the Visibility an owner gives a fact, within bounds the owner declares,
               never widen it. It may turn a fact's biographical-ness on or off where the owner allows. The
               fact log always records (INV-11). Who perceives a fact — and so what an NPC can remember —
               follows the envelope's Visibility through S11-C's unchanged audience function.

Enforcement    In the owning pack, through the SDK's pure lookups, on top of the pack's own validation: a
               list can restrict and parameterize, never grant what the pack does not implement. Denials
               are `PermissionDenied`, through the existing affordance mechanism. The kernel is unchanged.

Composition    pack bounds < pack reference list < a reference list the world extends < the world's section
               < a region (place) override. Within one level, the more specific selector wins, and on a
               tie deny wins (Cedar's forbid-overrides-permit). The resolved list is stored in the save.
               Drift is refused at resume.

Proof          A demonstration world shows the operator's three examples by configuration alone, with no
               code edit: two classes of people cannot talk; one class's lines enter no biography; one
               class of item cannot be traded. The ice/fragile physics demonstration stays.
```

---

# 2. Audit — what every pack hard-codes today (`main @ 21f96ff`)

## 2.1 What already exists that the design builds on

- **A-1 Tags.** `contracts/src/entity.rs:18–26`: "Tags are a taxonomy, not state: a system reads them to
  decide whether an entity is of interest to it." `Tags` is a `BTreeSet<Tag>` (fixed order) on every
  entity record, authored in every content file (`tags:`), read through `WorldRead::entity(..).tags()`,
  and disclosed to clients in observations (`systems/presence/src/observe.rs:130, :139`). Item kinds
  carry them (`worlds/market-town/items/apple.yaml`: `tags: [food, fruit]`). They are immutable after
  genesis.
- **A-2 Item kinds and categories (ARC-36).** An item file is a kind; holdings are counts of kinds; a
  kind has a `category` in item's own section, which consumption reads (`EATEN = "food"`,
  `DRUNK = "drink"`, `systems/consumption/src/action.rs:10, :13`). A loose object (bodies) is one Item
  entity whose file carries `body:` (12c).
- **A-3 Visibility is chosen in code per emission** (`contracts/src/event.rs:296`: `Public`,
  `Place(p)`, `Participants`, `Entities(S)`, `SystemInternal`). Counted on `main`: bodies 5, presence 3,
  conversation 2, economy 4, employment 4, group-activity 3, inventory 3, item 1, movement 1, naming 1,
  relationships 1, schedule 1.
- **A-4 Biographical-ness is a compile-time constant per pack** (ARC-29): `pub const BIOGRAPHICAL`,
  aggregated by `Capability::biographical`. Declared on `main`: group-activity (4 facts), relationships
  (`became-acquainted`, `relationship-changed`), employment (`hired`), schedule (`agenda-changed`).
  conversation, presence, movement, naming, economy, inventory, item, consumption, item-transfer and
  bodies declare none. So **`spoke` is not biographical today.**
- **A-5 Perception of facts is presence's `audience` function** (S10 §3.3, owned by S11-C, coordination
  ruling 2). It reads only the envelope's Visibility and presence's own whereabouts fold. Cognition's
  memory ingests only perceived events (S10 §3.8.1). So memory follows Visibility, by construction.
- **A-6 A semantic denial already exists.** `Rejection::PermissionDenied` (`contracts/src/action.rs:221`)
  is one of the five rejections `CLAUDE.md` §4 rule 15 names. Offers carry availability and an
  unavailable reason (`Offer`, ARC-34), so a client already learns "not allowed" from the affordance.
- **A-7 The configuration seam and extension catalogs** are frozen as S17's ARC-61 and ARC-62 (not yet
  implemented).
- **A-8 Resume does not re-read content** (`persistence/src/world.rs:116–121`); ARC-61's drift check is
  the decided remedy for configuration.

## 2.2 Every pack, mapped to the schema

Columns: **Rules** are who may do what to whom. **Parameters** are the numbers. **Consequences** are each
fact's audience and biography. "—" means the pack has none of that kind. Files are under `systems/<pack>/src/`.

| Pack | Actions | Rules a list would govern | Parameters (today's constant → default) | Consequences (fact: today's Visibility; biographical today) |
| --- | --- | --- | --- | --- |
| **presence** | — | — (owner of where people are and of the audience function; not configurable in this step) | — | `arrived`, `person-entered-place`, `stopped-short`: `Place`; no. **Not opened** (QIL-9) |
| **movement** | `move` | `move` by actor class into a **place class** (e.g. `staff-only`), checked when a stride crosses a passage | `MAX_STRIDE` 2 000 mm (`action.rs:23`) | `passage-opened`: `Public`, genesis; no |
| **conversation** | `talk` | `talk`: actor class × target class | `INTERACTION_RANGE` 3 000 mm (`action.rs:16`); `CONVERSATION_GAP` 300 s (`system.rs:29`); `REMEMBERED_AT_MOST` 32 (`component.rs:18`); engine: `UTTERANCE_MAX_BYTES` 480 | `spoke`: `Place` (overhearing), narrowable to `Participants`; biographical: no today, **configurable**. `conversation-started`: `Participants`; configurable. Pack-specific: whether the listener's `Remembered` log keeps the line (`remember`) |
| **relationships** | — (reacts) | `acquaint`: whether a pair of classes forms a relationship at all | `SPOKE_FAMILIARITY` 10, `ACCEPTED_REGARD` 50, `DECLINED_REGARD` −30, `ACTIVITY_FAMILIARITY` 50, `ACTIVITY_REGARD` 20 (`system.rs:22–30`), per class pair; engine: ranges (`component.rs:11–12`) | `became-acquainted`, `relationship-changed`: `Participants`; yes, configurable |
| **group-activity** | `invite`, `accept-invitation`, `decline-invitation`, `join-group-activity`, `leave-group-activity` | `invite`, `join`: actor class × target class | `INVITE_RANGE` 3 000 mm (`action.rs:21`), `INVITATION_LIFETIME` 1 800 s (`component.rs:23`), `ACTIVITY_LENGTH` 3 600 s (`process.rs:11`); engine: `KIND_MAX_BYTES` | its four facts: `Participants` or `Place`; yes, configurable |
| **item** | — | — | — | `item-kind-declared`: `Public`, genesis; no |
| **inventory** | — (constructors) | `hold`: holder class × **item class**: may this holder hold this kind at all (refusing a give, a buy, a production) | `PERSON_CAPACITY` 6 (`admit.rs:18`), per holder class | `stocked`, `items-transferred`, `items-produced`, `items-consumed`: `Entities` or `Participants`; no; narrowable only |
| **item-transfer** | `give` | `give`: actor class × target class × **item class** — the operator's "can this be traded" | `GIVE_RANGE` 3 000 mm (`action.rs:12`) | states inventory's `items-transferred` (inventory's section governs it) |
| **economy** | `buy` | `buy`: actor class × shop **place class** × item class | — (prices and wallets are content) | `funded`, `money-transferred`, `shop-opened`, `wage-unpaid`: `Public`, `Place`, `Participants`, `Entities`; no; narrowable only |
| **employment** | — (process) | — (jobs are content) | engine: `HOUR` | `hired` (yes), `shift-started`, `shift-ended`, `wage-due`: `Entities` or `Participants`; configurable biography |
| **consumption** | `eat`, `drink` | `eat`, `drink`: actor class × item class | `EATEN` "food", `DRUNK` "drink" → `eats: [categories]`, `drinks: [categories]` per actor class | states inventory's `items-consumed` |
| **naming** | — | — | — | `named`: `Public`; no. Gating names on acquaintance stays ARC-31's later decision |
| **schedule** | — | — | engine: `MAX_SEGMENTS` 24 | `agenda-changed`: `Participants`; yes, configurable |
| **bodies** | `kick`, `throw`, `shove` | the physics pair table (`step-18-physics-list.md` §4.3), with classes from tags (§4.9) | the 26 parameters of `step-18-physics-list.md` §4.4 | `object-moved`, `person-shoved`: `Place`; genesis facts `Public`/`Place`; no |

**Findings.**
- **F-IL-1** Three of the operator's examples are each one row: `talk` rules (conversation), `spoke`'s
  biography and audience (conversation), and `give`/`buy`/`hold` rules by item class (item-transfer,
  economy, inventory).
- **F-IL-2** No pack today asks "is this actor *allowed*": every refusal is spatial, a precondition, or
  a capacity. `PermissionDenied` is produced by no pack (grep on `main`). Rules are new behaviour in
  every pack, so each pack's conversion PR adds one lookup in `validate` and its offers.
- **F-IL-3** Facts stated through another pack's constructor (inventory's facts stated by item-transfer,
  economy and consumption) get their audience from the **owner**. So the owner's section governs them,
  and the stating pack cannot.
- **F-IL-4** "Enters an NPC's history" has three distinct meanings in this codebase, and the design must
  keep them apart (INV-4):
  1. the **fact log**, which always records (INV-11);
  2. the **biography**, the objective projection (ARC-29), which is configurable;
  3. **memory**, which is subjective and cognition's. It derives only from perceived facts (S10 §3.8),
     so it is governed through audience, never written by the world. Conversation's `Remembered` log
     (32 lines) is world state, conversation's own, and so a pack-specific consequence.

---

# 3. Reuse comparison for the generalized idea (`REUSE_POLICY.md` §§2, 11–12, 17)

## 3.1 The question

The generalized list must decide three things for a request or a fact, deterministically, inside a
pack's `validate`, its offers or its emission, many thousands of times per simulated day:

```text
(r) rule          is (actor class, action, target class, object class, place) allowed?
(p) parameter     which value of a pack's typed parameter applies to that tuple?
(c) consequence   which audience and which biographical flag does this fact get?
```

The physics-specific comparison (Geant4, Unity, Godot, Rapier, Box2D) stands, incorporated from
`step-18-physics-list.md` §3. Below are the candidates for the general idea. Facts were fetched from
primary pages by a web agent in this session. Points it could not confirm word for word are marked.

## 3.2 The candidates

**1. ECS relationships — flecs** (flecs.dev `Relationships.html`, `ComponentTraits.html`, v4.1;
github.com/SanderMertens/flecs; github.com/Indra-db/Flecs-Rust).
- *What it is.* A relationship is a pair `(Relationship, Target)` on an entity, for example
  `(Likes, Alice)`. Queries use wildcards (`(Eats, *)`). Traits exist: `Exclusive` ("an entity can have
  only a single instance of a relationship"), `Acyclic`, and `IsA` ("Used to express inheritance
  relationships"). The `Symmetric`/`Transitive` wording and IsA's override semantics came through a
  summary only (unconfirmed).
- *Licence and maturity.* flecs is C/C++ under MIT and mature. The Rust binding `flecs_ecs` is MIT but
  "Status: Alpha release": an experimental API with "potential bugs and breaking changes", and `World`
  is `!Send`/`!Sync`.
- *Fit.*
  - (r)–(c): none. flecs is a storage and query model, not a rule language.
  - Adopting an ECS is what `DEP-1` rejected, for unstable handles across save/load, an iteration
    profile MineWorld does not have, and `&mut World` access that would weaken INV-7.
  - Its pair-plus-wildcard query is a good model for our selectors, and IsA is the model for class
    inheritance if one is ever wanted.
- **Verdict: REFERENCE** (wildcard selectors; IsA as the shape of a later class hierarchy, QIL-6). No
  dependency.

**2. RimWorld defs and Factorio prototypes, as interaction tables** (rimworldwiki.com `ThingComp`,
`PatchOperations`; lua-api.factorio.com prototype and data-lifecycle pages; already verified for S17).
- *Fit.*
  - (p) and (c) by analogy: data names behaviour implemented in code, and classes are data mods add.
  - Factorio's staged composition is ordered and recorded.
  - Both formats are bound to their games, and RimWorld's XPath patches and Factorio's Lua stages are
    untyped.
- **Verdict: REFERENCE; ADAPT** "data names code by id" and "ordered, recorded composition". This is
  unchanged from S17.

**3. Cedar** (docs.cedarpolicy.com 4.5: `policies/syntax-policy.html`, `auth/authorization.html`,
`policies/validation.html`, `other/security.html`; github.com/cedar-policy/cedar).
- *What it is.*
  - A policy is `permit` or `forbid` over a mandatory scope (principal, action, resource) with optional
    `when`/`unless` conditions.
  - Hierarchy is expressed with `in` (`principal in Group::"…"`).
  - The decision rule, verbatim: "If any `forbid` policy evaluates to `true`, then the final result is
    `Deny`", else any satisfied permit gives `Allow`, else `Deny`. Its named properties are "default
    deny", "forbid overrides permit" and "skip on error".
  - Schema validation is separate from evaluation, and its soundness is "formally proved".
  - "Cedar policies of a bounded size are guaranteed to terminate, and are effect free"; "Cedar has no
    facilities for I/O".
- *Licence and maturity.* Apache-2.0; written in Rust (`cedar-policy`), "only the safe subset of
  Rust"; production use at AWS.
- *Fit.* The closest semantic fit for (r):
  - principal, action and resource map to actor, action and target;
  - `in` maps to class membership;
  - forbid-overrides-permit is exactly the tie rule this design needs;
  - validating against a schema is what each pack's section type does.
- *Over-fit and determinism, said plainly.*
  - Cedar decides (r) only. Parameters (p) and consequences (c) — two of the three columns — have no
    home in it, so we would build those anyway, beside a second language.
  - `when` conditions over attributes are a small expression language. QPL-1 decided "data only, no
    scripts", and conditions are the first step toward one.
  - Default deny is the opposite of what byte-identity needs, though a single `permit` would emulate
    today.
  - Every evaluation needs an entities set built from world state per request, which is a cost on the
    hottest path (`validate`, offers).
  - Decisions are effect-free and terminating, so not a determinism risk in themselves. "Skip on error"
    silently drops a policy that errors, which a deterministic, refuse-by-name world would rather
    reject at load.
- **Verdict: ADOPT THE SEMANTICS, NOT THE ENGINE** (DEP-28).
  - Our rule grammar is Cedar's scope triple without `when`.
  - Our tie rule is forbid-overrides-permit, and our sections are schema-validated.
  - Revisit the engine itself if attribute conditions ever become a requirement (for example "may
    enter only if regard > 50"). Cedar is then the candidate, not a language of our own (QIL-11).

**4. OPA / Rego** (openpolicyagent.org docs; github.com/open-policy-agent/opa; github.com/microsoft/regorus).
- *What it is.* A Datalog-inspired policy language; OPA itself is written in Go (Apache-2.0). Its
  builtins include `http.send`, `time.now_ns` and `rand.intn`, each fixed within one query and free to
  vary across queries (that last point is our inference; the docs do not word it as a warning). The Rust
  interpreter `regorus` (MIT) is "mostly compliant" with OPA v1.2.0 and does not support every builtin.
- *Fit.* A general-purpose policy language. Over-fit for a class-pair table. Its standard library
  exposes wall-clock time, randomness and network calls, which a replayed world must never reach, so
  every policy would need a builtin allow-list to stay deterministic. In Rust it is a partial
  re-implementation.
- **Verdict: REJECT.** Over-fit, and a determinism hazard unless fenced off builtin by builtin.

**5. Casbin** (casbin.apache.org `supported-models`, `syntax-for-models`; github.com/apache/casbin-rs).
- *What it is.* The PERM metamodel: a model file with `[request_definition]`, `[policy_definition]`,
  `[policy_effect]` and `[matchers]`, covering ACL, RBAC, ABAC, deny-override and priority models. The
  Rust crate `casbin` (Apache-2.0, 2.18.1) evaluates matchers with `rhai`, a scripting engine, defaults
  to Tokio (`runtime-tokio`), and its enforcer "is not thread-safe".
- *Fit.* Its effects (`!some(where (p.eft == deny))`) express our tie rule. But matchers are scripts
  (QPL-1), an async runtime would sit on a pure `validate` path, and model and policy are untyped CSV
  and conf files.
- **Verdict: REJECT.**

**6. Game "interaction matrices"** (Unity's layer collision matrix, Godot's layers and masks; verified
for S17).
- *Fit.* The mental model players and designers already have is a grid of class × class with a cell
  per interaction. Ours is that grid made sparse, with wildcards and typed cells.
- **Verdict: REFERENCE** for how the list is presented: `mineworld interactions <world>` can print the
  resolved matrix (§4.10).

**7. Build our own, narrowly** — an SDK schema (selectors, rules, typed parameters, consequences,
precedence) with pack-supplied meaning.
- *Fit.* The only option that covers (r), (p) and (c) with MineWorld's types: integers only, sorted
  data, pure lookups and a typed section per pack. It adds no dependency.
- *Cost.* Moderate: one SDK module, one lookup per action per pack, and the conversions.
- **Verdict: BUILD, narrowly, with Cedar's semantics** (DEP-28 revised).

## 3.3 Summary

| # | Candidate | (r) rule | (p) parameter | (c) consequence | Determinism | Licence | Verdict |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | flecs relationships | — | — | — | — | MIT (Rust binding alpha) | reference |
| 2 | RimWorld / Factorio | by analogy | by analogy | — | n/a | proprietary | reference; adapt |
| 3 | **Cedar** | **exact** | — | — | effect-free, terminates; "skip on error" | Apache-2.0, Rust | **adopt semantics, not engine** |
| 4 | OPA / Rego | yes | yes | yes | time, rand, http builtins | Apache-2.0 / MIT | **reject** (over-fit, hazard) |
| 5 | Casbin | yes | — | — | scripted matchers, async | Apache-2.0 | **reject** |
| 6 | interaction matrices | presentation | — | — | n/a | various | reference |
| 7 | our own SDK schema | yes | yes | yes | by construction | — | **build, narrowly** (DEP-28) |

**Recommendation.** Build the SDK schema; take Cedar's scope triple, its forbid-overrides-permit rule
and its schema validation as our semantics; keep conditions out (QPL-1); keep Cedar as the named
upgrade path if attribute conditions are ever required. Both failure modes of `REUSE_POLICY.md` §17 are
checked. Nothing commodity is rebuilt (no policy language, no parser beyond `serde-saphyr`). Nothing is
forced: no general policy engine on a hot path, no scripting, no async runtime and no wall clock.

---

# 4. Design — the World's Interaction List

## 4.1 Vocabulary

None of these terms reuses a defined term (`CORE_CONCEPTS.md`, `MODULE_SPEC.md`). They are proposed for
`MODULE_SPEC.md` (a new section, §10), not for the core ontology.

| Term | Meaning |
| --- | --- |
| **entity class** | A name the list gives to a tag selector over one entity type: `noble` = a Person carrying the tag `noble`. Every entity also belongs to the implicit class named by its type (`person`, `item`, `place`, `organization`). Supersedes S17's "body class". |
| **World's Interaction List** ("the list") | One document per world: a `classes` section and one section per System Pack, which that pack types and enforces. |
| **section** | One pack's part of the list: rules, parameters, consequences, regions. |
| **selector** | An entity class, or `*`, in one role of a rule. |
| **role** | A position in an interaction, declared by the owning pack per action and per fact: `actor`, `target`, `object` (an item kind or a loose object), `place`. |
| **rule** | `permit` or `forbid` for (action, role selectors). |
| **parameter block** | A pack's typed numbers (ranges, capacities, amounts, durations), bounded by the pack. |
| **consequence** | For one fact type and role selectors: its audience (narrowed within the owner's bounds) and its biographical flag. |
| **reference list** | A section compiled into a pack: always `default` (today's behaviour), and any named list the pack ships (`winter` from `ice`). |
| **region** | A place whose section entries override the world's for that place. |
| **interaction kind** | Code a pack provides through another pack's extension catalog (ARC-62): bodies' `slide`. |

## 4.2 Entity classes (ARC-64)

```yaml
# interactions/classes.yaml — in priority order: an entity's class is the first entry whose selector
# it matches; otherwise its type's implicit class
- { class: noble,      of: person, tag: noble }
- { class: servant,    of: person, tag: servant }
- { class: commoner,   of: person, tag: villager }
- { class: heirloom,   of: item,   tag: heirloom }
- { class: staff-only, of: place,  tag: back-room }
- { class: guild,      of: organization, tag: guild }
```

- **Membership is read from tags**, which exist on every entity, are authored in every content file,
  are kept in a fixed order, and are "a taxonomy, not state" (A-1). A class therefore needs no fact, no
  component and no owner: it is a name for a question every pack may already ask. A world without
  `classes.yaml` has only the implicit classes, and nothing is seeded.
- **One class per entity, by priority.** Several matching tags resolve by list order, so every lookup is
  a total function. Multiple simultaneous classes and class inheritance (flecs' `IsA`) are QIL-6.
- **Item kinds (ARC-36).** An item file is a kind, and its tags are the kind's. So `heirloom` classes a
  kind, and every holding of it inherits the class, because holdings are counts of kinds. A loose
  object (bodies, 12c) is its own Item entity with its own tags, so it is classed individually. An
  item's `category` (item's own section, read by consumption) is item's state, not a tag. A class never
  selects on it (that would make the list read a pack's state); consumption's categories become
  consumption's parameters instead (§2.2).
- **Immutable.** Tags do not change after genesis, so a class does not change while a world runs. A
  promotion (a servant becoming a guard) needs mutable classes, which is a later design (QIL-7).
- **S17's per-object `class:` in `body:`, `object-classed` and `BodyClass` are withdrawn.** Tags do the
  job with no new fact (§4.9).

## 4.3 Where the list lives in a World Pack (ARC-61's carrier)

```yaml
# world.yaml
interactions:            # optional: names interactions/<section>.yaml; `classes` is optional too
  - classes
  - conversation
  - item-transfer
  - bodies
```

```text
interactions/classes.yaml        the world's classes (§4.2), decoded by the sdk
interactions/<system id>.yaml    that pack's section, decoded straight into the pack's type
```

This is ARC-61's seam, unchanged in mechanism: owner-typed, decoded with line and column, refused by
name (owner not enabled, unknown section, missing file, unknown key, bound exceeded), seeded after
passages and locations and before sections, with a drift check at resume. Only the carrier's name
moves from `configure:` / `configure/` to `interactions:` / `interactions/`, because every configurable
thing in a pack is now part of its section (QIL-2). A world with no `interactions:` key seeds nothing
new.

## 4.4 The SDK schema: the SDK supplies the shape, the pack supplies the meaning (ARC-63)

**What every section looks like** (one YAML shape for every pack):

```yaml
# interactions/conversation.yaml
extends: default                   # optional: a reference list of this pack (or of a provider pack)
default: permit                    # optional: permit (today) | forbid — what no rule matches gets
rules:
  - { action: talk, actor: noble,    target: commoner, effect: forbid }
  - { action: talk, actor: commoner, target: noble,    effect: forbid }
parameters:
  - { range: 3000, gap: 300, remember: 32 }                 # unscoped: the section's base
  - { actor: guard, range: 6000 }                           # scoped: the fields it names
consequences:
  - { fact: spoke, actor: servant, biography: off }
  - { fact: spoke, audience: participants }                 # nobody overhears, in this world
  - { fact: spoke, actor: servant, remember: off }          # a pack-specific consequence knob
regions:
  library:                                                  # a place key
    parameters: [ { range: 1000 } ]
    consequences: [ { fact: spoke, audience: participants } ]
```

**What a pack writes, in Rust** (in `mineworld-sdk`, module `interactions`):

```rust
/// Implemented by a System Pack that has a section. The SDK decodes, resolves and looks up; the pack
/// declares what its section may say and enforces the answers.
pub trait InteractionSection: SystemIdentity {
    /// The pack's typed parameter block, generated by `parameters!` with a bound and a default per
    /// field, and its all-optional "partial" twin for scoped entries.
    type Parameters: Parameters;
    /// Pack-specific consequence knobs (e.g. conversation's `remember`), typed; `()` for none.
    type Knobs: Knobs;
    /// Each action: its roles and whether a region may scope it.
    const ACTIONS: &'static [ActionDecl];
    /// Each of its own fact types: which envelope entities fill which role, its default audience,
    /// the narrowest audience a list may choose, whether biography is configurable, its default.
    const FACTS: &'static [FactDecl];
    /// The `default` reference list — today's behaviour, written out — and any named lists.
    const REFERENCE: &'static [(&'static str, &'static str)];
}
```

and three pure lookups, each a function of a `WorldRead`, the place and the roles:

```rust
pub fn permits<S: InteractionSection>(w: &WorldRead, at: PlaceId, action: ActionTypeId, roles: &Roles)
    -> Result<(), Rejection>;                       // Err(PermissionDenied) when forbidden
pub fn parameters<S: InteractionSection>(w: &WorldRead, at: PlaceId, roles: &Roles) -> S::Parameters;
pub fn consequence<S: InteractionSection>(w: &WorldRead, at: Option<PlaceId>, fact: EventTypeId,
    roles: &Roles, owner_default: Visibility) -> Consequence<S::Knobs>;   // audience, biographical, knobs
```

- **Absent means the reference.** When the world configures nothing for `S`, each lookup returns the
  pack's `default` without touching state: `permits` is `Ok`, `parameters` are today's constants, and
  `consequence` returns `owner_default` and the compiled biographical set. This is what keeps every
  unconfigured world byte-identical (IL-I1), pack by pack.
- **Generated, not hand-written.** An `interactions!()` macro inside `impl SystemPack` (like
  `owns_section!()`) wires the decode, the seeding fact `<pack>-interactions-configured` (the pack's own
  vocabulary, ARC-26), the reduction and the component.

## 4.5 Composition and precedence

```text
L0  the pack's bounds         code; never overridden; a value outside them is refused at load
L1  the pack's `default`      compiled in; today's behaviour
L2  `extends`                 a named reference list of this pack, or one a provider pack ships to it
                              (bodies' `winter` from `ice`); chains ≤ 4, no cycles
L3  the world's section       interactions/<pack>.yaml
L4  a region                  that section's `regions.<place>` entries, for that place only
```

1. **Levels replace by selector.** An entry at a higher level replaces a lower level's entry with the
   same key (action or fact, plus its selectors); entries with different selectors coexist.
2. **Then specificity.** Among the entries that apply to a request, the one naming the most roles
   (non-`*` selectors; a place class counts) wins.
3. **Then forbid wins.** Among rules of equal specificity, `forbid` overrides `permit` (Cedar). For a
   parameter field or a consequence of equal specificity, two different values are **refused at load**,
   naming both entries, so ambiguity never reaches run time.
4. **Default.** No matching rule gives the section's `default`, which is `permit` unless the section says
   otherwise.
5. **Parameters field by field.** A scoped entry overrides only the fields it names.

**Storage and lookup** (determinism, ARC-25): a configured section is seeded as one genesis fact holding
the fully resolved section, with the classes it references copied in. The pack reduces it into a
component `Interactions<pack>` on every Place, holding the base plus that place's region entries. That
gives every lookup a place:
- a request uses the actor's place;
- a reaction uses the fact's place;
- a fact without a place uses the base, which every copy carries.

Lookups are binary searches over sorted vectors. Nothing is looked up from a file at run time.

## 4.6 Enforcement stays in the owning pack

- **Where.** In the pack's `validate`, after the payload, the actor and the target's existence and
  before its spatial requirement: `permits` → `PermissionDenied`. In its offers, through the same call,
  as `available: false` with `PermissionDenied` (ARC-34). At emission: `consequence` chooses the
  audience and the knobs. In reactions: `parameters` (e.g. relationships' regard values).
- **A list cannot grant.**
  - The decoded section names only the pack's declared actions and facts; an unknown name is refused.
  - `permit` is a filter AND-ed with the pack's own validation, so a permitted talk still needs the same
    place and the range.
  - Parameters stay within the pack's bounds.
  - Audience only narrows (§4.7), and biography changes only where the owner allows.
  - A section for a pack the world does not enable is refused, so no list can name an interaction no
    installed pack implements.
- **The kernel is ignorant.** It sees actions, rejections, facts and components it has always seen. No
  kernel, contract or persistence type changes.
- **New interaction kinds** are code from packs, through ARC-62's extension catalogs. Bodies' catalog is
  the only one in MVP-0 (`InteractionKind`, `step-18-physics-list.md` §4.7). Another pack opens its own
  catalog with one `extension` line when a kind first needs plugging in. A wholly new interaction (fish,
  craft) is a new pack's action, and that pack gets its own section automatically.

## 4.7 Consequence routing (ARC-65)

**What a list may govern, per fact type and role selectors:**

| Knob | Range | Who bounds it |
| --- | --- | --- |
| **audience** | narrowing along `Public ⊇ Place ⊇ Participants ⊇ Entities(⊆ participants)`, never below the owner's declared narrowest (e.g. `spoke` never below `Participants`, since the listener must hear) | the owner's `FactDecl` |
| **biography** | `on` / `off` | the owner's `FactDecl.configurable`; a fact the owner marks non-configurable keeps its compiled flag |
| **pack knobs** | typed per pack (conversation's `remember`) | the owner's `Knobs` type |

**What a list may never do.** Widen an audience (that would leak private state, against INV-13's
intent); stop a fact from being recorded (INV-11); change a payload, an owner or a fact's subjects;
route a fact owned by another pack. A fact stated through another pack's constructor gets its
consequence from the owner's section (F-IL-3): inventory's section governs `items-transferred` whoever
states it.

**"Does what is said enter an NPC's biography and history" — answered layer by layer (INV-4, F-IL-4):**

```text
fact log    always. `spoke` is recorded in every world; the log is the truth (INV-11)
biography   the list's `biography` for `spoke` and the speaker's class: the ARC-29 projection asks the
            save's resolved sections instead of only the compiled set
in-world    conversation's `Remembered` (the 32 lines a person was told) — the `remember` knob, the
recall      listener's class; conversation's own state
memory      cognition's, derived only from perceived facts (S10 §3.8.1). The list governs it only through
            audience: `spoke` narrowed to `participants` is not overheard, so bystanders' memories
            never hold it. The listener always perceives what was said to them; a world in which some
            people should not hear others forbids the talk instead. No list writes into a mind
```

**Coordination with S10 and S11-C (no change to either function).**
- The audience is chosen by the owner at emission and carried in the envelope's `Visibility`. S11-C's
  `mineworld_presence::audience::admits` reads only the envelope, so it applies the list's narrowing with
  no code change and no knowledge of lists.
- S10's ingestion reads only what S11-C delivers. I-4 of S10 (live + resumed = `mineworld perceived`)
  holds unchanged.
- **The biography projection changes.** ARC-29's `Capability::biographical` (compile-time) becomes the
  default. `mineworld biography` additionally reads, from the save:
  - the genesis `*-interactions-configured` facts;
  - the genesis journal row's assembled entities, for tags.

  It then asks each owner's `consequence` for the fact's roles. With no configured section it is
  ARC-29 exactly. An amendment to ARC-29's "reads a save's fact table and nothing else" (§10).
- **Cognition's own policy stays the operator's** (`cognition.toml`, S10 §3.7.4). A world cannot tell a
  mind what to forget (QIL-10).

## 4.8 What a list looks like for the operator's three examples

```yaml
# interactions/classes.yaml
- { class: noble,    of: person, tag: noble }
- { class: servant,  of: person, tag: servant }
- { class: commoner, of: person, tag: villager }
- { class: heirloom, of: item,   tag: heirloom }
```
```yaml
# interactions/conversation.yaml — nobles and commoners do not talk; servants' lines are not history
rules:
  - { action: talk, actor: noble,    target: commoner, effect: forbid }
  - { action: talk, actor: commoner, target: noble,    effect: forbid }
consequences:
  - { fact: spoke, actor: servant, biography: off }
```
```yaml
# interactions/item-transfer.yaml — heirlooms cannot be given
rules:
  - { action: give, object: heirloom, effect: forbid }
```
```yaml
# interactions/economy.yaml — nor bought
rules:
  - { action: buy, object: heirloom, effect: forbid }
```

`talk` from a noble to a commoner is refused `PermissionDenied`, and the offer shows it unavailable for
that reason; `spoke` by a servant is recorded and perceived but appears in no biography; `give` and `buy`
of an heirloom are refused. No code, no fork.

## 4.9 The bodies section (S17's design, kept and re-expressed)

What changes from `step-18-physics-list.md`:
- **"Body class" is entity class.** The pair table's `actor`/`body` become the generic roles
  `actor`/`target` and `object`, selected by entity classes from tags. `class:` on `body:`,
  `object-classed` and `BodyClass` are withdrawn.
- **`PlacePhysics` is `Interactions<bodies>`** (§4.5), and S17's `physics:` key on a place's `body:` is
  withdrawn in favour of the section's `regions:`.
- **The physics document's top-level blocks** (`people`, `pairs`, `launch`, `surfaces`, `combine`,
  `classes.*.material`) become bodies' `Parameters` and `Knobs`, scoped by selectors like any other
  section. The values, the bounds, the engine constants and the byte-identity argument of S17 §4.4 are
  unchanged.
- **Interaction kinds and consequence packs** are exactly S17 §4.7 (ARC-62).

## 4.10 Persistence, drift, versioning, tools

1. **The resolved list is state** (§4.5); a save carries it. Replay re-executes genesis from the
   journal (ARC-25).
2. **Drift is refused** (QPL-12). At resume from a World Pack, the host re-seeds every configured
   section and `classes.yaml` and compares them with the save's genesis facts. Any difference is refused
   by name: "the world's interaction list differs from the save's: section 'conversation', rule 2".
   Tag drift is content drift and is not checked (QPL-12's scope).
3. **A pack's `default` is pinned to its `SystemVersion`** by a test per pack. Changing a reference list
   is a version bump, so an old save is refused (A-12 of the old file). Under S16's QSE-7 that is a
   breaking release.
4. **A world's list** travels with the World Pack's semver (S16). A third-party pack's sections and
   reference lists are reached by enabling it and naming it in `requires:`, with nothing new.
5. **`mineworld interactions <world> [--place KEY] [--json]`** prints the resolved list per section and
   per place — the "interaction matrix" (§3 row 6) — so an author sees what precedence produced. It reads
   only the World Pack.

## 4.11 Clients

- **What reaches a client:**
  - affordances, already: an offer is available, or unavailable with `PermissionDenied` (ARC-34);
  - an offer's `SpatialRequirement`, already, which now carries a list's range;
  - tags, already, in observations;
  - facts the client's seat perceives, already, with the list's narrowed audience.
- **What does not:** the list, the classes table, parameters as such, and biography flags. A client
  cannot ask "what is forbidden"; it can only see what it is offered. I-8 and I-S14-1 hold, and S14's
  rule-constant scan needs no new entry, because the list is not a constant in a client.
- 2D and 3D use one semantic path, as before (`AC-13`).

---

# 5. Invariants (proposed; frozen only by the primary session or the operator)

- **IL-I1 Unconfigured is byte-identical, pack by pack.** A world with no `interactions:` produces facts
  and journal inputs byte-identical to `main` before each conversion PR. The references are:
  - both towns' 300-day seed-7 digests (after 12d);
  - bodies-yard's 30-day sha;
  - 12b's `long_run` bytes.

  A converted pack's declaration grows (its configuration fact and component), so its `SystemVersion`
  rises and older saves are refused by name (ARC-25). No fact digest is re-baselined.
- **IL-I2 Explicit default is behaviourally identical.** Naming every pack's `default` explicitly adds
  only the genesis configuration facts. Every later fact is identical, with event ids offset by exactly
  their number (S17's PL-I2, generalized).
- **IL-I3 A list cannot grant.** An unknown action or fact, a parameter outside its bound, a widened
  audience, biography on a non-configurable fact, and a section for a pack the world does not enable are
  each refused at load, by name. A `permit` never bypasses a pack's own refusal.
- **IL-I4 Kernel ignorance.** `kernel/`, `contracts/`, `persistence/` have no diff in any IL PR.
- **IL-I5 Single ownership.** A section's state is its pack's component, written only by its pack's
  reduction. Classes are not state. Consequences are chosen by a fact's owner, never by its stater.
- **IL-I6 Memory only through audience.** S11-C's audience function and S10's ingestion are unchanged.
  No list writes a mind. The fact log records every fact in every world.
- **IL-I7 Deterministic and total.** Integers, sorted data, pure lookups. Equal-specificity ambiguity is
  refused at load, so no tie is broken at run time except `forbid` over `permit`.
- **IL-I8 Drift is refused** at resume for the list and the classes, by name.
- **IL-I9 No list in a client.** Clients see affordances, requirements, tags and perceived facts only.
- **IL-I10 The precursors name no domain.** IL-a's and IL-b's diffs name no pack's vocabulary (talk,
  give, buy, physics …), proven by synthetic test packs and a vocabulary scan.
- **IL-I11 Bodies' engine invariants hold for every list** (S17's PL-I6).

---

# 6. PR split (replaces PL-a … PL-d and S18)

## 6.1 Order

```text
IL-a seam (∥ 12d, after S16 E-a) ─► IL-b SDK schema ─┬─► IL-c bodies refactor (after 12d) ─► IL-d bodies authored ─► IL-i ice/fragile
                                                     ├─► IL-e social: conversation, relationships, group-activity ─┐
                                                     ├─► IL-f ownership: inventory, item-transfer, economy, consumption ─┼─► IL-h the demonstration world
                                                     └─► IL-g movement, employment, schedule ──────────────────────┘
```

IL-e, IL-f and IL-g touch disjoint packs and run in parallel worktrees. Each PR is frozen alone, in a
fresh session, and writes its specification first (`CLAUDE.md` §2.2, §3). Criteria are fixed now,
before measuring (`ARC-23`). Each guarded criterion names the mutation that must break it.

## 6.2 The PRs

| PR | Scope | Integration checkpoint | Adversarial criteria |
| --- | --- | --- | --- |
| **IL-a** Configuration seam and extension catalogs (= PL-a; carrier `interactions:`) | ARC-61, ARC-62 into `DECISIONS.md`; `authoring::PackConfiguration`; `sdk` `extension` lines and `register_extensions`; presence's `resolution:` line migrated (QPL-10); `worldpack`: `interactions:`, `interactions/<id>.yaml`, seeding order, the drift check; one drift-check call per resuming host; test-only `tuning` and `relays` packs | The three worlds' 300-day facts byte-identical; a scratch world configuring `tuning` runs 30 days and states what it says; SIGKILL and resume byte-identical; resume after editing the file refused by name | As PL-a: (1) refusals by name with line and column; (2) mutation removing the drift check → the edited resume is accepted and the test fails; (3) mutation seeding after sections → `tuning`'s section check fails; (4) vocabulary scan (IL-I10); (5) no kernel/contracts/persistence diff |
| **IL-b** The SDK interaction schema (ARC-63, ARC-64, ARC-65, DEP-28) | `mineworld_sdk::interactions`: `InteractionSection`, `parameters!`, `interactions!()`, `classes.yaml` (sdk-decoded), selectors from tags, levels, specificity, forbid-overrides, load-time ambiguity refusal, `permits`/`parameters`/`consequence`, per-Place storage; `mineworld interactions`; the biography projection reading configured sections (ARC-29 amended); `tuning` gains a section (one action, one parameter, one fact) | On a scratch world: a forbidden `tuning` action is `PermissionDenied` and its offer unavailable for that reason; a scoped parameter applies only to its class; a narrowed fact is not perceived by a bystander (through the unchanged S11-C function and `mineworld perceived`); `biography: off` removes it from `mineworld biography` while the fact log still holds it; drift refused | (1) Property test: every lookup on every generated list is total and order-independent of authoring order. (2) Equal-specificity conflicting parameters refused at load, naming both. (3) A widening audience, biography on a non-configurable fact, an unknown action — each refused. (4) Mutation: make `permit` win ties → the forbid-overrides test fails. (5) Mutation: let the biography projection ignore sections → the biography test fails. (6) IL-I10 scan |
| **IL-c** Bodies reads its section; `default` reproduces 12c (= PL-b) | DEP-28's bodies half; bodies' `Parameters`/`Knobs`, `default` compiled in, every §4.4 constant read through the SDK; no authoring yet | IL-I1 on 12b's long run, bodies-yard, both towns after 12d; arm64 = x86_64 (Rosetta); cost ≤ +5 % per swept move and per kick | As PL-b: nudge 300 → 299 in `default` changes `long_run` (the list is read); no §4.4 value survives as a literal; float bit patterns pinned; `(VERSION, default)` pinned |
| **IL-d** Bodies authored: classes, pairs, regions (= PL-c, revised) | bodies' section authored; entity classes from tags in the pair table; `regions:`; materials; `extends` | Test-time copies of bodies-yard: explicit default (IL-I2); `warehouse` (a `heavy`-tagged box blocks, a `light` box flies to a hand-computed landing ± 1 mm, `kick` forbidden for `fixed` is `PermissionDenied`); a region with nudge 150 holds I-11 at 160 mm there; resume after editing refused | As PL-c: totality; refusals; mutation ignoring regions; mutation ignoring classes; PL-I6's 30-day scan under every list in the set |
| **IL-e** Social: conversation, relationships, group-activity | Sections for the three packs: `talk`, `invite`, `join`, `acquaint` rules; their parameters (§2.2); consequences for `spoke`, `conversation-started`, relationships' and group-activity's facts; conversation's `remember` knob; biography configurable where §2.2 says | IL-I1 on both towns (300 days, seed 7); IL-I2; on a test-time copy of social-cafe with two classes forbidden to talk: no `spoke` between them in 30 days, every attempt `PermissionDenied`, and AO-style activity of everyone else holds (fixed before measuring: every seat still talks in every 10-day bucket); a `spoke` with `biography: off` absent from biographies, present in the log | (1) Mutation: `permits` skipped in `talk`'s offers only → the paced controller attempts forbidden talks and the "every attempt refused" count rises: the test reads offers, not just validate. (2) `spoke` narrowed below `Participants` refused at load. (3) relationships never form between a pair whose `acquaint` is forbidden, over 30 days |
| **IL-f** Ownership: inventory, item-transfer, economy, consumption | `hold`, `give`, `buy`, `eat`, `drink` rules with the `object` role; `PERSON_CAPACITY` per holder class; `GIVE_RANGE`; consumption's categories as parameters; inventory's facts' consequences (narrowing only) | IL-I1 on market-town (300 days); on a copy with `heirloom`-tagged kinds: no `items-transferred` of an heirloom by give or buy in 30 days, each attempt `PermissionDenied`; the food loop still closes (every seat eats in every bucket) | (1) A give stated through inventory's constructor cannot choose inventory's audience: a structural test. (2) Mutation: drop the `object` role from `give`'s lookup → heirlooms move and the test fails. (3) capacity per class bounded 1 … 64; 0 refused |
| **IL-g** Movement, employment, schedule | `move` into a place class (`staff-only`) checked where movement decides a crossing; `MAX_STRIDE`; biography configurable for `hired` and `agenda-changed` | IL-I1 on both towns; on a copy with a `staff-only` back room: only `staff`-class people ever arrive there in 30 days; every other attempt `PermissionDenied` at the doorway | Mutation: check the rule after the stride instead of at the crossing → a non-staff arrival is recorded and the test fails by name |
| **IL-h** The demonstration: the operator's examples by configuration alone | `worlds/manor`: nobles, servants, villagers, a shop, heirlooms; `interactions/` exactly as §4.8; README with the operator's checklist; a test comparing `worlds/manor` with its unconfigured twin (built at test time by deleting `interactions/` and the key) | 30 days, seed 7: no talk noble ↔ villager (each attempt `PermissionDenied`); servants' `spoke` absent from every biography and present in the log; no heirloom changes holder; the twin shows all three happening; SIGKILL and resume byte-identical | (1) **No code edit:** the PR's diff is `worlds/manor/**`, its test and Markdown only — mechanically checked like ARC-35. (2) Each of the three examples is shown to need its own list entry: removing one entry makes exactly that example happen in the twin comparison. (3) The operator plays it (milestone hand-off) |
| **IL-i** Ice and fragile (= PL-d) | bodies' `InteractionKind` catalog, `object-removed`, `class_of` (now: a class from tags); packs `ice` and `fragile`; `worlds/rink` | As PL-d | As PL-d, including: the commit installing `ice` and `fragile` leaves `systems/bodies` unedited |

**Checkpoints between PRs.** After IL-b the primary session reviews the schema on its synthetic pack
before any pack converts. After IL-c the byte-identity evidence is reviewed before any bodies behaviour
becomes configurable. After IL-h the operator plays `worlds/manor` and, after IL-i, `worlds/rink`.

---

# 7. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-IL-1 | The schema is too generic for one pack and too narrow for another (bodies' pair table versus a single `talk` rule). | Roles, rules and consequences are uniform; `Parameters` and `Knobs` are the pack's own types. IL-b proves it on a synthetic pack, and IL-c (the richest section) comes before the simple ones. |
| R-IL-2 | A conversion changes an unconfigured result. | IL-I1 against three references per PR; the lookup returns the compiled default without reading state when unconfigured. |
| R-IL-3 | A list starves the paced controller (nobody may talk). | Offers follow the list. Each demonstration fixes an activity criterion before measuring; the remedy is content, never the controller (ARC-34). |
| R-IL-4 | Consequence routing is mistaken for memory control. | §4.7's four layers; IL-I6; the README of `worlds/manor` says what `biography: off` does and does not do. |
| R-IL-5 | Performance: a lookup on every offer for every target. | Sorted vectors, one per place; measured in IL-b and IL-e with a bound fixed in each PR's freeze (≤ +5 % on a 300-day town run). |
| R-IL-6 | Parallel conversions collide in the SDK. | The SDK is complete in IL-b; IL-e/f/g edit only their packs and worlds' test copies. |
| R-IL-7 | Tags are used as classes, and an author adds a tag for display and changes behaviour unknowingly. | A class names its tag explicitly in `classes.yaml`; `mineworld interactions` prints each entity's class; a tag no class names has no effect. |

---

# 8. Questions (QIL-1 …)

**[OM]** marks operator-material questions. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QIL-1 [OM]** | Merge S17 and S18 into one step, **S17 — The World's Interaction List**, retiring S18's number, with this file as the one authority and `step-18-physics-list.md` superseded (its bodies parts incorporated by reference)? | **Yes.** |
| **QIL-2 [OM]** | Rename ARC-61's carrier from `configure:` / `configure/` to `interactions:` / `interactions/` (mechanism unchanged; amends a frozen decision's wording). | **Yes.** Everything a pack lets a world configure is now part of its section. |
| **QIL-3 [OM]** | Entity classes are named tag selectors (no new state, immutable, one class per entity by priority). Alternative: a `classes` pack owning a mutable `Class` component and facts. | **Tags.** Byte-identical, no owner needed, already disclosed, already "a taxonomy, not state". |
| **QIL-4** | A section's default effect is `permit` (today); a section may say `default: forbid`. | **Yes.** |
| **QIL-5** | Precedence: level replaces by selector, then specificity, then forbid-overrides-permit; equal-specificity parameter conflicts refused at load. | **Yes** (Cedar's rule; no run-time tie-breaking). |
| **QIL-6** | Several classes per entity, or class inheritance (`IsA`)? | **Not now.** |
| **QIL-7 [OM]** | Mutable classes (a promotion changes what a person may do). Needs an owning pack and facts. | **Not in MVP-0.** |
| **QIL-8 [OM]** | Consequence routing as §4.7: audience narrows only, within owner bounds; biography on/off where the owner allows; memory only through audience; recording never suppressed. | **Yes.** It is the only form that keeps INV-4, INV-11 and INV-13 and leaves S10 and S11-C unchanged. |
| **QIL-9** | presence, naming and item get no section in MVP-0. | **Yes.** presence owns the audience function itself; naming's gating is ARC-31's later decision. |
| **QIL-10 [OM]** | May a world tell minds what to forget (a "do not remember" hint for cognition)? | **No.** Memory is cognition's and the operator's (`cognition.toml`); a world governs only what is perceived. |
| **QIL-11** | Attribute conditions (`when regard > 50`)? | **Out.** QPL-1 holds; Cedar is the named upgrade path if ever required. |
| **QIL-12** | A forbidden offer is shown unavailable with `PermissionDenied`, not omitted. | **Shown**, consistent with ARC-34 and the existing unavailable reasons. |
| **QIL-13** | In `validate`, `PermissionDenied` comes after existence checks and before the spatial requirement. | **Yes.** |
| **QIL-14** | A configured section is stored as a component on every Place (base + region). | **Yes**, measured in IL-b. |
| **QIL-15** | ARC-29 amended: `mineworld biography` also reads the save's genesis configuration and entity tags. | **Yes.** |
| **QIL-16** | The PR order of §6.1, with IL-e/f/g in parallel. | **Yes.** |
| **QIL-17 [OM]** | `worlds/manor` as the demonstration of the three examples, played by the operator. | **Yes.** |
| **QIL-18** | Converting a pack raises its `SystemVersion` (declaration grows); old saves refused; no digest moves. | **Accept.** |
| **QIL-19** | Movement's place-class rule is checked where movement decides a crossing. | **Yes.** |
| **QIL-20** | Consumption's food/drink categories become its parameters, per actor class. | **Yes.** |

---

# 9. Decision records (numbers reused from S17/S18; drafts, not in `DECISIONS.md`)

| Number | Was | Now |
| --- | --- | --- |
| ARC-61 | ARC-PL-a, configuration seam | unchanged mechanism; carrier `interactions:` (QIL-2) |
| ARC-62 | ARC-PL-b, extension catalogs | unchanged |
| ARC-63 | ARC-PL-c, bodies' physics list | **the World's Interaction List**: sections, the SDK schema, precedence, enforcement in the owning pack; bodies is one section |
| ARC-64 | S18 (configurable rules) | **entity classes from tags** (§4.2) |
| ARC-65 | S18 (configurable rules) | **consequence routing** (§4.7) and the ARC-29 amendment |
| DEP-28 | DEP-PL-a, our own physics format | **our own interaction schema, on Cedar's semantics**; flecs, OPA/Rego and Casbin compared and declined (§3) |

---

# 10. Proposed amendments (text only; applied by the primary session)

- **`overall.md`**:
  - "Physics list and configurable rules": a dated note that S17 and S18 merge into **S17 — The
    World's Interaction List**, pointing here.
  - QIL-1 … QIL-20, once decided.
  - The lanes of §6.1.
  - Operator decision 1's "PL-a to PL-d" reads "IL-a to IL-i".
  - Decision 2's S18 scope is IL-e … IL-g.
- **`step-18-physics-list.md`**: header "Superseded by `step-18-interaction-list.md` (2026-10-08); its
  bodies parts are incorporated there by reference" — applied in this branch, because it only records
  supersession (see this file's header).
- **Specifications, in the PRs that implement them:**
  - `MODULE_SPEC.md` §4, §4.1: `interactions:` and `interactions/`;
  - `MODULE_SPEC.md` §9: the configuration schema, implemented as ARC-61/63;
  - a new `MODULE_SPEC.md` section: the World's Interaction List and its vocabulary (§4.1);
  - `DECISIONS.md` ARC-29: a dated amendment;
  - each converted pack's README: its section.
