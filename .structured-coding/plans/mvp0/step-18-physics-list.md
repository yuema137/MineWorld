# Step 18 — The physics list: interaction rules between kinds of body as content and pluggable code

**Role:** step document for a new requirement of the operator (2026-10-08). It records the
requirement, an audit of the real source, a reuse comparison, the design of MineWorld's physics list,
a framework pluggability audit with its gaps, a PR split with checkpoints and adversarial criteria,
risks, questions (QPL-1 …) and draft decision records. It holds no frozen PR design.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S15, S16), "Parallel build-out,
2026-10-08"; sibling: [`step-11-bodies.md`](step-11-bodies.md) (S15), [`step-16-packages.md`](step-16-packages.md)
(S16).
**Lifecycle:** `STEP DESIGN FROZEN (2026-10-08)`, frozen at step level by the primary session under the operator decisions recorded in `overall.md` "Physics list and configurable rules (operator, 2026-10-08)", which bind and override this document where they differ. Superseded: `DRAFT — awaiting the primary session's review`. Nothing here authorizes
implementation. Proposed edits to `overall.md`, `docs/DECISIONS.md` and the specifications are stated
inside this document (§11, §12) and are not applied.
**Branch:** `plan/physics-list` from `main @ e1ec5ff`, worktree
`/Users/yuema137/mineworld-worktrees/plan-physics-list`, one writer.
**Source audited:** `main @ e1ec5ff` (12a and 12b merged; `systems/bodies` VERSION 1) and, read-only,
PR 12c's branch `mvp0/pr-12c-objects @ 5ef5bff` (READY FOR OPERATOR REVIEW; `systems/bodies` VERSION
2), through `git -C /Users/yuema137/mineworld-worktrees/s15-12c show`.
**Decision ids:** placeholders `ARC-PL-a`, `ARC-PL-b`, `ARC-PL-c`, `DEP-PL-a`; the primary session
assigns numbers (`overall.md`, coordination ruling 6).

## Why the file is `step-18`, and the step has no S-number yet

Step files are numbered in the order they are written; `step-17-cognition.md` is the last. This is
the eighteenth. Its S-number is the primary session's to assign (QPL-14); this document calls the step
**S-PL** until then.

---

# 1. The requirement

## 1.1 Verbatim (operator, 2026-10-08)

> 我们需要确保模块化可插拔，我们现在呈现的只是default demo，但是我们ship的需要是一个框架，大家可以按自己喜欢的画风，设定，enable不同的功能。我们做的infra是high level抽象层的人物 环境 物品等等，但是大家可以加入自己的"physics list"，决定不同object之间的相互作用

English rendering, which the requirement ids below cite:

```text
R-PL-1  Everything stays modular and pluggable.
R-PL-2  What exists today is the default demo. What ships is a framework: a user chooses an art
        style and a setting, and enables the features they want.
R-PL-3  MineWorld's infrastructure is the high-level abstraction layer: people, environment, items.
R-PL-4  A user adds their own "physics list", which decides how different kinds of object interact.
```

## 1.2 The answer in brief

```text
Physics list   A typed, integer-only, versioned data document that bodies interprets. It declares
               body classes (person, object, and whatever a list adds: light, heavy, fixed, fragile)
               and a pairwise interaction table: on contact (nudge, push, block, pass), kick and throw
               eligibility and their parameters, shove, materials, how a launched body comes to rest.
               It selects and parameterizes behaviour that code provides; it never contains code.
               The term is Geant4's, and so is the shape: a list chooses, per kind of particle, which
               process constructors apply, and an application picks a list by name.

Default        bodies ships one reference list, `default`, whose values are today's constants. A world
               that authors nothing gets it, and its facts are byte-identical to 12c's.

Where          Three sources, one format: reference lists compiled into bodies; lists shipped by an
               installed System Pack that provides new behaviour; lists authored in the World Pack.
               A world selects one with `configure: bodies:` (a new, generic, owner-typed world
               configuration seam, ARC-PL-a — the per-pack configuration MODULE_SPEC §9 already
               specifies and nothing implements); a place may select another (a region).

New behaviour  A pack adds an interaction kind ("ice: slides") as code through a bodies-owned trait,
               registered from the installed set by a generalized extension catalog (ARC-PL-b, the
               pattern of ARC-39's resolver catalog). It is pure, integer in and out, and bodies
               verifies whatever it returns. A consequence that changes another pack's state
               ("fragile: breaks into items") is not a provider at all: it is a System Pack reacting
               to bodies' public facts and stating facts through each owner's constructors (ARC-26).

Ownership      bodies still alone writes where a loose object lies; presence alone where a person is.
Persistence    an authored list is a genesis fact bodies reduces into state, so a save carries its own
               list; a resume against a world whose list has changed is refused by name.
Kernel         unchanged. Contracts unchanged.
Timing         nothing enters 12c. The framework seam (PL-a) can run beside 12d; the bodies refactor
               (PL-b) lands after 12d, byte-identical to the towns' re-baselined digests.
```

---

# 2. Audit — what exists, from source

Each finding was read in this session from the file named. `main` is `e1ec5ff`; "12c" is
`mvp0/pr-12c-objects @ 5ef5bff`.

## 2.1 Every physics behaviour of bodies is compile-time today

**A-1 — the constants.** `systems/bodies/src/geometry.rs` (12c) publishes 39 integer constants (and one private, `BIAS_NORM_MILLI`); three
float material values and the scene geometry live in `rapier.rs` because no float may appear outside
it (12b PB-15, DB-3). Classified by what each one is:

| Group | Constants (12c values) | What it is |
| --- | --- | --- |
| Person's body | `PERSON_RADIUS` 300 mm, `PERSON_HEIGHT` 1 720 mm | the one person shape; also the 3D client's capsule (`player.gd`) |
| Numerical method | `GAP` 10, `TOLERANCE` 5, `SNAP` 1, `HALVINGS` 8, `LATTICE` 50, `CAPACITY_GRID` 650, `COORDINATE_BOUND` 100 000, `PUSH_SEARCH` 1 024, `STEPS_PER_SECOND` 60, `PATH_EVERY` 6, `PATH_MAX` 40, `PULL_BACK_STEP` 10; `rapier.rs`: `DT`, `WALL_THICKNESS`, `WALL_HEIGHT`, `SLAB_THICKNESS`, `SLAB_OVERHANG`, `CENTRE_Z` | how the engine computes, and what makes its integer checks exact |
| Derived invariant | `CLEARANCE` 595 (= 2R − 5) | the non-interpenetration bound, derived from R and `TOLERANCE` |
| Nudging people | `NUDGE_MAX` 300, `CHAIN_MAX` 2, `NUDGED_MAX` 4, `BIAS_BAND` 200, `BIAS_TURN` (4, 1) | QB-10's bounds, QB-16's head-on bias |
| Object limits | `OBJECT_HALF_MIN` 50, `OBJECT_HALF_MAX` 400, `OBJECT_HALF_HEIGHT_MAX` 500, `OBJECTS_MAX` 32 | what a loose object may be |
| Actions | `KICK_REACH` 800, `THROW_REACH` 800, `SHOVE_REACH` 1 000, `SHOVE_OFFER_REACH` 800, `SHOVE_DISTANCE` 500, `KICK_SPEED` 5 000 mm/s, `KICK_STEPS` 180, `THROW_STEPS` 240, `THROW_FLIGHT` 48, `THROW_DEFAULT` 3 000, `THROW_RANGE_MAX` 6 000 | §8.1's actions, SD-O11 … SD-O15, the p1 offer rung |
| Coming to rest | `REST_SPEED` 50 mm/s, `REST_STEPS` 10, `REST_CLEARANCE` 300, `GRAVITY` 9 810 mm/s²; `rapier.rs`: `FRICTION` 0.5, `RESTITUTION` 0.1, `GRAVITY_Z` −9.81 | DC-4's rest test, rule p4 (DO-17), the material |

**A-2 — the rules, in code rather than in a number.** Each is a branch in a named file, and each is a
decision a different game could reasonably make differently:

| Rule | Where (12c) |
| --- | --- |
| A person walking into a person nudges them (bounded), never blocks unless the bounds fail | `stride.rs`, `resolve.rs` (SD-B6, SD-B7) |
| A person walking into a loose object on the floor pushes it; an object on a solid is never pushed | `push.rs` (SD-O8; "inert: an object on a solid") |
| A push that jams makes every object solid for that stride | `resolve.rs`, `stride.rs` (SD-O9 step 6) |
| Objects never push people; people are kinematic during a flight; other objects are fixed during a flight (no chain reactions) | `flight.rs`, `rapier.rs` (SD-O14, QO-10) |
| Only an object on the floor can be kicked; any object within reach can be thrown | `offer.rs`, `launch.rs` (SD-O11, SD-O13) |
| An unaimed kick or throw heads for the room's free centre, never into the kicker | `launch.rs` (rung p3, DO-16; DO-18) |
| A launched object never rests within 300 mm of a solid unless its flight was blocked | `flight.rs` `rests_clear`, `pulled_back` (rule p4, DO-17) |
| Shove is offered complete only within 800 mm; the request may come from 1 000 mm | `offer.rs` (rung p1, DO-13) |
| Every object is the same: no mass, no material, no class | `component.rs` (`BodyShape` is `Box` or `Ball`, nothing else) |

**A-3 — 12c's own history is the argument for this step.** DO-13, DO-16, DO-17 and DO-18 and the two AO-2
rulings changed bodies' behaviour after measurements in one sandbox world (`worlds/bodies-yard`): offer
policy (p1), unaimed direction (p3), rest clearance (p4), kicker avoidance. Each was a code change to the
pack, and each would have been a content change if the rule had been a parameter. A different world —
denser, emptier, an ice rink — would have wanted different values, and today it cannot have them
without forking `bodies`.

**A-4 — a switch already exists, for tests only.** `resolve.rs` (12c) has `pub(crate) struct Policy {
bias, verify }` and `const PRODUCTION`. It is the place where the head-on bias and verify-then-degrade
are turned off in unit tests (12b PB-8); it is not reachable from a world. It shows that the pipeline
already reads its behaviour from one value; this step makes that value come from the world.

## 2.2 How a body is authored

**A-5 — one section, two forms** (`systems/bodies/src/section.rs`, 12c; ARC-31, SD-O4). `body:` is
carried by place files (`floor`, `solids`) and item files (`shape: { box | ball }`, `at:
{ place, x, y }`). `CARRIED_BY = [Place, Item]`. Person files carry no `body:` (every person is the
default capsule, QP-6). `deny_unknown_fields` holds: a field this step adds must be added to the type,
or it is refused at its line.

**A-6 — what bodies stores** (`component.rs`, 12c): `PlaceShape` on a Place; `BodyShape` (`Box(HalfExtents)
| Ball(Millimetres)`) on an Item; `LooseObjects` (`Lying { object, at }`, sorted by `ItemId`) on a Place.
No component carries a class, a mass or a material.

## 2.3 How code reaches another pack's pipeline today

**A-7 — the resolver catalog** (`sdk/rust/src/installed.rs`, `worldpack/src/load.rs:193`,
`systems/installed/src/lib.rs`). `installed!` has a dedicated `resolution: <trait> => [<types>]` line;
it expands `Capability::resolvers()`, and `worldpack::compose` calls
`mineworld_presence::register_resolvers(...)` — so the catalog is presence-specific in two framework
crates. A second catalog (for bodies) would, today, mean a second dedicated line in the macro and a
second named call in `worldpack`: the change amplification `CLAUDE.md` §4 rule 5 forbids, the second
time it is observed (rule 11: now the abstraction is earned).

**A-8 — the providers are pure and process-wide** (ARC-39, QB-15 F1). A resolver keeps no state, reads
only `WorldRead`, is inert where its own state is absent, and is asked in `SystemId` order. Per-world
applicability comes from state, not from registration.

## 2.4 How a world configures a pack today

**A-9 — there is no per-world configuration of a System Pack.** `MODULE_SPEC.md` §9 specifies
"configuration schema" per pack; `step-16-packages.md` G-8 records that `SystemDeclaration` omits it
deliberately (`kernel/src/system.rs:99–101`) and that "configure" is done by sections (ARC-31).
Sections are carried by person, place, item and organization files only (`authoring/src/section.rs`
`ContentKind`, four variants); nothing is carried by the world. `world.yaml` refuses unknown keys by
name (`MODULE_SPEC.md` §4.1).

**A-10 — rule numbers are constants in every pack, not only bodies.** Examples on `main`:
`movement::MAX_STRIDE` 2 000 mm; `conversation::INTERACTION_RANGE` 3 000 mm, `CONVERSATION_GAP` 300 s;
`group_activity::INVITE_RANGE` 3 000 mm, `ACTIVITY_LENGTH` 3 600 s, `INVITATION_LIFETIME` 1 800 s;
`relationships::SPOKE_FAMILIARITY` 10, `ACCEPTED_REGARD` 50, `DECLINED_REGARD` −30;
`inventory::PERSON_CAPACITY` 6; `consumption::EATEN` "food", `DRUNK` "drink". Prices and wallets are
content (`economy:` section); wages and jobs are content (`job:` section); routines are content
(`routine:` section).

**A-11 — `MODULE_SPEC.md` §4 constraint 3: "A World Pack never redefines simulation rules. If a world
needs a new rule, that is a System Pack."** This binds the design: a physics list authored in a World
Pack may select and parameterize behaviour a System Pack defines, under bounds that pack declares; it
may not introduce behaviour. New behaviour is code (§4.7).

## 2.5 Persistence, composition and resume

**A-12 — the composition record** (`kernel/src/world.rs:415`, `kernel/src/snapshot.rs:45`,
`persistence/src/replay.rs:130`): each installed system's `SystemDeclaration` (which carries its
`SystemVersion`) and whether it is enabled, in registration order. A mismatch refuses a resume by name
(ARC-25). Nothing else about a world's configuration is in it, and adding anything would change a
kernel type.

**A-13 — resume does not re-read the World Pack's content** (`persistence/src/world.rs:116–121`,
`tools/cli/src/run.rs:241`, `tools/cli/src/main.rs:389`). The host composes a world (systems only),
checks the composition, restores the newest snapshot and re-executes the journal, genesis included,
from the save. A save is self-contained: editing a person's file after a save changes nothing for that
save, and nothing reports the difference.

## 2.6 Presentation, enablement and controllers (for Part 2, §8)

**A-14 — presentation.** `presentation/mineworld-default/{2D,3D}/` hold style manifests, art direction
and references; **no program reads them** (S16 F-E6). The 3D client transcribes the palette by hand
(`clients/3d-spike/scripts/slice/palette.gd`: "Read from presentation/mineworld-default/3D/references/")
and builds the café, the street and the interiors in GDScript bound to `worlds/social-cafe`
(`slice_world.gd:17`, `cafe_interior.gd`, `street.gd`). The 2D client is not on `main`; S12's design
makes it read a Presentation Pack through `--presentation <dir>` and `asset_bindings.yaml` roles, with
`ARC-14`'s four style variants (`town`, `full`, `people`, `procedural`) as binding sets
(`step-13-client-2d.md` A-5, QS12-4). `presentation_profile` is refused by `world.yaml` (S16 QSE-9:
declared and validated in S16, applied by S12/S14).

**A-15 — enablement.** A world enables a System Pack by naming it in `systems:` (ARC-33); a pack's
actions become `Unavailable` when it is absent (`INV-10`, `AC-2`). Installing a pack into the build is
two lines and a rebuild (ARC-33); from outside the repository, S16 E-c.

**A-16 — controllers.** `mineworld run` always drives every seat with `PacedRuleController::new(seed,
PACE)` (`tools/cli/src/run.rs:107`); `--agent` uses `RuleController` (`tools/cli/src/agent.rs:56`).
The rule controller is compiled into `tools/cli`. `cognition_profile` is refused; selection per world
or seat is S10/MVP-1 (S16 QSE-10; `step-17-cognition.md`).

---

# 3. Reuse comparison — both directions (`REUSE_POLICY.md` §§2, 4, 11–12, 17)

## 3.1 The question, split

The physics list is three separable things, and each is asked separately:

```text
(a) the concept and vocabulary   how a world names kinds of body and says which interact, and how
(b) the evaluation               how a pair's rule is applied to motion, contact and rest
(c) the format and its loading   where the document lives, how lists compose, how a mod adds one
```

Commodity infrastructure (`REUSE_POLICY.md` §4: "physics engines") is (b)'s motion, and it is already
reused: Rapier (`DEP-13`). (a) and (c) are "modular world semantics" and "world-pack composition",
which §4 lists as MineWorld's own differentiated work. The comparison below therefore asks, for each
candidate, what it can give to (a), (b) and (c), and does not stop at the first answer.

## 3.2 Sources, read on 2026-10-08

Facts below were fetched from the primary pages by two web agents in this session, or read from the
GitHub API. Where a fact is an inference or was not stated word for word, it is marked.

| # | Candidate | Primary sources |
| --- | --- | --- |
| 1 | Geant4 physics lists | geant4-userdoc.web.cern.ch `UsersGuides/ForApplicationDeveloper/html/UserActions/mandatoryActions.html` (Book For Application Developers 11.4); `UsersGuides/PhysicsListGuide/html/index.html`; geant4.web.cern.ch `download/license` |
| 2 | Unity layer collision matrix, Physics Material | docs.unity3d.com `Manual/LayerBasedCollision.html`, `Manual/create-layers.html`, `Manual/class-PhysicsMaterial.html`, `Manual/collider-surfaces-combine.html` (Unity 6.6) |
| 3 | Godot collision layers and masks, `PhysicsMaterial` | docs.godotengine.org `en/stable/tutorials/physics/physics_introduction.html`, `en/stable/classes/class_physicsmaterial.html` (4.7); licence from `gh api repos/godotengine/godot/license` |
| 4 | Rapier collision groups, solver groups, `PhysicsHooks` | rapier.rs `docs/user_guides/rust/colliders`, `…/advanced_collision_detection`, `…/determinism` (0.36); github.com/dimforge/rapier; `gh api repos/dimforge/rapier/license` |
| 5 | Box2D v3 filtering and pre-solve | box2d.org `documentation/md_simulation.html` (3.1.0); github.com/erincatto/box2d |
| 6 | Factorio prototypes | lua-api.factorio.com `latest/prototypes/CollisionLayerPrototype.html`, `latest/types/CollisionMaskConnector.html`, `latest/auxiliary/mod-structure.html`, `latest/auxiliary/data-lifecycle.html` (2.1.21) |
| 7 | RimWorld defs, comps and patches | rimworldwiki.com `Modding_Tutorials/ThingComp`, `…/PatchOperations`, `…/Mod_Folder_Structure` |

## 3.3 Each candidate

**1. Geant4 physics lists** — where the term comes from.
- *What it is.* "Physics Model = final state generator", "Physics Process = cross section + model",
  "Physics List = list of processes for each particle". `G4VUserPhysicsList` constructs particles and
  processes; `G4VModularPhysicsList` organizes them into modules registered with
  `RegisterPhysics(G4VPhysicsConstructor*)`, which can be removed or replaced. Reference lists
  (`FTFP_BERT`, `QBBC`, `QGSP_BERT`, `Shielding`, …) ship with the toolkit and are supported by its team;
  `G4PhysListFactory::GetReferencePhysList("FTFP_BERT_EMV")` selects one by name, a suffix swapping the
  electromagnetic constructor; `ReferencePhysList()` reads the name from the environment variable
  `PHYSLIST`.
- *Fit.* (a): exact in shape. Kinds of particle ↔ body classes; processes ↔ interaction behaviours
  implemented in code; modular constructors ↔ packs that provide behaviour; reference lists chosen by
  name ↔ `default` and lists shipped by packs; a suffix swapping one module ↔ a list that `extends`
  another and replaces one entry. (b), (c): none — Geant4 is particle transport in C++, the language
  of its listings (the page does not say "C++"; the code is), not a library MineWorld could call.
- *Licence.* The Geant4 Software License 1.0 (2006): permissive, with an attribution clause, a
  licence-back of published modifications "including any patents you own", a no-patent-filing clause,
  and termination on suit. Irrelevant to a pattern; it would matter only if code were taken, and none is.
- *Maturity.* Decades, the reference toolkit of high-energy physics.
- *Cost.* Zero: a pattern and a vocabulary.
- **Verdict: ADOPT THE PATTERN AND THE NAME; reuse no code.** It is the model the operator named, and it
  separates the three things this design needs separated: what kinds exist (data), what behaviour exists
  (code, modular), and which behaviour applies to which kind (a named, swappable list).

**2. Unity's layer collision matrix and Physics Material.**
- *What it is.* Each GameObject is on one layer; the matrix (Edit › Project Settings › Physics) is a
  checkbox per pair of layers saying whether they collide. A layer mask is an integer bitmask. Layer 31
  is reserved by the editor and "You can't add more Layers" — consistent with 32 layers, though no page
  read states the number (inference). A Physics Material has dynamic friction (0–1, default 0.6), static
  friction (0.6), bounciness (0), and a friction combine and a bounce combine; when two materials'
  combine modes differ, the higher-priority one wins: Maximum > Multiply > Minimum > Average.
- *Fit.* (a): the matrix is the simplest pairwise table — symmetric, boolean, one class per object — and
  shows that a project-wide table of pairs is something game authors already understand. The combine
  rule answers a real question the design must answer: two bodies with two materials meet; which
  friction applies? (b), (c): engine-bound; nothing callable from a Rust server.
- *Licence.* Proprietary (the pages are "Copyright ©2005-2026 Unity Technologies. All rights reserved";
  no licence page was fetched).
- **Verdict: REFERENCE.** Adapt the combine rule for materials (§4.3) and the idea of a project-wide
  table of class pairs. Its symmetry is too weak for MineWorld: "a person pushes a box" is not "a box
  pushes a person".

**3. Godot's collision layers and masks and `PhysicsMaterial`.**
- *What it is.* 32 layers. `collision_layer` is "the layers that the object appears in";
  `collision_mask` is "what layers the body will scan for collisions". One object's mask is tested
  against the other's layer, so a relation can be one-directional (an inference from the definitions,
  not the page's wording). `PhysicsMaterial` has `friction` (0–1), `rough`, `bounce` (0–1) and
  `absorbent`: if one body is rough its friction is used, otherwise the lower; absorbent subtracts
  bounce instead of adding it.
- *Fit.* (a): the one-directional mask is the right shape for asymmetric relations ("person nudges
  person" is symmetric; "person pushes box" is not). (b): it is the 3D client's engine (`DEP-4`, Jolt
  in Godot, `DEP-14`); the client may map body classes to layers for its local, non-authoritative
  prediction (§4.11). Never authoritative, so never the evaluator.
- *Licence.* MIT (GitHub API). Maturity: mature, already a dependency of the clients.
- **Verdict: REFERENCE for the table's asymmetry; ADOPT on the client only**, as presentation-side
  prediction derived from disclosed classes, never as a rule (`ENGINEERING_RULES.md` §§7–9, I-8).

**4. Rapier's collision groups, solver groups and `PhysicsHooks`.**
- *What it is.* An `InteractionGroups` value is a membership mask and a filter mask (`u32`: 32 groups);
  two colliders interact iff `(A.memberships & B.filter) != 0 && (B.memberships & A.filter) != 0`.
  Failing collision groups computes no contact; failing solver groups computes contacts but applies no
  force. `PhysicsHooks` (passed to `step`): `filter_contact_pair` (no contact, contact without forces, or
  with), `filter_intersection_pair` for sensors, and `modify_solver_contacts`, which may change normals,
  points, depth, tangent velocity, and friction and restitution for a manifold, may remove contacts, and
  cannot add them. Queries take a `QueryFilter` with a predicate (12c uses `only_fixed().predicate(..)`).
  Cross-platform determinism needs `enhanced-determinism`, IEEE 754-2008 targets, canonical insertion
  order, and nalgebra's `ComplexField`/`RealField` for transcendental inputs (verified for 12b, §17.3.3
  of step-11). The colliders page's groups section was read through the fetching agent's summary; its
  quoted formula matches.
- *Fit.* (b): exact, inside `systems/bodies/src/rapier.rs`. The table's contact column compiles into
  query-filter predicates per class for sweeps, and into collision or solver groups (or a
  `filter_contact_pair` hook) for flights; per-pair materials compile into each collider's friction and
  restitution or `modify_solver_contacts`. (a), (c): wrong level. A bitmask is an engine concept; if a
  World Pack named Rapier groups, a physics-engine concept would enter authored content, against
  `ENGINEERING_RULES.md` §12 and `DEP-13`'s isolating interface ("No Rapier type appears in a component,
  a fact, a contract or another crate"). And groups say only whether two shapes touch; nudge-or-push,
  kick eligibility and rest rules are above collision.
- *Licence.* Apache-2.0. Maturity: as `DEP-13`. Cost: low; already pinned (`=0.36.0`).
- **Verdict: ADAPT, behind bodies' adapter only.** The list is evaluated in integers by bodies; where it
  changes motion, `rapier.rs` expresses it with Rapier's own groups, filters, hooks and materials, so no
  contact filtering is re-implemented (`REUSE_POLICY.md` §5, thin adapters). The 32-group limit becomes
  a stated bound on classes per list (§4.5).

**5. Box2D v3's contact filtering and pre-solve callback.**
- *What it is.* `categoryBits` and `maskBits` with the same two-sided test as Rapier's; a `groupIndex`
  whose shared non-zero value overrides category and mask (positive: always collide; negative: never);
  `b2World_SetCustomFilterCallback` and `b2World_SetPreSolveCallback` (e.g. one-sided platforms), which
  "must be thread-safe and must not read from or write to the Box2D world". The page claims determinism
  across thread counts and platforms; that sentence came through the fetching agent's summary and was
  not confirmed word for word (marked unverified).
- *Fit.* (a): the group-index override is a precedent for this design's precedence rule — an explicit
  pair entry overrides what the two classes' defaults would give (§4.5). Its callbacks' contract — pure,
  no world access — is the contract this design gives providers (§4.7). (b): 2D, C17, `float`; MineWorld
  already has a 3D engine.
- *Licence.* MIT. Maturity: mature.
- **Verdict: REFERENCE** (precedence rule; callback purity). Nothing to adopt.

**6. Factorio's prototypes** — a data-driven interaction table in a moddable game.
- *What it is.* A `collision-layer` is a data prototype ("Prototype limited to 256 total instances"),
  so a mod adds layers in its data stage (an inference from the prototype mechanism; the page does not
  say it). Each entity's `collision_mask` is a dictionary of the layers it collides with, plus flags
  (`not_colliding_with_itself`, …). Data loads in three rounds — every mod's `data.lua`, then every
  `data-updates.lua`, then every `data-final-fixes.lua` — ordered by dependency depth and then by name,
  so a mod may change another mod's prototypes; "the game records which mod changed which prototype".
  `info.json` carries dependencies with version operators and prefixes (`!` incompatible, `?` optional).
- *Fit.* (a), (c): classes as data that mods add; a deterministic, recorded order of modification;
  versioned dependencies (S16 already adapts that vocabulary, its §5 row 9). The Lua stages are
  unconstrained code over a global table — exactly the untyped mutable blob `CLAUDE.md` §4 rule 7
  forbids at a boundary.
- *Licence.* Proprietary game; the API is documentation.
- **Verdict: REFERENCE; ADAPT two ideas** — classes are content that a list may add, and composition of
  lists is a fixed, recorded order (`extends`, one level of override, refused on conflict). Reject the
  free-form stages.

**7. RimWorld's defs, comps and patch operations.**
- *What it is.* XML Defs are content; a Def's `comps` list names C# classes (`compClass`, or
  `CompProperties` with a `Class` attribute), instantiated per thing at creation with the Def's
  properties. `Patches/` holds XPath `PatchOperation`s (`Add`, `Replace`, `Remove`, …, custom subclasses),
  applied after all Defs load, in mod-list order, before inheritance.
- *Fit.* (a), (c): **data names behaviour that code implements, by name** — the exact split between a
  list (data) and an interaction kind (code) this design needs (§4.7). XPath patching of another mod's
  data is the opposite of strong typing: a patch can change any node of any def, unchecked until it is
  read.
- *Licence.* Proprietary game; the wiki is documentation.
- **Verdict: REFERENCE; ADAPT "data names code by id".** Reject XPath patching; a list `extends` one
  other list and overrides typed entries, and every override is validated by the owner.

**8. Building our own** — the format and its interpreter, inside bodies.
- *What it is.* A typed YAML document decoded into bodies' own Rust types (the `AuthoredSection`
  pattern of ARC-31: deserializing is validating), evaluated by integer lookups in bodies' existing
  pipeline, with Rapier still doing every motion (row 4).
- *Fit.* The only option that gives (a) and (c) in MineWorld's types — `PersonId`, `ItemId`,
  `Millimetres`, integers only, deterministic, server-authoritative, versioned in a save. No library
  offers a semantic interaction table for a persistent world; physics engines offer collision masks,
  and games offer formats bound to their loaders.
- *Cost.* Moderate: a format, validation, a table lookup in perhaps a dozen places in bodies, and a
  refactor that must be byte-identical (§7).
- **Verdict: BUILD, narrowly** — the format and the lookup only. Motion stays Rapier's; YAML decoding
  stays `serde-saphyr` (`DEP-10`). Recorded as `DEP-PL-a` (§11), because declining every library in
  favour of our own code needs a record too (`REUSE_POLICY.md` §12).

## 3.4 Summary and recommendation

| # | Candidate | (a) concept | (b) evaluation | (c) format | Licence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Geant4 physics lists | exact shape | — | — | Geant4 SL 1.0 | **adopt the pattern and the name** |
| 2 | Unity matrix, Physics Material | pair table; combine rule | engine-bound | — | proprietary | reference; adapt the combine rule |
| 3 | Godot layers/masks, `PhysicsMaterial` | asymmetry | client prediction only | — | MIT | reference; adopt on the client only |
| 4 | Rapier groups, hooks, materials | wrong level | **exact, in `rapier.rs`** | — | Apache-2.0 | **adapt behind the adapter** |
| 5 | Box2D filtering, pre-solve | precedence; pure callbacks | — | — | MIT | reference |
| 6 | Factorio prototypes | classes as data | — | ordered, recorded composition | proprietary | reference; adapt two ideas |
| 7 | RimWorld defs, comps | data names code | — | typed, not XPath | proprietary | reference; adapt "data names code" |
| 8 | Build our own | MineWorld's types | Rapier underneath | YAML via `DEP-10` | — | **build, narrowly** (`DEP-PL-a`) |

**Recommendation.** Take Geant4's architecture and name; express the table in MineWorld's own typed,
integer format (8), with classes as content (6) and behaviour named by id and implemented in code (7);
evaluate it in bodies and let `rapier.rs` realize it with Rapier's groups, filters, hooks and materials
(4); adopt Unity's combine rule for materials (2), Box2D's precedence for overrides (5), and Godot's
asymmetric masks as the model for one-directional relations and for the 3D client's prediction (3).
Both failure modes of `REUSE_POLICY.md` §17 are checked: nothing commodity is rebuilt (no collision
detection, no contact filtering, no solver, no YAML parser); nothing is forced (no engine type in content,
no engine as the authority, no scripting runtime where a typed table suffices).

---

# 4. Design — MineWorld's physics list

## 4.1 Vocabulary

Every term below is new and is checked against `CORE_CONCEPTS.md` and `MODULE_SPEC.md` so that no
defined term is reused for another concept (`CLAUDE.md` §2.1(3)). They are bodies' vocabulary, proposed
for `MODULE_SPEC.md` §4.1's `body` row and a new bodies subsection (§12), not for the core ontology.

| Term | Meaning | Why not another word |
| --- | --- | --- |
| **body class** | A named category of body that a physics list gives rules to: `person` (every person) and the classes a list declares for loose objects (`object`, `light`, `heavy`, `fixed`, `fragile`, …). Content, not code. | Not *kind*: `ItemKind` is ARC-36's. Not *tag*: entity tags are free text with no owner. Not *layer*: an engine word (`ENGINEERING_RULES.md` §12). Not *type*: `EntityType`. |
| **physics list** | A named, versioned document of body classes, materials, a pairwise interaction table and parameters, interpreted by `bodies`. | The operator's word, and Geant4's. |
| **reference list** | A physics list compiled into a System Pack: `default` in bodies, and any list a pack providing new behaviour ships. | Geant4's term for the lists that ship with the toolkit. |
| **interaction kind** | A named behaviour a list may select — `nudge`, `push`, `block`, `launch` built into bodies; `slide` or any other provided by an installed pack as code. | Not *process*: `Process` is a defined term (`CORE_CONCEPTS.md`), and Geant4's "process" is exactly what must not be imported. Not *system*. |
| **region** | A place that selects a list other than its world's. | Geant4's regions carry per-region settings the same way; a place is already the unit of a resolution (step-11 §5). |

## 4.2 What a physics list is, and what it is not

A physics list **selects and parameterizes** behaviour that code provides. It never adds behaviour:
`MODULE_SPEC.md` §4 constraint 3 (A-11) binds, and a list is the "configuration schema" `MODULE_SPEC.md`
§9 already assigns to a pack. Five rules follow:

1. **Owned by bodies.** Bodies defines the document's type; deserializing it is validating it (the
   ARC-31 pattern). The loader never learns what a list means.
2. **Integers only.** Millimetres, millimetres per second, sub-steps, grams, per-mille. No float is
   authored, stored or disclosed (I-3 of step-11 holds). Floats exist only inside `rapier.rs`, converted
   by one function per unit.
3. **Total.** Every pair of classes the world can produce has exactly one answer, fixed at load: an
   explicit pair entry, else what the two classes' defaults give, else the engine's default (refuse the
   interaction). Lookup never fails at run time.
4. **Bounded.** Every parameter has an engine ceiling and floor published by bodies; a list outside them
   is refused at load, naming the parameter, the value and the bound. The invariants that hold for every
   list (no two people closer than `CLEARANCE`, nothing inside a wall, nothing tunnelling, verify then
   degrade) are engine properties no list can switch off (§5).
5. **Self-contained in a save.** What a world runs is the resolved list — every `extends` applied,
   every default filled in — stored as state. A save never refers to a list by name only.

## 4.3 The document

The `default` reference list, written out. It is the exact content of 12c's constants and rules (§4.4
maps each value to its source):

```yaml
# bodies' reference list `default` — compiled into the pack (include_str!), decoded by the same type
# as any authored list. A world that configures nothing runs this list and stores nothing.
id: default
version: 1                         # bumped with bodies' VERSION whenever this file changes (§4.9)

classes:
  person:                          # reserved; every person is of this class; the shape is the engine's
    material: none                 # people are kinematic: no material is applied (12c)
  object:                          # the class of every loose object whose body: names none
    material: { friction: 500, restitution: 100 }      # per mille

surfaces:                          # floor, walls and solids of a shaped place
  material: none                   # 12c sets no material on fixed geometry

combine: average                   # how two materials meet (Unity's rule; Rapier's default)

people:                            # person meets person — the only pair whose answer is built in
  contact: nudge
  nudge: { max: 300, generations: 2, people: 4 }
  head_on_bias: on                 # QB-16
  shove: { reach: 1000, offer_reach: 800, distance: 500 }

pairs:                             # actor class → body class; every pair not listed is `block`, no actions
  - actor: person
    body: object
    contact: push                  # walking into it pushes it out of the way (SD-O8)
    push: { from: floor }          # an object resting on a solid is never pushed
    kick:  { reach: 800, from: floor, speed: 5000, steps: 180 }
    throw: { reach: 800, from: any, steps: 240, flight: 48, unaimed: 3000, range: 6000 }

launch:                            # every kicked or thrown body
  unaimed: free_centre             # rung p3 (DO-16), never into the kicker (DO-18)
  others: fixed                    # no chain reactions (QO-10); engine-fixed in this step
  rest: { speed: 50, steps: 10, clearance: 300, pull_back: 10 }        # DC-4, rule p4 (DO-17)
  gravity: 9810
```

What a world might author instead (illustrative; not part of any PR's content):

```yaml
# configure/bodies.yaml of a world (§4.6)
physics: warehouse                 # the list every place runs unless it names another
lists:
  - id: warehouse
    version: 1
    extends: default               # one level; every override is validated as if written in full
    classes:
      light:   { material: { friction: 300, restitution: 400 } }
      heavy:   { mass: 80000 }     # grams; used only by kinds that read it
      fixed:   {}
      fragile: { material: { friction: 500, restitution: 0 } }
    pairs:
      - { actor: person, body: light,   contact: push, kick: { speed: 7000 }, throw: {} }
      - { actor: person, body: heavy,   contact: block, kick: none, throw: none }
      - { actor: person, body: fixed,   contact: block }
      - { actor: person, body: fragile, contact: push, kick: {}, throw: {} }
    people:
      nudge: { max: 150 }          # a calmer crowd
  - id: rink
    version: 1
    extends: warehouse
    interactions: [slide]          # a kind an enabled pack provides (§4.7); refused if none does
    surfaces: { kind: slide, slide: { extra: 1500 } }
```

and in an item file, the class beside the shape (12c's `body:` object form gains one optional key):

```yaml
body:
  shape: { box: { x: 300, y: 300, z: 300 } }
  at: { place: hall, x: 4000, y: 6000 }
  class: heavy                     # optional; `object` when absent — which is every 12c object
```

and in a place file, a region (12c's `body:` place form gains one optional key):

```yaml
body:
  floor: { min: { x: 0, y: 0 }, max: { x: 20000, y: 12000 } }
  physics: rink                    # optional; the world's list when absent
```

**Pair entries.** `actor` is the class that moves or acts (in this step always `person`: only people
walk, kick, throw and shove); `body` is the class met or acted on. `contact` is the interaction kind
that answers a stride meeting a body of that class: `push`, `block`, or a provided kind. `kick`,
`throw` are `none` (not offered, refused `NoSupportedInteraction`) or a parameter block; an empty
block takes every value from the pair's defaults, which are the `default` list's. Person meets person
is the separate `people` block because its answer is built in (`nudge` or `block`) and must keep
I-11's bound shape; a list cannot make people pass through one another (§5, PL-I6).

**Materials** combine like Unity's (§3.3 row 2): each class and the surfaces carry an optional material;
`combine` chooses how two meet. `none` means "the adapter sets nothing on that collider", which is what
12c does for people and fixed geometry and is what keeps the default byte-identical (§4.4). Rapier's own
combine rules (`CoefficientCombineRule`) are what `rapier.rs` uses to realize it; MineWorld names the
rule, Rapier applies it.

**Mass** is an optional integer per class, in grams. No built-in kind reads it in this step; a ratio
rule ("a person pushes a body only if it is at most this heavy") is expressed by choosing `block` for
that class, and a kind that derives speed from mass is a provided kind (§4.7). A built-in mass-ratio rule
is QPL-7.

## 4.4 Which constants become parameters, and which stay engine constants

**Parameters** (in the list; default = today's value, so the default list reproduces 12c):

| 12c constant or rule | List parameter | Default | Bound proposed (engine floor … ceiling) |
| --- | --- | --- | --- |
| `NUDGE_MAX` | `people.nudge.max` | 300 mm | 0 … 300 (I-11's 310 is the ceiling; 0 means `block`) |
| `CHAIN_MAX` | `people.nudge.generations` | 2 | 1 … 2 |
| `NUDGED_MAX` | `people.nudge.people` | 4 | 1 … 4 |
| QB-10's choice "nudge" | `people.contact` | `nudge` | `nudge` \| `block` |
| QB-16's bias on/off (`Policy::bias`) | `people.head_on_bias` | `on` | `on` \| `off` |
| `SHOVE_REACH` | `people.shove.reach` | 1 000 mm | 610 … 2 000 |
| `SHOVE_OFFER_REACH` (p1) | `people.shove.offer_reach` | 800 mm | 610 … `reach` |
| `SHOVE_DISTANCE` | `people.shove.distance` | 500 mm | 0 (`none`) … 1 000 |
| "walking pushes objects" (SD-O8) | `pairs[].contact` | `push` | `push` \| `block` \| provided kind |
| "an object on a solid is never pushed" | `pairs[].push.from` | `floor` | `floor` (only value in this step) |
| `KICK_REACH` | `pairs[].kick.reach` | 800 mm | ≥ R + GAP + the class's largest half-extent … 2 000 |
| "only objects on the floor are kicked" | `pairs[].kick.from` | `floor` | `floor` (only value in this step) |
| `KICK_SPEED` | `pairs[].kick.speed` | 5 000 mm/s | 500 … 15 000 |
| `KICK_STEPS` | `pairs[].kick.steps` | 180 | 1 … 600 |
| `THROW_REACH` | `pairs[].throw.reach` | 800 mm | as kick |
| "any object within reach is thrown" | `pairs[].throw.from` | `any` | `floor` \| `any` |
| `THROW_STEPS`, `THROW_FLIGHT` | `pairs[].throw.steps`, `.flight` | 240, 48 | 1 … 600; 1 … `steps` |
| `THROW_DEFAULT`, `THROW_RANGE_MAX` | `pairs[].throw.unaimed`, `.range` | 3 000, 6 000 mm | 0 … `range`; 0 … 20 000 |
| `kick: none` / `throw: none` | eligibility | allowed for `object` | allowed \| `none` |
| rung p3, DO-18 | `launch.unaimed` | `free_centre` | `free_centre` \| `away` (SD-O11's original rule) |
| `REST_SPEED`, `REST_STEPS` | `launch.rest.speed`, `.steps` | 50 mm/s, 10 | 1 … 500; 1 … 60 |
| `REST_CLEARANCE`, `PULL_BACK_STEP` (p4) | `launch.rest.clearance`, `.pull_back` | 300 mm, 10 mm | 0 … 1 000; 1 … 100 |
| `GRAVITY`, `rapier.rs GRAVITY_Z` | `launch.gravity` | 9 810 mm/s² | 0 … 30 000 |
| `rapier.rs FRICTION`, `RESTITUTION` | `classes.object.material` | 500, 100 ‰ | 0 … 2 000; 0 … 1 000 |
| — (none in 12c) | `classes.*.mass`, `surfaces.material`, `combine` | absent, `none`, `average` | — |

Every "only value in this step" is a field that exists so the type does not change when a later PR
adds the value; a list naming another value is refused by name, not ignored (`MODULE_SPEC.md` §4.1's
rule).

**Engine constants** (stay in code; changing one is a bodies `VERSION` bump, as today):

| Constant | Why it stays |
| --- | --- |
| `PERSON_RADIUS`, `PERSON_HEIGHT` | Every derived bound — `CLEARANCE`, `CAPACITY_GRID` (chosen so that 13 × `LATTICE` ≥ 2R + GAP), SD-B4's capacity proof, the push search's upper bound, the 3D client's capsule (`player.gd`) — is derived from R. A per-list person shape is a separate design (QPL-6). |
| `GAP`, `TOLERANCE`, `SNAP`, `HALVINGS`, `LATTICE`, `CAPACITY_GRID`, `PUSH_SEARCH`, `COORDINATE_BOUND`, `STEPS_PER_SECOND`, `PATH_EVERY`, `PATH_MAX`, `DT`, wall and slab dimensions | Numerical method. They make the integer checks exact and the floats bounded (DC-4, DC-6, SD-B5); a list that changed them could break the proofs I-11 and I-12 rest on. |
| `CLEARANCE` | The non-interpenetration invariant; derived. |
| `BIAS_BAND`, `BIAS_TURN` | QB-16's geometry; a list may turn the bias off, not reshape it (its 310 mm proof depends on it, SD-B10). |
| `OBJECT_HALF_MIN`, `OBJECT_HALF_MAX`, `OBJECT_HALF_HEIGHT_MAX`, `OBJECTS_MAX` | Validity bounds of a loose object; they keep `PUSH_SEARCH` and capacity sound. A list may narrow them per class (QPL-8), never widen. |
| "objects are fixed during a flight", "people are kinematic during a flight", "people never interpenetrate", verify-then-degrade | Engine properties, PL-I6 (§5). `launch.others: fixed` is written in the list so that a later PR can add chain reactions as a value, but `fixed` is its only value now. |

**Byte-identity of the default, argued value by value.** The integer parameters replace constants of the
same value, read in the same places. The two float materials and gravity are converted from integers by
one function each: `500 as f32 / 1000.0` is exactly 0.5; `100 as f32 / 1000.0` and `9810 as f32 / 1000.0`
are IEEE-correctly-rounded divisions, which yield the same nearest `f32` as the literals `0.1` and
`9.81` — a unit test pins the three bit patterns, so the argument is checked rather than trusted.
`material: none` sets nothing, as today. The canonical insertion order does not change. PL-b's acceptance
measures it (§7).

## 4.5 Resolution, lookup and validation

**Resolving a list** (at load, in bodies' configuration type):

```text
1  decode every authored list (deny unknown fields; integers; bounds per field)
2  resolve extends: one parent, which is `default`, a list an enabled pack ships, or another list of
   this world; no cycle; depth ≤ 4. A child's entry replaces the parent's entry with the same key
   (class, pair actor+body, block name); entries are never merged field by field across levels, except
   that an empty parameter block means "the parent's values" (Box2D's precedence: the explicit pair wins)
3  fill defaults: a pair entry's missing parameters take the default list's; a class without a pair
   entry for (person, it) gets contact `block`, kick `none`, throw `none`
4  cross-checks, each refused by name:
     every class an item names exists in the list its place runs
     kick.reach and throw.reach ≥ R + GAP + the largest half-extent any object of that class has
     every interaction kind named is built in or provided by an enabled pack (§4.7)
     at most 24 classes (the adapter's 32 Rapier groups, less the bits it reserves: slab, walls, solids,
     people, the flying body — QPL-9)
5  canonicalize: classes and pairs sorted by name; the result is the resolved list, encoded once
```

**Looking up** at run time is a function of (the place's resolved list, actor class, body class): a
sorted-vector binary search or a dense table indexed by class number, built when the list is reduced into
state (§4.9). Bodies' pipeline reads it where it reads a constant today: the stride's contact rule, the
push trigger, the offers, `validate`, the launch, the landing. No other pack reads it.

## 4.6 Where a list lives, and how a world selects one

**Three sources, one format, one owner.**

| Source | How it ships | Example |
| --- | --- | --- |
| A reference list of bodies | compiled into bodies (`include_str!` of a YAML file decoded by the same type) | `default` |
| A reference list of a pack that provides behaviour | compiled into that pack, handed to bodies with its interaction kinds (§4.7) | an `ice` pack ships `winter` |
| A world's own list | authored in the World Pack's `configure/bodies.yaml` | `warehouse`, `rink` above |

A **data-only list shared between worlds** (a physics list pack) is not in this step: S16's data packs
are Entity and Presentation Packs, and a new pack type is a framework change (`step-16-packages.md` §6.3).
Until then a shared list ships in a code pack or is copied. QPL-5 asks whether a later step adds one.

**Selecting.** A world names the list all its places run in `configure/bodies.yaml` `physics:`; absent,
`default`. A place's `body:` may name another (`physics:`), a region. Absent everything, the world runs
`default` and stores nothing.

**The world configuration seam (ARC-PL-a; a framework precursor, no kernel change).** `world.yaml` has
nowhere to carry a pack's configuration (A-9). The seam generalizes ARC-31's sections from content files
to the world:

```text
authoring   trait PackConfiguration: SystemIdentity            beside AuthoredSection
              type Configuration: DeserializeOwned              deserializing is validating
              fn references(&Configuration) -> Vec<Reference>   keys it names (places, items)
              fn requires(&Configuration) -> Vec<SystemId>      systems that must be enabled
              fn seed(&Seeding, &Configuration) -> Result<Vec<Emission>, Rejection>
                                                                its own vocabulary only
sdk         SystemPack gains `configures!()` (like owns_section!); Capability gains
            decode_configuration / configuration_of; a pack without one refuses, naming itself
worldpack   world.yaml gains optional `configure: [<system id>, …]`; each names
            configure/<system id>.yaml, decoded straight from the YAML stream into the owner's type
            (line and column, DEP-10). Refused by name: an owner not enabled; a system that configures
            nothing; a missing or undeclared file; a `requires` system not enabled; any refusal of the
            owner's type
order       configuration is seeded after passages and locations and before every section, in the
            order of `configure:`. A world without `configure:` seeds exactly what it seeds today, in
            the same order, so its ids, facts and digests do not move
```

It is generic: the first user is bodies; `economy`, `inventory` (`PERSON_CAPACITY`), `conversation`
(`INTERACTION_RANGE`), `relationships` (regard values), `movement` (`MAX_STRIDE`) can each adopt it later
with the same three-line change, which is what closes the "rules" gap of §8 (G-8 of S16).

**Why a separate file, not inline in `world.yaml`.** `ac1_composability` check 3 compares the two towns'
`world.yaml` key by key (S16 F-E5); a world that configures a pack differently would show as a world
delta in `world.yaml`. A file named by a list entry keeps `world.yaml` a manifest and puts configuration
where its owner's type governs it, as `places:` names `places/<key>.yaml`.

## 4.7 New interaction kinds as code, and consequences as packs

Two plug-in shapes, chosen by one question: **does the new behaviour change where a body ends up within
bodies' resolution, or does it change something else as a consequence?**

**(A) It changes where a loose object ends up → an interaction kind, a provider bodies asks.**

```rust
/// A behaviour a physics list may select by name. Implemented by the pack that provides it; asked by
/// bodies while it resolves a push or a launch, before anything is recorded. Pure: reads only what it
/// is handed and its WorldRead, keeps nothing, reads no clock, writes and emits nothing (the
/// ArrivalResolver contract, ARC-39 item 1, and Box2D's callback rule).
pub trait InteractionKind: Send + Sync {
    /// The name a list uses, e.g. "slide". One namespace with bodies' built-in kinds.
    fn id(&self) -> InteractionId;

    /// The System Pack that provides it; must be enabled for a list to name it.
    fn provided_by(&self) -> SystemId;

    /// Its parameters: named integers, each with a floor, a ceiling and a default. A list's block for
    /// this kind is decoded against this schema and refused by name outside it.
    fn parameters(&self) -> &'static [ParameterSpec];

    /// The reference lists this pack ships, as YAML decoded by bodies' list type.
    fn reference_lists(&self) -> &'static [&'static str] { &[] }

    /// Where a displaced loose object should end, given the straight displacement bodies proposes.
    /// Returns a displacement no longer than `proposed.length + params["extra"]` (bodies clips it).
    fn displace(&self, world: &WorldRead<'_>, at: &DisplacementContext, params: &Parameters,
                proposed: Displacement) -> Displacement;
}
```

- `DisplacementContext`: the place, the object, its class, the actor and its class, the cause (`pushed`,
  `kicked`, `thrown`), and whether the surface or the class selected this kind. Integers and ids only;
  no Rapier type (`DEP-13`'s isolating interface).
- **Bodies still decides.** Whatever `displace` returns is clipped to its bound, swept by Rapier's
  shape cast (so it cannot tunnel), and verified on integers (SD-O2's invariant, V-O); a failure degrades
  as today (the proposal, then the lattice, then stay). A provider can never move a person: people's
  positions go through presence's `arrivals()` and its rules (a)–(f), which already refuse a lengthened
  stride (step-11 §4.4.6), so "people slide on ice" is impossible by construction, not by convention.
- **Reaching bodies.** By ARC-PL-b's extension catalog (below), exactly as resolvers reach presence:
  process-wide, write-once, compiled-in code; per-world applicability comes from the world's list, and a
  kind no list names is never asked.
- **Worked example — `ice` (a third-party System Pack).** Depends on bodies. Provides `slide` with
  parameter `extra` (0 … 3 000 mm, default 1 000) and ships a `winter` reference list. `displace`
  extends a push or a landing along its direction by `extra` scaled by the material's friction (integer
  arithmetic). A world selects `winter`, or extends it. Removing `ice` from `systems:` makes any list
  naming `slide` refuse at load, by name; a world that never named it is unchanged.

**(B) It changes another pack's state as a consequence → a System Pack that reacts and states facts
through the owners' constructors (ARC-26). No provider.**

- **Worked example — `fragile`.** A System Pack depending on bodies. It subscribes to bodies' public
  `object-moved`; for an object whose class is one its own configuration names fragile (read through a
  published read-only `mineworld_bodies::class_of(world, object)`), moved `how: kicked | thrown`, it
  states, in one emission list: its own `broke { object, by }`, and bodies' **new**
  `object-removed { object, place }`, built by a bodies-owned checked constructor
  `bodies::remove(world, object)` which bodies reduces (the `Lying` row goes; the Item stays as a
  record, `INV-11`). Bodies, as owner, refuses anything its invariants refuse.
- **"Breaks into items" — the honest limit.** Shards that appear on the floor would be new Item entities
  created during a dispatch; the kernel creates entities only through `World::create_entity(&mut self)`
  (`kernel/src/world.rs:571`), at assembly, not from a System. So in this step `fragile` can (a) remove
  the object and (b) if `item` and `inventory` are enabled, produce shard *kinds* into the breaker's
  holdings through inventory's production constructor. Shards as new loose objects need runtime entity
  creation by a System: a kernel question, operator-material, QPL-11.

**The generalized extension catalog (ARC-PL-b; a framework precursor).** Today `installed!` has one
hard-wired `resolution:` line and `worldpack` one hard-wired `register_resolvers` call (A-7). The second
catalog makes the abstraction earned (`CLAUDE.md` §4 rule 11):

```text
installed! {
    perception: mineworld_presence::PerceptionProvider;
    extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers:
        [mineworld_bodies::BodiesSystem,];
    extension mineworld_bodies::InteractionKind => mineworld_bodies::register_interactions:
        [];                                                   // `ice` would join here
    Presence => mineworld_presence::PresenceSystem,
    …
}
```

- The macro expands `Capability::register_extensions()`, calling each named function with its list;
  `worldpack::compose` calls that one function. Neither the sdk nor `worldpack` names a pack or a trait
  again; a third catalog is a line in `systems/installed`.
- The `resolution:` line becomes the first `extension` line in the same PR; nothing about presence's
  catalog changes (write-once; a different list panics naming both; `require_registered`), and every
  fact is byte-identical.
- Installing a pack that provides a kind is ARC-33's two lines plus one entry in an `extension` list —
  the "three for an arrival resolver" S16 §6.1 already counts.

## 4.8 Ownership

| State or fact | Owner | Written only while | Change |
| --- | --- | --- | --- |
| Where a person is (`Presence`) | presence | reducing `arrived` | none; providers cannot reach it |
| Where a loose object lies (`LooseObjects`), its shape (`BodyShape`) | bodies | reducing `object-placed`, `object-moved`, **`object-removed`** (new, for consequence packs) | unchanged schemas |
| A loose object's class (**`BodyClass`**, new) | bodies | reducing **`object-classed { object, class }`** (genesis, seeded only when the item's `body:` names a class) | a separate fact and component, so `body-formed` and `BodyShape` keep schema 1 and unclassed worlds stay byte-identical (QPL-3) |
| The resolved physics list of each place (**`PlacePhysics`**, new) | bodies | reducing **`physics-configured`** (genesis, only when a world configures bodies) and `place-shaped` (a region's override) | new component; absent → `default` |
| A provider's code | its pack | — (no state) | registered by `installed!` |
| A consequence (`broke`) | the consequence pack | its own reductions | its own vocabulary |
| Shard holdings | inventory | reducing its production fact | stated by the consequence pack through inventory's constructor |

Single ownership holds: no component gains a second writer; every cross-pack effect is a fact stated
through its owner's constructor and reduced by its owner (ARC-26).

## 4.9 Determinism and persistence

1. **The list is state, reduced from a genesis fact.** `configure/bodies.yaml` seeds one
   `physics-configured { lists: [ResolvedList], world: ListId }` (bodies' vocabulary, public,
   subjectless). Bodies reduces it into `PlacePhysics { list: ResolvedList }` on every Place, and a
   region's `place-shaped` overrides its own place's. Snapshots carry it; replay re-executes genesis from
   the journal, so a resumed world runs the list it was created with, byte for byte (ARC-25).
2. **Nothing is stored when nothing is configured.** A world with no `configure: [bodies]` and no
   `physics:` on any place seeds no new fact and writes no `PlacePhysics`; the resolver, the reactions and
   the actions read "absent" as the compiled-in `default`. This is what makes PL-I1 (byte-identity) hold
   without re-baselining any digest.
3. **Changing the compiled-in `default` is a bodies `VERSION` bump.** A test pins `(VERSION, default
   list digest)` together, as `rapier_pin` pins `(VERSION, Rapier)` (DC-5). An old save is then refused by
   name.
4. **Changing a provider's code is its pack's `VERSION` bump.** The provider's pack must be enabled for a
   list to name its kind (§4.5), so its declaration is in the composition record and a resume against
   another version is refused by name (A-12) — no kernel change.
5. **Changing a world's list is detected on resume (the drift check, part of ARC-PL-a).** Resume does not
   re-read content today (A-13). The configuration seam adds one host-side check, in `worldpack`, used by
   every host that resumes from a World Pack (`run`, `server`; `replay` and `biography` read the save
   only): re-seed the configuration from the World Pack, compare it with the save's genesis
   configuration facts byte for byte, and refuse on a difference — "the world's configuration for
   'bodies' differs from the save's (list 'warehouse' v2 here, v1 in the save)". The same moment and the
   same refusal style as the composition check; no kernel type changes. Whether drift in *other* content
   (people, places, items) should also be refused is a separate, wider question (QPL-12).
6. **Floats.** Unchanged: inside `rapier.rs` only, converted from the list's integers by one function
   per unit; I-3, I-4, I-5 and DC-1 … DC-9 of step-11 hold for every list.
7. **Order.** Lookups are pure functions of sorted data; providers are asked in `InteractionId` order when
   more than one applies to one displacement (a surface's kind, then the class's); the canonical Rapier
   insertion order does not change.

## 4.10 Relation to S16 (`requires:`, versions)

- A world-authored list is versioned by its `version` counter and travels with the World Pack's semver
  (S16 §4.2).
- A list shipped by a third-party pack is reached by enabling the pack in `systems:` and, being
  third-party, naming it in `requires:` with a range (S16 §4.2 rule 1). Nothing new is needed in
  `requires:`.
- Raising `default`'s content is a bodies `SystemVersion` bump, so under S16's QSE-7 a breaking release.
- `packs show bodies` (S16 §4.8) could list a pack's reference lists and kinds; proposed, not required.

## 4.11 Clients and presentation

- **No rule in a client** (I-8, I-S14-1). A client never reads a list's parameters to decide anything.
- **Disclosure.** The `loose-objects` listing gains each object's `class` (bodies' disclosure; a client
  may choose a mesh or sprite by class through its Presentation Pack's bindings — R-S15-2's wish, now
  with a source). The list itself is not disclosed. A wire addition is S11's to carry (coordination
  ruling 1); proposed as R-S11-PL-1, additive.
- **3D prediction.** The 3D client may map classes to Jolt collision layers for its local prediction
  (12e's player does not mask loose objects; a later client may mask the classes whose contact is
  `block`, from disclosure only); it remains prediction, corrected by the 150 mm rule.
- **2D and 3D still share one semantic path** (`AC-13`): kick, throw and shove are the same requests;
  which are offered follows from the list, server-side.

---

# 5. Invariants (proposed; frozen only by the primary session or the operator)

- **PL-I1 Default is byte-identical.** A world that configures nothing — no `configure:` entry for
  bodies, no `physics:` and no `class:` in any `body:` — produces facts, journal inputs and snapshots
  byte-identical to the merged pack before the refactor: 12b's `long_run` bytes (F-O13), 12c's
  bodies-yard 30-day sha, and both towns' 300-day seed-7 digests as 12d re-baselines them. No digest is
  re-baselined by any PL PR.
- **PL-I2 Explicit default is behaviourally identical.** The same world with `configure/bodies.yaml`
  naming `default` (or a list that `extends: default` and overrides nothing) records exactly one more
  genesis fact (`physics-configured`), and every fact after genesis is identical to PL-I1's run in
  type, payload, subjects, place, causation and order — its event id, and every event id that refers to
  one, offset by exactly that one fact. The test compares the two logs with ids renumbered, and fails on
  any other difference.
- **PL-I3 Removability.** A world without `bodies` is a valid world and unchanged; a world that
  configures bodies, names a class or a region, or names a provided kind without enabling its pack is
  refused at load by name — never run differently (`AC-2`, S16 §6.1 rule 4).
- **PL-I4 No kernel, contract or persistence change.** `kernel/`, `contracts/`, `persistence/` have no
  diff in any PL PR. The configuration seam and the drift check live in `authoring`, `sdk`,
  `systems/installed`, `worldpack` and the hosts' resume call. A need beyond that is a material stop.
- **PL-I5 Single ownership.** Bodies alone writes `LooseObjects`, `BodyShape`, `BodyClass`,
  `PlacePhysics`; presence alone writes `Presence`. Providers write and emit nothing. A consequence pack
  changes bodies' state only through a bodies-owned constructor, reduced by bodies (ARC-26).
- **PL-I6 Engine invariants hold for every list.** For any accepted list: no two people in a place
  closer than `CLEARANCE` after any request (I-12); a nudge at most the list's `nudge.max` + `GAP`, at
  most its generations and people (I-11, now per list, never above 310 mm / 2 / 4); nothing inside a wall
  or a solid; no tunnelling; verify-then-degrade on integers after every resolution, every push, every
  landing, every provider's displacement. No list value or provider can switch these off.
- **PL-I7 Integers only, total, bounded.** Every authored and stored value is an integer within its
  published bounds; every class pair resolves at load; nothing is looked up at run time that can fail.
- **PL-I8 A save carries its list.** A configured world's resolved lists are state; a resume runs the
  saved list, and a resume against a World Pack whose configuration differs is refused by name. The
  compiled-in `default` is pinned to bodies' `VERSION`.
- **PL-I9 No rule in a client.** No client reads a list parameter to decide anything (I-8, I-S14-1).
- **PL-I10 The precursor names no physics.** PL-a's diff names no body, physics, Rapier, nudge, push,
  kick or collision (a scan in the style of 12a's `seam_vocabulary`), and is proven with synthetic
  test-only packs.

---

# 6. Change amplification (`CLAUDE.md` §4 rule 5)

**What adding a physics behaviour costs, before and after.**

| Wanted | Today (12c) | After S-PL |
| --- | --- | --- |
| Calmer crowds (smaller nudges) | edit `geometry.rs`, bump bodies `VERSION`, every world changes | one line in a world's `configure/bodies.yaml` |
| Heavy crates nobody can kick | a new code path in `offer.rs`, `launch.rs`, `push.rs`; a fork for one world | a class and one pair entry in that world |
| A different rest rule (AO-2's p4) | DO-17: a code change after measurement | a list parameter, per world |
| Ice that makes things slide | edit bodies (fork) | a new pack implementing `InteractionKind`; three lines in `systems/installed`; bodies unedited |
| Fragile things break | edit bodies and item and inventory (fork) | a new pack reacting to `object-moved`; bodies gains one constructor once (PL-d) |

**What S-PL itself edits that already exists**, and only this:

```text
authoring            PackConfiguration trait (PL-a)
sdk/rust             configures!(); installed!'s `extension` lines replace `resolution:` (PL-a)
systems/installed    the resolution line rewritten as an extension line (PL-a); later one entry per
                     provided kind (PL-d)
worldpack            world.yaml `configure:`, configure/<system>.yaml, seeding order, the drift check (PL-a)
tools/cli, server    one call to the drift check at each resume (PL-a)
systems/bodies       the list type, lookups, PlacePhysics, BodyClass, object-removed, the
                     InteractionKind trait and catalog (PL-b … PL-d)
docs                 ARC-PL-a, ARC-PL-b, ARC-PL-c, DEP-PL-a; MODULE_SPEC §4.1 (configure, body: class
                     and physics), §9 (configuration schema implemented); a bodies subsection
```

Not edited: `kernel/`, `contracts/`, `persistence/`, `cognition/`, `clients/` (until S11/S12/S14 take the
additive class disclosure), every System Pack other than bodies (presence untouched — its catalog only
moves to the generic line).

---

# 7. Timing and PR split

## 7.1 Where the step sits against 12c and 12d

```text
12c   READY FOR OPERATOR REVIEW (mvp0/pr-12c-objects). Nothing of S-PL enters it. Its constants and
      rules are exactly what PL-b's `default` must reproduce, so 12c's merged tree is PL-b's reference.
12d   the towns get bodies; the one re-baseline of both towns' digests (coordination ruling 5).
      PL-a may run beside it (it touches no bodies and no world). PL-b waits for 12d's merge.
12e   the 3D client; independent of PL-b and PL-c (no wire change until PL-c's additive class
      disclosure, which S11 carries).
S16   E-a edits sdk's SystemPack and installed! (`package!()`); E-b adds world.yaml keys. PL-a lands
      after E-a (same macro and trait), and rebases with E-b on worldpack/src/format.rs — whichever lands
      second takes a one-key conflict.
```

**Recommendation: a refactor after 12d, with the framework precursor in parallel.**

- *Not inside 12c or 12d.* 12c is finishing and must not be destabilized. 12d is the one PR allowed to
  re-baseline the towns; a refactor inside it could not prove byte-identity, because the digests move
  there anyway.
- *Not before 12d.* It would put a refactor between two frozen PRs on S14's critical path (12d → 12e →
  S14), and 12d's content needs no class: social-cafe's loose objects are the same boxes and balls.
- *After 12d* PL-b's byte-identity is checked against the strongest evidence available: 12b's long run,
  12c's sandbox and both towns' freshly re-baselined 300-day digests.
- *PL-a now-ish*, in parallel: it names no physics, touches no pack's behaviour, and is proven by
  synthetic packs, like 12a and 11c.

```text
PL-a  (∥ 12d, after S16 E-a) ─┐
                              ├─► PL-c ─► PL-d
12c ─► 12d ─► PL-b  (∥ 12e) ──┘
```

## 7.2 The PRs

Each PR starts in a fresh session on its own branch and worktree, is detailed to the commit and frozen
before code (`CLAUDE.md` §3), and writes its specifications in its first commit (§2.2). The criteria
below are fixed now, before anything is measured (`ARC-23`); each guarded criterion names the mutation
that must break it.

| PR | Scope | Integration checkpoint | Adversarial criteria (fixed before measuring) |
| --- | --- | --- | --- |
| **PL-a** World configuration and extension catalogs (framework precursor; names no physics) | ARC-PL-a, ARC-PL-b. `authoring`: `PackConfiguration`. `sdk`: `configures!()`, `extension` lines, `Capability::register_extensions()`. `systems/installed`: the resolution line rewritten. `worldpack`: `configure:`, `configure/<id>.yaml`, seeding order, the drift check. Hosts: one drift-check call at resume. Test-only packs: `tuning` (configures one integer that changes what it states) and `relays` (a pack-owned trait with a second extension catalog). `MODULE_SPEC.md` §4.1 and §9. | Through the real binary: the three worlds' 300-day seed-7 facts byte-identical to `main` (no `configure:` anywhere); a scratch world with `configure: [tuning]` runs 30 days and states what its configuration says; SIGKILL and resume byte-identical; resume after editing `configure/tuning.yaml` refused by name; resume after editing nothing accepted; both extension catalogs registered by `compose`, in the listed order. | (1) `configure:` naming a system the world does not enable, one that configures nothing, a missing file, an unknown key — each refused by name, with line and column for the YAML. (2) Mutation: remove the drift check → the edited-configuration resume is accepted and the test fails by name. (3) Mutation: seed configuration after sections → `tuning`'s own section, whose reduction checks it against the configuration, is reduced without it, and the scratch world is refused at load by name. (4) The vocabulary scan finds no physics word (PL-I10). (5) `kernel/`, `contracts/`, `persistence/` have no diff (PL-I4). |
| **PL-b** bodies reads its rules from a list; `default` reproduces 12c (refactor) | DEP-PL-a. bodies: the list type and its validation, `default.yaml` compiled in, the resolved-list lookup replacing every parameter constant of §4.4 at every read site, the material and gravity conversions; engine constants stay. No authoring surface, no new component, no new fact: `VERSION` unchanged. | bodies-yard's 30-day sha, 12b's `long_run` bytes, and both towns' 300-day seed-7 digests byte-identical to `main` after 12d; arm64 and x86_64 (Rosetta) identical; cost per swept move and per kick within +5 % of `main`, measured in release. | (1) Mutation: `default.yaml`'s `nudge.max` 300 → 299 → `long_run` bytes differ, by name (the list is read, not the constant). (2) A structural test: no §4.4 parameter value survives as a literal outside `default.yaml` and the engine-constant module. (3) The three float bit patterns (0.5, 0.1, 9.81) pinned. (4) `(VERSION, default digest)` pinned like `rapier_pin`; editing `default.yaml` without a `VERSION` bump fails. (5) A list widening an engine bound (`nudge.max` 400) is refused at decode, naming the bound. |
| **PL-c** Authored lists, classes and regions | ARC-PL-c. bodies `configures!()`: `configure/bodies.yaml` (`physics:`, `lists:`), `extends`, the cross-checks; `physics-configured`, `PlacePhysics`; `class:` on item `body:`, `object-classed`, `BodyClass`; `physics:` on place `body:`; materials and `combine`; class in the `loose-objects` disclosure (additive; S11 carries the wire note). bodies `VERSION` 3. `MODULE_SPEC.md` §4.1's `body` row; the bodies subsection. | Test-time copies of bodies-yard (the established `market_composition.rs` pattern): (a) `default` named explicitly — PL-I2; (b) a `warehouse` list — a heavy box blocks the walker at contact, a light box kicked flies to a hand-computed landing ± 1 mm, `kick: none` is not offered and is refused `NoSupportedInteraction` when requested; (c) a region with `nudge.max` 150 — the 30-day scan holds I-11 at 160 mm in that place and 310 mm elsewhere; SIGKILL and resume byte-identical; resume after editing the list refused by name. | (1) Totality: every class pair of every accepted list resolves (a property test over generated lists). (2) Refused by name: a cycle in `extends`; an unknown class on an item; a reach below R + GAP + half-extent; more than 24 classes; a region naming an unknown list. (3) Mutation: ignore the region override → (c) fails. (4) Mutation: drop `BodyClass` from the lookup → (b)'s heavy box is pushed and the test fails. (5) PL-I6's 30-day scan passes under every list in the test set, and fails on a world with bodies disabled (I-10). |
| **PL-d** A provided kind and a consequence pack, from outside bodies | The `InteractionKind` trait and bodies' catalog (`register_interactions`), bodies' `object-removed` and its constructor, `class_of`. A test-only kind (`skid`) proving the seam. Two packs under `systems/`: `ice` (kind `slide`, reference list `winter`) and `fragile` (a consequence pack). A small world for the operator, `worlds/rink`. | In `worlds/rink`: a box pushed on the ice region ends `extra` further, hand-computed ± 1 mm; a kicked fragile vase is removed, `broke` is caused by the kick's `ActionId` (`AC-9`); with `item`/`inventory` enabled, shard kinds appear in the kicker's holdings; 30 days, PL-I6 scan, SIGKILL and resume byte-identical; cross-architecture. | (1) **The physics change-amplification proof:** the commit that installs `ice` touches only `systems/ice/**`, `systems/installed/**` and `Cargo.lock`; bodies is unedited (a diff check, like `ARC-35`). (2) A kind returning an over-long displacement is clipped; one aiming through a wall is stopped by the cast; one aiming into a person degrades (V-O); each by a test. (3) Removing `ice` from `systems:` refuses a world naming `slide`, by name; a world not naming it is unchanged. (4) `fragile` never writes a bodies component (a structural test) — only bodies' constructor. |

**Checkpoints between PRs.** After PL-a: the primary session confirms the seam's shape before bodies
adopts it. After PL-b: byte-identity evidence on all three worlds is reviewed before any behaviour
becomes configurable. After PL-c: the operator plays a configured world (QPL-13) before third-party
kinds are designed in detail.

---

# 8. Part 2 — framework pluggability audit

The operator's test (R-PL-2): a user picks an art style and a setting and enables the features they
want, without forking. For each area: what is already pluggable, what is hard-coded (files named), and
what would make it pluggable. "Planned" means a step document already designs it.

## 8.1 Art style and presentation (2D and 3D)

- **Already pluggable.** Presentation is a removable layer: removing the 3D client changes no system, no
  world and no test (`step-15-demo-3d.md` §9.1); clients decide nothing (I-8, I-S14-1), so a different
  renderer cannot change outcomes. The 2D client's design reads a Presentation Pack (`--presentation
  <dir>`) whose `asset_bindings.yaml` maps semantic roles to sprites, with `ARC-14`'s four style
  variants as binding sets (`step-13-client-2d.md` A-5, QS12-4) — planned, in flight on
  `mvp0/pr-s12-13a-walkable-2d`, not on `main`. S16 E-a/E-b give Presentation Packs identity, a version
  and a `requires:` entry — planned.
- **Hard-coded.**
  - Nothing reads a Presentation Pack (S16 F-E6): `presentation/mineworld-default/{2D,3D}/` are
    references for humans.
  - The 3D client's style is transcribed into code: `clients/3d-spike/scripts/slice/palette.gd`
    (colours "Read from presentation/mineworld-default/3D/references/"), `mats.gd`, `props.gd`.
  - The 3D client's places are built per world in GDScript, bound to `worlds/social-cafe`:
    `scripts/slice/slice_world.gd:17` ("matching `worlds/social-cafe`"), `cafe.gd`, `cafe_interior.gd`,
    `shop_interior.gd`, `street.gd`, `streetscape.gd`, `terrace.gd`. A new world gets no 3D scene.
  - Assets live inside the client (`clients/3d-spike/assets/`, `ASSETS.md`); no Asset Pack exists
    (S16 QSE-11).
  - A world cannot say which presentation it is authored for (`presentation_profile` refused;
    S16 replaces it with `requires:`), and the server does not tell a client (S16 QSE-9).
- **What would make it pluggable.**
  1. The server discloses the world's required Presentation Packs in the welcome frame (S11; S16's
     recorded hook).
  2. The 3D client builds a place's architecture from disclosure — bodies' `PlaceShape` (floor,
     solids), movement's passages — dressed by the Presentation Pack's bindings (place role → kit,
     body class → mesh, person → character profile), and keeps hand-built scenes only as one pack's
     content. 12e already builds colliders from disclosure; the visual half is the gap.
  3. The 2D client lands as designed (S12).
  4. Asset Packs become packs with licences and semantic bindings (`PACKAGE_FORMAT.md` §4; a later step).

## 8.2 Setting and content (worlds, people, items, places)

- **Already pluggable.** A World Pack is a directory anywhere, read with no rebuild (`mineworld
  run|server|validate <path>`); people, places, items and organizations are YAML files; each pack owns
  its section of them (ARC-31); `mineworld create` writes a new world. Item kinds are content (ARC-36).
  S16 adds versions, licences and shared Entity Packs (E-d) — planned.
- **Hard-coded.**
  - The entity types are a closed set (`contracts`: Person, Place, Item, Organization) — by design; a new
    type is a contract change (`MODULE_SPEC.md` §1).
  - Setting metadata (`era`, `calendar`, `geography`) is refused (`MODULE_SPEC.md` §4.1).
  - `consumption`'s tag words are constants: `EATEN = "food"`, `DRUNK = "drink"`
    (`systems/consumption/src/action.rs:10, :13`); a setting with other food words must use those tags.
  - A day is the same every day, and time of day is schedule's convention (ARC-32 limitations).
  - Places have geometry only where bodies is enabled; the 3D client's layout is per world (§8.1).
- **What would make it pluggable.** Entity Packs (S16 E-d); `consumption`'s words as configuration
  (ARC-PL-a's seam); setting metadata when a system first reads it (not before: `CLAUDE.md` §4 rule 11).

## 8.3 Feature enablement (systems on or off)

- **Already pluggable.** A world enables a pack by naming it in `systems:`; an absent pack's actions are
  `Unavailable` and the world otherwise runs (`INV-10`, `AC-2`); `AC-1` proved two different games from
  the same entities by composition alone (S9). Installing a bundled pack is a directory and two lines
  (ARC-33); one from another repository is S16 E-c — planned.
- **Hard-coded.**
  - Installing needs a rebuild (ARC-33; installing without one is `ARC-8`'s Tier 1, a non-goal).
  - `worldpack` depends on presence and movement for the `location` and `passages` fields (ARC-31 item
    5, ARC-33 item 4).
  - Runtime `World::disable` of a resolver pack is not honoured (QB-17).
  - `systems/installed`'s `resolution:` line is presence-specific in the sdk and `worldpack` (A-7).
- **What would make it pluggable.** ARC-PL-b's generic extension line (PL-a); the rest is planned or a
  deliberate non-goal.

## 8.4 Rules (economy numbers, capacity, schedules, physics)

- **Already content.** Prices, wallets and shops (`economy:`), jobs and wages (`job:`), holdings
  (`holdings:`), routines (`routine:`), item kinds (`item:`), place geometry and loose objects (`body:`).
- **Hard-coded** (A-10): `inventory::PERSON_CAPACITY` 6 (`systems/inventory/src/admit.rs:18`);
  `conversation::INTERACTION_RANGE` 3 000 mm and `CONVERSATION_GAP` 300 s
  (`systems/conversation/src/action.rs:16`, `system.rs:29`); `group_activity::INVITE_RANGE`,
  `ACTIVITY_LENGTH`, `INVITATION_LIFETIME` (`systems/group-activity/src/{action,process,component}.rs`);
  `relationships`' regard and familiarity values (`systems/relationships/src/system.rs:22–30`);
  `movement::MAX_STRIDE` (`systems/movement/src/action.rs:23`); every bodies constant and rule (A-1, A-2).
- **What would make it pluggable.** ARC-PL-a's per-pack configuration (the `MODULE_SPEC.md` §9
  "configuration schema" nobody implemented, S16 G-8), adopted pack by pack with declared bounds: bodies
  first (this step), then each pack when a world first needs a different value. A different *rule* stays
  a System Pack (`MODULE_SPEC.md` §4 constraint 3).

## 8.5 Controllers and AI

- **Already pluggable.** A controller only reads `Observation`s and submits `ActionIntent`s
  (`MODULE_SPEC.md` §5); any process speaking the protocol can drive a seat (S11), so an external
  controller in any language needs no fork; complete affordances let a controller use packs it was never
  compiled against (ARC-34); the model backend is the operator's, never the world's (`AC-4`, S10
  planned).
- **Hard-coded.** `mineworld run` always uses `PacedRuleController` (`tools/cli/src/run.rs:107`);
  `server --agent` always `RuleController` (`tools/cli/src/agent.rs:56`); the rule controller is compiled
  into `tools/cli` and names five packs (S16 §2.1); its offer band is frozen (ARC-34); no per-world or
  per-seat selection (`cognition_profile` refused).
- **What would make it pluggable.** Per-seat controller selection by the operator (S10/MVP-1, S16
  QSE-10); hosted seats driven by an external controller process over the protocol (S11) — mostly
  planned.

## 8.6 The checklist: what a user can change without touching MineWorld's code

`yes` = today on `main`; `planned` = designed in a named step; `S-PL` = this step; `gap` = §8.7.

| A user wants to … | Status | How |
| --- | --- | --- |
| write a new world: people, places, items, organizations, seats | **yes** | a World Pack directory |
| give people names, routines, jobs, money, belongings | **yes** | sections in content files |
| set prices, wages, opening balances | **yes** | `economy:`, `job:` sections |
| turn talking, relationships, shops, work, bodies on or off | **yes** | `systems:` |
| add a feature written by someone else | **planned** (S16 E-c) | two lines in `systems/installed`, rebuild |
| share item kinds between worlds | **planned** (S16 E-d) | an Entity Pack, `requires:` |
| version a world and state its licence | **planned** (S16 E-a) | `world.yaml` fields |
| lay out walls, furniture and loose objects | **yes** (bodies) | `body:` sections |
| choose how people and objects interact (pushing, kicking, throwing, nudging, materials) | **S-PL** | a physics list in `configure/bodies.yaml` |
| add a new kind of physical behaviour (ice, sticky, bouncy) | **S-PL** | a pack implementing `InteractionKind` (code, but not *our* code) |
| change rule numbers in other packs (carrying capacity, talking range) | **gap** G-PL-4 | ARC-PL-a's seam, per pack |
| play in a 2D style of their choosing | **planned** (S12) | a Presentation Pack's bindings, `--presentation`, `--variant` |
| play in a 3D style of their choosing, or a new world in 3D | **gap** G-PL-1 | the 3D client must read a Presentation Pack and build from disclosure |
| bring their own assets as a package | **gap** G-PL-2 | Asset Packs |
| drive a character with their own AI | **yes** over the protocol (S11); per-seat selection **planned** (S10) | |
| install a feature without rebuilding | **no** (non-goal) | `ARC-8` Tier 1, after MVP |

## 8.7 Gaps, mapped to steps

| ID | Gap | Owning step |
| --- | --- | --- |
| G-PL-1 | The 3D client hard-codes the default style (`palette.gd`, `mats.gd`, `props.gd`) and social-cafe's layout (`slice_world.gd`, `cafe*.gd`, `street*.gd`); a new world or style gets no 3D scene | **S14**: a new PR "3D reads a Presentation Pack and builds places from disclosure", after 12e; **S11** carries the welcome-frame field; **S16** declares the pack |
| G-PL-2 | No Asset Packs; assets live in the client | **new step** after MVP-0 (S16 QSE-11 keeps it out of Milestone E) |
| G-PL-3 | No program reads a Presentation Pack; a world cannot select one | **S16** (declare) + **S11** (disclose) + **S12**, **S14** (apply) — QSE-9's hook, to be scheduled explicitly |
| G-PL-4 | Rule constants of every pack are compile-time (A-10); no per-pack configuration (S16 G-8) | **S-PL** PL-a (the seam) + **new step S-CFG** (adoption by conversation, inventory, group-activity, relationships, movement, consumption, as worlds need them) |
| G-PL-5 | Physics behaviour is compile-time (A-1, A-2) | **S-PL** (this step) |
| G-PL-6 | A second extension catalog would edit the sdk and `worldpack` again (A-7) | **S-PL** PL-a (ARC-PL-b) |
| G-PL-7 | Controllers are fixed by the host command; no per-seat selection | **S10** (per-seat selection), **S11** (external controllers over the protocol) |
| G-PL-8 | Shared, data-only physics lists | later (QPL-5) |
| G-PL-9 | A System cannot create an entity during a dispatch (shards, crafting, spawning) | **kernel question**, operator-material (QPL-11); outside MVP-0 unless decided |
| G-PL-10 | Resume does not detect drift in a World Pack's content | configuration: **S-PL** PL-a; all content: QPL-12 |
| G-PL-11 | `README.md` still says systems, scheduling, persistence and the clients "do not yet" exist, and does not say that the demo is one default composition | a docs PR, proposed wording §8.8 |
| G-PL-12 | Runtime disable of a resolver or interaction pack | **QB-17** (deferred, unchanged) |

## 8.8 Proposed wording for `README.md` (for humans; short, links onward)

Replaces the opening paragraph and the "Status" section; the rest of the README stays. Proposed, not
applied (the primary session decides, QPL-15):

~~~markdown
# MineWorld

> **An open-source framework for building persistent, modular, playable worlds.**

MineWorld is not a game. It is the layer underneath one: people, places, items and organizations,
the time they live in, and the rules of what can happen between them — each rule an independently
installable module. What you see when you run it today is the **default demo**, one composition of
those modules. Yours can look different, be set somewhere else, and play differently, without
forking MineWorld:

- **Setting** — write your own world: its people, places, items and organizations, as files.
- **Features** — switch systems on or off per world (talking, relationships, shops, work, bodies),
  or install systems other people wrote.
- **Rules** — tune the numbers a system exposes. Physical interaction is a **physics list**: you
  decide which kinds of object push, block, fly or break, and a new kind of behaviour is a module,
  not a patch.
- **Look** — choose a presentation: the bundled 2D and 3D styles are defaults, not requirements.
- **Minds** — drive any character as a human, with rules, or with your own AI.

```text
install modules  →  compose a world  →  configure  →  run
```

## Status

MVP-0 is being built in the open. The kernel, persistence, the server and the System Packs for
movement, conversation, relationships, group activity, items, money, work and bodies exist and are
tested; the 2D and 3D reference clients and package versioning are in progress. See
[`docs/MVP_STATUS.md`](docs/MVP_STATUS.md).
~~~

The statement "LM-native" in today's README is kept or dropped by the operator's choice (QPL-15); the
wording above does not depend on it.

---

# 9. Risks

| ID | Risk | Mitigation |
| --- | --- | --- |
| R-PL-1 | PL-b's refactor changes a result subtly — a float conversion, an insertion order, a bound read one step late — and the default stops being 12c. | PL-I1 on three independent references (12b's long run, 12c's sandbox, both towns after 12d); the float bit patterns pinned; a mutation proving the list is what is read; cross-architecture. |
| R-PL-2 | The invariants were proven for one set of numbers; an accepted list breaks one (a reach too short to touch, a nudge chain that cannot clear). | Engine bounds on every parameter (§4.4); cross-checks at load (§4.5); PL-I6's 30-day scan under every list in PL-c's test set; verify-then-degrade unchanged. |
| R-PL-3 | The configuration seam becomes a grab-bag — untyped maps, values nobody owns. | Owner-typed, one file per pack, unknown keys refused, bounds in the owner's type; no generic "settings" bag (`CLAUDE.md` §4 rule 7). |
| R-PL-4 | PL-a collides with S16 E-a/E-b in `sdk` and `worldpack/src/format.rs`. | Ordered after E-a; one-key rebase with E-b (§7.1). |
| R-PL-5 | A provided kind breaks determinism (a float, a `std` transcendental, a hash order). | Its interface is integers and ids only; no Rapier type crosses it; DC-3 applies to providers; PL-d runs cross-architecture with a provider active. |
| R-PL-6 | A list makes the paced controller's world degenerate (kicks everywhere, nothing reachable) — the AO-2 story again. | Offers follow the list, so a list that offers less is the remedy (ARC-34); each world that ships a list carries its own activity check; the controller is never the remedy. |
| R-PL-7 | Users read "physics list" as realistic physics. | R-3 of step-11 stands; the bodies README says what a list can and cannot do (no momentum between requests, no chain reactions, no rotation). |
| R-PL-8 | Per-place copies of the list grow snapshots. | A resolved list is under ~2 KB; places per world are tens; measured in PL-c and reported. A shared store is a later optimization, not a contract. |

---

# 10. Questions (QPL-1 …)

**[OM]** marks an operator-material question: it changes scope, a frozen specification or model, a
milestone, or the kernel. Each has a recommendation.

| ID | Question | Recommendation |
| --- | --- | --- |
| **QPL-1 [OM]** | Is a physics list *data that selects and parameterizes behaviour code provides*, with new behaviour always a System Pack (`MODULE_SPEC.md` §4 constraint 3 kept as written)? Or may a list define behaviour (a scripting language)? | **Data only.** It keeps determinism, typing and ownership where they are, and it is Geant4's own split. |
| **QPL-2 [OM]** | ARC-PL-a: a generic world configuration seam — `world.yaml` `configure: [<system>]`, `configure/<system>.yaml` typed by its owner, seeded before sections, with a drift check at resume. It changes the World Pack model (`MODULE_SPEC.md` §4, §4.1) and implements §9's "configuration schema". Alternatives: (b) lists only inside bodies' place sections (no world-level list, repeated per place); (c) inline in `world.yaml` (touches `ac1_composability` check 3's comparison). | **(a).** One seam closes G-PL-4 for every pack, and no pack-specific field enters the loader. |
| **QPL-3** | Classes as a separate genesis fact and component (`object-classed`, `BodyClass`) rather than a field of `body-formed` / `BodyShape`. | **Separate.** Unclassed worlds stay byte-identical (PL-I1). |
| **QPL-4 [OM]** | Timing: nothing in 12c or 12d; PL-a beside 12d, after S16 E-a; PL-b after 12d's merge, beside 12e; then PL-c, PL-d. | **As stated (§7.1).** |
| **QPL-5** | A data-only "physics list pack" shared between worlds (a new S16 pack type, or carried by Entity Packs)? | **Later.** Lists ship in code packs (with their kinds) or in worlds; add a data pack when a shared list without code first exists. |
| **QPL-6** | The person's shape (R 300 mm, 1 720 mm) stays an engine constant, not a list parameter. | **Yes.** Capacity, clearance, the push search and the 3D capsule derive from it; a per-world person shape is its own design. |
| **QPL-7** | A built-in mass-ratio rule (push only if the body is light enough; kick speed scaled by mass)? | **Not now.** Mass is recorded per class; a ratio is expressed by `block` per class or by a provided kind. |
| **QPL-8** | May a list narrow object limits (half-extents, count) per class? | **Yes, narrow only**, in PL-c. |
| **QPL-9** | At most 24 classes per list (Rapier's 32 groups, less reserved bits). | **Accept**; raise only with a measured need. |
| **QPL-10** | ARC-PL-b: generalize `installed!`'s `resolution:` line into `extension <trait> => <register fn>: [...]` and migrate presence's catalog in PL-a. | **Yes.** No compatibility shim (`CLAUDE.md` §4 rule 12); byte-identical. |
| **QPL-11 [OM]** | "Fragile breaks into items" as new loose objects needs a System to create entities during a dispatch — a kernel change. Take it up? | **Not in S-PL.** `fragile` removes the object and may produce shard *kinds* into holdings. Runtime entity creation is its own kernel design, if wanted. |
| **QPL-12 [OM]** | Refuse a resume when the World Pack's *configuration* differs from the save's (recommended), or when *any* content differs (all genesis facts re-seeded and compared)? | **Configuration only, now.** Content drift changes every host's behaviour on existing saves and deserves its own decision. |
| **QPL-13 [OM]** | Are PL-a … PL-c MVP-0 scope, and is PL-d (a third-party kind `ice`, a consequence pack `fragile`, `worlds/rink`) an MVP-0 gate? | **PL-a … PL-c in MVP-0**: they are what makes "a framework" true for physics. **PL-d recommended in MVP-0** as the physics counterpart of `AC-1` (bodies unedited when a behaviour is added), the operator's choice. The operator plays a configured world after PL-c. |
| **QPL-14** | S-number for this step, and decision numbers for ARC-PL-a, ARC-PL-b, ARC-PL-c, DEP-PL-a. | **The primary session assigns them** (coordination ruling 6). |
| **QPL-15 [OM]** | The README's "what MineWorld is" wording (§8.8); keep "LM-native" in the tagline? | **Adopt §8.8**; the tagline is the operator's call. |
| **QPL-16** | Disclose each loose object's class in the `loose-objects` listing (additive; S11 owns the wire, R-S11-PL-1). | **Yes, in PL-c**, coordinated with S11. |
| **QPL-17 [OM]** | G-PL-1: add an S14 PR in which the 3D client reads a Presentation Pack and builds places from disclosure, so a new world or style gets a 3D scene. | **Yes, after 12e**, not gating `AC-14`. It is the 3D half of R-PL-2. |
| **QPL-18** | A new step S-CFG: other packs adopt ARC-PL-a's configuration (capacity, ranges, regard values, consumption words), one PR per pack when a world needs it. | **Create it, not an MVP-0 gate**, unless the operator wants a second configured pack as evidence. |
| **QPL-19** | A world-authored list's version: an integer counter (like `SystemVersion`) rather than semver. | **Integer counter.** The world's own semver (S16) versions the pack that carries it. |
| **QPL-20** | PL-b's cost bound: at most +5 % per swept move and per kick, release build, against `main`. | **Accept**, fixed before measuring. |

---

# 11. Proposed decision records (drafts; not in `docs/DECISIONS.md`)

## 11.1 ARC-PL-a — A System Pack may be configured per world, by a file its owner types

**Problem.** `MODULE_SPEC.md` §9 gives every pack a configuration schema; nothing implements it (S16
G-8), so every rule number is compile-time (A-10) and a world that needs another value forks the pack.
**Choice.** `authoring::PackConfiguration` (sibling of `AuthoredSection`); `world.yaml` `configure:`
lists owners; `configure/<system>.yaml` is decoded straight into the owner's type; its facts are the
owner's vocabulary, seeded after passages and locations and before sections; the owner reduces them into
its own state. A host resuming from a World Pack re-seeds the configuration and refuses a difference
from the save's, by name. A world without `configure:` seeds exactly what it did. **Rejected.** Fields in
`world.yaml` per pack (the loader learns packs, F-1 again); configuration in the kernel's composition
record (a kernel change); an untyped settings map. **Limitations.** A different *rule* is still a System
Pack (`MODULE_SPEC.md` §4 constraint 3); content drift other than configuration is not detected.

## 11.2 ARC-PL-b — Extension catalogs: a pack-owned trait, implemented by other packs, listed in the installed set

**Problem.** ARC-39's resolver catalog is wired by name in the sdk and `worldpack` (A-7); a second
catalog would wire again. **Choice.** `installed!` takes any number of `extension <trait> => <register
fn>: [<types>]` lines; `Capability::register_extensions()` calls each; `worldpack::compose` calls that.
Each catalog keeps ARC-39's rules: write-once, process-wide, compiled-in, pure members, inert where the
world does not use them. Presence's catalog becomes the first line. **Rejected.** A kernel extension slot
(QB-15's F2: a kernel change); linker-section registration (DEP-12). **Limitations.** As ARC-39: one
catalog per build; runtime disable not honoured (QB-17).

## 11.3 ARC-PL-c — Bodies' physics list

**Problem.** Bodies' rules are 39 constants and a dozen code paths (A-1, A-2); the operator asks that a
user decide how kinds of object interact (R-PL-4). **Choice.** §4: body classes; a typed, integer,
bounded, total list owned by bodies; the compiled-in `default` equal to 12c; three sources (bodies,
provider packs, worlds); selection per world and per place; resolved lists stored as bodies' state;
interaction kinds as code through an ARC-PL-b catalog; consequences as packs through owners'
constructors. Engine invariants hold for every list. **Rejected.** Rapier groups or materials in
content (an engine concept in content, §3 row 4); a scripting language (QPL-1); constants per world by
fork. **Limitations.** No person shape per list; no chain reactions, no momentum, no rotation; no
runtime entity creation (QPL-11).

## 11.4 DEP-PL-a — The physics list's format and interpreter are our own, on Rapier and `serde-saphyr`

**Options.** §3: Geant4, Unity, Godot, Rapier, Box2D, Factorio, RimWorld, our own. **Choice.** Build the
format and the lookup only; adopt Geant4's pattern and name; realize motion through Rapier (`DEP-13`)
with its groups, filters, hooks and materials; decode with `serde-saphyr` (`DEP-10`). No new dependency.
**Why not the others.** No library offers a deterministic, integer, server-authoritative semantic
interaction table in MineWorld's types; engines' masks are collision-level and engine-bound; games'
formats are bound to their loaders and, for Factorio and RimWorld, untyped where it matters.

---

# 12. Proposed amendments (text only; applied by the primary session)

**`overall.md` §3, after S16:**

```markdown
### S-PL — The physics list *(proposed 2026-10-08, operator requirement)*

**Design:** [`step-18-physics-list.md`](step-18-physics-list.md). What bodies does between people and
objects becomes a physics list — content a world chooses — and new physical behaviour becomes a pack,
not an edit to bodies. A generic per-world configuration seam (ARC-PL-a) and generic extension catalogs
(ARC-PL-b) are the framework precursor. No kernel or contract change.

- **Depends on:** 12d merged (PL-b), S16 E-a merged (PL-a). **Feeds:** S12, S14 (class disclosure),
  S-CFG.
- **Acceptance checkpoint:** a world that configures nothing is byte-identical (12b's long run, 12c's
  sandbox, both towns after 12d); an explicitly configured default changes only genesis; a configured
  world blocks, pushes, kicks and lands as its list says, with every engine invariant held; a resume
  against a changed configuration is refused by name; installing a new behaviour leaves bodies unedited.
```

**`overall.md` "Parallel build-out" lanes:** `S-PL: PL-a after S16 E-a (∥ 12d); PL-b after 12d (∥ 12e);
PL-c; PL-d.`

**`overall.md` §4:** a row `Operator requirement 2026-10-08: a framework, not a demo; users add their own
physics list | S-PL; G-PL-1 → S14, G-PL-3 → S11/S12/S14/S16, G-PL-4 → S-CFG`.

**Specifications, in the PRs that implement them (`CLAUDE.md` §2.2):** `MODULE_SPEC.md` §4 (model:
`configure:`), §4.1 (the field, `configure/`, the `body` row's `class:` and `physics:`), §9
(configuration schema: implemented as ARC-PL-a); `CORE_CONCEPTS.md` unchanged (the new terms are bodies'
vocabulary, §4.1); `systems/bodies/README.md` (what a list can and cannot do); `README.md` per §8.8 if
QPL-15 is accepted.
