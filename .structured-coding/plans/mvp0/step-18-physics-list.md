# Step 18 — The physics list: interaction rules between kinds of body as content and pluggable code

**Role:** step document for a new requirement of the operator (2026-10-08). It records the
requirement, an audit of the real source, a reuse comparison, the design of MineWorld's physics list,
a framework pluggability audit with its gaps, a PR split with checkpoints and adversarial criteria,
risks, questions (QPL-1 …) and draft decision records. It holds no frozen PR design.
**Effort:** `mvp0` · parent: [`overall.md`](overall.md) §3 (S15, S16), "Parallel build-out,
2026-10-08"; sibling: [`step-11-bodies.md`](step-11-bodies.md) (S15), [`step-16-packages.md`](step-16-packages.md)
(S16).
**Lifecycle:** `DRAFT — awaiting the primary session's review`. Nothing here authorizes
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
