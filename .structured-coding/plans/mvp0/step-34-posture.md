# Step 34 — S34: posture — sitting and standing, owned by the server

**Role:** step document for **S34**, the `posture` System Pack step that S33's ruling Q-S33-6 made a
required, release-blocking follow-up ([`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §14.2;
[`overall.md`](overall.md), "Required follow-up before release (2026-10-15, S33 Q-S33-6)"). The document
contains:

- the binding requirement;
- an audit of bodies, presence and movement, the interaction seams, both clients' animation, S33's sit
  points and S26's pose tool;
- the ontology: what a sit point is, which System owns posture, and which component holds it;
- the actions, the refusals and the events;
- the effect on bodies and movement;
- how humans, NPC controllers and MCP agents sit;
- how the 2D and the 3D client present it, and how S33's shared sit rule yields;
- persistence and determinism;
- invariants, step acceptance with adversarial criteria, the PR split with dependencies, risks, records
  and the operator questions.

**Effort:** `mvp0`, **release-blocking** (operator principle of 2026-10-15, "我们要发布就得把这些功能全部做好",
applied to this step by Q-S33-6). · **Parent:** [`overall.md`](overall.md).

## DESIGN FROZEN — 2026-10-18 (operator approved in the primary session: recommended answers; Q4 auto-stand, Q5 not pushable)

**Lifecycle:** `DESIGN FROZEN 2026-10-18` (step level). The operator ruled in the primary session on
2026-10-18, relayed by the coordinator: Q-S34-4 auto-stand; Q-S34-5 a seated person cannot be pushed, a bodies
change in S34-b; every other question frozen with its recommended answer (§16, column "Ruling").

- The freeze fixes scope (§3), invariants (§12) and step acceptance (§13). Progress, evidence, findings and
  bounded corrections stay writable.
- Each PR of §14 is detailed to the commit in its own PR design. Under this ruling the coordinator directed
  that S34-a's PR design, written to follow this frozen step, is recorded as frozen.
- Spend: none authorized.

Superseded lifecycle: `STEP DESIGN (DRAFT) — awaiting the operator's rulings on §16` (PR #264, first commit).

**Author:** S34 design session (Opus), 2026-10-18.

- Worktree: `/Users/yuema137/mineworld-worktrees/design-s34-posture`.
- Branch: `docs/s34-posture`.
- Audit base: `origin/main @ b85f2804` (#260). Unmerged branches read for the audit, never checked out:
  `origin/mvp0/pr-12d-towns` (#171, 12d), `origin/mvp0/pr-12e` (#254, 12e), `origin/mvp0/pr-16c` (#245),
  `origin/mvp0/pr-16d` (#250), `origin/docs/decisions-split` (#258), `origin/feat/mcp-single-act` (#261),
  and the S33 PR designs on `origin/docs/pr-designs-s33abc`.

**Ids.** `ARC-147` is the record reserved for this step (S33 §15). It is used for the whole posture
design (§17). No `DEP-` id is needed unless Q-S34-12 is ruled toward an adopted clip library; then the
primary session assigns one. Local ids in this file:

| Prefix | Meaning |
| --- | --- |
| `A-n` | audit anchors |
| `F-S34-n` | findings |
| `SD-S34-n` | step decisions (proposed; fixed by approval) |
| `I-S34-n` | invariants |
| `AC-S34-n` | step acceptance |
| `X-S34-n` | planted mutations |
| `R-S34-n` | risks |
| `Q-S34-n` | operator questions |

**Parallel-safety.** This session writes only this file. Edits to `overall.md`, `step-33-lakeside-3d.md`,
`docs/CORE_CONCEPTS.md`, `docs/MODULE_SPEC.md`, `docs/DECISIONS.md` and the generated reference are
**proposed** in §18 and made by the PRs named there, or by the primary session at approval.

---

## 0. Binding inputs

### 0.1 The requirement (verbatim)

From [`overall.md`](overall.md), 2026-10-15:

> A `posture` System Pack step — sitting and standing owned by the server, with `sit`/`stand` for humans,
> NPCs and agents, disclosed to every observer, in every world — is a **required, release-blocking** step,
> not optional. Its record id is `ARC-147` (reserved).

From [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §14.2:

> **What it must deliver** (the requirement, not the design): a `posture` System that owns whether a
> person is standing or seated at a seat, with a `sit`/`stand` action that a human, an NPC controller and
> an MCP agent all use; seat exclusivity decided by the server; posture disclosed to every observer, so
> both clients and the agent view show it from the world rather than from a client rule; all worlds (the
> café, the station halls, Lakeside).
>
> **What S33 guarantees for it:** the shared sit rule yields to a disclosed posture (§4.5); sit points are
> pack data, which the System's seats can be bound to by tag; no S33 code needs to change when it lands
> except the rule's override branch.

The requirement as numbered here:

```text
R-S34-1  The server owns whether a person stands or sits, and where.
R-S34-2  `sit` and `stand` are actions; a human player, an NPC controller and an MCP agent use the same ones.
R-S34-3  Seat exclusivity is decided by the server.
R-S34-4  Posture is disclosed to every observer: both clients and the agent view show it from the world.
R-S34-5  Every world: the café, the station halls, Lakeside (and Market Town, which shares the café's places).
R-S34-6  Release-blocking.
```

### 0.2 Standing rules that bind this step

- `CLAUDE.md` §4 rule 14 and `ENGINEERING_RULES.md` §§5–6: semantic space is not render geometry. A sit
  point's server meaning is a place-local position and facing; its seat height, mesh and pelvis offset are
  presentation.
- `CLAUDE.md` §4 rule 15 and `ENGINEERING_RULES.md` §§4, 7–9: `sit` declares its spatial requirement in
  the System contract. Clients report intent only; the server answers with the resolved action or with
  `Unavailable`, `Busy`, `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction` (and the kernel's
  `TargetUnavailable`, `PreconditionFailed`).
- `ENGINEERING_RULES.md` §7 names `sit` explicitly as a spatially grounded action; `CORE_CONCEPTS.md`
  §6.3 names it as one that declares what it needs; `MODULE_SPEC.md` §6 (l. 2167–2170, 2443–2444) says
  "`Sit(actor, bench)` belongs to a System" and "an interaction anchor says where a `Sit` is *rendered*,
  never whether sitting is *allowed*".
- The two gating questions (`ENGINEERING_RULES.md` §§11–12, §22): §11 answers both.
- Single ownership (`ENGINEERING_STANDARDS.md`; `INV-7`): presence alone owns `Location`; bodies owns
  place geometry; movement owns walking. Posture owns posture and sit points and nothing else.
- Realistic defaults with cited references (operator memory "Realistic defaults").
- All three platforms (macOS, Linux, Windows) in design and CI.
- Agents are first-class players: an outside agent sits through the same server path and the same rules.
- NPCs are held to the protagonist's standard: an NPC sits exactly as the player does.

---

## 1. Audit (`origin/main @ b85f2804` unless a branch is named)

| Id | What exists | Where |
| --- | --- | --- |
| A-1 | **No posture anywhere in the simulation.** No component, action, event or anchor for sitting. `git grep -i -w "sit\|seat\|posture"` in `systems/`, `contracts/`, `kernel/` finds none in this sense | `systems/`; S33 A-10, F-S33-8 |
| A-2 | **Location is presence's alone.** `Presence { location: Location }` (`systems/presence/src/component.rs:26`, type `presence`); `Location { place, local: Option<LocalPosition>, facing: Option<Orientation> }` in integer mm and millidegrees (`contracts/src/spatial.rs:274`, `Orientation` l. 195). Facts `Arrived` (`systems/presence/src/event.rs:32`), `PersonEnteredPlace` (l. 217), `StoppedShort` (l. 256); constructors `arrival()` (l. 98), `arrivals()` (l. 135), `admit()` (l. 192), which other packs use to state presence's facts (bodies states `Arrived` and `StoppedShort` for `shove`) | `systems/presence/src/` |
| A-3 | **The arrival seam (`ARC-39`).** `trait ArrivalResolver` (`systems/presence/src/resolve.rs:157`): pure, asked in ascending `SystemId` order, may stop an arrival short or move other people within the place; presence refuses an answer that moves anybody it may not. Bodies' implementation (`systems/bodies/src/resolve.rs:153`) passes an arrival through untouched when an earlier resolver changed it (SD-B15) and otherwise treats **every** person present as a disc it may nudge by up to `NUDGE_MAX` (= `PERSON_RADIUS`) with chains of at most `CHAIN_MAX` 2, else stops the walker short | `systems/presence/src/resolve.rs`; `systems/bodies/src/resolve.rs`, `geometry.rs:13-37` |
| A-4 | **Bodies.** `BodiesSystem` VERSION 4 (`systems/bodies/src/system.rs:38-125`): provides `kick`, `throw`, `shove`; owns `PlaceShape` (a `Floor` and at most 64 `Solid` boxes, `component.rs:124`), `BodyShape` (items only, l. 283), `LooseObjects` (l. 394). A person has **no body component**; size is the constant `PERSON_RADIUS` (300 mm on main, **250 mm on 12d**), `GAP` 10 mm, `TOLERANCE` 5 mm. Furniture is solids: on 12d the café's window bench seat slab is `{min: {x: 2360, y: 460}, max: {x: 8020, y: 900}, height: 470}` (`worlds/social-cafe/places/cafe.yaml` on `origin/mvp0/pr-12d-towns`). Nothing marks a person immovable | `systems/bodies/src/` |
| A-5 | **12d (#171, unmerged)**: `PERSON_RADIUS` 300 → 250 mm, `CLEARANCE = 2R − TOLERANCE`; `body:` sections for every place of Market Town and Social Café; loose-object items. **12e (#254, unmerged)**: bodies in the 3D client — `client_bodies.gd`, `correction.gd`, `figures.gd`; the client corrects to the server's position beyond 150 mm (`ARC-50` note) | `origin/mvp0/pr-12d-towns`; `origin/mvp0/pr-12e` |
| A-6 | **Movement.** VERSION 2; owns `Passages` and `Walking`; provides `move{to}`, `walk-to{to: Destination}`, `walk-step{}` (`systems/movement/src/action.rs:38, 131, 158`). `Destination = Place(Location) \| Person(PersonId)`, `#[non_exhaustive]` (l. 117). `Walking` is persisted and advanced only by the walker's own `walk-step` (`ARC-75`). `pub fn is_walking()` (`systems/movement/src/system.rs:350`). Movement never answers `Busy` | `systems/movement/src/` |
| A-7 | **One provider per action type.** Posture cannot provide or veto `move` / `walk-to`; it can only react to presence's facts | kernel dispatch; A-6 |
| A-8 | **Refusals.** `pub enum Rejection` (`contracts/src/action.rs:215`): `Busy`, `TooFarAway`, `PermissionDenied`, `NoSupportedInteraction`, `TargetUnavailable`, `Unavailable`, `PreconditionFailed`, `System{code, detail}`. `ActionResult::Unavailable` is separate: no enabled system provides the action (`INV-10`; `CORE_CONCEPTS.md` §12.1) | `contracts/src/action.rs` |
| A-9 | **Busy precedent.** The only producer of `Rejection::Busy` is group-activity, when the person already holds `Participation` (`systems/group-activity/src/system.rs:166, 174`; `component.rs:143`). No shared activity lock exists | `systems/group-activity/` |
| A-10 | **Offers and disclosure.** `PerceptionProvider` (`systems/presence/src/interaction.rs:80`): `offers(world, observer, target)` and `discloses(world, observer, subject)`. An `Offer` carries action type, `SpatialRequirement`, target availability, an optional **complete** payload (`Offer::complete`, `ARC-34`) and an optional pack refusal (`Offer::refused`), reported before the requirement. `target` may be `None` for an action directed at nobody. A record is disclosed only when an enabled system owns its type (`observe.rs:196`, fail-closed) | `systems/presence/src/interaction.rs`, `observe.rs` |
| A-11 | **Interaction List (`ARC-63`).** A pack's `InteractionSection` gives a world `rules` (`permits` → `PermissionDenied`), bounded `parameters` and `consequences` (audience narrowing). Absent section = the pack's reference behaviour, byte-identical | `step-18-interaction-list.md` §§4.4–4.7 |
| A-12 | **Authored sections (`ARC-31`).** A pack owns one section of a person or place file, seeded at genesis (bodies' `body:` → `place-shaped`). Packs register with one line in `systems/installed/src/lib.rs` (`installed!{}`), extension catalogs with `extension …` lines (`ARC-33`, `ARC-62`) | `systems/installed/src/lib.rs`; `systems/bodies/src/genesis.rs` |
| A-13 | **No anchor type.** The only named points in a place are passages' `here`/`there` | `worlds/*/places/*.yaml` |
| A-14 | **"Seat" is taken.** It means a Person a client connects as: `seats:` in `world.yaml` (`MODULE_SPEC.md` l. 701), seat grants (`NETWORKING.md` l. 222, `ARC-97`), `Seated` in the agent library's models (the argument base class naming the agent's seat, `clients/agent/src/mineworld_agent/models.py:686`) | as named |
| A-15 | **"posture" is used twice.** S30-e's built-in attribute `posture` (`upright, relaxed, slight-stoop, stooped`, an appearance trait; `pr-s30e-builtin-catalogue.md` l. 616) and the 3D client's `Posture` SkeletonModifier3D class (`clients/3d-spike/scripts/posture.gd`, `human.gd:71`) | as named |
| A-16 | **Rule controller attempts complete offers.** The paced rule attempts an available complete affordance with probability `ATTEMPTS_OFFERED` 20/100 on draw 14, choosing on draw 15; constants frozen, "the remedy … is in what its packs offer" (`cognition/rule-controller/src/offered.rs`). The motivated model (DL-b) builds options from available affordances (`cognition/motivated/src/options.rs`) | `cognition/` |
| A-17 | **MCP agents.** `attempt` submits offer *n* of view *v* (`AttemptArgs`, `models.py:686`); #261 folds the act tools into one `act` tool whose `attempt` kind is unchanged. Disclosed components are mapped per pack in `clients/agent/src/mineworld_agent/disclosed.py` (e.g. `participation` → `MeLine.doing`, `PersonLine.doing`) | `clients/agent/` |
| A-18 | **3D client clips.** `scripts/human.gd`: Quaternius UAL library trimmed to `Idle`, `Walk`, `Jog_Fwd`, `A_TPose` (`ASSETS.md:191-194`); built-in clips `Sit` (one-key looping pose, `_sit_clip()` l. 456; Quaternius' `Sitting_Enter/Idle/Exit` are paid-tier, l. 446-450), `Stand`, `StandGrip` (`_stand_clip()` l. 607). AnimationTree (`_build_tree()` l. 362-443): `Locomotion` BlendSpace1D → `Rate` → `Mode` transition `move`/`sit`, `xfade_time = 0.0`. `sit(seat_y)` (l. 705) puts the Hips joint at `seat_y` (`_settle_seat` / `_place_on_seat`, l. 756-785). **There is no `stand()`**: sitting is one-way | `clients/3d-spike/scripts/human.gd` |
| A-19 | **3D server-driven people.** `npc.gd` `enum Pose { WALK, STAND, SIT, LEAN, PUPPET }` (l. 37); `SIT` only for scenery (`slice/streetscape.gd:354`, `town.gd:611, 618`). On main `slice_link.gd` snaps figures (l. 366-367) and plays `Stand` only. 16c/16d/12e add `slice/figures.gd`: gait from local displacement (`_walk()` l. 135/136), yaw `lerp_angle` on arrival, `PLACE_GAP` 4.0 m snap. No branch reads any sit state | `clients/3d-spike/scripts/` |
| A-20 | **2D client.** `clients/2d/scripts/scene/people.gd`: people from observed `location.local`, `lerp` follow (`FOLLOW_RATE` 9.0, snap past `SNAP_M` 4.0), four directional views (`front`, `front_b`, `back`, `back_b`), two stride frames (`STRIDE_M` 0.72). **No seated frame**; `facing` is not read | `clients/2d/scripts/` |
| A-21 | **S26 pose tool (merged).** `tools/character_pipeline/godot/pose_sheet.gd`, `mwcast/pose_sheet.py`, `PIPELINE.md` §15: a QA pose sheet (PNG frames, `report.json`, `sheet.html`) including **P-4 seated** (three-quarter and side), rendered with the client's own `human.gd` `Sit`. **Finding** (`PIPELINE.md:447-449`; `pose_sheet.gd:44`): `sit(seat_y)` with Hips at `seat_y` leaves the lowest buttock vertex 127 mm under the seat; the sheet uses hips-over-seat 0.250 m instead. The client was not fixed. Gates: G-8 (the `Sit` clip plays), G-13 (seat contact ± 20 mm; `step-26-cast.md` l. 705, 710); rig: the 52 `mixamorig:*` bones (`HUMANOID_PROFILE.md`) | `tools/character_pipeline/` |
| A-22 | **Appearance.** Discloses only an opaque `look`; `character.yaml` declares clip speeds, not a clip list; no sit capability flag | `step-26-cast.md` §§4.2, 4.5 |
| A-23 | **S33's sit points (`ARC-145`, S33-f, not implemented).** "A sit point is pack data: `{position, facing, seat_height_m, approach}` on a named seat" (presentation). The shared rule SD-S33-7 in `clients/shared/`: drawn seated iff within `approach` (≤ 0.40 m of the seat's front edge), still ≥ 2 s of world time, not already drawn occupied (ties by entity id). "A future `posture` System that discloses sitting overrides the rule". AC-S33-3 seat contact ± 20 mm; AC-S33-11 "sit on the pier bench"; `--seat` probe; I-S33-5 "no seated pose without a person at the sit point". `clients/shared/` holds no sit rule yet | `step-33-lakeside-3d.md` §4.5, §9.3, §11, §14.2 |
| A-24 | **Station halls (S29).** Long oak benches in rows, seat at 0.45 m (`step-29-station-landmark.md` l. 376, 401); the bench rows are 12d `body:` solids (l. 464) | `step-29-station-landmark.md` |
| A-25 | **Seat heights.** `VISUAL_SLICE.md` §3.4: 0.42–0.47 m | `docs/VISUAL_SLICE.md` |
| A-26 | **Persistence.** A world is its journal re-executed from the newest snapshot (`ARC-25`); snapshot rows are canonical JSON; replay must reproduce logged facts byte for byte (`PersistError::ReplayDiverged`, `persistence/src/replay.rs:87-109`). No floats, no `HashMap`/`HashSet` (`clippy.toml`); a change to results bumps a pack's `VERSION` | `persistence/`; `ARC-25` |
| A-27 | **Decision log layout.** #258 (open) splits `docs/DECISIONS.md` into `docs/decisions/<ID>.md`; ids in flight reach `ARC-153` (#261) | `origin/docs/decisions-split` |

## 2. Findings

- **F-S34-1. A seated person cannot be protected by a resolver posture owns.** Resolvers run in ascending
  id order (`bodies` < `posture`), bodies skips when an earlier resolver changed the arrival, and a later
  resolver cannot undo bodies' nudge (A-3). Protecting a seated person needs a seam **in bodies** (§7.1).
- **F-S34-2. A seat surface lies inside a solid.** The café's window bench seat slab is a solid (A-4); a
  person's position inside it is refused by bodies. So the server's position of a seated person cannot be
  the pelvis over the seat. It must be a point on free floor (§5.1, SD-S34-3).
- **F-S34-3. Posture cannot veto walking.** One provider per action (A-7). Standing up when a seated
  person walks away is a **reaction** to presence's facts, not a refusal of `move` (§6.4).
- **F-S34-4. NPCs can sit with no controller change.** The rule controller and the motivated model already
  attempt available complete affordances (A-16). If `sit` is offered as complete offers, NPCs sit through
  the same path as everyone else.
- **F-S34-5. MCP agents can sit with no adapter code for the act**, through `attempt` (A-17). Only the
  disclosed-component table needs a `posture` row so the view says who is seated.
- **F-S34-6. The term "seat" cannot be used for furniture** (A-14), and "posture" collides with S30-e's
  attribute and a 3D-client class (A-15).
- **F-S34-7. The 3D client cannot stand a person up** (A-18) and sits them 127 mm too low (A-21).
- **F-S34-8. The 2D client has no seated frame** (A-20).
- **F-S34-9. S33-f and S34 touch the same seam.** If S33-f lands first, it authors presentation-only sit
  points and the shared rule; S34 then re-authors the positions as world data. If S34-a lands first, S33-f
  authors server sit points once and only their depictions in presentation (Q-S34-9).

---

## 3. Goals and non-goals

**Goals (MVP-0, release-blocking):**

1. A `posture` System Pack that owns sit points and each person's posture.
2. `sit` and `stand` actions, offered as complete offers, used identically by a human in the 2D client, a
   human in the 3D client, an NPC controller and an MCP agent.
3. Server-decided exclusivity: one person per sit point, always.
4. Posture disclosed to every observer who perceives the person; sit points disclosed with their place.
5. A seated person is not nudged aside by walkers (bodies seam), and walking away stands them up.
6. Sit points authored in every shipped world: Social Café, Market Town, Lakeside, and the station halls.
7. Both reference clients draw posture from disclosure: the 3D client sits and stands with seat contact
   within ± 20 mm; the 2D client draws a seated figure. S33's shared rule becomes the fallback for worlds
   without the pack.
8. Persistence and determinism: seated people stay seated across save, SIGKILL and resume, byte for byte on
   arm64 and x86_64; worlds without the pack are byte-identical.

**Non-goals:**

- Lying (beds), kneeling, crouching, leaning as server facts. The type leaves room (SD-S34-5); none is
  built in MVP-0.
- Sitting as a Process with duration, fatigue, comfort or needs. Sitting is a state, not a Process.
- Sitting on the floor or anywhere that is not an authored sit point.
- Seat ownership, reservation, paid seating, or "this is my chair" rules beyond what a world's Interaction
  List can forbid by class (`PermissionDenied`).
- Vehicles (the train, a rowing boat). A boat seat is a later step's concern (non-preclusion only).
- Server-side compound "walk to and sit". Clients and controllers chain `walk-to` then `sit` (§9).
- A height or a mesh in the simulation. Seat height stays presentation (S33 F-S33-4).

---

## 4. Reuse, both halves (`REUSE_POLICY.md`; `CLAUDE.md` §4 rule 16)

| Concern | Candidates | Recommendation |
| --- | --- | --- |
| Posture state, occupancy, refusals | No library models server-authoritative seat occupancy over MineWorld's contracts; game engines (Godot, Unity, Unreal) keep "sit" as client animation state with local interaction volumes, which is exactly the renderer-side rule `ENGINEERING_RULES.md` §8 forbids | **Build ours** inside a System Pack (~a few hundred lines of domain logic). Not a substantial infrastructure component; no `DEP-` record needed |
| Exclusivity and contention | ECS crates (bevy_ecs, hecs) offer no occupancy semantics; the kernel's sequential dispatch already serializes requests deterministically | **Reuse the kernel's dispatch order**; no locking |
| Spatial checks | The kernel's one `SpatialRequirement` evaluation (`CORE_CONCEPTS.md` §6.3) | **Reuse** for `SamePlaceAsActor`; the distance to a sit point is posture's own check, because a sit point is not an entity (SD-S34-2) |
| 3D animation | Godot `AnimationTree` (in use); Quaternius UAL sitting clips (paid tier, `human.gd` l. 446-450); Mixamo sitting clips (the rig uses `mixamorig:*` names, so they would retarget; terms must be read at source before any adoption); CC0 motion-capture libraries | **Keep the procedural `Sit` and add procedural sit-down / stand-up transitions** in `human.gd`; decide clip adoption at S34-f's freeze after reading each licence at source (Q-S34-12). Nothing is bought (Q-S33-7(c) precedent) |
| 2D seated sprites | S32's 2D route renders layered sprites from the 3D characters (`step-32-wardrobe.md` §11.2) | **Reuse that route** with one more view, `seated` (Q-S34-10) |

Forcing a wheel where it does not fit: adopting an engine's "seat interaction" component would put the
exclusivity rule in the renderer. Rejected for that reason.

---

## 5. Ontology

### 5.1 Sit point (semantic; owned by `posture`) — SD-S34-1…4

A **sit point** is a place-local, authored, named position at which one person may sit:

```text
SitPoint
    key       SitPointKey      slug, unique within its place (e.g. "window-bench.3", "pier-bench.1")
    at        LocalPosition    x, y in mm in the place's frame; z = 0. The seated person's position:
                               the centre of their footprint on FREE FLOOR at the seat's front edge
    facing    Orientation      yaw in millidegrees: the direction a seated person faces
    group     SitGroupKey?     optional: the furniture the point belongs to ("window-bench"), so that a
                               depiction and a word ("the window bench") can name it once
```

- **SD-S34-1. Not an entity.** A sit point is data of its place, held in a component on the Place, not an
  entity. Reasons: Items in MVP-0 are kinds with counts (`ARC-36`), so a bench cannot be an item
  instance; a System may not create entities at run time (QPL-11); a bench is geometry, which bodies
  already holds as solids; and nothing about a sit point needs identity across places. (Q-S34-2.)
- **SD-S34-2. Referenced by key, within the actor's place.** An action names `point: SitPointKey`; the
  place is the actor's own. The distance check is posture's (a sit point has no `Presence`, so the
  kernel's range evaluation cannot measure to it); posture answers `TooFarAway` itself, in its offers and
  in `validate`, in the same order.
- **SD-S34-3. `at` is on free floor (F-S34-2).** The seated person's authoritative position is where their
  disc stands in bodies' model: in front of the seat surface, clear of every solid by `PERSON_RADIUS + GAP`
  (260 mm on 12d), exactly like a doorway point. The **pelvis over the seat** is presentation: the
  depiction places it at `at` moved back along `facing` by the depiction's `seat_depth_m` and up to its
  `seat_height_m`. So the server never learns a height (S33 F-S33-4), bodies needs no exception for the
  seat slab, and a seated person's knees and feet occupy the floor in front of the bench, as they do.
  (Q-S34-3.)
- **SD-S34-4. Spacing.** Two sit points of one place stand at least `2·PERSON_RADIUS + GAP` apart (510 mm
  on 12d) centre to centre, so two seated people never overlap. The authoring default for benches is
  600 mm per person; S34-c cites a published seating-allowance source for the number at its freeze
  (operator memory "Realistic defaults").

**Authored** as a place file's `sit_points:` section (`ARC-31`), owned by `posture`:

```yaml
# worlds/social-cafe/places/cafe.yaml (illustration; S34-c authors the real values against 12d's solids)
sit_points:
  - { key: window-bench.1, group: window-bench, at: { x: 2700, y: 1160 }, facing: 180000 }
  - { key: window-bench.2, group: window-bench, at: { x: 3300, y: 1160 }, facing: 180000 }
```

The window bench's slab ends at `y` 900, so `y` 1160 is 260 mm clear of it; `facing` 180 000 faces south,
toward the window (`CORE_CONCEPTS.md` §6.1 frame).

**Stated** at genesis as the posture-owned fact `place-sit-points` and reduced into the Place's component
`SitPoints` (type `sit-points`, schema 1), sorted by key. Its type is its validation, as `PlaceShape`'s is:
a duplicate key, more than 64 points per place, a coordinate beyond ± 100 000 mm, or two points closer
than SD-S34-4's spacing cannot be constructed. A point inside a solid is refused at load through bodies'
seam (§7.1).

### 5.2 Posture (on a Person; owned by `posture`) — SD-S34-5

```text
Posture                                   component type "posture-state", schema 1, on a Person
    Seated { point: SitPointKey, since: WorldTime }
```

- **Standing is the absence of the row.** A person with no `posture-state` row stands. That keeps every
  person who never sits free of a row, and every world without the pack byte-identical (I-S34-6).
- **The place is not stored twice.** A seated person's place is their `Presence` place; the row names only
  the key. Invariant I-S34-2 ties them.
- **Room to grow.** The value is an enum. `Lying { point }` (beds) or `Kneeling` are later variants; adding
  one is a schema change of this pack, not of the kernel. MVP-0 has `Seated` only.
- **Names (F-S34-6).** The pack id stays `posture` (the operator's word, under which `ARC-147` is
  reserved). The component type is `posture-state`, not `posture`, so it never reads as S30-e's appearance
  attribute `posture`. The furniture term is **sit point**, never "seat". (Q-S34-1.)
- **Occupancy is derived, never stored.** Whether a sit point is taken is "some person in this place has
  `Seated { point }`". One source of truth; a scan of the people in one place (tens) per request.

### 5.3 What posture owns, and what it does not

| State | Owner | Posture's relation |
| --- | --- | --- |
| `SitPoints` on a Place, `posture-state` on a Person | **posture** | writes, only in its own reductions |
| `Presence` (location, facing) | presence | states `Arrived` through presence's constructor when sitting moves the person onto `at` (§6.2); reads |
| `PlaceShape`, solids | bodies | none; bodies asks posture through the seam (§7.1) |
| `Walking` | movement | reads `is_walking()` (§6.1, Q-S34-6) |
| `Participation` | group-activity | none: sitting and an activity are independent |

Dependency direction: `posture → presence, movement` (contracts and public queries); `posture` implements
bodies' new extension trait, so its extension line names `bodies` (§7.1). Nothing depends on `posture`.

---

## 6. Actions, refusals and events

### 6.1 `sit { point: SitPointKey }`

```text
SpatialRequirement   place: SamePlaceAsActor, within_range: none (posture measures), line of access: false,
                     requires_target_available: false   (target is None; availability is posture's refusal)
```

Validation, in order (the same order in the offer and at dispatch, `ARC-34`):

| # | Condition | Answer |
| --- | --- | --- |
| 0 | the world does not enable `posture` | `ActionResult::Unavailable` (kernel, `INV-10`) |
| 1 | the actor is not a Person, or has no `Presence` | `PreconditionFailed` |
| 2 | the Interaction List forbids `sit` for the actor's class, here (`permits`) | `PermissionDenied` |
| 3 | no sit point `point` in the actor's place | `NoSupportedInteraction` |
| 4 | the actor is already seated (anywhere) | `Busy` |
| 5 | the actor is walking (`movement::is_walking`) | `Busy` |
| 6 | the actor's position is farther than `reach` from `at` (default 750 mm, bounds 300–1 500, the section's parameter); or the place models no positions and the point is in another place | `TooFarAway` |
| 7 | another person is seated at `point` | `TargetUnavailable` |
| 8 | another standing person's disc overlaps `at` (closer than `2·PERSON_RADIUS + GAP` to `at`) | `TargetUnavailable` |

- **`reach` 750 mm.** A person standing with their disc touching the seated disc's place stands at
  `2R + GAP` = 510 mm (12d); 750 mm admits approaching from the side or slightly behind a chair, and stays
  under an arm's span from the seat front. S34-a cites an anthropometric source at its freeze.
- **Without positions.** A place that models no `local` positions (a 2D-only or headless world authored
  without them) cannot hold sit points, because `at` is required. `sit_points:` in such a place is refused
  at load. (The café, Market Town, Lakeside and the station halls all have positions.)

On acceptance posture emits, in this order:

1. presence's `Arrived` to `Location { place, local: at, facing }` through presence's constructor, unless
   the actor already stands exactly there facing that way;
2. its own `sat-down { person, place, point, at, facing }`, which reduces into `Seated { point, since: now }`.

The short move onto `at` (≤ `reach`) is the person settling into the seat. Row 8 guarantees nobody stands
on `at`, so bodies has nothing to nudge. Clients animate the settle (§8).

### 6.2 `stand {}`

| # | Condition | Answer |
| --- | --- | --- |
| 1 | not a Person / no `Presence` | `PreconditionFailed` |
| 2 | the actor is not seated | `PreconditionFailed` |

On acceptance: `stood-up { person, place, point, cause: Requested }`, which removes the row. **The location
does not change**: `at` is already on free floor (SD-S34-3), so the person stands where their feet were.
The Interaction List cannot forbid `stand` (a list cannot trap a person; the section declares `stand`
non-configurable).

### 6.3 Offers

- For an observer **standing** in a place with sit points: one **complete** `sit { point }` offer per sit
  point, in key order, `target: None`. An occupied, too-far or forbidden point is still offered, with
  posture's refusal (`Offer::refused`), so a client and an agent can see why (`ARC-34`). An available
  offer is what a controller may attempt.
- For a **seated** observer: one complete `stand {}` offer; no `sit` offers (they would all be `Busy`).
- Nothing is offered about other people: one does not sit "on" Alice.

### 6.4 Standing up because the person moved (F-S34-3) — SD-S34-6

Posture subscribes to presence's `Arrived`, `StoppedShort` and `PersonEnteredPlace`. In its reaction, for a
person with a `Seated` row:

```text
the person's new Location is not (their place, at of their point)  →  stood-up { …, cause: Moved }
```

So `move`, `walk-to` and `walk-step` from a seated person stand them up as a consequence, in the same
dispatch, with no change to movement. The first `walk-step` leaves `at`; the reaction emits `stood-up`
before any observer is served the next frame. A client that wants the stand to read cleanly sends `stand`
first (§9.1); a controller that does not still gets a consistent world. (Q-S34-4.)

A seated person is never displaced by someone else (§7.1), so `cause: Moved` is only ever the person's own
movement — or, if a world removes bodies' protection, a nudge, which stands them up honestly.

### 6.5 Events (all posture-owned)

| Fact | Payload | Audience (default) | Biographical | Notes |
| --- | --- | --- | --- | --- |
| `place-sit-points` | place, points | genesis | no | seeding fact, like `place-shaped` |
| `sat-down` | person, place, point, at, facing | `Place` | no | narrowest a list may choose: `Participants` (the sitter) |
| `stood-up` | person, place, point, cause `Requested \| Moved` | `Place` | no | as above |
| `posture-interactions-configured` | the resolved section | genesis | no | generated by `interactions!()` (`ARC-63`) |

Not biographical by default: sitting down is not a life event. A world's list may not make it one (the
owner marks it non-configurable), which keeps biographies free of chair noise (`ARC-29`).

### 6.6 The Interaction List section `interactions/posture.yaml`

```yaml
default: permit
rules:
  - { action: sit, actor: commoner, region: throne-room, effect: forbid }   # illustration only
parameters:
  - { reach: 750 }
```

Declared: actions `sit` (configurable), `stand` (not configurable); parameter `reach` (mm, 300–1 500,
default 750); facts `sat-down`, `stood-up` (audience narrowing only). Sit points themselves are not list
content: they are place data.

---

## 7. Bodies and movement

### 7.1 The seated are not nudged — bodies' `Anchoring` catalog (SD-S34-7; Q-S34-5)

Bodies opens one extension catalog (`ARC-62`'s mechanism; bodies already has `InteractionKind`):

```rust
/// Asked by bodies, in SystemId order; pure; inert where the implementer's state is absent.
pub trait Anchoring: Send + Sync {
    fn anchoring_of(&self) -> SystemId;
    /// Whether `person` is held where they are: never nudged, never shoved, a fixed disc to walkers.
    fn anchored(&self, world: &WorldRead<'_>, person: PersonId) -> bool;
    /// Fixed points of `place` that must be clear of every solid by a radius and a gap (checked at load).
    fn fixed_points(&self, world: &WorldRead<'_>, place: PlaceId) -> Vec<(String, LocalPosition)>;
}
```

`posture` implements it (`anchored` = has a `Seated` row; `fixed_points` = its sit points). Bodies:

- treats an anchored person as a disc it may **not** nudge: a walker who would nudge them is stopped short,
  exactly as when a nudge chain fails today;
- refuses `shove` of an anchored person with `TargetUnavailable`;
- refuses at load a fixed point closer than `PERSON_RADIUS + GAP` to a solid or outside the floor, naming
  the place and the key (`BODIES_FIXED_POINT_IN_SOLID`);
- with no implementer registered, or no anchored person, behaves byte-identically to today.

Why a catalog in bodies and not a resolver in posture: F-S34-1. Why not bodies reading posture's component:
that would make bodies depend on posture, inverting the dependency for one pack's benefit, and the next pack
that pins a person (a bed, a vehicle) would edit bodies again. The catalog is the change-amplification
answer: one extension line per future pack, no edit to bodies.

**Bodies' `VERSION`** bumps (4 → 5, or the next after 12d) because its results change when an implementer
is registered. Saves of worlds with bodies made before the bump are refused, as the rule requires; the
shipped worlds are re-baselined in S34-c.

Ball flights (`kick`, `throw`) treat people as they do today; an anchored person is an obstacle like any
other. No change.

### 7.2 Movement

No change to movement's code. Posture reads `is_walking()` (row 5) and reacts to presence's facts (§6.4).
`walk-to` with `Destination::Place(Location)` already reaches the approach of a sit point; a
`Destination::SitPoint` would make movement know posture, and is not added.

### 7.3 Group activities and conversation

Independent. A person may talk, join an activity or be invited while seated, and may sit while
participating. Neither pack reads posture; posture reads neither.

---

## 8. Presentation: one rule for 2D and 3D

### 8.1 The shared decision (in `clients/shared/`) — SD-S34-8

```text
place discloses `sit-points`                        (posture installed here)
    person drawn seated at point p   iff   the person's disclosed `posture-state` is Seated { point: p }
    otherwise drawn standing / walking
place discloses no `sit-points`                     (posture not installed)
    S33's SD-S33-7 rule, unchanged
```

- This is S33 §4.5's "override branch" and the only S33 code that changes (S33 §14.2's guarantee).
- Both clients call the one shared function; neither holds a distance, a stillness time or an occupancy
  test when the world discloses sit points. S14's rule-constant scan gains the check that no posture
  constant appears in `clients/2d` or `clients/3d-spike`.

### 8.2 Depictions and binding

A presentation sit point (S33's data) becomes the **depiction** of a semantic sit point, bound by
`{place tag, sit point key}`:

```yaml
# presentation/mineworld-default/3D/… (S33-f's file; illustration)
sit_point_depictions:
  - { place: cafe, point: window-bench.1, seat_height_m: 0.47, seat_depth_m: 0.48 }
```

- Position and facing come from the server; height and depth from the depiction; defaults
  `seat_height_m` 0.45, `seat_depth_m` 0.45 (`VISUAL_SLICE.md` §3.4; S34-f cites the depth source).
- A drawn-vs-disclosed check (like 12e's walls, X-8) fails when a depiction's drawn seat front differs
  from the disclosed `at` by more than 150 mm, or a disclosed point has no depiction.

### 8.3 3D (`clients/3d-spike`) — S34-f

- `human.gd` gains `stand()` (transition `Mode` back to `move`, re-enable the `Posture` modifier) and short
  procedural sit-down / stand-up transitions (target 0.6 s each, `xfade_time` > 0) — F-S34-7.
- `sit()` takes the pelvis target from the pose sheet's calibration (hips-over-seat 0.250 m, A-21), so
  G-13 / AC-S33-3 hold: seat contact within ± 20 mm.
- `figures.gd` (16c/12e) reads `posture-state`: on `Seated`, walk to the disclosed `at`, turn to `facing`,
  play sit-down, place the pelvis per §8.2; on its removal play stand-up, then locomotion resumes.
- **The player (Minecraft-like).** Looking at a sit point's depiction within the interaction ray and
  pressing **E** selects the complete `sit { point }` affordance whose key the depiction names and submits
  it; the result (accept, `TooFarAway`, `TargetUnavailable` …) is the server's. While seated, **E** or any
  movement key submits `stand {}` first, then movement. The camera stays first/third person as today;
  seated eye height is presentation.
- The 3D client's internal `Posture` class may keep its name: it is not a contract term. Renaming it to
  avoid confusion is S34-f's bounded choice.

### 8.4 2D (`clients/2d`) — S34-e

- A seated person is drawn at the disclosed `at`, in the view chosen from `facing` (the 2D client starts
  reading `facing` for seated people), with the stride animation off.
- The sprite is the **seated view** rendered from the person's 3D character through S32's 2D route (a fifth
  view, `seated`, front and back); until a person's seated view exists, the fallback is the standing view
  cropped at the hip line and lowered onto the depiction (Q-S34-10).
- The player clicks a sit point's depiction: if the complete `sit` affordance is available, it is
  submitted; if it is `TooFarAway`, the client sends `walk-to` the point's approach and, on `walk_ended`,
  the same `sit` affordance (the chain lives in `clients/shared/`, so 3D's "walk then E" and 2D's click
  resolve to the same two intents). Clicking oneself, or moving, while seated submits `stand {}`.

### 8.5 The agent view (`clients/agent`) — S34-d

- `disclosed.py` gains a row: `posture-state` | posture | anyone perceiving | `MeLine.seated`,
  `PersonLine.seated` ("seated on the window bench"), with the group key as the words.
- `sit` and `stand` reach agents as complete offers: no new tool, no new act kind (A-17, #261).

---

## 9. Who sits: humans, NPCs, agents (R-S34-2)

### 9.1 One path

```text
2D click on depiction ─┐
3D look + E ───────────┼──► complete affordance  sit {point}  ──► posture.validate ──► sat-down | refusal
rule / motivated NPC ──┤
MCP agent `attempt` ───┘
```

`AC-13`'s harness records the 2D and the 3D intent for the same sit point and asserts an identical semantic
core.

### 9.2 NPCs (F-S34-4) — S34-d

- The paced rule's offered band (A-16) attempts available `sit` offers with its frozen 20 % and walks away
  later, which auto-stands (§6.4). That already makes people sit in places they linger.
- The motivated model (DL-b) gains a consideration for `sit` options only if S34-d's seeded 30-day run
  shows sitting that does not read as living (people bobbing down and up within a minute). The remedy
  allowed by the rule controller's charter is in what the pack offers, not in retuned constants: if needed,
  posture offers `sit` as **available** only to a person who has stood still in the place for ≥ 60 s of world
  time (Q-S34-11). That threshold is pack-owned, server-side, and the same for players.
- The LM controller sees `sit` and `stand` among its offers like any other action.

### 9.3 MCP agents

An agent reads the view, sees `sit {point}` offers with their verdicts, `walk_to`s within reach when
needed, and `attempt`s the offer. Same server path, same refusals. S34-d's scenario test drives this with a
recorded (deterministic) agent, never a live model.

---

## 10. Persistence, determinism, versions

- **State:** `SitPoints` (per place) and `posture-state` (per seated person) are ordinary component rows;
  snapshots carry them as canonical JSON; replay re-derives them from `place-sit-points`, `sat-down`,
  `stood-up` (`ARC-25`).
- **No floats, no hash maps.** Positions in mm, yaw in mdeg; sit points sorted by key; occupancy by a
  scan in `PersonId` order.
- **Contention is deterministic.** Two requests for one point in one tick are dispatched in journal order:
  the first is `sat-down`, the second `TargetUnavailable`, identically on replay and on every platform.
- **Resume.** A person seated at SIGKILL is seated after resume, at the same point, with the same `since`.
- **Byte identity.** A world that does not enable `posture` replays byte-identically after every S34 PR
  (bodies' catalog empty; no rows; no offers). Enabling `posture` in the shipped worlds (S34-c) changes their
  genesis and is a recorded re-baseline.
- **Versions.** `posture` starts at VERSION 1. Bodies' VERSION bumps once (§7.1). Movement and presence do
  not change.
- **Configuration drift.** `interactions/posture.yaml` follows `ARC-61` / QPL-12: resume is refused on drift.

---

## 11. The two gating questions

**Can it support a Minecraft-like embodied 3D client without redesigning the kernel?** Yes. The player
walks up, looks at the bench, presses E; the client sends one intent; the server answers. Position, facing,
exclusivity and refusals are server facts; seat height, pelvis placement and the sit-down animation are
the client's. No kernel, contract or persistence type changes: posture is a System Pack, bodies gains one
extension catalog.

**Can 2D and 3D use it without duplicating game logic?** Yes. Both read the same disclosure through one
shared function (§8.1); both submit the same complete affordance (§9.1); the walk-then-sit chain is shared.
What differs is acquisition (a click versus a look) and drawing (a sprite versus a skeleton).

---

## 12. Invariants (proposed)

| Id | Invariant |
| --- | --- |
| I-S34-1 | **Exclusivity.** After every request, no two persons hold `Seated` with the same `(place, point)` |
| I-S34-2 | **Seated means at the point.** After every request, a person with `Seated { point }` has `Presence` place = the point's place, `local` = `at`, `facing` = the point's facing |
| I-S34-3 | **Only posture writes posture.** `SitPoints` and `posture-state` change only in posture's reductions; the location change of sitting is presence's own `Arrived` |
| I-S34-4 | **Clients decide nothing.** In a world that discloses sit points no client evaluates distance, stillness or occupancy; a client's seated drawing follows disclosure only |
| I-S34-5 | **One path.** Humans (2D, 3D), NPC controllers and MCP agents sit through the same complete affordance and the same validation |
| I-S34-6 | **Byte identity without the pack.** Worlds that do not enable `posture` are byte-identical before and after every S34 PR |
| I-S34-7 | **The seated are not displaced by others.** With bodies and posture installed, no fact moves a seated person except their own request |
| I-S34-8 | **No height in the simulation.** No posture type, fact or section carries a height, a mesh or a clip |
| I-S34-9 | **Every shipped world has sit points** wherever it draws a bench or a chair a person can reach (I-S33-5 extended: no drawn seat without a semantic point, no point without a drawn seat) |

---

## 13. Step acceptance and adversarial criteria

### 13.1 Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Owner PR |
| --- | --- | --- |
| AC-S34-1 | In a test world, a person within reach sits; their `Presence` moves to `at` and `facing`; `sat-down` is recorded; the observer and a bystander both see `posture-state` Seated; `stand` removes it and leaves the location | S34-a |
| AC-S34-2 | Every refusal row of §6.1 and §6.2 is produced by a request built to trigger it, and the same reason appears on the corresponding offer before dispatch | S34-a |
| AC-S34-3 | `move` / `walk-to` from a seated person emits `stood-up { cause: Moved }` in the same dispatch; I-S34-2 holds after every request of a seeded run | S34-a |
| AC-S34-4 | Seated at SIGKILL → resumed → still seated, same `since`; replay byte-identical on arm64 and x86_64; Windows runs the same suite | S34-a, S34-c |
| AC-S34-5 | With bodies and posture installed, a walker aimed through a seated person is stopped short, never nudges them; `shove` of a seated person is `TargetUnavailable`; a sit point inside a solid is refused at load naming the key | S34-b |
| AC-S34-6 | Every shipped world (Social Café, Market Town, Lakeside, both station halls) has sit points at every drawn reachable seat; a seeded 30-day run per world holds I-S34-1, I-S34-2 and I-S34-7 after every request, and NPCs are seen seated | S34-c, S34-d |
| AC-S34-7 | An MCP agent (recorded, deterministic) walks to the pier bench, sits, is seen seated by a human seat and by an NPC's observation, and stands; zero lines change in the adapter's act dispatch | S34-d |
| AC-S34-8 | `AC-13`: the 2D click and the 3D look+E on the same sit point record the same semantic core | S34-e, S34-f |
| AC-S34-9 | 3D: every disclosed seated person is drawn seated at their point with seat contact within ± 20 mm for every cast body and extra; stand-up returns to locomotion; no seated pose without a disclosed Seated (I-S33-5) | S34-f |
| AC-S34-10 | 2D: a disclosed seated person is drawn seated facing `facing`; a still person at a sit point who is not disclosed seated is drawn standing | S34-e |
| AC-S34-11 | Removing `posture` from a world makes `sit`/`stand` `Unavailable`, removes every seated drawing from the server's side, restores S33's shared rule in clients, and changes nothing else | S34-c |
| AC-S34-12 | The operator walks it (`VIS-POSTURE`): sit at the café window bench in 3D, see Nils seated on the pier in Lakeside, try a taken seat and read the refusal, sit in the 2D café, stand by walking away | S34-f |

### 13.2 Planted mutations (each must turn a named test red)

| Id | Mutation | Expected red |
| --- | --- | --- |
| X-S34-1 | drop the occupancy check (row 7) | the two-requesters-one-point contention test; I-S34-1 scan in the seeded run |
| X-S34-2 | drop the `stood-up { Moved }` reaction | I-S34-2 scan; AC-S34-3 |
| X-S34-3 | unregister posture from bodies' `Anchoring` catalog | AC-S34-5 nudge test; I-S34-7 scan |
| X-S34-4 | make the offer evaluate `reach` but not `validate` (or the reverse) | the offer-equals-dispatch test of AC-S34-2 |
| X-S34-5 | a client draws seated from SD-S33-7 in a world that discloses sit points | AC-S34-10's "still but not seated is standing" test; the rule-constant scan |
| X-S34-6 | move one depiction 200 mm off its disclosed `at` | the drawn-vs-disclosed check names the place and key |
| X-S34-7 | raise one depiction's seat by 50 mm | the `--seat` probe names the point and the body (AC-S34-9) |
| X-S34-8 | iterate occupancy over a `HashMap` / sort sit points by insertion order | clippy ban; the cross-platform replay digest |
| X-S34-9 | persist `since` as a float, or a height in a sit point | schema test; I-S34-8 scan |
| X-S34-10 | place a sit point 100 mm from the café bench slab | load refused with `BODIES_FIXED_POINT_IN_SOLID` naming `window-bench.n` |

### 13.3 Adversarial scenarios (beyond the mutations)

- Two clients (2D and 3D) and one agent request the same point in one tick: exactly one `sat-down`, two
  `TargetUnavailable`; replayed identically.
- A person stands on a free point's `at`; another tries to sit there: `TargetUnavailable` (row 8), the
  stander is not nudged.
- A seated person is invited to a group activity and joins: still seated; leaves the activity: still seated.
- A seated person's controller disconnects and the seat is taken over: still seated (posture is world
  state, not session state).
- A world where `posture` is enabled but `bodies` is not: sitting works; there is no anchoring to ask; a
  seated person can still only be moved by their own request (nothing else moves people in such a world).
- A list forbids `sit` for a class: the offer carries `PermissionDenied` and dispatch answers the same; a
  list that tries to forbid `stand` is refused at load.
- Resume with `interactions/posture.yaml` edited: refused as drift.

---

## 14. PR split

### 14.1 Order and dependencies

```text
S34-0  this step design (docs) ──────────────────────────────┐
                                                             ▼
S34-a  posture System Pack (test world only) ─────────┬──────────────┬──────────────┐
                                                      ▼              ▼              ▼
12d (#171) ──► S34-b  bodies' Anchoring catalog ──► S34-c  sit points in every world, enabled
S29 station places ─────────────────────────────────►      (re-baseline)
S33-b Lakeside bodies ──────────────────────────────►            │
                                                                 ▼
                         S34-d  NPCs and agents sit (controllers, agent view, 30-day runs)
S33-f (shared module) ─┬──► S34-e  shared rule override + 2D seated drawing
16c/12e figures.gd ────┴──► S34-f  3D sit/stand, player input, seat contact, VIS-POSTURE
```

### 14.2 The PRs

| PR | Scope | Depends on | Integration checkpoint | Adversarial focus | Implementer |
| --- | --- | --- | --- | --- | --- |
| **S34-a** posture System Pack (`ARC-147`) | `systems/posture/` crate: `SitPoints`, `posture-state`, `sit_points:` section + genesis fact, `sit`/`stand`, offers, disclosure, the `Moved` reaction, the Interaction List section, registration in `systems/installed`, generated schema in `docs/reference/`, spec edits (§18); a test world under `tests/` or `worlds/` fixtures only — no shipped world enables it | S34-0 approved; main as is (presence, movement) | create world → sit → observe from a bystander → walk away (auto-stand) → persist → SIGKILL → resume → still seated, byte-identical, three platforms | X-S34-1, -2, -4, -8, -9; contention | Opus design, Haiku implement from a commit plan |
| **S34-b** bodies' `Anchoring` catalog | `Anchoring` trait and catalog in `systems/bodies`; posture's implementation; nudge/stop-short, `shove` refusal, load-time fixed-point check; bodies VERSION bump | S34-a; **12d merged** (#171: radius 250 mm, the shipped worlds' solids) | a walker through a seated person in the café on 12d's solids is stopped short; byte identity of every world without posture | X-S34-3, -10 | Opus (touches bodies' resolver; a framework seam) |
| **S34-c** sit points in every world | `sit_points:` in Social Café, Market Town, the station halls (S29), Lakeside; `posture` enabled in each `world.yaml`; spacing and reach sources cited; re-baselines recorded | S34-b; S29's hall places on main; S33-b (Lakeside `body:` sections) | each world loads; seeded 30-day runs hold I-S34-1, -2, -7; removing the pack gives AC-S34-11 | I-S34-9 audit (every drawn seat has a point) against S33-f / S29-c depictions | Haiku from a commit plan |
| **S34-d** NPCs and agents | evaluate the paced and motivated controllers on the 30-day runs; Q-S34-11's stillness gate if needed (inside posture); `clients/agent` `disclosed.py` row and words; the recorded-agent scenario | S34-c; S27-d (MCP) on main | AC-S34-6's "NPCs are seen seated"; AC-S34-7 | an agent attempts a stale sit offer (`stale_offer`); a taken point | Haiku |
| **S34-e** shared override + 2D | the override branch in `clients/shared/` (S33 §14.2), the shared walk-then-sit chain, 2D seated drawing and input, the rule-constant scan entry | S34-a (disclosure shape); S33-f (the shared module) — or S34-e creates the module if S33-f has not landed (Q-S34-9) | 2D café: click bench → walk → sit; X-S34-5 | X-S34-5 | Haiku |
| **S34-f** 3D | `human.gd` `stand()` and transitions, pelvis calibration, `figures.gd` posture, player E / move-to-stand, depictions bound by key, drawn-vs-disclosed check, `--seat` probe on server-disclosed points, `VIS-POSTURE` | S34-e; 16c / 12e merged (`figures.gd`); S33-f depictions for Lakeside | AC-S34-8, -9, -12 | X-S34-6, -7 | Opus frame review, Haiku implement |

S34-a, S34-b and S34-c are the simulation half and can land before any client work: until S34-e, clients
in a posture world still draw by S33's rule (they do not yet read `posture-state`), which is a temporary,
recorded gap, not a contradiction, because the server is already right.

### 14.3 What remains after S34

Lying, kneeling, vehicles' seats, and seat preferences in cognition. None is release-blocking.

---

## 15. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-S34-1 | 12d (#171) stays unmerged, blocking S34-b/S34-c | S34-a lands independently; S34-b is designed against 12d's constants and rebases |
| R-S34-2 | Bobbing NPCs: the frozen 20 % offered band seats and unseats people too often | Q-S34-11's pack-side stillness gate; measured in S34-d's runs before deciding |
| R-S34-3 | Sit-down/stand-up transitions look mechanical without clips | procedural transitions first; Q-S34-12 adoption if a reviewed frame says so |
| R-S34-4 | S33-f and S34-e both edit the shared sit module | Q-S34-9 sequencing; one owner per file at a time |
| R-S34-5 | Bodies' VERSION bump refuses existing saves of the shipped worlds | expected and recorded; worlds re-baselined in S34-c; release notes |
| R-S34-6 | A chair's sit point in a tight café layout cannot be 260 mm clear of the table solid | author chairs' points on the aisle side; move a solid within 12d's 150 mm match tolerance; or accept fewer points (recorded per place) |

---

## 16. Operator questions (each with a recommendation; ruled 2026-10-18)

| Id | Question | Recommendation | Ruling |
| --- | --- | --- | --- |
| **Q-S34-1** | **Names.** "Seat" already means a player/agent seat; "posture" is also an S30-e attribute. What do we call things? | Pack `posture` (your word, `ARC-147`); component `posture-state`; furniture term **sit point**, never "seat". S30-e keeps its attribute name (different namespace), with a glossary line in `CORE_CONCEPTS.md` | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-2** | Is a bench/chair an **entity** (targetable like a person) or **data of its place**? | **Data of its place** (`sit_points:` section). Items are kinds with counts (`ARC-36`) and Systems may not create entities (QPL-11); the 3D client targets the depiction and sends the point's key | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-3** | Where is a seated person, for the server? Over the seat (inside the bench's solid) or on the floor in front of it? | **On free floor at the seat's front edge.** No height in the server; bodies needs no exception; the client lifts the pelvis onto the seat | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-4** | If a seated person walks, does the server **refuse** the walk (`Busy`, stand first) or **stand them up automatically**? | **Stand up automatically** (a reaction to the move). Movement does not change, and agents cannot get stuck seated | **Ruled 2026-10-18 (operator): auto-stand.** Walking while seated stands the person up automatically |
| **Q-S34-5** | May others **push** a seated person (walkers' nudges, `shove`)? | **No.** Bodies gains a small `Anchoring` extension; walkers stop short; `shove` is refused `TargetUnavailable`. This is a bodies change, so it needs your ruling | **Ruled 2026-10-18 (operator): a seated person cannot be pushed.** A bodies change, in S34-b |
| **Q-S34-6** | `sit` while walking: refused `Busy`, or accepted (which stops the walk)? | **Refused `Busy`**: posture does not own walking; the client/agent stops or arrives first | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-7** | Refusal mapping: taken seat = `TargetUnavailable`; already seated or walking = `Busy`; too far = `TooFarAway`; no such point = `NoSupportedInteraction`; forbidden by the world's list = `PermissionDenied` | **Accept as listed** (§6.1) | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-8** | Reach and spacing defaults | **Reach 750 mm** (300–1 500, per world); **spacing ≥ 510 mm**, benches authored at 600 mm per person; sources cited at the PR freezes | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-9** | **Order with S33-f** (sit points of Lakeside). Should posture land before S33-f? | **Yes: S34-a before S33-f.** Then S33-f authors Lakeside's points once as world data and only their depictions in presentation; S33's rule stays as the fallback for worlds without posture. If S33-f is already in flight, it proceeds and S34-c moves its positions into place files | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-10** | 2D seated drawing | **A `seated` view rendered from each 3D character via S32's 2D route**; until a person has one, the standing view cropped at the hip as a fallback | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-11** | NPC sitting frequency: leave the frozen rule controller as is, or gate `sit` offers until a person has stood still ≥ 60 s? | **Measure first in S34-d's 30-day runs; add the 60 s gate inside posture only if people bob.** The gate applies to players too | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-12** | 3D sit-down/stand-up animation: our own procedural transitions, or adopt free clips (e.g. Mixamo, CC0 mocap) after a licence check? | **Our own procedural transitions now**; adoption only after the licence is read at source and a `DEP-` record is written; nothing is bought | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-13** | Scope: only sitting in MVP-0 (no lying/kneeling, no vehicle seats)? | **Yes, sitting only**; the type leaves room for lying and vehicles later | **Ruled 2026-10-18 (operator): as recommended** |
| **Q-S34-14** | Implementers | **Opus** for S34-b and S34-f's frame review; **Haiku** from frozen commit plans for S34-a, -c, -d, -e | **Ruled 2026-10-18 (operator): as recommended** |

---

## 17. Records

| Id | Content | Written by |
| --- | --- | --- |
| `ARC-147` | The `posture` System Pack: sit points as place data on free floor; `posture-state` (Seated only; standing = absence); `sit`/`stand` as complete offers; refusal mapping; auto-stand on movement; bodies' `Anchoring` catalog; disclosure; the shared presentation decision that overrides S33's rule (amends `ARC-145`'s scope, non-preclusion as S33 promised) | S34-a (the pack), with S34-b adding the bodies note |

Under #258's layout, the record lands as `docs/decisions/ARC-147.md`; otherwise in `docs/DECISIONS.md`.

---

## 18. Edits elsewhere (proposed; made by the PR named)

- **`overall.md`** (primary session, at approval): the S34 row in the S26–S33 table (renamed S26–S34);
  "its step document is still to be written" → this file.
- **`step-33-lakeside-3d.md`** (primary session, at approval): a dated note in §4.5 and §14.2 pointing here;
  S33-f's row gains "sit points authored as `sit_points:` in place files if S34-a has merged" (Q-S34-9).
- **`docs/CORE_CONCEPTS.md`** (S34-a C1): "sit point" in the glossary; one paragraph under §6 that a sit
  point is semantic place data and its drawing is presentation; the seat/sit-point distinction.
- **`docs/MODULE_SPEC.md`** (S34-a C1): the `sit_points:` place section; the posture section of the
  Interaction List; the depiction binding (S34-f).
- **`docs/reference/`** (S34-a): generated schemas for `sit_points:` and `interactions/posture.yaml`.
- **`clients/shared/README.md`** (S34-e): the override branch.
- **`docs/HUMANOID_PROFILE.md`** (S34-f): the stand clip and transitions, the pelvis calibration.
