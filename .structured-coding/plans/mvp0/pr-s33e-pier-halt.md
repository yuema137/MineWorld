# PR S33-e — The pier (Lakeside's landmark) and the halt (its station) in 3D

## PR DESIGN (DRAFT) — not frozen

```text
Design revision:        revision 1 (2026-10-18), DRAFT
Author:                 S33 lane lead (Opus), worktree /Users/yuema137/mineworld-worktrees/design-s33de,
                        branch docs/pr-designs-s33d-j, planning base origin/main @ a1e05874
Approved by / evidence: none yet. The step binds; Q-S33-1…11 and Q-S29a-5…7 are not re-asked; §14 is open
Implementation base:    origin/main at C0
Start condition:        S33-e1 (the pier): S33-d merged; the landmark check present (S29-d's, or this PR's per
                        Q-S33e-2). S33-e2 (the halt): S33-e1 merged; S29-e merged (or Q-S33e-1's fallback);
                        S29-a merged. Both: §14 ruled; this design frozen; the lane lead's dispatch. NONE holds
                        at 2026-10-18
Execution contract:     §12
Implementer:            S33-e1 Opus-class (the pier is the landmark: art judgement); S33-e2 Haiku-class from
                        this commit plan (S29-e's pattern), Opus frame review (step-33 §12.1)
Lifecycle:              PR DESIGN (DRAFT)
```

**Effort:** `mvp0`, release-blocking · **Step:** [`step-33-lakeside-3d.md`](step-33-lakeside-3d.md) §§1 (outcome
3), 4.3, 4.4, 4.6, 9.2 (landmark), 9.3 (the route), 9.4 (X-3, X-7, X-9), 10 (I-S33-5), 11 (AC-S33-2, AC-S33-6),
12.1 (the S33-e row) · **Consumes:** [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md)
(grading, the quay line), [`pr-s33b-lakeside-bodies.md`](pr-s33b-lakeside-bodies.md) (the pier's and the halt's
floors and solids), [`pr-s29a-town-contract.md`](pr-s29a-town-contract.md) (the halt's places, rev. 3),
[`step-29-station-landmark.md`](step-29-station-landmark.md) §§5.1, 9.2 (the state rule, the landmark check), S29-e
(the shared station pieces) · **Binding parents:** `CLAUDE.md` §§1, 2, 4; `docs/ENGINEERING_RULES.md` §§4–9;
`docs/VISUAL_SLICE.md`.

**Decision ids.** `ARC-171` (§5.6: the honesty scan — no moving prop without a disclosed Process — as a standing
client rule), written by C1. No other id.

No model is called. No money is spent.

---

## 0. Binding inputs, exactly restated

- **Scope (step §12.1, S33-e row).** "pier, rails, pavilion, moored boats; the halt building, window bay,
  platform, track and buffer stop with S29-e's shared pieces; `--slice-landmark` for Lakeside". Depends on S33-d;
  **S29-e merged**.
- **Integration checkpoint.** "LM-1/LM-2 pass; the route reaches the pier head and the platform; the board reads
  `no_service`."
- **Adversarial.** "X-3, X-7, X-9." X-3: remove the pier ↔ lane passage → S29's landmark reachability check at
  load, and the link route. X-7: animate a rowing boat across the lake → the honesty scan. X-9: hide the pavilion
  and pier → `FAIL landmark … LM-1`.
- **The pier (step §4.3).** Deck 4 × 20 m running south; round posts in pairs at ≈ 3 m; rails "no higher than 34
  in (865 mm) so that a seated angler can fish over them, with edge protection" (2010 ADA §1005); a pavilion ≈ 4
  × 4 m with the world's bench at the end, a lantern on a post; two or three rowing boats **moored, never
  moving**.
- **The halt (step §4.4; S29-a rev. 3 as ruled).** A small timber waiting room at the lane's **east** end, the
  ticket bay 4.6 × 4.6 m, a 40 × 4 m north–south platform with the track on its **east** edge, a buffer stop at
  its south end against the shore; S29-e's pieces; `station_state.gd`: board "No service yet", window shut,
  kiosk shut (Lakeside has no economy); no train (S29 §5.2).
- **The state rule (step-29 §5.1).** Every sign of service is drawn from disclosed state. **The clock (step-29
  §6.2, the fit-out table's clock row):** "It shows no time when the world has no calendar, rather than the host's
  wall time" — Lakeside has none.
- **Operator rule:** every town has a station and a landmark; real values with a cited source.

---

## 1. Goal

From the lane a player sees the pier run south over the water to a small shingled pavilion, walks to its head,
sits on the bench there (S33-f), and looks back at the village. At the lane's east end a timber halt stands
beside a platform whose track ends at a buffer stop by the water; inside, the window is shut and the board says
there is no service yet — because the world has none. Nothing moves that the world does not move.

---

## 2. Scope

The PR is split in two (Q-S33e-1, recommended), so the landmark does not wait for the depot's chain:

### 2.1 In scope — S33-e1, the pier

1. `scripts/worlds/lakeside/pier.gd` (`LakesidePier`): deck, bents, bracing, rails, edge curb, foot ramp, pavilion,
   lantern, moored boats; every node tagged `mw_category = "landmark.pier"`.
2. `scripts/worlds/lakeside/depiction.gd`: the pier entry drawn as ROOM (outdoor) with its door; viewpoints
   `landmark` (VP-L-2 south, VP-L-4, VP-L-7); perf view `pier head looking north`.
3. The landmark check for Lakeside: `--slice-landmark` with the category parameter (Q-S33e-2).
4. The honesty scan: check (e) in `tests/acceptance/tests/client_rules.rs` (§5.6, X-7).
5. `tools/cli/tests/client_far_field.rs` (or the landmark test file S29-d adds): the Lakeside landmark case.
6. `shots/lakeside/landmark/`.

### 2.2 In scope — S33-e2, the halt

7. `scripts/worlds/lakeside/halt.gd` (`LakesideHalt`): the waiting room, the ticket bay, the platform, the track
   and buffer stop, with S29-e's pieces; nodes tagged `mw_category = "station.halt"`.
8. `depiction.gd`: the three station entries bound by tag (`station`; `station` + `ticket-office`; `station` +
   `platform`); perf view `halt platform`; the probe route extended to the platform.
9. The board, window and kiosk through `clients/shared/station/station_state.gd` (S29-e's, or Q-S33e-1's
   fallback).
10. `shots/lakeside/review/` halt frames.

### 2.3 Not in scope

Interiors' furniture beyond the halt's benches and counter; sit points (S33-f); people (S33-g); 2D (S33-h); any
world, System or protocol change (I-S33-1). If the halt's merged passages cannot be drawn (F-S33e-1), that is
S29-a's to fix, not this PR's.

---

## 3. Audit (`origin/main @ a1e05874`)

### 3.1 Source anchors

| # | Fact | Where |
| --- | --- | --- |
| A-1 | The pier's floor (pier frame): (0, 0)–(4 000, 20 000), the foot (door) on the north edge (here (2 000, 19 600) after S33-b's move), the head at y 0; solids: the bench (400, 300)–(3 600, 750) h 450; the pavilion's four posts 200 × 200 at (200, 200), (3 600, 200), (200, 3 600), (3 600, 3 600), h 2 600 | `pr-s33b-lakeside-bodies.md` §§5.2, 5.3 |
| A-2 | Pier frame → scene: x = 1 + px/1000, z = 23 − py/1000 (lane x 16 000 + px, lane y py − 20 000; S33-c §5.5's translation). The deck is x 1…5, z 3…23; the head at z 23 | arithmetic on A-1 |
| A-3 | The grading S33-d proposes: lane y 0, full pool y −1.00, deck y −0.30, a 1:12 foot ramp over 3.6 m (Q-S33d-1); the quay line z 4 east of the pier | `pr-s33d-terrain-water-exterior.md` §5.2, F-S33d-2 |
| A-4 | The spike's jetty and rowing boat (procedural; boat at `WATER_Y + 0.22`, yaw 0.35) | `scripts/town.gd:553–566` |
| A-5 | S29-a rev. 3, Lakeside: `station` here (400, 2 500) / lane there (29 600, 3 000), ≈ 8 × 5 m; `station-tickets` here (1 500, 400) / station there (6 000, 400), bay 4.6 × 4.6 m; `station-platform` here (−3 600, −16 000) / station there (7 600, 2 500), floor x −4 000…0, y −20 000…20 000, track on the east edge (x 0), buffer stop at y −20 000 | `pr-s29a-town-contract.md`, Lakeside's halt table |
| A-6 | Halt → scene: the waiting room x 15…23, z −2.5…2.5; the platform x 23…27, z −36…4 (its south end at the quay line, A-3); the track's near rail ≈ x 27.9 | arithmetic on A-5 |
| A-7 | No `clients/shared/station/` exists on `main`; no `--slice-landmark` exists; S29-d and S29-e have no PR design (step-29 §11 rows only). S29-d waits for 16c and RL-b2; S29-e follows S29-d | `ls clients/shared`; `grep`; `step-29-station-landmark.md:724–750` |
| A-8 | The landmark check's method: S hides nothing, H hides `mw_category = "station.*"`; LM-1 ≥ 60 % of the open pixels differ in a 20° azimuth window; LM-2 ≥ 60 % of the footprint's pixels from a far-field viewpoint | `step-29-station-landmark.md` §9.2 |
| A-9 | Lakeside has no `calendar` System and no `economy` | `worlds/lakeside/world.yaml` |
| A-10 | RL-e0's far-field view already reserves `station` and `landmark` slot names | `scripts/slice/far_field_view.gd:487` |

### 3.2 Findings

| ID | Finding | Resolution |
| --- | --- | --- |
| **F-S33e-1** | **The ticket bay's passage points cannot be drawn.** S29-a rev. 3 keeps revision 2's bay points: `here` (1 500, 400) in the bay and `there` (6 000, 400) in the waiting room — both 400 mm inside each room's **south** edge. A shared wall needs the two points on opposite sides of it, so one of them must lie near its room's north edge. With `here` at x 1 500 and `there` at x 6 000, the bay spans scene x 19.5…24.1 whichever side it is drawn on, so it overlaps the platform (x 23…27) by 1.1 m; drawn south of the waiting room (scene z 2.5…7.1) it also stands over the water (the quay is at z 4, A-3) | **Requirement on S29-a (R-S29a-e-1, through the coordinator):** the bay north of the waiting room, `station-tickets` here (2 600, 400) / `station` there (6 000, 4 600). The bay is then x 18.4…23, z −7.1…−2.5: inside the halt's east line, clear of the square (x ≤ 15) and the platform. S33-b's disjoint-floors check (AC-b-6) would also catch the current points |
| **F-S33e-2** | **The landmark check does not exist yet** (A-7), and it is S29-d's, which waits for 16c and RL-b2 | Q-S33e-2: whichever of S29-d and S33-e1 lands first writes it with the hidden category as a parameter |
| **F-S33e-3** | **S29-e's shared pieces do not exist** (A-7), and S29-e is behind S29-d | Q-S33e-1: split the PR; the halt waits for S29-e, or lands `station_state.gd` itself if S29-e is later |
| **F-S33e-4** | **The track leads into the woods.** A track running north out of the frame suggests a way out, which step §4.2 forbids ("nothing past the boundary suggests a path") | The track enters a shallow cutting at the woods' edge (z ≈ −60) and is closed visually by the trees; the server never lets anyone onto it (the platform's east edge is a wall). Recorded as a stated exception: a railway line leaving a terminus is what a halt is |
| **F-S33e-5** | Lakeside has no calendar (A-9) | The halt's clock face shows no hands (step-29 §6.2, the clock row), not the host's time |

### 3.3 What does not exist

No pier, halt, landmark check, honesty scan or shared station state in the 3D client.

---

## 4. Reuse, both ways

| Need | Decision | Why |
| --- | --- | --- |
| Pier structure | our builder from boxes and cylinders, `weathered_brown_planks` | the town's one method (S29 §6.1); a pier is posts and planks |
| Boats | the spike's procedural rowing boat (A-4), moved into `pier.gd` | already shipped |
| Station pieces | S29-e's (window bay, board, bench row, `station_state.gd`) | the step requires them (step-33 §16.2's note to S29-e: written for a village-scale building) |
| Landmark check | S29 §9.2's method, one tool for both worlds | no second checker |
| Rails | `Props.railing` with a height argument | exists (`props.gd:438`) |

No new texture; no dependency.

---

## 5. Design

### 5.1 The pier (S33-e1)

| Element | Specification | Source / status |
| --- | --- | --- |
| Deck | 4.0 × 20.0 m at y −0.30 (Q-S33d-1 A); `weathered_brown_planks` laid across, boards 140 mm wide (a nominal 2 × 6, actual 1½ × 5½ in, 38 × 140 mm) with 6 mm gaps | NIST *American Softwood Lumber Standard* PS 20 (nominal sizes); the gap approximate |
| Foot | a 1:12 ramp from the lane (y 0) to the deck (y −0.30) over the first 3.6 m; the lane is its top landing and the deck its bottom landing (≥ 1 525 mm long, §405.7); because the rise (300 mm) exceeds 150 mm, the ramp has handrails on both sides (§405.8): the pier's 865 mm rails, drawn as graspable round rails (§505.7) within §505.4's 34–38 in band, run down the ramp | ADA §§405.2, 405.7, 405.8, 505.4, 505.7 |
| Bents | pairs of round piles at ≈ 2.86 m centres (**8 bents** over the 20 m deck), 300 mm diameter, from the lake bed to the deck; cap beams and X-bracing below the deck | round timber piles per ASTM D25 (the class sizes); spacing approximate, ≈ 10 ft spans for timber stringers |
| Rails | both sides and the head, top at **865 mm** above the deck (34 in) | 2010 ADA Standards §1005.2.1: "at least 25 percent of the railings, guards, or handrails shall be 34 inches (865 mm) maximum above the ground". No guard is drawn because the deck stands 0.70 m above **full pool**; IBC §1015.2 measures the drop to "the floor or grade below", here the lake bed, so reading the water surface as the grade is a **stated approximation** (the lake bed lies ≈ 1–3 m lower). §1005.2.1's exception (a complying guard replaces the 34 in rail) is the fallback if the reading is ruled out |
| Edge protection | a 51 mm (2 in) curb along both edges | ADA §1005.3.1 (curbs 2 in minimum above the surface); C0 re-quotes the clause from the Access Board's text |
| Clear space and fishing positions | fishing positions along the **two sides only**: 760 × 1 220 mm (30 × 48 in) clear in front of each side position, nothing drawn on the deck there. The head rail stands behind the bench and the pavilion's posts, so it is not a fishing position. The side rails alone are far more than §1005.2.1's 25 % | ADA §1005.4 and §305 |
| Pavilion | 4.0 × 4.0 m over the head, four 200 × 200 mm posts at S33-b's solids (A-1), height to eaves 2.6 m above the deck, eaves overhanging 0.3 m all round; a gabled shingle roof (S33-d's shingle, or `roof_slates_03`) with its **ridge running east–west**, so the roof's slope, not its gable end, faces the lane (§5.3); ridge 4.2 m above the deck | world data (posts); reference 06; approximate |
| Bench | the world's bench at the end, S33-b's solid (400, 300)–(3 600, 750), seat 450 mm (`painted_wooden_bench` scaled to 3.2 m, or our slat bench) | `VISUAL_SLICE.md` §3.4 (0.42–0.47 m); S33-f adds the sit points |
| Lantern | hung from a bracket on the pavilion's south-east post (a disclosed solid, A-1), so nothing stands on the deck that the server does not know; lit only by the fixed sun's ambient (no emissive glow at day) | reference 06 |
| Boats | two rowing boats moored in the water at the head's east side, one drawn up on the stone revetment west of the pier's foot (S33-d F-S33d-2), outside every place's floor; ≈ 4.3 m (14 ft) long, the spike's hull scaled; tied by drawn lines to the bents; **static**. The step put one "at the boathouse slip"; there is no slip, because the boathouse stands on the land side of the lane (S33-d F-S33d-3) — a recorded deviation | reference 06; length approximate |
| Category | every pier and pavilion node `mw_category = "landmark.pier"`; the boats `landmark.pier.boat` | §5.3 |

### 5.2 The halt (S33-e2)

| Element | Specification | Source / status |
| --- | --- | --- |
| Waiting room | 8 × 5 m, scene x 15…23, z −2.5…2.5; timber frame, board-and-batten walls, shingled roof with a deep canopy (2.5 m) over the platform side; west door onto the lane, east door onto the platform | step §4.4; S29-a A-5 |
| Ticket bay | 4.6 × 4.6 m, north of the waiting room per R-S29a-e-1 (if S29-a does not adopt it, C0 draws what merges or stops); the window bay piece from S29-e, shutter **closed** | S29-a ruling Q-S29a-6; S29 §5.1 |
| Benches | the bench row along the waiting room's far wall and on the platform, S33-b's solids, seat 450 mm | S33-b §5.3 |
| Board | S29-e's board piece through `station_state.gd`: "No service yet" (`station.no_service`) | S29 §5.1 |
| Clock | S29-e's clock, **no hands** (no calendar, F-S33e-5) | step-29 §6.2 (the clock row) |
| Platform | 4 × 40 m, x 23…27, z −36…4; surface at y 0.00 (the lane's level); the rail head at y −0.20, so the platform stands 0.20 m above the rails (a low heritage-line platform; approximate); granite edge stones | S29-a A-5; approximate |
| Track | standard gauge, **1 435 mm** between the rails' inner faces; timber sleepers at 0.6 m; the near rail ≈ 0.9 m from the platform edge | the standard gauge (UIC / AREMA standard gauge, 4 ft 8½ in); sleeper spacing approximate |
| Buffer stop | a timber-and-steel bumping post at z 3.5, inside the track's end, the quay line (z 4) just beyond (S33-d R-S33e-d-1) | step §4.4 |
| Category | `mw_category = "station.halt"` on every halt node | S29 §9.2 (H hides `station.*`) |
| No train | none drawn | S29 §5.2, I-S33-5 |

### 5.3 The landmark check for Lakeside (Q-S33e-2)

S29 §9.2's method with the hidden category `landmark.*`.

**Why the depot's LM-1 window cannot be reused unchanged.** Step-29's LM-1 takes a fixed 20° window between
the street's eaves line and the horizon, sized for a 52 m depot closing a street. Lakeside's landmark is a 4 m
pavilion at the end of a 20 m pier. Seen from VP-L-2 (0, 1.66, 0), its eaves outline (x 0.7…5.3, z 18.7…23.3)
spans bearings **164.2°–178.3° (14.1°)**; the roof's north slope runs from the eaves at **≈ 1.9°** above the
horizon to the ridge at **≈ 6.0°**; under the eaves only the posts (≈ 11 % of the width) and the lantern block the
view, and the deck, rails and bench lie below the horizon. So, computed at design time:

| Window (band: horizon to 6.0°) | Share of open pixels the pavilion covers | Passes 60 %? |
| --- | --- | --- |
| step-29's fixed 20°, centred on the pavilion (171.2°) | ≈ 51 % (≈ 61 deg² of 120 deg²) | **no** |
| the pavilion's own extent, 164.2°–178.3° (Q-S33e-4 A) | ≈ 72 % (roof 68 % of the band, posts ≈ 4 %) | yes, by ≈ 12 points |

A gable end facing the lane would cover about half of the roof band and fail both; hence the ridge runs east–west
(§5.1). These figures are estimates from the geometry; C3 measures the real share and records it.

- **LM-1 (Q-S33e-4, recommended A):** from VP-L-2, the window is the landmark's own azimuth extent computed from
  its drawn outline (164.2°–178.3° here), the band from the horizon to the outline's top (≈ 6.0°): ≥ 60 % of the
  pixels open in H differ in S. The window is computed from the geometry, never tuned to a result (R-e-2).
- **LM-2:** from VP-L-4 (the shore's water line) and VP-L-7 (0, 40, 0): the pier's projected footprint differs from
  H in ≥ 60 % of its pixels.
- Verdicts `all landmark checks pass` / `<n> LANDMARK CHECKS FAILED`; failure lines name the viewpoint and the
  criterion (`FAIL landmark VP-L-2 LM-1 …`).

### 5.4 The route

`--world=lakeside --link`'s probe route becomes: spawn → lane → pier foot → **pier head** (place `pier`) → lane →
**halt** (`station`) → `station-tickets` → `station` → `station-platform` → back to the lane. At each doorway the
server's `place` changes; 0 refusals; the board's text equals the `station.no_service` word in the active locale;
NW-3 ≤ 10 % geometry stops (step §9.3).

### 5.5 Boats moored, nothing moving

Each boat is a static mesh with no `AnimationPlayer`, no script and no `_process`. Its line to the bent is a mesh.

### 5.6 The honesty scan (`ARC-171`; X-7)

A rule the step names but no file holds: **no prop moves unless a disclosed Process moves it.** The check, in
`client_rules.rs` (S33-c's scanner, a new check (e)): in a depiction's files under `scripts/worlds/`, a node
tagged `landmark.*` or a prop group (`boat`, `train`, `ferry`) must not be the target of an `AnimationPlayer`,
`Tween` or per-frame transform write. The planted X-7 (a `Tween` on a boat's position) fails naming the file and
line. `ARC-171` records the rule for every world: presentation may idle (water, leaves) but never shows a
behaviour the world does not have.

---

## 6. Invariants this PR holds

| ID | Invariant | Checked by |
| --- | --- | --- |
| I-e-1 (I-S33-1) | No path under `kernel/`, `contracts/`, `systems/`, `server/`, `worlds/`, `clients/protocol/` changes | diff scan |
| I-e-2 (I-S33-5) | No train, no rowing ferry, no open window, no service on the board, no clock time | X-7; the route's board check; review |
| I-e-3 (I-S33-9) | Every drawn pier edge, post and bench, and every halt wall and bench, is within 50 mm of a disclosed face | 12e's `--geometry` on Lakeside (S33-d C9) |
| I-e-4 | The pier is a landmark by the check, not by assertion | AC-e-1 |
| I-e-5 (I-S33-2) | Social Café unchanged | `frame_diff` |
| I-e-6 (I-S33-3) | No place-key literal | check (d) |

---

## 7. Test ownership

| Layer | Owns | Where |
| --- | --- | --- |
| Rendered | LM-1/LM-2 on Lakeside, three OSes | the landmark test (`#[ignore]`), nightly `clients`; per-PR Linux job |
| Probe | the route (§5.4), X-3 | nightly `clients-probes`; local one at a time |
| Load | X-3 at load (S29's `TOWN_LANDMARK_UNREACHABLE`) | `mineworld validate worlds/lakeside` on a planted copy |
| Static | the honesty scan (X-7), check (d) | `cargo test -p mineworld-acceptance --test client_rules` |
| Gate 1 / 2 | NOT REQUIRED | — |

---

## 8. Acceptance criteria (fixed before measuring)

| ID | Criterion | Step ref |
| --- | --- | --- |
| AC-e-1 | LM-1 and LM-2 pass for Lakeside on Linux, macOS and Windows | AC-S33-6 |
| AC-e-2 | X-9 (pier and pavilion hidden) → `FAIL landmark VP-L-2 LM-1` | X-9 |
| AC-e-3 | The route of §5.4 passes: place changes at every doorway, 0 refusals, the board reads `station.no_service` | AC-S33-2 |
| AC-e-4 | X-3 (the pier ↔ lane passage removed in a planted copy) → `TOWN_LANDMARK_UNREACHABLE` at load, and the route fails at the pier | X-3 |
| AC-e-5 | X-7 (a boat tweened) → the honesty scan red by file and line | X-7 |
| AC-e-6 | The rails measure 865 mm ± 5 mm above the deck at 10 sampled points; the curb 51 mm ± 5 mm (`--slice-measure` rows) | §5.1 |
| AC-e-7 | Counts for `pier head looking north` and `halt platform` recorded under step §8.1's advisory figures | §8.1 |
| AC-e-8 | Bytes added ≤ 3 MB | §8.2 |

---

## 9. Commit plan

Procedure: [`pr-s33d-terrain-water-exterior.md`](pr-s33d-terrain-water-exterior.md) §9.0, from
`/Users/yuema137/mineworld-worktrees/impl-s33e1` (S33-e1) and `…/impl-s33e2` (S33-e2).

### S33-e1

#### C0 — Base, prerequisites, ledger

- Record the base; S33-d merged; the landmark check's state (S29-d merged or not, Q-S33e-2); S33-d's ruled grading.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: prerequisites with evidence.
- **Commit message:** `docs(plan): S33-e1 — open the ledger at the base`

#### C1 — `ARC-171`; the clause quotes

- **Files:** `docs/DECISIONS.md` (or `docs/decisions/`), this file.
- Steps: `ARC-171`; ADA §§405.2, 1005.2.1, 1005.3.1, 1005.4 quoted from the Access Board's text, with the date.
- [ ] Implementation. [ ] Validation: doc scripts. [ ] Review: each number in §5.1 matches its quoted clause.
- **Commit message:** `docs: ARC-171 — no prop moves unless the world moves it (S33-e)`

#### C2 — The pier

- **Files:** `scripts/worlds/lakeside/pier.gd`, `depiction.gd`, this file.
- Steps: §5.1, §5.5.
- Validation: AC-e-6; `--geometry` (the posts and the bench against S33-b's solids). Mutation **M-e-1** (the
  rail's height 0.95 m) → AC-e-6's measure row fails naming the rail.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames VR-6 the pier from the lane, VR-7 the village
  from the head).
- **Commit message:** `feat(3d): Lakeside's pier and pavilion, the landmark (S33-e)`

#### C3 — The landmark check

- **Files:** per Q-S33e-2: `scripts/slice/slice_landmark_check.gd` (new, if S29-d has not landed it) or its
  category parameter; the landmark test; `shots/lakeside/landmark/`; this file.
- Validation: AC-e-1 locally once; AC-e-2 (X-9).
- [ ] Implementation. [ ] Validation. [ ] Review: Social Café's landmark verdict unchanged if S29-d's check exists.
- **Commit message:** `test(3d): the pier passes the landmark check from the lane, the shore and above (S33-e)`

#### C4 — The honesty scan

- **Files:** `tests/acceptance/tests/client_rules.rs`, this file.
- Validation: AC-e-5 (X-7 planted, red, restored).
- [ ] Implementation. [ ] Validation. [ ] Review: the scan does not flag the water shader or leaf sway.
- **Commit message:** `test(acceptance): no landmark prop or boat is animated (X-7; S33-e)`

#### C5 — Close-out

- Push; PR; CI on the exact head; repair; train-ready.
- **Commit message:** `docs(plan): S33-e1 close-out`

### S33-e2

#### C0 — Base, prerequisites, ledger

- S33-e1 and S29-a merged; S29-e's state (Q-S33e-1); the halt's passages as merged, and R-S29a-e-1's outcome (if
  the bay is still undrawable: **material stop**).
- **Commit message:** `docs(plan): S33-e2 — open the ledger at the base`

#### C1 — The halt

- **Files:** `scripts/worlds/lakeside/halt.gd`, `depiction.gd`, `clients/shared/station/station_state.gd` (only
  under Q-S33e-1's fallback), this file.
- Steps: §5.2.
- Validation: `--geometry` on the halt's three places; the board's text. Mutation **M-e-2** (the window drawn open)
  → the route's state check fails naming `station.window_closed`.
- [ ] Implementation. [ ] Validation. [ ] Review (Opus, frames VR-8 the halt from the lane, VR-9 the platform
  south to the buffer stop).
- **Commit message:** `feat(3d): Lakeside's halt — waiting room, shut window, platform and buffer stop (S33-e)`

#### C2 — The route to the platform

- **Files:** `depiction.gd` (`probe_route`), `slice_probe_world.gd` (only if the route needs the board check),
  this file.
- Validation: AC-e-3; AC-e-4 (X-3).
- **Commit message:** `test(3d): walk Lakeside from the pier head to the platform (S33-e)`

#### C3 — Close-out

- Push; PR; CI; repair; train-ready.
- **Commit message:** `docs(plan): S33-e2 close-out`

---

## 10. Requirements this PR places on other PRs

- **R-S29a-e-1 (on S29-a, through the coordinator).** The ticket bay north of the waiting room:
  `station-tickets` here (2 600, 400), `station` there (6 000, 4 600) (F-S33e-1).
- **R-S29d-e-1 (on S29-d).** The landmark check takes the hidden category as a parameter (Q-S33e-2).
- **R-S29e-e-1 (on S29-e).** The window bay, board, clock and bench row are functions of size, not of the depot
  (step-33 §16.2's note to S29-e, restated).
- **R-S33f-e-1 (on S33-f).** Sit points on the pier bench, the waiting-room bench row and the platform benches, at
  450 mm.

## 11. Edits to parent documents (owner: the primary session)

| Document | Edit |
| --- | --- |
| `pr-s29a-town-contract.md` | R-S29a-e-1 (the bay's points), if ruled |
| `step-33-lakeside-3d.md` §12.1 | S33-e split into S33-e1 and S33-e2 (Q-S33e-1) |
| `step-29-station-landmark.md` §9.2 | the check's hidden category is a parameter; LM-1's window for a landmark narrower than 20° is its own computed extent (Q-S33e-4, if ruled A) |

---

## 12. Execution contract

```text
PROJECT / PR:            MineWorld mvp0, S33-e1 (pier) and S33-e2 (halt)
PRIMARY DESIGN DOC:      .structured-coding/plans/mvp0/pr-s33e-pier-halt.md
RELATED / BINDING DOCS:  step-33-lakeside-3d.md; step-29-station-landmark.md; pr-s29a-town-contract.md;
                         pr-s33b-lakeside-bodies.md; pr-s33d-terrain-water-exterior.md; CLAUDE.md
WORKTREE:                /Users/yuema137/mineworld-worktrees/impl-s33e1, then impl-s33e2; one session each
BRANCH:                  mvp0/pr-s33e1, mvp0/pr-s33e2, from origin/main
START CONDITION:         per the header (NONE met at 2026-10-18)
APPROVED SCOPE:          §2.1 (S33-e1), §2.2 (S33-e2)
FROZEN INVARIANTS:       step-33 I-S33-1, -2, -3, -5, -7, -9; §6 I-e-1…I-e-6
SEQUENCE:                S33-e1 C0…C5; S33-e2 C0…C3 (§9)
ALLOWED COMMANDS:        as pr-s33d §12, plus mineworld validate on a planted copy under the scratchpad
NEVER:                   as pr-s33d §12; a train, a moving boat, an open window; editing worlds/
MATERIAL STOPS:          a prerequisite missing; the bay still undrawable after S29-a merges; LM-1 unattainable
                         without changing the pier's world size; a needed world or System change
PLATFORMS:               rendered checks on ubuntu-24.04, macos-26, windows-2025
ENDPOINT AUTHORITY:      implementation, commits, push, PR, CI repair: authorized once started;
                         merge: through a merge train under the charter's merge authority
STOP CONDITION:          each part train-ready
```

---

## 13. Risks

| Id | Risk | Mitigation |
| --- | --- | --- |
| R-e-1 | S29-d/S29-e are late (they wait for 16c and RL-b2) | the split (Q-S33e-1): the pier lands without them |
| R-e-2 | The pavilion does not close the southward view (LM-1 < 60 %) | §5.3's design-time estimate is ≈ 72 % on the extent window and ≈ 51 % on the fixed 20° window, so the window is an operator question now (Q-S33e-4), not a discovery at C3. Within the ruled window, the roof (ridge east–west, ridge height within reference 06's proportions) is the lever; a failure is reported with the measured share, never by re-sizing the window |
| R-e-3 | S29-a keeps the undrawable bay points | material stop at S33-e2 C0; the question goes up now (R-S29a-e-1) |
| R-e-4 | Q-S33d-1 ruled B (deck at lane level) | rails become a 1 067 mm guard and §5.1's ADA rail row changes; AC-e-6 follows |

---

## 14. Operator questions (each with a recommendation)

| Id | Question | Recommendation |
| --- | --- | --- |
| **Q-S33e-1 [operator — split]** | S29-e's station pieces wait behind S29-d, which waits for 16c and RL-b2. Split S33-e into S33-e1 (the pier, needs only S33-d) and S33-e2 (the halt, needs S29-e)? And if S29-e is still unmerged when S33-e2 is otherwise ready, may S33-e2 land `clients/shared/station/station_state.gd` itself, exactly per S29 §5.1, for S29-e to reuse (as S29-f is already allowed to)? | **Yes to both.** The landmark stops waiting on the depot, and the shared state rule is written once whoever is first. *Chinese:* 把 S33-e 拆成码头（先做）和小站（等 S29-e）；若 S29-e 还没合，小站可以先按 S29 §5.1 落共享的 station_state.gd——同意吗？ |
| **Q-S33e-2 [operator — the landmark check]** | The landmark check is S29-d's, not yet built. May whichever of S29-d and S33-e1 lands first write it, with the hidden category as a parameter (`station.*` for the towns, `landmark.*` for Lakeside)? | **Yes.** One checker for every world, no waiting. *Chinese:* 地标检查谁先做谁写，参数化隐藏类别，两个世界共用——同意吗？ |
| **Q-S33e-4 [operator — LM-1's window for a small landmark]** | Step-29's LM-1 uses a fixed 20° window, sized for the 52 m depot. Lakeside's landmark, a 4 m pavilion 20 m down the pier, spans only 14° from the lane, so even a solid pavilion could cover at most ≈ 70 % of a 20° window, and the real one covers ≈ 51 % (§5.3). (A) For a landmark narrower than the window, size the window to the landmark's own drawn extent, computed from its geometry (≈ 72 % expected); (B) keep 20° and drop LM-1 for Lakeside, keeping LM-2; (C) make the pavilion much larger than a pier-head shelter | **(A).** It still tests what LM-1 is for — that the landmark fills its part of the view rather than being lost against the far field — and it is computed, not tuned. (B) removes a check; (C) draws an unrealistic building. *Chinese:* 地标检查 LM-1 的 20° 视窗是按 52 米的车站定的；湖边码头亭子从小路看只占 14°，改成按地标自身的角宽度计算视窗——同意吗？ |
| **Q-S33e-3 [coordinator → S29-a — the ticket bay]** | S29-a's Lakeside bay points put the bay either over the water or on the platform (F-S33e-1). Move it north of the waiting room: `station-tickets` here (2 600, 400), `station` there (6 000, 4 600)? | **Yes**, in S29-a before its freeze. *Chinese:* 湖边小站的售票间按现在的坐标会落到水上或站台上，改到候车室北侧——同意吗？ |

### 14.1 Rulings

(none yet)

---

## 15. Ledger (live during implementation)

| Commit | SHA | Implementation | Validation | Review |
| --- | --- | --- | --- | --- |
| e1 C0…C5, e2 C0…C3 | — | [ ] | [ ] | [ ] |

Mutations: M-e-1, M-e-2, X-3, X-7, X-9 (none run yet). Findings: F-S33e-1…F-S33e-5 (§3.2).

**Design review (2026-10-18).** One independent Opus reviewer, verdict REQUEST CHANGES, no blockers; F-S33e-1's
arithmetic confirmed. Applied before the PR opened: the buffer stop at z 3.5; LM-1's window computed for a small
landmark (§5.3, Q-S33e-4, the ridge east–west); fishing positions on the sides only; the ramp's handrails and
landings (ADA §§405.7, 405.8, 505); the IBC guard reading stated as an approximation; 8 bents; the lantern on the
south-east post; the beached boat on the revetment, the missing slip recorded; the platform at y 0.00 with the rail
head at −0.20; the clock and village-scale notes attributed to step-29 §6.2 and step-33 §16.2; one honesty-scan
location.
