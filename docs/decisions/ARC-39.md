## ARC-39 — An arrival is resolved before it is recorded

**Date** 2026-10-07 · **Approved by** the operator (step-11 QB-1, resolve before recording; QB-15, a
catalog registered at start-up, with its three bounds) and the primary session at PR 12a's design
freeze (step-11 §16.0: QR-2 and QR-4 decided, QR-1, QR-3, QR-5 … QR-12 accepted as recommended) ·
**Implements** [`ARCHITECTURE.md`](ARCHITECTURE.md) §§7–8 · **Relates to** `INV-7`, `INV-15`,
[`MVP.md`](MVP.md) §9 `AC-2`, `AC-9`, `ARC-15`, `ARC-23`, `ARC-25`, `ARC-26`, `ARC-33` · **Design**
`.structured-coding/plans/mvp0/step-11-bodies.md` §§4.4, 16 (S15, PR 12a)

**Problem.** Step S15 (bodies and physical interaction) needs a person's arrival to be able to end
short of where it was asked to go, and to move other people standing in the way. Three constraints
bound the answer:
- **The log holds only true arrivals** (the operator's QB-1). An `arrived` that is recorded and then
  corrected by a later fact is refused as a design, even though each fact would be true in sequence.
- **Movement still alone decides whether a move is allowed, and names no resolver.** A mover that had
  to call the S15 pack would make that pack impossible to remove (`AC-2`) and every future mover would
  have to remember it.
- **No kernel change and no contract change.** Presence owns where people are (`ARC-26`); the question
  "what does this arrival actually achieve" is presence's to ask, not the kernel's.

Presence already has the only public way to build an `arrived`: its constructor `arrival(world,
person, location)`, which every stating system calls under `ARC-26`. An arrival is built **inside**
`World::dispatch`, where the only things in reach are the stating system's `&self` and a `WorldRead`
(`kernel/src/view.rs`: entities, components, relations, processes — no composition, no registry, no
enabled state). So whatever answers the question must reach presence's constructor without being
handed to it per call.

**Options considered.**

```text
(a) correct after recording: the resolving pack     transient facts in the log (QB-1, refused by the
    reacts to each arrived and states a second one   operator)
(b) movement calls the resolving pack               the pack is no longer removable; every future
                                                     mover must remember it
(c) a kernel geometry seam                          a kernel change; the likely route for line of
                                                     access later, not needed here
(d) F2: a kernel extension slot, World::provide /   per-world rather than per-process, but a kernel
    WorldRead::provided                             change (QB-15)
(e) F1: presence asks every registered              chosen
    ArrivalResolver before it records, through a
    write-once catalog registered at start-up
```

**Choice: (e).**

1. **The seam is presence's** (`systems/presence/src/resolve.rs`).
   - `Arriving { person, from: Option<Location>, to }`: one arrival as presence knows it before it
     records anything. `from` is the person's current `Presence`, read by presence, `None` at a first
     placement. Only presence builds one.
   - `Resolution { reached, displaced: Vec<(PersonId, Location)>, stopped_by: Option<EntityId> }`,
     with private fields. A resolver starts from what it is handed and changes it only through
     `Resolution::stopped_at` and `Resolution::displacing`; `Resolution::unchanged` is the arrival
     exactly as decided.
   - `ArrivalResolver: Send + Sync`, with `resolver_of() -> SystemId` and `resolve(world, arriving,
     so_far) -> Resolution`. Its contract: read only the `WorldRead`; keep nothing; read no clock, no
     random source and no process-wide state; return `so_far` unchanged when the world holds none of
     the pack's own state about the people and the place involved — which is what keeps a world
     without the pack byte-identical; and the pack's `System::install` calls `require_registered`
     first (item 5).
   - A resolver cannot emit a fact. Its pack's own consequences of an arrival happen in that pack's
     reactions to the facts presence records.
2. **Two constructors.**
   - **`arrivals(world, person, to) -> Result<Vec<Emission>, Rejection>`, for movers.** It asks
     `admit`; builds the `Arriving`; folds the registered resolvers in ascending `SystemId`, starting
     from `Resolution::unchanged` and **checking the result after each resolver** (item 3); and
     returns, in this order: `arrived { person, reached }`; one `arrived` for each displaced person, in
     the resolution's order; and `stopped-short { person, wanted: to, reached, by }` when `reached ≠
     to`. Every `arrived` is built exactly as `arrival` builds it.
   - **`arrival(world, person, to) -> Result<Emission, Rejection>`, for placement.** Its signature is
     unchanged. It runs the same fold and refuses with item 3's code when the result is not unchanged;
     otherwise it returns exactly the one emission it returned before this decision. An arrival is
     therefore recorded as resolved or refused — never recorded unresolved.
   - Resolvers are asked in ascending `SystemId` order, so the order is a function of the resolvers'
     names alone: not of the `installed!` list, not of a world's `systems:` list.
3. **Presence still decides** (`ARC-26`). After each resolver, a resolution is refused with
   `Rejection::System { code: "resolution-refused", detail: "<resolver id>: <rule>" }`, which the
   stating system turns into `KernelError::FactRefusedByOwner` as it does any refusal of presence's:
   - (a) `reached` is in `to`'s place;
   - (b) `reached` has `to`'s facing, and a local position exactly when `to` has one;
   - (c) when `from` is in `to`'s place and both have local positions, `reached` is no farther from
     `from` than `to` is, compared as squared millimetres in `i128`: a resolver may shorten or bend an
     arrival, never lengthen it, so the stating system's own bound still bounds it;
   - (d) `admit(person, reached)`;
   - (e) every displaced entry names a living `Person`, not the walker, listed once, whose current
     `Presence` is in `to`'s place, displaced to a location in that place that keeps rule (b)'s local
     position rule and passes `admit`;
   - (f) `stopped_by`, when present, names an entity of this world, and is present only when `reached
     ≠ to`.
   Checking after each resolver names the resolver at fault (`ARC-23`); a check of the final result
   alone could not.
4. **`stopped-short` is presence's fact, stated by the stating system.** `StoppedShort { person,
   wanted, reached, by }`, owner presence, schema 1. An arrival that ends short of where it was asked
   to go is a fact about where a person is, which is presence's domain, and it names nobody who moved
   them. Presence reduces it into nothing. **Presence's declaration does not list it**: presence never
   states one — not at genesis, where `arrival` refuses a changed placement, and not in its reaction
   — and a declaration must be true. Only a stating system declares it (movement, with this decision).
   Every fact of one arrival — the walker's `arrived`, each displaced person's, the `stopped-short` — is
   stated in one emission list and caused by the request (`AC-9`).
5. **The catalog: the one named process-wide value.** `ENGINEERING_RULES.md` §15 forbids hidden global
   state; this is a stated exception, not a hidden one.
   - It is a `static OnceLock<Vec<Box<dyn ArrivalResolver>>>` in presence, with exactly three entry
     points: `register_resolvers(list)`, `registered_resolvers() -> Option<Vec<SystemId>>`, and
     `require_registered(&SystemId)`. Nothing else reads or writes it.
   - **It is not world state.** It is the build's compiled-in list of resolver code, identical for
     every world in the process, set before any world runs and never changed — the same kind of thing
     as the `installed!` list itself. Per-world applicability needs no filter, because a resolver is
     inert where its own state is absent: a world that does not install a resolver's pack has no table
     for that pack's components, and a read of a missing table answers `None` rather than failing.
   - `register_resolvers` sorts the list by `resolver_of()`. A list naming one id twice panics, naming
     it. If the catalog is unset, it is set. If it is set to the same ids, the call does nothing. If it
     is set to different ids, the call panics, naming both lists: one build has one catalog, and a
     second, different one is a defect of the host, not a condition to recover from.
   - **Never registered means no resolver.** `arrivals` and `arrival` consult the registered list, or
     none when nothing was registered. A process that composes worlds by hand and never registers —
     the kernel's, persistence's, the server's and every pack's own tests — therefore records exactly
     what it recorded before this decision.
6. **How the catalog is written.** `installed!` gains an optional line after `perception:`,
   `resolution: <trait path> => [ <type>, … ];`, each type followed by a comma, and expands it into
   `Capability::resolvers()`: one `Default` value of each listed type. A listed type that does not
   implement the trait does not compile; a test of the installed set holds that every listed resolver
   belongs to an installed pack. `systems/installed` writes `resolution:
   mineworld_presence::ArrivalResolver => [];` — empty until the first resolver pack is listed.
   `worldpack::compose`, the one assembly path of every host (server, `run`, `create`, `biography`,
   `inspect`, and `load` and `assemble` through it), registers `Capability::resolvers()` before it
   installs anything.
7. **A host that never registers fails loudly** (QB-15's second bound). Three guards, none in the
   kernel:
   - `compose` always registers, so a host cannot compose a world without registering;
   - **a resolver's pack refuses to join a world while its resolver is not registered**: its `install`
     calls `require_registered`, which panics — naming the pack, `register_resolvers` and this
     decision — when the catalog is unset or does not list it. A host that installs a resolver pack and
     never registered dies at assembly, before any arrival;
   - the first resolver pack checks, in its reactions to the recorded arrivals, the invariant it
     exists to keep, so an arrival that escaped resolution fails the dispatch (S15's next PR).
   A host that never registers and installs no resolver pack runs exactly as before: there is nothing
   to resolve, so nothing runs silently without resolvers. A panic rather than an error value, because
   the kernel's `install` hook refuses only with a `KernelError`, and a variant for this would be a
   kernel change.
8. **Versions.** Presence's `VERSION` is 3: its constructors now answer through the resolvers and its
   vocabulary gained a fact. `arrived` keeps schema 1. Movement's `VERSION` stays 1. A save written
   before this decision is refused by name at presence's record — "system 'presence' is v3 here, but
   the save was written by the older v2" — before any snapshot is restored (`ARC-25`: versions are
   refused, never guessed). For the same world, seed and inputs, the only bytes that differ between a
   save written before and one written after are the composition records: presence's version, and
   movement's declared emissions gaining `stopped-short`. Every fact is byte-identical while no
   resolver is listed.

**Runtime `World::disable` of a resolver pack is not honoured.** A world that disables a resolver's
pack at runtime still asks that pack's resolver, and the pack's state still answers it, because a
`WorldRead` cannot see whether a system is enabled and the catalog is per process, not per world.
Disabling exists for `AC-2` tests; `AC-2` for a resolver pack is shown at world level — a world that
does not list the pack, which this decision handles. Honouring runtime disable needs a kernel read of
enabled state, which is not decided (step-11 QB-17) and is not part of this decision.

**Tests that compose worlds by hand.** They never register, so they run with no resolver (item 5). A
test that needs resolvers registers its own list first and does not compose through `worldpack` in the
same process, because `compose` would register the build's list and a different list panics (item 5).
Cargo runs each integration-test file as its own process, which is what keeps one rule true for hosts
and tests alike.

**Accepted limitations.**
- **One catalog per process.** Two worlds in one process share it; a resolver's inertness where its
  state is absent is what keeps that harmless.
- **Runtime disable is not honoured** (above).
- **The constructor can be bypassed.** `Arrived::new` is public, and presence's reduction re-asks only
  `admit`, not the resolvers: re-resolving at reduction would run the resolvers on a displaced person's
  arrival against a state the walker's arrival has already changed. The contract's way is the
  constructor (`ARC-26`), and S15's first resolver pack catches the consequence of a bypass in its
  reactions (item 7).
- **Genesis is not resolved against other people.** A world's genesis facts are built against the
  assembled world before any of them is applied (`ARC-15`), so at genesis every `arrival` sees nobody
  placed. Two authored people placed on top of each other are refused by the resolving pack's own world
  validation, not by `arrival`.
- **No length bound on an arrival from another place, or from nowhere,** beyond "the same place": rule
  (c) applies only when the person is already in the destination place. A resolving pack's own bounds
  apply to its own resolutions; there is no seam-level number without geometry.
- **A resolver cannot emit**; its pack's consequences happen in its reactions to the recorded facts,
  which its resolver must predict.

**Note, 2026-10-07 (S15, PR 12b; step-11 §17.0 QP-1, QP-7, QR-11) — the first resolver.** The `bodies`
System Pack (`DEP-13`) is the first registered resolver. This note records what it adds to this
decision; items 1–8 are unchanged except where item 7 is refined below.

1. **Its bounds** (QR-11 moved them here from the seam). Per arrival that bodies resolves within a place:
   - no other person is moved by more than 310 mm (`NUDGE_MAX` 300 mm plus the character controller's
     10 mm `GAP`);
   - at most two generations of nudges (`CHAIN_MAX`), and at most four people moved (`NUDGED_MAX`); an
     arrival that needs more is stopped at contact instead, and nobody else moves;
   - nobody is moved into another place (rule (a) and (e) already refuse it) or through fixed
     geometry;
   - after quantization to whole millimetres, no two people in one place are closer than 595 mm
     (`CLEARANCE`, 2 × 300 mm − 5 mm), nobody's centre is outside the floor shrunk by 295 mm, and
     nobody's centre is within 295 mm of a solid. The check is on integers, and it degrades the result —
     blocked, then the advance halved up to eight times, then stay — rather than trusting the character
     controller (step-11 F-P6).
2. **Item 7, third bullet, is realized as a guard on the starting state** (QP-1). The bullet promised a
   check "in its reactions to the recorded arrivals". That check cannot be built: a reduction lets every
   subscriber react to one fact before the next fact of the same emission list is reduced, and
   `arrivals` returns the walker's `arrived` before the displaced people's, so a reaction to the walker's
   arrival sees the walker overlapping the people it has not yet seen move (step-11 F-B1). Bodies
   therefore checks, at the start of every resolution that is not inert, the place's **current** state:
   every pair of positioned people at least 595 mm apart, everyone inside the floor and out of every
   solid. Every resolved arrival keeps that state and genesis checked it, so a violation means an arrival
   escaped resolution — a constructor bypass (the limitation above) or a host that registered nothing.
   The resolver then **panics**, naming the pair (or the person), the place, the distance and this
   decision: a resolver returns a `Resolution` and has no error path, and `require_registered` already
   chose a panic for host defects. The violation is caught at the next arrival into that place, not at
   the bypassing fact itself.
3. **An arrival from another place, or from nowhere, is placed** (QP-7). A resolver cannot refuse an
   arrival or end it in another place (rule (a)), so an arrival into a place that has geometry always
   ends in that place: at `to` if a person fits there; else at `to` with the people in the way nudged,
   under the bounds of point 1; else at the nearest free point of a 50 mm lattice, with nobody moved and
   `stopped-short` recorded. Bodies' genesis check guarantees such a point exists: a place with geometry
   must hold, on a 650 mm sub-lattice, at least `4 × (people in the world − 1) + 1` free points. Rule (c)
   does not bound such an arrival (the limitation above), so the point may be farther from the doorway
   than `to` is.

**Note 2, 2026-10-08 (S15, PR 12c; step-11 §18.0, SD-O8 … SD-O10, SD-O15, QO-4, QO-9, QO-18) —
loose objects, and a pack that is also a mover.** Items 1–8 and the first note are unchanged except
where this note refines item 7 for objects.

1. **The resolver predicts object pushes.** Loose objects are `bodies`' own state (an Item with a
   `body:` section, `ARC-36` note), so presence cannot record their moves and a resolver cannot emit
   them: they move in `bodies`' reaction to each recorded `arrived`. That reaction pushes every loose
   object lying on the floor whose footprint the person's disc overlaps, straight away from the
   person's centre, by a shape cast against the fixed geometry and the other objects — computed
   **against the objects as they lay before the request**, with people not as obstacles. Reduction is
   breadth-first: every reaction of one emission list sees the objects as they lay, and the people
   not yet where the list leaves them (step-11 F-O2, F-B1). Before answering, the resolver runs the
   same function on the same three inputs — the person's end point, the room, the objects as they lay
   — for the walker and for every person it displaces, in emission order, and accepts the result only
   if no object is pushed twice, no push is cut short (a jam), and in the final state no person
   overlaps an object and no two objects overlap. Otherwise it resolves the arrival again with the
   objects solid, and nothing is pushed. So what the reactions push is exactly what the resolver
   checked, and a walker never ends inside a jammed box.
2. **The guard covers objects.** The starting-state guard (the first note, point 2) also checks that
   every object lies within its floor, rests on the floor or on one solid's top, keeps clear of every
   other solid and object, and is overlapped by no person's disc. And because a push is computed from
   the arrival and the objects alone, item 7's third bullet can be built for objects at the reaction
   itself: a push that jams there means an arrival escaped resolution, and the reaction fails the
   dispatch with `bodies-object-jammed`.
3. **`bodies` is also a mover.** Its `shove` action states presence's `arrived` and `stopped-short`
   through `arrivals()`, exactly as `movement` does (`ARC-26`): the shoved person's arrival is resolved
   by the registered resolvers like any other — walls stop them, the people behind them are nudged
   within the first note's bounds, objects are pushed or block, and the head-on bias applies to it as
   to every arrival within a place. A pack may subscribe to a fact type it also states (step-11 F-R7).
4. **One crate dependency outside presence, used for one read.** `bodies` refuses, at genesis, an
   Item that is both an object and a declared kind (`ARC-36` note, point 2). It asks the `item` pack's
   read-only `mineworld_item::is_declared`, and nothing else from that crate; a structural test holds
   that. This is a crate dependency, not a system dependency: `bodies` declares a system dependency on
   presence alone, a world may install `bodies` without `item`, and in such a world the answer is "not
   declared".

**Note 3, 2026-10-08 (S15, PR 12d-0; step-11 §20, SD-Z2, SD-Z5, SD-Z6) — bodies' cost, and one result
changed by design.** Items 1–8 and the earlier notes are unchanged except as follows.

1. **A stride away from a person within the controller's offset is not stopped by them (SD-Z5,
   FU-12c-1).** A person whose centre lies within two radii and the gap (610 mm) of the mover's start
   and on the far side of it from the stride (d · (p − start) ≤ 0) is left out of the mover's contact
   sweep. Rapier's character controller otherwise sticks on them: a shove from 600 mm moved its target
   301 mm instead of half a metre (§18.11 DO-11). Verification still counts every person, so the result
   never overlaps anybody; a stride *toward* such a person is still stopped by them. This changes
   results, so `bodies` is version 3 and refuses a version-2 save by name (`ARC-25`).
2. **The nearest free entry point is searched outward (SD-Z6).** An entry placed at the nearest free
   point of the 50 mm lattice (item E3) finds it by visiting the lattice in rings about the asked
   point, stopping once no unvisited point can be nearer, instead of filtering the whole floor: the
   same point, under the same order (distance, then y, then x), at a fraction of the cost. It was half
   of a town's with-bodies CPU (step-11 E-Z3).
3. **What was tried and not adopted, recorded so it is not tried again unmeasured.** A scene holding
   only what a stride can reach (SD-Z1), an exact integer corridor (SD-Z3) and integer wall strides
   (SD-Z4) each change results, because Rapier's controller is neither local — its answer moves when
   colliders far from the stride are removed — nor exact at its own offset from a face (step-11 E-Z1,
   E-Z2, E-Z5); none bought measurable CPU. They were dropped.

**Note 4, 2026-10-08 (S17, PR IL-a; `ARC-62`, QPL-10) — the installed set's line has a new spelling.**
The `resolution:` line of `installed!` and `Capability::resolvers()` are replaced by `ARC-62`'s generic
extension line, `extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers:
[mineworld_bodies::BodiesSystem,];`, and `worldpack::compose` registers it through
`Capability::register_extensions()`. No rule of this decision changes: the catalog, its write-once
storage, `require_registered`, the order resolvers are asked in and every fact are as items 1–8 and the
notes above state. Where items above say "the `resolution:` line", read "presence's extension line".

**Note 5, 2026-10-09 (S15, PR 12n-1; step-11 §21, SD-N3, SD-N4, SD-N11) — bodies is also a
wayfinder.** Items 1–8 and the earlier notes are unchanged.

1. **`bodies` answers route queries.** It implements `movement`'s `Wayfinder` (`ARC-75`), listed on
   movement's own extension line of the installed set, and its `install` calls movement's
   `require_wayfinder` after presence's `require_registered`. (Movement's guard is not named
   `require_registered`: movement's sources name nothing of the resolver seam, `seam_vocabulary.rs`.)
   A route is a plan, never a permission: every stride of a walk
   is still a `walk-step` request checked by movement and resolved by this decision's resolvers. The
   planner reads `bodies`' own state (`PlaceShape`, `LooseObjects`) and nothing else, keeps nothing, and
   answers "not mine" for a place without a shape, so a world without `bodies` walks straight.
2. **The crate dependency `bodies → movement`.** Implementing movement's trait needs movement's crate;
   it is a crate dependency, not a system dependency (`bodies` still declares presence alone), like the
   `item` read of note 2, point 4.
3. **Two person-relative bounds are expressions of the radius** (step-11 §21.6, TD-D8): `NUDGE_MAX` is
   `PERSON_RADIUS` and `BIAS_BAND` is ⌊2 · `PERSON_RADIUS` / 3⌋. At the radius of 300 mm both are the
   values the first note and 12b fixed (300 mm, 200 mm), so no result changes and `bodies`' version does
   not; a different radius scales them with the body.

**Note 6, 2026-10-10 (S15, PR 12n-2; step-11 §21.15 M-3, the operator's ruling) — a walk out of the
margin.** The resolver may leave a person nearer a wall or a solid than the planner's margin
(R + GAP + PLAN_MARGIN), though never nearer than R + GAP. From such a start the planner first steps
to the nearest free point of the 50 mm lattice, the goal's own snap (SD-N5), when that point lies
within the margin's width (360 mm at R 300). The route then goes on from there, and the step is a
stride the resolver resolves like any other. A start with no free point that near keeps the core rule
(step-11 N-D7): the box holding it is shrunk to the start for the start's own edges. Before this, a person pinned between the café's front wall and the bench's corner had no route
out, and every walk they asked for was refused `no-route` (step-11 E-NW4). This changes routes, and so
results, in every world with bodies and walkers, so `bodies` is version 4 and refuses a version-3 save
by name (`ARC-25`).
