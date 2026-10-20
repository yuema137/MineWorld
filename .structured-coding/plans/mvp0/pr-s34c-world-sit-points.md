# PR S34-c — sit points in the shipped worlds, and `posture` enabled there

## Lifecycle

```text
Design revision:        revision 1 (2026-10-19), DRAFT
Approved by / evidence: none yet — awaiting the operator's freeze (questions in §12)
Implementation base:    origin/main @ 5a8c6f28 (after #301, S34-a)
Start condition:        S34-b merged (the load-time check that a sit point clears every Solid);
                        for the Lakeside half (S34-c2): S33-b merged (Lakeside's `body:` sections)
Execution contract:     §10
Lifecycle:              DRAFT — awaiting freeze
```

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

## 12. Operator questions (each with a recommended answer)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S34c-1** | Lakeside has no furniture solids yet (S33-b unmerged). Split S34-c into c1 (Social Café, Market Town, now) and c2 (Lakeside, after S33-b)? | **Yes, split.** c1 does not wait; c2 writes S33-f's table verbatim inside Lakeside's single re-baseline |
| **Q-S34c-2** | Café chairs and stools have a table or ledge in front, so there is no free floor there. Put the sit point behind the chair (aisle side) and let the depiction pull the pelvis forward onto the seat (a signed offset instead of the step's "back by seat depth")? | **Yes.** The server keeps people on free floor (Q-S34-3 holds); S34-f's depiction field becomes a signed offset |
| **Q-S34c-3** | Enable `posture` in bodies-yard, which has no seats? | **No.** Enable only where a seat is drawn; bodies-yard stays byte-identical |
| **Q-S34c-4** | The 30-day seeded runs (AC-S34-6) — here or in S34-d? | **S34-d.** S34-c runs a 300-step invariant check; S34-d tunes NPC sitting and runs 30 days |
| **Q-S34c-5** | The café window bench cannot be reached (the drinks ledge stands 50 mm in front of it). Leave it without a sit point, or narrow the ledge? | **No sit point, recorded.** I-S34-9 covers seats a person can reach; moving furniture is a visual change for the S36 lane |
