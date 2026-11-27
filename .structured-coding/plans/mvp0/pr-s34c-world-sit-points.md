# PR S34-c — sit points in the shipped worlds, and `posture` enabled there

## Lifecycle

```text
Design revision:        revision 1 (2026-10-19), FROZEN
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §12)
Implementation base:    origin/main @ 5a8c6f28 (after #301, S34-a)
Start condition:        S34-b merged (the load-time check that a sit point clears every Solid);
                        for the Lakeside half (S34-c2): S33-b merged (Lakeside's `body:` sections)
Execution contract:     §10
Lifecycle:              DESIGN FROZEN — 2026-10-19 (operator approved in the primary session: every question answered as recommended)
```

**Dated note, 2026-10-27 (operator ruling Q-S34c3-7, in
[`pr-s34c3-station-sit-points.md`](pr-s34c3-station-sit-points.md)): S34-c2's frozen scope is widened.**
Lakeside's pier and square benches face the opposite way from S33-f §5.2's sit points (F-c3-6). S34-c2 no
longer copies S33-f's table verbatim where the two disagree. It checks each drawn bench against its sit
points, decides from the drawn geometry which side is wrong, and corrects that side: the bench's `facing`
in the place file, or the sit points' `facing` and `at`. It records the numbers in its ledger. The load-time
clearance check (S34-b) and every acceptance criterion of §8 still hold after the correction. S34-c3 runs
after S34-c2.

**Effort:** `mvp0`, release-blocking · **Step:** [`step-34-posture.md`](step-34-posture.md) §§5.1, 12–14
(the S34-c row; AC-S34-4, AC-S34-6's world half, AC-S34-11; I-S34-1, -2, -6, -7, -9; Q-S34-8; R-S34-6) ·
**Previous PRs:** [`pr-s34a-posture-pack.md`](pr-s34a-posture-pack.md) (#301),
[`pr-s34b-bodies-anchoring.md`](pr-s34b-bodies-anchoring.md) (DRAFT, #327) · **Consumes:**
[`pr-s33f-interiors-sit-points.md`](pr-s33f-interiors-sit-points.md) §5.2 (Lakeside's table, Q-S33f-1
option A) · **Record:** a dated note on `ARC-147`. No new id.

**Working name:** S34-c (two PRs: **S34-c1** Social Café and Market Town; **S34-c2** Lakeside, Q-S34c-1).
**Branches:** `mvp0/pr-s34c1`, `mvp0/pr-s34c2`. One worktree and one session each.

**In one sentence.** S34-a made sitting possible in a test world; this PR writes the sit points of every
real chair, stool and bench into the shipped worlds' place files, turns `posture` on there, and proves —
with one generic test over every world — that no sit point is inside furniture and that taking the pack
out again gives back the world as it was.

---

## 1. Scope

**In (S34-c1):**

1. `sit-points:` sections (the key is **`sit-points:` with a hyphen**, S34-a BC-10) in
   `worlds/social-cafe/places/{cafe,park,street}.yaml` and the same three files of
   `worlds/market-town/` (byte-identical copies today, audit row 3).
2. `posture` appended to `systems:` in both `world.yaml` files, with a comment line, as `appearance` is.
3. The generic world test (§4 SD-c-4) over **every** World Pack under `worlds/` plus a generated variant.
4. Re-baselines: both worlds' sections of `scripts/baselines.txt`, with a `reason S34-c1` line; any genesis
   golden or digest test that names these worlds.
5. Docs: the underscore spelling `sit_points:` corrected to `sit-points:` in `step-34-posture.md`,
   `step-33-lakeside-3d.md`, `pr-s33c-world-seam.md`, `pr-s33f-interiors-sit-points.md` (dated
   bounded-correction notes, not rewrites of frozen decisions); the seating-allowance source cited
   (Q-S34-8) in `ARC-147`'s note.

**In (S34-c2, after S33-b):** Lakeside's `sit-points:` from S33-f §5.2's table, verbatim; `posture` in
`worlds/lakeside/world.yaml`; Lakeside's baseline re-recorded in the same PR as any other Lakeside change
that lands with it (Q-S33-8: one re-baseline).

**Out:** bodies-yard (no seat is drawn there, so `posture` is not enabled and the world stays
byte-identical; Q-S34c-3); the station halls (S29's places are not on main; their points go in with the
places); NPC and agent behaviour and the 30-day runs that show NPCs seated (S34-d, Q-S34c-4); every
depiction and client change (S34-e, S34-f).

## 2. Source audit (`origin/main @ 5a8c6f28`)

| # | Fact | Where |
| --- | --- | --- |
| 1 | The section key is `sit-points:`; a point is `{ key, at: {x, y}, facing, group? }`; ≤ 64 per place; spacing ≥ 510 mm; reach 750 mm | `systems/posture/src/section.rs`, `point.rs`; S34-a SD-a-1, BC-10 |
| 2 | Bodies' `PERSON_RADIUS` 250 mm, `GAP` 10 mm: a point must clear every Solid by 260 mm and every wall by 250 mm | `systems/bodies/src/geometry.rs`; `genesis.rs:143-200` (the doorway precedent) |
| 3 | Social Café and Market Town share `cafe.yaml`, `park.yaml`, `street.yaml` byte for byte | `diff` of the two worlds' files |
| 4 | Café solids: window bench slab y 460–900; **drinks ledge y 950–1290 in front of it, the bench's full length**; three stools; three round tables with two chairs each, all Solids (`ARC-306`) | `worlds/social-cafe/places/cafe.yaml:40-75` |
| 5 | Park: two benches (1 900 × 700 mm boxes); street: two benches (1 900 × 700) | `worlds/social-cafe/places/park.yaml:27-35`; `street.yaml:97-99` |
| 6 | A bench's facing is not in the place data; it is the depiction's yaw in the 3D client | `clients/3d-spike/scripts/slice/cafe_interior.gd:430-495`; the slice's street and park builders |
| 7 | bodies-yard draws no seat | `worlds/bodies-yard/places/*.yaml` (no seat solid, no description of one) |
| 8 | Lakeside has no `body:` section yet (S33-b unmerged); its sit points are fixed in S33-f §5.2 | `worlds/lakeside/places/*.yaml`; `pr-s33f-interiors-sit-points.md` §5.2, F-S33f-2…5 |
| 9 | A section whose owner is not enabled is refused at load, so removing `posture` means removing the pack and its sections | `worldpack/src/load.rs:457-481` (`OwnerNotEnabled`); S33-f A-5 |
| 10 | Long-run baselines per world, re-recorded with a `reason` line | `scripts/baselines.txt` header |
| 11 | Facing frame: 0 mdeg faces north (+y), 90 000 mdeg faces east (+x) | `docs/CORE_CONCEPTS.md:440` |

## 3. Invariants

- **I-S34-9** (step): every drawn seat a person can reach has a sit point; every sit point has a drawn seat.
- **I-c-1. No interpenetration (operator 2026-10-19).** Every sit point of every world is on the floor and
  clear of every Solid by 260 mm. Enforced at load by S34-b; asserted over every world by SD-c-4.
- **I-c-2. Removable.** A world with `posture` and its `sit-points:` sections deleted loads with the same
  genesis digest as the same world at this PR's base.
- **I-c-3. Generic.** No world name, place name or coordinate appears in code. Every value is world data.

## 4. Decisions

- **SD-c-1. Where a sit point goes (one rule for every seat).** `at` is the free-floor point nearest the
  seat's pelvis position that clears every Solid by 260 mm, every wall by 250 mm and every other sit
  point by 510 mm. Three cases follow:
  - **open front** (a park or street bench): on the seat's front, 260 mm from the bench box's front face;
  - **front blocked by a table or a ledge** (café chairs, stools): behind the seat, 260 mm from the
    chair or stool box's back face, on the aisle side (R-S34-6's option "author chairs' points on the
    aisle side"); the depiction then carries the pelvis **forward** onto the seat (Q-S34c-2);
  - **behind blocked too**: beside the seat, on the side that fits; if no side fits, the seat gets no
    point and is recorded as unreachable (Q-S34c-5).
  `facing` is the way a person on the seat faces: toward the table or ledge for chairs and stools, away
  from the backrest for benches (from the depiction's yaw, audit row 6).
- **SD-c-2. Benches hold one person per 600 mm** (Q-S34-8, ruled). A 1 900 mm bench holds three points,
  at its centre and 600 mm either side. Source cited in `ARC-147`'s note: a published seating-allowance
  figure (e.g. the 24 in / 610 mm per person of BIFMA X5.4 lounge seating, or Panero and Zelnik's
  *Human Dimension & Interior Space* (1979) bench plates); C1 quotes the one it can verify, or marks the
  value approximate with S34-f as its replacer.
- **SD-c-3. Keys and groups.** `key` = `<group>.<n>` with `n` from 1 west-to-east then south-to-north;
  `group` names the furniture (`window-stools`, `table-1`, `park-bench-west`, …). Keys are data and
  appear in no code.
- **SD-c-4. One generic test over every world** (`tests/acceptance/tests/sit_points.rs`, or posture's
  integration tests if the acceptance crate cannot read worlds):
  1. loads every World Pack under `worlds/` (so S34-b's load check runs on each sit point);
  2. for each place with `sit-points` and a `PlaceShape`: every point lies within `reach` (750 mm) of at
     least one Solid — a sit point stands next to furniture, never in an empty room (the cheap,
     data-only half of I-S34-9; the drawn-vs-disclosed half needs depictions bound by key, S34-f);
  3. removability (I-c-2): the same world with `posture` and every `sit-points:` section removed (the
     test edits a scratch copy) has the genesis digest of the base world;
  4. runs the same three checks on a **generated variant** (a seeded random shaped place with random
     seats placed by SD-c-1's rule), so the rule — not only the shipped numbers — is tested.
- **SD-c-5. The planned points (S34-c1).** Frames are each place's (mm). Values are proposals; C1
  computes the final ones with SD-c-1 and S34-b's check confirms each.

  | Place | Group | Points | `at` (proposed) | `facing` | Case |
  | --- | --- | --- | --- | --- | --- |
  | café | window-stools | 3 | (3160, 2144), (4760, 2165), (6360, 2183) | 180 000 (south, to the ledge) | behind |
  | café | table-1 (4443–5277, 2743–3577) | 2 | (3648, 3060) facing east; (6092, 3240) facing west | 90 000 / 270 000 | behind |
  | café | table-2 (6673–7547, 4160–4997) | 2 | (5904, 4460) facing east; (7790, 5148) facing west | 90 000 / 270 000 | behind / beside (the east chair's back is 244 mm from the wall) |
  | café | table-3 (1866–2654, 8166–8954) | 2 | (1060, 8460) facing east; (3480, 8640) facing west | 90 000 / 270 000 | behind |
  | café | window bench | 0 | — | — | unreachable: the ledge stands 50 mm in front of the slab, the wall 460 mm behind it (Q-S34c-5) |
  | park | 2 benches | 3 each | centre, ±600 mm, 260 mm off the front face | from the depiction | open front |
  | street | 2 benches | 3 each | as the park | from the depiction | open front |

  Totals per world: café 9, park 6, street 6 — 21 sit points, the same in both worlds.

## 5. Acceptance (decided before measuring, `ARC-23`)

| Id | Criterion | Test |
| --- | --- | --- |
| AC-c-1 | Both worlds load with `posture` enabled; every sit point passes S34-b's check | SD-c-4 item 1 |
| AC-c-2 | Every sit point of every world is within 750 mm of a Solid; the generated variant passes the same | SD-c-4 items 2, 4 |
| AC-c-3 | AC-S34-11 / I-c-2: removing `posture` and its sections gives the base genesis digest, in both worlds and the variant | SD-c-4 item 3 |
| AC-c-4 | In each world, a scripted person walks to `window-stools.1` (Social Café) and to `table-3.1` (Market Town), sits, is observed seated by another person, walks away and is stood up | one scenario test per world |
| AC-c-5 | AC-S34-4's world half: seated in Social Café, the world is dropped and resumed from SQLite; still seated, same `since` | persisted scenario |
| AC-c-6 | I-S34-1, -2, -7 hold after every request of a seeded 300-step run of each world (the 30-day runs belong to S34-d, Q-S34c-4) | seeded run test |
| AC-c-7 | bodies-yard is byte-identical (its baseline does not move) | `scripts/baselines.txt` unchanged for it |

**Planted mutations:** X-c-1 move `window-stools.2` 100 mm south (into the stool) → AC-c-1 red, naming the
key; X-c-2 leave one `sit-points:` section behind when removing the pack → AC-c-3 red; X-c-3 a point
2 m from every Solid → AC-c-2 red.

## 6. Commit plan (S34-c1)

Each commit has **Implementation**, **Validation**, **Review**, each `[ ]` until done with evidence.

### C0 — Design (this file), docs only
- [ ] Implementation: this file.
- [ ] Validation: `python3 -I scripts/check_decision_ids.py`; `python3 -I scripts/check_doc_headings.py`.
- [ ] Review: scope equals the step's S34-c row, apart from §12.

### C1 — The data
- [ ] Implementation: the six `sit-points:` sections; `posture` in both `world.yaml`; the source citation.
- [ ] Validation: both worlds load (`mineworld` CLI `check`/load through `with-slot.sh`); S34-b's check passes.
- [ ] Review: every point traced to a row of SD-c-5 and a Solid; no point in an unreachable seat.

### C2 — The generic test and the scenarios
- [ ] Implementation: SD-c-4's test; AC-c-4…6 scenarios.
- [ ] Validation: the tests green; X-c-1…3 planted one at a time, each red, reverted.
- [ ] Review: no world name in the test's code (worlds are discovered by directory listing).

### C3 — Re-baselines and docs
- [ ] Implementation: `scripts/baselines.txt` (two worlds, `reason S34-c1` lines) from a long record of
  the PR head; the underscore corrections; `ARC-147` note.
- [ ] Validation: doc scripts PASS; the train or milestone CI's parity job equals the new baselines.
- [ ] Review: bodies-yard and Lakeside sections untouched.

## 7. Test ownership

| Failure class | Owner |
| --- | --- |
| A point inside furniture | S34-b's load check, run on every world by SD-c-4 |
| A point in an empty spot / a missing point | SD-c-4 item 2 (data half); S34-f (drawn half) |
| Removability | SD-c-4 item 3 |
| Long-run history | `scripts/baselines.txt` via the parity job (milestone CI) |
| NPCs seen seated | S34-d |

## 8. Is any of this material?

Q-S34c-2 (the pelvis may move forward from `at`) changes the depiction contract of step §8.2 that S34-f
implements, so it is the operator's. The rest follows the frozen step.

## 9. Risks

| Risk | Mitigation |
| --- | --- |
| Rule controllers start sitting as soon as `sit` is offered and NPC histories move a lot | Expected; recorded in the re-baseline reason; measured and tuned in S34-d (Q-S34-11) |
| The two worlds' copies of the same place files drift | Both written in one commit; SD-c-4 runs on both |
| 3D draws a seated person 0.5 m off the chair until S34-f | Known gap (step §14.2 last paragraph): clients still draw by S33's rule until S34-e/f |

## 10. Execution contract

```text
Repository / worktree:  yuema137/MineWorld · /Users/yuema137/mineworld-worktrees/s34c1-world-sit-points
Branch / base:          mvp0/pr-s34c1 from origin/main after S34-b merges and this design freezes
Authorized (on freeze): implement, validate locally (cast-slots/with-slot.sh), commit, push, open the PR
Validation budget:      the new tests and the two worlds' loads (speed-first charter); full suites at milestone
Stop condition:         MERGE-READY: PR number and head SHA reported to the coordinator
Merge boundary:         the coordinator merges
Escalate:               any code change outside tests; a change to another world
```

## 11. Evidence ledger

(empty until implementation)

**2026-10-19, S34-c1 implementation (branch `mvp0/pr-s34c1`, stacked on `mvp0/pr-s34b`, on main after
#343 vf-furnishings).**

- Bounded correction BC-c-1 (#343 landed first, as the brief warned): main's café now has solids for the
  shelves and two potted plants. The back plant (128–912, 7 602–8 418) stands 154 mm from SD-c-5's
  `table-3.1` at (1 060, 8 460), so that point moves beside its chair, south: (1 600, 7 980), 266 mm from
  table-3 and 688 mm from the plant. Every other SD-c-5 point clears the new solids by more than 260 mm
  (S34-b's load check and `tests/worlds.rs` confirm).
- Bounded correction BC-c-2: chairs' `facing` is the chair's own facing from `furnishings:` (81 406,
  261 406, 102 605, 282 605, 85 416, 265 416), not the rounded 90 000 / 270 000 of SD-c-5, so the seated
  person faces the way the drawn chair does; stools face the ledge (180 000).
- Bounded correction BC-c-3: the park has no drawn bench, so its benches face the lawn (north-west bench
  south, south-east bench north toward the fountain); the street's face the road, as the 3D slice yaws
  them.
- SD-c-4's removability check compares genesis facts, not a digest: the stripped pack's genesis facts
  equal the full pack's with posture's own (`place-sit-points`) left out. The kernel has no genesis
  digest to compare.
- Pending to the milestone (speed-first charter): `scripts/baselines.txt` for social-cafe and
  market-town (a long record of the merged head), and mutations X-c-1…3.

### 11.2 S34-c2 — Lakeside (2026-11-27, branch `mvp0/pr-s34c2`, base `origin/main @ 7718d952`)

**Authority.** This design's banner (frozen 2026-10-19) and the dated note of 2026-10-27 (Q-S34c3-7,
ruled with S34-c3's freeze) widen S34-c2. Every prerequisite is merged: S33-b #395, S34-b #363, S34-c1
#368, S33-f #462 (and S33-e2: the halt's benches are on main). Executed by a fresh session (Claude Opus
5.5 as the lane owner). S34-c3 is not on main; it runs after this PR (dated note).

**Audit (base `7718d952`).**

| # | Fact | Where |
| --- | --- | --- |
| A-c2-1 | `world.yaml` has no `posture`; no place file has `sit-points:` | `worlds/lakeside/world.yaml:47-66`; `worlds/lakeside/places/*.yaml` |
| A-c2-2 | Pier bench solid (450–3 550, 300–750), furnishing `bench.head` facing 180 000; the deck's south edge is y 0 | `pier.yaml:33-45` |
| A-c2-3 | Square: trunk (11 800–12 200, 8 800–9 200); `bench.west` (10 900–12 500, 9 600–10 000) facing 180 000; `bench.east` (12 600–13 000, 8 100–9 900) facing 270 000 | `square.yaml:25-41` |
| A-c2-4 | Bakery chairs `long-table.chair-1…5`, boxes x 900–1 400 centred y 1 800 … 4 200, facing 270 000 | `bakery.yaml:24-46` |
| A-c2-5 | Halt bench row `bench.south` (1 500–6 500, 0–450) facing 0; platform benches (−3 950…−3 500, ±4 000…±6 000) facing 90 000 | `station.yaml:20-27`; `station-platform.yaml:21-31` |
| A-c2-6 | 3D: `bench.pier` and `bench.park` are drawn by the generic backless `form: bench`, which ignores the furnishing's yaw. **2D (corrected after red-team F1): the 2D client draws them with the `bench` sprite (prefix to `furnishing.bench`) and mirrors it by the furnishing's `facing`** | `presentation/mineworld-default/3D/furniture/furnishings.yaml:81-82`; `clients/3d-spike/scripts/slice/furnishings.gd:264-285`; `presentation/mineworld-default/2D/assets/asset_bindings.yaml:99`; `clients/2d/scripts/scene/furnishings.gd:113-116` |
| A-c2-7 | `sit_points/lakeside.yaml` holds 29 depictions with `fallback` positions (S33-f's table; halt 7 + 6) | `presentation/mineworld-default/3D/sit_points/lakeside.yaml` |
| A-c2-8 | Both clients take the posture branch when the place discloses `sit-points`; the 3D pose reads the disclosed `at`/`facing`; a depiction without `fallback` is tolerated (`at` null) | `clients/shared/sitting/sit_rule.gd:58-72, 96-106`; `clients/3d-spike/scripts/slice/seating.gd:47-60` |
| A-c2-9 | `ARC-172` / R-S34c-f-1 require S34-c to delete the fallback positions and run the contested seat in the posture branch | `docs/decisions/ARC-172.md`; `pr-s33f-interiors-sit-points.md` §10 |
| A-c2-10 | `posture` declares dependencies on presence and movement only | `systems/posture/src/system.rs:58-62` |

**Bounded corrections (recorded; scope, invariants and acceptance unchanged).**

- **BC-c2-1 (Q-S34c3-7, the wrong side).** The pier bench and both square benches were authored facing
  the side nobody can stand on: a south-facing pier point would stand at y 40 (needs ≥ 250 from the edge);
  a south-facing west-bench point at y 9 340 is 172 mm from the trunk, a west-facing east-bench point at
  x 12 340 is 140 mm from it (needs 260). The 3D benches are backless and yaw-independent (A-c2-6), so
  the drawing does not pick a side; the geometry does. The furnishings' `facing` is the wrong side and is
  corrected: `bench.head` 180 000 → 0; `bench.west` 180 000 → 0; `bench.east` 270 000 → 90 000. The sit
  points keep S33-f's sides (north / north / east). Moving the pier bench 220 mm north instead
  (Q-S33f-2 (B)) was ruled out by Q-S33f-2 (A).
  **Correction (red-team F1, `_reviews/pr553.md`): the claim "no drawing change" held for 3D only.** The 2D
  client mirrors the bench sprite by `facing` (A-c2-6), so all three benches flip on screen in 2D. That flip
  is the intended consequence: the sprite now faces the side people sit on. Evidence: before (base facings
  on the head world) and after (head) stills, the same seat (Finn), the same walk (`steps-2d.json`, via
  `--drive=steps --capture`, windowed Godot 4.7, `shoot2d.sh`), cropped round the benches, in
  [`s34c2-evidence/`](s34c2-evidence/): `pier-before.png` / `pier-head.png`, `square-before.png` /
  `square-head.png`. The pier bench's sprite visibly mirrors, and so do the square benches.
- **BC-c2-2 (SD-c-2 on merged data).** S33-f's three points on the square's north bench assumed a 1.8 m
  bench; main's is 1.6 m (S33-f D-S33f-2 kept three). One person per 600 mm gives floor(1 600 / 600) = 2,
  centred: (11 400, 10 260), (12 000, 10 260). The depiction file loses `tree-bench-north.3`.
- **BC-c2-3 (Q-S34c3-4, ruled as recommended: "S34-c2 adopts the tool's count").** The halt's 5 m bench
  row holds floor(5 000 / 600) = 8, not S33-f's 7: x 1 900 … 6 100 step 600, y 710. The depiction file
  gains `wall-bench.8`.
- **BC-c2-4 (keys).** Keys and groups stay S33-f's (`pier-bench.n`, `tree-bench-north.n`,
  `long-table.n` grouped by chair, …), numbered west-to-east then south-to-north (SD-c-3), because the
  depiction file binds by them. S34-c3's generator (`<furnishing>.<i>`) does not regenerate Lakeside's
  hand-authored sections (its §1 Out).
- **BC-c2-4b (Q-S34c3-6's assignment, red-team F5).** Q-S34c3-6 (ruled) says "S34-c2 adds the chair
  form for Lakeside's bakery and regenerates S34-c1's sections with the tool". The generator is not on
  main, and the dated note runs S34-c3 after S34-c2, so S34-c2 cannot do either. Disposition: both items
  move to S34-c3 (or the first PR after it that adds the chair form), and S34-c2 hand-authors the
  bakery's five chair points by SD-c-1's "behind" case. A dated note in `pr-s34c3-station-sit-points.md`
  records the reassignment.
- **BC-c2-5 (ARC-172).** Fallback positions deleted from `sit_points/lakeside.yaml`; the contested-seat
  fixture now discloses the bakery's `sit-points` and 7's `posture-state` from at 102 (held on the point's
  `at`); expectation unchanged (7 at `long-table.3`, nobody at 100/101). The comments that described the
  removed fallback were brought up to date: `clients/2d/scripts/scene/seating.gd`,
  `clients/3d-spike/scripts/slice/seating.gd`, `clients/shared/sitting/sit_rule.gd`. These are comment-only
  edits, made at the coordinator's request; `clients/` belongs to S34-e. No client code changes. Follow-up
  for S34-e (red-team F6): no shipped data reaches the fallback branch any more, so decide there whether
  to keep it, for worlds that have seats drawn but no `posture`.
- **BC-c2-7 (AC-c-3 / I-c-2 for Lakeside, red-team F2).** AC-c-3 says that removing `posture` and its
  sections gives the base genesis. For Lakeside this holds against "base + BC-c2-1's three furnishing
  facings", not against `7718d952` itself. `bodies` discloses furnishing `facing`
  (`systems/bodies/src/furnishing.rs:136,156`), and Q-S34c3-7 assigned this facing correction to S34-c2
  ("the bench's `facing` in the place file, or the sit points' `facing` and `at`"). The two options are
  either a bodies-owned data change, or putting the points on the side nobody can reach, which I-c-1
  forbids. The generic test (`systems/posture/tests/worlds.rs`) compares the stripped head with the full
  head minus posture's facts. That is the removability property AC-c-3 is for (posture leaves nothing
  behind), measured against the corrected base. Social Café and Market Town are unaffected. This is
  recorded as a deviation within Q-S34c3-7's ruling; if the operator reads AC-c-3 literally, it is theirs
  to rule.

**Final table (29 points).** pier 5 (800 … 3 200, 1 010; 0); square north 2 (11 400, 12 000; 10 260;
0), east 3 (13 260; 8 400, 9 000, 9 600; 90 000); bakery 5 (1 660; 1 800 … 4 200; 270 000); station 8
(1 900 … 6 100, 710; 0); platform 3 + 3 (−3 240; ±4 400, ±5 000, ±5 600; 90 000). Each pelvis
(`at` − `seat_offset_m` × facing) lies over its prop's solid.

**Commit plan (S34-c2).**

- C1 — data: `world.yaml` (`posture`), the five place files (`sit-points:`, bench `facing`), the depiction
  file, `ARC-147` and `ARC-172` notes, this ledger.
  - [x] Implementation (`11c0bb27`) · [x] Validation: `mineworld-posture` `worlds` 2/2 and `shipped` 3/3 PASS on the C1 tree (the generic test now covers Lakeside: it loads under S34-b's check, every point is within reach of a solid, removability holds) · [x] Review: every point traced to the table and a solid; bench facings corrected only where the geometry forbids the authored side (BC-c2-1); no code changed
- C2 — tests: `tools/cli/tests/lakeside_sit_points.rs` (world points = table; one depiction per point; no
  fallback; clearance and pelvis), the contested fixture in the posture branch, `client_sit_rule.rs` text.
  - [x] Implementation (`8018d593`; AC-b-7's helper also strips `posture` and `sit-points:`, BC-c2-6) · [x] Validation: see the log · [x] Review: the test reads place files as text (an independent oracle), names no value the System computes; mutations M-c2-1/2 red
- C3 — Lakeside re-baseline (`scripts/baselines.txt` `[world lakeside]` only) and any Lakeside golden the
  change moves.
  - [x] Implementation: `[world lakeside]` only, from one local long record of `8018d593` (Darwin/arm64,
    `ci_parity.py record --profile long`, 2 687 s; Python 3.12, since the slot's `/usr/bin/python3` 3.9 lacks
    `tomllib`); `baseline write` was run on a scratch copy and only Lakeside's six keys were copied over, with
    a `reason S34-c2` line. `doorways-only-30d-seed7.txt` did not move (BC-c2-6).
    · [x] Validation: `baseline check` of the record against the new file reports no Lakeside difference.
    The same record differs from main's committed sections for bodies-yard, market-town and social-cafe too
    (bodies-yard has no posture, so this is main's drift since those were last recorded). Those sections are
    untouched (AC-c-7 holds for this PR: bodies-yard's data is unchanged); that drift is left to main's next
    re-record. A 30-day seed-7 Lakeside run: `sat-down 45`, `stood-up 45`, `faults 0`.
    · [x] Review: only Lakeside's section changed (`git diff scripts/baselines.txt`: 7+/6−); the reason line
    says that main's drift is folded in, as S33-b's did.

**Validation log.** (filled below as runs complete). Tree identities: C1 = `11c0bb27`; C2 = `8018d593`
(the test runs below were made on the C1 tree plus C2's working changes, then re-run after `cargo fmt` on
C2 itself); C3 = `f14f7649` (adds only `scripts/baselines.txt` and the ledger, nothing executable). The
release binary used for the long record and the 300-day runs was built from `8018d593`. This PR changes no
production Rust (`git diff 7718d952 HEAD -- '*.rs'` touches only `tools/cli/tests/*.rs`), so that binary is
also the base's. Remote CI: under the operator's relevant-tests policy (2026-11-27), only `fast` (and its
small companions) run per PR, and the other jobs report `skipping` by design (red-team F3). The local runs
listed here are the relevant tests.

- **F4: Lakeside's `facts-300s` / `journal-300s` drop (11 936 → 6 515, 7 979 → 4 214).** These keys count
  the rows of the 300-day save's live, unsealed tail (ARC-89: `facts`/`journal` hold the tail and `sealed`
  the rest; `scripts/ci_parity.py:103-107`). They do not count the facts. The same record against main's
  committed sections shows the tail moving just as much in a world this PR does not touch: Social Café
  10 422 → 6 104 facts-300s (−41 %) and 7 713 → 4 329 journal-300s; Market Town 6 135 → 8 717 (+42 %).
  So it is where the seal boundary falls on main today, not sitting. The behaviour change, measured
  directly with the same binary, seed 7, 300 days in memory (`s34c2-evidence/lakeside-300d-{base-7718d952,head}.txt`):
  - total facts 827 823 → 829 270 (+0.17 %);
  - new `sat-down` 507, `stood-up` 507 and `place-sit-points` 5;
  - `walk-started` 97 233 → 97 188, `conversation-started` 57 538 → 57 299, `fishing-started`
    4 360 → 4 310, `arrived` 320 180 → 321 544;
  - others within ±1 %.

  Residents sit about 1.7 times a day across the village and walk, talk and fish slightly less. No fact
  kind disappears.

- **BC-c2-6 (AC-b-7 in `lakeside_bodies.rs`).** `without_bodies_only_the_doorways_moved` FAILED on the C1
  tree (day-1 facts 2 507 vs the fixture's 2 502): removing `bodies` left `posture` and its sit points in,
  which S33-b's recording (before posture) does not describe. Fixed in the test, not the fixture: a new
  `without_bodies_or_posture` also removes `posture` and every `sit-points:` (I-c-2), so the evidence
  recorded at `30b7c521` still applies unchanged. NW-3's comparison keeps `without_bodies` (posture on both
  sides).

| Check (tree) | Verdict | Evidence |
| --- | --- | --- |
| `cargo test -p mineworld-posture --test worlds --test shipped` (C1) | PASS | 2 + 3 passed |
| `cargo test -p mineworld-cli --test lakeside_sit_points` (C2) | PASS | 3 passed |
| `--test lakeside_bodies` (C2) | PASS | 4 passed (AC-b-7 after BC-c2-6) |
| `--test milestone_e --test furnishings --test layout --test needs_sections --test sleep_sections --test newsstand --test activity_length` (C1 + tests) | PASS | 7, 5, 4, 6, 2, 6, 1 passed |
| `--test ac13_clients` | INCONCLUSIVE | 3 ignored (need Godot/the clients; not run) |
| `--test price_reporter` `clerk_publishes_and_keepers_follow` | FAIL, pre-existing | same failure ("no keeper moved away", 6 vs 0) at base `origin/main @ 7718d952` with this PR's changes stashed; it reads Social Café and Market Town only. Not this PR's; reported |
| `cargo test -p mineworld-cli --test client_sit_rule -- --ignored` (Godot 4.7, C2) | PASS | 2D and 3D both `SEATED 102 7 long-table.3`, `SEATED 103 7 long-table.3`; nobody at 100/101 |
| M-c2-1: `pier-bench.2` moved to y 700 (into the bench) | PASS (red) | all three `lakeside_sit_points` tests red; load refused `bodies-fixed-point-in-solid`; "0 mm from the solid (450, 300, 3550, 750)"; reverted |
| M-c2-2: depiction `tree-bench-east.3` deleted | PASS (red) | "square/tree-bench-east.3: no depiction"; one-to-one assertion red; reverted |
| `cargo fmt --all -- --check`; `cargo clippy -p mineworld-cli --test lakeside_sit_points --test lakeside_bodies --test client_sit_rule -D warnings` | PASS | clean |
| 3D `--seat` probe (`ci_godot.py slice --world=lakeside --seat`) | NOT RUN | needs a windowed Godot and a server; the depiction binding it reads is pinned by `lakeside_sit_points`; left to the clients-probes CI layer |

## 12. Operator questions (each with a recommended answer)

**Ruled 2026-10-19 (operator, primary session):** every question below is answered as recommended.

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34c-1** | Lakeside has no furniture solids yet (S33-b unmerged). Split S34-c into c1 (Social Café, Market Town, now) and c2 (Lakeside, after S33-b)? | **Yes, split.** c1 does not wait; c2 writes S33-f's table verbatim inside Lakeside's single re-baseline |
| **Q-S34c-2** | Café chairs and stools have a table or ledge in front, so there is no free floor there. Put the sit point behind the chair (aisle side) and let the depiction pull the pelvis forward onto the seat (a signed offset instead of the step's "back by seat depth")? | **Yes.** The server keeps people on free floor (Q-S34-3 holds); S34-f's depiction field becomes a signed offset |
| **Q-S34c-3** | Enable `posture` in bodies-yard, which has no seats? | **No.** Enable only where a seat is drawn; bodies-yard stays byte-identical |
| **Q-S34c-4** | The 30-day seeded runs (AC-S34-6) — here or in S34-d? | **S34-d.** S34-c runs a 300-step invariant check; S34-d tunes NPC sitting and runs 30 days |
| **Q-S34c-5** | The café window bench cannot be reached (the drinks ledge stands 50 mm in front of it). Leave it without a sit point, or narrow the ledge? | **No sit point, recorded.** I-S34-9 covers seats a person can reach; moving furniture is a visual change for the S36 lane |
