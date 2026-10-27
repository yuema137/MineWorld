# PR S34-e — the shared sit rule and seated people in 2D

## Lifecycle

```text
Design revision:        revision 1 (2026-10-26), DRAFT
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §13)
Implementation base:    origin/main @ 0b2faf2f (after #301 S34-a, #363 S34-b, #368 S34-c1, #462 S33-f)
Start condition:        this design frozen by the operator
Execution contract:     §10
Lifecycle:              DRAFT — never frozen by its author
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§8.1, 8.4, 9.1,
12–14 (the S34-e row; AC-S34-8 2D half, AC-S34-10; I-S34-4, I-S34-5; X-S34-5; Q-S34-9, Q-S34-10, Q-S34-14) ·
**Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (#363),
[`pr-s34c-world-sit-points.md`](pr-s34c-world-sit-points.md) (S34-c1, #368),
[`pr-s33f-interiors-sit-points.md`](pr-s33f-interiors-sit-points.md) (#462, which created the shared module) ·
**S33 parent:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §4.5, §6, §14.2 · **Record:** dated notes on
`ARC-145` and `ARC-147`. No new id.

**Working name:** S34-e (S34-e1 here; S34-e2 only if Q-S34e-2 splits the seated sprites off). **Branch:**
`mvp0/pr-s34e`. **Worktree:** `/Users/yuema137/mineworld-worktrees/s34e-2d-seated` (created at
implementation). One session holds it.

**In one sentence.** The server already knows who sits where, and both clients already decide "who is drawn
seated" through one shared function; but the 2D client still draws everyone standing and offers sitting only
as an anonymous "Sit down" line in the player's own menu. This PR makes the 2D client draw a seated person
seated, lets the player click a seat to walk there and sit, and puts the "walk, then sit" steps in the shared
module so the 3D client (S34-f) reuses them unchanged.

---

## 0. Binding inputs

- `CLAUDE.md` §4 rules 3, 14, 15: a client reports intent; the server decides. A seat's height and the
  pelvis offset are presentation. Reach, occupancy and stillness are the server's.
- Step-34 §8.1 (SD-S34-8), §8.4, §9.1, I-S34-4, I-S34-5, X-S34-5, AC-S34-8, AC-S34-10 (frozen 2026-10-18).
- Step-33 §4.5 and §14.2: the shared rule yields to a disclosed posture; no S33 code changes except the
  override branch.
- Q-S34-9 (ruled): S34-a before S33-f; S33-f proceeds if already in flight. It was: S33-f (#462) created
  `clients/shared/sitting/` with the posture branch first (audit row 1).
- Q-S34-10 (ruled): the seated view comes from each 3D character through S32's 2D route; until a person has
  one, the standing view cropped at the hip is the fallback.
- Q-S34-14 (ruled): Haiku implements S34-e from a frozen commit plan.
- Lane charter (2026-10-18): speed first; no world-specific hard-coding; every acceptance criterion names a
  second world or a generated variant.
- Operator memory: "Realistic defaults" (cite the source of every physical number); "All platforms".

## 1. Scope

**In:**

1. **Shared module** (`clients/shared/sitting/`):
   - `sit_rule.gd` gains `pose_of(place_tags, disclosed, point)` and `pelvis(at, facing, offset)`, moved
     from the 3D client's `SliceSeating` (§5.1). Both clients then place a seated pelvis by one function.
   - New `sit_chain.gd` (`MineWorldSitChain`): the walk-then-sit chain and the stand-before-moving step
     (§5.2). It names no action type and submits nothing.
   - A headless check `clients/shared/checks/sit_chain_check.gd`.
2. **2D drawing** (`clients/2d/scripts/scene/people.gd`, `scene/seating.gd`, `app.gd`): a person the shared
   decision seats is drawn at the pelvis point, in the view chosen from the point's facing, with the stride
   off (§5.3). Seated sprite: `<base>_seated` when the pack binds one; else the standing frame cropped at the
   hip line and lowered onto the seat (§5.4).
3. **2D input** (`app.gd`, `scene/seating.gd`, `intents.gd`): a click on a disclosed sit point runs the
   chain; a floor click or a walk key while seated sends the offered `stand` first (§5.5).
4. **Rule scan entry** (`tests/acceptance/tests/client_rules.rs`): only `clients/shared/sitting/` may read
   posture's component types (§5.6). Two `ACTION_LITERALS` admissions for `intents.gd`.
5. **3D delegation, one file** (`clients/3d-spike/scripts/slice/seating.gd`): `pose_of` and `pelvis` call the
   shared ones (Q-S34e-4). No 3D behaviour change.
6. **Depictions of the café's chairs and stools** (Q-S34e-3): `presentation/mineworld-default/3D/sit_points/
   social-cafe.yaml` and `market-town.yaml`, one row per sit point that has a disclosed furnishing, with a
   test that recomputes each offset from the world's own data.
7. **Tests and fixtures** (§8): a posture-branch observation fixture, a seeded generated variant, the 2D
   `DRAWN` lines, and a live 2D scenario on Social Café and Market Town.
8. **Docs:** `clients/2d/PRESENTATION.md` §3 (the seated sprite and the fallback), `clients/shared/README.md`
   (the chain), `clients/3d-spike/PRESENTATION.md` §10 (one line: the pose is the shared `pose_of`),
   notes on `ARC-145` and `ARC-147`.

**Out:**

- Rendering the `<base>_seated` sheets for the cast (Blender through `cast.py`): S34-e2 if Q-S34e-2 is ruled
  "split" (recommended); otherwise a last commit here.
- Any server, System Pack, world or contract change. Posture, bodies and movement are untouched.
- 3D sitting, the 3D player's E, the drawn-vs-disclosed check, the `--seat` probe on server points,
  park and street bench depictions, `VIS-POSTURE`: S34-f.
- The 2D menu's wording of `sit` entries (one "Sit down" per point stays as it is).
- Station halls: their places are not on main (S34-c scope note).

## 2. Source audit (`origin/main @ 0b2faf2f`)

| # | Fact | Where |
| --- | --- | --- |
| 1 | **The shared module exists, with the override branch.** `MineWorldSitRule.decide` returns the disclosed `posture-state` points when the place discloses `sit-points` (l. 108–113), else SD-S33-7's fallback (l. 114–136). Constants `SIT_NEAR_M` 0.40, `SIT_STILL_S` 2.0 (l. 27–28). `people_of` reads `posture-state` (l. 156); `decide_observed` reads `sit-points` (l. 172) | `clients/shared/sitting/sit_rule.gd` |
| 2 | **Depictions are loaded by the rule.** `load_for(pack_dir, world_key)` reads `<pack>/sit_points/<world>.yaml` (l. 46–75); a missing `seat_offset_m` defaults to 0.45 and `seat_height_m` to 0.45 (l. 73–74); `depiction(tags, point)` (l. 79–83). Only `lakeside.yaml` exists | `sit_rule.gd`; `presentation/mineworld-default/3D/sit_points/` |
| 3 | **The 2D client decides but does not draw.** `Seating.reconcile` stores `rule.decide_observed(...)` (l. 24–26); its header says drawing is S34-e's (l. 3–5). `app.gd` calls it after `people.reconcile` (l. 535) and uses nothing it returns | `clients/2d/scripts/scene/seating.gd`; `clients/2d/scripts/app.gd` |
| 4 | **The 3D client places the pelvis itself.** `SliceSeating.pose_of` merges the disclosed point with the depiction and returns `{}` when there is no depiction (l. 47–62); `pelvis` moves `at` against the facing (l. 65–67); it reads `"sit-points"` itself (l. 34) | `clients/3d-spike/scripts/slice/seating.gd` |
| 5 | **2D people.** `_make` builds a walk sheet (`<base>_walk`, rows `down`…`up`) or four stride views (l. 284–330); `_animate` / `_animate_sheet` choose the frame from travel (l. 397, 438); `_turn8` maps a plan direction to one of eight facings (l. 473); `pick` (l. 226). No seated frame; `facing` is never read | `clients/2d/scripts/scene/people.gd` |
| 6 | **2D click order.** `open_menu_at`: a person's figure, then a drawn object, then a walk (l. 428–444) | `app.gd` |
| 7 | **Walker.** `walk_to(place, plan)` sends one `walk-to`; `signal walk_ended(completed)` (l. 23); keys send `move` | `clients/2d/scripts/walker.gd` |
| 8 | **Intents.** The only 2D script that submits; `submit_offered(affordance)` sends a complete offer as offered (l. 96–99); `COMPOSED` lists composed types only (l. 18) | `clients/2d/scripts/intents.gd` |
| 9 | **Offers.** Posture offers, to a standing observer, one complete target-less `sit { point }` per point of the place, each refused with the reason dispatch would give; to a seated one, one complete `stand {}` (l. 25–57). The disclosed `sit-points` value is `[{key, at: {x, y}, facing, group?}]`; `posture-state` is `{seated: {place, point, since}}` | `systems/posture/src/perception.rs`; `point.rs` l. 80–110; `component.rs` l. 22–35 |
| 10 | **Verdict lookup.** `MineWorldObservation.affordances(type, target)`, `complete_affordances`, `unavailable_reason` (string `too_far_away` …) | `clients/protocol/mineworld/observation.gd` l. 200–284 |
| 11 | **Scans.** `client_rules.rs` check (a): every pack action type as a GDScript literal needs an `ACTION_LITERALS` admission (`sit` and `stand` are admitted only in 3D files, l. 186–208); `RULE_WORDS` already holds `reach` (l. 222); `no_client_holds_a_copy_of_the_sit_rule` (l. 754–800). `scripts/check_client_rules.py` R3: no verdict read in a submitting function of `intents.gd`, `menu.gd`, `hud/**`; R4: `distance_to` only in `walker.gd`, `projection.gd`, `town.gd`, `scene/**`, `harness/**` | `tests/acceptance/tests/client_rules.rs`; `scripts/check_client_rules.py` |
| 12 | **Recorded-observation check.** `client_sit_rule.rs` feeds `contested-long-table.json` (Lakeside, fallback branch) to both clients' `seat_check.gd` and compares `SEATED` lines | `tools/cli/tests/client_sit_rule.rs`; `clients/2d/scripts/harness/seat_check.gd` |
| 13 | **Worlds with posture.** `social-cafe` and `market-town` enable `posture` (`world.yaml` l. 72, l. 125). Their café points: stools and chairs; the chair points sit behind the chair on the aisle side (Q-S34c-2), e.g. `table-1.1` at (3648, 3060), its chair `table.east.chair-w` at (4200, 3060), facing 81 406 | `worlds/social-cafe/places/cafe.yaml` l. 50–64, 129–138 |
| 14 | **Wording exists.** `action.sit` "Sit down", `action.stand` "Stand up", `.done` lines (#337, #338) | `clients/shared/settings/locale/en.po` l. 703–714 |
| 15 | **Walk sheets** are rendered by `cast.py walk-sheets` from the 3D client's poses (rows `down`…`up`, column 0 standing) | `tools/character_pipeline/PIPELINE.md` §19 |
| 16 | **Open PRs on the same files.** #366 (S29-f: `furnishings.gd`, `places.gd`, `town.gd`, `drive.gd`, `clients/shared/README.md`, locale files); #460 (16f-d2: `client_rules.rs`); #426 (locale files); #496 (3D `slice_layout.gd`, `shown_parity.rs`) | `gh pr list` 2026-10-26 |

## 3. Findings

- **F-e-1. The override branch is done.** S33-f landed first and wrote it (row 1). S34-e does not rewrite it.
  What is left of the step's S34-e row: the chain, 2D drawing, 2D input, the scan entry.
- **F-e-2. The pelvis rule lives in the 3D client only** (row 4). If 2D wrote its own, the two clients could
  draw one person at two places. It moves to the shared module (Q-S34e-4).
- **F-e-3. Café chairs need a depiction.** A chair's sit point is 0.55 m behind the chair (row 13). Without a
  depiction the pelvis would be drawn behind the chair. Only Lakeside has depictions (row 2). (Q-S34e-3.)
- **F-e-4. The chain must read one verdict.** "Walk there first" needs `too_far_away`. R3 forbids that read in
  `intents.gd`, `menu.gd` and `hud/**` (row 11), so it lives in the shared chain, called from `app.gd`.
- **F-e-5. No seated sprite exists** (rows 5, 15). Rendering one per cast body is Blender work (Q-S34e-2).
- **F-e-6. Clicking oneself opens one's own menu today**, which holds `sleep` and the other self offers. The
  step's text says the click submits `stand`. That would hide the menu while seated (Q-S34e-1).

## 4. Invariants

| Id | Invariant | Held by |
| --- | --- | --- |
| I-e-1 | **Drawn = decided.** The 2D client draws a person seated iff the shared decision seats them, at that point (I-S34-4) | `DRAWN` vs `SEATED` lines (AC-e-1, AC-e-2) |
| I-e-2 | **One pose function.** Both clients place a seated pelvis by `MineWorldSitRule.pose_of` | the scan (AC-e-6); 3D delegation |
| I-e-3 | **The chain decides nothing.** It reads one server verdict (`too_far_away`) to choose "walk first"; it never measures a distance, never retries, and sends each offer exactly as offered (I-S34-5) | `sit_chain_check.gd`; AC-e-3, AC-e-7 |
| I-e-4 | **Only the shared module reads posture.** No client script outside `clients/shared/sitting/` holds `"posture-state"` or `"sit-points"` | AC-e-6 |
| I-e-5 | **No world in client code.** No script names a world, place, point key or tag | existing scans; the generated variant (AC-e-2) |
| I-e-6 | **Server untouched.** No file under `systems/`, `kernel/`, `contracts/`, `worlds/`, `server/` changes; every baseline is unchanged | AC-e-8 |

## 5. Design

### 5.1 One pose function (shared) — SD-e-1

`sit_rule.gd` gains, moved from `SliceSeating` with its tests:

```text
pose_of(place_tags, disclosed, point) -> { pelvis: Vector2 mm, at: Vector2 mm, facing: int mdeg,
                                           seat_height_m: float, prop: String, depicted: bool } or {}
    at, facing     the disclosed point's when the place discloses sit-points, else the depiction's fallback
    seat_*         the depiction's when one binds the point; else seat_height_m 0.45 (VISUAL_SLICE §3.4),
                   offset 0.0 (the pelvis drawn at `at`), depicted false
    {}             no position at all (no disclosed point and no fallback)
static pelvis(at, facing, offset_m) -> Vector2      unchanged code
```

- The 3D `SliceSeating.pose_of` becomes a one-line call. It keeps today's behaviour: when `depicted` is false
  it unseats the figure, as its `{}` did. 2D draws an undepicted point at `at`.
- An undepicted point is drawn at `at` because that is the only position the world gives. It is off by the
  seat depth (≈ 0.26 m for a bench). The café's chairs get depictions (§5.7), so the large case is gone.

### 5.2 The chain (shared) — SD-e-2

`clients/shared/sitting/sit_chain.gd`, `class_name MineWorldSitChain`. Built with the action types the client
names: `MineWorldSitChain.new(sit_type, stand_type)`. So the shared module names no action, as the trade
module's precedent (`TRADE_INPUTS`) requires.

```text
choose(observation, point) -> answer
    the frame's complete target-less offer of sit_type whose payload.point == point
      none                                  -> {}                       (no posture here, or no such point)
      unavailable_reason == "too_far_away"  -> {walk: {place, local: at}}, remember (place, point)
      anything else                         -> {submit: offer}          (sent as offered; the server answers)
walk_ended(observation, completed) -> answer
    a remembered point, completed, and the observer still in that place
                                            -> {submit: the newest offer for that point}, forget it
    otherwise                               -> {}, forget it
leave(observation) -> answer
    the frame offers a complete stand_type  -> {submit: offer}
    otherwise                               -> {}
cancel()                                    forget any remembered point
```

- **One try.** After the walk, the newest offer is sent whatever its verdict. A refusal (taken meanwhile,
  stopped short beyond reach) is shown as a toast. Nothing is retried (`INV-6`).
- **Walk target.** The point's disclosed `at`. The server plans the route; bodies keeps the walker off solids.
- **Cancel** on: a floor click, a walk key, another menu choice, a place change, a dropped connection.
- The 3D client (S34-f) calls the same three functions from look+E. So "walk then E" and "click" send the
  same two intents (step §8.4, §9.1).

### 5.3 Drawing a seated person in 2D — SD-e-3

`app.gd` passes the decision to `people.gd` after `people.reconcile`:

```text
for each drawn person id:
    point = seating.seated.get(id)
    pose  = point ? rule.pose_of(tags, disclosed sit-points, point) : {}
    people.set_seated(id, pose_in_plan or null)
```

`people.set_seated(id, pose)`:

- **Position.** The target and the drawn position become the pelvis point, converted with `town.to_plan`.
  The person snaps there (no lerp), as a sitter settles at once.
- **View.** The facing becomes a plan direction: `to_plan(at + 1000 mm · (sin yaw, cos yaw)) − to_plan(at)`.
  It is mapped to a view by the same mapping `_turn8` uses (refactored into a static `_view_of(direction)`),
  so a seated and a walking person use one angle rule. Four-view sets use front/back and the side sign.
- **Stride off.** `_animate` skips a seated person: no bob, no rock, no gait frame.
- **Captions.** Name, marker, Zz and tag move down by the height the figure lost, so they sit over the head.
- **Back to standing.** `set_seated(id, null)` restores the standing frame and the travel-driven facing.

### 5.4 The seated sprite and its fallback — SD-e-4 (Q-S34-10; Q-S34e-2, Q-S34e-5)

- **Seated sheet.** A sixth id on the person's base, `<base>_seated`: one atlas, rows `down`, `down_right`,
  `right`, `up_right`, `up` (the walk sheet's rows), one column. Fields as a walk sheet, plus
  `seat_height_m`, the seat height it was rendered on. Its `anchor` is the floor point under the pelvis. The
  client draws it at the pelvis point, raised by `(pose.seat_height_m − sheet.seat_height_m) × px per metre`.
- **Fallback.** No seated sheet: the standing frame (walk sheet column 0, or the `front` / `back` view) is
  cropped at the hip line and lowered so the hip line lies `seat_height_m` above the floor point. The hip
  line is `0.530 × height_m` above the anchor: the greater trochanter's height as a fraction of stature
  (Drillis and Contini 1966, as tabulated in D. A. Winter, *Biomechanics and Motor Control of Human
  Movement*, 4th ed., Fig. 4.1). A sprite without `height_m` uses its opaque height.
- **Plain drawing** (no pack, or an unbound role): the plain shape is drawn at the pelvis point, unchanged.

### 5.5 Input — SD-e-5

- **Click order** in `open_menu_at`: a person's figure, a drawn object, **a sit point**, then a walk.
- **Picking a sit point.** `seating.pick(at_screen)` returns the key of the disclosed point of the observer's
  place whose drawn pelvis point lies within `SIT_PICK_PX` (12 px at zoom 1, drawing only, like
  `PICK_HALF_WIDTH_PX`) of the click, the nearest one. It lives in `scene/` (R4).
- **Click on a sit point:** `answer = chain.choose(latest, key)`; `{submit}` → `intents.submit_offered`;
  `{walk}` → `walker.walk_to(place, to_plan(at))`. `walker.walk_ended(completed)` →
  `chain.walk_ended(latest, completed)` → `intents.submit_offered` when it answers.
- **Leaving the seat.** While the observer is seated, a floor click or the first walk key sends
  `chain.leave(latest)`'s offer first, then the walk. Both go in order, so the server stands the person up by
  request (`stood-up { Requested }`) before the move. If no `stand` is offered, the move alone auto-stands
  them on the server (§6.4 of the step).
- **Click on oneself** opens one's own menu, which lists the offered `Stand up` (Q-S34e-1).
- `intents.gd` gains `const SIT := "sit"` and `const STAND := "stand"`, read by `app.gd` to build the chain.
  It composes neither: both are complete offers.

### 5.6 The scan entry — SD-e-6

A new test in `client_rules.rs`, beside `no_client_holds_a_copy_of_the_sit_rule`:

```text
no_client_reads_posture_but_the_shared_sit_module
    every client script outside clients/shared/sitting/ (comments not read) holds no string literal
    equal to a component type the posture pack declares (read from systems/posture/src: the
    `component_type = "…"` lines, so a renamed or added type is covered); an empty set fails
```

`ACTION_LITERALS` gains `(clients/2d/scripts/intents.gd, "sit", …)` and `(…, "stand", …)`. The posture
`REACH` is already a rule word (`reach`), so no new rule word is needed.

### 5.7 Depictions of the café's seats (Q-S34e-3) — SD-e-7

- New files `sit_points/social-cafe.yaml` and `sit_points/market-town.yaml`, format 1, scope the world.
- One row per sit point whose seat is a disclosed furnishing (the café's three stools and six chairs):
  `tags` the café's, `point` its key, `prop` the furnishing key, `seat_offset_m` = the furnishing's `at`
  minus the point's `at`, projected **against** the point's facing (signed, metres, 3 decimals),
  `seat_height_m` the furnishing kind's seat surface as the 3D prop's manifest states it, else 0.45 with a
  `source` note.
- Park and street benches have no disclosed furnishing. They stay undepicted (drawn at `at`) until S34-f.
- A test (`tools/cli/tests/sit_point_depictions.rs`) runs over **every** depiction file: each row's `prop`
  names a furnishing of a place carrying its tags, the recomputed offset matches within 10 mm, and the
  sideways error is under 50 mm. It names no world: it lists `sit_points/*.yaml`.
- Effect on 3D: `SliceSeating` starts drawing café sitters seated (depicted now). That is the intended
  direction; its seat contact is S34-f's `--seat` gate.

## 6. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Worlds / variant |
| --- | --- | --- |
| AC-e-1 | (AC-S34-10, first half.) For every frame of the posture fixture, the 2D client prints one `DRAWN <at> <id> seated <point> <view> <x> <y>` per person the shared decision seats, and `standing` for everyone else. Each seated line's position equals `pose_of`'s pelvis within 1 mm, and its view equals the view of the point's facing | a recorded Social Café fixture, and the same fixture from **Market Town**; plus the seeded generated variant (AC-e-2) |
| AC-e-2 | (AC-S34-10, second half; X-S34-5.) A person standing still for 5 s of world time on a point's `at`, with no `posture-state`, is drawn standing in a place that discloses `sit-points` | the fixture, and a **generated variant**: the Rust test renames every point key and place tag through a seeded map and mirrors x; the decisions are equal under the map |
| AC-e-3 | **Click, walk, sit, stand.** Live, hosted: the 2D harness clicks the nearest sit point offered `too_far_away`. The journal holds one `walk-to`, then one `sit` whose semantic core equals the offered affordance (`differing_fields` empty but `ActorLocation`); `sat-down` follows; the observer's `posture-state` is disclosed and the observer is drawn seated. A floor click then sends `stand` before `walk-to`; `stood-up { Requested }` is recorded; the figure is drawn standing. A second click on the same point from within reach sends `sit` with no `walk-to` | **Social Café** and **Market Town** |
| AC-e-4 | (AC-S34-8, 2D half.) The 2D `sit` request of AC-e-3 is recorded in the transcript format `ac13_clients.rs` reads, keyed by the point. The 3D half and the comparison land in S34-f | Social Café, Market Town |
| AC-e-5 | **Fallback drawn.** On `contested-long-table.json` (Lakeside, no posture) the 2D client draws person 7 seated at `long-table.3` from at 102 on, and nobody else; `client_sit_rule.rs` stays green | **Lakeside** |
| AC-e-6 | **The scan.** `no_client_reads_posture_but_the_shared_sit_module` passes; planting `"posture-state"` in a 2D or 3D script fails it naming file and line. The existing I-f-6 test still passes | world-free: it reads no world, so it holds for every world and every variant |
| AC-e-7 | **A taken seat.** Live: a second hosted seat sits on point P first; the 2D player clicks P from within reach. Exactly one `sit` is sent; it is answered `target_unavailable`; a toast shows the server's reason; no further `sit` or `walk-to` follows | **Market Town** (two seats), and Social Café |
| AC-e-8 | **Server untouched.** The PR's diff names no file under `systems/`, `kernel/`, `contracts/`, `server/`, `worlds/`; `scripts/baselines.txt` is unchanged | every world (no world changes) |
| AC-e-9 | **Depictions agree with the world.** `sit_point_depictions.rs` passes over every depiction file | Social Café, Market Town, Lakeside (rows with a disclosed prop) |

## 7. Planted mutations (each turns a named test red, then is reverted)

| Id | Mutation | Expected red |
| --- | --- | --- |
| X-e-1 | (X-S34-5) `seating.gd` passes `null` for the disclosed `sit-points` | AC-e-2 (still person drawn seated) |
| X-e-2 | the chain walks whatever the verdict | AC-e-3's within-reach click (an extra `walk-to`) |
| X-e-3 | the chain re-sends `sit` after a refusal | AC-e-7 (two `sit` requests) |
| X-e-4 | `people.gd` keeps the travel view for a seated person | AC-e-1 (view differs from facing) |
| X-e-5 | `pelvis` moves `at` along the facing instead of against it | AC-e-1 position; AC-e-9 |
| X-e-6 | a 2D script reads `"posture-state"` directly | AC-e-6 |
| X-e-7 | `const SIT_NEAR_M` copied into `people.gd` | the existing I-f-6 test |

Speed-first charter: the mutations are run at the milestone, not per commit. The ledger records each.

## 8. Test ownership

| Failure class | Owner | Why here |
| --- | --- | --- |
| Chain logic (walk first, one try, cancel, leave) | `clients/shared/checks/sit_chain_check.gd`, headless, recorded frames | cheap, deterministic, local |
| Drawn ≠ decided; view; pelvis position | `seat_check.gd` `DRAWN` lines, compared by `client_sit_rule.rs` | recorded observations, no server |
| Fallback branch on Lakeside | `client_sit_rule.rs` (existing) + AC-e-5 | existing owner |
| Request equals offer; journal facts | `tools/cli/tests/client_2d_sit.rs` (`#[ignore]`, Godot, hosted world) | real lifecycle; runs at the milestone |
| A client reading posture itself | `client_rules.rs` new test | static, structural |
| Depiction data drift | `sit_point_depictions.rs` | data check, no Godot |
| Sprite rendering of seated sheets | S34-e2 (`test_walk_sheets.py` pattern) | out of this PR |

No test asserts a getter, a constant or an enum.

## 9. Commit plan

Each commit has **Implementation**, **Validation**, **Review**, each `[ ]` until done with evidence.

### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_doc_headings.py`.
- [ ] Review: scope equals the step's S34-e row minus F-e-1, plus §13's answers.

### C1 — Shared: one pose function and the chain
- **Files:** `clients/shared/sitting/sit_rule.gd` (`pose_of`, `pelvis`), new `sit_chain.gd` (+ `.uid`),
  new `clients/shared/checks/sit_chain_check.gd`, `clients/3d-spike/scripts/slice/seating.gd` (delegation),
  `clients/shared/README.md`.
- [ ] Implementation: §5.1, §5.2. 3D keeps `depicted == false` → unseat.
- [ ] Validation: `godot --headless --path clients/shared --script res://checks/sit_chain_check.gd` PASS
  (cases: in reach, too far, other refusal, no offer, walk incomplete, place changed, seated → leave);
  `client_sit_rule.rs -- --ignored` still equal (3D unchanged). Through `with-slot.sh`.
- [ ] Review: the chain names no action type, measures nothing, retries nothing.

### C2 — 2D drawing
- **Files:** `clients/2d/scripts/scene/people.gd` (`set_seated`, `_view_of`, seated sprite, fallback crop,
  captions), `scene/seating.gd` (pose per id), `app.gd` (wiring), `scripts/presentation.gd` (`<base>_seated`
  lookup in `directional`), `clients/2d/PRESENTATION.md` §3.
- [ ] Implementation: §5.3, §5.4.
- [ ] Validation: `seat_check.gd` prints `DRAWN` lines on the new fixtures; the existing 2D drive scenarios
  (`walk`, `strides`, `idle`) unchanged; `python3 scripts/check_client_rules.py` PASS.
- [ ] Review: no world, place, key or tag in code; the hip ratio cited; R4 respected.

### C3 — 2D input
- **Files:** `app.gd` (click order, chain calls, leave-before-move), `scene/seating.gd` (`pick`),
  `walker.gd` (a `setting_off` signal for the first key move, if needed), `intents.gd` (`SIT`, `STAND`),
  `tests/acceptance/tests/client_rules.rs` (two admissions).
- [ ] Implementation: §5.5.
- [ ] Validation: `check_client_rules.py` PASS (R1–R8); `cargo test -p mineworld-acceptance --test
  client_rules` PASS; a hand run on Social Café: click a stool, walk, sit, click the floor, stand.
- [ ] Review: verdict read only in the shared chain; nothing re-sent.

### C4 — Scan, fixtures and tests
- **Files:** `client_rules.rs` (new test §5.6), `tests/fixtures/observations/sitting/posture-cafe.json`
  (+ Market Town frames), `clients/2d/scripts/harness/seat_check.gd` (`DRAWN`), `tools/cli/tests/
  client_sit_rule.rs` (posture fixture, seeded variant, `DRAWN` checks), new `tools/cli/tests/
  client_2d_sit.rs`, `clients/2d/scripts/harness/interact.gd` (a `{"click_sit": "nearest-far" | key}` step).
- [ ] Implementation: AC-e-1…e-7 tests.
- [ ] Validation: each new test green once locally (ignored ones with `-- --ignored`, through
  `with-slot.sh`); X-e-1…7 recorded as milestone items.
- [ ] Review: tests choose points from disclosure, never from a world literal.

### C5 — Café depictions (only if Q-S34e-3 is "yes")
- **Files:** `presentation/mineworld-default/3D/sit_points/{social-cafe,market-town}.yaml`, new
  `tools/cli/tests/sit_point_depictions.rs`.
- [ ] Implementation: §5.7.
- [ ] Validation: `sit_point_depictions.rs` PASS over all three files; X-e-5 against it.
- [ ] Review: every offset traced to two disclosed `at`s; every height to a source.

### C6 — Records and ledger
- [ ] Implementation: dated notes on `ARC-145` (2D draws seated; one pose function; the chain) and
  `ARC-147` (S34-e landed); this file's ledger.
- [ ] Validation: `python3 -I scripts/check_decision_ids.py --write-index`; `check_doc_headings.py`.
- [ ] Review: no frozen text rewritten; notes are dated.

## 10. Execution contract

```text
PROJECT / PR:           MineWorld · S34-e (S34-e1)
PRIMARY DESIGN DOC:     .structured-coding/plans/mvp0/pr-s34e-2d-seated.md
BINDING DOCS:           step-34-posture.md; step-33-lakeside-3d.md §4.5, §14.2; pr-s33f-interiors-sit-points.md;
                        CLAUDE.md; docs/ENGINEERING_RULES.md; clients/2d/PRESENTATION.md
Implementer:            Haiku (Q-S34-14), from this frozen commit plan; Opus reviews the diff
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34e-2d-seated
Branch / base:          mvp0/pr-s34e from origin/main after this design freezes
Approved scope:         §1 "In" as answered in §13
Frozen invariants:      §4; step I-S34-4, I-S34-5, I-S34-6
Authorized (on freeze): implement, validate locally (cast-slots/with-slot.sh for Godot and cargo), semantic
                        commits, push, open the PR — source: lane charter 2026-10-18
Validation budget:      the new checks and tests once each, plus check_client_rules.py and client_rules;
                        full suites, Godot-ignored tests and mutations at the milestone (speed-first charter)
Spend:                  none
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges; never this session
Escalate:               any server, world or contract change; a change to 3D behaviour beyond §1 item 5 and
                        §5.7's effect; a frozen-step deviation not listed in §13
```

## 11. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | #366, #426 and #460 edit `client_rules.rs`, `clients/shared/README.md` and locale files | additive edits only; no locale change needed (row 14); rebase on conflict |
| R-e-2 | The crop fallback looks poor | it is the ruled fallback (Q-S34-10); S34-e2 renders real seated sheets |
| R-e-3 | 2D drive checks of figure height (AC-W12) read a seated observer | the drive's checks run while standing; a seated person is reported as such in `DRAWN` |
| R-e-4 | The live test is slow or flaky (walk timing) | bounded steps (`STEP_LIMIT_S`); `#[ignore]`, run at the milestone |
| R-e-5 | Café depictions make 3D draw café sitters before S34-f's seat gate | intended direction; S34-f's `--seat` probe owns contact; Q-S34e-3 lets the operator defer |
| R-e-6 | A walk stops short beyond reach (bodies), so the sit is refused | one try by design; the toast says why; the player clicks again |

## 12. Evidence ledger

(empty until implementation)

## 13. Operator questions (each with a recommended answer)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34e-1** | While seated, should clicking **oneself** send `stand` at once (the step's §8.4 text), or open one's own menu, which already lists "Stand up" beside sleep and the other self actions? | **Open one's own menu.** A floor click or a walk key still stands the player up first. Recorded as a bounded deviation from §8.4's wording |
| **Q-S34e-2** | Real seated sprites need a new `cast.py` render of every cast body's sit pose (Blender, local slots, no money). Do it here, or in a follow-up S34-e2, with the cropped fallback until then? | **Split into S34-e2.** S34-e1 ships drawing, input and the fallback now; S34-e2 renders `<base>_seated` sheets for every cast body and NPC alike |
| **Q-S34e-3** | Café chair points sit 0.55 m behind the chair, and no café depiction exists, so 2D would draw the sitter behind the chair. Write `sit_points/social-cafe.yaml` and `market-town.yaml` here (offsets computed from the world's own furnishing and point positions, checked by a test)? 3D will then also draw café sitters seated | **Yes**, chairs and stools only. Park and street benches stay undepicted until S34-f |
| **Q-S34e-4** | Move the 3D client's `pose_of` / `pelvis` into the shared sit rule, with a one-line delegation in `slice/seating.gd`, so both clients place the pelvis by one function? | **Yes.** No 3D behaviour changes; S34-f builds on it |
| **Q-S34e-5** | Fallback crop line: hip at 0.530 × stature (Drillis and Contini, via Winter Fig. 4.1)? | **Yes**, used only until a body has a seated sheet |
