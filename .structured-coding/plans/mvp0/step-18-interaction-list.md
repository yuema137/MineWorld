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
| **IL-b** The SDK interaction schema (ARC-63, ARC-64, ARC-65, DEP-28) — *expanded into a full PR design in §12 (2026-10-08), with five items added by later rulings and audit (§12.1)* | `mineworld_sdk::interactions`: `InteractionSection`, `parameters!`, `interactions!()`, `classes.yaml` (sdk-decoded), selectors from tags, levels, specificity, forbid-overrides, load-time ambiguity refusal, `permits`/`parameters`/`consequence`, per-Place storage; `mineworld interactions`; the biography projection reading configured sections (ARC-29 amended); `tuning` gains a section (one action, one parameter, one fact) | On a scratch world: a forbidden `tuning` action is `PermissionDenied` and its offer unavailable for that reason; a scoped parameter applies only to its class; a narrowed fact is not perceived by a bystander (through the unchanged S11-C function and `mineworld perceived`); `biography: off` removes it from `mineworld biography` while the fact log still holds it; drift refused | (1) Property test: every lookup on every generated list is total and order-independent of authoring order. (2) Equal-specificity conflicting parameters refused at load, naming both. (3) A widening audience, biography on a non-configurable fact, an unknown action — each refused. (4) Mutation: make `permit` win ties → the forbid-overrides test fails. (5) Mutation: let the biography projection ignore sections → the biography test fails. (6) IL-I10 scan |
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

# 12. PR IL-b — the Interaction List schema and its first sections (full design)

**Lifecycle:** `PR design — ready for freeze review`. Drafted by the planning session on
`plan/s17-il-b` (worktree `/Users/yuema137/mineworld-worktrees/plan-il-b`), 2026-10-08. Not frozen.
Implementation needs the primary session's `DESIGN FROZEN` header on this section, a fresh session and
its own worktree.

**Placement note.** PR IL-a's design is §11 of this file on `mvp0/pr-il-a-seam` (PR #80, open). This
section is numbered §12 so that both read in order once both merge. Both append at the end of the file,
so whichever merges second resolves one mechanical conflict: §11 first, then §12, nothing else changed.

**Binding rulings this section implements.**
- `overall.md` "The World Interaction List" (operator and primary session, 2026-10-08):
  - QIL-2 overruled: the carrier is `configure:` and `configure/<key>.yaml`, and classes live in
    `configure/classes.yaml`;
  - QIL-3, QIL-7: classes are tag selectors and fixed during play;
  - QIL-8: consequence routing;
  - QIL-4 … QIL-6, QIL-9, QIL-11 … QIL-16, QIL-18 … QIL-20 as recommended;
  - the decision numbers ARC-63, ARC-64, ARC-65 and DEP-28.
- IL-a's freeze ruling on QIA-1, and the S16 E-b coordination note (§11.9): IL-b wires both reserved
  keys. The `packages` key is owned by the framework crate `mineworld-packages`. It is let through to
  that crate, not resolved against the installed set, not seeded and not drift-checked.
- S19 (`step-19-time-weather.md`, primary session's rulings on QTW-7 and QTW-15):
  - QTW-15: IL-b makes `CONVERSATION_GAP` and `INVITATION_LIFETIME` configurable content, and the
    headless defaults do not change (INV-TW-1);
  - QTW-7: the seam gains a typed `data:` attachment, whose decoded content is carried inside the
    owner's configured fact.

## 12.1 Identity, base, approved scope

```text
PR            IL-b — the Interaction List schema (ARC-63, ARC-64, ARC-65, DEP-28) and its first sections:
              conversation's gap and group-activity's invitation lifetime (QTW-15); the seam's data:
              attachments (QTW-7); the framework keys classes and packages (QIA-1). S17, second PR
base          main after PR IL-a (#80) has merged, at the commit named at freeze. Re-audit §12.2 if any of
              these moved: authoring/src, sdk/rust/src, worldpack/src, systems/{presence,conversation,
              group-activity}/src, cognition/rule-controller/src, tools/cli/src or
              tests/acceptance/tests/configuration*. Re-audit above all if 12d has merged (§12.6)
branch        mvp0/pr-il-b-sections, in a fresh worktree named at freeze, held by the implementing
              session only
audit         §12.2 (main @ 9cf8f8e and mvp0/pr-il-a-seam @ b6a954b, 2026-10-08)
depends on    IL-a merged. 12d is NOT a dependency: IL-b touches no bodies constant and no world (§12.6)
merge         a merge commit, never a squash (ARC-5)
```

**Goal.** A world can configure what an installed pack declares configurable, through one schema that
every pack shares. Concretely:
- **The schema works.** A section can forbid an action for a class of actor or target. It can scope a
  parameter to a class or a place. It can narrow a fact's audience and switch its biographical flag
  where the owning pack allows. The schema is proven end to end on the test-only `test-tuning` pack.
- **Two real numbers move behind it.** `conversation` reads its gap and `group-activity` reads its
  invitation lifetime from the world's list. A world that configures nothing produces byte-identical
  digests.
- **A pack can be handed a data file** (`data:`). The decoded content becomes the pack's own genesis
  fact, so the drift check covers a changed file.
- **A world can narrow or extend the licence policy** through `configure/packages.yaml`.

**Revisions to the §6.2 IL-b row, each with its source.** The row's scope is kept whole, and five
items are added:
1. The two parameter sections and their packs' conversions (QTW-15).
2. `data:` attachments (QTW-7).
3. The `packages` key wired (QIA-1 and the E-b note).
4. A pack-stated refusal on `Offer` (F-IB-5). Without it, the row's own checkpoint ("its offer
   unavailable for that reason") cannot be built.
5. `group-activity`'s `Invitation` carries its expiry, and the paced rule controller reads it (F-IB-3).

The row's "a narrowed fact is not perceived by a bystander … through `mineworld perceived`" cannot be
built yet (F-IB-6): neither S11-C's audience function nor `mineworld perceived` exists on `main`.
IL-b proves the envelope's Visibility, and the perception proof moves to whichever of IL-e and S11-C
lands second (QIB-12).

**Change set** (every path this PR may touch):

```text
docs/DECISIONS.md                     ARC-63, ARC-64, ARC-65, DEP-28 (new); dated notes on ARC-29 (the
                                      projection reads configured sections), ARC-34 (a pack-stated
                                      refusal on an offer), ARC-55 (configure/packages.yaml wired),
                                      ARC-61 (framework keys, data: attachments, the seeding context)
docs/MODULE_SPEC.md                   §4 model (configure/classes.yaml, configure/packages.yaml, data/);
                                      §4.1 (the framework keys, attachments, their refusals); a new §4.2
                                      "The World's Interaction List" (the vocabulary of §4.1 of this
                                      file, the section shape, precedence, what a list cannot do)
docs/MVP_STATUS.md                    one capability row; the S17 row
systems/README.md                     "Adding a pack": a pack's interaction section
authoring/src/configuration.rs        ConfigurationContext; PackConfiguration::attachments; seed takes
                                      the context
authoring/src/classes.rs              NEW: EntityClasses, ClassName, the classes file type
authoring/src/attachment.rs           NEW: Attachment (a path under data/), Attached (bytes by path)
authoring/src/lib.rs                  module lines, re-exports
sdk/rust/src/interactions/            NEW module: decl.rs (ActionDecl, FactDecl, roles), selector.rs,
                                      section.rs (the authored section, parameters!), resolve.rs (levels,
                                      extends, regions, specificity, forbid-overrides, ambiguity refusal),
                                      lookup.rs (permits, parameters, consequence), biography.rs (the
                                      selection the projection asks), tests.rs
sdk/rust/src/{lib,pack,installed}.rs  interactions!(); SystemPack and Capability aggregates; __private
sdk/rust/tests/interactions.rs        NEW: the property test (IB-6) and the macro's expansion
worldpack/src/configure.rs            framework keys (classes, packages) replace RESERVED; attachments
                                      read; the context handed to seed
worldpack/src/configure/tests.rs      probe tests for the above
worldpack/src/read.rs                 packages read before requirements::resolve; classes kept
worldpack/src/requirements.rs         the policy passed in rather than LicencePolicy::default()
worldpack/src/{error,format,lib}.rs   new refusals; FoundConfiguration's attachments; re-exports
worldpack/tests/configuration.rs      the two "reserved" tests become "framework key" tests (§12.8)
worldpack/tests/interaction_sections.rs   NEW: the real sections on scratch copies of social-cafe
systems/presence/src/{interaction,observe}.rs   Offer::refused; the verdict puts it first (SD-IB-12)
systems/presence/tests/…              one test of the refusal's precedence (file named at C4)
systems/conversation/src/{system,interactions,lib}.rs, README.md, tests/   the `gap` section; VERSION 2
systems/group-activity/src/{component,system,perception,interactions,lib}.rs, README.md, tests/
                                      the `invitation_lifetime` section; Invitation's `until`; VERSION 2
cognition/rule-controller/src/{social,social_tests}.rs   reads an invitation's `until`, not the constant
tools/cli/src/{main,biography,packs}.rs   the projection asks configured sections; packs validate
                                      judges with the world's policy
tools/cli/src/interactions.rs         NEW: `mineworld interactions <world> [--place KEY] [--json]`
tools/cli/tests/{configure,interactions,interaction_runs}.rs   configure.rs's reserved case changes;
                                      two NEW files
tests/acceptance/tests/configuration/mod.rs   test-tuning gains a section, an offer and a data: file
tests/acceptance/tests/{configuration_seam,configuration_vocabulary}.rs   updated (§12.8)
tests/acceptance/tests/interaction_schema.rs  NEW: the schema proven with test-tuning
.structured-coding/plans/mvp0/{step-18-interaction-list,handoff-il-b}.md
```

**Paths with no diff:**
- `kernel/`, `contracts/`, `persistence/`, `server/`, `clients/`, `worlds/**`;
- every System Pack other than presence's offer refusal, conversation and group-activity (in particular
  `systems/bodies`, `systems/item` and `systems/movement`, which 12d edits);
- `systems/installed/src/lib.rs` (the aggregates come from the macro);
- the root `Cargo.toml` and `Cargo.lock` (no new crate dependency: tools/cli reaches the SDK's selection
  through `mineworld-worldpack`'s re-exports, SD-IB-14);
- `tests/acceptance/tests/{ac1_composability,precursor_vocabulary,seam_vocabulary}.rs`.

**Non-goals.**
- Rules or consequences for any real pack: IL-e, IL-f, IL-g. In IL-b, conversation and group-activity
  declare parameters only. A `rules:` or `consequences:` entry in their sections is refused as naming an
  undeclared action or fact.
- Bodies: IL-c after 12d (§12.6).
- Any change to a World Pack's content. No world sets a larger gap or lifetime (QIB-5).
- Reference lists that one pack ships to another (bodies' `winter` from `ice`): IL-i (QIB-10).
- Perceiving a narrowed fact: S11-C plus IL-e (QIB-12).
- Mutable classes (QIL-7) and several classes per entity (QIL-6).

## 12.2 Source audit (`main @ 9cf8f8e`; IL-a at `mvp0/pr-il-a-seam @ b6a954b`; 2026-10-08)

| ID | Finding | Evidence | Consequence |
| --- | --- | --- | --- |
| **F-IB-1** | Every simulation constant in `systems/*/src` is compile-time. No pack reads a configuration today, and no installed pack implements `PackConfiguration` even on IL-a's branch. The full list is §12.3. | `grep -rnE 'const [A-Z_0-9]+' systems/*/src`, filtered of identifiers, schema versions and rejection codes (the remainder is §12.3); `git grep configures!` on IL-a: only test-tuning | IL-b is the first real consumer of the seam. |
| **F-IB-2** | `CONVERSATION_GAP` (300 s) is read in exactly one place: `within_the_gap` (`systems/conversation/src/system.rs:377–380`), called from `continues_a_conversation` (`:360`) in `resolve`. Outside the pack it is re-exported (`lib.rs:61`) and used by one pack test (`tests/conversation_and_presence.rs:710`). It is named in the doc comments of `event.rs:18` and `tools/cli/tests/restart.rs:15`. | as cited | The gap is decided in the speaker's `resolve`, where the speaker, the listener and the place are known, so a lookup there sees all three roles. |
| **F-IB-3** | `INVITATION_LIFETIME` (1 800 s) is read in two crates. In group-activity, `Invitation::is_open_at` (`component.rs:55–58`) serves the pruning on every write (`:101–103`) and `open_invitation` in validate (`system.rs:337–348`). **Outside it, the paced rule controller compares an invitation's age with the constant itself** (`cognition/rule-controller/src/social.rs:25, :68–74`), because "an offer cannot see the clock" (`perception.rs:14–19`, step-09 D-B4). The invitation is reduced at `react` on `invited` (`system.rs:242–249`), with `now` = the fact's instant. `group_activity.rs:100` asserts the constant's value. | as cited | A configured lifetime would leave the controller judging by the wrong number. The remedy is SD-IB-15: the invitation carries `until`, and the controller reads it. Unconfigured, `until = at + 1 800`, the same test, so the requests are identical. |
| **F-IB-4** | `Invitations` is disclosed only to its holder and read only by the rule controller. No client, server or tool reads it. | `git grep -il invitations -- clients server tools tests cognition worldpack` → `cognition/rule-controller` only | Adding a field to `Invitation` changes no client and no digest. It changes the component's schema (1 → 2) and the pack's VERSION. |
| **F-IB-5** | `Affordance` carries `unavailable_reason: Option<Rejection>` (`contracts/src/observation.rs:186–194`). `Offer` carries only `target_available: bool` (`systems/presence/src/interaction.rs:150–215`). Presence derives the reason only from the spatial evaluation (`observe.rs:260–279`). **So no pack can make an offer unavailable *for `PermissionDenied`*.** §2.1 A-6's "offers carry … an unavailable reason" holds for the affordance, not for the offer. | as cited | IL-b adds `Offer::refused(Rejection)`, which presence's verdict reports before the spatial evaluation (SD-IB-12; a dated note on ARC-34). No contract changes: `Affordance::unavailable` already takes any `Rejection`. |
| **F-IB-6** | S11-C's audience function (`mineworld_presence::audience::admits`) and `mineworld perceived` do not exist on `main`. Presence's modules are `codec, component, event, interaction, observe, resolve, system`, and the CLI has no `perceived` subcommand (`tools/cli/src/main.rs:97–237`). | as cited | IL-b proves the narrowed `Visibility` in the envelope. Perceiving it is S11-C's to deliver (QIB-12). |
| **F-IB-7** | A Person's biography is `tools/cli/src/biography.rs`. It reads the save's manifest and its facts. The selection is `biographical(&Manifest)`: each composed capability's `biographical()` (`:168–178`), plus `names(fact, person)`. The save's genesis journal row holds the assembled world, entities and their tags included (`persistence/src/input.rs:47–52`). | as cited | The amended projection (ARC-29 note) takes from the save: the configured section facts (genesis), the tags (the genesis row), and each capability's `FactDecl`s. A pure selection in the SDK decides. |
| **F-IB-8** | The licence policy is `LicencePolicy::default()` in two places: `worldpack/src/requirements.rs:53`, called from `read_with` *before* `configure::read` (`read.rs:116`, IL-a's line follows it), and `tools/cli/src/packs.rs:242` (`packs validate`). `LicencePolicy` already deserializes from `{ allowed: [..] }` with `deny_unknown_fields` and SPDX checking (`packages/src/policy.rs:17–55`); `packages/tests/policy.rs:49–66` is its decode test. | as cited | `configure/packages.yaml` is read straight after `resolve_systems` and before `requirements::resolve`. It is decoded into `LicencePolicy`, with line and column, and passed in. `packs validate` judges a World Pack with that world's policy. |
| **F-IB-9** | IL-a's `RESERVED` (`worldpack/src/configure.rs`, IL-a branch) refuses `classes` and `packages` by name. Three files assert the reservation. The first is `worldpack/tests/configuration.rs` (reserved ×2, and "no installed id is a reserved key"). The second is `tools/cli/tests/configure.rs` (one `validate` case). The third is `configuration_vocabulary.rs`, whose D-14 admissions for `classes` fail if they admit nothing. | IL-a §11.6 IA-C4, D-14 | Those claims change on purpose in IL-b (§12.8): the keys become framework keys, and "no installed id is a framework key" stays. |
| **F-IB-10** | `PackConfiguration::seed(&Seeding, &Configuration)` receives the world and the resolved keys only (`authoring/src/configuration.rs`, IL-a). `Seeding` is shared with sections (`authoring/src/section.rs:136–164`). | as cited | Classes and attachment bytes reach a configuration's `seed` through a new `ConfigurationContext`. `Seeding`, which sections share, does not grow (SD-IB-3). |
| **F-IB-11** | A test-tuning-like pack reduces its configured fact in `react` into a component on every Place (`tests/acceptance/tests/configuration/mod.rs:267–290`, IL-a). The kernel's `System` trait has `react` but no separate reducer (`kernel/src/system.rs:503–620`). | as cited | `interactions!()` generates the configured fact, the per-Place component, and a reduction helper that the pack calls from its `react`. A pack's `declaration()` names them through two generated helpers (SD-IB-9). |
| **F-IB-12** | `run`'s printed history is facts and request outcomes only. The fingerprint is FNV-1a over encoded facts (`tools/cli/src/run.rs:380–409, :470–508`), and no line prints a component or a SystemVersion. SystemVersions live in the save's manifest (`persistence/src/world.rs:76, :121`). | as cited | A VERSION bump or a component schema change moves no `run` digest. Only facts and request outcomes can. |
| **F-IB-13** | Test-only packs cannot be resolved by `Capability::resolve`, so neither `mineworld biography` nor a World Pack can include `test-tuning` (IL-a D-16). | IL-a §11.12 D-16 | The schema's proofs run at library level with `test-tuning` (tests/acceptance). The binary-level proofs run with the two real sections (tools/cli/tests, worldpack/tests). |
| **F-IB-14** | Neither `proptest` nor `quickcheck` is a workspace dependency (`Cargo.toml [workspace.dependencies]`). The tree's deterministic generator idiom is SplitMix64 (`cognition/rule-controller/src/paced.rs:311`, `kernel/tests/long_run.rs:54`). | as cited | QIB-9. |
| **F-IB-15** | 12d's change set (`step-11-bodies.md` §19.1 on `plan/s15-12d-refresh @ 91d8b07`, DESIGN FROZEN). It touches `worlds/{social-cafe,market-town}/**`, `systems/{bodies,item}/**`, a comment in `systems/movement/src/action.rs` and in `worldpack/src/read.rs`, `tests/acceptance/tests/{ac1_composability,seam_vocabulary}.rs`, `tools/cli/tests/{town_bodies.rs,bodies/mod.rs}`, `docs/{DECISIONS,MODULE_SPEC,MVP_STATUS}.md`. Its no-diff list includes `authoring/`, `sdk/`, `cognition/`, `tools/cli/src/`, and every pack but bodies, item and a movement comment. | as cited | No file both PRs edit in code, except one comment line region in `worldpack/src/read.rs`. 12d re-baselines the towns' digests once (overall.md "Parallel build-out" item 5). §12.6. |

## 12.3 The constants: which IL-b makes configurable first, and where every other one goes

Classes: **P** a world parameter (a number a world may choose; a section field with a bound). **E** an
engine or encoding bound (L0: a type's limit, a storage cap, a unit, a codec length). E is never
configurable; a world that wants more needs a new pack version. **V** vocabulary (a category or a
name the pack keys on).

| Pack | Constant (file:line) | Value | Class | Goes to |
| --- | --- | --- | --- | --- |
| **conversation** | `CONVERSATION_GAP` (`system.rs:29`) | 300 s | P | **IL-b** (`gap`) |
| **group-activity** | `INVITATION_LIFETIME` (`component.rs:23`) | 1 800 s | P | **IL-b** (`invitation_lifetime`) |
| conversation | `INTERACTION_RANGE` (`action.rs:16`) | 3 000 mm | P | IL-e: it is a `SpatialRequirement` that offers show, so it changes affordances and belongs with `talk`'s rules |
| conversation | `REMEMBERED_AT_MOST` (`component.rs:18`) | 32 | P | IL-e (with the `remember` knob) |
| conversation | `UTTERANCE_MAX_BYTES` (`utterance.rs:13`) | 480 | E | never |
| group-activity | `INVITE_RANGE` (`action.rs:21`) | 3 000 mm | P | IL-e (as `INTERACTION_RANGE`) |
| group-activity | `ACTIVITY_LENGTH` (`process.rs:11`) | 3 600 s | P | IL-e (S19 §4.5 ruled it a calendar constant needing no remedy) |
| group-activity | `KIND_MAX_BYTES` (`kind.rs:8`) | 32 | E | never |
| relationships | `SPOKE_FAMILIARITY` 10, `ACCEPTED_REGARD` 50, `DECLINED_REGARD` −30, `ACTIVITY_FAMILIARITY` 50, `ACTIVITY_REGARD` 20 (`system.rs:22–30`) | | P | IL-e (per class pair) |
| relationships | `FAMILIARITY` (0, 1 000), `REGARD` (−1 000, 1 000) (`component.rs:11–12`) | | E | never; the parameters above are bounded by them |
| inventory | `PERSON_CAPACITY` (`admit.rs:18`) | 6 | P | IL-f (per holder class, 1 … 64) |
| item-transfer | `GIVE_RANGE` (`action.rs:12`) | 3 000 mm | P | IL-f |
| consumption | `EATEN` "food", `DRUNK` "drink" (`action.rs:10, :13`) | | V→P | IL-f (QIL-20: categories per actor class) |
| item | `CATEGORY_MAX_BYTES` (`category.rs:7`) | 32 | E | never (item has no section, QIL-9) |
| movement | `MAX_STRIDE` (`action.rs:23`) | 2 000 mm | P | IL-g (after 12d, which edits a comment in this file) |
| employment | `HOUR` (`component.rs:11`) | 3 600 | E | never (a unit) |
| schedule | `MIN_SEGMENTS` 2, `MAX_SEGMENTS` 24 (`segment.rs:12–14`), `DAY` 86 400 (`time.rs:9`), `LABEL_MAX_BYTES` 32 (`label.rs:8`) | | E | never |
| naming | `NAME_MAX_BYTES` (`name.rs:7`) | 64 | E | never |
| presence | — | | | no section (QIL-9) |
| **bodies** | `geometry.rs:13–148`: 40 constants (`PERSON_RADIUS`, `NUDGE_MAX`, `KICK_*`, `THROW_*`, `SHOVE_*`, `REST_*`, `GRAVITY`, `LATTICE`, `OBJECTS_MAX` …); `component.rs:15–18` (`SOLIDS_MAX`, `SOLID_HEIGHT_MAX`); `rapier.rs:48–67` (`DT`, `FRICTION`, `RESTITUTION`, `GRAVITY_Z`, wall and slab sizes) | | P and E per `step-18-physics-list.md` §4.4 (26 P) | **IL-c, after 12d** (§12.6). Not one of them is touched by IL-b |

**Why these two first.**
1. They are the two QTW-15 assigns to IL-b, by the primary session's ruling.
2. Both are world-time durations that only their own pack decides from. Neither appears in an offer or
   a `SpatialRequirement`, so making them configurable changes no affordance and needs no rule
   enforcement. They are the smallest real proof of the parameter path.
3. Neither touches bodies, a world, or any file 12d edits.
4. One of them (F-IB-3) has a consumer outside its pack. So IL-b meets, on a small case, the question
   every later conversion faces: who else assumed this number?

## 12.4 Design (SD-IB-1 … SD-IB-17)

| ID | Decision | Rationale |
| --- | --- | --- |
| **SD-IB-1** | **Framework keys.** IL-a's `RESERVED` becomes `FRAMEWORK: [("classes", …), ("packages", …)]`. A framework key is never resolved against the installed set. `classes` is decoded by authoring's `EntityClasses`. `packages` is decoded by `mineworld_packages::LicencePolicy`. Listing either is optional, and a file in `configure/` that is not listed is still refused (IL-a). "No installed pack's id is a framework key" stays a test. | QIA-1 and the E-b note. One seam and two owners, each named. |
| **SD-IB-2** | **`packages`** is read in `read_with` straight after `resolve_systems` and before `requirements::resolve`. It is decoded with `serde_saphyr` (line and column), and the `LicencePolicy` is passed to `requirements::resolve` instead of `LicencePolicy::default()`. It is kept on `WorldPack` (`licence_policy()`). It is **not seeded and not drift-checked**: it governs which packs may compose a world, which is decided again at every read, including every resume. `packs validate <world dir>` judges with that world's policy. `packs validate <pack dir>` keeps the default. | F-IB-8; ARC-55 item 5. A policy is not world state. |
| **SD-IB-3** | **The seeding context.** `PackConfiguration::seed(&Seeding, &Configuration, &ConfigurationContext)`, with `ConfigurationContext { classes: &EntityClasses, attached: &Attached }`. `AuthoredConfiguration::seed` and the loader follow. IL-a's implementors (test-tuning, the probes) gain the argument. No shim: the contract is pre-stable (`CLAUDE.md` §4 rule 12). | F-IB-10. `Seeding`, which sections share, stays as it is. |
| **SD-IB-4** | **Entity classes (ARC-64)** are `configure/classes.yaml`, a list of `{ class, of, tag }`. `class` is a `ClassName` (the `SystemId` grammar). `of` is one of `person`, `place`, `item`, `organization`. `tag` is a `Tag`. Priority is list order. Each type's implicit class is its type name, and those four names are reserved. The file is refused when it defines a class twice, reuses an implicit name, or gives an invalid `of`. It is read before any configuration is decoded. It is **not seeded on its own**: each section's resolved fact copies the classes that section references (SD-IB-8). So an edit to a referenced class is drift, and an edit to an unreferenced one changes nothing and is not drift. An entity's class is found at lookup: the first entry whose `of` is its type and whose `tag` is among its `Tags` (immutable after genesis, A-1); otherwise its implicit class. | §4.2, QIL-3, QIL-6, QIL-7. Classes are not state. |
| **SD-IB-5** | **`data:` attachments.** `authoring::Attachment` is a relative path under the World Pack's `data/`, checked when it deserializes, at its line and column: relative; first component `data`; every component normal (no `..`, no root, no prefix). A configuration lists its attachments (`PackConfiguration::attachments(&Configuration) -> Vec<&Attachment>`, default none). After decoding, the loader reads each one into `Attached`. It refuses one that is missing, one that resolves outside the pack root after canonicalization (a symlink), and one over `ATTACHMENT_MAX_BYTES` = 4 MiB (QIB-6). `seed` receives the bytes, and the owner decodes them and states what it needs in its own fact. Files under `data/` that are not referenced are allowed. | QTW-7 option (b). The drift check compares the seeded facts, so a changed file is drift with no new mechanism. |
| **SD-IB-6** | **The section shape (ARC-63)**, one YAML shape for every pack, as §4.4: `extends`, `default` (`permit` \| `forbid`), `rules`, `parameters`, `consequences`, `regions`. Every key is optional. `deny_unknown_fields` applies throughout. A selector is a declared class, a type's implicit class, or `*`. Only roles the pack declares for that action or fact may be named. Rule effect: `permit` \| `forbid`. Consequence fields: `audience` (`public` \| `place` \| `participants`), `biography` (`on` \| `off`), and the pack's knobs. `regions` is keyed by place key; each region holds `parameters`, `rules` and `consequences`. | One schema, many owners (§4.4). |
| **SD-IB-7** | **What a pack declares**, through `InteractionSection`: `type Parameters` (made by `parameters!`, with each field's type, bound and default, plus its all-optional partial twin); `type Knobs` (`()` for none); `ACTIONS: &[ActionDecl { action, roles, regional }]`; `FACTS: &[FactDecl { fact, roles, default_audience, narrowest, biography_configurable }]`; `REFERENCE: &[(&str, &str)]`, whose `default` entry must exist. `FactDecl.roles` maps each role to an envelope position, `subjects[i]` or `participants[i]`, so the projection can find roles from the envelope alone. | §4.4. The SDK supplies the shape; the pack supplies the meaning. |
| **SD-IB-8** | **Resolution** (pure, in `sdk::interactions::resolve`) follows §4.5. L0 bounds are checked at decode. L1 is the pack's `default`. L2 is `extends`, of the pack's own reference lists only, in a chain of at most 4, with no cycle. L3 is the world's section. L4 is regions. A higher level replaces a lower level's entry with the same key (an action or fact plus its selectors). Then specificity: the number of named roles, where an implicit class counts as named. Then, between rules, forbid beats permit. **Ambiguity is refused at load.** Two parameter entries, or two consequence entries, of equal specificity that overlap and give one field different values are refused. Two entries overlap when, for every role, their selectors are equal, or one is `*`, or one is an implicit class whose type the other's class selects. The refusal names both entries (file, list, index). The result is `Resolved<S>`: sorted vectors, the base, each region, and the referenced classes. It is serialized with the pack's codec as **one genesis fact `<pack>-interactions-configured`**, `Visibility::SystemInternal`, with no subjects (ARC-61 item 8). | §4.5, IL-I7, QIL-5. A pure function of the files, so the same input always gives the same bytes. |
| **SD-IB-9** | **`interactions!()`**, inside `impl SystemPack`. It implements `PackConfiguration` for the pack: `Configuration = Section<S>`, `FACTS` = the configured fact, `references` = region place keys, `seed` = resolve and state. It does so through `configures!()`'s path, so IL-a's seam is used unchanged. It generates the event type `<pack>-interactions-configured` and an owned component `<pack>-interactions` on every Place, holding the base plus that place's regions. It also generates `declare(SystemDeclaration) -> SystemDeclaration`, which adds `owning`, `emitting` and `subscribing_to`, `install_component(&mut Declarations)`, and `reduce(&mut WorldView, &EventEnvelope) -> Result<bool, KernelError>`, which the pack's `react` calls first. The installed set's `Capability` gains `interaction_section() -> Option<&'static SectionDecl>` (the configured fact type, `ACTIONS`, `FACTS`) for the tools. The macro's internal form (generic types or generated concrete ones) is left to implementation, since `owned_component!`'s support for generics is unverified. The observable contract above is what is frozen. | F-IB-11; like `owns_section!()` and `configures!()`. A pack that has a section cannot also have a plain configuration: its configuration *is* its section. |
| **SD-IB-10** | **Lookups** (pure; `sdk::interactions::lookup`): `permits::<S>(read, place, action, &Roles) -> Result<(), Rejection>`, `parameters::<S>(read, place, &Roles) -> S::Parameters`, and `consequence::<S>(read, place: Option<PlaceId>, fact, &Roles, owner_default: Visibility) -> Consequence<S::Knobs>`. With no component on the place (unconfigured), each returns the pack's compiled `default` and reads nothing else: `Ok(())`, the default parameters, and `owner_default` with the compiled biographical flag. A lookup without a place uses the base, which every Place's copy holds. Lookups are binary searches. | §4.4–4.6, IL-I1. |
| **SD-IB-11** | **Audience narrowing** runs along `Public ⊇ Place(p) ⊇ Participants`, never below the owner's `narrowest`. `Entities(S)` and `SystemInternal` are never produced by a list. A list asking for a wider audience than the owner's default is refused at load (IL-I3). | §4.7, INV-13. `Entities` would need the list to name entities, which it cannot. |
| **SD-IB-12** | **A pack-stated refusal on an offer.** `Offer::refused(self, Rejection) -> Self` (presence). `verdict` reports a refusal before it evaluates the requirement, as `Affordance::unavailable(.., reason)`. The requirement is still shown and a complete offer's payload still travels (ARC-34). This is a dated note on ARC-34, with no contract change. In IL-b only test-tuning uses it. IL-e … IL-g use it through `permits`. | F-IB-5, QIL-12, QIL-13: the offer and the dispatch give the same answer, in the same order. |
| **SD-IB-13** | **The biography projection (ARC-29 amended).** `sdk::interactions::biography::selected(fact, person, compiled: &BTreeSet<EventTypeId>, configured: &Configured)` answers whether a fact enters a person's biography. `Configured` is assembled from the save's genesis configured-section facts, the tags in the genesis journal row, and each composed capability's `FactDecl`s. With nothing configured it is ARC-29 exactly. `tools/cli/src/biography.rs` builds `Configured` and calls the selection. It still writes nothing and resumes nothing. | F-IB-7, QIL-15. |
| **SD-IB-14** | **`mineworld interactions <world> [--place KEY] [--json]`** reads the World Pack only. It prints each configured section resolved (base, then each region), each entity's class, and for every pack with a section but no configuration "default (compiled)". The JSON uses sorted keys and is stable. tools/cli reaches the SDK through `mineworld-worldpack`'s re-exports, so no `Cargo.toml` changes. | §4.10 item 5, R-IL-7. |
| **SD-IB-15** | **The invitation carries its expiry.** `Invitation { from, kind, at, until }`. At `react` on `invited`, `until = at + parameters::<GroupActivity>(…, roles {actor: inviter, target: invitee}, place = the fact's place).invitation_lifetime`. `is_open_at(now)` = `at ≤ now ≤ until`, which is exactly the old test when `until = at + 1 800`. The rule controller reads `until` and no longer imports `INVITATION_LIFETIME`. `Invitations`' schema goes 1 → 2, and group-activity's VERSION 1 → 2. `INVITATION_LIFETIME` stays public as the default's value. `perception.rs`'s module doc is reworded. | F-IB-3, F-IB-4. The world states what it means, and the controller reads the observation (I-3, ARC-27). |
| **SD-IB-16** | **conversation's section.** `Parameters { gap: Seconds 1 … 86 400 = 300 }`, with roles `actor` (speaker), `target` (listener) and the speaker's place. There are no `ACTIONS` and no `FACTS` in IL-b. `within_the_gap` takes the looked-up gap. VERSION 1 → 2. **group-activity's section.** `Parameters { invitation_lifetime: Seconds 1 … 86 400 = 1 800 }`, with no actions and no facts. Each pack's README documents its section. One pinned test per pack holds `(VERSION, default)` (§4.10 item 3). | QTW-15, QIB-13. Parameters only: no affordance changes. |
| **SD-IB-17** | **Refusals** are new `PackError` variants, each naming the file and, where serde reaches it, the line and column: `ClassesInvalid`, `ClassUndefined { section, entry }`, `RoleNotDeclared`, `ActionNotDeclared`, `FactNotDeclared`, `AudienceWidened`, `BiographyNotConfigurable`, `AmbiguousEntries { first, second }`, `ExtendsUnknown`, `ExtendsCycle`, `ExtendsTooDeep`, `RegionUnknownPlace` (through IL-a's `references`), `AttachmentOutside`, `AttachmentMissing`, `AttachmentTooLarge`, `LicencePolicyInvalid`. Bound and shape errors are serde's own, at their line and column. Whether a refusal raised after decoding (an undefined class, an ambiguity) can carry a line and column depends on the YAML decoder. The design requires the file plus a list name and index. | `MODULE_SPEC.md` §4.1: refused by name, never ignored. |

**Byte-identity argument for an unconfigured world** (IL-I1):
1. No `<pack>-interactions-configured` fact is seeded, so no `react` inserts a component.
2. With no component, every lookup returns the compiled default (SD-IB-10): `gap` 300 and
   `invitation_lifetime` 1 800.
3. `until = at + 1 800` keeps the validate test and the pruning test exact (SD-IB-15). The controller's
   `now ≤ until` equals its old `age ≤ 1 800` for `now ≥ at`, which every observation satisfies.
4. No offer is refused, because no pack calls `Offer::refused` except test-tuning.
5. So the facts and the request outcomes are unchanged. The VERSION bumps and the component's schema
   are outside `run`'s output (F-IB-12).

## 12.5 Acceptance (decided before measuring, `ARC-23`)

Rules (IL-a's):
- Every guarded criterion names its mutation.
- A mutation is applied in the working tree, observed to fail by name, then reverted. `git status` and
  `git grep MUTATION -- '*.rs'` are recorded afterwards.
- Expected values are literals from the test's own layout, or E-IB-0's captured values.

```text
IB-1  Byte identity (IL-I1). E-IB-0 is captured on the base before any code. On the final head:
        social-cafe and market-town `mineworld run <w> --headless --seed 7 --days 300`: the output sha
        (every line but `wall`), the fact count, the fingerprint and faults 0 equal E-IB-0
        (main @ 78ca5ae values, if 12d has not merged: ad49c723…c64b, 365b50e0…1d1d);
        bodies-yard 30 days seed 7: its sha equals E-IB-0 (bd6a1002…80e6 on main @ 78ca5ae);
        bodies' long_run and long_run_objects second-process bytes equal E-IB-0;
        the three worlds' `mineworld validate` output: cmp-identical.
      M-IB1a: conversation's compiled default gap 300 → 1 → the social-cafe sha differs (the gap is
      read, and the instrument sees it).
      M-IB1b: group-activity's compiled default lifetime 1 800 → 900 → the social-cafe sha differs
      (QB-1: invitations to lower seats expire).
IB-2  Explicit default (IL-I2). On scratch copies of social-cafe, made at test time by the scratch
      helper, 30 days seed 7 with --save: the copy with `configure: [conversation, group-activity]`, each
      file `extends: default` and nothing else, against the unconfigured copy. The facts are equal
      except exactly two genesis configuration facts, and every later event id, and every id a fact
      refers to, is offset by exactly 2. Request outcomes are equal.
      M-IB2: let an explicit section with no `parameters` resolve its base from the partial type's
      empty values instead of the reference list → gap and lifetime fall back to their lower bound →
      the test fails at the first differing fact.
IB-3  Configured, through the binary (tools/cli/tests/interaction_runs.rs, scratch copies of
      social-cafe, 30 days seed 7). Fixed before measuring:
      (a) `gap: 3600`: `conversation-started` count strictly below the unconfigured copy's; every seat
          has ≥ 1 accepted `talk` in every 10-day bucket (activity holds); faults 0.
      (b) `invitation_lifetime: 60`: `requests accept-invitation rejected …` is 0 (the controller never
          answers an expired invitation); `invitation-accepted` count strictly below the unconfigured
          copy's; faults 0.
      M-IB3: the rule controller compares the age with the constant 1 800 again → (b)'s rejected count
      is above 0 and the test fails.
IB-4  Region and class scoping, on the real conversation pack (worldpack/tests/interaction_sections.rs,
      scratch social-cafe, scripted requests through World::dispatch):
        `regions: { cafe: { parameters: [ { gap: 3600 } ] } }`: two `talk`s 400 s apart in the café
        give one `conversation-started`; the same two in another place give two;
        with classes.yaml `{ class: regular, of: person, tag: <a tag one person carries> }` and
        `parameters: [ { actor: regular, gap: 3600 } ]`: only that speaker's second talk continues the
        conversation.
      M-IB4a: lookups ignore the region (always the base) → the café case fails.
      M-IB4b: class resolution ignores `of` (any entity with the tag) → a case with a Place carrying the
      same tag fails. That case exists in the test, fixed now.
IB-5  The schema, with test-tuning (tests/acceptance/tests/interaction_schema.rs, World::dispatch and
      presence's observe):
      (a) a `forbid` of `advance` for actor class A against target class B: the dispatch is refused
          `PermissionDenied`; the observation's affordance is unavailable with reason PermissionDenied,
          its requirement still shown; the same request against another class is accepted;
      (b) `default: forbid` with one `permit`: only the permitted pair is accepted;
      (c) a scoped parameter changes `advanced.by` only for its class, a region only in its place;
      (d) `audience: participants` on test-tuning's fact → the emitted envelope's Visibility is
          Participants (owner default Place); `audience: public` is refused at load (widening);
      (e) `biography: off` for class A → `biography::selected` excludes A's facts and keeps B's, and
          the fact is in the log for both.
      M-IB5a: permit wins ties → (a)'s forbid-overrides case fails. M-IB5b: the offer omits `permits`
      → (a)'s affordance case fails. M-IB5c: `selected` ignores `Configured` → (e) fails.
IB-6  Totality and order independence (sdk/rust/tests/interactions.rs), on a synthetic declaration (3
      roles, 4 classes, 2 places, 3 parameter fields, 1 fact). 10 000 sections are generated by
      SplitMix64 from a fixed seed. Each one is either refused at load, or every lookup over every
      class tuple × place × fact returns a value. For sections of ≤ 5 entries, every permutation of the
      entry order, and 100 seeded shuffles above that, gives byte-identical `Resolved` and identical
      answers. Recorded: how many were refused, and by which variant.
      M-IB6: specificity ties broken by authoring order instead of refused → the permutation check
      fails.
IB-7  Load-time refusals: each SD-IB-17 variant, through WorldPack::read on a scratch world (probe-
      labelled in-crate where only test-tuning could reach it, IL-a D-8's rule). Also, through the real
      `mineworld validate`: `rules:` in conversation.yaml (an undeclared action), `gap: 0` (a bound,
      with line and column), an undefined class, an ambiguous pair (both named), `data: ../x`.
      M-IB7: remove the ambiguity check → its test fails.
IB-8  Drift (IL-I8): `run --save`, then the file edited, then `run` resumes and `replay` → both refused
      `ConfigurationDrift` naming the pack, for: a changed gap; a changed referenced class in
      classes.yaml; an attachment's bytes changed (test-tuning, in-crate, through
      check_configuration). A changed classes.yaml entry that no section references is not drift.
      A changed packages.yaml is not drift: the next read uses it.
      M-IB8: the resolved fact omits its referenced classes → the class-edit case is accepted and the
      test fails.
IB-9  packages: `configure: [packages]` with `allowed: [Apache-2.0]` → `mineworld validate` and
      `mineworld packs validate <world>` refuse, naming a bundled MIT pack, its expression and the
      allowed list; with `allowed: [MIT]` both pass; the genesis facts are identical to the same world
      without the key (not seeded); `allowed: [Not-A-Licence]` is refused at its line and column.
      M-IB9: requirements::resolve passed the default again → the Apache-only case passes and the test
      fails.
IB-10 data: attachments (test-tuning gains `table: data/table.csv`, three integers a line): its fact
      carries the decoded rows; outside, missing and over-size refused by name.
      M-IB10: `seed` is handed empty bytes → the decoded-rows assertion fails.
IB-11 `mineworld interactions`: on a scratch social-cafe with both sections, a region and a class, the
      text and the --json output show the resolved base, the café's region, each person's class, and
      "default (compiled)" for packs with no configuration; the --json is byte-stable over two runs;
      the World Pack directory is unchanged afterwards (a recursive listing with sizes and mtimes,
      compared before and after).
IB-12 Vocabulary (IL-I10 as restated, QIB-2): configuration_vocabulary.rs is extended to authoring/
      src/{configuration,classes,attachment}.rs, sdk/rust/src/interactions/**,
      worldpack/src/configure.rs, tools/cli/src/{interactions,biography}.rs and IL-b's framework
      tests. None of them may contain the seam scan's physics words, or a word beginning talk, spoke,
      convers, give, buy, sell, trade, eat, drink, kick, throw, shove or invit. The schema's own words
      (permit, forbid, class, biograph, audience) are allowed there now.
      M-IB12: plant `// invit` in lookup.rs → the scan fails naming file and line.
IB-13 Unchanged guards: no diff under kernel/, contracts/, persistence/, server/, clients/, worlds/;
      no System Pack diff outside presence's two files, conversation and group-activity;
      ac1_composability, precursor_vocabulary and seam_vocabulary unedited and passing; the root
      Cargo.toml and Cargo.lock unchanged (git diff --stat recorded).
IB-14 Cost (R-IL-5), social-cafe 300 days seed 7, dev profile, sequential on one idle machine, the
      median of 3 walls each: head unconfigured ≤ base × 1.05; a configured copy (both sections, a
      region on every place, two classes) ≤ head unconfigured × 1.05. If the base's own three walls
      spread by more than 5 %, the result is INCONCLUSIVE: re-run once, then report to the primary
      session rather than retry.
IB-15 The full gate on the final head: cargo fmt --check, check, clippy -D warnings, test (workspace),
      both doc scripts, check_scratch.py scan; CI green on the exact PR head.
```

## 12.6 Sequencing with 12d, and every shared file

**Decision: IL-b keeps every bodies constant out, and runs in parallel with 12d.** Bodies' constants
move in IL-c, which stays after 12d as §6.1 says (12d takes bodies to VERSION 4 and re-baselines the
towns). IL-b edits no world, so it queues no content change into 12d's single re-baseline (12d
QD-17).

| Shared file | 12d | IL-b | Rule |
| --- | --- | --- | --- |
| `worldpack/src/read.rs` | FU-12a-1: one comment reworded | the `packages` read before `requirements::resolve` | different lines; whichever merges second merges |
| `docs/DECISIONS.md`, `docs/MODULE_SPEC.md` §4.1, `docs/MVP_STATUS.md` | notes on ARC-35, ARC-37, ARC-39; `item`'s name; `body`'s doorway | ARC-63 … 65, DEP-28, notes on ARC-29/34/55/61; §4.1, §4.2 | appended blocks; keep both |
| `tests/acceptance/tests/seam_vocabulary.rs`, `ac1_composability.rs` | edited | **not edited** | none |
| the towns' digests | re-baselined once | must equal the base's | **if 12d merges while IL-b is in implementation, main is merged into IL-b and E-IB-0 is re-captured from main** (IL-a's E-IA-13 precedent). IB-1 compares against main's values at IL-b's merge, never against a value 12d replaced |
| bodies-yard, long_run, long_run_objects | unchanged by 12d's design (no bodies rule changes; §19.1 non-goals) | must be unchanged | as above |

**Other lanes.**
- **S19 TW-d** needs `data:` (QTW-7). TW-a and TW-b need only IL-a's seam, but their `seed` gains the
  context argument (SD-IB-3) if they are implemented after IL-b merges. If they merge first, IL-b
  updates their implementors as it does IL-a's.
- **S11-C** (audience) is independent. QIB-12 says where the perception proof goes.
- **S12/S14 clients** need nothing: an affordance can already be unavailable for any `Rejection`, and
  a client that does not know `PermissionDenied` greys the action out.

## 12.7 Commit plan

Every commit lists implementation, validation and review as separate items. Each is ticked only with
evidence in §12.13.

### IB-C0 — Design (this section), docs only

**Goal.** A reviewable design before code (`CLAUDE.md` §2.2).
**Scope.** This section; one pointer in §6.2's IL-b row. **Boundary.** Markdown only.
- [x] Implementation: §12 written from the audit in §12.2.
- [x] Validation: `python3 scripts/check_doc_headings.py`, `python3 scripts/check_decision_ids.py`
  (E-IB-d).
- [x] Review: every finding cites a file and line; every guarded criterion names a mutation; the 12d
  shared lines are named. Self-review only; the primary session's freeze is pending.

### IB-C1 — Specs before code

**Goal.** The decisions and the specification exist before the code they govern.
**Scope.**
- `DECISIONS.md`: ARC-63 (the schema: SD-IB-6 … 11), ARC-64 (classes: SD-IB-4), ARC-65 (consequence
  routing: §4.7, SD-IB-11, SD-IB-13), DEP-28 (§3: built on Cedar's semantics; flecs, OPA/Rego, Casbin
  and the Cedar engine declined); notes on ARC-29, ARC-34 (SD-IB-12), ARC-55 (SD-IB-2) and ARC-61
  (SD-IB-1, 3, 5). Each id re-checked on every `origin/*` branch before writing.
- `MODULE_SPEC.md` §4, §4.1, new §4.2; `systems/README.md`; `MVP_STATUS.md`.
**Non-goals.** Code. **Depends on.** IB-C0 frozen.
- [ ] Implementation: the records and sections above.
- [ ] Validation: both doc scripts; `git grep` shows no "reserved for" wording left in the specs for
  `classes`/`packages`, except in ARC-61's history.
- [ ] Review: no defined term redefined or given a synonym (entity class, section, selector, role,
  rule, parameter block, consequence, reference list, region are §4.1's words); ARC-63 states "a list
  cannot grant"; ARC-65 states the four layers of "enters history".

### IB-C2 — `authoring`: the seeding context, classes, attachments

**Goal.** The contract a configuration's `seed` receives (SD-IB-3, 4, 5).
**Scope.** `configuration.rs` (context, `attachments`, `seed`'s argument); NEW `classes.rs`,
`attachment.rs`; `lib.rs`; IL-a's implementors updated, since otherwise the workspace does not compile:
test-tuning (`tests/acceptance/tests/configuration/mod.rs`), the probes in `authoring` and
`worldpack/src/configure/tests.rs`. `worldpack/src/configure.rs::seed` passes an empty context.
**Non-goals.** Reading `classes.yaml` or any attachment (IB-C3).
- [ ] Implementation: as above.
- [ ] Validation: `cargo test -p mineworld-authoring -p mineworld-worldpack -p mineworld-acceptance`.
  Unit tests: `ClassName`/`of` refusals; implicit names refused; an attachment path refused when
  absolute, when it contains `..`, or when it is not under `data/`, each at its line and column through
  `serde_saphyr` in worldpack (authoring has no YAML dependency, IL-a D-1). IB-1 is not yet run (no
  behaviour reachable).
- [ ] Review: the context is read-only; `Seeding` is unchanged; no shim kept for the old signature.

### IB-C3 — `worldpack`: framework keys, attachments, the policy

**Goal.** SD-IB-1, 2, 5 in the loader.
**Scope.** `configure.rs` (FRAMEWORK, classes read first, attachments read, the context built);
`read.rs` (packages before `requirements::resolve`; `licence_policy()`); `requirements.rs` (the policy
as an argument); `error.rs`; `format.rs` (`FoundConfiguration.attached`); `lib.rs`;
`tools/cli/src/packs.rs` (a World Pack is judged with its policy). Tests:
`worldpack/tests/configuration.rs` (the reserved tests become framework-key tests, §12.8),
`configure/tests.rs`, and `tools/cli/tests/configure.rs` (its reserved case → the packages case of
IB-9).
- [ ] Implementation: as above.
- [ ] Validation: IB-9 and M-IB9; IB-10's refusals (outside, missing, over-size) with a probe; the
  undeclared-file refusal still covers `configure/classes.yaml` when it is not listed. The three worlds'
  `validate` output is `cmp`-identical to E-IB-0.
- [ ] Review: the order in `read_with` (resolve_systems → packages → requirements → configuration);
  the policy is neither seeded nor compared; `check_configuration` re-reads attachments from the same
  root.

### IB-C4 — presence: a pack-stated refusal on an offer

**Goal.** SD-IB-12.
**Scope.** `systems/presence/src/interaction.rs` (`Offer::refused`, the field, its accessor);
`observe.rs` (`verdict`: refusal first); one presence test. No VERSION change: an offer is not state.
- [ ] Implementation: as above.
- [ ] Validation: a presence test where a refused offer is unavailable with its reason, its
  requirement shown, its payload carried, even when the target is out of range (the refusal wins);
  `complete_affordances` and every presence test pass unedited.
- [ ] Review: no pack but test-tuning calls it; `Affordance` and `contracts/` untouched.

### IB-C5 — `sdk::interactions`: declarations, sections, resolution

**Goal.** SD-IB-6, 7, 8, 11, 17: the pure half.
**Scope.** NEW `sdk/rust/src/interactions/{mod,decl,selector,section,resolve}.rs` and `tests.rs`;
`parameters!`; `lib.rs` exports; NEW `sdk/rust/tests/interactions.rs` (IB-6).
- [ ] Implementation: as above.
- [ ] Validation: unit tests for each precedence rule (level replacement, specificity, forbid
  overrides, the overlap definition including implicit classes); each refusal; IB-6 with M-IB6; one
  `serde_saphyr` decode with line and column through worldpack's in-crate probe.
- [ ] Review: no `HashMap` iteration, no float, no clock; every `Vec` is sorted before it is serialized;
  `resolve` is total over its input or refuses.

### IB-C6 — `sdk::interactions`: lookups, `interactions!()`, the capability aggregate

**Goal.** SD-IB-9, 10, 13 (the selection).
**Scope.** `lookup.rs`, `biography.rs`; `interactions!()` in `pack.rs`; `installed.rs` (`@catalog`:
`interaction_section()`); `__private`.
- [ ] Implementation: as above.
- [ ] Validation: lookups on a probe world (unconfigured → defaults without a component read beyond
  the place's; configured → base, region, class); the selection with and without `Configured`.
- [ ] Review: the macro expands to what SD-IB-9 says; a pack without `interactions!()` is unaffected
  (sdk tests, the installed set's tests unedited).

### IB-C7 — The schema proven with test-tuning

**Goal.** IB-5, IB-7's in-crate half, IB-8's attachment half, IB-10.
**Scope.** `tests/acceptance/tests/configuration/mod.rs`. test-tuning becomes `interactions!()`: its
`step` moves into its `Parameters`, it gains a role-bearing `advance` with a target, one fact
`advanced` with a declared audience and a configurable biography, a `PerceptionProvider` offering
`advance` through `permits`, and `table: data/…`. Updated: `configuration_seam.rs`. NEW:
`interaction_schema.rs`.
- [ ] Implementation: as above.
- [ ] Validation: IB-5 (a)–(e) with M-IB5a/b/c; IB-10 with M-IB10; IL-a's IA-2, IA-4 a and IA-7 still
  pass, with any edit to them recorded (§12.8).
- [ ] Review: test-tuning still names no pack vocabulary; its tests assert behaviour (refusals,
  envelopes, selections), not getters.

### IB-C8 — Tools: the biography projection and `mineworld interactions`

**Goal.** SD-IB-13 (the CLI half), SD-IB-14.
**Scope.** `tools/cli/src/{biography,main}.rs`; NEW `interactions.rs`; NEW
`tools/cli/tests/interactions.rs`; `worldpack/src/lib.rs` re-exports.
- [ ] Implementation: as above.
- [ ] Validation: `tools/cli/tests/biography.rs` passes unedited (unconfigured = ARC-29); IB-11; a
  structural test that `biography` builds `Configured` from the genesis facts (QIB-11).
- [ ] Review: both commands are read-only; no new Cargo dependency.

### IB-C9 — conversation reads its gap from its section

**Goal.** SD-IB-16 (conversation).
**Scope.** `systems/conversation/src/{interactions.rs NEW, system.rs, lib.rs}`; README; the pinned
`(VERSION, default)` test; one test with an explicit gap.
- [ ] Implementation: `interactions!()`; `declaration()` through `declare`; `react` calls `reduce` first;
  `continues_a_conversation` takes the looked-up gap; VERSION 2.
- [ ] Validation: conversation's tests pass (`conversation_and_presence.rs:710` unedited: the default is
  300); IB-4 (worldpack/tests/interaction_sections.rs) with M-IB4a/b; `rules:` in its section refused
  (IB-7).
- [ ] Review: no other use of `CONVERSATION_GAP` remains except as the default's value and in docs.

### IB-C10 — group-activity reads its lifetime; the controller reads `until`

**Goal.** SD-IB-15, SD-IB-16 (group-activity). Atomic: the field and its one outside reader change
together.
**Scope.** `systems/group-activity/src/{interactions.rs NEW, component.rs, system.rs, perception.rs,
lib.rs}`; README; `cognition/rule-controller/src/{social,social_tests}.rs`.
- [ ] Implementation: as SD-IB-15; schema 2; VERSION 2.
- [ ] Validation: group-activity's and rule-controller's tests (`social_tests.rs` builds invitations with
  `until`); `git grep INVITATION_LIFETIME -- cognition` is empty; a pack test that an invitation from a
  configured world expires at its own `until`.
- [ ] Review: `is_open_at` is the old test exactly when `until = at + 1 800` (an `at`/`now` table in a
  unit test); the controller judges nothing it cannot read.

### IB-C11 — Close: byte identity, the runs, cost, the scan, the gate, the ledger

**Scope.** IB-1 (with M-IB1a/b), IB-2 (M-IB2), IB-3 (M-IB3), IB-8's binary half (M-IB8), IB-12
(M-IB12), IB-13, IB-14, IB-15; NEW `tools/cli/tests/interaction_runs.rs`;
`configuration_vocabulary.rs`; `MVP_STATUS.md`; the ledger and handoff.
- [ ] Implementation: as above.
- [ ] Validation: each criterion with evidence in §12.13.
- [ ] Review: every changed path is in §12.1's change set; deviations recorded in §12.14.

## 12.8 Test ownership

| Test | Owner | Edited by IL-b |
| --- | --- | --- |
| `worldpack/tests/configuration.rs` (reserved ×2; no installed id reserved) | IL-a | **yes, claim changed on purpose**: framework keys are accepted and routed; "no installed id is a framework key" kept |
| `tools/cli/tests/configure.rs` (reserved `validate` case) | IL-a | **yes, claim changed**: becomes IB-9's case |
| `tests/acceptance/tests/configuration/mod.rs`, `configuration_seam.rs` | IL-a | **yes**: test-tuning becomes a section (IB-C7); IA-2/IA-4 a/IA-7's assertions kept, any rewording recorded |
| `tests/acceptance/tests/configuration_vocabulary.rs` | IL-a | **yes**: D-14's `classes` admissions removed, IL-b's files added (IB-12) |
| `systems/group-activity/tests/group_activity.rs:100` (`INVITATION_LIFETIME == 1800`) | group-activity | no: the default's value is unchanged |
| `systems/conversation/tests/conversation_and_presence.rs` | conversation | no |
| `cognition/rule-controller/src/social_tests.rs` | rule-controller | yes: `Invitation::new` gains `until` |
| `tools/cli/tests/{biography,run,restart,run_restart,bodies_yard_restart,market_town}.rs` | cli | **no** |
| `tests/acceptance/tests/{ac1_composability,precursor_vocabulary,seam_vocabulary,complete_affordances}.rs` | acceptance | **no** |
| new: `sdk/rust/tests/interactions.rs`, `worldpack/tests/interaction_sections.rs`, `tools/cli/tests/{interactions,interaction_runs}.rs`, `tests/acceptance/tests/interaction_schema.rs` | IL-b | new |

Layers: static (fmt, clippy, the vocabulary scan); unit (sdk, authoring, presence); integration through
`World::dispatch` and the loader (worldpack, acceptance); real lifecycle through the binary (IB-1, 2, 3,
8, 14). Real-LLM Gate: **N/A**, since no language model is involved (`CLAUDE.md` §5). CI: IB-15.

## 12.9 Is any of this material?

- No kernel, contract, persistence, server or client change. A need for one is a material stop.
- No `run` digest moves. A moved digest is a material stop, unless 12d moved it on main first (§12.6).
- Presence's API grows by one method (QIB-3), and its behaviour is unchanged for every existing offer.
- A cognition crate changes (QIB-4). It reads more of the observation and nothing else.
- IL-a's seam contract changes `seed`'s signature (SD-IB-3). This is pre-stable, with no shim.
- No world content changes (QIB-5).

## 12.10 Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-IB-1 | The schema is built for rules and consequences that no real pack uses until IL-e: premature abstraction (`CLAUDE.md` §4 rule 11). | Accepted at step level (R-IL-6: the SDK is complete before parallel conversions). test-tuning exercises every part. IL-e is the first real consumer and may amend the schema through a recorded deviation. |
| R-IB-2 | `interactions!()` cannot express a per-pack owned component generically. | SD-IB-9 freezes the contract, not the expansion. Generated concrete types are the fallback. |
| R-IB-3 | The controller change alters a journal. | IB-1 on both towns; SD-IB-15's equivalence table test; M-IB3. |
| R-IB-4 | 12d merges mid-implementation and moves the references. | §12.6's re-capture rule. IB-1 compares against main at merge. |
| R-IB-5 | The ambiguity refusal is too strict: authors meet "ambiguous" for lists that read naturally. | The refusal names both entries and says to add a more specific entry. `mineworld interactions` shows the result. IL-h's manor list is checked against it before IL-h freezes. |
| R-IB-6 | A refusal raised after decoding (an undefined class, an ambiguity) loses its line and column. | SD-IB-17 requires the file, list and index; line and column where reachable. |
| R-IB-7 | A large attachment inflates the genesis fact (S19's ~40 KB). | The 4 MiB cap; S19's 1 MiB `check` warning; the owner decides what to keep. |
| R-IB-8 | One PR of 12 commits is heavy to review. | The commits are ordered so the PR can be cut after IB-C8 (schema) and the rest moved to a second PR at freeze (QIB-1). |
| R-IB-9 | IL-a is not merged at freeze. | IL-b's base is "main after IL-a". The freeze names the commit, and IL-b does not start before it. |
| R-IB-10 | The step-18 EOF conflict between this docs PR and IL-a's. | Placement note at the top of §12. |

## 12.11 Questions (QIB-1 …)

**[OM]** marks operator-material questions. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QIB-1 [OM]** | IL-b's scope is §6.2's row plus five items: the two sections (QTW-15), `data:` (QTW-7), `packages` (QIA-1), the offer refusal (F-IB-5) and the invitation's `until` (F-IB-3). Ship them as one PR, or split after IB-C8 into IL-b (schema, keys, attachments, tools) and IL-b2 (the two sections)? | **One PR.** The two sections are what make IL-I1 mean something in IL-b: without a real pack converted, byte identity is trivially true. The cut point after IB-C8 remains available if the primary session prefers. |
| **QIB-2 [OM]** | Restate IL-I10 for IL-b: the *framework* files (authoring, sdk, worldpack, tools) name no pack vocabulary, and the two packs' own crates gain their sections. Step §6.2's "the primary session reviews the schema on its synthetic pack before any pack converts" becomes this PR's review. | **Yes.** QTW-15 already put real packs into IL-b. |
| **QIB-3 [OM]** | Presence's `Offer` gains `refused(Rejection)`, reported before the spatial evaluation (a dated note on ARC-34, no contract change). | **Yes, in IL-b.** Every rule in IL-e … IL-g needs it, and the parallel PRs must not each add it. |
| **QIB-4** | group-activity's `Invitation` carries `until`; the paced rule controller reads it instead of `INVITATION_LIFETIME`; schema 2, VERSION 2. | **Yes.** Otherwise a configured lifetime leaves the controller judging by a number the world does not use. |
| **QIB-5 [OM]** | QTW-15 also said "set larger values in interactive World Packs". IL-b edits no world: headless defaults stay (INV-TW-1), and no content change is queued into 12d's one re-baseline. The values are set by the PR that makes a world interactive at scale (S19 TW-c), or by a content PR after 12d. Each of those moves that world's digest once, and records it. | **Yes, defer.** |
| **QIB-6** | `data:` attachments: only under `data/`; a 4 MiB hard cap; bytes handed to `seed`; drift through the seeded fact; unreferenced files under `data/` allowed. | **Yes.** |
| **QIB-7** | `packages`: read before requirements resolve; not seeded, not drift-checked; `packs validate` judges a World Pack with its own policy. | **Yes** (the E-b note). |
| **QIB-8** | `classes.yaml` must be listed in `configure:` to be read; not seeded on its own; copied into each section that references it, so only a referenced class's edit is drift. | **Yes.** |
| **QIB-9** | IB-6's property test: a SplitMix64 generator with exhaustive small permutations (no new dependency), or `proptest` as a dev-dependency (shrinking; a DEP record). | **Own generator.** One test; the tree's idiom; no new dependency (`REUSE_POLICY.md` §17: not forcing a framework for one test). Revisit if IL-c needs shrinking. |
| **QIB-10** | `extends:` names only the pack's own reference lists in IL-b; lists one pack ships to another arrive with IL-i. | **Yes.** |
| **QIB-11** | The biography projection's CLI half is proven structurally and unedited-unconfigured; the library selection is proven with test-tuning, since no installed pack declares a configurable biography until IL-e (IL-a D-8's rule). | **Accept.** IL-e's checkpoint proves it through the binary. |
| **QIB-12** | Perception of a narrowed fact is proven at the envelope (Visibility) in IL-b. The bystander proof through S11-C's audience function and `mineworld perceived` moves to whichever of IL-e and S11-C lands second. | **Accept.** |
| **QIB-13** | Names and units: `gap` and `invitation_lifetime`, whole seconds, bounds 1 … 86 400. | **Yes.** |
| **QIB-14** | Section facts are `<pack>-interactions-configured`, one per configured pack, `SystemInternal`; the component is `<pack>-interactions` on every Place. | **Yes** (§4.5, ARC-61 item 8). |

## 12.12 Proposed execution contract for PR IL-b

```text
PROJECT / PR        MVP-0 · S17 / PR IL-b — the Interaction List schema and its first sections
PRIMARY DESIGN DOC  .structured-coding/plans/mvp0/step-18-interaction-list.md §12; evidence §12.13
                    (E-IB<n>); deviations §12.14
RELATED / BINDING   this file §§4, 5, 6, 11 (IL-a, as merged); overall.md "The World Interaction List";
                    step-19-time-weather.md §4.5, §6.5, QTW-7, QTW-15; step-11-bodies.md §19 (12d's change
                    set); DECISIONS ARC-5, ARC-23, ARC-25, ARC-27, ARC-29, ARC-31, ARC-33, ARC-34, ARC-55,
                    ARC-61, ARC-62, DEP-10; MODULE_SPEC §§3.1, 4, 4.1; CLAUDE.md §§2–4
IMPLEMENTATION BASE main after IL-a (#80) merged, the commit named at freeze (re-audit §12.2); branch
                    mvp0/pr-il-b-sections; one worktree, one session
APPROVED SCOPE      §12.1's change set; IB-C1 … IB-C11; SD-IB-1 … SD-IB-17 as answered by QIB-1 … QIB-14
FROZEN INVARIANTS   No diff under kernel/, contracts/, persistence/, server/, clients/, worlds/; no
                    System Pack diff beyond presence's offer refusal, conversation and group-activity; root
                    Cargo.toml and Cargo.lock unchanged. IB-1 equal to E-IB-0 (re-captured from main if
                    12d merges). ac1_composability, precursor_vocabulary, seam_vocabulary unedited and
                    passing. An unconfigured world seeds exactly what it seeded. A list cannot grant.
SEQUENCE            IB-C1 → IB-C11, each committed and pushed when coherent; E-IB-0 before IB-C2
VALIDATION BUDGET   unit/integration/static unrestricted; 300-day town runs at most 14 in all (E-IB-0:
                    social-cafe ×3, market-town ×1; M-IB1a, M-IB1b; IB-1: social-cafe ×3 (also IB-14's
                    head walls), market-town ×1; IB-14 configured ×3), plus 2 more on a 12d re-capture;
                    bodies-yard 30-day, long_run and long_run_objects twice each; 30-day scratch runs
                    unrestricted; one full workspace gate on the final head (background); real-model
                    NOT REQUIRED
LIVE DOCUMENTATION  §12 checkboxes; the E-IB ledger; deviations
HANDOFF             .structured-coding/plans/mvp0/handoff-il-b.md, created at IB-C1
ENDPOINT AUTHORITY
  implementation + local validation   at the primary session's freeze message
  semantic commits, branch push       recommended authorized
  PR creation / update                recommended authorized, marked READY FOR OPERATOR REVIEW
  merge                               operator only, with a merge commit
COORDINATION        12d: §12.6; whichever merges second performs the merge and re-runs IB-1 and IB-13.
                    S19 TW-a/TW-b: whichever merges second updates the other's seed signature
NORMAL STOP         PR IL-b READY FOR OPERATOR REVIEW — DO NOT MERGE
MATERIAL STOP       an edit outside §12.1's change set, above all in kernel/contracts/persistence/server/
                    clients/worlds or a bodies file; any IB-1 difference not explained by main moving; an
                    unedited guard failing; an answer to QIB-1 … QIB-14 other than the design's
```

## 12.13 Evidence ledger (E-IB)

```text
E-IB-d  2026-10-08, design commit on plan/s17-il-b (from main @ 9cf8f8e): check_doc_headings → 191
        numbered sections across 26 documents, none duplicated; check_decision_ids → 67 ids, all
        distinct. PASS (docs only; no code exists to test).
```

## 12.14 Deviations

None yet.

## 12.15 After IL-b: IL-c … IL-i, outlined with their dependencies

Medium scope only. Each PR is detailed and frozen in its turn, after IL-b merges and the primary
session has reviewed the schema (§6.2's checkpoint, now IL-b's review).

```text
                       ┌─ 12d merged ─┐
IL-a ─► IL-b ─┬────────┴─► IL-c bodies refactor ─► IL-d bodies authored ─► IL-i ice/fragile, worlds/rink
              ├─► IL-e social ───────────┐
              ├─► IL-f ownership ────────┼─► IL-h worlds/manor (needs IL-e and IL-f; IL-g optional)
              └─► IL-g movement, employment, schedule (after 12d: movement's action.rs) ┘
S19 TW-d ◄── IL-b (data:)
S11-C ──────► IL-e's perception proof (QIB-12)
```

| PR | Scope after IL-b | Depends on | Notes from IL-b's audit |
| --- | --- | --- | --- |
| **IL-c** | Bodies reads its section; `default` reproduces 12d's bodies; the 26 P constants of `step-18-physics-list.md` §4.4 through `parameters::<Bodies>`; E constants (`rapier.rs` sizes, `SOLIDS_MAX`, `COORDINATE_BOUND` …) stay code | IL-b; **12d merged** (bodies VERSION 4, towns re-baselined) | IL-I1 against 12d's digests; the float pins of 12c/12d-0 unchanged |
| **IL-d** | Bodies authored: classes in the pair table, regions, materials, `extends` | IL-c | As §6.2; uses IL-b's classes and regions unchanged |
| **IL-e** | conversation: `talk` rule, `INTERACTION_RANGE`, `REMEMBERED_AT_MOST` and `remember`, `spoke`/`conversation-started` consequences; relationships: `acquaint` rule, the five regard parameters; group-activity: `invite`/`join` rules, `INVITE_RANGE`, `ACTIVITY_LENGTH`, its facts' consequences | IL-b (sections exist for conversation and group-activity; `Offer::refused`); S11-C for the perception proof | The first real `permits` in `validate` and offers, the first real configurable biography; proves QIB-11/12 through the binary |
| **IL-f** | inventory `PERSON_CAPACITY` and `hold`; item-transfer `give` and `GIVE_RANGE`; economy `buy`; consumption categories (QIL-20); inventory's facts' consequences | IL-b; market-town's digest after 12d | `item` gets no section (QIL-9), so 12d's `systems/item` edits do not collide |
| **IL-g** | movement: `move` into a place class at the crossing, `MAX_STRIDE`; employment `hired` and schedule `agenda-changed` biography | IL-b; 12d merged (movement's `action.rs`) | As §6.2 |
| **IL-h** | `worlds/manor`: nobles, servants, villagers, a shop, heirlooms; `configure/` as §4.8 (with `configure:` per QIL-2's ruling); the twin comparison; README; the operator plays it | IL-e and IL-f (talk, `spoke` biography, give/buy of a class); IL-g only if the manor uses a staff-only room | Its list is checked against IL-b's ambiguity rule before freeze (R-IB-5); diff is `worlds/manor/**`, its test and Markdown only |
| **IL-i** | bodies' `InteractionKind` catalog (ARC-62's second user), `object-removed`, packs `ice` and `fragile`, reference lists one pack ships to another (QIB-10), `worlds/rink`; the operator plays it | IL-d | The commit installing `ice` and `fragile` leaves `systems/bodies` unedited |
