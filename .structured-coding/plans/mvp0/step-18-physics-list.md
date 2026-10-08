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
| Where a loose object lies (`LooseObjects`), its shape (`BodyShape`) | bodies | reducing `object-placed`, `object-moved`, **`object-removed`** (new, for consequence packs) | `BodyShape` gains `class` (schema 2; QPL-3) |
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
