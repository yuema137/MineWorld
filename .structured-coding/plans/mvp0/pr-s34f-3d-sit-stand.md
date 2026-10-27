# PR S34-f — sitting and standing in the 3D client

## Lifecycle

```text
Design revision:        revision 1 (2026-10-26), DRAFT
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §14)
Audit base:             origin/main @ 0b2faf2f (after #462 S33-f, #368 S34-c1, #363 S34-b, #301 S34-a)
Start condition:        S34-e merged (the shared walk-then-sit chain in clients/shared/sitting/, and the
                        2D half of AC-S34-8); #460 (16f-d2) merged or this branch rebased on it (§2.3)
Execution contract:     §12
Lifecycle:              DRAFT — not frozen. Only the operator freezes this design.
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§8.2, 8.3, 9.1,
12–14 (the S34-f row; AC-S34-8 3D half, AC-S34-9, AC-S34-12; X-S34-6, X-S34-7; I-S34-4, -5, -8, -9;
Q-S34-12, Q-S34-14) · **Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (#363),
[`pr-s34c-world-sit-points.md`](pr-s34c-world-sit-points.md) (#368, S34-c1) · **Sibling:**
`pr-s34e-2d-seated.md` (S34-e, designed in parallel; not on main at this audit) · **Consumes:**
[`pr-s33f-interiors-sit-points.md`](pr-s33f-interiors-sit-points.md) (#462: the shared decision, the depiction
file, `Human.stand()`, `SeatContact`, the `--seat` probe) · **Record:** a dated note on `ARC-147`; one record
for the clip-library decision (Q-S34f-2), id from the coordinator.

**Working name:** S34-f. **Branch:** `mvp0/pr-s34f`. One worktree, one session.

**In one sentence.** S33-f already draws a seated person in 3D, but only by placing them on the seat in one
frame, only for other people, and only in Lakeside; this PR makes sitting down and standing up visible
motions, lets the player sit with E and stand by moving, draws the player's own body seated, gives Social
Café and Market Town their seat depictions, and turns the `--seat` probe into a gate over the points the
server discloses, in every world that has them.

---

## 0. Binding inputs

- `CLAUDE.md` §4 rules 3, 14, 15, 16. The client reports intent. The server decides who sits where.
  Height, pelvis and animation are presentation.
- Step 34 (frozen 2026-10-18): §8.1 SD-S34-8 (one shared decision), §8.2 (depictions bound by
  `{place tag, key}`; drawn-vs-disclosed check), §8.3 (the S34-f list), §9.1 (one path), I-S34-4
  (clients decide nothing), I-S34-8 (no height in the simulation).
- Q-S34c-2 (ruled 2026-10-19): `seat_offset_m` is signed. A chair's point stands behind the chair; the
  pelvis is drawn forward onto the seat.
- Q-S34-12 (ruled): procedural transitions now; clip adoption only after reading each licence at source;
  nothing is bought. Decided at this freeze (§11, Q-S34f-2).
- Q-S34-14 (ruled): Opus frame review, Haiku implementation.
- Lane-lead charter: no world-specific hard-coding; no interpenetration at rest or across animation;
  tools first, no hand-tuned per-asset numbers; every acceptance criterion names a second world or a
  generated variant.

## 1. What S34-f must still deliver (gap analysis)

The step's S34-f row predates S33-f. S33-f (#462) landed part of it. This table separates done from open.

| Step §8.3 item | State on main | Left for S34-f |
| --- | --- | --- |
| `human.gd` `stand()` | **Done**: `stand()` returns the tree to `move` and re-enables `Posture`, ground and feet (`human.gd:738-750`) | Nothing |
| Pelvis calibration | **Done, better than planned**: `sit_on(surface_y)` measures each body's own seat vertex (`SeatContact.hips_over_seat`, `seat_contact.gd:79-85`) instead of the fixed 0.250 m | Nothing |
| Sit-down / stand-up transitions, `xfade_time > 0` | **Open**: `mode.xfade_time = 0.0` (`human.gd:512`); a person snaps between poses | C1 |
| `figures.gd` reads `posture-state` | **Half**: `SliceSeating` reads it through the shared rule and calls `figures.seat()` (`seating.gd:24-42`), which **places** the figure on the seat in one frame (`figures.gd:69-80`) | C2: walk to `at`, turn, sit down; stand up, then walk |
| Player E / move-to-stand | **Open**: E is "ask words of the person faced" (`slice_link.gd:947`); nothing submits `sit` or `stand` | C3 |
| The player's own body seated | **Open**: `seating.observe` only walks `figures.nodes`; the observer is not among them (`slice_link.gd:372-395`) | C3 |
| Depictions bound by key | **Done for Lakeside only**: `sit_points/lakeside.yaml`. Social Café and Market Town disclose 21 points each (S34-c1) but have **no depiction file**, so `pose_of` returns `{}` and every seated person there is drawn standing (`seating.gd:47-50`) | C4 |
| Drawn-vs-disclosed check | **Open**: the probe only fails when no drawn seat lies under the pelvis (`slice_probe_seat.gd:138-142`); a disclosed point with no depiction is never reported | C5 |
| `--seat` probe on server-disclosed points | **Half**: the probe iterates the depiction file's points, and uses the disclosed `at` where the world discloses one (`slice_probe_seat.gd:127-131`); it runs only for Lakeside in CI (`scripts/ci_layer.py`, S33-f line) | C5 |
| AC-S34-8 (3D half of AC-13) | **Open** | C6 |
| `VIS-POSTURE` | **Open** | C7 |

## 2. Source audit (`origin/main @ 0b2faf2f`)

### 2.1 The 3D client

| # | Fact | Where |
| --- | --- | --- |
| A-1 | The tree is `Locomotion` → `Rate` → `Mode` (an `AnimationNodeTransition` with inputs `move`, `sit`), `xfade_time = 0.0` | `clients/3d-spike/scripts/human.gd:497-528` |
| A-2 | `Sit` is a one-key looping clip built in code; Quaternius' `Sitting_Enter/Idle/Exit` are in the paid tier | `human.gd:537-571` |
| A-3 | `sit(seat_y)` puts the `Hips` joint at `seat_y` after two frames (`_settle_seat`, `_place_on_seat`); `sit_on(surface_y)` then lifts it by the measured hips-over-seat | `human.gd:701-735`, `796-825` |
| A-4 | `SeatContact.lowest` CPU-skins the seat vertices (hips and upper legs) of the current pose; one rule for every body | `clients/3d-spike/scripts/seat_contact.gd:23-74` |
| A-5 | `SliceFigures.seat(id, at, yaw, height)` sets position and yaw and calls `sit_on` at once; `unseat` calls `stand()`; a seated figure is skipped by `_process` | `slice/figures.gd:66-90`, `152-159` |
| A-6 | `SliceSeating.pose_of` takes `at` and `facing` from the disclosed `sit-points` value when present, else the depiction's fallback; pelvis = `at` moved by `seat_offset_m` against facing | `slice/seating.gd:47-67` |
| A-7 | The shared decision has a posture branch: where the place discloses `sit-points`, a person is seated iff their `posture-state` names a point | `clients/shared/sitting/sit_rule.gd:95-113` |
| A-8 | The depiction file is `presentation/mineworld-default/3D/sit_points/<world>.yaml`; only `lakeside.yaml` exists | `presentation/mineworld-default/3D/sit_points/` |
| A-9 | `--seat` probe (`SliceProbeSeat`): walks the world's probe route; for each depiction point bound to the place, rays down for the drawn seat, seats every cast body, extra, default and the town figure, and fails beyond 20 mm. Windowed (skinning needs it) | `slice/slice_probe_seat.gd:1-199`; `mineworld-slice` (`--seat`) |
| A-10 | CI runs `--seat` for Lakeside only | `scripts/ci_layer.py` (the S33-f line under the clients layer) |
| A-11 | Key handling: E talk, F kick, G throw, R shove; `aim()` knows people and loose objects only | `slice/slice_link.gd:856-873`, `943-950` |
| A-12 | Every request is built in `SliceIntents`; `submit_offered` sends a complete affordance unchanged | `slice/intents.gd:1-70` |
| A-13 | `affordances(action, target)` with `target == null` returns offers directed at nobody; `sit` and `stand` have `target: None` and payloads `{point}` and `{}` | `clients/protocol/mineworld/observation.gd:200-226`; `systems/posture/src/action.rs:1-60` |
| A-14 | The player's body is an `NPC` PUPPET with a `Human` body; hidden in first person | `scripts/player.gd:159-177`; `npc.gd:83, 124-160` |
| A-15 | Furnishings are drawn from the world's `body: furnishings` by key and kind (`ARC-307`); café chairs and stools have keys such as `table.east.chair-w`, `stool.1` | `slice/furnishings.gd:149-349`; `worlds/social-cafe/places/cafe.yaml:50-64` |
| A-16 | The café's sit points stand behind their chairs and stools (Q-S34c-2): e.g. `table-1.1` at (3 648, 3 060) for the chair at (4 200, 3 060), facing 81 406 | `worlds/social-cafe/places/cafe.yaml:129-138` |
| A-17 | The street's benches are drawn by the slice's code, not from furnishings; the park has no drawn bench | `worlds/social-cafe/places/street.yaml:106-126`; `park.yaml:36-46` |
| A-18 | AC-13's harness compares frames the real Godot client submitted (`clients/protocol/evidence/request-<flavour>.json`) through `mineworld_server::semantic_core` | `tools/cli/tests/ac13_semantic_parity.rs:1-60, 176-230` |
| A-19 | The pose sheet renders P-4 seated with the client's own `human.gd` and `seat_contact.gd`; G-13 is decided there | `tools/character_pipeline/godot/pose_sheet.gd:21-35, 438-530`; `mwcast/pose_sheet.py:36` |
| A-20 | S36's contact module exists: libigl (`DEP-142`) and trimesh / python-fcl (`DEP-143`) behind `mwcast/contact.py`; tolerances in `contact_tolerances.json` (`item.overlap.parts`, `floor.sink.foot`, …) | `tools/character_pipeline/mwcast/contact.py:1-40`; `contact_tolerances.json` |
| A-21 | The station halls of Social Café and Market Town draw 34 bench rows each from furnishings, but no sit point is authored there | `worlds/social-cafe/places/station.yaml:25-40`; `grep sit-points worlds/` |
| A-22 | Lakeside does not enable `posture` yet (S34-c2 pending); its probe runs on the fallback branch | `worlds/lakeside/world.yaml` |

### 2.2 Licences read at source for Q-S34-12 (2026-10-26)

| Library | What the source says | Fit |
| --- | --- | --- |
| Quaternius Universal Animation Library | CC0, "free to use in personal, educational and commercial projects"; the free Standard tier is 60–70 % of the pack; the rest is Pro / Source (paid). The repository already notes `Sitting_Enter/Idle/Exit` are paid-tier (`human.gd:537`) | Licence fine; the clips need a purchase. Spend is not authorized |
| Mixamo (Adobe) | Royalty-free use in projects; distributing raw character or animation files is not allowed (Adobe FAQ, quoted in Adobe's community reposts; the helpx page refused the fetch, HTTP 403) | **Incompatible**: MineWorld is open source and commits its animation files |
| CMU Graphics Lab mocap | Free for research and for inclusion in commercial products; the data may not be resold (terms as mirrored by Papers with Code and 4TU; the official page failed TLS) | Licence fits. Raw mocap needs retargeting to the `mixamorig:*` profile, cleanup, and a seat of matching height; a real project, not a drop-in |

Sources: <https://quaternius.com/packs/universalanimationlibrary.html>,
<https://community.adobe.com/questions-696/mixamo-faq-licensing-royalties-ownership-eula-and-tos-589400>,
<https://paperswithcode.com/dataset/cmu-motion-capture>, <https://data.4tu.nl/datasets/0448aab2-3332-449f-a8e2-d208cb58c7df>.

### 2.3 Open PRs touching the same files

| PR | Touches | Effect on S34-f |
| --- | --- | --- |
| #460 (16f-d2, conflicting) | `slice/figures.gd` (one line in `_leave`), `slice_main.gd`, `slice_probe_world.gd`, both depiction scripts, `ci_layer.py` | Textual overlap only. S34-f starts after #460 merges, or rebases on it (start condition) |
| #496 | `slice/slice_layout.gd` | None expected; S34-f does not edit the layout |

## 3. Dependency on S34-e, stated precisely

S34-e owns three things S34-f consumes. S34-f does not write them.

1. **The shared walk-then-sit chain**, in `clients/shared/sitting/` (step §8.4). Given an observation and a
   sit point key, it answers the next intent: the complete `sit {point}` offer when it is available or
   refused for any reason but distance; a `walk-to` the point's `at` when the offer says `TooFarAway`; and,
   on the walk's end, the same `sit` offer again. Its file and function names are S34-e's; S34-f calls it
   by whatever name S34-e freezes.
2. **The 2D half of AC-S34-8**: the 2D client's recorded `sit` frame in `clients/protocol/evidence/` and the
   test case that compares it with the 3D frame.
3. **The rule-constant scan entry** (step §8.1) that no posture constant appears in `clients/3d-spike`.

If S34-e's design moves any of these into S34-f, that is a scope change for both designs and goes to the
operator. The posture branch of the shared decision itself already exists (A-7); neither PR rewrites it.

## 4. Scope

**In:**

1. `human.gd`: sit-down and stand-up transitions (C1).
2. `slice/figures.gd`: a seated person walks to `at`, turns, sits down; on standing, stands up, then walks (C2).
3. The player: E on a sit point, movement or E to stand, the player's own body drawn seated, the seated
   eye height (C3).
4. `sit_points/social-cafe.yaml` and `sit_points/market-town.yaml`, generated from world data by a small
   committed tool (C4).
5. The `--seat` probe as the drawn-vs-disclosed gate over disclosed points, in every world with a depiction
   file, plus a generated variant; CI lines for Social Café and Market Town (C5).
6. The 3D half of AC-S34-8 (C6).
7. `VIS-POSTURE`, `docs/HUMANOID_PROFILE.md`, `clients/3d-spike/PRESENTATION.md` §10, the `ARC-147` note (C7).

**Out:**

- The shared chain, the 2D drawing, the rule-constant scan (S34-e).
- Sit points in the station halls (A-21). That is world data (S34-c's kind of work), Q-S34f-6.
- Lakeside's `posture` and its server sit points (S34-c2).
- NPC frequency and the agent view (S34-d).
- Pulling a tucked chair out when a person sits behind it (Q-S34f-1 recommends a follow-up, S34-f2).
- Any server, contract or System change. If one seems needed, stop and escalate.

## 5. Design

### 5.1 Transitions in `human.gd` (C1) — SD-34f-1

- Replace the `Mode` transition with Godot's own `AnimationNodeStateMachine` (adopt the engine's tool; no
  hand-written blend timer). States: `move` (the existing `Rate` → `Locomotion` chain), `sit_down`, `sit`,
  `stand_up`. Transitions `sit_down → sit` and `stand_up → move` advance automatically at the end of the
  clip. `move → sit_down` and `sit → stand_up` are travelled on request.
- `SitDown` and `StandUp` are procedural clips built in code, as `Sit` is (A-2): keys at 0 s take the
  bones from the `Stand` clip's first frame, keys at the end from `Sit`, interpolated by Godot. Length
  0.6 s each (step §8.3 target). No new asset, no purchase (Q-S34-12).
- The pelvis height is part of the motion. Over `sit_down` the root's vertical offset goes from standing
  (`-ground.sole_y`, as `stand()` sets it) to the seated offset `sit_on` measures; over `stand_up` it goes
  back. The horizontal path is the caller's (§5.2), because only the caller knows `at` and the pelvis point.
- `sit_on(surface_y, animate := true)` and `stand(animate := true)`. `animate := false` keeps today's
  behaviour exactly; the probe and the pose sheet use it, so G-13 and the `--seat` numbers do not move.
- The hips-over-seat value is measured once per body, in the `Sit` pose, and cached on the `Human`. It is
  a property of the body and the clip, not of the seat. The transition therefore knows its end height
  before it starts.
- `is_settling()` reports a running transition, so callers do not start another.

### 5.2 Other people (`figures.gd`, C2) — SD-34f-2

```text
posture-state appears (Seated {point})
    the figure walks to the disclosed `at` (its goal already: the server moved them there)
    → turns to the point's facing
    → sit_down: the root slides from `at` to the pelvis point while the pose and height blend
seated  (held; `_process` skips it, as today)
posture-state disappears
    stand_up: the root slides from the pelvis point back to `at`
    → locomotion resumes toward the newest observed position
```

- **A figure first seen seated** is placed seated with no transition (it was seated before we looked).
- **A seat where the straight slide would pass through the drawn chair** is placed in one frame, as main
  does today, until S34-f2. The rule is data-derived and names no world: a negative `seat_offset_m`
  (the point behind a chair, Q-S34c-2) means the slide crosses the chair's back. The probe reports
  each such point as `cut` (never as a pass of the transition check). Q-S34f-1.
- A person who stands up and walks away at once (the server's auto-stand, SD-S34-6) plays `stand_up`
  before walking; the figure catches up by `CATCH_UP`'s existing rule. No new constant.

### 5.3 The player (C3) — SD-34f-3

- **Aiming at a seat.** `SliceSeating` adds one pickable body per depicted point in a drawn place: a box
  on the targeting layer at the pelvis point, the seat's width along the facing's normal from the drawn
  seat, carrying meta `sit_point = <key>`. `aim()` gains the kind `"sit_point"`. Nothing here evaluates
  distance or occupancy.
- **E** acts on what the reticle meets (Q-S34f-3): a person → talk (unchanged); a sit point → the shared
  chain of §3 with that key; while the player is disclosed seated → the complete `stand {}` offer.
- **Moving while seated** (Q-S34f-5). The first movement key press while disclosed seated submits the
  `stand {}` offer once. The body does not move until the observation shows the player standing. The
  client never predicts standing.
- **Far seat** (Q-S34f-4). The shared chain sends `walk-to` and then `sit`; the player's body follows the
  server's walk exactly as it does for any `walk-to`. 2D and 3D then send the same two intents.
- **The player's own body.** The observer's `posture-state` is read from the same observation; when Seated,
  the player's `Human` plays the same `sit_down` to the same pelvis point (`SliceSeating.pose_of`), and the
  correction rule (`ARC-50`) is suspended while seated, because the server's `at` and the drawn pelvis
  differ on purpose. On standing, the body returns to `at` and correction resumes.
- **Camera.** First person: the eye follows the `Head` bone, so the seated eye height is whatever the body
  gives. Third person: unchanged rig. Both are presentation only.
- Refusals arrive as today (`_on_refused`) and are said in the HUD in the player's language: "that seat is
  taken" for `target_unavailable`, and so on. New catalog keys only; no rule.

### 5.4 Depictions for Social Café and Market Town (C4) — SD-34f-4

- A committed tool, `tools/presentation/sit_point_depictions.py`, writes a world's depiction file from
  world data. For each disclosed sit point it finds the furnishing whose solids are nearest along the
  point's facing line; `prop` is that furnishing's key; `seat_offset_m` is the signed distance from `at`
  to the furnishing's `at` along facing (A-16: `table-1.1` → about −0.55 m); `seat_height_m` is the
  kind's seat height from the furnishings catalogue when present, else 0.45 m (`VISUAL_SLICE.md` §3.4;
  ADA 2010 §903.5, as `lakeside.yaml` cites). A point with no furnishing on its line (the street's
  code-drawn benches) gets `prop: ""` and the bench default 0.45 m.
- The tool reads only the world's files. It names no world, place or key. Social Café and Market Town
  share place files today, so their outputs are equal; they are two files because a depiction file is
  scoped to one world (`sit_rule.gd:59`).
- Every value is then checked against what is drawn by the `--seat` probe (§5.5). A value the probe
  refuses is fixed in the world data or the catalogue, never patched in the output by hand.
- Lakeside's file stays as S33-f wrote it; S34-c2 removes its `fallback` blocks.

### 5.5 The gate: `--seat` over disclosed points (C5) — SD-34f-5

In a place that discloses `sit-points`, the probe's list is the **disclosed** points, not the file's:

1. A disclosed point with no depiction fails, naming world, place and key.
2. The pelvis point must lie over the drawn seat with margin: rays down at the pelvis and at ±100 mm
   along facing must all meet the same drawn surface within 20 mm of height. A depiction 200 mm off
   (X-S34-6) puts one ray off the seat or onto the table and fails, naming the key.
3. Every body (cast looks, extras, default, the town figure) seated there: contact within ±20 mm of the
   drawn seat (unchanged, G-13). X-S34-7 (+50 mm) fails naming point and body.
4. **Transitions, no interpenetration.** For each point not marked `cut`, one body per build class plays
   `sit_down` and `stand_up`; at 30 samples per second the trunk and thigh vertices (A-4's skinning) are
   tested against the drawn furnishings' triangles, excluding the seat surface under the pelvis. Depth
   beyond `item.overlap.parts` in `contact_tolerances.json` fails. Feet below the floor by more than
   `floor.sink.foot` fails. Bounds come from the tolerance file; none is set in code.
5. A place the slice does not draw is reported `SKIP <place> not drawn`, never passed.

In a place that discloses none (Lakeside before S34-c2), the probe keeps S33-f's behaviour over the
fallback points. The verdict line is unchanged, so the CI layer reads it as before.

### 5.6 AC-S34-8, the 3D half (C6)

A real run of the 3D client against a real server, in Social Café: walk to `window-stools.1`, look at it,
press E. The submitted frame is committed as `clients/protocol/evidence/request-3d-sit.json` with its
transcript, the same way AC-13's talk frames were (A-18). S34-e's test case then compares it with the 2D
frame through `semantic_core`. The expected values come from the real clients, never from this PR's code.

## 6. Invariants

| Id | Invariant |
| --- | --- |
| I-34f-1 | **Clients decide nothing** (I-S34-4). No distance, occupancy or stillness test in `clients/3d-spike` for a world that discloses sit points. E sends; the server answers |
| I-34f-2 | **No seated pose without a disclosed Seated** (I-S33-5) in posture worlds, the player's own body included |
| I-34f-3 | **No interpenetration.** Seated: contact within ±20 mm. Transitions: no trunk or thigh in drawn furniture beyond the tolerance file's bound. A seat that cannot pass is `cut`, never drawn through furniture |
| I-34f-4 | **Generic.** No world, place, key or furnishing name in code or in the tool. Every number comes from world data, the furnishings catalogue, the depiction file or the tolerance file |
| I-34f-5 | **Unchanged at rest.** `animate := false` reproduces main's pose and height; the pose sheet's G-13 values do not move |
| I-34f-6 | **No server change** (I-S34-8). Nothing in `systems/`, `contracts/`, `kernel/`, `server/` changes |

## 7. Acceptance criteria (decided before measuring, `ARC-23`)

| Id | Criterion | Worlds |
| --- | --- | --- |
| AC-34f-1 | Another person who sits is seen walking to the point, turning and sitting down in about 0.6 s; on standing, standing up, then walking. A person first seen seated is drawn seated at once | Social Café (café table and street bench) **and** Lakeside (pier bench, fallback branch) |
| AC-34f-2 | The player looks at a free sit point within reach and presses E: the server records `sat-down`; the player's body (third person) is drawn seated; the first-person eye drops to the head bone. A movement key stands the player up; nothing moves before the observation says so | Social Café **and** Market Town |
| AC-34f-3 | E on a taken point shows the server's refusal and sends nothing else. E on a far point walks there and sits (the shared chain) | Social Café; Market Town |
| AC-34f-4 (AC-S34-9) | `--seat` passes: every disclosed point has a depiction; its pelvis is over the drawn seat with margin; every cast body and extra is within ±20 mm; every non-`cut` transition passes the contact check | Social Café, Market Town, Lakeside, **and a generated variant**: a seeded scratch copy of Market Town whose benches and sit points are moved by a generator, with its depiction file written by C4's tool |
| AC-34f-5 (AC-S34-8) | The 3D frame for `window-stools.1` and S34-e's 2D frame have the same `semantic_core` | Social Café (the evidence world); the test is world-agnostic |
| AC-34f-6 | `animate := false` gives byte-identical probe numbers and pose-sheet G-13 values to main | Lakeside probe; the pose sheet's cast |
| AC-34f-7 (AC-S34-12) | `VIS-POSTURE`: the operator sits at a café table in 3D, watches a person sit and stand on a Social Café street bench, sees a seated person on the pier in Lakeside, tries a taken seat and reads the refusal, and stands by walking away | Social Café, Lakeside (the 2D step of AC-S34-12 is S34-e's) |

## 8. Planted mutations

| Id | Mutation | Must turn red |
| --- | --- | --- |
| X-S34-6 | move one Social Café depiction's `seat_offset_m` by 0.20 m | AC-34f-4 item 2, naming the key |
| X-S34-7 | raise one depiction's `seat_height_m` by 0.05 m | AC-34f-4 item 3, naming the point and the body |
| X-34f-1 | delete one point from `market-town.yaml` | AC-34f-4 item 1 |
| X-34f-2 | let `sit_down` slide straight through the chair for a negative offset (drop the `cut` rule) | AC-34f-4 item 4, naming the key and the depth |
| X-34f-3 | draw the player seated on the E press, before the observation | AC-34f-2's scripted run (`SEATED` logged before `sat-down` is observed) |
| X-34f-4 | a distance test in `slice_link.gd` that refuses E beyond 0.75 m | S34-e's rule-constant scan entry |

## 9. Test ownership

| Failure class | Owner | Runs where |
| --- | --- | --- |
| Seat contact, depiction coverage, pelvis over seat, transition contact | `--seat` probe (`slice_probe_seat.gd`), CI clients layer | Godot, windowed, Linux CI per `ci_layer.py`; locally through `with-slot.sh` |
| The 3D request equals the 2D one | `ac13_semantic_parity.rs` (S34-e's case, S34-f's evidence) | cargo |
| Player and figure behaviour (AC-34f-1…3) | a scripted connected run (`mineworld-slice --world=<w> --sit`), printing `SAT`, `STOOD`, `REFUSED <code>`, `CUT <key>` lines, compared by a `tools/cli/tests/` test like `agent_with_3d_observer.rs` | Godot + server, `#[ignore]`d locally, run by the clients layer |
| The tool's output | its own `pytest` over world fixtures plus the generated variant | Python |
| No server change | `git diff --stat` in review | — |
| Real-LLM Gate | **NOT REQUIRED**: no model is involved | — |

Static ownership: `gdlint`/Godot import errors and `ruff`/`pyright` for the tool; no unit test repeats them.

## 10. Commit plan

Each commit tracks Implementation, Validation and Review separately.

### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_doc_headings.py`.
- [ ] Review: scope equals step §8.3 minus S33-f's delivered part (§1); every question in §14 has a recommendation.

### C1 — Transitions in `human.gd`
- **Goal:** sitting and standing are motions, not snaps. **Files:** `clients/3d-spike/scripts/human.gd`.
- [ ] Implementation: the state machine (SD-34f-1); `SitDown`, `StandUp` clips; cached hips-over-seat; `animate` flags; `is_settling()`.
- [ ] Validation: `--seat` on Lakeside with `animate := false` gives main's numbers (AC-34f-6); the pose sheet's G-13 table is unchanged; a frame-diff of a figure through `sit_down` shows no frame with a T-pose arm or a straight leg.
- [ ] Review: no caller of `sit`, `sit_on`, `stand` changes behaviour without asking for it.

### C2 — Other people sit and stand
- **Files:** `slice/figures.gd`, `slice/seating.gd`.
- [ ] Implementation: SD-34f-2, including the first-seen and `cut` rules.
- [ ] Validation: the scripted run on Social Café and on Lakeside logs walk, turn, `sit_down`, `stand_up` per AC-34f-1.
- [ ] Review: `_process` still skips seated figures; `_leave` unseats; no position reaches the server.

### C3 — The player
- **Files:** `slice/slice_link.gd`, `slice/seating.gd`, `slice/intents.gd` (only if the chain needs a send helper), `player.gd`, `correction.gd` (suspension while seated), the HUD catalog files.
- [ ] Implementation: SD-34f-3.
- [ ] Validation: AC-34f-2 and -3 on Social Café and Market Town; X-34f-3 red then reverted.
- [ ] Review: I-34f-1 — grep the diff for any distance or occupancy test; refusals come only from the server.

### C4 — Depiction files from world data
- **Files:** `tools/presentation/sit_point_depictions.py` and its test; `presentation/mineworld-default/3D/sit_points/social-cafe.yaml`, `market-town.yaml`.
- [ ] Implementation: SD-34f-4; sources block as `lakeside.yaml` has.
- [ ] Validation: tool tests on both worlds and on the generated variant; X-34f-1 red with C5.
- [ ] Review: the tool reads no name; each output value traces to a world field or a cited default.

### C5 — The gate
- **Files:** `slice/slice_probe_seat.gd`, `mineworld-slice` (help text), `scripts/ci_layer.py` (Social Café and Market Town `--seat` lines), `contact_tolerances.json` only if an existing region does not fit (then a new region with rationale and source).
- [ ] Implementation: SD-34f-5 items 1–5.
- [ ] Validation: the probe passes on three worlds and the variant; X-S34-6, X-S34-7, X-34f-2 each red, reverted, outputs recorded.
- [ ] Review: no bound set in code; `SKIP` and `cut` never counted as passes.

### C6 — AC-S34-8 evidence
- **Files:** `clients/protocol/evidence/request-3d-sit.json` and its transcript.
- [ ] Implementation: §5.6. If S34-e's test case is not on main, this commit waits; it is not rewritten here.
- [ ] Validation: `cargo test -p mineworld-cli --test ac13_semantic_parity` through `with-slot.sh`.
- [ ] Review: the frame came from a real run, not from an editor.

### C7 — Docs and VIS-POSTURE
- **Files:** `docs/ACCEPTANCE.md` (a `VIS-POSTURE` row), `docs/HUMANOID_PROFILE.md` (transitions, the cached hips-over-seat), `clients/3d-spike/PRESENTATION.md` §10 (disclosed points first; the tool), `clients/3d-spike/README.md` (one line: how to sit), `docs/decisions/ARC-147.md` (dated note), the Q-S34f-2 record, `python3 -I scripts/check_decision_ids.py --write-index`.
- [ ] Implementation: as listed; a short operator test list for `VIS-POSTURE`.
- [ ] Validation: doc scripts PASS.
- [ ] Review: no spec text contradicts step §8; the README stays short.

## 11. Decisions proposed

- **SD-34f-1…5** as in §5.
- **Q-S34-12 answered (recommendation):** keep procedural transitions; adopt no clip library now. Mixamo is
  rejected (raw files may not be redistributed). Quaternius' sitting clips need a paid tier (no spend
  authorized). CMU mocap is licence-compatible but needs a retargeting and cleanup project; it is the
  candidate if `VIS-POSTURE` finds the procedural motion mechanical. The rejection is recorded, as
  `REUSE_POLICY.md` requires for both directions.

## 12. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34f-3d-sit-stand
Branch / base:          mvp0/pr-s34f from origin/main after S34-e and #460 merge and this design freezes
Implementer:            Haiku, from this commit plan (Q-S34-14); Opus reviews C1's frames and C5's gate
Authorized (on freeze): implement, validate locally through cast-slots/with-slot.sh, commit, push, open the PR
Validation budget:      the probes and tests named per commit (speed-first charter); full suites at milestone
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges; never this session
Escalate:               any server or contract change; any change to S34-e's shared files; a bound in code
Post-merge sync owner:  this session owns this file's ledger; the primary session owns step-34 and overall
```

## 13. Risks

| Risk | Mitigation |
| --- | --- |
| #460 and S34-f both edit `figures.gd` and the probe family | Start after #460, or rebase; the overlap is small (§2.3) |
| S34-e slips | C1, C2, C4, C5 do not need it; C3's far-seat path and C6 wait |
| Café chairs are all `cut` in S34-f, so the café shows snaps at tables | Recorded per point by the probe; S34-f2 (Q-S34f-1) |
| Procedural transitions read as mechanical | `VIS-POSTURE` judges; CMU mocap is the next candidate (§11) |
| The transition contact check is slow (CPU skinning per frame per body) | One body per build class for transitions; all bodies only for the seated contact, as today |
| Correction suspended while seated hides a real server move | Suspension ends on any change of the disclosed point or on standing; a moved seated person is a server bug S34-b's tests own |
| The player's own seated body clips the camera in first person | The body is hidden in first person (A-14); only the eye moves |

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34f-1** | A chair's sit point stands behind the chair (Q-S34c-2). Sliding the body onto the seat would pass through the chair's back. Split the "pull the chair out, sit, tuck in" motion into a follow-up S34-f2, and draw those seats with today's one-frame placement until then? | **Yes, split.** S34-f animates benches and stools with open fronts; chair seats are `cut` and reported per point. S34-f2 adds the chair motion with its own contact gate |
| **Q-S34f-2** | Q-S34-12: adopt a sitting clip library? | **No, not now.** Procedural clips through Godot's own state machine. Mixamo is out (no redistribution of raw files); Quaternius' sitting clips are paid; CMU mocap is allowed but is a retargeting project, kept as the next candidate. Record the rejection |
| **Q-S34f-3** | E is "talk" today. Make E act on what the reticle meets: a person → talk, a seat → sit, while seated → stand? | **Yes.** One key, as Minecraft-like play expects; no new binding to learn |
| **Q-S34f-4** | E on a seat that is too far: walk there and sit (the shared chain), or only say "too far"? | **Walk there and sit**, through S34-e's shared chain, so 2D and 3D send the same two intents (step §8.4) |
| **Q-S34f-5** | A movement key while seated: send `stand` and wait for the world (one round trip), or move at once and let the server auto-stand? | **Send `stand` and wait.** The client never predicts posture; the delay is one observation |
| **Q-S34f-6** | The station halls draw 34 benches each in Social Café and Market Town, but have no sit points. Add them here? | **No: a small data PR, S34-c3,** under S34-c's frozen rules. S34-f's gate covers whatever points the world discloses |
| **Q-S34f-7** | Social Café and Market Town depiction values: generate them from world data with a committed tool, or author them by hand? | **Generate** (tools first, no hand-tuning). The probe then checks every value against what is drawn |

## 15. Evidence ledger

(empty until implementation)
