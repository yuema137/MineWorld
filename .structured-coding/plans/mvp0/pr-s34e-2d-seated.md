# PR S34-e — the shared sit rule and seated people in 2D

## Lifecycle

```text
Design revision:        revision 1 (2026-10-26), DESIGN FROZEN 2026-10-27 (operator approved in the primary session: every question answered as recommended)
Approved by / evidence: operator ruling 2026-10-27 (§13), recorded in legacy commit 6736d63e; recovery dispatch 2026-11-22
Implementation base:    origin/main @ 0b2faf2f (after #301 S34-a, #363 S34-b, #368 S34-c1, #462 S33-f)
Start condition:        met; recovery from reviewed local base listed in §12
Execution contract:     §10
Lifecycle:              LOCAL REVIEW READY; execution closed pending coordinator integration (recovery closeout below)
```

**Dated note, 2026-10-27 (operator ruling Q-S34c3-3; SD-c3-5 in
[`pr-s34c3-station-sit-points.md`](pr-s34c3-station-sit-points.md) §5.5).** Posture will offer `sit`
only for points within 5 m (Q-S34c3-1). A far seat is then disclosed in the place's sit points but has no
offer. The shared chain (SD-e-2) changes one row: `none -> {}` becomes
`none, and the point is disclosed -> {walk: {place, local: at}}`, then the newest offer is used on arrival.
`none` for a point that is not disclosed stays `{}`. The client measures no distance (I-S34-4): "not
offered" is the server's answer. S34-f reuses this chain unchanged. Add one test arm for the new row.

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
Current completion marks are reconciled against
the exact-head independent reviews and the evidence mapping in the recovery closeout below.
They establish local completion only; publication, canonical CI and merge are not complete.

### C0 — Design (this file), docs only
- [x] Implementation: this file.
- [x] Validation: `python3 -I scripts/check_doc_headings.py` (PASS; recovery ledger below).
- [x] Review: scope equals the step's S34-e row minus F-e-1, plus §13's answers and approved S34-B amendment.

### C1 — Shared: one pose function and the chain
- **Files:** `clients/shared/sitting/sit_rule.gd` (`pose_of`, `pelvis`), new `sit_chain.gd` (+ `.uid`),
  new `clients/shared/checks/sit_chain_check.gd`, `clients/3d-spike/scripts/slice/seating.gd` (delegation),
  `clients/shared/README.md`.
- [x] Implementation: §5.1, §5.2. 3D keeps `depicted == false` → unseat.
- [x] Validation: `godot --headless --path clients/shared --script res://checks/sit_chain_check.gd` PASS
  (cases: in reach, too far, other refusal, no offer, walk incomplete, place changed, seated → leave);
  `client_sit_rule.rs -- --ignored` still equal (3D unchanged). Through `with-slot.sh`.
- [x] Review: the chain names no action type, measures nothing, retries nothing.

### C2 — 2D drawing
- **Files:** `clients/2d/scripts/scene/people.gd` (`set_seated`, `_view_of`, seated sprite, fallback crop,
  captions), `scene/seating.gd` (pose per id), `app.gd` (wiring), `scripts/presentation.gd` (`<base>_seated`
  lookup in `directional`), `clients/2d/PRESENTATION.md` §3.
- [x] Implementation: §5.3, §5.4.
- [x] Validation: `seat_check.gd` prints `DRAWN` lines on the new fixtures; the existing 2D drive scenarios
  (`walk`, `strides`, `idle`) unchanged; `python3 scripts/check_client_rules.py` PASS.
- [x] Review: no world, place, key or tag in code; the hip ratio cited; R4 respected.

### C3 — 2D input
- **Files:** `app.gd` (click order, chain calls, leave-before-move), `scene/seating.gd` (`pick`),
  `walker.gd` (a `setting_off` signal for the first key move, if needed), `intents.gd` (`SIT`, `STAND`),
  `tests/acceptance/tests/client_rules.rs` (two admissions).
- [x] Implementation: §5.5.
- [x] Validation: `check_client_rules.py` PASS (R1–R8); `cargo test -p mineworld-acceptance --test
  client_rules` PASS; real locally hosted automated client lifecycle on Social Café and Market Town:
  click a disclosed seat, walk, sit, click the floor, stand. No separate manual playtest is claimed.
- [x] Review: verdict read only in the shared chain; nothing re-sent.

### C4 — Scan, fixtures and tests
- **Files:** `client_rules.rs` (new test §5.6), `tests/fixtures/observations/sitting/posture-cafe.json`
  (+ Market Town frames), `clients/2d/scripts/harness/seat_check.gd` (`DRAWN`), `tools/cli/tests/
  client_sit_rule.rs` (posture fixture, seeded variant, `DRAWN` checks), new `tools/cli/tests/
  client_2d_sit.rs`, `clients/2d/scripts/harness/interact.gd` (a `{"click_sit": "nearest-far" | key}` step).
- [x] Implementation: AC-e-1…e-7 tests (C4 continuation evidence below).
- [x] Validation: each new test green once locally (ignored ones with `-- --ignored`, through
  `with-slot.sh`); X-e-1…7 recorded as milestone items.
- [x] Review: tests choose points from disclosure, never from a world literal.

### C5 — Café depictions (only if Q-S34e-3 is "yes")
- **Files:** `presentation/mineworld-default/3D/sit_points/{social-cafe,market-town}.yaml`, new
  `tools/cli/tests/sit_point_depictions.rs`.
- [x] Implementation: §5.7 as superseded by the approved S34-B amendment below.
- [x] Validation: approved replacement oracle PASS over all 47 rows; four vector-axis mutations
  fail actual placement. Original X-e-5 proves nonzero legacy scalar placement (AC-e-1/5),
  as explicitly recorded in C4; it is not relabelled as a new AC-e-9 mutation.
- [x] Review: vector offsets/heights trace to the independently measured anchors and source
  triangles under approved S34-B; legacy scalar outputs remain exact.

### C6 — Records and ledger
- [x] Implementation: dated notes on `ARC-145` (2D draws seated; one pose function; the chain) and
  `ARC-147` (local S34-e implementation, not merge); this file's ledger.
- [x] Validation: `python3 -I scripts/check_decision_ids.py --write-index`; `check_doc_headings.py`.
- [x] Review: no frozen text rewritten; notes are dated.

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

### Recovery — 2026-11-22

This session owns `/tmp/mineworld-recover-s34e`, branch `recover/s34e-2d-seated`, based on
`003d35437d6b7f873db1a2c32a29f0b6f554cb58`, the independently reviewed local F-RC-7 stack
(base `4cd160bb`). This prerequisite is **not claimed merged**. The coordinator explicitly
assigned this base and implementation ownership on 2026-11-22. A separate local clone shares
read-only objects because sandbox permissions deny writing the main checkout's Git metadata.
The legacy checkout remains read-only at `6736d63e07b57d75cd703d1e9abe11232b94ff33`.
Its committed patch, staged/dirty patches, tracked tree and all 14 changed/untracked files are
preserved at `/tmp/mineworld-s34e-legacy-snapshot`; SHA256 of `snapshot.sha256.json`:
`8fa5745ca7f30b67f5f7ecfd253cbe014bd63d07d0fee8ae9e3470c01e39f665`.

Endpoint authority: implementation, targeted local validation and semantic commits authorized
by the recovery dispatch; no public writes, merge, subagents, dashboard or shared-root plan
edits. The primary coordinator owns decisions, parallel lanes and independent red-team review.
This supersedes the historical implementer, publication and review assignments below, not the
frozen scope, invariants or acceptance. S34-e2 sprites remain separate; no spend, new content
or CMU usage. Live evidence is maintained here in the isolated checkout, not the shared root.

Current checkpoint: C1 shared chain/pose check PASS; cross-client recorded check pending C4. The legacy chain
omitted the October 27 disclosed-but-unoffered walk row; restored it and added checks for that
row, newest refused offer and one-shot consumption. The shared pose preserves 3D's depicted-only
behavior. The probe edit only replaces a direct disclosure read with the equivalent shared
accessor; F-RC-7 assertions, logger and helpers are unchanged. Later legacy drawing/input needs
repair for unknown stature, updated poses, restoration and key-cancellation ordering.

Validation envelope: smallest focused checks, Godot/cargo through the existing machine-wide
`cast-slots/with-slot.sh`; no priority override; no full suite or asset import. Each real client
run targets <=10 minutes, total real runtime <=1 hour; no paid/API calls. Gate 1 not required
(no LLM-facing behavior). Next: run C1 headless checks, review and commit, then C2/C3 drawing/input,
C4 recorded/live evidence and mutations, C5 depictions, C6 documentation. Stop dependent work
for any frozen-contract/ownership conflict; otherwise continue to an exact local review SHA.

Legacy C0 commit `6736d63e` records the same October 27 freeze. Its patch did not apply
verbatim to the recovery base, so only the lifecycle correction is integrated above.
C1 local evidence: Godot 4.7.2, `with-slot.sh godot --headless --path clients/shared
--script res://checks/sit_chain_check.gd`: PASS (exit 0, 0.03 s after import).
Checks exercise exact offers, refused/newest offer, undisclosed/disclosed missing offers,
cancel/place change/incomplete walk, one-shot consumption, stand, signed pelvis and disclosure
precedence. Logic review: no action literals, no distance/retry, remembered state consumed before
submission; 3D return fields and depicted-only behavior preserved. Independent review pending.

Initial headless chain invocation exposed a legacy GDScript inference error in `leave`;
explicit `Array` typing fixes the boundary. No acceptance PASS claimed from that invocation.



### Independent recovery checkpoint — C2/C3/C4

C2/C3 implemented in the isolated checkout. The legacy patch is integrated selectively and
corrected: `seated_sprite.gd` owns image construction (67 lines), while `people.gd` keeps
reconciliation and gait (660 lines, existing size review trigger). Updated poses at the same
point rebuild their drawing; immediate node placement precedes frame animation; seating clears
bob/rock; standing restores exactly one view and its travel facing. Missing stature uses opaque
height as approved. Caption picking and `figure_at` follow the new height; the fixture verifies seated picking. Both walk and seated images use the
same row selection. The app remains the composition root (756 lines); no new gameplay ownership.

Cancellation is synchronous before key-driven walk completion, preventing an old chain from
submitting sit when a key takes over. Place changes cancel before walker reconciliation.
Menu opening, floor clicks and disconnects cancel as well. The rule scan admits only the two
submitter action constants and confines posture component names to the shared module.

Validation so far:
- `check_client_rules.py`: PASS, zero findings.
- Real Godot 4.7.2 app composition-root parse: PASS after correcting a legacy inferred String.
- `seated_sprite_check.gd`: PASS through actual people/sprite nodes, four drawing modes
  (known/unknown stature crop, seated atlas, four views), position/facing updates, gait suppression,
  standing restoration. In-memory image is a test fixture, not shipped content or generated art.
- `seat_check.gd` on preserved Lakeside frames: PASS output, person 7 alone seated at times 102/103,
  actual node pelvis (1090,3000) mm, view 3; 20 DRAWN rows. Full Rust comparison pending.
- Targeted `client_rules` first run: 12 PASS, 1 FAIL because the legacy check fixture's tag
  literal `cafe` violated the world-name scan. Renamed the fixture tag to `fixture-room`;
  production behavior unchanged. Rerun PASS: all 13 tests, 0.05 s.
- `client_2d_sit`: real lifecycle run declared before launch: exact-source server, two worlds,
  real 2D input path, one far click, seated disclosure/drawing, floor click, explicit requested
  stand preceding movement, persistent facts. <=150 s per client; no LM/API. Building in isolated
  `/tmp/mineworld-s34e-target`, shared slot. PASS: both worlds, one Rust test, 11.94 s runtime
  after 91 s compilation. Godot headless emitted existing unsupported keyboard-label display-server
  diagnostics; no script parse/runtime failure or lifecycle failure observed.

The 2D import unexpectedly included existing `shots/` images (46 local fixture/reference images),
not just scripts; no 3D/full repository texture import ran. Future checks use the existing cache.
No renderer-ready claim is inferred from a zero exit when Godot prints a script error.

Checkpoint limitations: this is **not full S34-e acceptance**. C4 still needs the frozen
cross-client Rust comparator, within-reach second sit, occupied-seat offer fidelity, self-menu/key
and cancellation real-client cases, and the specified mutations. C5 is blocked below; C6 final
acceptance documentation awaits that decision. No ignored test outside this focused lifecycle
case, full suite, public CI, or 3D contact acceptance is claimed. Independent red-team is pending.
The exact review commit is supplied in the coordinator handoff (avoids a self-referential SHA).

### PROPOSED C5 decision packet — not approved, no implementation

**Frozen requirements in conflict.** §5.7 requires `seat_offset_m` from furnishing minus point,
projected against facing, and a corpus test with <=10 mm longitudinal disagreement and <50 mm
sideways error for every depiction row. §4 I-e-2 and §5.1 fix the shared scalar pose; §4 I-e-6
forbids world changes. Existing F-RC-7 behavior/assertions remain frozen. No proposed option below
has been implemented or treated as approved.

**Evidence and conventions.** `contracts/src/spatial.rs` defines integer mm and millidegrees;
`worlds/social-cafe/places/cafe.yaml:45` states clockwise from north. Let point A=(x,y), furnishing
origin F=(fx,fy), yaw θ=facing/1000 degrees, forward U=(sinθ,cosθ), right V=(cosθ,−sinθ).
The scalar pose is P=A−1000×offset_m×U. The requested center-derived value is
`offset_m=−dot(F−A,U)/1000`; irreducible lateral error is `dot(F−A,V)` mm. No scalar value can
remove a nonzero lateral error. These are drawing coordinates, not new server reach calculations.

**Café measurements, identical in Social Café and Market Town.** Sources are each world's
`places/cafe.yaml:51–63` furnishings and `:130–138` sit points. Signed values in mm:

| Point | Furnishing | Against-facing offset | Lateral error |
| --- | --- | ---: | ---: |
| window-stools.1 | stool.1 | -484.00 | 0.00 |
| window-stools.2 | stool.2 | -505.00 | 0.00 |
| window-stools.3 | stool.3 | -523.00 | 0.00 |
| table-1.1 | table.east.chair-w | -545.80 | 82.49 |
| table-1.2 | table.east.chair-e | -545.80 | 82.49 |
| table-2.1 | table.north.chair-w | -532.84 | -119.15 |
| table-2.2 | table.north.chair-e | 110.86 | -495.76 |
| table-3.1 | table.back.chair-w | -38.36 | -478.46 |
| table-3.2 | table.back.chair-e | -538.27 | 43.16 |

Thus 5/9 café points per world (10/18 total) cannot meet the frozen lateral bound using a scalar
and the furnishing origin. Two chairs per world miss by approximately half a metre. User-visible
result: the torso floats beside the chair, even with a perfectly computed longitudinal offset.
This is not rounding; rounding to 3 decimal metres contributes at most 0.5 mm longitudinally.

**The content is not proven defective.** `pr-s34c-world-sit-points.md:220–227` records BC-c-1:
table-3.1 was moved south to (1600,7980) because the prior point (1060,8460) violated the plant's
260 mm body clearance. BC-c-2 intentionally uses the chair's actual facing. These are valid
semantic free-floor positions; forcing their line through the chair can reintroduce collisions.
Therefore the old S34-e assumption “every point lies behind its chair along its facing” is false.

**Furnishing origin is not a seat-center oracle.** Lakeside's multi-person benches intentionally
have several pelvis positions along one furnishing. A single furnishing origin cannot be the
center of all of them. `sit_points/lakeside.yaml` documents longitudinal seat-middle calibration:
long-table chair points at x=1660, furnishing centers x=1150 (`worlds/lakeside/places/bakery.yaml:34–38`),
face west, declared offset -570 mm, versus origin-derived -510 mm. The 60 mm difference is deliberate,
not evidence of drift. Bakery 5/5 rows fail the proposed 10 mm center-based oracle despite their
recorded calibration. Pier 5 rows: origin-derived 485 vs declared 450 mm, lateral 0/±600/±1200 mm;
square 6 rows: 460 vs 400 mm, lateral up to 700 mm; station wall 7 rows: 485 vs 450 mm, lateral
0/±600/±1200/±1800 mm; platform 6 rows: 485 vs 450 mm, lateral 0/±600 mm. All 29 existing Lakeside
rows exceed the proposed 10 mm longitudinal center comparison. Their correct seat targets need
per-seat surface/calibration evidence, not equality to the furnishing origin. No existing depiction
or probe assertion has been changed.

**Options, all requiring a decision.**

| Option | Concrete deliverable and tests | Files/API/dependencies | Consequence |
| --- | --- | --- | --- |
| A: corrected oracle with explicit gaps | Define per-seat calibrated targets and keep lateral failures visible; longitudinal-only café check cannot count as AC-e-9 PASS | Revise this design's AC-e-9/§5.7; new `sit_point_depictions.rs`, café JSON rows; no shared API change | Can document center-oracle error but does not solve 10 café lateral failures. Not recommended as completion |
| B: vector depiction contract | Add explicit place-local pelvis displacement while retaining scalar fallback; pin old scalar outputs byte-for-byte, vector outputs, generated renamed/mirrored points, both-client placement and real seat-contact | Re-freeze shared `pose_of`/loader contract, 3D presentation spec, two café depiction files, pose and depiction tests; coordinated S34-f/contact validation dependency | Correct semantic point and chair-facing remain intact; resolves lateral geometry. Material presentation API/3D change, outside current authority |
| C: split/defer C5 | Deliver independently reviewed C1–C4 drawing/input with disclosed-at fallback; keep C5 and AC-e-9 explicitly outstanding in a separately frozen correction | Explicit scope/acceptance revision here; coordinator owns new design and downstream dependency updates | Smallest safe independent checkpoint; fallback may draw beside chairs and cannot claim full S34-e ready |
| D: existing-contract content correction | Propose revised semantic sit points lying on chair-facing rays, revalidate clearance, spacing, reach, all-world loads, interaction tests and deterministic baselines | World Pack edits in both café copies, S34-c contract and baseline ownership; preserve scalar API | Not bounded under I-e-6; prior plant-clearance correction demonstrates risk. No feasible all-seat solution established; do not move worlds merely to satisfy a rendering oracle |

**Recommendation:** C for a clearly partial local handoff, followed by a separately reviewed B
presentation correction with a seat-surface oracle that respects multi-seat benches and intentional
calibration. A alone hides a user-visible gap; D crosses world ownership and can invalidate correct
body clearance. Primary/operator must choose; dependent C5 remains stopped. Independent C2/C3/C4
validation proceeds only within its already approved narrow scope. No broad validation campaign.

### C4 continuation after checkpoint 9c41ba69 — 2026-11-22

Coordinator authorized independent C4 continuation; the C5 packet above stays frozen for review.
Planned bounded lifecycle rerun: same two exact-source worlds, plus seated self-menu, nearby floor
walk and within-reach second click, current-offer semantic comparison and observation recording.
Expected <=30 s runtime plus incremental build, through shared slot; no API/LM or asset generation.
This rerun also exercises `figure_at` through the real self-click path; the previous live run did
not cover the final figure helper change. Results pending. Recording goes to an isolated temporary
directory; subsequent comparator fixtures will identify any projection or counterfactual explicitly.

C4 bounded results so far:
- Existing `client_sit_rule` comparator PASS, 1.17 s: both real client decision paths agree;
  added exact actual-node DRAWN rows for all 20 Lakeside people/frames, including seated view 3
  and (1090,3000) mm. Reused reviewed-base 3D class-name cache only (no texture import);
  each script still parses from current source, and script errors fail the comparator.
- X-e-6 and X-e-7 planted in `people.gd`: both expected RED, naming the file and line;
  original bytes restored in `finally`. No production mutation retained.
- First extended live run failed after 98.09 s: the intended floor destination was inside the
  drawn seated person's pick area and correctly opened self-menu. Test destination changed to
  sideways 350 mm. The new evidence logger also assumed a flat request payload; corrected it
  to the existing action-record envelope and required one evidence record per sent sit.
  This run is FAIL, not product/acceptance PASS. Retry stays within the existing one-hour budget.

C4 additional evidence:
- Extended two-world lifecycle PASS (19.94 s): seated self-menu, requested stand then nearby walk,
  within-reach second sit without walk, two complete-offer semantic comparisons through server
  `differing_fields`, point-keyed `SIT_TRANSCRIPT` records retaining AC-13 request envelopes.
  Later rerun after exposed-seat click selection also passed the added exact journal sequence
  `[walk-to, sit, stand, walk-to, sit]` (walk-step records omitted only for this sequence check).
- Captured real server observations, projected only to identity/location/tags/posture disclosures,
  into `posture-social-cafe.json` and `posture-market-town.json`; each records raw SHA256 provenance.
  No counterfactual is labelled a recording. Seeded renaming/mirroring and five-second stillness
  overlays are generated by Rust. Restored comparator: 3 PASS, 7.14 s, both real clients.
- X-e-4 (kept travel view), X-e-5 (reversed scalar sign) and X-e-1 (removed 2D disclosure) each
  expected RED through actual drawing/decision mismatches. The first X-e-1 attempt was inconclusive
  due to a test-generated null components field; fixed the fixture generator and reran X-e-1 only.
  All mutated production bytes restored; X-e-5 proves AC-e-1/AC-e-5 here, not blocked AC-e-9.
- The second 350 mm sideways live destination still intersected the plain-mode 15 px half-width
  at 24 px/m; run failed after 97.91 s. Corrected test destination to 700 mm, beyond picking and
  within server-reported reach; added immediate wrong-pick failure. No production picking changed.
- Occupied-seat setup initially retained the preparation player's reserved connection seat. The
  existing harness takeover flag only operates via its seatless join-screen path; both setup runs
  were stopped after diagnosing this, not counted as acceptance. Corrected test uses
  `Drive::start_seatless` plus explicit `--pick-seat=visitor --take-over`. Narrow rerun PASS: both worlds, 11.51 s.

Remaining frozen lifecycle mutations completed: X-e-2 forces the shared chain to walk regardless
of offer verdict; within-reach test RED (12.40 s). X-e-3 plants a delayed resend at the app's chain
answer submission boundary for `target_unavailable`; occupied-seat test RED for two requests
(6.57 s). This practical integration mutation represents the forbidden retry without inventing a
second-answer API in the shared chain. Both production files restored byte-for-byte. Final restored
live rerun PASS: both tests (both worlds each), 27.99 s. No script errors. Unchanged headless
keyboard-label display-server diagnostics remain unrelated to these assertions.

C4 checkpoint status: AC-e-1…e-7 now have targeted local evidence and X-e-1…e-7 have
expected-red evidence. AC-e-8 source inventory PASS: no server/system/kernel/contract/world or
baseline edits. Restored Rust client-rule gate 13 PASS (0.06 s); Python rules/headings and diff
whitespace PASS. Independent implementation red-team remains pending; no full S34-e acceptance,
partial shipping permission, public CI or merge readiness claimed. Existing C2 walk/strides/idle
baseline scenarios and extra key/disconnect integration paths were not run in this bounded C4
continuation; chain cancellation is covered by the shared deterministic check. C5 and final C6
remain outstanding. No production behavior changed after checkpoint 9c41ba69; changes here are
harnesses, fixtures, assertions and appended design/evidence. Rust formatting only followed the
final runs. Changed harness click selection was revalidated by both restored live cases.

Durable local evidence: `/tmp/mineworld-s34e-c4-live-restored.log`,
`/tmp/mineworld-s34e-c4-replay-restored.log`, and `/tmp/mineworld-s34e-X-e-{1..7}.log`.
SHA256 manifest `/tmp/mineworld-s34e-c4-evidence-sha256.json`:
`bdf8658f267ae4956deb7b691ee50cd37dc2302b902ac2a58553e2716d77f738`.
Reviewed C5 packet content remains byte-identical, SHA256
`24f09b891e218118d61e0fd13d637bd91faa98eaf258619d4c6b91a22f95cd1f` (section text, trailing whitespace stripped).

### PROPOSED B amendment contract — design only, not frozen

Source: independent C5 review `/tmp/mineworld-s34e-c5-red-team-review.md`, bound to
`9c41ba69c174f9b51735d3f490730608f5bdd6f1`, and coordinator's 2026-11-22 direction to refine B.
This append does not revise §5.7/AC-e-9, approve B, or alter the reviewed C5 packet. Correction to
its interpretation: 10/18 lateral failures measure **furnishing-origin error**, not independently
measured seat-surface contact failure. Actual chair/stool support regions remain to be measured.

**One representation.** Propose optional format-2 `seat_displacement_m: {east: number, north: number}`,
place-local horizontal metres, independent of point facing. For a vector row:
`pelvis_mm = selected_at_mm + 1000 * (east, north)`. Neither world coordinates nor model axes are
accepted in this field. Facing still controls body yaw/view only. Reflection across place x transforms
`east -> -east`, `north` unchanged, and `facing -> (360000-facing) mod 360000`.

**Versioning.** Keep all existing files format 1. New vector-bearing café files use format 2;
new loader accepts both, and vector fields in format 1 are an error. Old loaders already reject
unknown formats, avoiding a silent legacy 0.45 m default when they see a vector-only row. A format-2
file may contain scalar rows, but their arithmetic remains unchanged. No bulk migration.

**Precedence and validity.** Presence of `seat_displacement_m` selects the vector path; a row containing
both it and `seat_offset_m` is invalid, not silently prioritized. Require a dictionary with exactly
`east` and `north`, finite numeric non-boolean values; reject null, missing/extra axes, strings,
nonfinite/out-of-range parser values, mixed fields. Report file and point; reject the depiction file
atomically with no partially loaded rows (new-vector validation only). Do not tighten unrelated legacy
scalar validation in this amendment. Disclosed `at`/facing continue to override fallback exactly as
current `pose_of`; without disclosure use fallback; without either position return empty. Vector
rows do not alter `decide`, inference ownership, sit reach, or chaining.

**Exact legacy compatibility.** When the new field is absent, execute today's scalar path unchanged:
loader defaults (`seat_offset_m` 0.45, height 0.45), `pelvis(at,facing,offset)` signature/formula,
depiction matching and fallback behavior. No migration of the 29 Lakeside rows. Before editing,
freeze all existing row outputs at the executable baseline for disclosed and fallback inputs;
compare coordinates/fields exactly at the same precision, not by calling a relocated alias as the
oracle. Preserve the 3D wrapper's undepicted -> empty behavior and all F-RC-7 files/assertions.

**Independent target oracle, prerequisite to writing café offsets.** Read-only measurement of the
existing three model bindings, not furnishing origins: `furniture/furnishings.yaml:24–28` names
`dining_chair_02`, `painted_wooden_chair_01`, `bar_chair_round_01`, front_md 180000. The rendered
transform comes from `clients/3d-spike/scripts/slice/furnishings.gd:169–218` and `SliceProps.put`;
`assets/models/contact.json` declares floor mounting only and supplies no seat surface. Thus this
source audit does **not** establish any numeric target or stool height.

Proposed independent evidence artifact `tools/cli/tests/fixtures/seat-surfaces.json`:
model/source SHA, actual import scale/origin/front correction, model-local support polygon and seat
top height, measured per-seat pelvis target/calibration with rationale, furnishing key/pose, and
transformed place-local target. Measure model meshes or existing authored primitive dimensions
without reading proposed displacement values; save provenance and images/mesh samples sufficient
for independent review. Multi-seat benches have individual target stations/regions, not one origin.
Retain the bakery's intentional 60 mm calibration as historical evidence and verify against its
actual surface; do not erase it to satisfy a center test. Existing rows whose surfaces cannot be
established are explicitly unresolved, never silently excluded from an all-row PASS.

Proposed amended AC-e-9: every depicted row resolves to a furnishing and independently evidenced
seat target/region; computed horizontal pelvis is within 10 mm of the calibrated target in **both**
axes, and the target lies on the measured support region (boundary margin derived from the source,
not chosen after results). Height agrees with independently measured seat top within 10 mm.
These are proposed numerical authoring tolerances, not proven human contact tolerances; freeze only
after the measurement precision and all legacy rows have been audited. No universal origin test,
no longitudinal-only waiver, no claim of full rig/foot/contact correctness. S34-f retains its real
3D contact, support, approach and traversal acceptance with unchanged probe assertions.

**Bounded implementation if approved.** Shared `sit_rule.gd` loader/pose only; shared README and
`clients/3d-spike/PRESENTATION.md` schema; two café depiction files; focused shared pose/loader check,
new `tools/cli/tests/sit_point_depictions.rs` and the independent surface fixture; extend existing
cross-client drawing check for approved vector cases. Public `pose_of` return shape and scalar
`pelvis` signature stay unchanged. No new world/server/kernel/contracts files, baseline updates,
model generation, avatar sheets, physics, settings, input redesign, or S34-f probe changes. Primary
owns parent/dependency reconciliation; implementer owns this PR ledger and named files after freeze.

**Required tests.** Frozen scalar corpus parity; new vector east/north/non-cardinal cases; disclosure
over fallback and missing-position case; malformed/mixed field rejection; renamed/mirrored pose
fixtures; actual 2D and 3D placement (not decision-only equivalence); independent all-row surface
oracle. Drop/reverse each displacement axis must fail its target/placement test. Keep C4's currently
undepicted café oracle explicitly dependent on the approved new data and revise it only with B.
S34-f real contact remains separately owned and is not replaced by these assertions.

**Feasibility / freeze gaps.** The additive representation and bounded API are feasible by source
inspection. No target measurements have yet been obtained, so exact café values, support margins,
legacy-surface compatibility and measurement runtime are unknown. First bounded measurement pass:
existing three café model sources and one representative of each legacy seat form, no render/import
campaign; report missing assets or conflicts before expanding. If all-row surface audit disproves a
legacy target, propose a separately owned content correction; do not change legacy outputs under
this compatibility promise. B cannot honestly be marked freeze-ready until that evidence fixes
or rejects the proposed tolerances and target artifact. This is one concrete amendment candidate
with explicit closure work, not an implementation request or a request to approve unknown geometry.

## 13. Operator questions (each with a recommended answer)

**Ruled 2026-10-27 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34e-1** | While seated, should clicking **oneself** send `stand` at once (the step's §8.4 text), or open one's own menu, which already lists "Stand up" beside sleep and the other self actions? | **Open one's own menu.** A floor click or a walk key still stands the player up first. Recorded as a bounded deviation from §8.4's wording |
| **Q-S34e-2** | Real seated sprites need a new `cast.py` render of every cast body's sit pose (Blender, local slots, no money). Do it here, or in a follow-up S34-e2, with the cropped fallback until then? | **Split into S34-e2.** S34-e1 ships drawing, input and the fallback now; S34-e2 renders `<base>_seated` sheets for every cast body and NPC alike |
| **Q-S34e-3** | Café chair points sit 0.55 m behind the chair, and no café depiction exists, so 2D would draw the sitter behind the chair. Write `sit_points/social-cafe.yaml` and `market-town.yaml` here (offsets computed from the world's own furnishing and point positions, checked by a test)? 3D will then also draw café sitters seated | **Yes**, chairs and stools only. Park and street benches stay undepicted until S34-f |
| **Q-S34e-4** | Move the 3D client's `pose_of` / `pelvis` into the shared sit rule, with a one-line delegation in `slice/seating.gd`, so both clients place the pelvis by one function? | **Yes.** No 3D behaviour changes; S34-f builds on it |
| **Q-S34e-5** | Fallback crop line: hip at 0.530 × stature (Drillis and Contini, via Winter Fig. 4.1)? | **Yes**, used only until a body has a seated sheet |

### S34-B approved freeze amendment — 2026-11-22

Operator explicitly approved: “也同意 S34-B，继续实施”, relayed by primary coordinator in
message `f9d1f61b-da8a-4141-aac7-a4a3863f389e`, after clarification of presentation-only 2D
horizontal displacement preserving semantic world coordinates and old behavior. This authorizes
C5/C6 continuation from clean `4ce72437c8206826a5d5b5eba9fc3471024032c6`; it does not approve
body-fit/contact changes, public writes, merge, or S34-f scope.

Binding concrete amendment: `/tmp/mineworld-s34e-seat-measure/ENGINE-B-PACKET.md`, SHA256
`c7727ba68ff0207ae7553414656196ba4fecc4bc27f9240cb6445eb713387634`, with measured numbers in
`engine-targets.json`, source/engine provenance, geometry review PASS and engine closeout PASS
(`/tmp/mineworld-s34e-engine-closeout-review.md`). The two source Café files are byte-identical;
the packet's solids-difference wording concerned the older recorded fixture, not the two worlds.

This supersedes §5.7's format-1/scalar-only Café requirement and furnishing-origin oracle, and
AC-e-9's corresponding acceptance rule: the two Café depiction files use format 2, optional
`seat_displacement_m: {east, north}` in place-local metres, mutually exclusive with scalar offset.
Validate finite numeric non-boolean exact axes, reject malformed/mixed vector files atomically,
and reject vectors in format 1. Disclosed position/facing precedence and pose return shape remain
unchanged. Scalar absence/default/formula, all 29 Lakeside scalar outputs, world semantic points,
and 3D undepicted-empty behavior remain exact. No world/production asset changes are authorized.

The accepted central seat drawing anchors are mesh-derived and engine-verified; 10 mm per-axis
horizontal and height tolerance measures coordinate/height reproducibility only. Require selected
visual triangle support; do not derive expected anchors from authored offsets or equate a
furnishing origin with a seat. Legacy calibrated anchors stay fixed, including bakery's 60 mm
calibration. Armrest/body/foot/contact correctness remains S34-f. Existing C4 undepicted Café
expectations may now change for the approved depicted Café points; genuinely undepicted cases
remain covered. Scope/files/validation follow the concrete packet and existing affected-test budget.
The original 9c41 C5 packet and 4ce72437 review payload remain historical immutable evidence.

### S34-B implementation and C5/C6 local closeout — 2026-11-22

Implemented the approved amendment in the same isolated branch. Only production logic change is
shared `sitting/sit_rule.gd`: formats 1/2, a pre-load validation pass that atomically rejects invalid
vectors, and a place-local vector drawing branch. The two Café depiction files contain nine rows
each with the reviewed engine-measured values. No point fallback was invented. Every legacy scalar
calculation and world semantic position stays unchanged; existing source/mesh hashes are checked.

Independent oracle: `tools/cli/tests/fixtures/seat-surfaces.json` contains pre-authoring measured
anchors, selected source triangles, engine/source provenance, all 29 legacy surface samples and
58 pre-edit executable legacy outputs. It contains no proposed displacement fields from which to
reconstruct expected targets. The scalar corpus was captured before editing production code;
its helper and original JSON are retained in `/tmp/mineworld-s34e-seat-measure/`. Compact triangle
formatting keeps the fixture inspectable without removing mesh samples. Existing legacy primitive
source provenance is included. `sit_point_depictions.rs` checks all 18 Café and 29 legacy rows;
`checks/sit_pose_check.gd` checks exact engine scalar outputs, finite/range/shape validation,
format/exclusivity, atomic rejection, fallback/disclosed precedence and renamed/mirrored vectors.

The client comparator now checks actual 2D drawn nodes and actual 3D figure-node positions/seat
height inputs, through `SliceSeating.observe` and `SliceFigures.seat`; it deliberately loads no body
rig (contact remains S34-f). Recorded fixtures, a labelled all-nine-point counterfactual, seeded
renaming/mirroring with a temporary depiction pack, standing still on disclosed points, and the
Lakeside fallback fixture pass. The independent measured targets are transformed for the generated
mirror; authored displacement values never supply expected coordinates. `DRAWN3` horizontal values
are millimetres; height is micrometres. The 2D `Town.local_in` observation truncates to integer mm,
so its rounded independent target comparison permits 1 mm (AC-e-1 and below B's 10 mm authoring
threshold). Production arithmetic was not altered to satisfy the comparison.

Final local evidence:

| Check | Result / evidence |
| --- | --- |
| Independent depiction oracle | 2 PASS; all 47 rows; `/tmp/mineworld-s34e-B-oracle-final.log` |
| Shared pose/schema/source provenance | PASS; 29 fallback + 29 disclosed exact outputs; `/tmp/mineworld-s34e-B-final-pose.log` |
| Shared chain | PASS; `/tmp/mineworld-s34e-B-final-chain.log` |
| Restored-source real placement comparator | 3 PASS, 21.26 s; `/tmp/mineworld-s34e-B-final-restored-clients.log` |
| Live hosted interaction after drawing/picking moved | 2 PASS, 20.08 s, both worlds: exact offers/transcripts, floor requested-stand, near click, occupied refusal/no retry; `/tmp/mineworld-s34e-B-live.log` |
| Required remaining walk/strides/idle baselines | 3 PASS, 36.55 s: `walks_from_the_apartments_into_the_cafe`, `a_walk_is_one_walk_to_paced_by_walk_steps`, `reconnects_to_a_restarted_world_and_to_a_replaced_one` (includes existing idle drive); `/tmp/mineworld-s34e-B-final-baseline-walk-strides-idle.log` |
| Client rules | 13 PASS, 0.08 s; `/tmp/mineworld-s34e-B-final-rules.log`; Python R1–R8 scan PASS |
| New axis mutations | Drop/reverse east and drop/reverse north each make actual 3D-placement comparator RED; source restored byte-for-byte, then comparator GREEN; `/tmp/mineworld-s34e-B-mutations.json` and four mutation logs |
| Documentation/static | decision IDs/index PASS (257 distinct, no index diff); headings PASS; scoped Rust formatting and `git diff --check` PASS |

Failures were not hidden: the first legacy comparator compared JSON float facing to engine integer
facing; normalizing only that test type restored exact equality. The first Rust comparator build
had a missing local `place` in the generated-frame helper and was fixed before runtime. The first
new drawing comparison exposed the existing integer-mm truncation above; its 1 mm observation
accounting is explicit and approved by coordinator message `8c177d03-ebf6-4241-b2dc-6bceddb3d399`.
Expected malformed-input warnings and the existing headless display-server key-label diagnostics
remain in logs; test assertions/summaries passed. A provenance-formatting helper rejected malformed
JSON before writing; the prior fixture remained intact and final provenance checks passed.

Evidence hash manifest `/tmp/mineworld-s34e-B-evidence-sha256.json` SHA256
`1b46114d79607c2d5893bedc548acaae18997b4247fe0844254bca359ca2c369`.
The original 9c41 C5 packet remains byte-identical (trimmed section SHA256
`24f09b891e218118d61e0fd13d637bd91faa98eaf258619d4c6b91a22f95cd1f`).
Earlier C4 X-e-1…7 evidence remains recorded; no additional broad mutation/body/asset campaign ran.

C6 ownership clarified explicitly by the coordinator: dated amendments are now in the isolated
branch's `docs/decisions/ARC-145.md` and `ARC-147.md`. Historical decisions remain; they record
implementation and approval, not merge/full acceptance. The decision index is unchanged. No root
checkout, shared overall plan/dashboard or unrelated decision record was edited.

Readability/debt review: one shared loader/pose owns the new representation; validation is isolated
in a small helper and runs before any rows are appended. Numeric ranges are constrained only by
finite drawable coordinates. No new world names, point keys, action decisions, clock reads or
renderer access entered production shared logic. Test-only pack injection stays in the two check
harnesses. The fixture is independent measured evidence, with explicit body-clearance exclusions;
no extra production abstraction, duplicate pose formula, baseline update or TODO was introduced.

C5 implementation/local validation and C6 records are complete for independent review. Mandatory
independent final red-team review is still pending; this is not full S34-e acceptance or shipping
approval. No new live key/disconnect variants were added (shared cancellation cases are covered;
existing reconnect baseline now passes). S34-f contact/armrest/foot validation and S34-e2 rendered
sprite work remain excluded. F-RC-7 assertions/logger/helpers and all protected server/world paths
are unchanged by this amendment. No public writes or merge were performed.


### Recovery closeout and independent review reconciliation — 2026-11-22

The previous paragraph's pending-review status is historical. Independent final review
`/tmp/mineworld-s34e-B-final-red-team-review.md` reports **PASS for local PR readiness** at
executable HEAD `bcee317356a9cc5535cbbaf5907232ec885a0200`, carrying forward the C1–C4
review at `4ce72437c8206826a5d5b5eba9fc3471024032c6` in
`/tmp/mineworld-s34e-checkpoint-red-team-review.md`. The lane base remains
`003d35437d6b7f873db1a2c32a29f0b6f554cb58`; none of these local prerequisites is
claimed merged. The final reviewer found no mandatory local acceptance blocker or material
introduced maintainability debt. S34-f contact and S34-e2 rendered sprites remain excluded.

A replacement recovery owner audited the original clean checkout and created the isolated
shared clone `/tmp/mineworld-s34e-closeout-20261122`, branch `recover/s34e-closeout`,
from that executable HEAD. The original `/tmp/mineworld-recover-s34e` was not edited.
This closeout changes this ledger only. Current process inspection found no process naming
the original S34-e checkout/target; no old process, cache or evidence was altered. No test,
heavy job, paid call, publication, workflow dispatch or merge was launched during recovery.

Completion evidence for the §9 checkboxes:

| Milestone | Implementation, validation and review evidence |
| --- | --- |
| C0 | Frozen design and approval retained; recorded heading validation; both independent reviews assess the frozen/amended scope. |
| C1 | `cd0613ad`; shared pose/chain checks and retained client comparator; independent C1–C4 review confirms offer fidelity, single consumption, cancellation and shared ownership. |
| C2/C3 | `9c41ba69`; actual drawing nodes, sprite check and two-world input lifecycle; final walk/stride/reconnect baseline 3 PASS and client rules 13 PASS; independent C1–C4 review covers drawing, input and maintainability. |
| C4 | `4ce72437`; recorded/generated comparator and live interaction evidence, seven expected-red mutations, protected-path inventory; independent C1–C4 review verifies the nine retained evidence hashes and accepted coverage. |
| C5 | `bcee3173`; approved S34-B versioned representation, 18 Cafe/29 legacy rows, 58 exact legacy outputs, independent oracle 2 PASS, placement comparator 3 PASS and four expected-red axis mutations; final independent review closes amended AC-e-9. |
| C6 | `bcee3173`; dated ARC-145/147 amendments and recorded headings/decision-index checks; final independent review confirms documentation and local scope closure. This recovery reconciles stale status/checkboxes without changing acceptance. |

Recovery independently verified all **19/19** files in the retained final evidence manifest;
manifest SHA256 remains `1b46114d79607c2d5893bedc548acaae18997b4247fe0844254bca359ca2c369`.
Actual log summaries confirm oracle 2 PASS, placement 3 PASS, live interaction 2 PASS,
walk/stride/reconnect 3 PASS and client rules 13 PASS. These are inspected prior execution
results, not newly rerun tests. Final independent review additionally records verification
of 23 source hashes and 12 measurement artifacts; recovery does not claim to repeat that
source review. Extra live key/disconnect variants remain NOT RUN; the independent reviewer
accepted the existing cancellation/source and reconnect coverage for the frozen boundary.

Endpoint: **LOCAL REVIEW READY / CLOSED AWAITING COORDINATOR INTEGRATION**. Coordinator
owns parent/dashboard synchronization and any later publication/integration decision. Public
PR CI for this candidate, full-suite result, full MVP acceptance and merge remain unestablished.
Any later executable change requires affected revalidation and independent review; this
ledger-only closeout inherits the exact executable evidence without claiming new-head CI.

Closeout documentation review: independent `recovery_review` PASS on the ledger-only diff;
completion claims agree with the two preserved implementation reviews. Recovery heading scan
PASS (254 numbered sections across 337 documents), and `git diff --check` PASS. No executable
content changed in this reconciliation.

### Packaged root correction — local playable recovery 2026-11-22

Coordinator explicitly delegated the bounded existing-contract correction after auditing
`/tmp/mineworld-playable-evidence/REVIEW.md`. No invariant or scope change: the
2D PRESENTATION.md §1 and ARC-80 require exported resources to resolve against
parsed `--root`; app.gd currently omits it only for Seating. Source mode must retain
its existing empty-root fallback. Change precisely `repo_path(".")` to
`repo_path(".", root)` in Seating construction. No world/vector/fixture/threshold,
server, shared-rule or 3D change. Prior package reports477.133mm north and239.300mm
height error against the pre-authoring fixture; it remains immutable evidence.

Owner recover_s34e; isolated branch fix/playable-export-root based2157605b.
Local-only endpoint, semantic commit permitted under existing execution contract;
no publication/merge/full-MVP assertion. One standard shared heavy slot, <=10min
per job, <=1hour initial work. Build exact-head server/launchers from copied cache;
existing build measured282s, assembly54s. Export through existing package APIs.

Acceptance: actual packaged root with spaces/nonASCII; independently measured
pelvis/height within existing10mm tolerance; unchanged source mode exercised;
real window sit/stand observed separately from harness; actual launcher smoke and
persistence/restart. Gate1 N/A (no model semantics). Gate2 owns exported loading,
window/input and persistence. Broad suites/remoteCI NOT RUN (local-only lane).
Source assertions alone cannot validate this escaped integration defect.

- [x] Implementation: propagate parsed root to existing Seating constructor.
- [x] Validation: exported pose oracle and source-mode regression; preserve failures.
- [x] Independent review: concrete diff and final artifact/evidence review;
  `/tmp/mineworld-playable-20261122-independent-review.md` approves bounded local trial.

Regression implementation: `scripts/check_packaged_seating.py` reuses the existing
packager server lifecycle and client drive steps, with a fresh evidence directory
that refuses overwrite. It compares two actual seated poses and the intervening
stand against the immutable mesh fixture. `--source-client` omits `--root` to
check checkout fallback with the same server/actions. Historical broken exported
log is RED for both477.133mm/239.300mm cases; assertions/tolerances unchanged.
Python parse check and client-rule scan PASS. Initial py_compile could not write
the system cache; redirected only its bytecode cache into isolated evidence.
Independent1line production review PASS; regression/final review pending.

First exported regression (pose-exported,6.079s) FAIL in fixture navigation:
new actual pelvis3160.012/1666.867mm and height0.6893m resolve correctly, but
`floor_near_seat` chooses a point under the default sprite, opening the self
menu. That helper was designed/tested with plain drawing. The root regression
now uses existing open-self/choose-stand to observe the same real posture
transition without assuming a free sprite pixel. No production input logic,
fixture target or tolerance changes. Floor-click/window checks stay separate.

Exported pose with real self-menu stand PASS5.167s; checkout mode (no root)
PASS5.367s. Two seated poses in each match the independently measured stool
anchor within0.001mm horizontally/height. Actual Metal windowed2D sequence
PASS28.243s: walk-to, sit, self-menu stand, sit;3screenshots1600x900 retained.
The previous windowed click failure did not recur; its exact cause remains
unestablished. This automated real-window evidence is not manual human play.
CUA getApp timed out after299s when the scripted window had already exited;
no control permissions changed and no native input success is claimed.

Independent reviewer required explicit drawn:true and same-point assertions in
the regression. Added both; all3retained successful logs/6seated poses PASS
under the final comparator, and a drawn:false mutation is rejected. Fixture
coordinates and tolerance remain unchanged.

Final local package checks: existing2D,3D,both launchers PASS in272.353s total,
with terminal verdicts and clean server/child/scratch shutdown. Combined walker
met the observer at the doorway, completed without collision bypass, and passed
its unchanged assertions. Save restart/replay PASS2.330s:8revisions,108facts and
1snapshot reproduced byte-for-byte. Archivechecksum and AC8reference comparison
remain explicitly NOTRUN. Windowed2D doorway traversal still running.

Windowed doorway scenario completed PASS 163.366 s, exact drive terminal,
31/31 strides accepted, two place changes, Cafe facade lifted in 0.22 s
(bound 1 s), four saved 1600x900 screenshots. All target package checks are
complete. Independent review verified all 772 package hashes and approved
bounded local operator trial; final doorway supplement was submitted.
Durable copy under `target/local-playable-20261122/` in the primary checkout
uses the existing launchers, with isolated trial-save commands in START-HERE.txt;
all 772 copied file hashes match. This is gitignored delivery output, not a
public release or a modification to the primary source tree.

Final source checkpoint contains only regression/ledger updates after executable
commit `e3f207186ea76d562800dbd6c53cb40daf9dacc1`; BUNDLE.toml intentionally
retains that actual executable checkpoint. Final test comparator SHA and all
191 retained evidence hashes are recorded outside the bundle. No re-export or
repeat broad suite is required for the test-only correction. The reviewer will
record the final source commit and doorway supplement in its external report.
