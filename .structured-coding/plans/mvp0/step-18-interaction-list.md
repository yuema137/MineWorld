# Step 18 — The World's Interaction List: interactions between any entities, as content and pluggable code

**Role:** the step design for one concept that replaces two: **S17 — Physics list** and **S18 —
Configurable rules** (`overall.md`, "Physics list and configurable rules (operator, 2026-10-08)"). It
records the operator's broadened requirement, an audit of every pack's hard-coded rules, a reuse
comparison, the unified design, a PR split replacing PL-a … PL-d and S18, invariants, risks and
questions (QIL-1 …). It holds no frozen PR design.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md); siblings: [`step-11-bodies.md`](step-11-bodies.md)
(S15), [`step-16-packages.md`](step-16-packages.md) (S16), [`step-17-cognition.md`](step-17-cognition.md)
(S10), [`step-12-server.md`](step-12-server.md) (S11-C perception).
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)` — frozen at step level by the primary session under the operator decisions in `overall.md` "The World Interaction List (operator, 2026-10-08)", which bind and override this document where they differ (notably: the carrier stays `configure:`, QIL-2). Each PR (IL-a … IL-i) is detailed and frozen in turn.
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

---

# 11. PR IL-a — the configuration seam and the extension catalogs (full design; DESIGN FROZEN 2026-10-08)

**Lifecycle:** `DESIGN FROZEN (2026-10-08), primary session` — the primary session's freeze of 2026-10-08: "IL-a is DESIGN FROZEN (2026-10-08), primary session. §11 is accepted as written. QIA-1 to QIA-6 are accepted as recommended." The execution contract (§11.10) is confirmed; the scratch canary (QIA-5) is authorized on condition that it is never pushed and is deleted afterwards. Drafted by the planning session; implemented in a fresh session in its own worktree.

**Binding rulings this section implements** (`overall.md` "The World Interaction List"):
- QIL-2 is overruled. The carrier is `configure:` (ARC-61), the files are `configure/<pack>.yaml`, and
  classes live in `configure/classes.yaml`. So §4.3's `interactions:` reads `configure:` throughout,
  and §4.5's `<pack>-interactions-configured` is that pack's configuration fact.
- QPL-10: presence's `resolution:` line migrates to the generic extension form here, with no shim.
- QPL-12: resume is refused on configuration drift.

## 11.1 Identity, base, approved scope

```text
PR            IL-a — the configuration seam (ARC-61) and the extension catalogs (ARC-62); S17, first PR
base          main @ 0d35d6b (#73 merged; E-a merged as 1a1d08e; 12c merged). Re-audit §11.2 if
              authoring/src, sdk/rust/src, systems/installed, worldpack/src, systems/presence/src/resolve.rs,
              tools/cli/src/{main,run}.rs or tests/acceptance/tests moved — above all if E-b or 12d merged
branch        mvp0/pr-il-a-seam, worktree /Users/yuema137/mineworld-worktrees/plan-physics-list (or a
              fresh one named at freeze), held by the implementing session only
scope         §4.3 (with the carrier `configure:`), §4.10 item 2 (drift), step-18-physics-list.md §4.7's
              catalog paragraph (ARC-62); nothing of §4.4–§4.9 (that is IL-b onward)
merge         a merge commit, never a squash (ARC-5)
```

**Goal.** A World Pack can hand any enabled System Pack a configuration that the pack types, validates
and seeds as its own genesis facts. A resume against changed configuration is refused by name. A
pack-owned trait can be implemented by other packs and listed in the installed set through one generic
line, of which presence's resolvers are the first. No pack in the build configures anything yet, and
every existing world runs byte for byte as before.

**Change set** (every path this PR may touch):

```text
docs/DECISIONS.md                    ARC-61, ARC-62 (new); a dated note on ARC-39 (the line's new form)
docs/MODULE_SPEC.md                  §3.1 (the extension line), §4 model and §4.1 (configure:, configure/),
                                     §9 (configuration schema: implemented by ARC-61)
docs/MVP_STATUS.md                   the arrival-resolution row's wording; one capability row
systems/README.md                    "Adding a pack": the extension line (line 80's paragraph)
authoring/src/configuration.rs       NEW: PackConfiguration, AuthoredConfiguration, DecodeConfiguration
authoring/src/lib.rs                 one module line, re-exports
sdk/rust/src/pack.rs                 SystemPack: CONFIGURATION, CONFIGURATION_FACTS, decode_configuration;
                                     configures!()
sdk/rust/src/installed.rs            `extension` lines replace `resolution:`; register_extensions,
                                     extension_types, configuration, decode_configuration,
                                     configuration_facts
sdk/rust/src/{lib,section}.rs        re-exports, __private additions, the crate doc's line 13
systems/installed/src/lib.rs         the resolution line rewritten as an extension line; its doc paragraph
systems/installed/tests/resolution.rs   rewritten over Capability::extension_types (file name kept: the
                                     seam scan lists it)
systems/presence/src/resolve.rs      one panic message's wording ("resolution: line" → "extension line");
                                     no behaviour, no VERSION change (QIA-3)
worldpack/src/format.rs              WorldManifest.configure; FoundConfiguration
worldpack/src/configure.rs           NEW: reading configure/, ordering, references, the drift comparator
worldpack/src/read.rs                one call, one field, one accessor, step 4c in the module doc
worldpack/src/load.rs                compose() calls register_extensions; initial_facts seeds configuration
                                     after locations and before sections; the module doc's order
worldpack/src/error.rs               the configuration refusals (§11.3 SD-IA-7)
worldpack/src/lib.rs                 `pub mod configure;` and re-exports
worldpack/tests/configuration.rs     NEW: refusals through WorldPack::read on scratch worlds
worldpack/tests/registration.rs      its doc comment's "resolution: line" only (LISTED unchanged)
tools/cli/src/run.rs                 one drift-check call before resume
tools/cli/src/main.rs                one drift-check call before resume (server) and before verify (replay);
                                     the helper saved_genesis
tools/cli/tests/configure.rs         NEW: binary-level refusals; the three worlds' validate unchanged
tests/acceptance/tests/configuration/mod.rs   NEW: test-only packs test-tuning and test-relay(-a,-b)
tests/acceptance/tests/configuration_seam.rs  NEW: genesis, dispatch, persistence, SIGKILL, drift
tests/acceptance/tests/configuration_vocabulary.rs  NEW: the scan of §11.4 IA-8
.structured-coding/plans/mvp0/{step-18-interaction-list,handoff-il-a}.md
sdk/rust/tests/extensions.rs         NEW (QIA-6): the test-local two-catalog installed! of IA-5
tests/acceptance/Cargo.toml          AMENDMENT A-1 (below): two dev-dependencies
Cargo.lock                           AMENDMENT A-1: exactly two lines in mineworld-acceptance's list
```

**Amendment A-1 (primary session ruling, 2026-10-08, during implementation) — a design gap the planning
session missed.** IA-C7's test-only packs live in `tests/acceptance`, but that crate depended on neither
`mineworld-authoring` (`PackConfiguration`, `AuthoredSection`) nor `mineworld-sdk` (`SystemPack`,
`installed!`, `configures!`), and no crate it depends on re-exports them (`git grep "pub use
mineworld_authoring\|pub use mineworld_sdk"`: only `SectionOwner` and `ContentKind`). §11.1 listed only
test files there. Reported as a material stop (Cargo.lock); the ruling, verbatim in substance: "your
smallest revision is approved, as a bounded amendment to IL-a's change set": add both, `{ workspace =
true }`, to `tests/acceptance/Cargo.toml` `[dev-dependencies]` with a comment; `Cargo.lock` changes by
exactly those two dependency-list lines in `mineworld-acceptance` — no new package, no version change,
nothing else — shown with the lock diff; IA-7's PersistentWorld/SIGKILL proof stays in acceptance.
Checks required by the ruling: `ac1_composability` and the I-2 scan pass unedited; AC-1 check 2 sees no
path from tests/acceptance to a market pack through the two new dev-dependencies.

**Paths with no diff:**
- `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `cognition/`;
- every System Pack other than presence's one message and the installed set;
- `worlds/**`; the root `Cargo.toml` (no new dependency);
- `tests/acceptance/tests/{ac1_composability,precursor_vocabulary,seam_vocabulary}.rs`.

**Non-goals.**
- The interaction schema, classes, `configure/classes.yaml`'s meaning, and any pack's section: IL-b
  onward.
- Overriding E-b's licence policy through `configure/packages.yaml`: QIA-1.
- Runtime `World::disable` of a catalog's pack (QB-17).
- Drift in content other than configuration (QPL-12's scope).

## 11.2 Source audit (`main @ 0d35d6b`, 2026-10-08)

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-IA-1** | `installed!` has two arms, one with a hard-wired `resolution: <trait> => [<types>,]` line that expands only `Capability::resolvers()`; `@catalog` generates the rest. | `sdk/rust/src/installed.rs:56–90`, `:68–79` | The generic form replaces the first arm; the catalog arm is extended, not duplicated. |
| **F-IA-2** | `worldpack::compose` calls `mineworld_presence::register_resolvers(Capability::resolvers())` by name; `worldpack/tests/registration.rs` asserts compose registers the listed resolvers once and refuses a different list. | `worldpack/src/load.rs:192–193`; `worldpack/tests/registration.rs` | compose calls `Capability::register_extensions()` instead; registration.rs must pass **unedited in its code** (it reads presence's catalog, not the macro). |
| **F-IA-3** | `Capability::resolvers()` is used only by `systems/installed/tests/resolution.rs` (and its `stray` test set). | `git grep "resolvers()"` | No shim: that test is rewritten over `extension_types()`. |
| **F-IA-4** | The seam scan (`seam_vocabulary.rs`) scans `sdk/rust/src`, `systems/installed/src`, `worldpack/src`, presence and movement for physics words. It admits `bodies` only on lines containing `mineworld_bodies::BodiesSystem` in `systems/installed/src/lib.rs`, and `const LISTED: &str = "bodies";` in `worldpack/tests/registration.rs`. It lists `systems/installed/tests/resolution.rs` among the files it scans, and fails if a listed file is missing. | `tests/acceptance/tests/seam_vocabulary.rs:69–120` | The extension line keeps `mineworld_bodies::BodiesSystem` on one physical line; `resolution.rs` keeps its name; neither admitted line is edited. The scan stays unedited (IA-9). |
| **F-IA-5** | Sections are decoded with `Capability::decode_section(MapAccess)` into `Arc<dyn AuthoredContent>`; genesis seeds them in `initial_facts` after passages and locations, refusing another pack's vocabulary (`seeded`). In-crate tests seed probe sections through `WorldPack::in_memory` labelled with a real capability (F-22's precedent). | `worldpack/src/load.rs:58–90, :295–340, :585–630`; `read.rs:256–276` | Configuration uses the same pattern: a whole-file deserializer, a type-erased value, a probe-labelled in-crate test. |
| **F-IA-6** | `WorldManifest` is `deny_unknown_fields`; its last field is E-a's `mineworld: Option<Compatibility>` (`format.rs:71–74`). `configure:` is refused as unknown today. | `worldpack/src/format.rs:39–75` | One appended field. |
| **F-IA-7** | `WorldPack::read`: manifest → `check_pack_id` → `mineworld:` → `resolve_systems` → keys → content → `check_locations` → `check_passages` → `Self { … }` → `check_sections`. | `worldpack/src/read.rs:86–171` | Configuration is read after `resolve_systems` (it needs the enabled set) and its references are checked with `check_sections` (they need declared keys). |
| **F-IA-8** | Resume does not read content. The hosts that resume or verify from a World Pack are three: `run` (`tools/cli/src/run.rs:239–241`), `server --save` (`tools/cli/src/main.rs:424–426`, `persisted`), `replay` (`main.rs:462–466`, `verify`). Each composes from the pack first. A save's genesis facts are `backend.facts_of(WorldRevision::GENESIS)`, encoded `EventEnvelope`s (`run.rs:267–275` already decodes one). `inspect`, `biography`, `perceived` read the save only. | as cited | Three call sites, one helper; nothing in `persistence/` changes. |
| **F-IA-9** | `Emission` exposes `event_type()`, `record()`, `owner()`, `visibility()` (`kernel/src/system.rs:335–374`); an envelope carries the record it was built from. | as cited | The comparator compares (event type, record, visibility), in order, over the configuration event types. |
| **F-IA-10** | E-b (frozen, in implementation on `mvp0/pr-eb-requirements`) appends `requires` after `mineworld` in `WorldManifest`, moves `read`'s body into `read_with(root, &PackRoots)` with one call after `resolve_systems`, adds `worldpack/src/requirements.rs`, one `PackError::Requirements` variant and `lib.rs` exports, and does **not** touch `load.rs`. Its licence policy is a typed value "a later `configure/packages.yaml` … can override" (FQ-b2). | `step-16-packages.md` (E-b branch) §15.0, §15.2 | §11.5's shared lines; `packages` is reserved here (QIA-1). |
| **F-IA-11** | No `ARC-61`/`ARC-62` heading exists on any `origin/*` branch. | `git show <branch>:docs/DECISIONS.md` over every remote branch, 2026-10-08 | The ids are free. |

## 11.3 Design (SD-IA-1 … SD-IA-11)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-IA-1** | **`authoring::PackConfiguration`**, beside `AuthoredSection`: `trait PackConfiguration: SystemIdentity { type Configuration: DeserializeOwned + Debug + Send + Sync + 'static; const FACTS: &'static [EventTypeId]; fn references(&Self::Configuration) -> Vec<Reference<'_>> { vec![] } fn requires(&Self::Configuration) -> Vec<SystemId> { vec![] } fn seed(&Seeding<'_, '_>, &Self::Configuration) -> Result<Vec<Emission>, Rejection>; }`. `FACTS` are the event types its configuration may seed: the drift comparator's filter. Type-erased as `AuthoredConfiguration` (`owner`, `references`, `requires`, `seed`, `Debug`) with a `DecodeConfiguration<T>` `DeserializeSeed`, as `AuthoredContent`/`Decode` are. | ARC-31's shape, one level up; deserializing is validating. A configuration has no subject, so `seed` takes none. |
| **SD-IA-2** | **`SystemPack` gains** `const CONFIGURATION: Option<SystemId>` (its own id when it configures, else `None`), `const CONFIGURATION_FACTS: &'static [EventTypeId] = &[]`, and `fn decode_configuration<'de, D: Deserializer<'de>>(d: D) -> Result<Arc<dyn AuthoredConfiguration>, D::Error>` whose default refuses "the '<id>' system takes no configuration". `configures!()` defines all three from the pack's `PackConfiguration` impl. | The safe default: a pack that declares nothing is never silently configured. |
| **SD-IA-3** | **`installed!`'s grammar**: `perception: <path>;` then zero or more `extension <trait path> => <register fn path>: [ <type>, … ];` then the pack lines. It expands `Capability::register_extensions()` (each register fn called once, with `vec![Box::new(T::default()) as Box<dyn Trait>, …]`, lines in listed order, types in listed order), `Capability::extension_types() -> &'static [(&'static str, &'static [&'static str])]` (trait path, type names), and in `@catalog`: `configuration(self) -> Option<SystemId>`, `decode_configuration(self, d)`, `configuration_facts(self)`. The `resolution:` arm and `resolvers()` are deleted. | One generic line per catalog; neither the sdk nor `worldpack` names a trait or a pack again. |
| **SD-IA-4** | **The installed set's line** becomes `extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers: [mineworld_bodies::BodiesSystem,];` on one physical line (F-IA-4). Presence's `register_resolvers`, its write-once catalog, `require_registered` and its panics are unchanged except one message's wording. | Byte-identical; the seam scan's admission still matches. |
| **SD-IA-5** | **`world.yaml` `configure:`** is a list of keys, `Vec<ConfigurationKey>` where `ConfigurationKey` is a `SystemId`-validated name. Each names `configure/<key>.yaml`. Order is the author's and is the seeding order. Absent = empty. | As `places:` names `places/<key>.yaml` (§4.3). |
| **SD-IA-6** | **Reserved keys**: `classes` (IL-b) and `packages` (E-b's licence-policy hook). Listing either in IL-a is refused, "reserved for <what>; not configurable in this build". A test holds that no installed pack's id is a reserved key. | No world can give these names another meaning before their owners land (QIA-1). |
| **SD-IA-7** | **Refusals**, each a `PackError` variant naming the key and the file (line and column where a YAML value is involved): `ConfigurationOfUnknownSystem`, `ConfigurationOwnerNotEnabled`, `NotConfigurable`, `ConfigurationReserved`, `ConfigurationListedTwice`, `ConfigurationFileMissing`, `ConfigurationFileNotDeclared` (a `.yaml` in `configure/` not listed), `ConfigurationRequiresSystem { requires, by }`, `ConfigurationNamesUnknownEntity`, `ConfigurationRefusedByOwner`, `ConfigurationStatedAnotherPacksFact`, `ConfigurationDrift { system, saved, here }`. Malformed YAML uses the existing `Malformed`. | Refused by name, never ignored (`MODULE_SPEC.md` §4.1). |
| **SD-IA-8** | **Reading** (`configure.rs`, called from `read` after `resolve_systems`): each key resolved (unknown, reserved, not enabled, not configurable, twice), each file parsed with `serde_saphyr::with_deserializer_from_str` into the owner's type, every `.yaml` under `configure/` declared, every `requires` enabled. References are checked after content, beside `check_sections`, against the declared keys and types. | The order of F-IA-7; each check assumes the previous. |
| **SD-IA-9** | **Seeding** (`load.rs::initial_facts`): passages, locations, **configuration in `configure:` order**, then sections. Each configuration's facts must be its owner's vocabulary and of an event type in its `FACTS`; otherwise refused. A world with no `configure:` seeds exactly what it seeds today. | §4.3: what a section's reduction may check against is reduced before it; byte-identity. |
| **SD-IA-10** | **The drift comparator**: `configure::compare(saved_genesis: &[EventEnvelope], here: &[Emission], facts: &BTreeSet<EventTypeId>) -> Result<(), Drift>` — pure, comparing in order the (event type, record, visibility) of every element whose type is in `facts` (the union of `configuration_facts` over the enabled capabilities). `WorldPack::check_configuration(&self, saved_genesis)` assembles, seeds, filters and compares, returning `PackError::ConfigurationDrift` naming the first differing system and both sides. Each host calls it before `PersistentWorld::resume` / `verify`, through one helper `saved_genesis(&backend)`. | QPL-12, at the moment and in the style of the composition check, with no kernel or persistence type changed. Both directions are drift: configuration added or removed. |
| **SD-IA-11** | **Configuration facts' audience**: owners are told (ARC-61's text and the `configures!` doc) to state them `Visibility::SystemInternal` with no subjects — a world's configuration is nobody's perception and nobody's biography. The test packs do so. | INV-13 by default; IL-b's sections follow it. |

## 11.4 Acceptance (decided before measuring, `ARC-23`)

Rules: every guarded criterion names its mutation. A mutation is applied in the working tree, observed
to fail by name, then reverted; `git status` and `git grep MUTATION` are recorded afterwards. Expected
values are literals from the test's own layout.

```text
IA-1  Byte-identity. With E-IA-0 captured on the base before any code:
        social-cafe and market-town, `mineworld run <w> --headless --seed 7 --days 300`: the printed
        history fingerprint, the fact count and faults 0 equal E-IA-0 (today ad49c723…c64b and
        365b50e0…1d1d; if 12d merges first, 12d's re-baselined values, re-captured);
        bodies-yard, 30 days seed 7: its sha equals E-IA-0;
        12b's long_run second-process bytes: equal E-IA-0;
        the three worlds' `mineworld validate` output: byte-identical.
      M-IA1: make initial_facts seed one empty configuration emission for every enabled capability →
      the social-cafe fingerprint differs (the instrument sees genesis).
IA-2  Configured genesis, in order. In worldpack's in-crate tests (probe-labelled, F-IA-5) and in
      tests/acceptance with test-tuning: configuration facts follow passages and locations and precede
      every section, in configure: order; test-tuning's section reduction refuses a `start` that is not a
      multiple of the configured `step`, and accepts one that is.
      M-IA2: seed configuration after sections → test-tuning's section is refused at load by name.
IA-3  Refusals by name: each SD-IA-7 variant through WorldPack::read on a scratch world (and, for
      unknown system, not enabled, not configurable, reserved, file missing, undeclared file, through
      the real `mineworld validate`), with the file, and line and column for YAML values.
      M-IA3: remove the undeclared-file check → its test fails by name.
IA-4  Drift refused, both directions, both by the comparator and by the hosts:
      (a) configure::compare over test-tuning: changed step, configuration removed, configuration
          added — each Drift naming test-tuning; unchanged — Ok;
      (b) a structural test in tools/cli/tests/configure.rs: each of run, server's `persisted`, and
          `replay` calls check_configuration before PersistentWorld::resume or verify;
      (c) the canary (IA-10): an edited configure/test-tuning.yaml refuses `run` resume and `replay`.
      M-IA4a: compare returns Ok → (a) fails. M-IA4b: remove run.rs's call → (b) fails.
IA-5  Extension catalogs. A test-local installed! with two extension lines (test-relay's trait, listing
      test-relay-a then test-relay-b; and presence's, listing nothing) registers both, each list in
      order, once; registering a different list for one trait refuses, naming both.
      worldpack/tests/registration.rs and tests/acceptance/tests/arrival_resolvers*.rs pass unedited
      (code). No `resolution:` arm or `resolvers()` remains (git grep over *.rs, recorded).
      M-IA5a: register_extensions skips its second line → test-relay is unregistered and the test fails.
      M-IA5b: reverse each list → the order assertion fails.
IA-6  The installed set's guard (resolution.rs, rewritten): every type on an extension line is an
      installed pack, listed once per line; the `stray` set is refused with both messages.
      M-IA6: list a type twice in the stray set's line and drop the duplicate check → the test fails.
IA-7  Persistence: a test-tuning world in tests/acceptance runs requests through World::dispatch and
      PersistentWorld, is killed (SIGKILL, the arrival_resolvers_resume pattern) and resumes byte for
      byte; a second process's replay is identical.
IA-8  Vocabulary. seam_vocabulary.rs passes unedited. configuration_vocabulary.rs: no word of the seam
      scan's physics list, and none beginning talk, spoke, convers, give, buy, sell, trade, eat, drink,
      kick, throw, shove, permit, forbid, biograph, class — in authoring/src/configuration.rs,
      worldpack/src/configure.rs and every test file this PR adds (that scan file excepted).
      M-IA8: plant `// talk` in configure.rs → the scan fails naming file and line.
IA-9  Unchanged guards: tests/acceptance ac1_composability.rs and precursor_vocabulary.rs pass,
      unedited; no diff under kernel/, contracts/, persistence/, server/, worlds/; the root Cargo.toml
      and Cargo.lock unchanged (recorded with git diff --stat).
IA-10 The canary (evidence, never merged; 11a C5's precedent): on a scratch branch, test-tuning is
      installed with its two ARC-33 lines; a scratch world `configure: [test-tuning]` is validated, run
      30 days with --save, killed and resumed byte-identical; then configure/test-tuning.yaml is edited
      and `run` and `replay` are refused with ConfigurationDrift naming test-tuning; `git status` clean
      after the branch is deleted.
IA-11 Full gate on the final head: cargo fmt --check, check, clippy -D warnings, test (workspace),
      both doc scripts.
```

## 11.5 Coordination with S16 E-b — every shared line

Whichever PR merges second performs the merge (`overall.md`). The shared regions, and the rule for each:

| File | E-b's edit | IL-a's edit | Rule |
| --- | --- | --- | --- |
| `worldpack/src/format.rs` `WorldManifest` (after `:74`, `pub mineworld: Option<Compatibility>,`) | appends `pub requires: BTreeMap<PackId, Compatibility>` with its doc | appends `pub configure: Vec<ConfigurationKey>` with its doc | both fields kept; IL-a's after E-b's if E-b merged first, else E-b appends after IL-a's. `FoundConfiguration` is added after `SectionState` (`:126`), away from E-b's lines |
| `worldpack/src/read.rs` `WorldPack::read` body (after `resolve_systems`, `:105`) | body moves into `read_with`; one line `requirements::resolve(&manifest, &systems, roots)?` | one line `let configuration = configure::read(&root, &manifest.configure, &systems)?;` | IL-a's line directly after E-b's, in whichever body holds them (`read_with`) |
| `read.rs` struct `WorldPack` and `Self { … }` (`:71–82`, `:150–163`), `in_memory` (`:256–276`) | one field `composition` | one field `configuration: Vec<FoundConfiguration>` | both fields at the struct's end; `in_memory` gains IL-a's `Vec::new()` (cfg(test) only) |
| `read.rs` accessors (after `seats`, `:221`) | `composition()` | `configuration()` | adjacent |
| `read.rs` module doc's step list (`:8–24`) | "4b requirements" | "4c configuration" | adjacent |
| `read.rs` `check_sections` call (`:170`) | — | followed by `configure::check_references(&pack)?;` | IL-a only |
| `worldpack/src/error.rs` `PackError` (after the last variant, `Composition`, `:397`) | `Requirements { path, refusal }` | SD-IA-7's variants | appended in blocks; no shared line beyond the enum's end |
| `worldpack/src/lib.rs` | exports for `read_with`'s types | `pub mod configure;` (between `content` and `error`) and its re-exports | adjacent `pub use` lines |
| `worldpack/src/load.rs` | not touched (E-b's material stop) | compose and initial_facts | IL-a only |
| `worldpack/src/requirements.rs`, `configure.rs` | new | new | disjoint files |
| `tools/cli/src/main.rs` | `--packs` plumbing at `read`/`read_with` call sites (`:252, :294, :425` per S16 §4.2) | drift calls in `persisted` (`:424`) and `replay` (`:463`) | different lines of the same functions; mechanical |

**The `packages` hook (QIA-1).** IL-a reserves the key. Wiring `configure/packages.yaml` into E-b's
`LicencePolicy` is a framework configuration, not world state: not seeded, not drift-checked. It needs
both PRs, and it lands in IL-b, which already introduces the second framework key (`classes`), or in a
two-line follow-up if the primary session prefers.

## 11.6 Commit plan

### IA-C0 — Design (this section) — docs only

- [x] Implementation: §11, from the audit in §11.2.
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py` (E-IA-d).
- [x] Review: every finding cites a file and line; each guard has a mutation; the E-b shared lines are
  named. Self-review only; the primary session's freeze is pending.

### IA-C1 — Specs before code

**Scope.**
- `docs/DECISIONS.md`:
  - **ARC-61** — *A System Pack may be configured per world, by a file its owner types.*
    SD-IA-1/2/5–11; the reserved keys; the drift rule; the configuration facts' audience.
  - **ARC-62** — *Extension catalogs: a pack-owned trait, implemented by other packs, listed in the
    installed set.* SD-IA-3/4; the rules carried from ARC-39 (write-once, process-wide, pure, inert);
    no shim.
  - A dated note on **ARC-39** (the line's new form).
  - The ids are re-checked on every remote branch before writing.
- `docs/MODULE_SPEC.md`:
  - §3.1 (the extension line replaces `resolution:` at `:195`, `:205`);
  - §4's model and §4.1 (`configure:`, `configure/<key>.yaml`, refusals, reserved keys);
  - §9 (configuration schema → ARC-61).
- `systems/README.md` (`:80`); `docs/MVP_STATUS.md` (row `:31`'s wording, one row).

- [x] Implementation: ARC-61, ARC-62 appended to `DECISIONS.md`; ARC-39 "Note 3" (the line's new
  spelling, no rule changed); `MODULE_SPEC.md` §3.1 (trait table row, presence's line respelled, new
  "Extension catalogs" and "A configurable pack" paragraphs), §4 model (`configure/`), §4.1
  (`configure:` in the manifest, a "Configuration" paragraph with every refusal and drift, the genesis
  order), §9 (configuration schema → ARC-61); `systems/README.md`; `MVP_STATUS.md` (row wording, one
  new row); `handoff-il-a.md` created.
- [x] Validation: `check_doc_headings` → 177 sections / 25 documents, none duplicated;
  `check_decision_ids` → 55 ids distinct (53 + ARC-61, ARC-62). ARC-61/62 free on every `origin/*`
  branch (`git show <b>:docs/DECISIONS.md` over `git branch -r`, after `git fetch`, 2026-10-08: none).
  `git grep "resolution:"` in docs: only ARC-39's historical items (covered by Note 3) and ARC-62's own
  account of what it replaces.
- [x] Review: no defined term redefined — "configuration", "extension catalog", "reserved key" are new
  words in `MODULE_SPEC.md` §3.1/§4.1, not ontology terms; ARC-61's limitations state that content
  drift is unchecked and that configuration is not a rule (§4 constraint 3); the reserved keys and
  QIA-4 guidance are recorded as ruled.

### IA-C2 — `authoring`: the configuration contract

**Scope.**
- `authoring/src/configuration.rs`: `PackConfiguration`, `AuthoredConfiguration`,
  `DecodeConfiguration`, and the blanket impl from `PackConfiguration` to `AuthoredConfiguration`.
- `authoring/src/lib.rs` (module and re-exports).
- Unit tests:
  - decoding through `serde_json` and through `serde_saphyr`, the latter keeping line and column;
  - owner and requires reported;
  - seed refusal propagated.

- [x] Implementation: `authoring/src/configuration.rs` (`PackConfiguration` with `Configuration`,
  `FACTS`, defaulted `references`/`requires`, `seed`; type-erased `AuthoredConfiguration` over a private
  `Held<P>`; `DecodeConfiguration<P>` `DeserializeSeed`); `lib.rs` module line, re-exports and a doc
  paragraph. Unit tests decode through `serde::de::value::MapDeserializer` (D-1): the owner's bound
  refuses with its own message; owner, references and requires are reported; a seed refusal propagates.
- [x] Validation: `cargo test -p mineworld-authoring` → 2 passed (lib), 0 failed; `cargo clippy -p
  mineworld-authoring --all-targets -D warnings` clean; `cargo fmt --all --check` clean (E-IA-2).
- [x] Review: `authoring/Cargo.toml` unchanged (contracts, kernel, serde only); the file's words checked
  against IA-8's lists by grep (one doc word "biography" reworded before commit); the erased trait
  mirrors `AuthoredContent` exactly, minus the section name and subject a configuration does not have.

### IA-C3 — `sdk` and the installed set: configuration in `SystemPack`; the generic extension line (atomic: the macro and its one invocation change together)

**Scope.**
- SD-IA-2 in `pack.rs` and `configures!()`; SD-IA-3 in `installed.rs`, with the `resolution:` arm and
  `resolvers()` deleted; `lib.rs` docs and `__private`.
- `systems/installed/src/lib.rs` per SD-IA-4.
- `systems/installed/tests/resolution.rs` rewritten generically. File name kept; messages "is listed on
  an extension line but is not an installed pack" and "is listed twice on one extension line".
- presence's one message (QIA-3).
- `worldpack/src/load.rs` `compose` → `Capability::register_extensions()` (a one-line edit needed for
  the workspace to compile).
- The test-local two-catalog `installed!` of IA-5, in `sdk/rust/tests/extensions.rs` (NEW; add it to
  the change set).

- [x] Implementation: `pack.rs` — `CONFIGURATION`, `CONFIGURATION_FACTS`, `decode_configuration`
  (default refusal "the '<id>' system takes no configuration") and `configures!()`, plus a unit test of
  the default; `installed.rs` — the grammar is now an entry arm, an accumulator arm per `extension`
  line (a tt-muncher: a `$(extension …)*` repetition followed by `$($Variant:ident …)+` would be a
  macro_rules local ambiguity, since `extension` is itself an ident) and the pack-lines arm, which
  expands `register_extensions()` and `extension_types()`; `@catalog` gains `configuration`,
  `decode_configuration`, `configuration_facts`; the `resolution:` arm and `resolvers()` are deleted.
  `lib.rs`: crate doc, `__private` gains `AuthoredConfiguration`, `DecodeConfiguration`,
  `PackConfiguration`, `DeserializeSeed`, `Deserializer`. `systems/installed/src/lib.rs`: the line,
  one physical line, and its doc paragraph. `resolution.rs` rewritten over `extension_types()` and
  `type_name()`. Presence: one message string. `load.rs` compose → `Capability::register_extensions()`
  and its doc. `registration.rs`: the `LISTED` doc comment only. New `sdk/rust/tests/extensions.rs`.
- [x] Validation (E-IA-3): `cargo check --workspace --all-targets` clean; `cargo test -p mineworld-sdk
  -p mineworld-installed-systems -p mineworld-presence -p mineworld-worldpack` all ok (sdk extensions 1,
  installed resolution 2, worldpack registration 1 — unedited code); `-p mineworld-acceptance --test
  seam_vocabulary` 3, `arrival_resolvers` 7, `arrival_resolvers_unregistered` 2, `arrival_resolvers_resume`
  PASS — all unedited. Clippy `-D warnings` workspace clean; fmt clean. Mutations, each observed then
  reverted (`git grep MUTATION` empty after):
  - M-IA5a (the accumulator arm drops earlier lines, so the first line is skipped) → `extensions` FAILS
    "the first line, in its listed order: left None".
  - M-IA5b (each list reversed before its register call) → FAILS "left [test-relay-b, test-relay-a]".
  - M-IA6 (the duplicate check disabled) → `resolution` FAILS
    `the_guard_sees_a_type_that_is_not_installed_and_one_listed_twice` (2 faults instead of 3).
  `git grep "resolution:\|resolvers()" -- '*.rs'`: no arm, no `resolvers()`; the remaining hits are
  presence's own `registered_resolvers()`, unrelated prose, and two message strings in the frozen,
  unedited `seam_vocabulary.rs` and `registration.rs` (recorded, not edited: §11.7).
- [x] Review: no shim; `extension_types()` returns a `Vec` rather than the designed `&'static [..]`
  because `core::any::type_name` is not `const` (D-4); presence's diff is one string, no `VERSION`
  change; the seam scan's admitted lines (`mineworld_bodies::BodiesSystem` in the installed set,
  `LISTED` in registration.rs) are untouched in substance and still admit (scan passes).

### IA-C4 — `worldpack`: reading `configure:`

**Scope.**
- SD-IA-5–8: `format.rs` (field, `FoundConfiguration`), `configure.rs` (read, check_references),
  `read.rs` (one call, one field, one accessor, the doc step), `error.rs` (variants), `lib.rs`.
- `worldpack/tests/configuration.rs`: every refusal reachable with this build's packs, on scratch worlds
  under `CARGO_TARGET_TMPDIR`, removed by each test (test hygiene, coordination ruling 10).
- In-crate probe tests: the positive decode, references, requires.

- [x] Implementation: `format.rs` — `WorldManifest.configure: Vec<ConfigurationKey>` (appended after
  `mineworld`), `ConfigurationKey` (transparent over `SystemId`, so it validates at its line),
  `FoundConfiguration` after `SectionState`; `error.rs` — the SD-IA-7 variants appended after
  `Composition`, plus `ConfigurationStatedUndeclaredFact` (D-5); `configure.rs` — `read` composed of
  `resolve_keys` (listed twice first — D-6 — then reserved, unknown, not enabled, not configurable),
  `read_files` (missing, then decoded by the owner through `serde_saphyr::with_deserializer_from_str`),
  `check_nothing_undeclared`, `check_requires`; `check_references`; `RESERVED`; `DIRECTORY`. `read.rs`
  — one call after `resolve_systems`, one field, `configuration()` after `seats()`, step 4c in the doc,
  `configure::check_references(&pack)?` after `check_sections`; plus `parse_with` and
  `declared_entities` made `pub(crate)` and a `#[cfg(test)] with_configuration` (D-7). `lib.rs` — `pub
  mod configure;` and the two re-exports. Tests: `worldpack/tests/configuration.rs` (8: unknown,
  reserved ×2, not enabled, not configurable, twice, undeclared file with a README control, a malformed
  key at line 12 column 5, no installed id reserved) on scratch worlds removed on drop;
  `worldpack/src/configure/tests.rs` (3, probe labelled `Presence`: file missing; the owner's bound
  refused at "line 2 column 7" with its own message; a valid decode; requires; references undeclared
  and mistyped).
- [x] Validation (E-IA-4): `cargo test -p mineworld-worldpack` → every target ok (lib 38 incl. the 3
  new, configuration 8, registration 1, refusals 15, …); clippy `-D warnings` and fmt clean. M-IA3
  (`check_nothing_undeclared` not called) → `a_configuration_file_that_is_not_listed_is_refused_naming_the_file`
  FAILS "expected ConfigurationFileNotDeclared, got Ok(..)"; reverted, `git grep MUTATION` empty. The
  three worlds' `mineworld validate` output `cmp`-identical to E-IA-0's. No scratch directory left
  behind (`target/tmp`, `$TMPDIR` checked).
- [x] Review: §11.5's shared lines placed as stated (field after `mineworld`; call directly after
  `resolve_systems`; field and accessor at the ends; `FoundConfiguration` after `SectionState`; error
  variants appended in one block). `load.rs` untouched in this commit. Sizes: `read.rs` 675 (653 on
  base), `error.rs` 583 (418): both past ~500 — `error.rs` is one enum of refusals, each a few lines,
  which is its one responsibility; `read.rs`'s growth is 22 lines, and the new logic lives in
  `configure.rs` (220) by design. Recorded, not split.

### IA-C5 — `worldpack`: seeding and the drift comparator

**Scope.**
- SD-IA-9 in `initial_facts`, and the module doc's order.
- SD-IA-10: `configure::compare`, `WorldPack::check_configuration`.
- In-crate tests: the order (IA-2, M-IA2) with a probe configuration and a probe section labelled as
  F-IA-5; the comparator's four cases (IA-4 a, M-IA4a); another pack's fact refused; a fact type
  outside `FACTS` refused.

- [x] Implementation: `configure.rs` — `seed` (in `configure:` order; owner's refusal →
  `ConfigurationRefusedByOwner`; another vocabulary → `ConfigurationStatedAnotherPacksFact`; a type
  outside the owner's FACTS → `ConfigurationStatedUndeclaredFact`), `Drift`, the pure `compare(saved,
  here, owners: &BTreeMap<EventTypeId, SystemId>)` (a map rather than SD-IA-10's set, so a drift names
  its system — D-10), and `WorldPack::check_configuration(saved_genesis)` (assembles, then compares over
  the enabled capabilities' `configuration_facts`). `load.rs` — one call in `initial_facts` after
  locations, before sections, and the module doc's order. `authoring` — `AuthoredConfiguration::facts()`
  (D-9). In-crate tests (`configure/tests.rs`, probe labelled with real capabilities): the order
  (`arrived`, `probe-configured` ×2 in `configure:` order, `probe-sectioned`; and without configuration
  exactly `arrived`, `probe-sectioned`), the three seeding refusals, the comparator's four cases.
- [x] Validation (E-IA-5): `cargo test -p mineworld-worldpack -p mineworld-authoring` all ok (worldpack
  lib 10 — 4 pre-existing + 6 configure; the "38" in E-IA-4 is the `content_kinds` target, not lib);
  clippy `-D warnings` workspace clean; fmt clean. Mutations, each observed then reverted:
  - M-IA2 (configuration seeded after sections) → `configuration_is_seeded_after_locations_and_before_sections_in_configure_order`
    FAILS: left `[arrived, probe-sectioned, probe-configured, probe-configured]`.
  - M-IA4a (`compare` returns `Ok` first) → `the_comparator_refuses_changed_removed_and_added_configuration`
    FAILS at "a changed step".
  - M-IA1, realized as "initial_facts seeds one extra genesis emission at the configuration position"
    (a copy of the last location fact — no installed capability has a configuration fact type to seed
    "empty", D-11): `run worlds/social-cafe --headless --seed 7 --days 300` → exit 0, faults 0,
    365 331 facts (E-IA-0: 365 330), fingerprint f2fad0b519f4e894 (E-IA-0: 59339a9c281829c9), sha
    022b3ace…95c0 ≠ ad49c723…c64b; wall 18.4 s. The instrument sees genesis. Town run 3 of 5.
  `git grep MUTATION -- '*.rs'` empty after.
- [x] Review: a world without `configure:` reaches `configure::seed` with an empty slice, which
  returns an empty vector — no new fact, no new id; `check_configuration` is read-only (it assembles a
  throwaway world). `load.rs` grows to 688 lines (683 at IA-C4; 682 on base): pre-existing size,
  growth of 6 lines, recorded.

### IA-C6 — Hosts: the drift check at every resume

**Scope.**
- `tools/cli/src/main.rs`: the helper `saved_genesis(&SqliteBackend) -> Result<Vec<EventEnvelope>,
  String>` (decoding `facts_of(GENESIS)`, as `genesis_instant` does); one call in `persisted` and one
  in `replay`.
- `tools/cli/src/run.rs`: one call before `PersistentWorld::resume`.
- `tools/cli/tests/configure.rs`:
  - the structural test of IA-4 b, with M-IA4b;
  - the binary-level refusals of IA-3;
  - the three worlds' `validate` output byte-identical (IA-1, last line).

- [x] Implementation: `main.rs` — `saved_genesis(&SqliteBackend) -> Result<Vec<EventEnvelope>,
  PersistError>` (decodes `facts_of(GENESIS)` with `persistence::format::decode`, as `genesis_instant`
  does; each caller maps the error its own way, D-12); one `check_configuration` call in `persisted`
  after opening the save and before `PersistentWorld::resume`; one in `replay` before `compose` and
  `verify`. `run.rs` — one call before `compose`/`PersistentWorld::resume` in `Begun::new`'s resume
  branch. Done on the merged base (main @ f842c52, S11-A's `main.rs`, D-13). New
  `tools/cli/tests/configure.rs`: the structural test (IA-4 b) over the three functions' bodies, and
  `validate` refusing six reachable mistakes by name, naming the file, exit non-zero.
- [x] Validation (E-IA-6): `cargo test -p mineworld-cli --test configure` 2 passed; `restart` 1,
  `run_restart` 2, `server_command`-independent `bodies_yard_restart` 2 (and `run_restart` 2) — the
  restart tests, unedited, pass (57.6 s, 33.0 s wall). M-IA4b (run.rs's call removed) → FAILS "run.rs
  `fn new(..)` never checks the configuration"; reverted, `git grep MUTATION` empty. Clippy `-D warnings`
  workspace and fmt clean. The three worlds' `validate` byte identity is E-IA evidence (C4, C8), not a
  test, because a test holding it would need a frozen copy of today's output (D-12).
- [x] Review: no file under `server/` differs from `origin/main` (the server diff against 0d35d6b is
  S11-A's, merged); each call precedes resume or verify; the check reads the save through the backend's
  existing `facts_of`, nothing in `persistence/` changes.

### IA-C7 — The proof with test-only packs; the vocabulary scan

**Scope.** `tests/acceptance/tests/configuration/mod.rs`:
- **`test-tuning`**: configuration `{ step: 1 … 100 }`, seeded as `tuning-configured { step }`
  (`SystemInternal`), reduced into a `Step` component on each Place. It owns a section `tuned:` on
  person files, `{ start }`, whose reduction refuses a start that is not a multiple of the step. It
  provides one action, `advance`, stating `advanced { by: step }`.
- **`test-relay`**: owns a trait `Relay` and a catalog `register_relays`.
- **`test-relay-a`** and **`test-relay-b`**: each implements `Relay`.

Neither pack names a physics word or an interaction word.

`configuration_seam.rs` covers IA-2, IA-4 a, IA-5 and IA-7 through `World::dispatch` and
`PersistentWorld`. `configuration_vocabulary.rs` covers IA-8, with M-IA8.

- [x] Implementation: `tests/acceptance/tests/configuration/mod.rs` — `test-tuning` (configuration
  `{ step: 1 … 100 }` by its own `TryFrom` bound, seeded `tuning-configured { step }` `SystemInternal`,
  reduced into `Stride` on every Place; section `tuned:` `{ start }` refused off-step or unconfigured;
  action `advance` stating `advanced { person, by }`), `composed()` (what a save resumes into),
  `world()`, and `genesis_facts()`, which decodes through `SystemPack::decode_configuration` (from
  `configures!()`) and authoring's `Decode` exactly as the loader does. `configuration_seam.rs`: IA-2
  (a multiple taken, an off-step start refused by name, sections-before-configuration refused as
  unconfigured, the pack's bound refused at "line 1 column 7"), IA-4 a (a save's genesis read back from
  SQLite: unchanged and reformatted Ok; changed, removed, added each `Drift` naming test-tuning), IA-7
  (240 `advance` requests through `World::dispatch` in a `PersistentWorld`, SIGKILL at mid-run, resumed
  by a new process, journal/facts/snapshots byte-identical to an uninterrupted control, whole history
  `verify`d). `configuration_vocabulary.rs`: IA-8 over `authoring/src/configuration.rs`,
  `worldpack/src/configure.rs`, `configure/tests.rs` and every test file IL-a adds, with three
  admissions for the reserved key `classes` (D-14). test-relay's catalog is `sdk/rust/tests/extensions.rs`
  (IA-C3, D-2): it was not duplicated here. Rewordings for IA-8 in IL-a's own tests: `conversation` →
  `schedule` as the not-enabled system, a helper `body` → `function`, one doc word. After merging main
  (#77, D-15), IL-a's scratch in `worldpack/tests/configuration.rs`, `tools/cli/tests/configure.rs` and
  `configuration_seam.rs` moved onto `mineworld_test_support::scratch!`. Amendment A-1 applied
  (`tests/acceptance/Cargo.toml`, `Cargo.lock` +2 lines).
- [x] Validation (E-IA-7): see the ledger.
- [x] Review: the test packs exist only in `tests/acceptance/tests/`; no library depends on the acceptance
  crate (it has no `[lib]` dependents); the lock diff is the two lines; AC-1 check 2 passes (no normal or
  build path from the framework crates to a market pack; `authoring` and `sdk` depend on contracts,
  kernel, packages and serde only).

### IA-C8 — Close: byte-identity, the canary, the full gate, the ledger

**Scope.**
- IA-1 against E-IA-0 (two 300-day runs, one bodies-yard run, `long_run`).
- IA-10's canary, on a scratch branch never pushed, deleted after.
- IA-9's `git diff --stat`.
- IA-11's full gate on the final head, in the background.
- `MVP_STATUS.md`; the §11.11 ledger; the handoff.

- [x] Implementation: IA-1 captured on the head; the canary built, run and discarded; IA-9's diff
  recorded; `MVP_STATUS.md` (written at IA-C1) re-checked current; the ledger (E-IA-7, E-IA-8, E-IA-9)
  and the handoff closed.
- [x] Validation: IA-1, IA-9, IA-10 PASS (E-IA-8); IA-11 the full gate (E-IA-9).
- [x] Review: IA-1 … IA-11 each with evidence below; deviations D-1 … D-15 recorded in §11.12.

**Acceptance summary.** IA-1 PASS (E-IA-8). IA-2 PASS (E-IA-5 in-crate order and M-IA2; E-IA-7
test-tuning). IA-3 PASS (E-IA-4 reader, M-IA3; E-IA-6 binary; file-missing at the binary on the canary,
D-8). IA-4 PASS (a: E-IA-5 M-IA4a and E-IA-7 over a real save; b: E-IA-6 M-IA4b; c: E-IA-8 canary).
IA-5 PASS (E-IA-3, M-IA5a, M-IA5b; registration.rs and arrival_resolvers* unedited). IA-6 PASS (E-IA-3,
M-IA6). IA-7 PASS (E-IA-7). IA-8 PASS (E-IA-7, M-IA8; seam_vocabulary unedited). IA-9 PASS (E-IA-8).
IA-10 PASS (E-IA-8). IA-11 see E-IA-9.

## 11.7 Test ownership

| Test | Owner | Edited by IL-a |
| --- | --- | --- |
| `systems/installed/tests/resolution.rs` | the installed set | rewritten (generic), same file |
| `worldpack/tests/registration.rs` | worldpack | doc comment only |
| `tests/acceptance/tests/{ac1_composability,precursor_vocabulary,seam_vocabulary,arrival_resolvers*}.rs` | acceptance | **no** |
| `tools/cli/tests/{restart,run_restart,bodies_yard_restart,run,market_town}.rs` | cli | **no** |
| new: `authoring` unit tests, `sdk/rust/tests/extensions.rs`, `worldpack/tests/configuration.rs`, `tools/cli/tests/configure.rs`, `tests/acceptance/tests/configuration*` | IL-a | new |

## 11.8 Is any of this material?

- **No kernel, contract, persistence or server change** — a need for one is a material stop.
- Editing presence is limited to one message (QIA-3).
- No digest moves; a moved digest is a material stop.
- The ARC-39 note changes no rule, only the line's spelling (QPL-10, decided).
- The reserved `packages` key touches E-b's frozen design only as recorded in its FQ-b2 hook (QIA-1).

## 11.9 Questions (QIA-1 …)

| ID | Question | Recommendation |
| --- | --- | --- |
| **QIA-1** | `packages` and `classes`: reserve both in IL-a, wiring `packages` → `LicencePolicy` in IL-b (with `classes`), or have IL-a wire `packages` if E-b merges before IL-a's freeze? | **Reserve both here; wire both in IL-b.** IL-a stays domain-free and independent of E-b's merge order. |
| **QIA-2** | Drift compares only each pack's declared configuration fact types (`FACTS`), in order. | **Yes.** |
| **QIA-3** | Edit presence's one panic message ("resolution: line" → "extension line"), no VERSION change. | **Yes**: otherwise it names a line that no longer exists. |
| **QIA-4** | Configuration facts are `SystemInternal`, with no subjects, by guidance in ARC-61 (not enforced by the loader). | **Guidance, not enforcement**: a loader rule over Visibility would be the loader judging a pack's vocabulary. |
| **QIA-5** | The canary install (IA-10) on a scratch branch, never pushed: authorized? | **Yes**, as 11a C5. |
| **QIA-6** | `sdk/rust/tests/extensions.rs` added to the change set for IA-5's test-local catalog. | **Yes.** |

**Freeze rulings (primary session, 2026-10-08).** QIA-1 to QIA-6 accepted as recommended. QIA-1: `packages` and `classes` are reserved in IL-a and both are wired in IL-b — consistent with the E-b freeze (FQ-b2), under which E-b ships the default licence policy and its evaluator, and a later `configure/packages.yaml` override is a recorded hook. QIA-4: guidance only, stated in ARC-61. QIA-5: authorized, never pushed, deleted afterwards.

**Coordination note from S16 E-b (2026-10-08, relayed by the primary session during IL-a's
implementation).** The `packages` key is owned by the framework crate `mineworld-packages`, not by a System
Pack; the `configure:` seam is keyed by system id. IL-a reserves `packages` and refuses it; IL-a's reservation
and messages do not assume a key names a system (`ConfigurationReserved` says "reserved for the licence
policy's override", and `ConfigurationKey`'s documentation says a key is a system id *or* a reserved key).
**IL-b must let the framework-owned `packages` key through** — routed to `mineworld-packages`, not resolved
against the installed set, not seeded, not drift-checked (§11.5's hook paragraph).

## 11.10 Proposed execution contract for PR IL-a

```text
PROJECT / PR        MVP-0 · S17 / PR IL-a — the configuration seam and the extension catalogs (framework
                    precursor; names no physics and no interaction)
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-18-interaction-list.md §11; evidence §11.11
                    (E-IA<n>); deviations §11.12, added at the first deviation
RELATED / BINDING   this file §§4.3, 4.10, 5 (IL-I1, IL-I4, IL-I8, IL-I10), 6; overall.md "The World
                    Interaction List"; step-18-physics-list.md §4.6–§4.7 (ARC-61/62 drafts);
                    step-16-packages.md §15 (E-b, frozen); DECISIONS ARC-5, ARC-23, ARC-25, ARC-26, ARC-31,
                    ARC-33, ARC-35, ARC-39, DEP-10, DEP-12; MODULE_SPEC §§3.1, 4, 4.1, 9; CLAUDE.md §§2–4
IMPLEMENTATION BASE main @ 0d35d6b, or the main named at freeze (re-audit §11.2); branch mvp0/pr-il-a-seam;
                    one worktree, one session
APPROVED SCOPE      §11.1's change set; IA-C1 … IA-C8; SD-IA-1 … SD-IA-11 as answered by QIA-1 … QIA-6
FROZEN INVARIANTS   No diff under kernel/, contracts/, persistence/, server/, clients/, cognition/, worlds/;
                    no System Pack other than presence's one message and systems/installed; root Cargo.toml
                    and Cargo.lock unchanged. IA-1: the towns' 300-day seed-7 fingerprints, bodies-yard's
                    30-day sha and 12b's long_run bytes equal E-IA-0. ac1_composability.rs,
                    precursor_vocabulary.rs, seam_vocabulary.rs unedited and passing. No shim for
                    `resolution:`. A world without configure: seeds exactly what it seeded.
SEQUENCE            IA-C1 → IA-C8, each committed and pushed when coherent; E-IA-0 captured before IA-C2
VALIDATION BUDGET   unit/integration/static unrestricted; real runs: each 300-day town run at most five times
                    in all (E-IA-0 ×2, M-IA1 ×1, IA-1 ×2), bodies-yard 30-day and long_run twice each, the
                    canary once; one full workspace gate on the final head (background); real-model NOT
                    REQUIRED
LIVE DOCUMENTATION  §11 checkboxes; the E-IA ledger; deviations
HANDOFF             .structured-coding/plans/mvp0/handoff-il-a.md, created at IA-C1
ENDPOINT AUTHORITY
  implementation + local validation   at the primary session's freeze message
  semantic commits, branch push       recommended authorized
  PR creation / update                recommended authorized, marked READY FOR OPERATOR REVIEW
  scratch canary branch               recommended authorized (QIA-5), never pushed, deleted after
  merge                               operator only, with a merge commit
COORDINATION        E-b: §11.5; whichever merges second performs the merge and re-runs IA-1 and IA-9
NORMAL STOP         PR IL-a READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       an edit outside §11.1's change set, above all in kernel/contracts/persistence/server or a
                    System Pack's behaviour; any IA-1 difference; an unedited guard failing; a need to edit
                    load.rs in a way E-b's design forbids it to merge over; an answer to QIA-1 … QIA-6 other
                    than the design's
```

## 11.11 Evidence ledger (E-IA)

```text
E-IA-d  2026-10-08, design commit: check_doc_headings and check_decision_ids, recorded at commit.
E-IA-0  2026-10-08, captured on f6489fd (code = main @ 0d35d6b; the branch's diff from 0d35d6b outside
        .structured-coding/ is empty), dev profile, before IA-C2, by target/il-a/capture.sh base
        (artifacts target/il-a/base-*; "sha" = sha-256 of every output line but `wall`):
        social-cafe `run --headless --seed 7 --days 300`: exit 0, 339 lines, faults 0, 365 330 facts,
          fingerprint 59339a9c281829c9, sha ad49c7235f672153b328b8d8e283a7409f23b35ba847d319e1fcab4e9716c64b
          (= the S9 E-0), wall 16.6 s
        market-town, same: exit 0, 355 lines, faults 0, 372 755 facts, fingerprint 085ed9c55cae7947,
          sha 365b50e06638795912b12304b20b0f2fc33dbbc2ba1c20ac6648261195391d1d, wall 24.5 s
        bodies-yard `--days 30`: exit 0, 52 lines, faults 0, 62 855 facts, fingerprint e9398cf4e97ecaf2,
          sha 6e4c4015077924b6747184dd0107164654cefc11532375aff3a9d94a4d7c8395
        long_run (BODIES_LONG_RUN_SECOND_PROCESS=1 cargo test -p mineworld-bodies --test long_run --
          --nocapture): exit 0, LONG-RUN line 4 091 748 bytes, sha
          d7025dbcdb63c0aa5162c10552f43e2650d24e510dc67f4662ac30d1b2479eaf (= 12c's PO-13 b reference)
        validate: social-cafe 27 lines sha ebcd60a0…f56a8, market-town 51 lines sha 64f41086…73502,
          bodies-yard 40 lines sha 7356b8f8787f4e24e23120acfddc278bb4815d12f93a3eea60534bd73a12063f
        PASS (captured; the towns' values equal the recorded references, so no 12d re-baseline applies).
E-IA-7  2026-10-08, IA-C7 on c6ca232 + the IA-C7 tree: `cargo test -p mineworld-acceptance` → every
        target ok: ac1_composability 13 (unedited; check 2 passes with the two A-1 dev-dependencies),
        arrival_resolvers 7, arrival_resolvers_resume PASS, arrival_resolvers_unregistered 2,
        complete_affordances 4, configuration_seam 4 (control 241 revisions; killed at 123, 123 on disk;
        resumed and verified), configuration_vocabulary 2, precursor_vocabulary 4 (unedited),
        seam_vocabulary 3 (unedited). `-p mineworld-worldpack --test configuration` 8, `-p mineworld-cli
        --test configure` 2. Clippy `--workspace --all-targets -D warnings` clean; fmt clean.
        `check_scratch.py scan`: 148 test sources, none outside the helper (1 pre-existing exemption).
        M-IA8 (`// talk MUTATION M-IA8` appended to worldpack/src/configure.rs) → the scan FAILS naming
        "worldpack/src/configure.rs:361: talk"; reverted (file restored), `git grep MUTATION -- '*.rs'`
        empty. Lock diff: exactly `+ "mineworld-authoring"` and `+ "mineworld-sdk"` in
        mineworld-acceptance's list. PASS.
E-IA-8  2026-10-08, IA-C8 on a83b103 (origin/main e98321a merged), by target/il-a/capture.sh head, dev
        profile (artifacts target/il-a/head-*):
        IA-1  social-cafe 300 days seed 7: exit 0, sha ad49c723…c64b (= E-IA-0), wall 17 s;
              market-town: exit 0, faults 0, 372 755 facts, fingerprint 085ed9c55cae7947, sha
              365b50e0…1d1d (= E-IA-0), wall 24 s; bodies-yard 30 days: sha 6e4c4015…c8395 (= E-IA-0);
              long_run second process: exit 0, 4 091 748 bytes, sha d7025dbc…79eaf (= E-IA-0);
              validate ×3: `cmp` identical to E-IA-0's files (and shas ebcd60a0…, 64f41086…,
              7356b8f8… equal). Town runs used: 5 of 5 (E-IA-0 ×2, M-IA1 ×1, IA-1 ×2). PASS.
        IA-9  `git diff origin/main mvp0/pr-il-a-seam --stat -- kernel contracts persistence server clients
              cognition worlds Cargo.toml`: empty. The three guards (ac1_composability, precursor_vocabulary,
              seam_vocabulary): no diff, all pass (E-IA-7). Cargo.lock: exactly A-1's two lines (E-IA-7).
              Every changed path (32 files, `--name-only`) is in §11.1's change set as amended. PASS.
        IA-10 the canary, on local branch canary/il-a-ia10 from a83b103, never pushed: test-tuning as a
              crate systems/test-tuning (the IA-C7 pack plus an empty PerceptionProvider impl) installed
              with its two ARC-33 lines; a scratch test copied social-cafe as `tuning-cafe` with
              `test-tuning` enabled and `configure: [test-tuning]` (`step: 5`), under the scratch helper:
              `validate` ok; `run --days 30 --save` control: faults 0, 37 086 facts, exactly one
              `tuning-configured` fact in the save; a second save SIGKILLed at day 15 (14 282 of 28 523
              journal rows) and re-run: "resumed … at revision 14282 (snapshot 14272 + 10 re-executed)",
              facts, journal and snapshots byte-identical to the control; `replay` ok (28 523 revisions,
              37 086 facts, 446 snapshots); configure/test-tuning.yaml edited to `step: 10` → `run` and
              `replay` both exit non-zero, "the world's configuration differs from the save's: system
              'test-tuning' (save: tuning-configured {"step":5} SystemInternal; this pack:
              tuning-configured {"step":10} SystemInternal)"; the file removed → `validate` refuses
              "configure: names 'test-tuning', but …/configure/test-tuning.yaml does not exist" (D-8's
              file-missing case). Wall 12.5 s. Then the tree restored, the branch deleted
              (`git branch -D`), `git ls-remote origin 'refs/heads/canary*'` empty, `git status` clean.
              PASS.
E-IA-9  2026-10-08, IA-11 full gate on c86ffb3 (the final code head; later commits are this ledger
        only), one background run (target/il-a/gate.log, gate-test.log): `cargo fmt --all --check` 0;
        `cargo check --workspace --all-targets` 0; `cargo clippy --workspace --all-targets -- -D
        warnings` 0; `cargo test --workspace` 0 — 161 result lines, 715 passed, 0 failed;
        `check_doc_headings` 191 sections / 26 documents, none duplicated; `check_decision_ids` 58
        distinct. `check_scratch.py scan` 0 (148 sources, none outside the helper, 1 pre-existing
        exemption). `check_scratch.py left` exit 1 with 64 entries (2.6 GiB) — identical, by `diff`, to
        the listing taken before the gate (left-before-gate.txt): every entry predates #77's helper
        (target/tmp/<name> and $TMPDIR/mineworld-kill-*, from this worktree's pre-merge runs); none is
        a mineworld-scratch-<pid> container and none is IL-a's (no `configure-*`, `configuration-*` or
        canary name). The gate left nothing new. PASS.
```

## 11.12 Deviations

```text
D-1  (bounded, accepted by the primary session 2026-10-08) authoring's unit tests decode through
     serde::de::value deserializers, not serde_json and serde_saphyr: either would be a new dev-dependency
     of mineworld-authoring and so a Cargo.lock change. Decoding through serde_saphyr with line and column
     is proven in worldpack (which already depends on it), IA-3.
D-2  (bounded, accepted) sdk/rust/tests/extensions.rs's second catalog is a test-local trait, not
     presence's: the sdk cannot depend on presence (it sits below every pack, ARC-33). IA-5's presence
     half is held by worldpack/tests/registration.rs and arrival_resolvers*.rs, unedited.
D-3  (material, ruled) Amendment A-1, §11.1.
D-4  (bounded) Capability::extension_types() returns Vec<(&'static str, Vec<&'static str>)>, not a
     &'static slice (SD-IA-3): the types' Rust paths come from core::any::type_name, which is not const,
     and the installed set's guard compares them with Capability::type_name. No caller beyond the guard.
D-5  (bounded) One more refusal than SD-IA-7 lists: ConfigurationStatedUndeclaredFact, for a seeded fact of
     the owner's own vocabulary whose type is outside its FACTS (SD-IA-9 requires the refusal; a separate
     variant names it honestly instead of overloading "another pack's fact").
D-6  (bounded) `configure:` duplicates are checked before each key is resolved, not after: it assumes
     nothing, and it makes the refusal reachable with this build's packs (`[movement, movement]`) through
     the real loader instead of only through a probe.
D-7  (bounded) read.rs: parse_with and declared_entities become pub(crate) (configure.rs uses them; one
     saphyr call site per decoding module, DEP-10's comment updated), and a #[cfg(test)]
     with_configuration beside in_memory. In-crate tests live in worldpack/src/configure/tests.rs (a
     child module file) to keep configure.rs small.
D-8  (scope finding) Of SD-IA-7's refusals, only unknown system, reserved, not enabled, not configurable,
     listed twice and undeclared file are reachable with this build's packs (none is configurable). File
     missing, owner decode, requires, references and the seeding refusals are proven in-crate with a probe
     and through the real binary only on the canary (IA-10). IA-3's "file missing through the real
     `mineworld validate`" is therefore canary evidence, not a merged test.
D-9  (bounded) AuthoredConfiguration gains facts() -> &'static [EventTypeId] (its owner's FACTS). The loader
     checks a seeded fact against the decoded value's own owner and FACTS, as sections check against
     content.owner(), rather than against the capability label — the same values in production, and the
     only form a probe-labelled in-crate test can exercise.
D-10 (bounded) compare's filter is a BTreeMap<EventTypeId, SystemId> (the union of configuration_facts with
     each type's system), not a BTreeSet, so the Drift names the system on either side.
D-11 (bounded) M-IA1's literal form ("one empty configuration emission per enabled capability") cannot be
     built: no installed pack declares a configuration fact type. Realized as one extra genesis emission
     at the configuration position, which is what the mutation is for (does the instrument see genesis).
D-12 (bounded) saved_genesis returns PersistError, not String, so the server path can wrap it in HostError
     and the others format it; tools/cli/tests/configure.rs does not hold the three worlds' validate bytes
     (that would freeze a copy of today's output in a test); IA-1's cmp against E-IA-0 holds it.
D-13 (coordination) origin/main (S11-A, #76, f842c52) was merged into the branch before IA-C4 (6ed1eea),
     clean; S11-A touches server/, clients/ and tools/cli/src/main.rs, none of the simulation paths, so
     E-IA-0 (captured on 0d35d6b) remains the IA-1 reference.
D-14 (bounded) IA-8's scan admits `classes` on exactly three lines — the RESERVED entry in configure.rs, the
     reserved-key test in worldpack/tests/configuration.rs, and the validate case in tools/cli/tests/
     configure.rs — because SD-IA-6 requires the seam to name the reserved key in order to refuse it. Each
     admission is (file, line substring, word, reason) and fails if it admits nothing (seam_vocabulary's
     rule). Every other listed word was reworded out of IL-a's own files.
D-15 (coordination; session resumed after an API rate-limit cut-off) The interrupted session left
     tests/acceptance/Cargo.toml, Cargo.lock (A-1, exactly as ruled) and the uncommitted IA-C7 files
     (configuration/mod.rs, configuration_seam.rs). All were kept and finished, not discarded: the
     resume/create path was corrected (a save resumes into composed(), not a populated world), the
     SIGKILL child's first report now starts its own line, and an advance is counted by decoding the
     stored fact. origin/main (#77 test-hygiene, e98321a) was merged (c6ca232): one conflict in
     DECISIONS.md (ARC-61/62 vs DEP-29, both appended; resolved by keeping both), and Cargo.lock
     regenerated from main's plus A-1's two lines. #77 touched no file IL-a's IA-C6 calls live in
     (tools/cli/src/{main,run}.rs unchanged by the merge).
```
